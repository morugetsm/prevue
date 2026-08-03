use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use html5ever::{
    QualName,
    driver::ParseOpts,
    ns, parse_fragment,
    tendril::{StrTendril, TendrilSink},
};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom};

/// A `<template>` holds its content in a separate document that the serializer
/// never walks, so move it into place once rendering is done.
pub(crate) fn hoist_template_contents(handle: &Handle) {
    let contents = match &handle.data {
        NodeData::Element {
            template_contents, ..
        } => template_contents.borrow_mut().take(),
        _ => None,
    };

    if let Some(contents) = contents {
        let hoisted = std::mem::take(&mut *contents.children.borrow_mut());
        for child in hoisted.iter() {
            child.parent.set(Some(Rc::downgrade(handle)));
        }
        handle.children.borrow_mut().extend(hoisted);
    }

    for child in handle.children.borrow().iter() {
        hoist_template_contents(child);
    }
}

/// HTML forbids children here, so content given to one is dropped by the parser
/// or escapes the element on serialization.
pub(crate) fn is_void_element(name: &QualName) -> bool {
    name.ns == ns!(html)
        && matches!(
            name.local.as_ref(),
            "area"
                | "base"
                | "basefont"
                | "bgsound"
                | "br"
                | "col"
                | "embed"
                | "frame"
                | "hr"
                | "img"
                | "input"
                | "keygen"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
}

pub(crate) fn is_raw_text_element(handle: &Handle) -> bool {
    let NodeData::Element { name, .. } = &handle.data else {
        return false;
    };

    matches!(name.local.as_ref(), "script" | "style")
}

pub(crate) fn text_content(handle: &Handle) -> String {
    let mut text = String::new();
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { contents } = &child.data {
            text.push_str(&contents.borrow());
        }
    }
    text
}

// Plain <template> contents are inert; structural directives expand them explicitly.
pub(crate) fn is_inert_template(handle: &Handle) -> bool {
    matches!(
        &handle.data,
        NodeData::Element { name, template_contents, .. }
            if name.local.as_ref() == "template" && template_contents.borrow().is_some()
    )
}

pub(crate) fn parse_html_fragment(context_name: &QualName, html: &str) -> Vec<Handle> {
    let dom = parse_fragment(
        RcDom::default(),
        ParseOpts::default(),
        context_name.clone(),
        Vec::new(),
        false,
    )
    .one(html);

    let root_nodes = dom
        .document
        .children
        .borrow()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let nodes = match root_nodes.as_slice() {
        [root]
            if matches!(
                &root.data,
                NodeData::Element { name, .. } if name.local.as_ref() == "html"
            ) =>
        {
            root.children.borrow().iter().cloned().collect::<Vec<_>>()
        }
        _ => root_nodes,
    };

    // Clone before attaching: moving RcDom fragment handles directly can leave
    // their subtree tied to the temporary parser document.
    nodes
        .iter()
        .map(|node| {
            let cloned = clone_node(node);
            cloned.parent.take();
            cloned
        })
        .collect()
}

pub(crate) fn replace_element_children(handle: &Handle, new_children: Vec<Handle>) {
    for child in handle.children.borrow().iter() {
        child.parent.take();
    }

    for child in new_children.iter() {
        child.parent.set(Some(Rc::downgrade(handle)));
    }

    *handle.children.borrow_mut() = new_children;
}

// An empty `new_nodes` also takes the whitespace that indented the node, so the
// surrounding lines keep their shape.
pub(crate) fn replace_node_in_parent(node: &Handle, new_nodes: &[Handle]) {
    let Some(node_parent_weak) = node.parent.take() else {
        return;
    };
    node.parent.set(Some(Weak::clone(&node_parent_weak)));
    let Some(node_parent) = node_parent_weak.upgrade() else {
        return;
    };

    let mut children = node_parent.children.borrow_mut();
    let Some(pos) = children.iter().position(|c| Rc::ptr_eq(c, node)) else {
        return;
    };

    let has_indent_before = pos > 0 && {
        if let NodeData::Text { contents } = &children[pos - 1].data {
            let text = contents.borrow();
            text.chars().all(|c| c.is_whitespace())
                || text
                    .rfind('\n')
                    .is_some_and(|nl| text[nl + 1..].chars().all(|c| c.is_whitespace()))
        } else {
            false
        }
    };

    if new_nodes.is_empty() {
        if has_indent_before {
            if let NodeData::Text { contents } = &children[pos - 1].data {
                let text = contents.borrow().to_string();
                if let Some(nl) = text.rfind('\n') {
                    let before_nl = &text[..nl];
                    if before_nl.is_empty() {
                        children.remove(pos - 1);
                        children.remove(pos - 1);
                    } else {
                        contents.replace(StrTendril::from(before_nl));
                        children.remove(pos);
                    }
                } else if text.chars().all(|c| c.is_whitespace()) {
                    children.remove(pos - 1);
                    children.remove(pos - 1);
                } else {
                    children.remove(pos);
                }
            }
        } else {
            children.remove(pos);
        }
    } else {
        children.remove(pos);
        for (i, new_node) in new_nodes.iter().enumerate() {
            new_node.parent.set(Some(Weak::clone(&node_parent_weak)));
            children.insert(pos + i, Rc::clone(new_node));
        }
    }
}

pub(crate) fn expand_targets(node: &Handle) -> Vec<Handle> {
    if let NodeData::Element {
        template_contents, ..
    } = &node.data
        && let Some(tc) = template_contents.borrow().as_ref()
    {
        let count_spaces = |s: &String| s.chars().filter(|c| *c == ' ').count();
        let template_indent = get_indent(node).as_ref().map(count_spaces).unwrap_or(0);
        let first_child_indent = tc
            .children
            .borrow()
            .iter()
            .find(|c| !is_whitespace_text_node(c))
            .and_then(get_indent)
            .as_ref()
            .map(count_spaces)
            .unwrap_or(0);

        let indent_adjustment = template_indent as isize - first_child_indent as isize;

        return tc
            .children
            .borrow()
            .iter()
            .filter(|c| !is_whitespace_text_node(c))
            .map(|c| {
                let cloned = clone_node(c);
                cloned.parent.take();
                if indent_adjustment != 0 {
                    adjust_indent_in_subtree(&cloned, indent_adjustment);
                }
                cloned
            })
            .collect();
    }

    let cloned = clone_node(node);
    cloned.parent.take();
    vec![cloned]
}

pub(crate) fn attribute_value(
    attrs: &RefCell<Vec<html5ever::Attribute>>,
    name: &str,
) -> Option<String> {
    attrs
        .borrow()
        .iter()
        .find(|attr| attr.name.local.as_ref() == name)
        .map(|attr| attr.value.to_string())
}

pub(crate) fn take_attribute(
    attrs: &RefCell<Vec<html5ever::Attribute>>,
    name: &str,
) -> Option<String> {
    let mut attrs_mut = attrs.borrow_mut();
    let pos = attrs_mut
        .iter()
        .position(|a| a.name.local.as_ref() == name)?;
    Some(attrs_mut.remove(pos).value.to_string())
}

pub(crate) fn clone_node(node: &Handle) -> Handle {
    fn clone_children(from: &Handle, to: &Handle) {
        for child in from.children.borrow().iter() {
            let cloned_child = clone_node(child);
            cloned_child.parent.set(Some(Rc::downgrade(to)));
            to.children.borrow_mut().push(cloned_child);
        }
    }

    match &node.data {
        NodeData::Document => {
            let cloned = Node::new(NodeData::Document);
            clone_children(node, &cloned);
            cloned
        }
        NodeData::Doctype {
            name,
            public_id,
            system_id,
        } => Node::new(NodeData::Doctype {
            name: name.clone(),
            public_id: public_id.clone(),
            system_id: system_id.clone(),
        }),
        NodeData::Text { contents } => Node::new(NodeData::Text {
            contents: RefCell::new(contents.borrow().clone()),
        }),
        NodeData::Comment { contents } => Node::new(NodeData::Comment {
            contents: contents.clone(),
        }),
        NodeData::Element {
            name,
            attrs,
            template_contents,
            mathml_annotation_xml_integration_point,
        } => {
            let cloned_template_contents = template_contents.borrow().as_ref().map(|tc| {
                let clone = Node::new(NodeData::Document);
                clone_children(tc, &clone);
                clone
            });

            let cloned = Node::new(NodeData::Element {
                name: name.clone(),
                attrs: RefCell::new(attrs.borrow().clone()),
                template_contents: RefCell::new(cloned_template_contents),
                mathml_annotation_xml_integration_point: *mathml_annotation_xml_integration_point,
            });
            clone_children(node, &cloned);
            cloned
        }
        NodeData::ProcessingInstruction { target, contents } => {
            Node::new(NodeData::ProcessingInstruction {
                target: target.clone(),
                contents: contents.clone(),
            })
        }
    }
}

pub(crate) fn create_text_node(text: &str) -> Handle {
    Node::new(NodeData::Text {
        contents: RefCell::new(StrTendril::from(text)),
    })
}

pub(crate) fn is_element(node: &Handle) -> bool {
    matches!(&node.data, NodeData::Element { .. })
}

pub(crate) fn is_whitespace_text_node(node: &Handle) -> bool {
    if let NodeData::Text { contents } = &node.data {
        contents.borrow().chars().all(|c| c.is_whitespace())
    } else {
        false
    }
}

pub(crate) fn is_non_whitespace_text_node(node: &Handle) -> bool {
    if let NodeData::Text { contents } = &node.data {
        contents.borrow().chars().any(|c| !c.is_whitespace())
    } else {
        false
    }
}

/// The whitespace a node sits behind, so a rendered copy can be placed at the
/// same depth as the original.
pub(crate) fn get_indent(node: &Handle) -> Option<String> {
    let parent_weak = node.parent.take()?;
    node.parent.set(Some(Weak::clone(&parent_weak)));
    let parent = parent_weak.upgrade()?;

    let children = parent.children.borrow();
    let pos = children.iter().position(|c| Rc::ptr_eq(c, node))?;

    if pos == 0 {
        return None;
    }

    if let NodeData::Text { contents } = &children[pos - 1].data {
        let text = contents.borrow();
        if let Some(last_nl) = text.rfind('\n') {
            let indent_text = &text[last_nl..];
            return Some(
                indent_text
                    .chars()
                    .map(|c| if c == '\n' { '\n' } else { ' ' })
                    .collect(),
            );
        }
    }
    None
}

pub(crate) fn adjust_indent_in_subtree(node: &Handle, indent_adjustment: isize) {
    if let NodeData::Text { contents } = &node.data {
        let text = contents.borrow().to_string();
        let adjusted = adjust_text_indent(&text, indent_adjustment);
        contents.replace(StrTendril::from(adjusted.as_str()));
    }

    if let NodeData::Element {
        template_contents, ..
    } = &node.data
        && let Some(tc) = template_contents.borrow().as_ref()
    {
        for child in tc.children.borrow().iter() {
            adjust_indent_in_subtree(child, indent_adjustment);
        }
    } else {
        for child in node.children.borrow().iter() {
            adjust_indent_in_subtree(child, indent_adjustment);
        }
    }
}

fn adjust_text_indent(text: &str, adjustment: isize) -> String {
    if adjustment == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i == 0 {
            result.push_str(line);
        } else {
            result.push('\n');

            let spaces = line.chars().take_while(|c| *c == ' ').count();
            let new_spaces = (spaces as isize + adjustment).max(0) as usize;
            let rest = &line[spaces..];
            result.push_str(&" ".repeat(new_spaces));
            result.push_str(rest);
        }
    }
    result
}
