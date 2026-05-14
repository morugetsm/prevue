mod helper;

use helper::assert_render_body_eq;
use serde_json::json;

// === Basic Behavior ===

#[test]
fn mustache_eval() {
    assert_render_body_eq!(
        r#"<div>
            {{ 1 + 1 }}
        </div>"#,
        json!({}),
        r#"<div>
            2
        </div>"#,
    );
}

#[test]
fn mustache_multiple() {
    assert_render_body_eq!(
        r#"<div>
            {{ 1 + 1 }} and {{ 2 + 2 }}
        </div>"#,
        json!({}),
        r#"<div>
            2 and 4
        </div>"#,
    );
}

#[test]
fn mustache_multiline() {
    assert_render_body_eq!(
        r#"<div>
            {{ 
                1 + 
                1 
            }}
        </div>"#,
        json!({}),
        r#"<div>
            2
        </div>"#,
    );
}

#[test]
fn mustache_string_can_contain_closing_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            {{ "}}" }}
        </div>"#,
        json!({}),
        r#"<div>
            }}
        </div>"#,
    );
}

#[test]
fn mustache_string_variants_can_contain_closing_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ "{{" }}</p>
            <p>{{ '}}' }}</p>
            <p>{{ `}}` }}</p>
            <p>{{ "{{}}" }}</p>
            <p>{{ "escaped \" }} still string" }}</p>
            <p>{{ "}}" }} and {{ 2 + 2 }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>{{</p>
            <p>}}</p>
            <p>}}</p>
            <p>{{}}</p>
            <p>escaped " }} still string</p>
            <p>}} and 4</p>
        </div>"#,
    );
}

#[test]
fn mustache_comments_can_contain_closing_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{
                // {{ and }} inside a line comment
                "line"
            }}</p>
            <p>{{ /* {{ and }} inside a block comment */ "block" }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>line</p>
            <p>block</p>
        </div>"#,
    );
}

#[test]
fn mustache_comments_can_contain_opening_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{
                // {{ inside a line comment
                // }} inside a line comment
                "line"
            }}</p>
            <p>{{ /* {{ inside a block comment */ "block" }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>line</p>
            <p>block</p>
        </div>"#,
    );
}

#[test]
fn mustache_regex_literals_can_contain_closing_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ /}}/.test("}}") }}</p>
            <p>{{ "a}}b".replace(/}}/g, "x") }}</p>
            <p>{{ /[}}]+/.test("}") }}</p>
            <p>{{ /a\/b}}/gi.test("a/b}}") }}</p>
            <p>{{ /<br \/>/.test("<br />") }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>true</p>
            <p>axb</p>
            <p>true</p>
            <p>true</p>
            <p>true</p>
        </div>"#,
    );
}

#[test]
fn mustache_regex_literals_can_start_after_expression_boundaries() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ true ? /}}/.test("}}") : false }}</p>
            <p>{{ let matched = /}}/.test("}}"); matched }}</p>
            <p>{{ (() => /}}/.test("}}"))() }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>true</p>
            <p>true</p>
            <p>true</p>
        </div>"#,
    );
}

#[test]
fn mustache_division_still_works_with_regex_scanner() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ total / count }}</p>
            <p>{{ (total / count) / 2 }}</p>
            <p>{{ let count = 4; count++ / 2 }}</p>
            <p>{{ value.return / 2 }}</p>
        </div>"#,
        json!({
            "total": 8,
            "count": 2,
            "value": {
                "return": 8
            }
        }),
        r#"<div>
            <p>4</p>
            <p>2</p>
            <p>2</p>
            <p>4</p>
        </div>"#,
    );
}

#[test]
fn mustache_html_like_string_is_rendered_as_text() {
    assert_render_body_eq!(
        r#"<div>{{ content.split('\n').join('<br />') }}</div>"#,
        json!({ "content": "first\nsecond" }),
        r#"<div>first&lt;br /&gt;second</div>"#,
    );
}

#[test]
fn mustache_html_like_literal_is_rendered_as_text() {
    assert_render_body_eq!(
        r#"<div>{{ '<span>text</span>' }}</div>"#,
        json!({}),
        r#"<div>&lt;span&gt;text&lt;/span&gt;</div>"#,
    );
}

#[test]
fn mustache_html_like_literal_with_multiple_interpolations() {
    assert_render_body_eq!(
        r#"<div>{{ '<span>text</span>' }} and {{ 2 + 2 }}</div>"#,
        json!({}),
        r#"<div>&lt;span&gt;text&lt;/span&gt; and 4</div>"#,
    );
}

#[test]
fn mustache_in_static_attribute_is_not_interpolated() {
    assert_render_body_eq!(
        r#"<div title="{{ '<br />' }}">{{ 1 + 1 }}</div>"#,
        json!({}),
        r#"<div title="{{ '&lt;br /&gt;' }}">2</div>"#,
    );
}

#[test]
fn html_like_static_attribute_value_is_preserved() {
    assert_render_body_eq!(
        r#"<div data-html="<br />">{{ 1 + 1 }}</div>"#,
        json!({}),
        r#"<div data-html="&lt;br /&gt;">2</div>"#,
    );
}

#[test]
fn mustache_unclosed() {
    // Unclosed mustache is left untouched.
    assert_render_body_eq!(
        r#"<div>
            {{ unclosed }
        </div>"#,
        json!({}),
        r#"<div>
            {{ unclosed }
        </div>"#,
    );
}

#[test]
fn mustache_empty() {
    // Empty mustache evaluates to empty or undefined
    assert_render_body_eq!(
        r#"<div>
            [{{ }}]
        </div>"#,
        json!({}),
        r#"<div>
            []
        </div>"#,
    );
}

// === Value Types ===

#[test]
fn mustache_array() {
    assert_render_body_eq!(
        r#"<div>
            <p>Hello, world!</p>
            <div>{{ list }}</div>
        </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
            <p>Hello, world!</p>
            <div>[ 1, 2, 3 ]</div>
        </div>"#,
    );
}

#[test]
fn mustache_object() {
    assert_render_body_eq!(
        r#"<div>
            <p>Hello, world!</p>
            <div>{{ user }}</div>
            <div>{{ user.name }}</div>
            <div>{{ user.age }}</div>
        </div>"#,
        json!({
            "user": {
                "name": "Alice",
                "age": 21
            }
        }),
        r#"<div>
            <p>Hello, world!</p>
            <div>{ "name": "Alice", "age": 21 }</div>
            <div>Alice</div>
            <div>21</div>
        </div>"#,
    );
}

// === Data Alias ===

#[test]
fn data_alias_mustache() {
    assert_render_body_eq!(
        r#"<div>
            <div>{{ user.name }}</div>
            <div>{{ $.user.name }}</div>
        </div>"#,
        json!({
            "user": {
                "name": "Alice"
            }
        }),
        r#"<div>
            <div>Alice</div>
            <div>Alice</div>
        </div>"#,
    );
}

#[test]
fn data_alias_directives() {
    assert_render_body_eq!(
        r#"<div>
            <p v-if="$.user.age >= 18">{{ $.user.name }}</p>
            <ul>
                <li v-for="item in $.list" :data-id="$.user.name">{{ item }}</li>
            </ul>
        </div>"#,
        json!({
            "list": [1, 2, 3],
            "user": {
                "name": "Alice",
                "age": 21
            }
        }),
        r#"<div>
            <p>Alice</p>
            <ul>
                <li data-id="Alice">1</li>
                <li data-id="Alice">2</li>
                <li data-id="Alice">3</li>
            </ul>
        </div>"#,
    );
}

#[test]
fn data_alias_non_object() {
    assert_render_body_eq!(
        r#"<div>{{ $ }}</div>"#,
        json!(["a", "b"]),
        r#"<div>[ "a", "b" ]</div>"#,
    );
}

#[test]
fn data_alias_reserved_collision() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ $.user.name }}</p>
            <p>{{ $["$"] }}</p>
        </div>"#,
        json!({
            "$": "custom",
            "user": {
                "name": "Alice",
            },
        }),
        r#"<div>
            <p>Alice</p>
            <p>custom</p>
        </div>"#,
    );
}

#[test]
fn mustache_falsy() {
    assert_render_body_eq!(
        r#"<div>
            <div>{{ false }}</div>
            <div>{{ null }}</div>
            <div>{{ undefined }}</div>
            <div>{{ 0 }}</div>
            <div>{{ "" }}</div>
        </div>"#,
        json!({}),
        r#"<div>
            <div>false</div>
            <div></div>
            <div></div>
            <div>0</div>
            <div></div>
        </div>"#,
    );
}

// === Statements ===

#[test]
fn mustache_statement() {
    // unlike Vue, prevue currently allows both expressions and statements (e.g., `{{ let x = 1; x + 1 }}`)
    assert_render_body_eq!(
        r#"<div>
            {{ let exist = true; exist }}
        </div>"#,
        json!({}),
        r#"<div>
            true
        </div>"#,
    );
}

#[test]
fn mustache_error() {
    // an expression that throws an error (e.g. ReferenceError) should fallback to an empty string safely
    assert_render_body_eq!(
        r#"<div>
            [{{ does_not_exist }}]
            [{{ foo.bar.baz }}]
        </div>"#,
        json!({}),
        r#"<div>
            []
            []
        </div>"#,
    );
}

// === Scope & Isolation ===

#[test]
fn mustache_this() {
    // can't serialize this
    assert_render_body_eq!(
        r#"<div>
            {{ this }}
        </div>"#,
        json!({}),
        r#"<div>
            
        </div>"#,
    );
}

#[test]
fn mustache_comment() {
    // JavaScript comments inside mustache are valid
    assert_render_body_eq!(
        r#"<div>
            {{ 
                // single line comment
                1 + 1 
                /* multi
                   line
                   comment */
                + 1
            }}
        </div>"#,
        json!({}),
        r#"<div>
            3
        </div>"#,
    );
}

#[test]
fn mustache_isolation() {
    assert_render_body_eq!(
        r#"<div>
            <h1>{{ let x = 1; x }}</h1>
            <h2>{{ x }}</h2>
        </div>"#,
        json!({}),
        r#"<div>
            <h1>1</h1>
            <h2></h2>
        </div>"#,
    );
}
