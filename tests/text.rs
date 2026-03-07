use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "str": "Hello, world!",
        "arr": [1, 2, 3],
    })
}

#[test]
fn test_text() {
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

#[test]
fn test_text_with_contents() {
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
fn test_text_with_mustache() {
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