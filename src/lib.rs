use std::rc::Rc;

use boa_engine::{JsValue, JsVariant};
use html5ever::{serialize, tendril::StrTendril};
use markup5ever_rcdom::{Handle, NodeData, SerializableHandle};
use serde::Serialize;

mod attr_value;
mod dom;
mod engine;
mod error;
mod indent;
mod interpolation;
mod template;
use attr_value::{AttrEdits, apply_modifiers, normalize_bound_attribute, validate_attribute_name};
use dom::{
    clone_node, create_text_node, expand_targets, is_element, is_inert_template,
    is_non_whitespace_text_node, is_raw_text_element, is_whitespace_text_node, parse_html_fragment,
    replace_element_children, replace_node_in_parent, take_attribute, text_content,
};
use engine::{Engine, ForBinding};
pub use error::{Directive, DirectiveErrorKind, Error, Result};
use indent::get_indent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Walk {
    Children,
    Done,
}

/// Position in a `v-if` / `v-else-if` / `v-else` run of adjacent siblings.
/// While `Closed`, a `v-else-if` or `v-else` is an orphan.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum IfChain {
    #[default]
    Closed,
    Open,
    Matched,
}

impl IfChain {
    fn reset(&mut self) {
        *self = Self::Closed;
    }

    fn is_open(self) -> bool {
        self != Self::Closed
    }

    fn has_matched(self) -> bool {
        self == Self::Matched
    }

    fn set_matched(&mut self, matched: bool) {
        *self = if matched { Self::Matched } else { Self::Open };
    }
}

/// A reusable renderer that keeps its JavaScript context alive across renders.
///
/// Building that context is the dominant cost of a small render, so reusing one
/// `Renderer` is several times faster than calling [`render`] repeatedly.
/// Compiled template expressions are cached on it too, so re-rendering the same
/// template never re-parses them. The HTML itself is still parsed per call —
/// pair the renderer with a [`Template`] to skip that as well.
///
/// The context is shared, which has two consequences worth knowing:
///
/// - A `Renderer` is **not** `Send`, because the underlying engine is not. Use
///   one per thread, or a pool.
/// - Render data is replaced on every call, but JavaScript globals a template
///   creates on purpose — `var` inside `{{ }}`, an undeclared assignment, a
///   write to `globalThis`, a mutated built-in — outlive the render. Setup
///   scripts do not leak; their declarations are scoped.
///
/// # Examples
///
/// ```
/// use prevue::Renderer;
/// use serde_json::json;
///
/// let mut renderer = Renderer::new().unwrap();
/// let template = "<p>{{ name }}</p>";
///
/// let first = renderer.render(template, json!({ "name": "Ada" })).unwrap();
/// let second = renderer.render(template, json!({ "name": "Grace" })).unwrap();
///
/// assert!(first.contains("Ada"));
/// assert!(second.contains("Grace"));
/// ```
pub struct Renderer {
    engine: Engine,
}

impl Renderer {
    /// Build a renderer, paying the JavaScript context setup cost once.
    pub fn new() -> Result<Self> {
        Ok(Self {
            engine: Engine::new()?,
        })
    }

    /// Render `source` with `data`, replacing the data from any prior render.
    pub fn render(&mut self, source: impl AsRef<str>, data: impl Serialize) -> Result<String> {
        let dom = template::parse(source.as_ref());
        self.render_document(&dom.document, data)
    }

    /// Render an already parsed [`Template`], skipping the parse this time.
    pub fn render_template(&mut self, template: &Template, data: impl Serialize) -> Result<String> {
        // Rendering rewrites the tree, so it works on a copy.
        let document = clone_node(&template.document);
        self.render_document(&document, data)
    }

    fn render_document(&mut self, document: &Handle, data: impl Serialize) -> Result<String> {
        self.engine.install_data(data)?;
        traverse(document, &mut self.engine)?;

        let mut buffer = Vec::new();
        serialize(
            &mut buffer,
            &SerializableHandle::from(Rc::clone(document)),
            Default::default(),
        )
        .map_err(|source| Error::RenderOutput {
            message: format!("failed to serialize HTML: {source}"),
        })?;

        String::from_utf8(buffer).map_err(|source| Error::RenderOutput {
            message: format!("failed to convert rendered HTML to UTF-8: {source}"),
        })
    }
}

/// A parsed template that can be rendered repeatedly without re-parsing.
///
/// Parsing dominates a small render, so pairing this with a reused [`Renderer`]
/// is about twice as fast as passing the source string every time. Loop-heavy
/// templates gain little, since evaluation dominates them instead.
///
/// Like `Renderer`, a `Template` is **not** `Send`; build one per thread.
/// Cloning is cheap and shares the parsed tree.
///
/// # Examples
///
/// ```
/// use prevue::{Renderer, Template};
/// use serde_json::json;
///
/// let mut renderer = Renderer::new().unwrap();
/// let template = Template::new("<p>{{ name }}</p>");
///
/// let first = renderer.render_template(&template, json!({ "name": "Ada" })).unwrap();
/// let second = renderer.render_template(&template, json!({ "name": "Grace" })).unwrap();
///
/// assert!(first.contains("Ada"));
/// assert!(second.contains("Grace"));
/// ```
#[derive(Clone)]
pub struct Template {
    document: Handle,
}

impl Template {
    /// Parse `source`, applying the same error recovery a browser would.
    pub fn new(source: impl AsRef<str>) -> Self {
        Self {
            document: template::parse(source.as_ref()).document,
        }
    }
}

impl std::fmt::Debug for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Template").finish_non_exhaustive()
    }
}

/// Render template with data
///
/// Each call builds a fresh JavaScript context. To render repeatedly, use
/// [`Renderer`] instead and pay that cost once.
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
    Renderer::new()?.render(template, data)
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

fn take_v_pre(handle: &Handle) -> bool {
    let NodeData::Element { attrs, .. } = &handle.data else {
        return false;
    };
    take_attribute(attrs, "v-pre").is_some()
}

// Returns whether the node should stay in the output.
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

    if render_content(handle, engine)? == Walk::Done {
        return Ok(true);
    }

    if is_raw_text_element(handle) {
        return Ok(true);
    }
    if handle.children.borrow().is_empty() || is_inert_template(handle) {
        return Ok(true);
    }

    // Only element children can be dropped or expanded, so without them the
    // child list cannot change and interpolating in place is enough.
    if !handle.children.borrow().iter().any(is_element) {
        for child in handle.children.borrow().iter() {
            if is_non_whitespace_text_node(child) {
                render_content(child, engine)?;
            }
        }
        return Ok(true);
    }

    let children: Vec<Handle> = handle.children.borrow().iter().cloned().collect();

    // The nodes are already in the tree, so `Keep` needs no work here.
    walk_siblings(children, engine, |node, placement| {
        match placement {
            Placement::Keep => {}
            Placement::Drop => replace_node_in_parent(&node, &[]),
            Placement::Replace(nodes) => replace_node_in_parent(&node, &nodes),
        }
        Ok(())
    })?;

    Ok(true)
}

/// What the structural directives decided about one node of a sibling sequence.
enum Placement {
    Keep,
    Drop,
    Replace(Vec<Handle>),
}

/// Apply `v-pre` and the structural directives across siblings under one shared
/// [`IfChain`]. Callers differ only in where `place` puts the results.
fn walk_siblings(
    nodes: impl IntoIterator<Item = Handle>,
    engine: &mut Engine,
    mut place: impl FnMut(Handle, Placement) -> Result<()>,
) -> Result<()> {
    let mut if_chain = IfChain::default();

    for node in nodes {
        if take_v_pre(&node) {
            if_chain.reset();
            place(node, Placement::Keep)?;
            continue;
        }

        let is_text = is_non_whitespace_text_node(&node);
        if is_text {
            if_chain.reset();
        }

        // Comments and whitespace carry no directives and leave the chain alone.
        if !is_text && !is_element(&node) {
            place(node, Placement::Keep)?;
            continue;
        }

        match apply_directives(&node, engine, &mut if_chain)? {
            Some(replacements) => place(node, Placement::Replace(replacements))?,
            None if traverse(&node, engine)? => place(node, Placement::Keep)?,
            None => place(node, Placement::Drop)?,
        }
    }

    Ok(())
}

// Render v-bind, v-text, v-html, and mustache on the current node.
fn render_content(handle: &Handle, engine: &mut Engine) -> Result<Walk> {
    match &handle.data {
        NodeData::Element { name, attrs, .. } => {
            let (has_v_text, has_v_html) = {
                let attrs_ref = attrs.borrow();
                if attrs_ref.is_empty() {
                    return Ok(Walk::Children);
                }
                let mut has_v_text = false;
                let mut has_v_html = false;
                for attr in attrs_ref.iter() {
                    match attr.name.local.as_ref() {
                        "v-text" => has_v_text = true,
                        "v-html" => has_v_html = true,
                        _ => {}
                    }
                }
                (has_v_text, has_v_html)
            };
            if has_v_text && has_v_html {
                return Err(Error::ConflictingDirectives {
                    directives: vec![Directive::Text, Directive::Html],
                });
            }

            let mut action = Walk::Children;
            let mut edits = AttrEdits::default();
            let mut bound: Vec<(usize, String)> = Vec::new();
            let attrs_ref = attrs.borrow();

            for (i, attr) in attrs_ref.iter().enumerate() {
                let name_ref: &str = attr.name.local.as_ref();

                if name_ref == "v-text" {
                    if let Some(value) = engine.eval_str(attr.value.as_ref()) {
                        replace_element_children(handle, vec![create_text_node(&value)]);
                        action = Walk::Done;
                    }
                    edits.remove(i);
                    continue;
                }

                if name_ref == "v-html" {
                    if let Some(value) = engine.eval_str(attr.value.as_ref()) {
                        replace_element_children(handle, parse_html_fragment(name, &value));
                        action = Walk::Done;
                    }
                    edits.remove(i);
                    continue;
                }

                // v-bind object spread: v-bind="obj" or v-bind="{ key: value }"
                if name_ref == "v-bind" {
                    if let Ok(bound) = engine.eval_expr(attr.value.as_ref())
                        && let JsVariant::Object(obj) = bound.variant()
                    {
                        for key in engine.object_keys(bound.clone()) {
                            let Some(key_string) = key.as_string() else {
                                continue;
                            };
                            let key = key_string.to_std_string_escaped();
                            let value = engine.get_prop(&obj, key_string);
                            let value = normalize_bound_attribute(engine, &key, &value);
                            if let Some(value) = value {
                                validate_attribute_name(&key)?;
                                edits.add(key, attr.name.clone(), value);
                            }
                        }
                        edits.remove(i);
                    }
                    continue;
                }

                // v-bind argument syntax: :attr="value" or v-bind:attr="value"
                if let Some(arg_raw) = name_ref
                    .strip_prefix(':')
                    .or_else(|| name_ref.strip_prefix("v-bind:"))
                {
                    let value_expr = attr.value.trim();
                    let (arg, modifiers) = split_modifiers(arg_raw);

                    let name = match dynamic_arg(arg) {
                        // A dynamic name has no expression to fall back on.
                        Some(_) if value_expr.is_empty() => {
                            edits.remove(i);
                            continue;
                        }
                        Some(inner) => match engine.eval_str(inner) {
                            Some(name) if !name.is_empty() => name,
                            _ => {
                                edits.remove(i);
                                continue;
                            }
                        },
                        None => arg.to_string(),
                    };
                    let name = apply_modifiers(name, modifiers)?;

                    let target = if value_expr.is_empty() {
                        arg
                    } else {
                        value_expr
                    };

                    let value = engine
                        .eval_expr(target)
                        .ok()
                        .and_then(|value| normalize_bound_attribute(engine, &name, &value));
                    match (matches!(name.as_str(), "class" | "style"), value) {
                        (true, Some(value)) => {
                            edits.add(name, attr.name.clone(), value);
                            edits.remove(i);
                        }
                        (false, Some(value)) => {
                            validate_attribute_name(&name)?;
                            shadow_duplicates(&attrs_ref, &bound, i, &name, &mut edits);
                            bound.push((i, name.clone()));
                            edits.set(i, name, value);
                        }
                        _ => edits.remove(i),
                    }
                }
            }

            drop(attrs_ref);
            edits.apply(attrs);
            Ok(action)
        }
        NodeData::Text { contents } => {
            let mut content = contents.borrow_mut();

            if let Some(rendered) = interpolation::render_text(&content, engine) {
                *content = StrTendril::from(rendered.as_str());
            }
            Ok(Walk::Children)
        }
        _ => Ok(Walk::Children),
    }
}

/// Split `view-box.camel`, skipping a dynamic argument's brackets first.
fn split_modifiers(arg: &str) -> (&str, &str) {
    let from = if arg.starts_with('[') {
        arg.find(']').map_or(arg.len(), |end| end + 1)
    } else {
        0
    };

    match arg[from..].find('.') {
        Some(offset) => arg.split_at(from + offset),
        None => (arg, ""),
    }
}

/// The expression inside `:[key]`, or `None` for a literal argument.
fn dynamic_arg(arg: &str) -> Option<&str> {
    arg.strip_prefix('[')?.strip_suffix(']')
}

/// Leave a binding as the only source for its name. `bound` holds only earlier
/// bindings, so two of them cannot delete each other and lose both values.
fn shadow_duplicates(
    attrs: &[html5ever::Attribute],
    bound: &[(usize, String)],
    binding: usize,
    name: &str,
    edits: &mut AttrEdits,
) {
    for (idx, attr) in attrs.iter().enumerate() {
        let local: &str = attr.name.local.as_ref();
        let is_binding =
            local.starts_with(':') || local == "v-bind" || local.starts_with("v-bind:");

        if idx != binding && local == name && !is_binding {
            edits.remove(idx);
        }
    }

    for (idx, resolved) in bound {
        if resolved == name {
            edits.remove(*idx);
        }
    }
}

fn render_targets(node: &Handle, engine: &mut Engine) -> Result<Vec<Handle>> {
    let mut rendered = Vec::new();

    walk_siblings(expand_targets(node), engine, |node, placement| {
        match placement {
            Placement::Keep => rendered.push(node),
            Placement::Drop => {}
            Placement::Replace(nodes) => rendered.extend(nodes),
        }
        Ok(())
    })?;

    Ok(rendered)
}

// Returns `None` to keep the node, `Some(nodes)` to replace it.
fn apply_directives(
    node: &Handle,
    engine: &mut Engine,
    if_chain: &mut IfChain,
) -> Result<Option<Vec<Handle>>> {
    let NodeData::Element { attrs, .. } = &node.data else {
        return Ok(None);
    };

    if attrs.borrow().is_empty() {
        if_chain.reset();
        return Ok(None);
    }

    let directive_if = take_attribute(attrs, "v-if");
    let directive_else_if = take_attribute(attrs, "v-else-if");
    let directive_else = take_attribute(attrs, "v-else");
    let directive_for = take_attribute(attrs, "v-for");
    let invalid_directive = |directive, kind, expression| Error::InvalidDirective {
        directive,
        kind,
        expression,
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

    if let Some(expr) = directive_if {
        if expr.trim().is_empty() {
            return Err(invalid_directive(
                Directive::If,
                DirectiveErrorKind::MissingExpression,
                Some(expr),
            ));
        }
        let matched = engine.eval_bool(&expr).unwrap_or(false);
        if_chain.set_matched(matched);
        return Ok(Some(if matched {
            render_targets(node, engine)?
        } else {
            Vec::new()
        }));
    }

    if let Some(expr) = directive_else_if {
        if expr.trim().is_empty() {
            return Err(invalid_directive(
                Directive::ElseIf,
                DirectiveErrorKind::MissingExpression,
                Some(expr),
            ));
        }
        if !if_chain.is_open() {
            return Err(invalid_directive(
                Directive::ElseIf,
                DirectiveErrorKind::MissingAdjacentConditional,
                Some(expr),
            ));
        }
        if if_chain.has_matched() {
            return Ok(Some(Vec::new()));
        }
        let matched = engine.eval_bool(&expr).unwrap_or(false);
        if_chain.set_matched(matched);
        return Ok(Some(if matched {
            render_targets(node, engine)?
        } else {
            Vec::new()
        }));
    }

    if let Some(expr) = directive_else {
        if !expr.trim().is_empty() {
            return Err(invalid_directive(
                Directive::Else,
                DirectiveErrorKind::UnexpectedExpression,
                Some(expr),
            ));
        }
        if !if_chain.is_open() {
            return Err(invalid_directive(
                Directive::Else,
                DirectiveErrorKind::MissingAdjacentConditional,
                None,
            ));
        }
        // `v-else` ends the chain either way, so anything after it starts fresh.
        let already_matched = if_chain.has_matched();
        if_chain.reset();
        return Ok(Some(if already_matched {
            Vec::new()
        } else {
            render_targets(node, engine)?
        }));
    }

    if_chain.reset();

    Ok(match directive_for {
        Some(expr) => Some(render_for(node, engine, &expr)?),
        None => None,
    })
}

struct ForExpr {
    binding: ForBinding,
    iter: String,
}

fn render_for(node: &Handle, engine: &mut Engine, expr: &str) -> Result<Vec<Handle>> {
    let for_expr = parse_for_expr(engine, expr).ok_or_else(|| Error::InvalidDirective {
        directive: Directive::For,
        kind: DirectiveErrorKind::InvalidExpression,
        expression: Some(expr.to_string()),
    })?;

    // The iterable is evaluated in the enclosing scope, before the loop scope
    // is pushed.
    let iterable = match engine.eval_expr(for_expr.iter.trim()) {
        Ok(iterable) => iterable,
        Err(_) => return Ok(Vec::new()),
    };

    // One scope for the whole loop: each iteration rebinds it instead of
    // installing and tearing down a fresh one.
    engine.enter_scope().map_err(|err| Error::Internal {
        message: format!("failed to manage JavaScript scope: {err}"),
    })?;
    let rendered = render_for_iterations(node, engine, &for_expr, iterable);
    engine.exit_scope();
    rendered
}

fn render_for_iterations(
    node: &Handle,
    engine: &mut Engine,
    for_expr: &ForExpr,
    iterable: JsValue,
) -> Result<Vec<Handle>> {
    let indent_opt = get_indent(node);
    let mut result_nodes = Vec::new();
    let mut render_iteration = |engine: &mut Engine, slots: &[JsValue]| -> Result<()> {
        if engine.bind_for_slots(&for_expr.binding, slots) {
            render_for_item(node, engine, &indent_opt, &mut result_nodes)
        } else {
            Ok(())
        }
    };

    match iterable.variant() {
        JsVariant::Object(obj) if obj.is_array() => {
            let Some(length) = engine.array_length(&obj) else {
                return Ok(result_nodes);
            };
            for (idx, item_idx) in (0..length).enumerate() {
                let item = engine.get_prop(&obj, item_idx);
                render_iteration(engine, &[item, JsValue::new(idx)])?;
            }
        }
        JsVariant::Object(obj) => {
            if let Some(items) = engine.iterable_values(iterable.clone()) {
                for (idx, item) in items.into_iter().enumerate() {
                    render_iteration(engine, &[item, JsValue::new(idx)])?;
                }
            } else {
                let keys = engine.object_keys(iterable.clone());
                for (idx, key) in keys.into_iter().enumerate() {
                    let Some(key_string) = key.as_string() else {
                        continue;
                    };
                    let value = engine.get_prop(&obj, key_string.clone());
                    render_iteration(engine, &[value, key, JsValue::new(idx)])?;
                }
            }
        }
        JsVariant::Integer32(val) => {
            for (idx, num) in (1..=val).enumerate() {
                render_iteration(engine, &[JsValue::new(num), JsValue::new(idx)])?;
            }
        }
        JsVariant::String(val) => {
            for (idx, ch) in val.to_std_string_escaped().chars().enumerate() {
                render_iteration(engine, &[JsValue::new(ch), JsValue::new(idx)])?;
            }
        }
        _ => {}
    }

    Ok(result_nodes)
}

fn parse_for_expr(engine: &mut Engine, expr: &str) -> Option<ForExpr> {
    for (binding_raw, iterable_expr) in split_for_expr(expr) {
        let binding_raw = binding_raw.trim();
        let binding_raw = if binding_raw.starts_with('(') && binding_raw.ends_with(')') {
            binding_raw[1..binding_raw.len() - 1].trim()
        } else {
            binding_raw
        };
        if let Some(binding) = engine.parse_for_binding(binding_raw) {
            return Some(ForExpr {
                binding,
                iter: iterable_expr.trim().to_string(),
            });
        }
    }

    None
}

fn split_for_expr(expr: &str) -> impl Iterator<Item = (&str, &str)> {
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

fn render_for_item(
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

    // Every target after the first starts a fresh line, so it needs the indent
    // the original `v-for` node sat at.
    let mut target_idx = 0;
    walk_siblings(targets, engine, |node, placement| {
        match placement {
            Placement::Keep => {
                let should_indent = target_idx > 0 && !iteration_nodes.is_empty();
                push_node(&mut iteration_nodes, node, should_indent);
            }
            Placement::Drop => {}
            Placement::Replace(nodes) => {
                for (idx, new_node) in nodes.into_iter().enumerate() {
                    let should_indent =
                        (target_idx > 0 && idx == 0 && !iteration_nodes.is_empty()) || idx > 0;
                    push_node(&mut iteration_nodes, new_node, should_indent);
                }
            }
        }
        target_idx += 1;
        Ok(())
    })?;

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
