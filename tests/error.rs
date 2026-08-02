use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde::{Serialize, Serializer};
use serde_json::json;

struct BrokenData;

impl Serialize for BrokenData {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom("broken data"))
    }
}

#[test]
fn data_serialization_error() {
    let err = render("<p>{{ message }}</p>", BrokenData).unwrap_err();
    assert!(matches!(err, Error::DataSerialize { .. }));
}

#[test]
fn content_directive_conflict() {
    let err = render(r#"<p v-html="html" v-text="text"></p>"#, json!({})).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
        if directives == vec![Directive::Text, Directive::Html]));
}

#[test]
fn unknown_directive_error() {
    for template in [r#"<div v-fi="a">x</div>"#, "<div v-els>x</div>", "<div v->x</div>"] {
        let err = render(template, json!({})).unwrap_err();
        assert!(matches!(err, Error::UnknownDirective { .. }), "{template}");
    }
}

#[test]
fn unknown_directive_names_the_attribute() {
    let err = render(r#"<div v-els="a">x</div>"#, json!({})).unwrap_err();
    assert!(matches!(err, Error::UnknownDirective { name } if name == "v-els"));
}

#[test]
fn branch_directive_conflict() {
    let err = render(r#"<p v-if="true" v-else>text</p>"#, json!({})).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
        if directives == vec![Directive::If, Directive::Else]));
}

#[test]
fn else_expression_error() {
    let err = render(
        r#"
        <p v-if="false">if</p>
        <p v-else="ok">else</p>
        "#,
        json!({}),
    )
    .unwrap_err();
    assert!(matches!(err, Error::InvalidDirective {
            directive: Directive::Else,
            kind: DirectiveErrorKind::UnexpectedExpression,
            expression: Some(expr),
        } if expr == "ok"));
}

#[test]
fn for_expression_error() {
    let err = render(r#"<p v-for="Hello, world!">item</p>"#, json!({})).unwrap_err();
    assert!(matches!(err, Error::InvalidDirective {
            directive: Directive::For,
            kind: DirectiveErrorKind::InvalidExpression,
            expression: Some(expr),
        } if expr == "Hello, world!"));
}

#[test]
fn missing_directive_expression() {
    let err = render(r#"<p v-if="">empty</p>"#, json!({})).unwrap_err();
    assert!(matches!(err, Error::InvalidDirective {
            directive: Directive::If,
            kind: DirectiveErrorKind::MissingExpression,
            expression: Some(expr),
        } if expr.is_empty()));
}

#[test]
fn setup_script_error() {
    let err = render(
        r#"
        <script type="prevue">
            throw new Error('boom');
        </script>
        "#,
        json!({}),
    )
    .unwrap_err();

    assert!(matches!(err, Error::SetupScript { .. }));
}
