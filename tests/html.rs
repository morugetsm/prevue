use prevue::{Directive, Error, render};
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "html": "<strong>Hello</strong>",
        "with_mustache": "<span>{{ name }}</span>",
        "with_directives": "<span v-if=\"false\">Hidden</span><script type=\"prevue\">const leaked = true;</script>",
        "name": "Alice",
        "num": 42,
        "bool_val": true,
        "null_val": null,
        "items": ["<b>A</b>", "<i>B</i>"],
    })
}

#[test]
fn html_basic() {
    let input = r#"
    <div>
        <p v-html="html"></p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p><strong>Hello</strong></p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_overrides_inner_content() {
    let input = r#"
    <div>
        <p v-html="html">fallback</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p><strong>Hello</strong></p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_overrides_compact_inner_content() {
    let input = r#"<div><p v-html="html">fallback</p></div>"#;
    let output = render(input, data()).unwrap();

    let expected =
        r#"<html><head></head><body><div><p><strong>Hello</strong></p></div></body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_inserted_mustache_is_inert() {
    let input = r#"
    <div>
        <p v-html="with_mustache"></p>
        <p>{{ name }}</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p><span>{{ name }}</span></p>
        <p>Alice</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_inserted_directives_and_scripts_are_inert() {
    let input = r#"
    <div>
        <div v-html="with_directives"></div>
        <p>{{ typeof leaked }}</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div><span v-if="false">Hidden</span><script type="prevue">const leaked = true;</script></div>
        <p>undefined</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_value_types() {
    let input = r#"
    <div>
        <p v-html="num"></p>
        <p v-html="bool_val"></p>
        <p v-html="'plain'"></p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>42</p>
        <p>true</p>
        <p>plain</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_null_keeps_children() {
    let input = r#"
    <div>
        <p v-html="null_val"><em>fallback</em></p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p><em>fallback</em></p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_missing_keeps_children() {
    let input = r#"
    <div>
        <p v-html="missing"><em>{{ name }}</em></p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p><em>Alice</em></p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_if_false_does_not_eval() {
    let input = r#"
    <script type="prevue">
        var count = 0;
        function inc() {
            count = count + 1;
            return '<b>x</b>';
        }
    </script>
    <div>
        <p v-if="false" v-html="inc()"></p>
        <span>{{ count }}</span>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head>
    </head><body><div>
        <span>0</span>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_for_uses_loop_scope() {
    let input = r#"
    <ul>
        <li v-for="item in items" v-html="item"></li>
    </ul>
    "#;
    let output = render(input, data()).unwrap();

    assert!(output.contains("<li><b>A</b></li>"));
    assert!(output.contains("<li><i>B</i></li>"));
}

#[test]
fn html_inside_pre_is_preserved() {
    let input = r#"
    <div v-pre>
        <p v-html="html">{{ name }}</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p v-html="html">{{ name }}</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn html_text_conflict_errors() {
    let input = r#"
    <div>
        <p v-html="html" v-text="name"></p>
    </div>
    "#;

    let err = render(input, data()).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
        if directives == vec![Directive::Text, Directive::Html]));
}
