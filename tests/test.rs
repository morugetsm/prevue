mod helper;

use helper::{assert_render_body_eq, assert_render_eq};
use serde_json::json;

#[test]
fn example() {
    assert_render_body_eq!(
        r#"<div>
        <a :id="id">link</a>
        <p v-if="user.age >= 18">{{ user.name }} is adult</p>
        <ul>
            <li v-for="item in list">{{ item }}</li>
        </ul>
    </div>"#,
        json!({
            "id": "link-id",
            "list": ["a", "b", "c"],
            "user": { "name": "James", "age": 28 },
        }),
        r#"<div>
        <a id="link-id">link</a>
        <p>James is adult</p>
        <ul>
            <li>a</li>
            <li>b</li>
            <li>c</li>
        </ul>
    </div>"#,
    );
}

#[test]
fn example_with_less_indent() {
    assert_render_body_eq!(
        r#"<div>
    <a :id="id">link</a>
    <p v-if="user.age >= 18">{{ user.name }} is adult</p>
    <ul>
      <li v-for="item in list">{{ item }}</li>
    </ul>
  </div>"#,
        json!({
            "id": "link-id",
            "list": ["a", "b", "c"],
            "user": { "name": "James", "age": 28 },
        }),
        r#"<div>
    <a id="link-id">link</a>
    <p>James is adult</p>
    <ul>
      <li>a</li>
      <li>b</li>
      <li>c</li>
    </ul>
  </div>"#,
    );
}

#[test]
fn empty_template() {
    assert_render_eq!("", json!({}), "<html><head></head><body></body></html>");
}

#[test]
fn attr_lowercase() {
    assert_render_body_eq!(
        r#"<div>
        <h1 TTT></h1>
    </div>"#,
        json!({}),
        r#"<div>
        <h1 ttt=""></h1>
    </div>"#,
    );
}
