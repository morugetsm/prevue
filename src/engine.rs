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
    object::{JsObject, ObjectInitializer, builtins::JsArray},
    property::PropertyKey,
    script::Script,
};
use boa_parser::{Parser, Source as ParserSource};
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::{Error, Result};

#[derive(Clone, Debug)]
pub(crate) enum ForBinding {
    Slots(Vec<JsString>),
    Pattern {
        source: String,
        locals: Vec<JsString>,
    },
}

// One `with` level of the scope chain. The object stays installed on
// `globalThis` so `v-for` iterations reuse it instead of churning globals.
struct ScopeFrame {
    object: JsObject,
    written: Vec<JsString>,
}

impl ScopeFrame {
    fn record(&mut self, key: JsString) {
        if !self.written.contains(&key) {
            self.written.push(key);
        }
    }
}

pub(crate) struct Engine {
    context: Context,
    // Compared against on every evaluation; building it each time is pure churn.
    global: JsValue,
    scopes: Vec<ScopeFrame>,
    depth: usize,
    temp_depth: usize,
    eval_cache: Vec<HashMap<String, Script>>,
    for_binding_cache: HashMap<String, ForBinding>,
}

fn scope_key(depth: usize) -> String {
    format!("__scope_{depth}")
}

impl Engine {
    /// Build the JavaScript context. This is by far the most expensive part of
    /// a render, so callers that render repeatedly should keep the engine alive
    /// and call [`Engine::install_data`] per render.
    pub fn new() -> Result<Self> {
        let context = Context::default();
        let mut engine = Self {
            global: JsValue::new(context.global_object()),
            context,
            scopes: Default::default(),
            depth: 0,
            temp_depth: 0,
            eval_cache: Default::default(),
            for_binding_cache: Default::default(),
        };

        engine.enter_scope().map_err(|err| Error::Internal {
            message: format!("failed to manage JavaScript scope: {err}"),
        })?;

        Ok(engine)
    }

    /// Replace the render data, dropping whatever a previous render installed.
    pub fn install_data(&mut self, data: impl Serialize) -> Result<()> {
        // A render that failed inside a `v-for` still unwinds its scopes, but
        // pin the depth anyway so one bad render cannot poison the next.
        self.depth = 1;
        self.reset_scope(&[]);

        let json = serde_json::to_value(data).map_err(|source| Error::DataSerialize { source })?;
        let root = JsValue::from_json(&json, &mut self.context).map_err(|err| Error::DataInit {
            field: None,
            message: err.to_string(),
        })?;
        self.set_val("$", root.clone())
            .map_err(|err| Error::DataInit {
                field: None,
                message: err.to_string(),
            })?;

        // Objects and arrays are read back off `$` rather than converted twice,
        // so `list` and `$.list` are the same object. Primitives have no
        // identity to share, so re-converting them is equivalent and cheaper.
        if let (Some(fields), Some(root)) = (json.as_object(), root.as_object()) {
            for (name, value) in fields.iter().filter(|(name, _)| name.as_str() != "$") {
                let field = Some(name.clone());
                let value = match value {
                    JsonValue::Object(_) | JsonValue::Array(_) => {
                        root.get(JsString::from(name.as_str()), &mut self.context)
                    }
                    primitive => JsValue::from_json(primitive, &mut self.context),
                }
                .map_err(|err| Error::DataInit {
                    field: field.clone(),
                    message: err.to_string(),
                })?;
                self.set_val(name, value).map_err(|err| Error::DataInit {
                    field,
                    message: err.to_string(),
                })?;
            }
        }

        Ok(())
    }

    pub fn enter_scope(&mut self) -> JsResult<()> {
        if self.depth == self.scopes.len() {
            let object = ObjectInitializer::new(&mut self.context).build();
            let key = JsString::from(scope_key(self.depth).as_str());
            self.context
                .global_object()
                .set(key, object.clone(), false, &mut self.context)?;
            self.scopes.push(ScopeFrame {
                object,
                written: Vec::new(),
            });
        }
        self.depth += 1;
        self.reset_scope(&[]);
        Ok(())
    }

    pub fn exit_scope(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// Drop bindings left by a previous user of this scope, except those the
    /// caller is about to overwrite anyway.
    fn reset_scope(&mut self, keep: &[JsString]) {
        let Some(index) = self.depth.checked_sub(1) else {
            return;
        };
        let Some(scope) = self.scopes.get_mut(index) else {
            return;
        };
        if scope.written.is_empty() {
            return;
        }

        let object = scope.object.clone();
        let mut written = std::mem::take(&mut scope.written);
        written.retain(|key| {
            if keep.contains(key) {
                return true;
            }
            let _ = object.delete_property_or_throw(key.clone(), &mut self.context);
            false
        });
        self.scopes[index].written = written;
    }

    pub fn set_val(&mut self, key: &str, value: JsValue) -> JsResult<()> {
        self.set_val_js(JsString::from(key), value)
    }

    fn set_val_js(&mut self, key: JsString, value: JsValue) -> JsResult<()> {
        let scope = match self
            .depth
            .checked_sub(1)
            .and_then(|i| self.scopes.get_mut(i))
        {
            Some(scope) => {
                scope.record(key.clone());
                scope.object.clone()
            }
            None => self.context.global_object(),
        };

        scope.set(key, value, false, &mut self.context)?;
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

        let to_js = |names: Vec<String>| {
            names
                .iter()
                .map(|name| JsString::from(name.as_str()))
                .collect::<Vec<_>>()
        };
        let binding = if let Some(slots) =
            simple_slot_names(array_pattern.bindings(), self.context.interner())
        {
            ForBinding::Slots(to_js(slots))
        } else {
            let mut locals = Vec::new();
            collect_binding_names(binding, self.context.interner(), &mut locals);
            ForBinding::Pattern {
                source: pattern.to_string(),
                locals: to_js(locals),
            }
        };
        self.for_binding_cache
            .insert(pattern.to_string(), binding.clone());
        Some(binding)
    }

    pub fn bind_for_slots(&mut self, binding: &ForBinding, slots: &[JsValue]) -> bool {
        match binding {
            ForBinding::Slots(names) => {
                self.reset_scope(names);
                for (idx, name) in names.iter().enumerate() {
                    let value = slots.get(idx).cloned().unwrap_or_else(JsValue::undefined);
                    if self.set_val_js(name.clone(), value).is_err() {
                        return false;
                    }
                }
                true
            }
            ForBinding::Pattern { source, locals } => {
                self.reset_scope(locals);
                let Some(index) = self.depth.checked_sub(1) else {
                    return false;
                };
                // The generated copies bypass `set_val_js`, so record them here.
                for name in locals {
                    self.scopes[index].record(name.clone());
                }

                let scope_ref = js_string_literal(&scope_key(index));
                let slot_array = JsArray::from_iter(slots.iter().cloned(), &mut self.context);
                self.with_temp(slot_array.into(), |engine, temp_ref| {
                    let copies = locals
                        .iter()
                        .map(|name| copy_to_scope(&scope_ref, &name.to_std_string_escaped()))
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

    /// `None` when the value has no JSON form (`undefined`, function, symbol).
    pub fn json_value(&mut self, value: &JsValue) -> Option<JsonValue> {
        value.to_json(&mut self.context).ok().flatten()
    }

    pub fn get_prop(&mut self, obj: &JsObject, key: impl Into<PropertyKey>) -> JsValue {
        obj.get(key, &mut self.context)
            .unwrap_or_else(|_| JsValue::undefined())
    }

    /// `None` if the object is not an array.
    pub fn array_length(&mut self, obj: &JsObject) -> Option<u64> {
        JsArray::from_object(obj.clone())
            .ok()?
            .length(&mut self.context)
            .ok()
    }

    /// Every element of an array object, holes read as `undefined`.
    pub fn array_values(&mut self, obj: &JsObject) -> Vec<JsValue> {
        let Some(length) = self.array_length(obj) else {
            return Vec::new();
        };

        (0..length).map(|idx| self.get_prop(obj, idx)).collect()
    }

    /// `Object.keys(value)` — own enumerable string keys, in insertion order.
    pub fn object_keys(&mut self, value: JsValue) -> Vec<JsValue> {
        self.eval_with_temp_val(value, |temp_ref| {
            format!("Object.keys(globalThis[{temp_ref}])")
        })
        .ok()
        .and_then(|keys| keys.as_object().map(|obj| self.array_values(&obj)))
        .unwrap_or_default()
    }

    /// The values `Symbol.iterator` yields, or `None` when not iterable.
    pub fn iterable_values(&mut self, value: JsValue) -> Option<Vec<JsValue>> {
        let value = self
            .eval_with_temp_val(value, |temp_ref| {
                format!(
                    "let value = globalThis[{temp_ref}]; \
                     let iterator = value == null ? undefined : value[Symbol.iterator]; \
                     typeof iterator === 'function' ? Array.from(value) : null"
                )
            })
            .ok()?;

        match value.variant() {
            JsVariant::Object(obj) if obj.is_array() => Some(self.array_values(&obj)),
            _ => None,
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
        let depth = self.depth;
        let script =
            if let Some(script) = self.eval_cache.get(depth).and_then(|cache| cache.get(code)) {
                script.clone()
            } else {
                let scoped = (0..depth).rev().fold(code.to_string(), |acc, index| {
                    format!(
                        r#"with (globalThis["{key}"]) {{ {acc} }}"#,
                        key = scope_key(index)
                    )
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

        if evaluated.strict_equals(&self.global) {
            Ok(JsValue::null())
        } else {
            Ok(evaluated)
        }
    }

    pub fn eval_setup(&mut self, code: &str) -> JsResult<()> {
        let names = self.parse_setup_bindings(code)?;
        let Some(index) = self.depth.checked_sub(1) else {
            return Ok(());
        };

        let scope_ref = js_string_literal(&scope_key(index));
        let copies = names
            .iter()
            .filter(|name| name.as_str() != "$")
            .map(|name| copy_to_scope(&scope_ref, name))
            .collect::<String>();
        // Wrapped in a function so `var` and (Annex B) function declarations
        // land in its scope instead of leaking onto `globalThis`; the copies
        // still publish them to the scope object for later expressions.
        self.eval(&format!("(function () {{\n{code}\n{copies}\n}})();"))?;

        // Without this a reused scope would carry these into the next iteration.
        for name in names.iter().filter(|name| name.as_str() != "$") {
            self.scopes[index].record(JsString::from(name.as_str()));
        }
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
        self.stringify(&value)
    }

    pub fn eval_fmt(&mut self, code: &str) -> Option<String> {
        let value = self.eval_expr(code).ok()?;
        to_display_string(&value, self)
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

/// Vue's `toDisplayString`, which is what `{{ }}` renders with.
fn to_display_string(value: &JsValue, engine: &mut Engine) -> Option<String> {
    match value.variant() {
        JsVariant::Null | JsVariant::Undefined => None,
        JsVariant::String(text) => Some(text.to_std_string_escaped()),
        JsVariant::Boolean(flag) => Some(flag.to_string()),
        JsVariant::Integer32(number) => Some(number.to_string()),
        // Floats, BigInt, symbols and objects all have JavaScript-specific
        // spellings, so they go through the engine rather than Rust's.
        _ => engine
            .eval_with_temp_val(value.clone(), |temp_ref| {
                format!("{DISPLAY_STRING_JS}(globalThis[{temp_ref}])")
            })
            .ok()?
            .as_string()
            .map(|text| text.to_std_string_escaped()),
    }
}

// The tail of `toDisplayString`, ported as JavaScript so `JSON.stringify` keeps
// its own cycle detection and the replacer reaches nested values.
const DISPLAY_STRING_JS: &str = r#"(function (value) {
    const stringifySymbol = (v, i = '') =>
        typeof v === 'symbol' ? `Symbol(${v.description ?? i})` : v;

    const replacer = (_key, val) => {
        if (val instanceof Map) {
            const entries = {};
            let index = 0;
            for (const [key, item] of val.entries()) {
                entries[stringifySymbol(key, index++) + ' =>'] = item;
            }
            return { [`Map(${val.size})`]: entries };
        }
        if (val instanceof Set) {
            return { [`Set(${val.size})`]: [...val.values()].map((v) => stringifySymbol(v)) };
        }
        if (typeof val === 'symbol') {
            return stringifySymbol(val);
        }
        if (val !== null && typeof val === 'object' && !Array.isArray(val)
            && Object.prototype.toString.call(val) !== '[object Object]') {
            return String(val);
        }
        return val;
    };

    if (typeof value === 'string') return value;
    if (value === null || value === undefined) return '';

    // An array is always JSON. Other objects are too, unless they spell
    // themselves out — a Date, a RegExp, anything with its own toString.
    const asJson = Array.isArray(value)
        || (typeof value === 'object'
            && (value.toString === Object.prototype.toString
                || typeof value.toString !== 'function'));

    return asJson ? JSON.stringify(value, replacer, 2) : String(value);
})"#;

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
