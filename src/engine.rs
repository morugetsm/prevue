use std::sync::atomic::AtomicUsize;

use boa_ast::{
    Statement, StatementListItem,
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
pub(crate) struct ForBinding {
    pattern: String,
    locals: Vec<String>,
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

        if let Ok(json) = serde_json::to_value(data) {
            if let Ok(val) = JsValue::from_json(&json, &mut engine.context) {
                engine.set_val("$", val);
            }

            if let Some(obj) = json.as_object() {
                for (key, value) in obj.iter().filter(|(key, _)| key.as_str() != "$") {
                    if let Ok(val) = JsValue::from_json(value, &mut engine.context) {
                        engine.set_val(key.as_str(), val);
                    }
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

    pub fn parse_for_binding(&mut self, pattern: &str) -> Option<ForBinding> {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return None;
        }

        let code = format!("let [{pattern}] = [];");
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
        let Binding::Pattern(Pattern::Array(array_pattern)) = binding else {
            return None;
        };
        if array_pattern.bindings().is_empty() || array_pattern.bindings().len() > 3 {
            return None;
        }

        let mut locals = Vec::new();
        collect_binding_names(binding, self.context.interner(), &mut locals);

        Some(ForBinding {
            pattern: pattern.to_string(),
            locals,
        })
    }

    pub fn bind_for_slots<I>(&mut self, binding: &ForBinding, slots: I) -> bool
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
        let copies = binding
            .locals
            .iter()
            .map(|name| {
                format!(
                    "globalThis[{scope_ref}][{}] = {};",
                    js_string_literal(name),
                    name
                )
            })
            .collect::<String>();
        let code = format!(
            "let [{}] = globalThis[{temp_ref}]; {copies}",
            binding.pattern
        );
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

    pub fn eval_setup(&mut self, code: &str) -> JsResult<()> {
        let names = self.parse_setup_bindings(code)?;
        let Some(scope_key) = self.scope_keys.last() else {
            return Ok(());
        };

        let scope_ref = js_string_literal(scope_key);
        let copies = names
            .iter()
            .filter(|name| name.as_str() != "$")
            .map(|name| {
                format!(
                    "globalThis[{scope_ref}][{}] = {};",
                    js_string_literal(name),
                    name
                )
            })
            .collect::<String>();
        self.eval(&format!("{code}\n{copies}"))?;

        Ok(())
    }

    pub fn eval_expr(&mut self, code: &str) -> JsResult<JsValue> {
        let trimmed = code.trim();
        if trimmed.starts_with('{') {
            self.eval(&format!("({trimmed})"))
        } else {
            self.eval(code)
        }
    }

    pub fn eval_str(&mut self, code: &str) -> Option<String> {
        let value = self.eval_expr(code).ok()?;
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
        let value = self.eval_expr(code).ok()?;
        fmt_text(&value, &mut self.context)
    }

    pub fn eval_bool(&mut self, code: &str) -> Option<bool> {
        Some(self.eval_expr(code).ok()?.to_boolean())
    }

    fn parse_setup_bindings(&mut self, code: &str) -> JsResult<Vec<String>> {
        let mut parser = Parser::new(ParserSource::from_bytes(code.as_bytes()));
        let script = parser.parse_script(&Scope::new_global(), self.context.interner_mut())?;

        let mut locals = Vec::new();
        for statement in script.statements().statements() {
            match statement {
                StatementListItem::Declaration(declaration) => {
                    collect_declaration_names(declaration, self.context.interner(), &mut locals);
                }
                StatementListItem::Statement(statement) => {
                    if let Statement::Var(var) = statement.as_ref() {
                        collect_variable_names(&var.0, self.context.interner(), &mut locals);
                    }
                }
            }
        }

        Ok(locals)
    }
}

fn fmt_text(value: &JsValue, context: &mut Context) -> Option<String> {
    match value.variant() {
        JsVariant::Null | JsVariant::Undefined => None,
        JsVariant::String(val) => Some(val.to_std_string_escaped()),
        JsVariant::Object(_) => {
            let json = value.to_json(context).ok()??;
            Some(fmt_json(&json))
        }
        _ => Some(value.display().to_string()),
    }
}

fn fmt_json(val: &JsonValue) -> String {
    match val {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{}\"", s),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                "[]".to_string()
            } else {
                format!(
                    "[ {} ]",
                    arr.iter().map(fmt_json).collect::<Vec<_>>().join(", ")
                )
            }
        }
        JsonValue::Object(obj) => {
            if obj.is_empty() {
                "{}".to_string()
            } else {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, fmt_json(v)))
                    .collect();
                format!("{{ {} }}", items.join(", "))
            }
        }
    }
}

fn collect_binding_names(
    binding: &Binding,
    interner: &boa_engine::interner::Interner,
    locals: &mut Vec<String>,
) {
    match binding {
        Binding::Identifier(ident) => push_identifier(ident, interner, locals),
        Binding::Pattern(pattern) => collect_pattern_names(pattern, interner, locals),
    }
}

fn collect_declaration_names(
    declaration: &Declaration,
    interner: &boa_engine::interner::Interner,
    locals: &mut Vec<String>,
) {
    match declaration {
        Declaration::FunctionDeclaration(function) => {
            push_identifier(&function.name(), interner, locals);
        }
        Declaration::GeneratorDeclaration(function) => {
            push_identifier(&function.name(), interner, locals);
        }
        Declaration::AsyncFunctionDeclaration(function) => {
            push_identifier(&function.name(), interner, locals);
        }
        Declaration::AsyncGeneratorDeclaration(function) => {
            push_identifier(&function.name(), interner, locals);
        }
        Declaration::ClassDeclaration(class) => {
            push_identifier(&class.name(), interner, locals);
        }
        Declaration::Lexical(lexical) => {
            collect_variable_names(lexical.variable_list(), interner, locals);
        }
    }
}

fn collect_variable_names(
    variables: &boa_ast::declaration::VariableList,
    interner: &boa_engine::interner::Interner,
    locals: &mut Vec<String>,
) {
    for variable in variables.as_ref() {
        collect_binding_names(variable.binding(), interner, locals);
    }
}

fn collect_pattern_names(
    pattern: &Pattern,
    interner: &boa_engine::interner::Interner,
    locals: &mut Vec<String>,
) {
    match pattern {
        Pattern::Object(pattern) => {
            for element in pattern.bindings() {
                match element {
                    ObjectPatternElement::SingleName { ident, .. }
                    | ObjectPatternElement::RestProperty { ident } => {
                        push_identifier(ident, interner, locals);
                    }
                    ObjectPatternElement::Pattern { pattern, .. } => {
                        collect_pattern_names(pattern, interner, locals);
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
                        push_identifier(ident, interner, locals);
                    }
                    ArrayPatternElement::Pattern { pattern, .. }
                    | ArrayPatternElement::PatternRest { pattern } => {
                        collect_pattern_names(pattern, interner, locals);
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
    locals: &mut Vec<String>,
) {
    let name = interner.resolve_expect(*ident.sym_ref()).to_string();
    if !locals.iter().any(|existing| existing == &name) {
        locals.push(name);
    }
}

fn js_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}
