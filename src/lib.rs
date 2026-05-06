use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    str::FromStr,
};

use boa_engine::{JsValue, JsVariant, property::PropertyKey};
use html5ever::{
    QualName,
    driver::ParseOpts,
    parse_fragment, serialize,
    tendril::{StrTendril, TendrilSink},
};
use markup5ever_rcdom::{Handle, Node, NodeData, RcDom, SerializableHandle};
use serde::Serialize;
use serde_json::Value as JsonValue;

mod engine;
mod error;
mod interpolation;
mod template;
use engine::{Engine, ForBinding};
pub use error::{Directive, DirectiveErrorKind, Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContentAction {
    TraverseChildren,
    SkipChildren,
}

/// Render template with data
///
/// # Examples
///
/// ```
/// use prevue::render;
/// use serde_json::json;
///
/// let template = r#"<div v-if="show">{{ message }}</div>"#;
/// let data = json!({ "show": true, "message": "Hello" });
/// let result = render(template, data).unwrap();
/// assert!(result.contains("Hello"));
/// ```
pub fn render(template: impl AsRef<str>, data: impl Serialize) -> Result<String> {
    let dom = template::parse(template.as_ref())?;
    let mut engine = Engine::new(data)?;
    traverse(&Rc::clone(&dom.document), &mut engine)?;

    let mut buffer = Vec::new();
    serialize(
        &mut buffer,
        &SerializableHandle::from(Rc::clone(&dom.document)),
        Default::default(),
    )
    .map_err(|source| Error::SerializeHtml { source })?;

    let rendered = String::from_utf8(buffer).map_err(|source| Error::OutputUtf8 { source })?;
    Ok(rendered)
}

fn is_setup_script(handle: &Handle) -> bool {
    let NodeData::Element { name, attrs, .. } = &handle.data else {
        return false;
    };

    name.local.as_ref() == "script"
        && attrs.borrow().iter().any(|attr| {
            attr.name.local.as_ref() == "type" && attr.value.trim().eq_ignore_ascii_case("prevue")
        })
}

fn is_raw_text_element(handle: &Handle) -> bool {
    let NodeData::Element { name, .. } = &handle.data else {
        return false;
    };

    matches!(name.local.as_ref(), "script" | "style")
}

fn text_content(handle: &Handle) -> String {
    let mut text = String::new();
    for child in handle.children.borrow().iter() {
        if let NodeData::Text { contents } = &child.data {
            text.push_str(&contents.borrow());
        }
    }
    text
}

fn take_v_pre(handle: &Handle) -> bool {
    let NodeData::Element { attrs, .. } = &handle.data else {
        return false;
    };
    find_and_remove_directive(attrs, "v-pre").is_some()
}

// Traverse and process a node. Returns whether the node should stay in output.
fn traverse(handle: &Handle, engine: &mut Engine) -> Result<bool> {
    if take_v_pre(handle) {
        return Ok(true);
    }

    if is_setup_script(handle) {
        engine
            .eval_setup(&text_content(handle))
            .map_err(|err| Error::SetupScript {
                message: err.to_string(),
            })?;
        return Ok(false);
    }

    if process_node_content(handle, engine)? == ContentAction::SkipChildren {
        return Ok(true);
    }

    if is_raw_text_element(handle) {
        return Ok(true);
    }

    let children: Vec<Handle> = children_for_traversal(handle);

    let mut if_chain_active = false;
    let mut if_chain_matched = false;

    for node in children.iter() {
        if take_v_pre(node) {
            if_chain_active = false;
            if_chain_matched = false;
            continue;
        }

        let is_non_whitespace_text = is_non_whitespace_text_node(node);
        if is_non_whitespace_text {
            if_chain_active = false;
            if_chain_matched = false;
        }

        if !matches!(&node.data, NodeData::Element { .. }) && !is_non_whitespace_text {
            continue;
        }

        let processed =
            process_directives(node, engine, &mut if_chain_active, &mut if_chain_matched)?;

        if let Some(replacements) = processed {
            replace_node_in_parent(node, &replacements);
        } else if !traverse(node, engine)? {
            replace_node_in_parent(node, &[]);
        }
    }

    Ok(true)
}

// Process node content: v-bind, v-text, v-html, and mustache.
fn process_node_content(handle: &Handle, engine: &mut Engine) -> Result<ContentAction> {
    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let content_directives = {
                let attrs_ref = attrs.borrow();
                [
                    (
                        attrs_ref.iter().any(|a| a.name.local.as_ref() == "v-text"),
                        Directive::Text,
                    ),
                    (
                        attrs_ref.iter().any(|a| a.name.local.as_ref() == "v-html"),
                        Directive::Html,
                    ),
                ]
                .into_iter()
                .filter_map(|(has_directive, directive)| has_directive.then_some(directive))
                .collect::<Vec<_>>()
            };
            if content_directives.len() > 1 {
                return Err(Error::ConflictingDirectives {
                    directives: content_directives,
                });
            }

            let mut action = ContentAction::TraverseChildren;
            let mut renames: Vec<(usize, String, String)> = Vec::new();
            let mut removals: Vec<usize> = Vec::new();
            let mut additions: Vec<(String, QualName, String)> = Vec::new();

            for (i, attr) in attrs.borrow().iter().enumerate() {
                let name_ref: &str = attr.name.local.as_ref();

                if name_ref == "v-text" {
                    if let Some(value) = engine.eval_str(attr.value.as_ref()) {
                        replace_element_children(handle, vec![create_text_node(&value)]);
                        action = ContentAction::SkipChildren;
                    }
                    removals.push(i);
                    continue;
                }

                if name_ref == "v-html" {
                    if let Some(value) = engine.eval_str(attr.value.as_ref()) {
                        replace_element_children(handle, parse_html_fragment(name, &value));
                        action = ContentAction::SkipChildren;
                    }
                    removals.push(i);
                    continue;
                }

                // v-bind object spread: v-bind="obj" or v-bind="{ key: value }"
                if name_ref == "v-bind" {
                    if let Some(json_val) = eval_json(engine, attr.value.as_ref())
                        && let Some(obj) = json_val.as_object()
                    {
                        for (key, val) in obj.iter() {
                            let value = match key.as_str() {
                                "class" => normalize_class(val),
                                "style" => normalize_style(val),
                                _ if val.is_null() => None,
                                _ => Some(
                                    val.as_str()
                                        .map(str::to_string)
                                        .unwrap_or_else(|| val.to_string()),
                                ),
                            };
                            if let Some(value) = value {
                                additions.push((key.clone(), attr.name.clone(), value));
                            }
                        }
                        removals.push(i);
                    }
                    continue;
                }

                // v-bind argument syntax: :attr="value" or v-bind:attr="value"
                if let Some(arg_raw) = name_ref
                    .strip_prefix(':')
                    .or_else(|| name_ref.strip_prefix("v-bind:"))
                {
                    let value_expr = attr.value.trim();

                    if arg_raw.starts_with('[') && arg_raw.ends_with(']') {
                        if value_expr.is_empty() {
                            removals.push(i);
                            continue;
                        }
                        let inner = &arg_raw[1..arg_raw.len() - 1];
                        match (engine.eval_fmt(inner), engine.eval_fmt(value_expr)) {
                            (Some(resolved), Some(value)) => renames.push((i, resolved, value)),
                            _ => removals.push(i),
                        }
                    } else {
                        let target = if value_expr.is_empty() {
                            arg_raw
                        } else {
                            value_expr
                        };
                        if matches!(arg_raw, "class" | "style") {
                            let value = eval_json(engine, target).and_then(|val| match arg_raw {
                                "class" => normalize_class(&val),
                                "style" => normalize_style(&val),
                                _ => None,
                            });
                            if let Some(value) = value {
                                additions.push((arg_raw.to_string(), attr.name.clone(), value));
                            }
                            removals.push(i);
                        } else {
                            match engine.eval_fmt(target) {
                                Some(value) => renames.push((i, arg_raw.to_string(), value)),
                                None => removals.push(i),
                            }
                        }
                    }
                }
            }

            // Apply modifications
            let mut attrs_mut = attrs.borrow_mut();
            for (idx, new_name, new_value) in renames.iter().rev() {
                attrs_mut[*idx].name.local = html5ever::LocalName::from(new_name.as_str());
                attrs_mut[*idx].value = StrTendril::from_str(new_value.as_str()).unwrap();
            }
            for idx in removals.iter().rev() {
                attrs_mut.remove(*idx);
            }
            drop(attrs_mut);

            let mut attrs_mut = attrs.borrow_mut();
            for (local_name, template_qn, value) in additions.iter() {
                if let Some(existing) = attrs_mut
                    .iter_mut()
                    .find(|a| a.name.local.as_ref() == local_name.as_str())
                {
                    let merged = match local_name.as_str() {
                        "class" => merge_class(existing.value.as_ref(), value),
                        "style" => merge_style(existing.value.as_ref(), value),
                        _ => value.clone(),
                    };
                    existing.value = StrTendril::from_str(merged.as_str()).unwrap();
                } else {
                    attrs_mut.push(html5ever::Attribute {
                        name: QualName::new(
                            template_qn.prefix.clone(),
                            template_qn.ns.clone(),
                            html5ever::LocalName::from(local_name.as_str()),
                        ),
                        value: StrTendril::from_str(value.as_str()).unwrap(),
                    });
                }
            }
            Ok(action)
        }
        NodeData::Text { contents } => {
            let mut content = contents.borrow_mut();

            if let Some(rendered) = interpolation::render_text(&content, engine) {
                *content = StrTendril::from_str(&rendered).unwrap();
            }
            Ok(ContentAction::TraverseChildren)
        }
        _ => Ok(ContentAction::TraverseChildren),
    }
}

fn parse_html_fragment(context_name: &QualName, html: &str) -> Vec<Handle> {
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

    // Clone fragment nodes before attaching them to the main document. Moving
    // RcDom fragment handles directly can leave their subtree tied to the
    // temporary parser document.
    nodes
        .iter()
        .map(|node| {
            let cloned = clone_node(node);
            cloned.parent.take();
            cloned
        })
        .collect()
}

fn replace_element_children(handle: &Handle, new_children: Vec<Handle>) {
    for child in handle.children.borrow().iter() {
        child.parent.take();
    }

    for child in new_children.iter() {
        child.parent.set(Some(Rc::downgrade(handle)));
    }

    *handle.children.borrow_mut() = new_children;
}

fn eval_json(engine: &mut Engine, code: &str) -> Option<JsonValue> {
    engine
        .eval_expr(code)
        .ok()?
        .to_json(&mut engine.context)
        .ok()?
}

fn normalize_class(value: &JsonValue) -> Option<String> {
    fn collect(value: &JsonValue, classes: &mut Vec<String>) {
        match value {
            JsonValue::String(value) => {
                let value = value.trim();
                if !value.is_empty() {
                    classes.push(value.to_string());
                }
            }
            JsonValue::Array(values) => {
                for value in values {
                    collect(value, classes);
                }
            }
            JsonValue::Object(values) => {
                for (name, enabled) in values {
                    if json_truthy(enabled) {
                        classes.push(name.clone());
                    }
                }
            }
            _ => {}
        }
    }

    let mut classes = Vec::new();
    collect(value, &mut classes);
    (!classes.is_empty()).then(|| classes.join(" "))
}

fn normalize_style(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        }
        JsonValue::Array(values) => values
            .iter()
            .filter_map(normalize_style)
            .reduce(|merged, value| merge_style(&merged, &value)),
        JsonValue::Object(values) => {
            let styles = values
                .iter()
                .filter_map(|(name, value)| {
                    style_value(value)
                        .map(|value| format!("{}: {};", css_property_name(name), value))
                })
                .collect::<Vec<_>>();
            (!styles.is_empty()).then(|| styles.join(" "))
        }
        _ => None,
    }
}

fn style_value(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) if !value.is_empty() => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        JsonValue::Array(values) => values.iter().rev().find_map(style_value),
        _ => None,
    }
}

fn css_property_name(name: &str) -> String {
    if name.starts_with("--") {
        return name.to_string();
    }

    name.chars().fold(String::new(), |mut property, ch| {
        if ch.is_ascii_uppercase() {
            property.push('-');
            property.push(ch.to_ascii_lowercase());
        } else {
            property.push(ch);
        }
        property
    })
}

fn json_truthy(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => false,
        JsonValue::Bool(value) => *value,
        JsonValue::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
        JsonValue::String(value) => !value.is_empty(),
        JsonValue::Array(_) | JsonValue::Object(_) => true,
    }
}

fn merge_class(existing: &str, value: &str) -> String {
    match (existing.trim(), value.trim()) {
        ("", value) => value.to_string(),
        (existing, "") => existing.to_string(),
        (existing, value) => format!("{existing} {value}"),
    }
}

fn merge_style(existing: &str, value: &str) -> String {
    match (existing.trim(), value.trim()) {
        ("", value) => value.to_string(),
        (existing, "") => existing.to_string(),
        (existing, value) if existing.ends_with(';') => format!("{existing} {value}"),
        (existing, value) => format!("{existing}; {value}"),
    }
}

// Replace node with new_nodes in its parent's children
fn replace_node_in_parent(node: &Handle, new_nodes: &[Handle]) {
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

    // Check if previous sibling is whitespace indent
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
                        contents.replace(StrTendril::from_str(before_nl).unwrap());
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
        // Replacing node with new nodes
        children.remove(pos);
        for (i, new_node) in new_nodes.iter().enumerate() {
            new_node.parent.set(Some(Weak::clone(&node_parent_weak)));
            children.insert(pos + i, Rc::clone(new_node));
        }
    }
}

// Plain <template> contents are inert; structural directives expand them explicitly.
fn children_for_traversal(handle: &Handle) -> Vec<Handle> {
    if let NodeData::Element {
        name,
        template_contents,
        ..
    } = &handle.data
        && name.local.as_ref() == "template"
        && template_contents.borrow().is_some()
    {
        return Vec::new();
    }
    handle.children.borrow().iter().cloned().collect()
}

// Process directives on a node
// Returns None to keep node, Some(vec) to replace
fn process_directives(
    node: &Handle,
    engine: &mut Engine,
    if_chain_active: &mut bool,
    if_chain_matched: &mut bool,
) -> Result<Option<Vec<Handle>>> {
    let NodeData::Element { attrs, .. } = &node.data else {
        return Ok(None);
    };

    let directive_if = find_and_remove_directive(attrs, "v-if");
    let directive_else_if = find_and_remove_directive(attrs, "v-else-if");
    let directive_else = find_and_remove_directive(attrs, "v-else");
    let directive_for = find_and_remove_directive(attrs, "v-for");
    let invalid_directive = |directive, kind, expression| Error::InvalidDirective {
        directive,
        kind,
        expression,
    };
    let render_targets = |node: &Handle, engine: &mut Engine| -> Result<Vec<Handle>> {
        let mut rendered = Vec::new();
        let mut child_if_chain_active = false;
        let mut child_if_chain_matched = false;

        for target in expand_targets(node) {
            if take_v_pre(&target) {
                child_if_chain_active = false;
                child_if_chain_matched = false;
                rendered.push(target);
                continue;
            }

            if is_non_whitespace_text_node(&target) {
                child_if_chain_active = false;
                child_if_chain_matched = false;
            }

            let replacement = process_directives(
                &target,
                engine,
                &mut child_if_chain_active,
                &mut child_if_chain_matched,
            )?;

            if let Some(nodes) = replacement {
                rendered.extend(nodes);
            } else if traverse(&target, engine)? {
                rendered.push(target);
            }
        }
        Ok(rendered)
    };

    let branch_directives = [
        (directive_if.is_some(), Directive::If),
        (directive_else_if.is_some(), Directive::ElseIf),
        (directive_else.is_some(), Directive::Else),
    ]
    .into_iter()
    .filter_map(|(has_directive, directive)| has_directive.then_some(directive))
    .collect::<Vec<_>>();
    if branch_directives.len() > 1 {
        return Err(Error::ConflictingDirectives {
            directives: branch_directives,
        });
    }

    // v-if
    if let Some(expr) = directive_if {
        if expr.trim().is_empty() {
            return Err(invalid_directive(
                Directive::If,
                DirectiveErrorKind::MissingExpression,
                Some(expr),
            ));
        }
        *if_chain_active = true;
        *if_chain_matched = engine.eval_bool(&expr).unwrap_or(false);
        return Ok(Some(if *if_chain_matched {
            render_targets(node, engine)?
        } else {
            Vec::new()
        }));
    }

    // v-else-if
    if let Some(expr) = directive_else_if {
        if expr.trim().is_empty() {
            return Err(invalid_directive(
                Directive::ElseIf,
                DirectiveErrorKind::MissingExpression,
                Some(expr),
            ));
        }
        if !*if_chain_active {
            return Err(invalid_directive(
                Directive::ElseIf,
                DirectiveErrorKind::MissingAdjacentConditional,
                Some(expr),
            ));
        }
        if *if_chain_matched {
            return Ok(Some(Vec::new()));
        }
        *if_chain_matched = engine.eval_bool(&expr).unwrap_or(false);
        return Ok(Some(if *if_chain_matched {
            render_targets(node, engine)?
        } else {
            Vec::new()
        }));
    }

    // v-else
    if let Some(expr) = directive_else {
        if !expr.trim().is_empty() {
            return Err(invalid_directive(
                Directive::Else,
                DirectiveErrorKind::UnexpectedExpression,
                Some(expr),
            ));
        }
        if !*if_chain_active {
            return Err(invalid_directive(
                Directive::Else,
                DirectiveErrorKind::MissingAdjacentConditional,
                None,
            ));
        }
        *if_chain_active = false;
        return Ok(Some(if *if_chain_matched {
            Vec::new()
        } else {
            *if_chain_matched = true;
            render_targets(node, engine)?
        }));
    }

    *if_chain_active = false;

    // v-for
    Ok(match directive_for {
        Some(expr) => Some(process_for(node, engine, &expr)?),
        None => None,
    })
}

struct ForExpression {
    binding: ForBinding,
    iterable_expr: String,
}

// Process for directive
fn process_for(node: &Handle, engine: &mut Engine, expr: &str) -> Result<Vec<Handle>> {
    let expression = parse_for_expression(engine, expr).ok_or_else(|| Error::InvalidDirective {
        directive: Directive::For,
        kind: DirectiveErrorKind::InvalidExpression,
        expression: Some(expr.to_string()),
    })?;

    let indent_opt = get_indent(node);
    let mut result_nodes = Vec::new();
    let mut render_iteration = |engine: &mut Engine, slots: Vec<JsValue>| -> Result<()> {
        engine.enter_scope().map_err(|err| Error::Scope {
            message: err.to_string(),
        })?;

        let result = if engine.bind_for_slots(&expression.binding, slots) {
            process_for_iteration(node, engine, &indent_opt, &mut result_nodes)
        } else {
            Ok(())
        };

        engine.exit_scope();
        result
    };

    match engine
        .eval_expr(expression.iterable_expr.trim())
        .map(|val| val.variant())
    {
        Ok(JsVariant::Object(obj)) if obj.is_array() => {
            let Some(keys) = obj.own_property_keys(&mut engine.context).ok() else {
                return Ok(Vec::new());
            };

            for property_key in keys.iter() {
                let PropertyKey::Index(index) = property_key else {
                    continue;
                };

                let item = obj
                    .get(property_key.clone(), &mut engine.context)
                    .unwrap_or(JsValue::undefined());
                render_iteration(engine, vec![item, JsValue::new(index.get())])?;
            }
        }
        Ok(JsVariant::Object(obj)) => {
            let Some(property_keys) = obj.own_property_keys(&mut engine.context).ok() else {
                return Ok(Vec::new());
            };

            for (idx, property_key) in property_keys.iter().enumerate() {
                let value = obj
                    .get(property_key.clone(), &mut engine.context)
                    .unwrap_or(JsValue::undefined());
                render_iteration(
                    engine,
                    vec![value, property_key.into(), JsValue::new(idx as i32)],
                )?;
            }
        }
        Ok(JsVariant::Integer32(val)) => {
            for (idx, num) in (1..=val).enumerate() {
                render_iteration(engine, vec![JsValue::new(num), JsValue::new(idx)])?;
            }
        }
        Ok(JsVariant::String(val)) => {
            for (idx, ch) in val.to_std_string_escaped().chars().enumerate() {
                render_iteration(engine, vec![JsValue::new(ch), JsValue::new(idx)])?;
            }
        }
        _ => {}
    }

    Ok(result_nodes)
}

fn parse_for_expression(engine: &mut Engine, expr: &str) -> Option<ForExpression> {
    for (binding_raw, iterable_expr) in split_for_expressions(expr) {
        let binding_raw = binding_raw.trim();
        let binding_raw = if binding_raw.starts_with('(') && binding_raw.ends_with(')') {
            binding_raw[1..binding_raw.len() - 1].trim()
        } else {
            binding_raw
        };
        if let Some(binding) = engine.parse_for_binding(binding_raw) {
            return Some(ForExpression {
                binding,
                iterable_expr: iterable_expr.trim().to_string(),
            });
        }
    }

    None
}

fn split_for_expressions(expr: &str) -> impl Iterator<Item = (&str, &str)> {
    expr.char_indices().filter_map(|(idx, _)| {
        for keyword in ["in", "of"] {
            if is_keyword_at(expr, idx, keyword) {
                let alias = expr[..idx].trim();
                let iter = expr[idx + keyword.len()..].trim();
                if !alias.is_empty() && !iter.is_empty() {
                    return Some((alias, iter));
                }
            }
        }

        None
    })
}

fn is_keyword_at(input: &str, idx: usize, keyword: &str) -> bool {
    input[idx..].starts_with(keyword)
        && input[..idx]
            .chars()
            .next_back()
            .is_none_or(|ch| !is_identifier_continue(ch))
        && input[idx + keyword.len()..]
            .chars()
            .next()
            .is_none_or(|ch| !is_identifier_continue(ch))
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_alphanumeric()
}

// Process a single iteration of v-for
fn process_for_iteration(
    node: &Handle,
    engine: &mut Engine,
    indent_opt: &Option<String>,
    result_nodes: &mut Vec<Handle>,
) -> Result<()> {
    let targets = expand_targets(node);
    if targets.is_empty() {
        return Ok(());
    }

    let mut iteration_nodes = Vec::new();
    let push_node = |nodes: &mut Vec<Handle>, node: Handle, should_indent: bool| {
        if should_indent
            && !is_whitespace_text_node(&node)
            && let Some(indent) = indent_opt
        {
            nodes.push(create_text_node(indent));
        }
        nodes.push(node);
    };

    let mut child_if_chain_active = false;
    let mut child_if_chain_matched = false;

    for (target_idx, target) in targets.into_iter().enumerate() {
        if take_v_pre(&target) {
            child_if_chain_active = false;
            child_if_chain_matched = false;
            let should_indent = target_idx > 0 && !iteration_nodes.is_empty();
            push_node(&mut iteration_nodes, target, should_indent);
            continue;
        }

        if is_non_whitespace_text_node(&target) {
            child_if_chain_active = false;
            child_if_chain_matched = false;
        }

        let replacement = process_directives(
            &target,
            engine,
            &mut child_if_chain_active,
            &mut child_if_chain_matched,
        )?;

        match replacement {
            Some(new_nodes) => {
                for (idx, new_node) in new_nodes.into_iter().enumerate() {
                    let should_indent =
                        (target_idx > 0 && idx == 0 && !iteration_nodes.is_empty()) || idx > 0;
                    push_node(&mut iteration_nodes, new_node, should_indent);
                }
            }
            None => {
                if traverse(&target, engine)? {
                    let should_indent = target_idx > 0 && !iteration_nodes.is_empty();
                    push_node(&mut iteration_nodes, target, should_indent);
                }
            }
        }
    }

    if !iteration_nodes.is_empty() {
        if !result_nodes.is_empty()
            && let Some(indent) = indent_opt
        {
            result_nodes.push(create_text_node(indent));
        }
        result_nodes.extend(iteration_nodes);
    }

    Ok(())
}

fn expand_targets(node: &Handle) -> Vec<Handle> {
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

fn find_and_remove_directive(
    attrs: &RefCell<Vec<html5ever::Attribute>>,
    name: &str,
) -> Option<String> {
    let mut attrs_mut = attrs.borrow_mut();
    let pos = attrs_mut
        .iter()
        .position(|a| a.name.local.as_ref() == name)?;
    Some(attrs_mut.remove(pos).value.to_string())
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

fn get_indent(node: &Handle) -> Option<String> {
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

fn adjust_indent_in_subtree(node: &Handle, indent_adjustment: isize) {
    if let NodeData::Text { contents } = &node.data {
        let text = contents.borrow().to_string();
        let adjusted = adjust_text_indent(&text, indent_adjustment);
        contents.replace(StrTendril::from_str(&adjusted).unwrap());
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

fn is_non_whitespace_text_node(node: &Handle) -> bool {
    if let NodeData::Text { contents } = &node.data {
        contents.borrow().chars().any(|c| !c.is_whitespace())
    } else {
        false
    }
}
