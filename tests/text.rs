use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "null_val": null,
        "bool_val": true,
        "str": "Hello, world!",
        "num": 42,
        "arr": [1, 2, 3],
        "obj": { "key": "value" },
        "mixed_arr": [null, true, "hello", 1, [4, 5, 6], { "a": "b" }],
        "mixed_obj": { "a": null, "b": true, "c": "hello", "d": 1, "e": [4, 5, 6], "f": { "g": "h" } },
    })
}

// === Basic Behavior ===

#[test]
fn test_text_explicit_close() {
    let input = r#"
    <div>
        <p v-text="str"></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_self_closing() {
    let input = r#"
    <div>
        <p v-text="str" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_self_closing_with_explicit_close() {
    // /> followed by </p> — HTML5 treats /> as > for non-void elements
    let input = r#"
    <div>
        <p v-text="str" /></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Overrides Inner Content ===

#[test]
fn test_text_overrides_inner_content() {
    let input = r#"
    <div>
        <p v-text="str">original content</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_overrides_mustache() {
    // v-text wins over mustache expression in inner content
    let input = r#"
    <div>
        <p v-text="str">{{ arr }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_self_closing_overrides_inner_text() {
    // text between /> and </p> is treated as inner content and overridden
    let input = r#"
    <div>
        <p v-text="str" />Hello</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_self_closing_overrides_mustache() {
    // mustache between /> and </p> is treated as inner content and overridden
    let input = r#"
    <div>
        <p v-text="str" />{{ true }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Sibling Preservation ===

#[test]
fn test_text_self_closing_preserves_text_sibling() {
    // text sibling on the next line is preserved
    let input = r#"
    <div>
        <p v-text="str" />
        wow
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        wow
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_self_closing_preserves_inline_sibling() {
    // inline text immediately after /> remains after the element
    let input = r#"
    <div>
        <p v-text="str" />wow
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>wow
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Value Types ===

#[test]
fn test_text_null() {
    // null data field: v-text is removed, inner content unchanged
    let input = r#"
    <div>
        <p v-text="null_val"></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p></p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_undefined() {
    // undefined value: v-text is removed, inner content unchanged
    let input = r#"
    <div>
        <p v-text="undefined"></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p></p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_boolean() {
    let input = r#"
    <div>
        <p v-text="bool_val" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>true</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_string() {
    let input = r#"
    <div>
        <p v-text="str" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_number() {
    let input = r#"
    <div>
        <p v-text="num" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>42</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_array_self_closing() {
    let input = r#"
    <div>
        <p v-text="arr" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>1,2,3</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_array_explicit_close() {
    let input = r#"
    <div>
        <p v-text="arr"></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>1,2,3</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_array_with_format() {
    let input = r#"
    <div>
        <p v-text="arr" />
        <p>{{ arr }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>1,2,3</p>
        <p>[ 1, 2, 3 ]</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_array_mixed() {
    // array containing multiple value types
    let input = r#"
    <div>
        <p v-text="mixed_arr" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>,true,hello,1,4,5,6,[object Object]</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_array_mixed_with_format() {
    let input = r#"
    <div>
        <p v-text="mixed_arr" />
        <p>{{ mixed_arr }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>,true,hello,1,4,5,6,[object Object]</p>
        <p>[ null, true, "hello", 1, [ 4, 5, 6 ], { "a": "b" } ]</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_object() {
    // plain object coerces to string like JS: [object Object]
    let input = r#"
    <div>
        <p v-text="obj" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>[object Object]</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_object_with_format() {
    let input = r#"
    <div>
        <p v-text="obj" />
        <p>{{ obj }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>[object Object]</p>
        <p>{ "key": "value" }</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_object_mixed() {
    // object with multiple value types coerces to string like JS: [object Object]
    let input = r#"
    <div>
        <p v-text="mixed_obj" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>[object Object]</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_object_mixed_with_format() {
    let input = r#"
    <div>
        <p v-text="mixed_obj" />
        <p>{{ mixed_obj }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>[object Object]</p>
        <p>{ "a": null, "b": true, "c": "hello", "d": 1, "e": [ 4, 5, 6 ], "f": { "g": "h" } }</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Multiple Elements ===

#[test]
fn test_text_multiple_self_closing() {
    let input = r#"
    <div>
        <p v-text="arr" />
        <p v-text="arr" />
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>1,2,3</p>
        <p>1,2,3</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_multiple_explicit_close() {
    let input = r#"
    <div>
        <p v-text="arr"></p>
        <p v-text="arr"></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>1,2,3</p>
        <p>1,2,3</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_text_multiple_different_values() {
    // multiple v-text elements with different value types in the same container
    let input = r#"
    <div>
        <p v-text="null_val"></p>
        <p v-text="bool_val"></p>
        <p v-text="str"></p>
        <p v-text="num"></p>
        <p v-text="arr"></p>
        <p v-text="obj"></p>
        <p v-text="mixed_arr"></p>
        <p v-text="mixed_obj"></p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p></p>
        <p>true</p>
        <p>Hello, world!</p>
        <p>42</p>
        <p>1,2,3</p>
        <p>[object Object]</p>
        <p>,true,hello,1,4,5,6,[object Object]</p>
        <p>[object Object]</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
