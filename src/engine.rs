use std::sync::atomic::AtomicUsize;

use boa_ast::{
    StatementListItem,
    declaration::{Binding, Declaration},
    expression::Identifier,
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    scope::Scope,
};
use boa_engine::{
    Context, JsResult, JsString, JsValue, JsVariant, Source,
    object::{ObjectInitializer, builtins::JsArray},
};
use boa_parser::{Parser, Source as ParserSource};
use serde::Serialize;
use serde_json::Value as JsonValue;

#[derive(Clone, Debug)]
pub(crate) struct BindingAlias {
    source: String,
    names: Vec<String>,
}

pub(crate) struct Engine {
    pub context: Context,
    scope_keys: Vec<String>,
    scope_next: AtomicUsize,
    binding_next: AtomicUsize,
}

impl Engine {
    pub fn new(data: impl Serialize) -> Self {
        let mut engine = Self {
            context: Context::default(),
            scope_keys: Default::default(),
            scope_next: AtomicUsize::new(0),
            binding_next: AtomicUsize::new(0),
        };

        engine.enter_scope().unwrap();

        if let Ok(json) = serde_json::to_value(data)
            && let Some(obj) = json.as_object()
        {
            for (key, value) in obj.iter() {
                if let Ok(val) = JsValue::from_json(value, &mut engine.context) {
                    engine.set_val(key.as_str(), val);
                }
            }
        }

        engine
    }

    pub fn enter_scope(&mut self) -> JsResult<()> {
        let key = format!(
            "__scope_{}",
            self.scope_next
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        let scope = ObjectInitializer::new(&mut self.context).build();
        self.context.global_object().set(
            JsString::from(key.as_str()),
            scope,
            false,
            &mut self.context,
        )?;
        self.scope_keys.push(key);
        Ok(())
    }

    pub fn exit_scope(&mut self) {
        if let Some(key) = self.scope_keys.pop() {
            let _ = self
                .context
                .global_object()
                .delete_property_or_throw(JsString::from(key), &mut self.context);
        }
    }

    pub fn set_val(&mut self, key: &str, value: JsValue) {
        let mut scope = self.context.global_object();

        if let Some(scope_key) = self.scope_keys.last()
            && let Ok(scope_val) = scope.get(JsString::from(scope_key.as_str()), &mut self.context)
            && let Some(local) = scope_val.as_object()
        {
            scope = local;
        }

        let _ = scope.set(JsString::from(key), value, false, &mut self.context);
    }

    pub fn parse_binding_alias(&mut self, source: &str) -> Option<BindingAlias> {
        let source = source.trim();
        if source.is_empty() {
            return None;
        }

        let code = format!("let [{source}] = [];");
        let mut parser = Parser::new(ParserSource::from_bytes(code.as_bytes()));
        let script = parser
            .parse_script(&Scope::new_global(), self.context.interner_mut())
            .ok()?;

        let statements = script.statements().statements();
        let [StatementListItem::Declaration(decl)] = statements else {
            return None;
        };
        let Declaration::Lexical(lexical) = decl.as_ref() else {
            return None;
        };
        let [variable] = lexical.variable_list().as_ref() else {
            return None;
        };

        let binding = variable.binding();
        let Binding::Pattern(Pattern::Array(pattern)) = binding else {
            return None;
        };
        if pattern.bindings().is_empty() || pattern.bindings().len() > 3 {
            return None;
        }

        let mut names = Vec::new();
        collect_binding_names(binding, self.context.interner(), &mut names);

        Some(BindingAlias {
            source: source.to_string(),
            names,
        })
    }

    pub fn bind_alias<I>(&mut self, alias: &BindingAlias, slots: I) -> bool
    where
        I: IntoIterator<Item = JsValue>,
    {
        let Some(scope_key) = self.scope_keys.last().cloned() else {
            return false;
        };
        let temp_key = format!(
            "__temp_{}",
            self.binding_next
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );

        let slot_array = JsArray::from_iter(slots, &mut self.context);

        if self
            .context
            .global_object()
            .set(
                JsString::from(temp_key.as_str()),
                slot_array,
                false,
                &mut self.context,
            )
            .is_err()
        {
            return false;
        }

        let temp_ref = js_string_literal(&temp_key);
        let scope_ref = js_string_literal(&scope_key);
        let copies = alias
            .names
            .iter()
            .map(|name| {
                format!(
                    "globalThis[{scope_ref}][{}] = {};",
                    js_string_literal(name),
                    name
                )
            })
            .collect::<String>();
        let code = format!("let [{}] = globalThis[{temp_ref}]; {copies}", alias.source);
        let result = self.eval(&code).is_ok();

        let _ = self
            .context
            .global_object()
            .delete_property_or_throw(JsString::from(temp_key), &mut self.context);

        result
    }

    pub fn eval(&mut self, code: &str) -> JsResult<JsValue> {
        let scoped = self
            .scope_keys
            .iter()
            .rev()
            .fold(code.to_string(), |acc, key| {
                format!(r#"with (globalThis["{key}"]) {{ {acc} }}"#)
            });
        let evaluated = self.context.eval(Source::from_bytes(scoped.as_bytes()))?;

        if evaluated.equals(
            &JsValue::new(self.context.global_object()),
            &mut self.context,
        )? {
            Ok(JsValue::null())
        } else {
            Ok(evaluated)
        }
    }

    pub fn eval_str(&mut self, code: &str) -> Option<String> {
        let value = self.eval(code).ok()?;
        match value.variant() {
            JsVariant::Null | JsVariant::Undefined => None,
            JsVariant::String(val) => Some(val.to_std_string_escaped()),
            _ => Some(
                value
                    .to_string(&mut self.context)
                    .ok()?
                    .to_std_string_escaped(),
            ),
        }
    }

    pub fn eval_fmt(&mut self, code: &str) -> Option<String> {
        fn fmt(val: &JsonValue) -> String {
            match val {
                JsonValue::Null => "null".to_string(),
                JsonValue::Bool(b) => b.to_string(),
                JsonValue::Number(n) => n.to_string(),
                JsonValue::String(s) => format!("\"{}\"", s),
                JsonValue::Array(arr) => {
                    if arr.is_empty() {
                        "[]".to_string()
                    } else {
                        format!("[ {} ]", arr.iter().map(fmt).collect::<Vec<_>>().join(", "))
                    }
                }
                JsonValue::Object(obj) => {
                    if obj.is_empty() {
                        "{}".to_string()
                    } else {
                        let items: Vec<String> = obj
                            .iter()
                            .map(|(k, v)| format!("\"{}\": {}", k, fmt(v)))
                            .collect();
                        format!("{{ {} }}", items.join(", "))
                    }
                }
            }
        }

        let value = self.eval(code).ok()?;
        match value.variant() {
            JsVariant::Null | JsVariant::Undefined => None,
            JsVariant::String(val) => Some(val.to_std_string_escaped()),
            JsVariant::Object(_) => {
                let json = value.to_json(&mut self.context).ok()??;
                Some(fmt(&json))
            }
            _ => Some(value.display().to_string()),
        }
    }

    pub fn eval_bool(&mut self, code: &str) -> Option<bool> {
        Some(self.eval(code).ok()?.to_boolean())
    }
}

fn collect_binding_names(
    binding: &Binding,
    interner: &boa_engine::interner::Interner,
    names: &mut Vec<String>,
) {
    match binding {
        Binding::Identifier(ident) => push_identifier(ident, interner, names),
        Binding::Pattern(pattern) => collect_pattern_names(pattern, interner, names),
    }
}

fn collect_pattern_names(
    pattern: &Pattern,
    interner: &boa_engine::interner::Interner,
    names: &mut Vec<String>,
) {
    match pattern {
        Pattern::Object(pattern) => {
            for element in pattern.bindings() {
                match element {
                    ObjectPatternElement::SingleName { ident, .. }
                    | ObjectPatternElement::RestProperty { ident } => {
                        push_identifier(ident, interner, names);
                    }
                    ObjectPatternElement::Pattern { pattern, .. } => {
                        collect_pattern_names(pattern, interner, names);
                    }
                    ObjectPatternElement::AssignmentPropertyAccess { .. }
                    | ObjectPatternElement::AssignmentRestPropertyAccess { .. } => {}
                }
            }
        }
        Pattern::Array(pattern) => {
            for element in pattern.bindings() {
                match element {
                    ArrayPatternElement::SingleName { ident, .. }
                    | ArrayPatternElement::SingleNameRest { ident } => {
                        push_identifier(ident, interner, names);
                    }
                    ArrayPatternElement::Pattern { pattern, .. }
                    | ArrayPatternElement::PatternRest { pattern } => {
                        collect_pattern_names(pattern, interner, names);
                    }
                    ArrayPatternElement::Elision
                    | ArrayPatternElement::PropertyAccess { .. }
                    | ArrayPatternElement::PropertyAccessRest { .. } => {}
                }
            }
        }
    }
}

fn push_identifier(
    ident: &Identifier,
    interner: &boa_engine::interner::Interner,
    names: &mut Vec<String>,
) {
    let name = interner.resolve_expect(*ident.sym_ref()).to_string();
    if !names.iter().any(|existing| existing == &name) {
        names.push(name);
    }
}

fn js_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}
