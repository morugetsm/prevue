mod helper;

use helper::assert_render_eq;
use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::json;

// === Setup Script Helpers ===

#[test]
fn script_function_in_mustache() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        function fullName(user) {
            return `${user.first} ${user.last}`;
        }
    </script>
    <div>{{ fullName(user) }}</div>
    "#,
        json!({
            "user": { "first": "Ada", "last": "Lovelace" },
        }),
        r#"<html><head>
    </head><body><div>Ada Lovelace</div>
    </body></html>"#,
    );
}

#[test]
fn script_arrow_in_if() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        const isAdult = user => user.age >= 18;
    </script>
    <div>
        <p v-if="isAdult(user)">{{ user.name }}</p>
    </div>
    "#,
        json!({
            "user": { "name": "Alice", "age": 21 },
        }),
        r#"<html><head>
    </head><body><div>
        <p>Alice</p>
    </div>
    </body></html>"#,
    );
}

#[test]
fn script_helpers_for_bind() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        const visible = items => items.filter(item => item.visible);
        const label = item => item.name.toUpperCase();
    </script>
    <ul>
        <li v-for="item in visible(items)" :data-name="label(item)">{{ label(item) }}</li>
    </ul>
    "#,
        json!({
            "items": [
                { "name": "one", "visible": true },
                { "name": "two", "visible": false },
                { "name": "three", "visible": true },
            ],
        }),
        r#"<html><head>
    </head><body><ul>
        <li data-name="ONE">ONE</li>
        <li data-name="THREE">THREE</li>
    </ul>
    </body></html>"#,
    );
}

// === Execution Order & Scope ===

#[test]
fn script_execution_order() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        const base = 2;
    </script>
    <script type="prevue">
        const double = value => value * base;
    </script>
    <p>{{ double(value) }}</p>
    "#,
        json!({ "value": 3 }),
        r#"<html><head>
    </head><body><p>6</p>
    </body></html>"#,
    );
}

#[test]
fn script_helpers_unavailable_before_exec() {
    assert_render_eq!(
        r#"
    <p>{{ helper() }}</p>
    <script type="prevue">
        const helper = () => 'ready';
    </script>
    <p>{{ helper() }}</p>
    "#,
        json!({}),
        r#"<html><head></head><body><p></p>
    <p>ready</p>
    </body></html>"#,
    );
}

#[test]
fn script_var_class_declarations() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        var prefix = 'hi';
        class Greeter {
            constructor(name) {
                this.name = name;
            }
            greet() {
                return `${prefix}, ${this.name}`;
            }
        }
    </script>
    <p>{{ new Greeter(user.name).greet() }}</p>
    "#,
        json!({ "user": { "name": "Alice" } }),
        r#"<html><head>
    </head><body><p>hi, Alice</p>
    </body></html>"#,
    );
}

// === Inert Script & Style ===

#[test]
fn regular_script_not_executed() {
    assert_render_eq!(
        r#"
    <script>
        const helper = () => 'client';
    </script>
    <p>{{ helper() }}</p>
    "#,
        json!({}),
        r#"<html><head><script>
        const helper = () => 'client';
    </script>
    </head><body><p></p>
    </body></html>"#,
    );
}

#[test]
fn regular_script_style_inert() {
    let input = r#"
    <script>
        const template = "{{ '<br />' }}";
    </script>
    <style>
        .x::before { content: "{{ '<br />' }}"; }
    </style>
    <p>{{ missing }}</p>
    "#;
    let output = render(input, json!({})).unwrap();

    assert!(output.contains(r#"const template = "{{ '<br />' }}";"#));
    assert!(output.contains(r#".x::before { content: "{{ '<br />' }}"; }"#));
    assert!(output.contains("<p></p>"));
}

// === v-pre & Template Boundaries ===

#[test]
fn script_inside_pre_inert() {
    assert_render_eq!(
        r#"
    <div v-pre>
        <script type="prevue">
            const helper = () => 'pre';
        </script>
        {{ helper() }}
    </div>
    "#,
        json!({}),
        r#"<html><head></head><body><div>
        <script type="prevue">
            const helper = () => 'pre';
        </script>
        {{ helper() }}
    </div>
    </body></html>"#,
    );
}

#[test]
fn script_with_pre_inert() {
    assert_render_eq!(
        r#"
    <script type="prevue" v-pre>
        const helper = () => 'pre';
    </script>
    <p>{{ helper() }}</p>
    "#,
        json!({}),
        r#"<html><head><script type="prevue">
        const helper = () => 'pre';
    </script>
    </head><body><p></p>
    </body></html>"#,
    );
}

#[test]
fn script_plain_template_inert() {
    assert_render_eq!(
        r#"
    <template>
        <script type="prevue">
            const helper = () => 'template';
        </script>
    </template>
    <p>{{ helper() }}</p>
    "#,
        json!({}),
        r#"<html><head><template></template>
    </head><body><p></p>
    </body></html>"#,
    );
}

// === Structural Directives ===

#[test]
fn script_if_false_inert() {
    assert_render_eq!(
        r#"
    <script type="prevue" v-if="false">
        const helper = () => 'ready';
    </script>
    <p>{{ helper() }}</p>
    "#,
        json!({}),
        r#"<html><head>
    </head><body><p></p>
    </body></html>"#,
    );
}

#[test]
fn script_if_true_removed() {
    assert_render_eq!(
        r#"
    <script type="prevue" v-if="true">
        const helper = () => 'ready';
    </script>
    <p>{{ helper() }}</p>
    "#,
        json!({}),
        r#"<html><head>
    </head><body><p>ready</p>
    </body></html>"#,
    );
}

#[test]
fn script_for_iteration_scope() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        const seen = [];
    </script>
    <script type="prevue" v-for="item in list">
        seen.push(item);
    </script>
    <p>{{ seen.join(',') }}</p>
    "#,
        json!({ "list": ["a", "b", "c"] }),
        r#"<html><head>
    </head><body><p>a,b,c</p>
    </body></html>"#,
    );
}

#[test]
fn script_structural_template_executes() {
    assert_render_eq!(
        r#"
    <template v-if="ready">
        <script type="prevue">
            const helper = () => 'template';
        </script>
    </template>
    <p>{{ helper() }}</p>
    "#,
        json!({ "ready": true }),
        r#"<html><head>
    </head><body><p>template</p>
    </body></html>"#,
    );
}

#[test]
fn script_skipped_template_inert() {
    assert_render_eq!(
        r#"
    <template v-if="ready">
        <script type="prevue">
            const helper = () => 'template';
        </script>
    </template>
    <p>{{ helper() }}</p>
    "#,
        json!({ "ready": false }),
        r#"<html><head>
    </head><body><p></p>
    </body></html>"#,
    );
}

// === Data Alias ===

#[test]
fn script_data_alias_access() {
    assert_render_eq!(
        r#"
    <script type="prevue">
        const first = () => $.items[0];
    </script>
    <p>{{ first() }}</p>
    "#,
        json!({ "items": ["a", "b"] }),
        r#"<html><head>
    </head><body><p>a</p>
    </body></html>"#,
    );
}

// === Errors ===

#[test]
fn script_syntax_error() {
    let input = r#"
    <script type="prevue">
        const =
    </script>
    "#;

    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(err, Error::SetupScript { .. }));
}

#[test]
fn script_runtime_error() {
    let input = r#"
    <script type="prevue">
        throw new Error('boom');
    </script>
    "#;

    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(err, Error::SetupScript { .. }));
}

#[test]
fn script_orphan_else_before_exec() {
    let input = r#"
    <script type="prevue" v-else>
        const helper = () => 'should not run';
    </script>
    <p>{{ helper() }}</p>
    "#;

    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Else,
            kind: DirectiveErrorKind::MissingAdjacentConditional,
            expression: None
        }
    ));
}
