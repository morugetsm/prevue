use std::{borrow::Cow, cell::RefCell};

use boa_engine::{JsValue, JsVariant};
use html5ever::{QualName, tendril::StrTendril};
use serde_json::Value as JsonValue;

use crate::{Directive, DirectiveErrorKind, Error, Result, engine::Engine};

pub(crate) fn normalize_bound_attribute(
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
        return is_present(value).then(String::new);
    }

    engine.stringify(value)
}

/// The values Vue is willing to write into an attribute.
fn is_renderable_attr_value(value: &JsValue) -> bool {
    matches!(
        value.variant(),
        JsVariant::String(_)
            | JsVariant::Integer32(_)
            | JsVariant::Float64(_)
            | JsVariant::Boolean(_)
    )
}

/// Vue keeps a boolean attribute when the value is truthy or the empty string.
fn is_present(value: &JsValue) -> bool {
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
            unknown => Err(Error::InvalidDirective {
                directive: Directive::Bind,
                kind: DirectiveErrorKind::UnknownModifier,
                expression: Some(unknown.to_string()),
            }),
        })
}

/// `view-box` becomes `viewBox`, matching Vue's `camelize`.
fn camelize(name: &str) -> String {
    let mut camelized = String::with_capacity(name.len());
    let mut chars = name.chars();

    while let Some(ch) = chars.next() {
        if ch != '-' {
            camelized.push(ch);
            continue;
        }
        match chars.next() {
            Some(next) if next.is_ascii_alphanumeric() || next == '_' => {
                camelized.push(next.to_ascii_uppercase());
            }
            Some(next) => {
                camelized.push('-');
                camelized.push(next);
            }
            None => camelized.push('-'),
        }
    }

    camelized
}

/// Vue's `isSSRSafeAttrName`, plus `\0` and `\r` — rejecting more than Vue can
/// only keep output Vue would never produce anyway.
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

/// A style object in source order. Vue relies on JavaScript object semantics
/// here, so a repeated key keeps its position but takes the newer value.
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
                    set_declaration(declarations, hyphenate(name).into_owned(), value);
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
                declarations.push((name.to_string(), value.trim().to_string()));
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

/// Vue writes only strings and numbers into a declaration.
fn declaration_value(value: &JsonValue) -> Option<String> {
    match value {
        JsonValue::String(value) if !value.is_empty() => Some(value.clone()),
        JsonValue::Number(value) => Some(value.to_string()),
        _ => None,
    }
}

fn stringify_style(declarations: &Declarations) -> String {
    declarations
        .iter()
        .map(|(name, value)| format!("{name}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `viewBox` becomes `view-box`, the inverse of [`camelize`]. Matching Vue's
/// `\B([A-Z])` the leading character is left alone, so `MozTransform` becomes
/// `moz-transform` rather than `-moz-transform`.
fn hyphenate(name: &str) -> Cow<'_, str> {
    // Custom properties keep their case; no camelCase hump means no rewriting.
    if name.starts_with("--") || !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Cow::Borrowed(name);
    }

    let mut hyphenated = String::with_capacity(name.len() + 4);
    for (idx, ch) in name.char_indices() {
        if ch.is_ascii_uppercase() {
            if idx > 0 {
                hyphenated.push('-');
            }
            hyphenated.push(ch.to_ascii_lowercase());
        } else {
            hyphenated.push(ch);
        }
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

/// Vue's compiler folds a static `style` and a binding into one array, so they
/// go through the same key merge rather than sitting side by side.
pub(crate) fn merge_style(existing: &str, value: &str) -> String {
    let mut declarations = parse_string_style(existing);
    for (name, value) in parse_string_style(value) {
        set_declaration(&mut declarations, name, value);
    }

    stringify_style(&declarations)
}

#[derive(Default)]
pub(crate) struct AttrEdits {
    sets: Vec<(usize, String, String)>,
    removes: Vec<usize>,
    adds: Vec<(String, QualName, String)>,
}

impl AttrEdits {
    pub(crate) fn set(&mut self, idx: usize, name: String, value: String) {
        self.sets.push((idx, name, value));
    }

    pub(crate) fn remove(&mut self, idx: usize) {
        self.removes.push(idx);
    }

    pub(crate) fn add(&mut self, name: String, template: QualName, value: String) {
        self.adds.push((name, template, value));
    }

    pub(crate) fn apply(mut self, attrs: &RefCell<Vec<html5ever::Attribute>>) {
        // Removal runs back to front below, which needs them in order.
        self.removes.sort_unstable();
        self.removes.dedup();

        let mut attrs_mut = attrs.borrow_mut();
        for (idx, name, value) in self.sets.iter().rev() {
            attrs_mut[*idx].name.local = html5ever::LocalName::from(name.as_str());
            attrs_mut[*idx].value = StrTendril::from(value.as_str());
        }
        for idx in self.removes.iter().rev() {
            attrs_mut.remove(*idx);
        }
        drop(attrs_mut);

        let mut attrs_mut = attrs.borrow_mut();
        for (name, template, value) in self.adds {
            if let Some(existing) = attrs_mut
                .iter_mut()
                .find(|attr| attr.name.local.as_ref() == name.as_str())
            {
                let value = match name.as_str() {
                    "class" => merge_class(existing.value.as_ref(), &value),
                    "style" => merge_style(existing.value.as_ref(), &value),
                    _ => value,
                };
                existing.value = StrTendril::from(value.as_str());
            } else {
                attrs_mut.push(html5ever::Attribute {
                    name: QualName::new(
                        template.prefix.clone(),
                        template.ns.clone(),
                        html5ever::LocalName::from(name.as_str()),
                    ),
                    value: StrTendril::from(value.as_str()),
                });
            }
        }
    }
}
