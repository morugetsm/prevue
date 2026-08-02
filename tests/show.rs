mod helper;

use helper::assert_render_body_eq;
use serde_json::json;

// === Basic ===

#[test]
fn show_falsy_hides() {
    assert_render_body_eq!(
        r#"<div>
        <p v-show="no">x</p>
    </div>"#,
        json!({ "no": false }),
        r#"<div>
        <p style="display: none;">x</p>
    </div>"#,
    );
}

#[test]
fn show_truthy_adds_nothing() {
    // Vue's server renderer writes `style=""` here; prevue leaves the element
    // untouched, as its own bindings do when there is no value.
    assert_render_body_eq!(
        r#"<div>
        <p v-show="yes">x</p>
    </div>"#,
        json!({ "yes": true }),
        r#"<div>
        <p>x</p>
    </div>"#,
    );
}

#[test]
fn show_uses_javascript_truthiness() {
    assert_render_body_eq!(
        r#"<div>
        <p v-show="0">a</p>
        <p v-show="''">b</p>
        <p v-show="[]">c</p>
        <p v-show="'no'">d</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="display: none;">a</p>
        <p style="display: none;">b</p>
        <p>c</p>
        <p>d</p>
    </div>"#,
    );
}

#[test]
fn show_treats_a_failed_expression_as_hidden() {
    assert_render_body_eq!(
        r#"<div>
        <p v-show="nope">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="display: none;">x</p>
    </div>"#,
    );
}

#[test]
fn show_without_an_expression_hides() {
    // An empty expression evaluates to `undefined`, which is falsy.
    assert_render_body_eq!(
        r#"<div>
        <p v-show>x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="display: none;">x</p>
    </div>"#,
    );
}

// === Style Merging ===

#[test]
fn show_merges_with_a_static_style() {
    assert_render_body_eq!(
        r#"<div>
        <p style="color: red" v-show="no">x</p>
    </div>"#,
        json!({ "no": false }),
        r#"<div>
        <p style="color: red; display: none;">x</p>
    </div>"#,
    );
}

#[test]
fn show_overrides_an_existing_display() {
    assert_render_body_eq!(
        r#"<div>
        <p style="display: block; color: red" v-show="no">x</p>
    </div>"#,
        json!({ "no": false }),
        r#"<div>
        <p style="display: none; color: red;">x</p>
    </div>"#,
    );
}

#[test]
fn show_is_merged_after_a_bound_style() {
    // Vue merges directive props last, so writing v-show first changes nothing.
    assert_render_body_eq!(
        r#"<div>
        <p :style="{ display: 'block' }" v-show="no">a</p>
        <p v-show="no" :style="{ display: 'block' }">b</p>
    </div>"#,
        json!({ "no": false }),
        r#"<div>
        <p style="display: none;">a</p>
        <p style="display: none;">b</p>
    </div>"#,
    );
}

#[test]
fn show_leaves_other_elements_alone() {
    // v-show adds one declaration to its own element and nothing else: an empty
    // style written by hand stays exactly as it was.
    assert_render_body_eq!(
        r#"<div>
        <p style="">a</p>
        <p v-show="no">b</p>
        <p>c</p>
    </div>"#,
        json!({ "no": false }),
        r#"<div>
        <p style="">a</p>
        <p style="display: none;">b</p>
        <p>c</p>
    </div>"#,
    );
}

// === With Structural Directives ===

#[test]
fn show_is_evaluated_per_for_item() {
    assert_render_body_eq!(
        r#"<ul><li v-for="n in list" v-show="n > 1">{{ n }}</li></ul>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<ul><li style="display: none;">1</li><li>2</li><li>3</li></ul>"#,
    );
}

#[test]
fn show_is_irrelevant_when_if_removes_the_node() {
    assert_render_body_eq!(
        r#"<div><p v-if="false" v-show="true">x</p></div>"#,
        json!({}),
        r#"<div></div>"#,
    );
}
