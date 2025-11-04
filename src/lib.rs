use boa_engine::{JsValue, JsVariant, property::PropertyKey};
use html5ever::{
    QualName,
    driver::ParseOpts,
    parse_document, serialize,
    tendril::{StrTendril, TendrilSink},
};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};
use regex::Regex;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::LazyLock;

mod engine;
use engine::Engine;

static SYNTAX_MUSTACHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{\s*(.+?)\s*\}\}").unwrap());
static SYNTAX_BIND_ARG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(?:(?:v-bind:)|:)(?P<arg>\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_\-:]*)$"#).unwrap()
});
static SYNTAX_FOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^\s*(?<val>\w+)\s*(?:,\s*(?<key>\w+)\s*(?:,\s*(?<idx>\w+)\s*)?)?\s+in\s+(?<iter>.+)\s*$",
    )
    .unwrap()
});

/// Render HTML template with data
///
/// # Examples
///
/// ```
/// use prevue::render;
/// use serde_json::json;
///
/// let html = r#"<div v-if="show">{{ message }}</div>"#;
/// let data = json!({ "show": true, "message": "Hello" });
/// let result = render(html.to_string(), data).unwrap();
/// assert!(result.contains("Hello"));
/// ```
pub fn render(html: String, data: impl Serialize) -> Result<String, anyhow::Error> {
    let json_value = serde_json::to_value(data)?;
    let mut engine = Engine::new(json_value);
    let dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())?;

    traverse(&dom.document.clone(), &mut engine);

    let mut buff = Vec::new();
    serialize(
        &mut buff,
        &SerializableHandle::from(dom.document.clone()),
        Default::default(),
    )?;

    let rendered = String::from_utf8(buff)?;
    Ok(rendered)
}

// Traverse and process a node
fn traverse(handle: &Handle, engine: &mut Engine) {
    hydrate_node(handle, engine);

    let children_source = get_children_source(handle);
    let snapshot: Vec<Handle> = children_source.to_vec();

    let mut in_if_chain = false;
    let mut if_chain_hit = false;

    for node in snapshot.iter() {
        if let NodeData::Element { attrs, .. } = &node.data
            && find_and_remove_directive(attrs, "v-pre").is_some()
        {
            continue;
        }

        hydrate_node(node, engine);

        let replacement = process_directives(node, engine, &mut in_if_chain, &mut if_chain_hit);

        match replacement {
            Some(new_nodes) => {
                replace_in_children_source(node, &new_nodes);
                for new_node in new_nodes.iter() {
                    traverse(new_node, engine);
                }
            }
            None => {
                traverse(node, engine);
            }
        }
    }
}

// Replace node with new_nodes in its parent's children
fn replace_in_children_source(node: &Handle, new_nodes: &[Handle]) {
    let Some(node_parent_weak) = node.parent.take() else {
        node.parent.set(None);
        return;
    };
    node.parent.set(Some(node_parent_weak.clone()));

    let Some(node_parent) = node_parent_weak.upgrade() else {
        return;
    };

    let mut children = node_parent.children.borrow_mut();
    if let Some(pos) = children.iter().position(|c| Rc::ptr_eq(c, node)) {
        let has_leading_ws = if pos > 0 {
            if let NodeData::Text { contents } = &children[pos - 1].data {
                let text = contents.borrow().to_string();
                if text.chars().all(|c| c.is_whitespace()) {
                    true
                } else if let Some(last_nl) = text.rfind('\n') {
                    let after_nl = &text[last_nl + 1..];
                    !after_nl.is_empty() && after_nl.chars().all(|c| c.is_whitespace())
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if new_nodes.is_empty() {
            if has_leading_ws {
                if let NodeData::Text { contents } = &children[pos - 1].data {
                    let text = contents.borrow().to_string();
                    if text.chars().all(|c| c.is_whitespace()) {
                        children.remove(pos - 1);
                        children.remove(pos - 1);
                    } else if let Some(last_nl) = text.rfind('\n') {
                        let trimmed = &text[..=last_nl];
                        contents.replace(StrTendril::from_str(trimmed).unwrap());
                        children.remove(pos);
                    } else {
                        children.remove(pos);
                    }
                } else {
                    children.remove(pos);
                }
            } else {
                children.remove(pos);
            }
        } else {
            children.remove(pos);
            for (i, new_node) in new_nodes.iter().enumerate() {
                new_node.parent.set(Some(node_parent_weak.clone()));
                children.insert(pos + i, new_node.clone());
            }
        }
    }
}

// Get children source: template_contents for <template> with directives, else children
fn get_children_source(handle: &Handle) -> Vec<Handle> {
    if let NodeData::Element {
        template_contents, ..
    } = &handle.data
        && let Some(tc) = template_contents.borrow().as_ref()
    {
        return tc.children.borrow().iter().cloned().collect();
    }
    handle.children.borrow().iter().cloned().collect()
}

// Hydrate node: process v-bind and mustache
fn hydrate_node(handle: &Handle, engine: &mut Engine) {
    match &handle.data {
        NodeData::Element { attrs, .. } => {
            let mut renames: Vec<(usize, String, String)> = Vec::new();
            let mut removals: Vec<usize> = Vec::new();
            let mut additions: Vec<(String, QualName, String)> = Vec::new();

            for (i, attr) in attrs.borrow().iter().enumerate() {
                let name_ref: &str = attr.name.local.as_ref();

                if name_ref == "v-bind" {
                    let expr = attr.value.to_string();
                    if let Ok(js_val) = engine.eval(expr.as_str())
                        && let Ok(Some(json_val)) = js_val.to_json(&mut engine.context)
                        && let Some(obj) = json_val.as_object()
                    {
                        for (key, val) in obj.iter() {
                            if val.is_null() {
                                continue;
                            }
                            let value_str = if val.is_string() {
                                val.as_str().unwrap_or("").to_string()
                            } else {
                                val.to_string()
                            };
                            additions.push((key.clone(), attr.name.clone(), value_str));
                        }
                        removals.push(i);
                    }
                    continue;
                }

                if let Some(caps) = SYNTAX_BIND_ARG.captures(name_ref) {
                    let arg_raw = caps
                        .name("arg")
                        .map(|m| m.as_str().to_string())
                        .unwrap_or_default();

                    let value_expr_raw = attr.value.to_string();
                    let value_expr_trim = value_expr_raw.trim();

                    if arg_raw.starts_with('[')
                        && arg_raw.ends_with(']')
                        && value_expr_trim.is_empty()
                    {
                        removals.push(i);
                        continue;
                    }

                    let arg =
                        if arg_raw.starts_with('[') && arg_raw.ends_with(']') && arg_raw.len() >= 2
                        {
                            let inner = &arg_raw[1..arg_raw.len() - 1];
                            let Some(resolved) = engine.eval_str(inner) else {
                                removals.push(i);
                                continue;
                            };
                            resolved
                        } else {
                            arg_raw
                        };

                    let value_opt = if value_expr_trim.is_empty() {
                        engine.eval_str(&arg)
                    } else {
                        engine.eval_str(value_expr_trim)
                    };

                    let Some(value) = value_opt else {
                        removals.push(i);
                        continue;
                    };

                    renames.push((i, arg, value));
                    continue;
                }
            }

            let mut attrs_mut = attrs.borrow_mut();
            for (idx, new_name, new_value) in renames.iter().rev() {
                attrs_mut[*idx].name.local = html5ever::LocalName::from(new_name.as_str());
                attrs_mut[*idx].value = StrTendril::from_str(new_value.as_str()).unwrap();
            }
            for idx in removals.iter().rev() {
                attrs_mut.remove(*idx);
            }
            drop(attrs_mut);

            for (local_name, template_qn, value) in additions.iter() {
                let mut attrs_mut = attrs.borrow_mut();
                if let Some(existing) = attrs_mut
                    .iter_mut()
                    .find(|a| a.name.local.as_ref() == local_name.as_str())
                {
                    existing.value = StrTendril::from_str(value.as_str()).unwrap();
                } else {
                    let qn = QualName::new(
                        template_qn.prefix.clone(),
                        template_qn.ns.clone(),
                        html5ever::LocalName::from(local_name.as_str()),
                    );
                    attrs_mut.push(html5ever::Attribute {
                        name: qn,
                        value: StrTendril::from_str(value.as_str()).unwrap(),
                    });
                }
            }
        }
        NodeData::Text { contents } => {
            let mut content = contents.borrow_mut();
            let replacements: Vec<(std::ops::Range<usize>, String)> = SYNTAX_MUSTACHE
                .captures_iter(&content)
                .filter_map(|capture| {
                    let range = capture.get(0)?.range();
                    let expr = capture.get(1)?.as_str();
                    Some((range, engine.eval_str(expr).unwrap_or_default()))
                })
                .collect();

            for (range, evaluated) in replacements.iter().rev() {
                let mut text_value = content.to_string();
                text_value.replace_range(range.clone(), evaluated);
                *content = StrTendril::from_str(&text_value).unwrap();
            }
        }
        _ => (),
    }
}

// Process directives on a node
// Returns None to keep node, Some(vec) to replace
fn process_directives(
    node: &Handle,
    engine: &mut Engine,
    in_if_chain: &mut bool,
    if_chain_hit: &mut bool,
) -> Option<Vec<Handle>> {
    let NodeData::Element { attrs, .. } = &node.data else {
        return None;
    };

    let directive_if = find_and_remove_directive(attrs, "v-if");
    let directive_elif = find_and_remove_directive(attrs, "v-else-if");
    let directive_else = find_and_remove_directive(attrs, "v-else");
    let directive_for = find_and_remove_directive(attrs, "v-for");

    // if
    if let Some(expr) = directive_if {
        *in_if_chain = true;
        if engine.eval_bool(&expr).unwrap_or(false) {
            *if_chain_hit = true;
            return Some(expand_targets(node));
        } else {
            *if_chain_hit = false;
            return Some(Vec::new());
        }
    }

    // else-if
    if let Some(expr) = directive_elif {
        if !*in_if_chain {
            return None;
        }

        if *if_chain_hit {
            return Some(Vec::new());
        }
        if engine.eval_bool(&expr).unwrap_or(false) {
            *if_chain_hit = true;
            return Some(expand_targets(node));
        } else {
            *if_chain_hit = false;
            return Some(Vec::new());
        }
    }

    // else
    if directive_else.is_some() {
        if !*in_if_chain {
            return None;
        }

        *in_if_chain = false;
        if *if_chain_hit {
            return Some(Vec::new());
        }
        *if_chain_hit = true;
        return Some(expand_targets(node));
    }

    *in_if_chain = false;

    // for
    if let Some(expr) = directive_for {
        return Some(process_for(node, engine, &expr));
    }

    None
}

// Process for directive
fn process_for(node: &Handle, engine: &mut Engine, expr: &str) -> Vec<Handle> {
    let mut result_nodes = Vec::new();

    let Some(syntax) = SYNTAX_FOR.captures(expr) else {
        return result_nodes;
    };
    let Some(val_iden) = syntax.name("val") else {
        return result_nodes;
    };
    let Some(iter_iden) = syntax.name("iter") else {
        return result_nodes;
    };

    let separator_rest = get_indent(node);

    let iter_expr = iter_iden.as_str().trim();
    let iter_wrapped = if iter_expr.starts_with('{') {
        format!("({})", iter_expr)
    } else {
        iter_expr.to_string()
    };

    match engine.eval(iter_wrapped.as_str()).map(|val| val.variant()) {
        Ok(JsVariant::Object(obj)) if obj.is_array() => {
            let Ok(keys) = obj.own_property_keys(&mut engine.context) else {
                return result_nodes;
            };

            for property_key in keys.iter() {
                let PropertyKey::Index(index) = property_key else {
                    continue;
                };
                if engine.enter_scope().is_err() {
                    continue;
                }

                let item = obj
                    .get(property_key.clone(), &mut engine.context)
                    .unwrap_or(JsValue::undefined());
                engine.set_val(val_iden.as_str(), item);

                if let Some(key_iden) = syntax.name("key") {
                    engine.set_val(key_iden.as_str(), JsValue::new(index.get()));
                }

                let targets = expand_targets(node);
                if targets.is_empty() {
                    engine.exit_scope();
                    continue;
                }

                let mut iteration_nodes = Vec::new();
                for (target_idx, target) in targets.into_iter().enumerate() {
                    let mut dummy_in_chain = false;
                    let mut dummy_hit = false;
                    let replacement =
                        process_directives(&target, engine, &mut dummy_in_chain, &mut dummy_hit);

                    match replacement {
                        Some(new_nodes) => {
                            for (idx, new_node) in new_nodes.iter().enumerate() {
                                if ((target_idx > 0 && idx == 0 && !iteration_nodes.is_empty())
                                    || idx > 0)
                                    && !is_whitespace_text_node(new_node)
                                    && let Some(sep) = &separator_rest
                                {
                                    iteration_nodes.push(create_text_node(sep));
                                }
                                traverse(new_node, engine);
                                iteration_nodes.push(new_node.clone());
                            }
                        }
                        None => {
                            if target_idx > 0
                                && !iteration_nodes.is_empty()
                                && let Some(sep) = &separator_rest
                            {
                                iteration_nodes.push(create_text_node(sep));
                            }
                            traverse(&target, engine);
                            iteration_nodes.push(target);
                        }
                    }
                }

                if !iteration_nodes.is_empty() {
                    if !result_nodes.is_empty()
                        && let Some(sep) = &separator_rest
                    {
                        result_nodes.push(create_text_node(sep));
                    }
                    result_nodes.extend(iteration_nodes);
                }

                engine.exit_scope();
            }
        }
        Ok(JsVariant::Object(obj)) => {
            let Ok(property_keys) = obj.own_property_keys(&mut engine.context) else {
                return result_nodes;
            };

            for (idx, property_key) in property_keys.iter().enumerate() {
                if engine.enter_scope().is_err() {
                    continue;
                }

                let value = obj
                    .get(property_key.clone(), &mut engine.context)
                    .unwrap_or(JsValue::undefined());
                engine.set_val(val_iden.as_str(), value);

                if let Some(key_iden) = syntax.name("key") {
                    engine.set_val(key_iden.as_str(), property_key.into());
                }
                if let Some(idx_iden) = syntax.name("idx") {
                    engine.set_val(idx_iden.as_str(), JsValue::new(idx as i32));
                }

                let targets = expand_targets(node);
                if targets.is_empty() {
                    engine.exit_scope();
                    continue;
                }

                let mut iteration_nodes = Vec::new();
                for (target_idx, target) in targets.into_iter().enumerate() {
                    let mut dummy_in_chain = false;
                    let mut dummy_hit = false;
                    let replacement =
                        process_directives(&target, engine, &mut dummy_in_chain, &mut dummy_hit);

                    match replacement {
                        Some(new_nodes) => {
                            for (idx, new_node) in new_nodes.iter().enumerate() {
                                if ((target_idx > 0 && idx == 0 && !iteration_nodes.is_empty())
                                    || idx > 0)
                                    && !is_whitespace_text_node(new_node)
                                    && let Some(sep) = &separator_rest
                                {
                                    iteration_nodes.push(create_text_node(sep));
                                }
                                traverse(new_node, engine);
                                iteration_nodes.push(new_node.clone());
                            }
                        }
                        None => {
                            if target_idx > 0
                                && !iteration_nodes.is_empty()
                                && let Some(sep) = &separator_rest
                            {
                                iteration_nodes.push(create_text_node(sep));
                            }
                            traverse(&target, engine);
                            iteration_nodes.push(target);
                        }
                    }
                }

                if !iteration_nodes.is_empty() {
                    if !result_nodes.is_empty()
                        && let Some(sep) = &separator_rest
                    {
                        result_nodes.push(create_text_node(sep));
                    }
                    result_nodes.extend(iteration_nodes);
                }

                engine.exit_scope();
            }
        }
        _ => {}
    }

    result_nodes
}

fn expand_targets(node: &Handle) -> Vec<Handle> {
    if let NodeData::Element {
        name,
        template_contents,
        ..
    } = &node.data
        && name.local.as_ref() == "template"
    {
        if let Some(tc) = template_contents.borrow().as_ref() {
            let template_indent = get_indent(node)
                .map(|s| s.chars().filter(|c| *c == ' ').count())
                .unwrap_or(0);

            let first_child_indent = tc
                .children
                .borrow()
                .iter()
                .find(|c| !is_whitespace_text_node(c))
                .and_then(get_indent)
                .map(|s| s.chars().filter(|c| *c == ' ').count())
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
        } else {
            return Vec::new();
        }
    }
    let cloned = clone_node(node);
    cloned.parent.take();
    vec![cloned]
}

fn find_and_remove_directive(
    attrs: &RefCell<Vec<html5ever::Attribute>>,
    name: &str,
) -> Option<String> {
    let mut attrs_mut = attrs.borrow_mut();
    if let Some(pos) = attrs_mut.iter().position(|a| a.name.local.as_ref() == name) {
        let attr = attrs_mut.remove(pos);
        Some(attr.value.to_string())
    } else {
        None
    }
}

fn clone_node(node: &Handle) -> Handle {
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
            let cloned_template_contents =
                if let Some(template_content) = template_contents.borrow().as_ref() {
                    let template_clone = Node::new(NodeData::Document);
                    clone_children(template_content, &template_clone);
                    Some(template_clone)
                } else {
                    None
                };

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

fn get_indent(node: &Handle) -> Option<String> {
    let parent_weak = node.parent.take()?;
    node.parent.set(Some(parent_weak.clone()));
    let parent = parent_weak.upgrade()?;

    let children = parent.children.borrow();
    let pos = children.iter().position(|c| Rc::ptr_eq(c, node))?;

    if pos == 0 {
        return None;
    }

    if let NodeData::Text { contents } = &children[pos - 1].data {
        let text = contents.borrow().to_string();
        if let Some(last_nl) = text.rfind('\n') {
            return Some(
                text[last_nl..]
                    .chars()
                    .map(|char| if char == '\n' { '\n' } else { ' ' })
                    .collect::<String>(),
            );
        }
    }
    None
}

fn adjust_indent_in_subtree(node: &Handle, indent_adjustment: isize) {
    if let NodeData::Text { contents } = &node.data {
        let text = contents.borrow().to_string();
        let adjusted = adjust_text_indent(&text, indent_adjustment);
        contents.replace(StrTendril::from_str(&adjusted).unwrap());
    }

    for child in node.children.borrow().iter() {
        adjust_indent_in_subtree(child, indent_adjustment);
    }

    if let NodeData::Element {
        template_contents, ..
    } = &node.data
        && let Some(tc) = template_contents.borrow().as_ref()
    {
        for child in tc.children.borrow().iter() {
            adjust_indent_in_subtree(child, indent_adjustment);
        }
    }
}

fn adjust_text_indent(text: &str, adjustment: isize) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i == 0 {
                line.to_string()
            } else {
                let spaces = line.chars().take_while(|c| *c == ' ').count();
                let new_spaces = (spaces as isize + adjustment).max(0) as usize;
                let rest = line.trim_start();
                format!("{}{}", " ".repeat(new_spaces), rest)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn create_text_node(text: &str) -> Handle {
    Node::new(NodeData::Text {
        contents: RefCell::new(StrTendril::from_str(text).unwrap()),
    })
}

fn is_whitespace_text_node(node: &Handle) -> bool {
    if let NodeData::Text { contents } = &node.data {
        contents.borrow().chars().all(|c| c.is_whitespace())
    } else {
        false
    }
}
