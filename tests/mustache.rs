use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "list": [1, 2, 3],
        "user": {
            "name": "Alice",
            "age": 21
        },
    })
}

// === Basic Behavior ===

#[test]
fn test_mustache_eval() {
    let input = r#"
    <div>
        {{ 1 + 1 }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        2
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_multiple() {
    let input = r#"
    <div>
        {{ 1 + 1 }} and {{ 2 + 2 }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        2 and 4
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_multiline() {
    let input = r#"
    <div>
        {{ 
            1 + 
            1 
        }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        2
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_unclosed() {
    // Unclosed mustache should be left as is or ignored. Let's see what prevue currently outputs.
    // Actually, in test_for_array in tests/for.rs: <h1>{{ notclosed }</h1> outputs as <h1>{{ notclosed }</h1>
    let input = r#"
    <div>
        {{ unclosed }
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        {{ unclosed }
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_empty() {
    // Empty mustache evaluates to empty or undefined
    let input = r#"
    <div>
        [{{ }}]
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        []
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Value Types ===

#[test]
fn test_mustache_array() {
    let input = r#"
    <div>
        <p>Hello, world!</p>
        <div>{{ list }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <div>[ 1, 2, 3 ]</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_object() {
    let input = r#"
    <div>
        <p>Hello, world!</p>
        <div>{{ user }}</div>
        <div>{{ user.name }}</div>
        <div>{{ user.age }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <div>{ "name": "Alice", "age": 21 }</div>
        <div>Alice</div>
        <div>21</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_falsy() {
    let input = r#"
    <div>
        <div>{{ false }}</div>
        <div>{{ null }}</div>
        <div>{{ undefined }}</div>
        <div>{{ 0 }}</div>
        <div>{{ "" }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>false</div>
        <div></div>
        <div></div>
        <div>0</div>
        <div></div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Statements ===

#[test]
fn test_mustache_statement() {
    // unlike Vue, prevue currently allows both expressions and statements (e.g., `{{ let x = 1; x + 1 }}`)
    let input = r#"
    <div>
        {{ let exist = true; exist }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        true
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_error() {
    // an expression that throws an error (e.g. ReferenceError) should fallback to an empty string safely
    let input = r#"
    <div>
        [{{ does_not_exist }}]
        [{{ foo.bar.baz }}]
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        []
        []
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Scope & Isolation ===

#[test]
fn test_mustache_this() {
    // can't serialize this
    let input = r#"
    <div>
        {{ this }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_comment() {
    // JavaScript comments inside mustache are valid
    let input = r#"
    <div>
        {{ 
            // single line comment
            1 + 1 
            /* multi
               line
               comment */
            + 1
        }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        3
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_this_json() {
    // can't serialize this
    let input = r#"
    <div>
        {{ JSON.stringify(this) }}
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        {"__scope_0":{"list":[1,2,3],"user":{"name":"Alice","age":21}}}
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_mustache_isolation() {
    let input = r#"
    <div>
        <h1>{{ let x = 1; x }}</h1>
        <h2>{{ x }}</h2>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>1</h1>
        <h2></h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
