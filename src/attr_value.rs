use std::{borrow::Cow, cell::RefCell};

use boa_engine::JsValue;
use html5ever::{QualName, tendril::StrTendril};
use serde_json::Value as JsonValue;

use crate::{Error, Result, engine::Engine};

pub(crate) fn normalize_bound_attribute(
    engine: &mut Engine,
    key: &str,
    value: &JsValue,
) -> Option<String> {
    match key {
        "class" | "style" => {
            let value = engine.json_value(value)?;
            if key == "class" {
                normalize_class(&value)
            } else {
                normalize_style(&value)
            }
        }
        _ => engine.stringify(value),
    }
}

pub(crate) fn validate_attribute_name(name: &str) -> Result<()> {
    (!name.is_empty()
        && name
            .chars()
            .all(|ch| !ch.is_ascii_whitespace() && !matches!(ch, '\0' | '>' | '/' | '=')))
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

fn css_property_name(name: &str) -> Cow<'_, str> {
    // Custom properties keep their case; no camelCase hump means no rewriting.
    if name.starts_with("--") || !name.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Cow::Borrowed(name);
    }

    Cow::Owned(name.chars().fold(String::new(), |mut property, ch| {
        if ch.is_ascii_uppercase() {
            property.push('-');
            property.push(ch.to_ascii_lowercase());
        } else {
            property.push(ch);
        }
        property
    }))
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

pub(crate) fn merge_style(existing: &str, value: &str) -> String {
    match (existing.trim(), value.trim()) {
        ("", value) => value.to_string(),
        (existing, "") => existing.to_string(),
        (existing, value) if existing.ends_with(';') => format!("{existing} {value}"),
        (existing, value) => format!("{existing}; {value}"),
    }
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

    pub(crate) fn apply(self, attrs: &RefCell<Vec<html5ever::Attribute>>) {
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
