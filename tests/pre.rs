use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "message": "Hello, world!",
        "count": 42,
        "isVisible": true,
        "elementId": "item-1"
    })
}

// === Basic Behavior ===

#[test]
fn test_pre_basic() {
    let input = r#"
    <div>
        <div v-pre>PRE</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

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
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div></div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Interpolation ===

#[test]
fn test_pre_with_mustache() {
    let input = r#"
    <div>
        <div v-pre>{{ message }} {{ count }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>{{ message }} {{ count }}</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_multiline() {
    let input = r#"
    <div>
        <div v-pre>
            Line 1: {{ message }}
            Line 2: {{ count }}
        </div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            Line 1: {{ message }}
            Line 2: {{ count }}
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Directives & Attributes ===

#[test]
fn test_pre_with_if() {
    let input = r#"
    <div>
        <div v-pre v-if="isVisible">{{ message }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div v-if="isVisible">{{ message }}</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_pre_with_bind() {
    let input = r#"
    <div>
        <div v-pre :id="elementId">{{ message }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div :id="elementId">{{ message }}</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Nested Elements ===

#[test]
fn test_pre_with_nested_directives() {
    let input = r#"
    <div>
        <div v-pre>
            <p v-if="isVisible">{{ message }}</p>
            <span v-for="item in [1, 2, 3]">{{ item }}</span>
            <div :id="elementId"></div>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            <p v-if="isVisible">{{ message }}</p>
            <span v-for="item in [1, 2, 3]">{{ item }}</span>
            <div :id="elementId"></div>
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
            Outer {{ message }}
            <div v-pre>Inner {{ message }}</div>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            Outer {{ message }}
            <div v-pre="">Inner {{ message }}</div>
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Isolation ===

#[test]
fn test_pre_sibling_elements() {
    let input = r#"
    <div>
        <p>{{ message }}</p>
        <p v-pre>{{ message }}</p>
        <p>{{ message }}</p>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>Hello, world!</p>
        <p>{{ message }}</p>
        <p>Hello, world!</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
