use prevue::render;
use serde_json::json;

#[test]
fn test_script_function_in_mustache() {
    let input = r#"
    <script type="prevue">
        function fullName(user) {
            return `${user.first} ${user.last}`;
        }
    </script>
    <div>{{ fullName(user) }}</div>
    "#;
    let output = render(
        input.to_string(),
        json!({
            "user": { "first": "Ada", "last": "Lovelace" },
        }),
    )
    .unwrap();

    let expected = r#"<html><head>
    </head><body><div>Ada Lovelace</div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_const_arrow_function_in_if() {
    let input = r#"
    <script type="prevue">
        const isAdult = user => user.age >= 18;
    </script>
    <div>
        <p v-if="isAdult(user)">{{ user.name }}</p>
    </div>
    "#;
    let output = render(
        input.to_string(),
        json!({
            "user": { "name": "Alice", "age": 21 },
        }),
    )
    .unwrap();

    let expected = r#"<html><head>
    </head><body><div>
        <p>Alice</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_helpers_in_for_and_bind() {
    let input = r#"
    <script type="prevue">
        const visible = items => items.filter(item => item.visible);
        const label = item => item.name.toUpperCase();
    </script>
    <ul>
        <li v-for="item in visible(items)" :data-name="label(item)">{{ label(item) }}</li>
    </ul>
    "#;
    let output = render(
        input.to_string(),
        json!({
            "items": [
                { "name": "one", "visible": true },
                { "name": "two", "visible": false },
                { "name": "three", "visible": true },
            ],
        }),
    )
    .unwrap();

    let expected = r#"<html><head>
    </head><body><ul>
        <li data-name="ONE">ONE</li>
        <li data-name="THREE">THREE</li>
    </ul>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_execution_order() {
    let input = r#"
    <script type="prevue">
        const base = 2;
    </script>
    <script type="prevue">
        const double = value => value * base;
    </script>
    <p>{{ double(value) }}</p>
    "#;
    let output = render(input.to_string(), json!({ "value": 3 })).unwrap();

    let expected = r#"<html><head>
    </head><body><p>6</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_helpers_are_not_available_before_execution() {
    let input = r#"
    <p>{{ helper() }}</p>
    <script type="prevue">
        const helper = () => 'ready';
    </script>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head></head><body><p></p>
    <p>ready</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_var_and_class_declarations() {
    let input = r#"
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
    "#;
    let output = render(input.to_string(), json!({ "user": { "name": "Alice" } })).unwrap();

    let expected = r#"<html><head>
    </head><body><p>hi, Alice</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_regular_script_is_preserved_and_not_executed() {
    let input = r#"
    <script>
        const helper = () => 'client';
    </script>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head><script>
        const helper = () => 'client';
    </script>
    </head><body><p></p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_inside_pre_is_preserved_and_not_executed() {
    let input = r#"
    <div v-pre>
        <script type="prevue">
            const helper = () => 'pre';
        </script>
        {{ helper() }}
    </div>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head></head><body><div>
        <script type="prevue">
            const helper = () => 'pre';
        </script>
        {{ helper() }}
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_with_pre_is_preserved_and_not_executed() {
    let input = r#"
    <script type="prevue" v-pre>
        const helper = () => 'pre';
    </script>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head><script type="prevue">
        const helper = () => 'pre';
    </script>
    </head><body><p></p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_inside_plain_template_is_inert() {
    let input = r#"
    <template>
        <script type="prevue">
            const helper = () => 'template';
        </script>
    </template>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head><template></template>
    </head><body><p></p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_if_false_does_not_execute() {
    let input = r#"
    <script type="prevue" v-if="false">
        const helper = () => 'ready';
    </script>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head>
    </head><body><p></p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_if_true_executes_and_is_removed() {
    let input = r#"
    <script type="prevue" v-if="true">
        const helper = () => 'ready';
    </script>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({})).unwrap();

    let expected = r#"<html><head>
    </head><body><p>ready</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_for_executes_in_iteration_scope() {
    let input = r#"
    <script type="prevue">
        const seen = [];
    </script>
    <script type="prevue" v-for="item in list">
        seen.push(item);
    </script>
    <p>{{ seen.join(',') }}</p>
    "#;
    let output = render(input.to_string(), json!({ "list": ["a", "b", "c"] })).unwrap();

    let expected = r#"<html><head>
    </head><body><p>a,b,c</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_inside_structural_template_executes_when_reached() {
    let input = r#"
    <template v-if="ready">
        <script type="prevue">
            const helper = () => 'template';
        </script>
    </template>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({ "ready": true })).unwrap();

    let expected = r#"<html><head>
    </head><body><p>template</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_inside_skipped_structural_template_does_not_execute() {
    let input = r#"
    <template v-if="ready">
        <script type="prevue">
            const helper = () => 'template';
        </script>
    </template>
    <p>{{ helper() }}</p>
    "#;
    let output = render(input.to_string(), json!({ "ready": false })).unwrap();

    let expected = r#"<html><head>
    </head><body><p></p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_can_access_data_alias() {
    let input = r#"
    <script type="prevue">
        const first = () => $.items[0];
    </script>
    <p>{{ first() }}</p>
    "#;
    let output = render(input.to_string(), json!({ "items": ["a", "b"] })).unwrap();

    let expected = r#"<html><head>
    </head><body><p>a</p>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_script_syntax_error_returns_error() {
    let input = r#"
    <script type="prevue">
        const =
    </script>
    "#;

    assert!(render(input.to_string(), json!({})).is_err());
}

#[test]
fn test_script_runtime_error_returns_error() {
    let input = r#"
    <script type="prevue">
        throw new Error('boom');
    </script>
    "#;

    assert!(render(input.to_string(), json!({})).is_err());
}
