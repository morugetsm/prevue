mod helper;

use helper::assert_render_body_eq;
use serde_json::json;

// === Basic Behavior ===

#[test]
fn pre_basic() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>PRE</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>PRE</div>
    </div>"#,
    );
}

#[test]
fn pre_empty() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre></div>
    </div>"#,
        json!({}),
        r#"<div>
        <div></div>
    </div>"#,
    );
}

// === Interpolation ===

#[test]
fn pre_with_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>{{ message }} {{ count }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>{{ message }} {{ count }}</div>
    </div>"#,
    );
}

#[test]
fn pre_multiline() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>
            Line 1: {{ message }}
            Line 2: {{ count }}
        </div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>
            Line 1: {{ message }}
            Line 2: {{ count }}
        </div>
    </div>"#,
    );
}

#[test]
fn pre_html_like_mustache() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>{{ '<br />' }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>{{ '&lt;br /&gt;' }}</div>
    </div>"#,
    );
}

// === Directives & Attributes ===

#[test]
fn pre_with_if() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre v-if="isVisible">{{ message }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div v-if="isVisible">{{ message }}</div>
    </div>"#,
    );
}

#[test]
fn pre_with_bind() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre :id="elementId">{{ message }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div :id="elementId">{{ message }}</div>
    </div>"#,
    );
}

#[test]
fn pre_leaves_a_static_style_alone() {
    // Vue still normalizes it here; prevue reads `v-pre` as covering every
    // rendering step.
    assert_render_body_eq!(
        r#"<div>
        <div v-pre style="marginTop: 1px">{{ message }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div style="marginTop: 1px">{{ message }}</div>
    </div>"#,
    );
}

// === Nested Elements ===

#[test]
fn pre_with_nested_directives() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>
            <p v-if="isVisible">{{ message }}</p>
            <span v-for="item in [1, 2, 3]">{{ item }}</span>
            <div :id="elementId"></div>
        </div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>
            <p v-if="isVisible">{{ message }}</p>
            <span v-for="item in [1, 2, 3]">{{ item }}</span>
            <div :id="elementId"></div>
        </div>
    </div>"#,
    );
}

#[test]
fn pre_nested_pre() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>
            Outer {{ message }}
            <div v-pre>Inner {{ message }}</div>
        </div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>
            Outer {{ message }}
            <div v-pre="">Inner {{ message }}</div>
        </div>
    </div>"#,
    );
}

// === Isolation ===

#[test]
fn pre_sibling_elements() {
    assert_render_body_eq!(
        r#"<div>
        <p>{{ message }}</p>
        <p v-pre>{{ message }}</p>
        <p>{{ message }}</p>
    </div>"#,
        json!({
            "message": "Hello, world!",
        }),
        r#"<div>
        <p>Hello, world!</p>
        <p>{{ message }}</p>
        <p>Hello, world!</p>
    </div>"#,
    );
}
