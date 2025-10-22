use prevue::render;
use serde_json::{Value, json};

fn payload() -> Value {
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
fn test_eval() {
    let input = r#"
    <div>
        {{ 1 + 1 }}
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        2
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_array() {
    let input = r#"
    <div>
        <p>Hello, world!</p>
        <div>{{ list }}</div>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <div>[1,2,3]</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_object() {
    let input = r#"
    <div>
        <p>Hello, world!</p>
        <div>{{ user }}</div>
        <div>{{ user.label }}</div>
        <div>{{ user.value }}</div>
        <div>{{ user.age }}</div>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <div>{"label":"User","value":"Morrison","age":28}</div>
        <div>User</div>
        <div>Morrison</div>
        <div>28</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_statement() {
    // unlike Vue, prevue currently allows both expressions and statements (e.g., `{{ let x = 1; x + 1 }}`)
    let input = r#"
    <div>
        {{ let exist = true; exist }}
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        true
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_this() {
    // can't serialize this
    let input = r#"
    <div>
        {{ this }}
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_this_json() {
    // can't serialize this
    let input = r#"
    <div>
        {{ JSON.stringify(this) }}
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        {"__scope_0":{"list":[1,2,3],"number":9999,"user":{"label":"User","value":"Morrison","age":28}}}
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_isolation() {
    let input = r#"
    <div>
        <h1>{{ let x = 1; x }}</h1>
        <h2>{{ x }}</h2>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>1</h1>
        <h2></h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
