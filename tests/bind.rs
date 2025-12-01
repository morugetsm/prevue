use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "id": "id-value",
        "value": 333,
        "attrs": {
            "str": "hello",
            "num": 123,
            "truthy": true,
            "falsy": false,
            "nullval": null,
        },
        "dynamicKey": "data-id",
        "dynamic-key": "data-id",
    })
}

#[test]
fn test_bind() {
    let input = r#"
    <div>
        <h1 v-bind:id="id">h1 elem</h1>
        <h2 :value="value">h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 id="id-value">h1 elem</h1>
        <h2 value="333">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_shorthand() {
    let input = r#"
    <div>
        <h1 v-bind:id>h1 elem</h1>
        <h2 :value>foo</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 id="id-value">h1 elem</h1>
        <h2 value="333">foo</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_dynamic() {
    let input = r#"
    <div>
        <h1 v-bind:[id]="id">h1 elem</h1>
        <h2 :[value]="value">h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 id-value="id-value">h1 elem</h1>
        <h2 333="333">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_dynamic_unclosed() {
    let input = r#"
    <div>
        <h1 v-bind:[id="id">h1 elem</h1>
        <h2 :value]="value">h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 [id="id-value">h1 elem</h1>
        <h2 value]="333">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_dynamic_shorthand() {
    // dynamic shorthand should not be supported
    let input = r#"
    <div>
        <h1 v-bind:[id]>h1 elem</h1>
        <h2 :[value]>h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>h1 elem</h1>
        <h2>h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_eval() {
    let input = r#"
    <div>
        <h1 :format="`hello ${id}`">h1 elem</h1>
        <h2 :calc="value * 2">h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 format="hello id-value">h1 elem</h1>
        <h2 calc="666">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_statement() {
    // unlike Vue, prevue currently allows both expressions and statements in v-bind
    let input = r#"
    <div>
        <h1 :format="let x = 1; x + 1">h1 elem</h1>
        <h2 :calc="let y = 2; y * 2">h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 format="2">h1 elem</h1>
        <h2 calc="4">h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_nullish() {
    let input = r#"
    <div>
        <h1 :foo="null">h1 elem</h1>
        <h2 :bar="undefined">h2 elem</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>h1 elem</h1>
        <h2>h2 elem</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_bind_object() {
    let input = r#"
    <div>
        <span v-bind="attrs"></span>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <span str="hello" num="123" truthy="true" falsy="false"></span>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_attr_case_dynamic() {
    let input = r#"
    <div>
        <h1>{{ dynamicKey }}</h1>
        <h2>{{ dynamic-key }}</h2>
        <h3>{{ value }}</h3>
        <h4 :[dynamicKey]="value">link</h4>
        <h5 :[dynamic-key]="value">link</h5>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

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
