mod helper;

use helper::assert_render_body_eq;
use prevue::render;
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
fn mustache_string_closing_delimiter() {
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
fn mustache_string_variants_delimiter() {
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
fn mustache_template_interpolation_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ `before ${ "}}" } after` }}</p>
            <p>{{ `object ${ { value: "}}" }.value } done` }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>before }} after</p>
            <p>object }} done</p>
        </div>"#,
    );
}

#[test]
fn mustache_nested_template_delimiter() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ `outer ${ `inner }}` }` }}</p>
            <p>{{ `outer ${ `${ "}}" }` }` }}</p>
        </div>"#,
        json!({}),
        r#"<div>
            <p>outer inner }}</p>
            <p>outer }}</p>
        </div>"#,
    );
}

#[test]
fn mustache_comments_closing_delimiter() {
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
fn mustache_comments_opening_delimiter() {
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
fn mustache_regex_closing_delimiter() {
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
fn mustache_regex_after_expr_boundary() {
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
fn mustache_division_regex_scanner() {
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
fn mustache_html_string_text() {
    assert_render_body_eq!(
        r#"<div>{{ content.split('\n').join('<br />') }}</div>"#,
        json!({ "content": "first\nsecond" }),
        r#"<div>first&lt;br /&gt;second</div>"#,
    );
}

#[test]
fn mustache_html_literal_text() {
    assert_render_body_eq!(
        r#"<div>{{ '<span>text</span>' }}</div>"#,
        json!({}),
        r#"<div>&lt;span&gt;text&lt;/span&gt;</div>"#,
    );
}

#[test]
fn mustache_html_literal_interpolations() {
    assert_render_body_eq!(
        r#"<div>{{ '<span>text</span>' }} and {{ 2 + 2 }}</div>"#,
        json!({}),
        r#"<div>&lt;span&gt;text&lt;/span&gt; and 4</div>"#,
    );
}

#[test]
fn mustache_static_attr_not_interpolated() {
    assert_render_body_eq!(
        r#"<div title="{{ '<br />' }}">{{ 1 + 1 }}</div>"#,
        json!({}),
        r#"<div title="{{ '&lt;br /&gt;' }}">2</div>"#,
    );
}

#[test]
fn html_static_attr_preserved() {
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
            <div>[
  1,
  2,
  3
]</div>
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
            <div>{
  "name": "Alice",
  "age": 21
}</div>
            <div>Alice</div>
            <div>21</div>
        </div>"#,
    );
}

#[test]
fn mustache_json_string_escaping() {
    assert_render_body_eq!(
        r#"<div>
            <p>{{ list }}</p>
            <p>{{ object }}</p>
        </div>"#,
        json!({
            "list": ["a\"b", "c\\d", "line\nbreak", "<tag>"],
            "object": {
                "a\"b": "c\\d",
                "line\nkey": "line\nvalue",
                "html": "<span>",
            },
        }),
        r#"<div>
            <p>[
  "a\"b",
  "c\\d",
  "line\nbreak",
  "&lt;tag&gt;"
]</p>
            <p>{
  "a\"b": "c\\d",
  "line\nkey": "line\nvalue",
  "html": "&lt;span&gt;"
}</p>
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
fn data_alias_shares_identity() {
    // A top-level field and its `$` counterpart are the same JavaScript value,
    // not independent copies.
    assert_render_body_eq!(
        r#"<div>{{ list === $.list }}</div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>true</div>"#,
    );
}

#[test]
fn data_alias_mutation_is_shared() {
    assert_render_body_eq!(
        r#"<div>{{ list.push(4); $.list.join(",") }}</div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>1,2,3,4</div>"#,
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
        r#"<div>[
  "a",
  "b"
]</div>"#,
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

// === toDisplayString ===
//
// `{{ }}` follows Vue's `toDisplayString`, which is not plain `String(v)`:
// arrays and plain objects become indented JSON, and anything that spells
// itself out is left to do so.

#[test]
fn mustache_object_is_indented_json() {
    assert_render_body_eq!(
        r#"<div>{{ o }}</div>"#,
        json!({ "o": { "a": 1 } }),
        "<div>{\n  \"a\": 1\n}</div>",
    );
}

#[test]
fn mustache_date_spells_itself() {
    // A Date defines its own toString, so it is not turned into JSON.
    let out = render(r#"<p>{{ new Date(0) }}</p>"#, json!({})).unwrap();
    assert!(out.contains("1970"), "{out}");
    assert!(!out.contains('{'), "{out}");
}

#[test]
fn mustache_regexp_spells_itself() {
    assert_render_body_eq!(r#"<p>{{ /ab+c/g }}</p>"#, json!({}), "<p>/ab+c/g</p>");
}

#[test]
fn mustache_custom_to_string_wins() {
    assert_render_body_eq!(
        r#"<p>{{ ({ toString() { return 'CUSTOM' } }) }}</p>"#,
        json!({}),
        "<p>CUSTOM</p>",
    );
}

#[test]
fn mustache_map_and_set() {
    assert_render_body_eq!(
        r#"<p>{{ new Map([['x', 1]]) }}</p><p>{{ new Set(['a']) }}</p>"#,
        json!({}),
        "<p>{\n  \"Map(1)\": {\n    \"x =&gt;\": 1\n  }\n}</p>\
         <p>{\n  \"Set(1)\": [\n    \"a\"\n  ]\n}</p>",
    );
}

#[test]
fn mustache_symbol_is_bare_at_top_level_and_quoted_when_nested() {
    // A top-level symbol never reaches the replacer, so it has no quotes.
    assert_render_body_eq!(
        r#"<p>{{ Symbol('houu') }}</p><p>{{ ({ s: Symbol('houu') }) }}</p>"#,
        json!({}),
        "<p>Symbol(houu)</p><p>{\n  \"s\": \"Symbol(houu)\"\n}</p>",
    );
}

#[test]
fn mustache_bigint_is_a_plain_number() {
    assert_render_body_eq!(r#"<p>{{ 10n }}</p>"#, json!({}), "<p>10</p>");
}

#[test]
fn mustache_cycle_renders_nothing() {
    assert_render_body_eq!(
        r#"<p>{{ (() => { const o = {}; o.self = o; return o })() }}</p>"#,
        json!({}),
        "<p></p>",
    );
}

#[test]
fn mustache_object_inside_for_keeps_its_indentation() {
    // `v-for` re-indents cloned subtrees before interpolation runs, so the JSON
    // must come through untouched.
    assert_render_body_eq!(
        r#"<ul><li v-for="n in [1]">{{ ({ a: n }) }}</li></ul>"#,
        json!({}),
        "<ul><li>{\n  \"a\": 1\n}</li></ul>",
    );
}
