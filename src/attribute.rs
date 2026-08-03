use std::{borrow::Cow, cell::RefCell};

use boa_engine::{JsValue, JsVariant};
use html5ever::{QualName, ns, tendril::StrTendril};
use serde_json::Value as JsonValue;

use crate::{
    engine::Engine,
    error::{Directive, DirectiveErrorKind, Error, Result},
};

pub(crate) fn render_dynamic_attr(
    engine: &mut Engine,
    key: &str,
    value: &JsValue,
) -> Option<String> {
    if let "class" | "style" = key {
        let json = engine.json_value(value)?;
        return if key == "class" {
            normalize_class(&json)
        } else {
            normalize_style(&json)
        };
    }

    // Ahead of the name check, so `:disabled="[]"` drops despite being truthy.
    if !is_renderable_attr_value(value) {
        return None;
    }

    if is_boolean_attribute(key) {
        return include_boolean_attr(value).then(String::new);
    }

    engine.stringify(value)
}

/// The value kinds an attribute may hold. Anything else drops the attribute.
fn is_renderable_attr_value(value: &JsValue) -> bool {
    matches!(
        value.variant(),
        JsVariant::String(_)
            | JsVariant::Integer32(_)
            | JsVariant::Float64(_)
            | JsVariant::Boolean(_)
    )
}

/// A boolean attribute is kept when the value is truthy or the empty string.
pub(crate) fn include_boolean_attr(value: &JsValue) -> bool {
    value.to_boolean() || matches!(value.variant(), JsVariant::String(text) if text.is_empty())
}

/// Decided by presence alone, so `disabled="false"` still disables.
fn is_boolean_attribute(name: &str) -> bool {
    matches!(
        name,
        "allowfullscreen"
            | "async"
            | "autofocus"
            | "autoplay"
            | "checked"
            | "controls"
            | "default"
            | "defer"
            | "disabled"
            | "formnovalidate"
            | "hidden"
            | "inert"
            | "ismap"
            | "itemscope"
            | "loop"
            | "multiple"
            | "muted"
            | "nomodule"
            | "novalidate"
            | "open"
            | "readonly"
            | "required"
            | "reversed"
            | "scoped"
            | "seamless"
            | "selected"
    )
}

/// Apply `v-bind` modifiers to a resolved attribute name.
pub(crate) fn apply_modifiers(name: String, modifiers: &str) -> Result<String> {
    modifiers
        .split('.')
        .filter(|modifier| !modifier.is_empty())
        .try_fold(name, |name, modifier| match modifier {
            "camel" => Ok(camelize(&name)),
            // Rendering to HTML has no DOM property to set, so `.prop` only
            // reads as documentation. A `.`-prefixed key still drops.
            "prop" => Ok(name),
            "attr" => Ok(format!("^{name}")),
            unknown => Err(Error::InvalidDirective {
                directive: Directive::Bind,
                kind: DirectiveErrorKind::UnknownModifier,
                expression: Some(unknown.to_string()),
            }),
        })
}

/// `view-box` becomes `viewBox`: a `-` before a word character is dropped and
/// that character uppercased.
fn camelize(name: &str) -> String {
    let mut camelized = String::with_capacity(name.len());
    let mut chars = name.chars().peekable();

    while let Some(ch) = chars.next() {
        // Peeking leaves a `-` that matched nothing free to start the next
        // match itself, so `a--b` becomes `a-B`.
        let word = ch == '-'
            && chars
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_');

        if word {
            camelized.extend(chars.next().map(|next| next.to_ascii_uppercase()));
        } else {
            camelized.push(ch);
        }
    }

    camelized
}

/// Names that never reach the output: render bookkeeping, and event listeners
/// like `onClick` — the plain `onclick` attribute still goes through.
pub(crate) fn is_ignored_prop(key: &str) -> bool {
    matches!(
        key,
        "key" | "ref" | "innerHTML" | "textContent" | "ref_key" | "ref_for"
    ) || is_on(key)
        // A `.`-prefixed key is a DOM property, which has no HTML spelling.
        || key.starts_with('.')
}

/// An event listener name: `on` followed by a character that is not lowercase
/// ASCII, which is what separates `onClick` from the `onclick` attribute.
fn is_on(key: &str) -> bool {
    key.strip_prefix("on")
        .and_then(|rest| rest.chars().next())
        .is_some_and(|ch| !ch.is_ascii_lowercase())
}

/// SVG, MathML and custom elements keep a name as written, since case carries
/// meaning there. Everything else folds down to an HTML attribute name.
///
/// A leading `^` forces a name to render as an attribute; the marker itself
/// never reaches HTML, whichever of the two paths the name takes.
pub(crate) fn attribute_name_for(tag: &QualName, key: String) -> String {
    let key = match key.strip_prefix('^') {
        Some(rest) => rest.to_string(),
        None => key,
    };

    if tag.ns == ns!(svg) || tag.ns == ns!(mathml) || tag.local.contains('-') {
        return key;
    }

    match key.as_str() {
        "acceptCharset" => "accept-charset".to_string(),
        "className" => "class".to_string(),
        "htmlFor" => "for".to_string(),
        "httpEquiv" => "http-equiv".to_string(),
        _ if key.bytes().any(|byte| byte.is_ascii_uppercase()) => key.to_ascii_lowercase(),
        _ => key,
    }
}

/// A textarea has no value attribute, so a bound `value` becomes its content.
pub(crate) fn is_textarea_value(tag: &QualName, name: &str) -> bool {
    name == "value" && tag.local.as_ref() == "textarea"
}

/// A name that would break out of the attribute it is written into, or split
/// into two, is refused rather than serialized.
pub(crate) fn validate_attribute_name(name: &str) -> Result<()> {
    (!name.is_empty()
        && name.chars().all(|ch| {
            !ch.is_ascii_whitespace() && !matches!(ch, '\0' | '>' | '/' | '=' | '"' | '\'')
        }))
    .then_some(())
    .ok_or_else(|| Error::InvalidAttributeName {
        name: name.to_string(),
    })
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

/// Declarations in source order, with object semantics: a repeated key keeps
/// its position but takes the newer value.
type Declarations = Vec<(String, String)>;

fn set_declaration(declarations: &mut Declarations, name: String, value: String) {
    match declarations.iter_mut().find(|(key, _)| *key == name) {
        Some(entry) => entry.1 = value,
        None => declarations.push((name, value)),
    }
}

/// A top-level string passes through untouched; everything else collapses into
/// one declaration list, later keys winning.
fn normalize_style(value: &JsonValue) -> Option<String> {
    if let JsonValue::String(text) = value {
        let text = text.trim();
        return (!text.is_empty()).then(|| text.to_string());
    }

    let mut declarations = Declarations::new();
    collect_declarations(value, &mut declarations);
    (!declarations.is_empty()).then(|| stringify_style(&declarations))
}

fn collect_declarations(value: &JsonValue, declarations: &mut Declarations) {
    match value {
        JsonValue::String(text) => {
            for (name, value) in parse_string_style(text) {
                set_declaration(declarations, name, value);
            }
        }
        JsonValue::Array(values) => {
            for value in values {
                collect_declarations(value, declarations);
            }
        }
        JsonValue::Object(values) => {
            for (name, value) in values {
                if let Some(value) = declaration_value(value) {
                    set_declaration(declarations, name.clone(), value);
                }
            }
        }
        _ => {}
    }
}

/// Split `a: 1; b: 2` into declarations, ignoring `;` inside `()` so that
/// `background: url(a;b)` survives.
fn parse_string_style(css: &str) -> Declarations {
    let mut declarations = Declarations::new();
    let mut depth = 0usize;
    let mut start = 0;

    let mut push = |item: &str| {
        if let Some((name, value)) = item.split_once(':') {
            let name = name.trim();
            if !name.is_empty() {
                set_declaration(
                    &mut declarations,
                    name.to_string(),
                    value.trim().to_string(),
                );
            }
        }
    };

    let without_comments = strip_css_comments(css);
    for (idx, ch) in without_comments.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => {
                push(&without_comments[start..idx]);
                start = idx + 1;
            }
            _ => {}
        }
    }
    push(&without_comments[start..]);

    declarations
}

/// A static `style` attribute takes the same normalization as a binding.
/// `None` leaves the attribute exactly as written.
pub(crate) fn normalize_style_attribute(css: &str) -> Option<String> {
    let declarations = parse_string_style(css);
    (!declarations.is_empty()).then(|| stringify_style(&declarations))
}

fn strip_css_comments(css: &str) -> Cow<'_, str> {
    if !css.contains("/*") {
        return Cow::Borrowed(css);
    }

    let mut stripped = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(open) = rest.find("/*") {
        stripped.push_str(&rest[..open]);
        rest = match rest[open + 2..].find("*/") {
            Some(close) => &rest[open + 2 + close + 2..],
            None => "",
        };
    }
    stripped.push_str(rest);

    Cow::Owned(stripped)
}

/// Only strings and numbers become a declaration value.
fn declaration_value(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) if !value.is_empty() => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

/// Hyphenating here rather than while collecting keeps a key written in CSS
/// and the same key written in JavaScript distinct until the very end. Custom
/// properties are exempt, which is also what keeps their case.
fn stringify_style(declarations: &Declarations) -> String {
    declarations
        .iter()
        .map(|(name, value)| {
            let name = match name.starts_with("--") {
                true => Cow::Borrowed(name.as_str()),
                false => hyphenate(name),
            };
            format!("{name}: {value};")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `viewBox` becomes `view-box`, the inverse of [`camelize`]. A `-` only goes
/// in after a word character, so `a-Bc` becomes `a-bc` rather than `a--bc`,
/// and the whole name is lowercased whether or not one went in.
fn hyphenate(name: &str) -> Cow<'_, str> {
    if name.is_ascii() && !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Cow::Borrowed(name);
    }

    let mut hyphenated = String::with_capacity(name.len() + 4);
    let mut after_word = false;
    for ch in name.chars() {
        if after_word && ch.is_ascii_uppercase() {
            hyphenated.push('-');
        }
        after_word = ch.is_ascii_alphanumeric() || ch == '_';
        hyphenated.extend(ch.to_lowercase());
    }

    Cow::Owned(hyphenated)
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

pub(crate) fn merge_class(existing: &str, value: &str) -> String {
    match (existing.trim(), value.trim()) {
        ("", value) => value.to_string(),
        (existing, "") => existing.to_string(),
        (existing, value) => format!("{existing} {value}"),
    }
}

/// A static `style` and a binding fold into one declaration list rather than
/// sitting side by side, so a repeated property resolves once.
pub(crate) fn merge_style(existing: &str, value: &str) -> String {
    let mut declarations = parse_string_style(existing);
    for (name, value) in parse_string_style(value) {
        set_declaration(&mut declarations, name, value);
    }

    stringify_style(&declarations)
}

enum Op {
    /// Rewrite the attribute at this position.
    Set(String, String),
    /// An attribute a directive produced at this position.
    Add(String, QualName, String),
}

#[derive(Default)]
pub(crate) struct AttrEdits {
    ops: Vec<(usize, Op)>,
    trailing: Vec<(String, QualName, String)>,
    removes: Vec<usize>,
}

impl AttrEdits {
    pub(crate) fn set(&mut self, idx: usize, name: String, value: String) {
        self.ops.push((idx, Op::Set(name, value)));
    }

    pub(crate) fn remove(&mut self, idx: usize) {
        self.removes.push(idx);
    }

    pub(crate) fn add(&mut self, idx: usize, name: String, template: QualName, value: String) {
        self.ops.push((idx, Op::Add(name, template, value)));
    }

    /// Lands after everything the element wrote itself, so `v-show` wins over a
    /// `style` binding no matter which came first.
    pub(crate) fn add_last(&mut self, name: String, template: QualName, value: String) {
        self.trailing.push((name, template, value));
    }

    pub(crate) fn apply(mut self, attrs: &RefCell<Vec<html5ever::Attribute>>) {
        if self.ops.is_empty() && self.removes.is_empty() && self.trailing.is_empty() {
            return;
        }

        // The walk below consumes ops in index order.
        self.ops.sort_by_key(|(idx, _)| *idx);

        let mut attrs_mut = attrs.borrow_mut();
        let mut merged = Vec::with_capacity(attrs_mut.len() + self.trailing.len());
        let mut ops = self.ops.into_iter().peekable();

        for (idx, attr) in attrs_mut.iter().enumerate() {
            let mut consumed = false;
            while let Some((_, op)) = ops.next_if(|(op_idx, _)| *op_idx == idx) {
                merge_props(
                    &mut merged,
                    match op {
                        Op::Set(name, value) => rename(&attr.name, &name, &value),
                        Op::Add(name, template, value) => rename(&template, &name, &value),
                    },
                );
                consumed = true;
            }

            if !consumed && !self.removes.contains(&idx) {
                merge_props(&mut merged, attr.clone());
            }
        }

        for (name, template, value) in self.trailing {
            merge_props(&mut merged, rename(&template, &name, &value));
        }

        *attrs_mut = merged;
    }
}

/// An attribute carrying `template`'s namespace under a different name.
fn rename(template: &QualName, name: &str, value: &str) -> html5ever::Attribute {
    html5ever::Attribute {
        name: QualName::new(
            template.prefix.clone(),
            template.ns.clone(),
            html5ever::LocalName::from(name),
        ),
        value: StrTendril::from(value),
    }
}

/// A repeated name keeps its first position and takes the newer value, except
/// `class` and `style`, which merge instead.
fn merge_props(attrs: &mut Vec<html5ever::Attribute>, attr: html5ever::Attribute) {
    let Some(existing) = attrs
        .iter_mut()
        .find(|kept| kept.name.local == attr.name.local)
    else {
        attrs.push(attr);
        return;
    };

    let merged = match existing.name.local.as_ref() {
        "class" => merge_class(existing.value.as_ref(), attr.value.as_ref()),
        "style" => merge_style(existing.value.as_ref(), attr.value.as_ref()),
        _ => {
            existing.value = attr.value;
            return;
        }
    };
    existing.value = StrTendril::from(merged.as_str());
}

/// What is left of a directive attribute once every render branch has passed
/// on it.
pub(crate) enum Unhandled {
    /// A directive that names something the rendered HTML cannot carry, so it
    /// leaves no trace rather than becoming an attribute.
    Unrendered,
    /// Not a directive at all — a misspelling.
    Unknown,
}

pub(crate) fn classify(local: &str) -> Option<Unhandled> {
    let name = directive_name(local)?;

    Some(match is_builtin(name) {
        true => Unhandled::Unrendered,
        false => Unhandled::Unknown,
    })
}

/// A directive is written `v-name:arg.modifier`, and each shorthand stands in
/// for the directive it names.
fn directive_name(local: &str) -> Option<&str> {
    let rest = match local.as_bytes().first()? {
        b'@' => return Some("on"),
        b'#' => return Some("slot"),
        b':' | b'.' => return Some("bind"),
        _ => local.strip_prefix("v-")?,
    };

    rest.split([':', '.']).next()
}

/// Every directive name the template syntax defines, whether or not prevue
/// renders it.
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "bind"
            | "cloak"
            | "else-if"
            | "else"
            | "for"
            | "html"
            | "if"
            | "model"
            | "on"
            | "once"
            | "pre"
            | "show"
            | "slot"
            | "text"
            | "memo"
    )
}
