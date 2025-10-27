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
fn test_example() {
    let html = r#"
    <div>
        <p v-if="user.age >= 18">{{ user.name }} is adult</p>
        <ul>
            <li v-for="item in list">{{ item }}</li>
        </ul>
        <a :[attribute]="value">link</a>
    </div>
    "#
    .to_string();

    let data = json!({
        "user": { "name": "Alice", "age": 28 },
        "list": ["a", "b", "c"],
        "attribute": "link-id",
        "value": 123,
    });

    let output = render(html, data).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Alice is adult</p>
        <ul>
            <li>a</li>
            <li>b</li>
            <li>c</li>
        </ul>
        <a link-id="123">link</a>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_html5ever() {
    let input = "";
    let output = render(input.to_string(), data()).unwrap();

    let expected = "<html><head></head><body></body></html>";
    assert_eq!(output, expected);
}

#[test]
fn test_attr_case() {
    let input = r#"
    <div>
        <h1 TTT></h1>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1 ttt=""></h1>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
