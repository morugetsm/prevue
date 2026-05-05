use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "list": [1, 2, 3],
        "user": {
            "name": "Alice",
            "age": 21
        },
    })
}

// === Basic Behavior ===

#[test]
fn test_if_basic() {
    let input = r#"
    <div>
        <p>Hello, world!</p>
        <div v-if="true">TRUE</div>
        <div v-if="false">FALSE</div>
        <div v-if="list">LIST</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <div>TRUE</div>
        <div>LIST</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_truthy_cast() {
    // JS truthiness: 0 is false, non-empty array is true
    let input = r#"
    <div>
        <div v-if="0">0 is false</div>
        <div v-if="list">array is true</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>array is true</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_edge_cases() {
    // empty string literal, null, undefined, NaN are falsy; Infinity is truthy
    let input = r#"
    <div>
        <div v-if="''">empty string</div>
        <div v-if="null">null</div>
        <div v-if="undefined">undefined</div>
        <div v-if="NaN">NaN</div>
        <div v-if="Infinity">Infinity</div>
        <div v-if="notexist">notexist</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>Infinity</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_if_empty_string_literal_is_falsy() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="''">ELSE-IF</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Expression ===

#[test]
fn test_if_expression() {
    // v-if evaluates arbitrary JS expressions against data
    let input = r#"
    <div>
        <p v-if="user.age >= 18">{{ user.name }} ({{ user.age }})</p>
        <p v-if="user.age < 18">minor</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Alice (21)</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Same-Element Directives ===

#[test]
fn test_if_with_else_on_same_element() {
    let input = r#"
    <div>
        <div v-if="true" v-else>first</div>
        <div v-else v-if="true">second</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
            if directives == vec![Directive::If, Directive::Else]));
}

#[test]
fn test_if_with_else_if_on_same_element() {
    let input = r#"
    <div>
        <div v-if="true" v-else-if="false">first</div>
        <div v-else-if="false" v-if="true">second</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
            if directives == vec![Directive::If, Directive::ElseIf]));
}

#[test]
fn test_if_empty_expression_error() {
    let input = r#"
    <div>
        <div v-if="">empty</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::If, kind: DirectiveErrorKind::MissingExpression, expression: Some(expr) }
            if expr.is_empty())
    );
}

#[test]
fn test_else_if_empty_expression_error() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="">empty</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::ElseIf, kind: DirectiveErrorKind::MissingExpression, expression: Some(expr) }
            if expr.is_empty())
    );
}

#[test]
fn test_else_with_expression_error() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else="ok">ELSE</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::Else, kind: DirectiveErrorKind::UnexpectedExpression, expression: Some(expr) }
            if expr == "ok")
    );
}

// === Priority over v-for ===

#[test]
fn test_if_takes_priority_over_for() {
    // v-if takes precedence over v-for — element renders once, not per iteration
    let input = r#"
    <div>
        <div v-if="true" v-for="item in list">IF</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_for_scope_unavailable() {
    // v-if is evaluated before v-for regardless of attribute order,
    // so the loop variable is not available to v-if
    let input = r#"
    <div>
        <div v-for="item in list" v-if="item > 1">IF{{ item }}</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
