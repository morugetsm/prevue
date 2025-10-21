use std::sync::atomic::AtomicUsize;

use boa_engine::{Context, JsResult, JsString, JsValue, Source, object::ObjectInitializer};
use serde::Serialize;

pub struct Engine {
    pub(crate) context: Context,
    scope_keys: Vec<String>,
    scope_next: AtomicUsize,
}

impl Engine {
    pub fn new(payload: impl Serialize) -> Self {
        let mut engine = Self {
            context: Context::default(),
            scope_keys: Default::default(),
            scope_next: AtomicUsize::new(0),
        };

        engine.enter_scope().unwrap();

        let json = serde_json::to_value(payload).unwrap();
        if let Some(obj) = json.as_object() {
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
            && let Ok(JsValue::Object(local)) =
                scope.get(JsString::from(scope_key.as_str()), &mut self.context)
        {
            scope = local;
        }

        let _ = scope.set(JsString::from(key), value, false, &mut self.context);
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
            &JsValue::Object(self.context.global_object()),
            &mut self.context,
        )? {
            Ok(JsValue::Null)
        } else {
            Ok(evaluated)
        }
    }

    pub fn eval_str(&mut self, code: &str) -> Option<String> {
        let value = self.eval(code).ok()?;
        match value {
            JsValue::Null => None,
            JsValue::Undefined => None,
            JsValue::String(val) => Some(val.to_std_string_escaped()),
            JsValue::Object(_) => {
                let json = value.to_json(&mut self.context).ok()?;
                Some(json.to_string())
            }
            _ => Some(value.display().to_string()),
        }
    }

    pub fn eval_bool(&mut self, code: &str) -> Option<bool> {
        let value = self.eval(code).ok()?;
        Some(value.to_boolean())
    }
}
