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
fn test_else1() {
    let input = r#"
    <div>
        <div v-if="true">IF</div>
        <div v-else>ELSE</div>
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
fn test_else2() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else>ELSE</div>
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
fn test_else_if1() {
    let input = r#"
    <div>
        <div v-if="true">IF</div>
        <div v-else-if="true">ELSE-IF</div>
        <div v-else>ELSE</div>
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
fn test_else_if2() {
    let input = r#"
    <div>
        <div v-if="true">IF</div>
        <div v-else-if="false">ELSE-IF</div>
        <div v-else>ELSE</div>
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
fn test_else_if3() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="true">ELSE-IF</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE-IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_if4() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="false">ELSE-IF</div>
        <div v-else>ELSE</div>
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
fn test_else_chain1() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="true">ELSE-IF1</div>
        <div v-else-if="true">ELSE-IF2</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE-IF1</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_chain2() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="true">ELSE-IF1</div>
        <div v-else-if="false">ELSE-IF2</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE-IF1</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_chain3() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="false">ELSE-IF1</div>
        <div v-else-if="true">ELSE-IF2</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE-IF2</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_chain4() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="false">ELSE-IF1</div>
        <div v-else-if="false">ELSE-IF2</div>
        <div v-else>ELSE</div>
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
fn test_else_standalone() {
    let input = r#"
    <div>
        <div>Normal</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    // v-else without v-if: should render as normal (not error in prevue)
    let expected = r#"<html><head></head><body><div>
        <div>Normal</div>
        <div>ELSE</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_if_standalone() {
    let input = r#"
    <div>
        <div>Normal</div>
        <div v-else-if="false">ELSE-IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>Normal</div>
        <div>ELSE-IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_adjacent() {
    let input = r#"
    <div>
        <div v-if="true">IF1</div>
        <div v-else>ELSE1</div>
        <div v-if="false">IF2</div>
        <div v-else>ELSE2</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF1</div>
        <div>ELSE2</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_if_adjacent() {
    let input = r#"
    <div>
        <div v-if="true">IF1</div>
        <div v-else-if="false">ELSE-IF1</div>
        <div v-if="false">IF2</div>
        <div v-else-if="true">ELSE-IF2</div>
        <div v-else>ELSE2</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF1</div>
        <div>ELSE-IF2</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_if_chain_mix() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else>ELSE</div>
        <div v-else-if="true">ELSE-IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE</div>
        <div>ELSE-IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
