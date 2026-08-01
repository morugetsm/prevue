mod helper;

use helper::assert_render_body_eq;
use serde_json::json;

// === Basic Behavior ===

#[test]
fn text_explicit_close() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str"></p>
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p>
    </div>"#,
    );
}

#[test]
fn text_self_closing() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str" />
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p></div>"#,
    );
}

#[test]
fn text_self_closing_explicit() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str" /></p>
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p>
    </div>"#,
    );
}

// === Overrides Inner Content ===

#[test]
fn text_overrides_inner_content() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str">original content</p>
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p>
    </div>"#,
    );
}

#[test]
fn text_overrides_compact() {
    assert_render_body_eq!(
        r#"<div><p v-text="str">fallback</p></div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div><p>Hello, world!</p></div>"#,
    );
}

#[test]
fn text_overrides_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str">{{ arr }}</p>
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p>
    </div>"#,
    );
}

#[test]
fn text_self_closing_text() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str" />Hello</p>
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p>
    </div>"#,
    );
}

#[test]
fn text_self_closing_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str" />{{ true }}</p>
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p>
    </div>"#,
    );
}

// === Value Types ===

#[test]
fn text_null() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="null_val"></p>
    </div>"#,
        json!({ "null_val": null }),
        r#"<div>
        <p></p>
    </div>"#,
    );
}

#[test]
fn text_undefined() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="undefined"></p>
    </div>"#,
        json!({}),
        r#"<div>
        <p></p>
    </div>"#,
    );
}

#[test]
fn text_boolean() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="bool_val" />
    </div>"#,
        json!({ "bool_val": true }),
        r#"<div>
        <p>true</p></div>"#,
    );
}

#[test]
fn text_string() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="str" />
    </div>"#,
        json!({ "str": "Hello, world!" }),
        r#"<div>
        <p>Hello, world!</p></div>"#,
    );
}

#[test]
fn text_number() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="num" />
    </div>"#,
        json!({ "num": 42 }),
        r#"<div>
        <p>42</p></div>"#,
    );
}

#[test]
fn text_array_self_closing() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="arr" />
    </div>"#,
        json!({ "arr": [1, 2, 3] }),
        r#"<div>
        <p>1,2,3</p></div>"#,
    );
}

#[test]
fn text_array_explicit_close() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="arr"></p>
    </div>"#,
        json!({ "arr": [1, 2, 3] }),
        r#"<div>
        <p>1,2,3</p>
    </div>"#,
    );
}

#[test]
fn text_array_vs_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="arr" />
        <p>{{ arr }}</p>
    </div>"#,
        json!({ "arr": [1, 2, 3] }),
        r#"<div>
        <p>1,2,3</p><p>[
  1,
  2,
  3
]</p>
    </div>"#,
    );
}

#[test]
fn text_array_mixed() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="mixed_arr" />
    </div>"#,
        json!({
            "mixed_arr": [null, true, "hello", 1, [4, 5, 6], { "a": "b" }],
        }),
        r#"<div>
        <p>,true,hello,1,4,5,6,[object Object]</p></div>"#,
    );
}

#[test]
fn text_array_mixed_vs_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="mixed_arr" />
        <p>{{ mixed_arr }}</p>
    </div>"#,
        json!({
            "mixed_arr": [null, true, "hello", 1, [4, 5, 6], { "a": "b" }],
        }),
        r#"<div>
        <p>,true,hello,1,4,5,6,[object Object]</p><p>[
  null,
  true,
  "hello",
  1,
  [
    4,
    5,
    6
  ],
  {
    "a": "b"
  }
]</p>
    </div>"#,
    );
}

#[test]
fn text_object() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="obj" />
    </div>"#,
        json!({
            "obj": { "key": "value" },
        }),
        r#"<div>
        <p>[object Object]</p></div>"#,
    );
}

#[test]
fn text_object_vs_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="obj" />
        <p>{{ obj }}</p>
    </div>"#,
        json!({
            "obj": { "key": "value" },
        }),
        r#"<div>
        <p>[object Object]</p><p>{
  "key": "value"
}</p>
    </div>"#,
    );
}

#[test]
fn text_object_mixed() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="mixed_obj" />
    </div>"#,
        json!({
            "mixed_obj": { "a": null, "b": true, "c": "hello", "d": 1, "e": [4, 5, 6], "f": { "g": "h" } },
        }),
        r#"<div>
        <p>[object Object]</p></div>"#,
    );
}

#[test]
fn text_object_mixed_vs_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="mixed_obj" />
        <p>{{ mixed_obj }}</p>
    </div>"#,
        json!({
            "mixed_obj": { "a": null, "b": true, "c": "hello", "d": 1, "e": [4, 5, 6], "f": { "g": "h" } },
        }),
        r#"<div>
        <p>[object Object]</p><p>{
  "a": null,
  "b": true,
  "c": "hello",
  "d": 1,
  "e": [
    4,
    5,
    6
  ],
  "f": {
    "g": "h"
  }
}</p>
    </div>"#,
    );
}

// === Multiple Elements ===

#[test]
fn text_multiple_self_closing() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="arr" />
        <p v-text="arr" />
    </div>"#,
        json!({ "arr": [1, 2, 3] }),
        r#"<div>
        <p>1,2,3</p><p>1,2,3</p></div>"#,
    );
}

#[test]
fn text_multiple_explicit_close() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="arr"></p>
        <p v-text="arr"></p>
    </div>"#,
        json!({ "arr": [1, 2, 3] }),
        r#"<div>
        <p>1,2,3</p>
        <p>1,2,3</p>
    </div>"#,
    );
}

#[test]
fn text_multiple_different_values() {
    assert_render_body_eq!(
        r#"<div>
        <p v-text="null_val"></p>
        <p v-text="bool_val"></p>
        <p v-text="str"></p>
        <p v-text="num"></p>
        <p v-text="arr"></p>
        <p v-text="obj"></p>
        <p v-text="mixed_arr"></p>
        <p v-text="mixed_obj"></p>
    </div>"#,
        json!({
            "arr": [1, 2, 3],
            "bool_val": true,
            "mixed_arr": [null, true, "hello", 1, [4, 5, 6], { "a": "b" }],
            "mixed_obj": { "a": null, "b": true, "c": "hello", "d": 1, "e": [4, 5, 6], "f": { "g": "h" } },
            "null_val": null,
            "num": 42,
            "obj": { "key": "value" },
            "str": "Hello, world!",
        }),
        r#"<div>
        <p></p>
        <p>true</p>
        <p>Hello, world!</p>
        <p>42</p>
        <p>1,2,3</p>
        <p>[object Object]</p>
        <p>,true,hello,1,4,5,6,[object Object]</p>
        <p>[object Object]</p>
    </div>"#,
    );
}
