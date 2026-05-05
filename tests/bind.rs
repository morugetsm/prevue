use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "id": "title",
        "value": 333,
        "attrs": {
            "str": "hello",
            "num": 123,
            "truthy": true,
            "falsy": false,
            "nullish": null,
        },
        "dynamicKey": "data-id",
        "dynamic-key": "data-id",
    })
}

// === Basic Binding ===

#[test]
fn bind_basic() {
    // v-bind:attr="expr" (long form) and :attr="expr" (colon shorthand)
    let input = r#"
    <div>
        <h1 v-bind:id="id">h1 elem</h1>
        <h2 :value="value">h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 id="title">h1 elem</h1>
        <h2 value="333">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_same_name_shorthand() {
    // :attr with no value uses the attribute name as the expression
    let input = r#"
    <div>
        <h1 v-bind:id>h1 elem</h1>
        <h2 :value>foo</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 id="title">h1 elem</h1>
        <h2 value="333">foo</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Dynamic Key Binding ===

#[test]
fn bind_dynamic_key() {
    // :[expr]="value" — attribute name resolved from expression
    let input = r#"
    <div>
        <h1 v-bind:[id]="id">h1 elem</h1>
        <h2 :[value]="value">h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 title="title">h1 elem</h1>
        <h2 333="333">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_dynamic_key_no_value() {
    // :[expr] with no value is not supported — attribute is removed
    let input = r#"
    <div>
        <h1 v-bind:[id]>h1 elem</h1>
        <h2 :[value]>h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>h1 elem</h1>
        <h2>h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_dynamic_key_unclosed() {
    // malformed dynamic key (missing ] or [) falls through as a literal attr name
    let input = r#"
    <div>
        <h1 v-bind:[id="id">h1 elem</h1>
        <h2 :value]="value">h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 [id="title">h1 elem</h1>
        <h2 value]="333">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_dynamic_key_lowercase() {
    // HTML5 lowercases attribute names, so camelCase dynamic keys silently fail
    let input = r#"
    <div>
        <h1>{{ dynamicKey }}</h1>
        <h2>{{ dynamic-key }}</h2>
        <h3>{{ value }}</h3>
        <h4 :[dynamicKey]="value">link</h4>
        <h5 :[dynamic-key]="value">link</h5>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>data-id</h1>
        <h2></h2>
        <h3>333</h3>
        <h4>link</h4>
        <h5>link</h5>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Expression Values ===

#[test]
fn bind_expression() {
    // attribute value can be any JS expression
    let input = r#"
    <div>
        <h1 :format="`hello ${id}`">h1 elem</h1>
        <h2 :calc="value * 2">h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 format="hello title">h1 elem</h1>
        <h2 calc="666">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_statement() {
    // unlike Vue, prevue currently allows both expressions and statements in v-bind
    let input = r#"
    <div>
        <h1 :format="let x = 1; x + 1">h1 elem</h1>
        <h2 :calc="let y = 2; y * 2">h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 format="2">h1 elem</h1>
        <h2 calc="4">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Null / Undefined ===

#[test]
fn bind_null_undefined() {
    // null or undefined value removes the attribute entirely
    let input = r#"
    <div>
        <h1 :foo="null">h1 elem</h1>
        <h2 :bar="undefined">h2 elem</h2>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>h1 elem</h1>
        <h2>h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === False ===

#[test]
fn bind_false_kept() {
    // false keeps the attribute as "false" — only null/undefined remove it
    let input = r#"
    <div>
        <h1 :foo="false">h1 elem</h1>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 foo="false">h1 elem</h1>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Object Syntax ===

#[test]
fn bind_object() {
    // v-bind="obj" spreads object properties as attributes; null values are filtered out
    let input = r#"
    <div>
        <span v-bind="attrs"></span>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <span str="hello" num="123" truthy="true" falsy="false"></span>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_object_overrides_existing_attr() {
    // v-bind="obj" overrides a static attribute that shares the same name
    let input = r#"
    <div>
        <span str="old" v-bind="attrs">elem</span>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <span str="hello" num="123" truthy="true" falsy="false">elem</span>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_object_literal() {
    let input = r#"
    <div>
        <span v-bind="{ id, value: value * 2, hidden: null, skip: undefined, ok: false }">elem</span>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <span id="title" value="666" ok="false">elem</span>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Class / Style Normalization ===

#[test]
fn bind_class_merges_static_and_object() {
    let input = r#"
    <div>
        <p class="base" :class="{ active: true, hidden: false, titled: id }">elem</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p class="base active titled">elem</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_class_array() {
    let input = r#"
    <div>
        <p :class="['btn', [id, { active: true, hidden: false }], null]">elem</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p class="btn title active">elem</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_object_class_merges_static() {
    let input = r#"
    <div>
        <p class="base" v-bind="{ class: ['from-bind', { active: true }], id }">elem</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p class="base from-bind active" id="title">elem</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_style_merges_static_and_object() {
    let input = r#"
    <div>
        <p style="color: red" :style="{ fontSize: '12px', 'line-height': 1.5, '--gap': '4px', display: null }">elem</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p style="color: red; font-size: 12px; line-height: 1.5; --gap: 4px;">elem</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_style_array() {
    let input = r#"
    <div>
        <p :style="['color: red', { fontSize: '12px' }, { marginTop: value + 'px' }]">elem</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p style="color: red; font-size: 12px; margin-top: 333px;">elem</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn bind_object_style_merges_static() {
    let input = r#"
    <div>
        <p style="color: red;" v-bind="{ style: [{ fontSize: '12px' }, 'background: blue'], id }">elem</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p style="color: red; font-size: 12px; background: blue" id="title">elem</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
