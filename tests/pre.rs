use prevue::render;
use serde_json::{Value, json};

#[test]
fn test_pre() {
    let input = r#"
    <div>
        <div v-pre>PRE</div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>PRE</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_empty() {
    let input = r#"
    <div>
        <div v-pre></div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div></div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_with_mustache() {
    let input = r#"
    <div>
        <div v-pre>{{ text }} {{ number }}</div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>{{ text }} {{ number }}</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_with_if() {
    let input = r#"
    <div>
        <div v-pre v-if="show">{{ text }}</div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div v-if="show">{{ text }}</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_with_nested_directives() {
    let input = r#"
    <div>
        <div v-pre>
            <p v-if="show">{{ text }}</p>
            <span v-for="item in [1, 2, 3]">{{ item }}</span>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            <p v-if="show">{{ text }}</p>
            <span v-for="item in [1, 2, 3]">{{ item }}</span>
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_with_bind() {
    let input = r#"
    <div>
        <div v-pre :id="id">{{ text }}</div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div :id="id">{{ text }}</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_multiline() {
    let input = r#"
    <div>
        <div v-pre>
            Line 1: {{ text }}
            Line 2: {{ number }}
        </div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            Line 1: {{ text }}
            Line 2: {{ number }}
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_nested_pre() {
    let input = r#"
    <div>
        <div v-pre>
            Outer {{ text }}
            <div v-pre>Inner {{ text }}</div>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            Outer {{ text }}
            <div v-pre="">Inner {{ text }}</div>
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_sibling_elements() {
    let input = r#"
    <div>
        <p>{{ text }}</p>
        <p v-pre>{{ text }}</p>
        <p>{{ text }}</p>
    </div>
    "#;
    let output = render(input.to_string(), json!({ "text": "Hello" })).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello</p>
        <p>{{ text }}</p>
        <p>Hello</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
