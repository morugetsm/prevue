use std::collections::HashMap;

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
    script::Script,
};
use boa_parser::{Parser, Source as ParserSource};
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::{Error, Result};

#[derive(Clone, Debug)]
pub(crate) enum ForBinding {
    Slots(Vec<String>),
    Pattern { source: String, locals: Vec<String> },
}

pub(crate) struct Engine {
    pub context: Context,
    scope_keys: Vec<String>,
    temp_depth: usize,
    eval_cache: Vec<HashMap<String, Script>>,
    for_binding_cache: HashMap<String, ForBinding>,
}

impl Engine {
    pub fn new(data: impl Serialize) -> Result<Self> {
        let mut engine = Self {
            context: Context::default(),
            scope_keys: Default::default(),
            temp_depth: 0,
            eval_cache: Default::default(),
            for_binding_cache: Default::default(),
        };

        engine.enter_scope().map_err(|err| Error::Internal {
            message: format!("failed to manage JavaScript scope: {err}"),
        })?;

        let json = serde_json::to_value(data).map_err(|source| Error::DataSerialize { source })?;
        let value =
            JsValue::from_json(&json, &mut engine.context).map_err(|err| Error::DataInit {
                field: None,
                message: err.to_string(),
            })?;
        engine.set_val("$", value).map_err(|err| Error::DataInit {
            field: None,
            message: err.to_string(),
        })?;

        if let Some(obj) = json.as_object() {
            for (key, value) in obj.iter().filter(|(key, _)| key.as_str() != "$") {
                let field = Some(key.clone());
                let value = JsValue::from_json(value, &mut engine.context).map_err(|err| {
                    Error::DataInit {
                        field: field.clone(),
                        message: err.to_string(),
                    }
                })?;
                engine.set_val(key, value).map_err(|err| Error::DataInit {
                    field,
                    message: err.to_string(),
                })?;
            }
        }

        Ok(engine)
    }

    pub fn enter_scope(&mut self) -> JsResult<()> {
        let key = format!("__scope_{}", self.scope_keys.len());
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

    pub fn set_val(&mut self, key: &str, value: JsValue) -> JsResult<()> {
        let mut scope = self.context.global_object();

        if let Some(scope_key) = self.scope_keys.last() {
            let scope_val = scope.get(JsString::from(scope_key.as_str()), &mut self.context)?;
            if let Some(local) = scope_val.as_object() {
                scope = local;
            }
        }

        scope.set(JsString::from(key), value, false, &mut self.context)?;
        Ok(())
    }

    pub fn parse_for_binding(&mut self, pattern: &str) -> Option<ForBinding> {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return None;
        }
        if let Some(binding) = self.for_binding_cache.get(pattern) {
            return Some(binding.clone());
        }

        let code = format!("let [{pattern}] = [];");
        let mut parser = Parser::new(ParserSource::from_bytes(code.as_bytes()));
        let script = parser
            .parse_script(&Scope::new_global(), self.context.interner_mut())
            .ok()?;
        let [StatementListItem::Declaration(decl)] = script.statements().statements() else {
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

        let binding = if let Some(slots) =
            simple_slot_names(array_pattern.bindings(), self.context.interner())
        {
            ForBinding::Slots(slots)
        } else {
            let mut locals = Vec::new();
            collect_binding_names(binding, self.context.interner(), &mut locals);
            ForBinding::Pattern {
                source: pattern.to_string(),
                locals,
            }
        };
        self.for_binding_cache
            .insert(pattern.to_string(), binding.clone());
        Some(binding)
    }

    pub fn bind_for_slots(&mut self, binding: &ForBinding, slots: &[JsValue]) -> bool {
        match binding {
            ForBinding::Slots(names) => {
                for (idx, name) in names.iter().enumerate() {
                    let value = slots.get(idx).cloned().unwrap_or_else(JsValue::undefined);
                    if self.set_val(name, value).is_err() {
                        return false;
                    }
                }
                true
            }
            ForBinding::Pattern { source, locals } => {
                let Some(scope_key) = self.scope_keys.last().cloned() else {
                    return false;
                };
                let slot_array = JsArray::from_iter(slots.iter().cloned(), &mut self.context);
                self.with_temp(slot_array.into(), |engine, temp_ref| {
                    let scope_ref = js_string_literal(&scope_key);
                    let copies = locals
                        .iter()
                        .map(|name| copy_to_scope(&scope_ref, name))
                        .collect::<String>();
                    engine
                        .eval(&format!(
                            "let [{source}] = globalThis[{temp_ref}]; {copies}"
                        ))
                        .map(|_| ())
                })
                .is_ok()
            }
        }
    }

    pub fn eval_with_temp_val<F>(&mut self, value: JsValue, build_code: F) -> JsResult<JsValue>
    where
        F: FnOnce(&str) -> String,
    {
        self.with_temp(value, |engine, temp_ref| engine.eval(&build_code(temp_ref)))
    }

    fn with_temp<F, T>(&mut self, value: JsValue, f: F) -> JsResult<T>
    where
        F: FnOnce(&mut Self, &str) -> JsResult<T>,
    {
        let temp_key = format!("__temp_{}", self.temp_depth);
        self.temp_depth += 1;

        self.context.global_object().set(
            JsString::from(temp_key.as_str()),
            value,
            false,
            &mut self.context,
        )?;

        let temp_ref = js_string_literal(&temp_key);
        let result = f(self, &temp_ref);

        let _ = self
            .context
            .global_object()
            .delete_property_or_throw(JsString::from(temp_key), &mut self.context);
        self.temp_depth -= 1;

        result
    }

    pub fn eval(&mut self, code: &str) -> JsResult<JsValue> {
        let depth = self.scope_keys.len();
        let script =
            if let Some(script) = self.eval_cache.get(depth).and_then(|cache| cache.get(code)) {
                script.clone()
            } else {
                let scoped = self
                    .scope_keys
                    .iter()
                    .rev()
                    .fold(code.to_string(), |acc, key| {
                        format!(r#"with (globalThis["{key}"]) {{ {acc} }}"#)
                    });
                let script = Script::parse(
                    Source::from_bytes(scoped.as_bytes()),
                    None,
                    &mut self.context,
                )?;
                if self.eval_cache.len() <= depth {
                    self.eval_cache.resize_with(depth + 1, HashMap::new);
                }
                self.eval_cache[depth].insert(code.to_string(), script.clone());
                script
            };
        let evaluated = script.evaluate(&mut self.context)?;

        if evaluated.strict_equals(&JsValue::new(self.context.global_object())) {
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
            .map(|name| copy_to_scope(&scope_ref, name))
            .collect::<String>();
        self.eval(&format!("{code}\n{copies}")).map(|_| ())
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
        self.stringify(&value)
    }

    pub fn eval_fmt(&mut self, code: &str) -> Option<String> {
        let value = self.eval_expr(code).ok()?;
        fmt_text(&value, &mut self.context)
    }

    pub fn stringify(&mut self, value: &JsValue) -> Option<String> {
        match value.variant() {
            JsVariant::Null | JsVariant::Undefined => None,
            JsVariant::Boolean(value) => Some(value.to_string()),
            JsVariant::Integer32(value) => Some(value.to_string()),
            JsVariant::String(value) => Some(value.to_std_string_escaped()),
            _ => Some(
                value
                    .to_string(&mut self.context)
                    .ok()?
                    .to_std_string_escaped(),
            ),
        }
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
        JsVariant::Boolean(val) => Some(val.to_string()),
        JsVariant::Integer32(val) => Some(val.to_string()),
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
        JsonValue::String(s) => js_string_literal(s),
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
                    .map(|(k, v)| format!("{}: {}", js_string_literal(k), fmt_json(v)))
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

fn simple_slot_names(
    elements: &[ArrayPatternElement],
    interner: &boa_engine::interner::Interner,
) -> Option<Vec<String>> {
    elements
        .iter()
        .map(|element| match element {
            ArrayPatternElement::SingleName {
                ident,
                default_init: None,
            } => Some(identifier_name(ident, interner)),
            _ => None,
        })
        .collect()
}

fn push_identifier(
    ident: &Identifier,
    interner: &boa_engine::interner::Interner,
    locals: &mut Vec<String>,
) {
    let name = identifier_name(ident, interner);
    if !locals.iter().any(|existing| existing == &name) {
        locals.push(name);
    }
}

fn identifier_name(ident: &Identifier, interner: &boa_engine::interner::Interner) -> String {
    interner.resolve_expect(*ident.sym_ref()).to_string()
}

fn js_string_literal(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string())
}

fn copy_to_scope(scope_ref: &str, name: &str) -> String {
    format!(
        "globalThis[{scope_ref}][{}] = {name};",
        js_string_literal(name)
    )
}
