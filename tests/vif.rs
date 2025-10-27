use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "list": [1, 2, 3],
        "number": 9999,
        "user": {
            "label": "User",
            "value": "Morrison",
            "age": 28
        },
    })
}

#[test]
fn test_if() {
    let input = r#"
    <div>
        <p>Hello, world!</p>
        <div v-if="true">TRUE</div>
        <div v-if="false">FALSE</div>
        <div v-if="list">LIST</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <div>TRUE</div>
        <div>LIST</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_cast() {
    let input = r#"
    <div>
        <div v-if="0">0 is false</div>
        <div v-if="list">array is true</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>array is true</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_safety() {
    let input = r#"
    <div>
        <div v-if="">empty</div>
        <div v-if="null">null</div>
        <div v-if="undefined">undefined</div>
        <div v-if="NaN">NaN</div>
        <div v-if="Infinity">Infinity</div>
        <div v-if="notexist">notexist</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>Infinity</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
