use prevue::{Error, render};
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
fn data_serialization_error_returns_error() {
    let err = render("<p>{{ message }}</p>", BrokenData).unwrap_err();
    assert!(matches!(err, Error::DataSerialize { .. }));
}

#[test]
fn conflict_display_uses_template_directive_names() {
    let err = render(r#"<p v-html="html" v-text="text"></p>"#, json!({})).unwrap_err();
    assert_eq!(err.to_string(), "conflicting directives: v-text, v-html");

    let err = render(r#"<p v-if="true" v-else>text</p>"#, json!({})).unwrap_err();
    assert_eq!(err.to_string(), "conflicting directives: v-if, v-else");
}

#[test]
fn invalid_directive_display_includes_expression() {
    let err = render(
        r#"
        <p v-if="false">if</p>
        <p v-else="ok">else</p>
        "#,
        json!({}),
    )
    .unwrap_err();
    assert_eq!(
        err.to_string(),
        r#"invalid v-else: unexpected expression "ok""#
    );

    let err = render(r#"<p v-for="Hello, world!">item</p>"#, json!({})).unwrap_err();
    assert_eq!(
        err.to_string(),
        r#"invalid v-for: invalid expression "Hello, world!""#
    );
}

#[test]
fn missing_directive_expression_display_stays_compact() {
    let err = render(r#"<p v-if="">empty</p>"#, json!({})).unwrap_err();
    assert_eq!(err.to_string(), "invalid v-if: missing expression");
}

#[test]
fn setup_script_display_mentions_prevue_script() {
    let err = render(
        r#"
        <script type="prevue">
            throw new Error('boom');
        </script>
        "#,
        json!({}),
    )
    .unwrap_err();

    assert!(err.to_string().contains(r#"<script type="prevue">"#));
}

#[test]
fn data_field_display_identifies_field() {
    let err = Error::DataToJs {
        field: Some("user".to_string()),
        message: "boom".to_string(),
    };
    assert_eq!(
        err.to_string(),
        r#"failed to convert render data field "user" to JavaScript: boom"#
    );

    let err = Error::DataInject {
        field: Some("user".to_string()),
        message: "boom".to_string(),
    };
    assert_eq!(
        err.to_string(),
        r#"failed to inject render data field "user": boom"#
    );
}
