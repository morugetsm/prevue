mod helper;

use helper::assert_render_body_eq;
use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::json;

// === Basic Behavior ===

#[test]
fn if_basic() {
    assert_render_body_eq!(
        r#"<div>
        <p>Hello, world!</p>
        <div v-if="true">TRUE</div>
        <div v-if="false">FALSE</div>
        <div v-if="list">LIST</div>
    </div>"#,
        json!({
            "list": [1, 2, 3],
        }),
        r#"<div>
        <p>Hello, world!</p>
        <div>TRUE</div>
        <div>LIST</div>
    </div>"#,
    );
}

#[test]
fn if_truthy_cast() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="0">0 is false</div>
        <div v-if="list">array is true</div>
    </div>"#,
        json!({
            "list": [1, 2, 3],
        }),
        r#"<div>
        <div>array is true</div>
    </div>"#,
    );
}

#[test]
fn if_edge_cases() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="''">empty string</div>
        <div v-if="null">null</div>
        <div v-if="undefined">undefined</div>
        <div v-if="NaN">NaN</div>
        <div v-if="Infinity">Infinity</div>
        <div v-if="notexist">notexist</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>Infinity</div>
    </div>"#,
    );
}

// === Expression ===

#[test]
fn if_expression() {
    assert_render_body_eq!(
        r#"<div>
        <p v-if="user.age >= 18">{{ user.name }} ({{ user.age }})</p>
        <p v-if="user.age < 18">minor</p>
    </div>"#,
        json!({
            "user": {
                "name": "Alice",
                "age": 21,
            },
        }),
        r#"<div>
        <p>Alice (21)</p>
    </div>"#,
    );
}

// === Same-Element Directives ===

#[test]
fn if_else_same_element() {
    let input = r#"
    <div>
        <div v-if="true" v-else>first</div>
        <div v-else v-if="true">second</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
            if directives == vec![Directive::If, Directive::Else]));
}

#[test]
fn if_else_if_same_element() {
    let input = r#"
    <div>
        <div v-if="true" v-else-if="false">first</div>
        <div v-else-if="false" v-if="true">second</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
            if directives == vec![Directive::If, Directive::ElseIf]));
}

#[test]
fn if_empty_expression_error() {
    let input = r#"
    <div>
        <div v-if="">empty</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::If, kind: DirectiveErrorKind::MissingExpression, expression: Some(expr) }
            if expr.is_empty())
    );
}

// === Priority over v-for ===

#[test]
fn if_priority_over_for() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="true" v-for="item in list">IF</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>IF</div>
    </div>"#,
    );
}

#[test]
fn if_for_scope_unavailable() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in list" v-if="item > 1">IF{{ item }}</div>
    </div>"#,
        json!({
            "list": [1, 2, 3],
        }),
        r#"<div>
    </div>"#,
    );
}
