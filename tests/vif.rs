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

#[test]
fn test_if_with_else1() {
    let input = r#"
    <div>
        <div v-if="true" v-else>IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_with_else2() {
    let input = r#"
    <div>
        <div v-else v-if="true">ELSE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_with_else_if1() {
    let input = r#"
    <div>
        <div v-if="true" v-else-if="false">IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_with_else_if2() {
    let input = r#"
    <div>
        <div v-else-if="false" v-if="true">IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_with_for1() {
    // v-if takes precedence over v-for
    let input = r#"
    <div>
        <div v-if="true" v-for="item in list">IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_with_for2() {
    // Directive precedence is fixed (not by order of appearance)
    let input = r#"
    <div>
        <div v-for="item in list" v-if="item > 1">IF{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
