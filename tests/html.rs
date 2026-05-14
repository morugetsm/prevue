mod helper;

use helper::{assert_render_body_eq, assert_render_eq};
use prevue::{Directive, Error, render};
use serde_json::json;

#[test]
fn html_basic() {
    assert_render_body_eq!(
        r#"<div>
        <p v-html="html"></p>
    </div>"#,
        json!({
            "html": "<strong>Hello</strong>",
        }),
        r#"<div>
        <p><strong>Hello</strong></p>
    </div>"#,
    );
}

#[test]
fn html_overrides_inner_content() {
    assert_render_body_eq!(
        r#"<div>
        <p v-html="html">fallback</p>
    </div>"#,
        json!({
            "html": "<strong>Hello</strong>",
        }),
        r#"<div>
        <p><strong>Hello</strong></p>
    </div>"#,
    );
}

#[test]
fn html_overrides_compact_inner_content() {
    assert_render_body_eq!(
        r#"<div><p v-html="html">fallback</p></div>"#,
        json!({
            "html": "<strong>Hello</strong>",
        }),
        r#"<div><p><strong>Hello</strong></p></div>"#,
    );
}

#[test]
fn html_inserted_mustache_is_inert() {
    assert_render_body_eq!(
        r#"<div>
        <p v-html="with_mustache"></p>
        <p>{{ name }}</p>
    </div>"#,
        json!({
            "name": "Alice",
            "with_mustache": "<span>{{ name }}</span>",
        }),
        r#"<div>
        <p><span>{{ name }}</span></p>
        <p>Alice</p>
    </div>"#,
    );
}

#[test]
fn html_inserted_directives_and_scripts_are_inert() {
    assert_render_body_eq!(
        r#"<div>
        <div v-html="with_directives"></div>
        <p>{{ typeof leaked }}</p>
    </div>"#,
        json!({
            "with_directives": "<span v-if=\"false\">Hidden</span><script type=\"prevue\">const leaked = true;</script>",
        }),
        r#"<div>
        <div><span v-if="false">Hidden</span><script type="prevue">const leaked = true;</script></div>
        <p>undefined</p>
    </div>"#,
    );
}

#[test]
fn html_value_types() {
    assert_render_body_eq!(
        r#"<div>
        <p v-html="num"></p>
        <p v-html="bool_val"></p>
        <p v-html="'plain'"></p>
    </div>"#,
        json!({
            "bool_val": true,
            "num": 42,
        }),
        r#"<div>
        <p>42</p>
        <p>true</p>
        <p>plain</p>
    </div>"#,
    );
}

#[test]
fn html_null_keeps_children() {
    assert_render_body_eq!(
        r#"<div>
        <p v-html="null_val"><em>fallback</em></p>
    </div>"#,
        json!({
            "null_val": null,
        }),
        r#"<div>
        <p><em>fallback</em></p>
    </div>"#,
    );
}

#[test]
fn html_missing_keeps_children() {
    assert_render_body_eq!(
        r#"<div>
        <p v-html="missing"><em>{{ name }}</em></p>
    </div>"#,
        json!({
            "name": "Alice",
        }),
        r#"<div>
        <p><em>Alice</em></p>
    </div>"#,
    );
}

#[test]
fn html_for_uses_loop_scope() {
    assert_render_body_eq!(
        r#"<ul>
        <li v-for="item in items" v-html="item"></li>
    </ul>"#,
        json!({
            "items": ["<b>A</b>", "<i>B</i>"],
        }),
        r#"<ul>
        <li><b>A</b></li>
        <li><i>B</i></li>
    </ul>"#,
    );
}

#[test]
fn html_inside_pre_is_preserved() {
    assert_render_body_eq!(
        r#"<div v-pre>
        <p v-html="html">{{ name }}</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p v-html="html">{{ name }}</p>
    </div>"#,
    );
}

#[test]
fn html_if_false_does_not_eval() {
    assert_render_eq!(
        r#"<script type="prevue">
        var count = 0;
        function inc() {
            count = count + 1;
            return '<b>x</b>';
        }
    </script>
    <div>
        <p v-if="false" v-html="inc()"></p>
        <span>{{ count }}</span>
    </div>"#,
        json!({}),
        r#"<html><head>
    </head><body><div>
        <span>0</span>
    </div></body></html>"#,
    );
}

#[test]
fn html_text_conflict_errors() {
    let input = r#"
    <div>
        <p v-html="html" v-text="name"></p>
    </div>
    "#;

    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(err, Error::ConflictingDirectives { directives }
        if directives == vec![Directive::Text, Directive::Html]));
}
