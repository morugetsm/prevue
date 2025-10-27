mod engine;

use engine::Engine;
use std::{cell::RefCell, rc::Rc, str::FromStr, sync::LazyLock};

use boa_engine::{JsString, JsValue, JsVariant};
use html5ever::{
    serialize,
    tendril::{Tendril, TendrilSink},
};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};
use regex::Regex;
use serde::Serialize;

static SYNTAX_MUSTACHE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"\{\{\s*(.*?)\s*\}\}"#).unwrap());
static SYNTAX_FOR: LazyLock<Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"^\s*\(?\s*(?P<value>\p{XID_Start}\p{XID_Continue}*)(?:\s*,\s*(?P<key>\p{XID_Start}\p{XID_Continue}*))?(?:\s*,\s*(?P<index>\p{XID_Start}\p{XID_Continue}*))?\s*\)?\s+(?:of|in)\s+(?P<iter>.+?)\s*$"#).unwrap()
});
static SYNTAX_BIND_ARG: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"^(?:(?:v-bind:)|:)(?P<arg>\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_\-:]*)$"#).unwrap()
});

/// Renders the HTML template with the given data.
///
/// # Example
/// ```
/// # use prevue::render;
/// # use serde_json::json;
///
/// let html = r#"<div><p v-if="show">{{ text }}</p></div>"#.to_string();
/// let data = json!({"show": true, "text": "Hello"});
/// let rendered = render(html, data).unwrap();
/// ```
pub fn render(html: String, data: impl Serialize) -> Result<String, anyhow::Error> {
    let dom = html5ever::parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(&mut html.as_bytes())?;
    let mut engine = engine::Engine::new(data);

    traverse(&dom.document.clone(), &mut engine);

    let mut buff = Vec::new();
    let document: SerializableHandle = dom.document.clone().into();
    serialize(&mut buff, &document, Default::default())?;

    let rendered = String::from_utf8(buff)?;
    Ok(rendered)
}

fn traverse(handle: &Handle, engine: &mut Engine) {
    // hydrate
    match &handle.data {
        NodeData::Element { attrs, .. } => {
            // Collect v-bind operations first to avoid borrow conflicts.
            let mut renames: Vec<(usize, String, String)> = Vec::new();
            let mut removals: Vec<usize> = Vec::new();
            let mut additions: Vec<(String, html5ever::QualName, String)> = Vec::new();

            {
                let attrs_ro = attrs.borrow();
                for (i, attr) in attrs_ro.iter().enumerate() {
                    let name_ref: &str = attr.name.local.as_ref();

                    // v-bind object form: v-bind
                    if name_ref == "v-bind" {
                        let expr = attr.value.to_string();
                        if let Ok(js_val) = engine.eval(expr.as_str())
                            && let Ok(Some(json_val)) = js_val.to_json(&mut engine.context)
                            && let Some(obj) = json_val.as_object()
                        {
                            for (key, val) in obj.iter() {
                                // skip null
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

                    // v-bind with arg: :attr or v-bind:attr
                    if let Some(caps) = SYNTAX_BIND_ARG.captures(name_ref) {
                        let arg_raw = caps
                            .name("arg")
                            .map(|m| m.as_str().to_string())
                            .unwrap_or_default();

                        let value_expr_raw = attr.value.to_string();
                        let value_expr_trim = value_expr_raw.trim();

                        // dynamic arg shorthand without value => not supported, drop the attribute
                        if arg_raw.starts_with('[')
                            && arg_raw.ends_with(']')
                            && value_expr_trim.is_empty()
                        {
                            removals.push(i);
                            continue;
                        }

                        // resolve arg (evaluate inside [] if dynamic)
                        let arg = if arg_raw.starts_with('[')
                            && arg_raw.ends_with(']')
                            && arg_raw.len() >= 2
                        {
                            let inner = &arg_raw[1..arg_raw.len() - 1];
                            let Some(resolved) = engine.eval_str(inner) else {
                                removals.push(i);
                                continue;
                            };
                            if resolved.trim().is_empty() {
                                removals.push(i);
                                continue;
                            }
                            resolved
                        } else {
                            arg_raw
                        };

                        // shorthand without value => evaluate by arg name itself
                        let value_expr = if value_expr_trim.is_empty() {
                            arg.clone()
                        } else {
                            value_expr_raw
                        };

                        // evaluate value using eval_str-only; drop on nullish markers
                        if let Some(value_str) = engine.eval_str(value_expr.as_str()) {
                            // Plan to rename this attribute in-place and set its value
                            renames.push((i, arg, value_str));
                        } else {
                            // evaluation failed: drop attribute for safety
                            removals.push(i);
                        }
                        continue;
                    }
                }
            }

            // Apply renames (in-place)
            if !renames.is_empty() {
                let mut attrs_mut = attrs.borrow_mut();
                for (idx, new_local, new_value) in renames.into_iter() {
                    if let Some(attr) = attrs_mut.get_mut(idx) {
                        attr.name.local = html5ever::LocalName::from(new_local.as_str());
                        attr.value = Tendril::from_str(new_value.as_str()).unwrap();
                    }
                }
            }

            // Apply removals (from the end)
            if !removals.is_empty() {
                let mut attrs_mut = attrs.borrow_mut();
                removals.sort_unstable();
                for idx in removals.into_iter().rev() {
                    let _ = attrs_mut.remove(idx);
                }
            }

            // Apply additions
            if !additions.is_empty() {
                let mut attrs_mut = attrs.borrow_mut();
                for (local_name, template_qn, value) in additions.into_iter() {
                    // If attribute already exists, update; else push
                    if let Some(existing) = attrs_mut
                        .iter_mut()
                        .find(|a| a.name.local.as_ref() == local_name.as_str())
                    {
                        existing.value = Tendril::from_str(value.as_str()).unwrap();
                    } else {
                        let qn = html5ever::QualName::new(
                            template_qn.prefix,
                            template_qn.ns,
                            html5ever::LocalName::from(local_name.as_str()),
                        );
                        attrs_mut.push(html5ever::Attribute {
                            name: qn,
                            value: Tendril::from_str(value.as_str()).unwrap(),
                        });
                    }
                }
            }
        }
        NodeData::Text { contents } => {
            let mut content = contents.borrow_mut();

            let replacements: Vec<(std::ops::Range<usize>, String)> = SYNTAX_MUSTACHE
                .captures_iter(&content)
                .map(|capture| {
                    (
                        capture.get(0).unwrap().range(),
                        capture.get(1).unwrap().as_str().to_owned(),
                    )
                })
                .collect();

            for (range, key) in replacements.into_iter().rev() {
                let evaluated = engine.eval_str(key.as_str()).unwrap_or_default();
                let mut text_value = content.to_string();
                text_value.replace_range(range, &evaluated);
                *content = Tendril::from_str(&text_value).unwrap();
            }
        }
        _ => (),
    }

    let snapshot: Vec<Handle> = handle.children.borrow().iter().cloned().collect();

    for node in snapshot.iter() {
        match &node.data {
            NodeData::Element {
                attrs,
                template_contents,
                ..
            } => {
                // <template>
                if node.children.borrow().is_empty()
                    && let Some(template_content) = template_contents.take()
                {
                    let template_children: Vec<Handle> =
                        template_content.children.borrow().iter().cloned().collect();
                    for template_child in template_children.iter() {
                        let child = deep_clone_subtree(template_child);
                        child.parent.replace(Some(Rc::downgrade(node)));
                        node.children.borrow_mut().push(child);
                    }
                }

                // v-if
                if let Some(attr_value) = find_and_remove_directive(attrs, "v-if")
                    && !engine.eval_bool(&attr_value).unwrap_or(false)
                {
                    remove_leading_whitespace_text(node);
                    remove_node(node);
                    continue;
                }

                // v-for
                if let Some(attr_value) = find_and_remove_directive(attrs, "v-for") {
                    let Some(syntax) = SYNTAX_FOR.captures(attr_value.as_str()) else {
                        remove_node(node);
                        continue;
                    };
                    let Some(value_iden) = syntax.name("value") else {
                        remove_node(node);
                        continue;
                    };
                    let key_iden_opt = syntax.name("key");
                    let index_iden_opt = syntax.name("index");
                    let Some(iter_iden_opt) = syntax.name("iter") else {
                        remove_node(node);
                        continue;
                    };

                    let mut anchor = node.clone();
                    // Extract only the whitespace after the last newline for repetition
                    let separator_opt = leading_whitespace_text(node)
                        .and_then(|ws| ws.rfind('\n').map(|idx| ws[idx..].to_string()));

                    // Closure for common rendering logic (clone, traverse, insert)
                    let mut render_iteration = |engine: &mut Engine, index: usize, total: usize| {
                        let child = deep_clone_subtree(node);
                        traverse(&child, engine);

                        insert_after(&anchor, &child);
                        anchor = child;

                        if index + 1 < total
                            && let Some(separator) = &separator_opt
                        {
                            let separator_node = make_text_node(separator);
                            insert_after(&anchor, &separator_node);
                            anchor = separator_node;
                        }
                    };

                    match engine.eval(iter_iden_opt.as_str()).map(|val| val.variant()) {
                        Ok(JsVariant::Object(obj)) if obj.is_array() => {
                            let total = obj
                                .get(JsString::from("length"), &mut engine.context)
                                .ok()
                                .and_then(|v| v.to_u32(&mut engine.context).ok())
                                .unwrap_or(0) as usize;

                            for index in 0..total {
                                if engine.enter_scope().is_err() {
                                    continue;
                                }

                                let item = obj
                                    .get(JsString::from(index.to_string()), &mut engine.context)
                                    .unwrap_or(JsValue::undefined());

                                // Array: bind (item, index) only
                                engine.set_val(value_iden.as_str(), item);
                                if let Some(key_iden) = key_iden_opt {
                                    engine.set_val(key_iden.as_str(), JsValue::new(index as i32));
                                }

                                render_iteration(engine, index, total);
                                engine.exit_scope();
                            }
                        }
                        Ok(JsVariant::Object(obj)) => {
                            let keys_arr = engine
                                .eval(format!("Object.keys(({}))", iter_iden_opt.as_str()).as_str())
                                .ok();
                            if let Some(JsVariant::Object(keys_obj)) =
                                keys_arr.map(|val| val.variant())
                            {
                                let total = keys_obj
                                    .get(JsString::from("length"), &mut engine.context)
                                    .ok()
                                    .and_then(|v| v.to_u32(&mut engine.context).ok())
                                    .unwrap_or(0)
                                    as usize;

                                for index in 0..total {
                                    if engine.enter_scope().is_err() {
                                        continue;
                                    }

                                    let key_val = keys_obj
                                        .get(JsString::from(index.to_string()), &mut engine.context)
                                        .unwrap_or(JsValue::undefined());
                                    let key_jsstr = key_val
                                        .to_string(&mut engine.context)
                                        .unwrap_or_else(|_| JsString::from(""));
                                    let value = obj
                                        .get(key_jsstr.clone(), &mut engine.context)
                                        .unwrap_or(JsValue::undefined());

                                    // Object: bind (value, key, index)
                                    engine.set_val(value_iden.as_str(), value);
                                    if let Some(key_iden) = key_iden_opt {
                                        engine.set_val(key_iden.as_str(), JsValue::from(key_jsstr));
                                    }
                                    if let Some(index_iden) = index_iden_opt {
                                        engine.set_val(
                                            index_iden.as_str(),
                                            JsValue::new(index as i32),
                                        );
                                    }

                                    render_iteration(engine, index, total);
                                    engine.exit_scope();
                                }
                            }
                        }
                        _ => (),
                    }
                    remove_node(node);
                    continue;
                }

                traverse(node, engine);
            }
            NodeData::Text { .. } => {
                traverse(node, engine);
            }
            _ => (),
        }
    }

    let snapshot: Vec<Handle> = handle.children.borrow().iter().cloned().collect();
    for node in snapshot.iter() {
        if let NodeData::Element { name, .. } = &node.data
            && name.local.as_ref() == "template"
        {
            let snapshot_children: Vec<Handle> = node.children.borrow().iter().cloned().collect();
            if !snapshot_children.is_empty() {
                // Get the leading whitespace before this template (for proper indentation)
                let separator_opt = leading_whitespace_text(node);

                // Filter out all whitespace-only text nodes
                let trimmed_children: Vec<Handle> = snapshot_children
                    .iter()
                    .filter(|child| !is_whitespace_text_node(child))
                    .cloned()
                    .collect();

                if !trimmed_children.is_empty() {
                    let mut clones: Vec<Handle> =
                        trimmed_children.iter().map(deep_clone_subtree).collect();
                    if let Some(common_indent) = compute_common_leading_indent(&trimmed_children) {
                        adjust_leading_indent_inplace(&mut clones, &common_indent);
                    }

                    // Insert clones with separators before each element
                    let mut anchor = node.clone();
                    for child in clones.iter() {
                        // Add separator before each non-whitespace element
                        if !is_whitespace_text_node(child)
                            && let Some(separator) = &separator_opt
                        {
                            let separator_node = make_text_node(separator);
                            insert_after(&anchor, &separator_node);
                            anchor = separator_node;
                        }
                        insert_after(&anchor, child);
                        anchor = child.clone();
                    }
                }
            }
            remove_leading_whitespace_text(node);
            remove_node(node);
        }
    }
}

fn find_and_remove_directive(
    attrs: &RefCell<Vec<html5ever::Attribute>>,
    directive: &str,
) -> Option<String> {
    let attr_idx = attrs
        .borrow()
        .iter()
        .position(|attr| &attr.name.local == directive)?;
    let attr = attrs.borrow_mut().remove(attr_idx);
    Some(attr.value.to_string())
}

fn insert_after(original: &Handle, new_sibling: &Handle) {
    let Some(parent) = original.parent.take() else {
        return;
    };
    original.parent.set(Some(parent.clone()));

    let Some(parent_rc) = parent.upgrade() else {
        return;
    };

    let mut children = parent_rc.children.borrow_mut();
    let pos = children
        .iter()
        .position(|c| Rc::ptr_eq(c, original))
        .map(|i| i + 1)
        .unwrap_or(children.len());

    new_sibling.parent.set(Some(parent.clone()));
    children.insert(pos, new_sibling.clone());
}

fn remove_node(node: &Handle) {
    let parent_weak_opt = node.parent.take();
    if let Some(parent_rc) = parent_weak_opt.and_then(|w| w.upgrade()) {
        let mut children = parent_rc.children.borrow_mut();
        if let Some(pos) = children.iter().position(|c| Rc::ptr_eq(c, node)) {
            children.remove(pos);
        }
    }
    node.parent.replace(None);
}

fn deep_clone_subtree(node: &Handle) -> Handle {
    let clone_children = |source: &Handle, parent: &Handle| {
        for child in source.children.borrow().iter() {
            let cloned_child = deep_clone_subtree(child);
            cloned_child.parent.replace(Some(Rc::downgrade(parent)));
            parent.children.borrow_mut().push(cloned_child);
        }
    };

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
        NodeData::ProcessingInstruction { target, contents } => {
            Node::new(NodeData::ProcessingInstruction {
                target: target.clone(),
                contents: contents.clone(),
            })
        }
        NodeData::Element {
            name,
            attrs,
            template_contents,
            mathml_annotation_xml_integration_point,
        } => {
            // Clone template_contents if present
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
    }
}

fn compute_common_leading_indent(nodes: &[Handle]) -> Option<String> {
    let mut common: Option<String> = None;
    for n in nodes.iter() {
        if let NodeData::Text { contents } = &n.data {
            let s = contents.borrow();
            if let Some((_, rest)) = s.split_once('\n') {
                let indent: String = rest
                    .chars()
                    .take_while(|c| *c == ' ' || *c == '\t')
                    .collect();
                if indent.is_empty() {
                    continue;
                }
                match &mut common {
                    Some(c) => {
                        let mut new_len = 0usize;
                        for (a, b) in c.chars().zip(indent.chars()) {
                            if a == b {
                                new_len += 1;
                            } else {
                                break;
                            }
                        }
                        c.truncate(new_len);
                        if c.is_empty() {
                            return None;
                        }
                    }
                    None => common = Some(indent),
                }
            }
        }
    }
    common
}

fn adjust_leading_indent_inplace(nodes: &mut [Handle], remove_indent: &str) {
    if remove_indent.is_empty() {
        return;
    }
    for n in nodes.iter() {
        if let NodeData::Text { contents } = &n.data {
            let mut s = contents.borrow().to_string();
            if let Some(idx) = s.find('\n') {
                let after_nl = idx + 1;
                if s[after_nl..].starts_with(remove_indent) {
                    s.replace_range(after_nl..after_nl + remove_indent.len(), "");
                    *contents.borrow_mut() = html5ever::tendril::StrTendril::from_slice(&s);
                }
            }
        }
    }
}

fn leading_whitespace_text(node: &Handle) -> Option<String> {
    let parent_weak = node.parent.take()?;
    node.parent.set(Some(parent_weak.clone()));
    let parent = parent_weak.upgrade()?;
    let children = parent.children.borrow();
    let idx = children.iter().position(|c| Rc::ptr_eq(c, node))?;
    if idx == 0 {
        return None;
    }
    let before = idx - 1;
    if is_whitespace_text_node(&children[before])
        && let NodeData::Text { contents } = &children[before].data
    {
        return Some(contents.borrow().to_string());
    }
    None
}

fn make_text_node(text: &str) -> Handle {
    Node::new(NodeData::Text {
        contents: RefCell::new(html5ever::tendril::StrTendril::from_slice(text)),
    })
}

fn remove_leading_whitespace_text(node: &Handle) {
    let Some(parent_weak) = node.parent.take() else {
        return;
    };
    node.parent.set(Some(parent_weak.clone()));
    if let Some(parent) = parent_weak.upgrade() {
        let mut children = parent.children.borrow_mut();
        if let Some(idx) = children.iter().position(|c| Rc::ptr_eq(c, node))
            && idx > 0
            && is_whitespace_text_node(&children[idx - 1])
        {
            children.remove(idx - 1);
        }
    }
}

fn is_whitespace_text_node(node: &Handle) -> bool {
    if let NodeData::Text { contents } = &node.data {
        let s = contents.borrow();
        s.chars().all(|c| c.is_whitespace())
    } else {
        false
    }
}
