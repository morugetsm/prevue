mod helper;

use helper::assert_render_body_eq;
use serde_json::json;

// === Basic Binding ===

#[test]
fn bind_basic() {
    assert_render_body_eq!(
        r#"<div>
        <h1 v-bind:id="id">h1 elem</h1>
        <h2 :value="value">h2 elem</h2>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <h1 id="title">h1 elem</h1>
        <h2 value="333">h2 elem</h2>
    </div>"#,
    );
}

#[test]
fn bind_same_name_shorthand() {
    assert_render_body_eq!(
        r#"<div>
        <h1 v-bind:id>h1 elem</h1>
        <h2 :value>foo</h2>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <h1 id="title">h1 elem</h1>
        <h2 value="333">foo</h2>
    </div>"#,
    );
}

// === Dynamic Key Binding ===

#[test]
fn bind_dynamic_key() {
    assert_render_body_eq!(
        r#"<div>
        <h1 v-bind:[id]="id">h1 elem</h1>
        <h2 :[value]="value">h2 elem</h2>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <h1 title="title">h1 elem</h1>
        <h2 333="333">h2 elem</h2>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_key_no_value() {
    assert_render_body_eq!(
        r#"<div>
        <h1 v-bind:[id]>h1 elem</h1>
        <h2 :[value]>h2 elem</h2>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <h1>h1 elem</h1>
        <h2>h2 elem</h2>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_key_unclosed() {
    assert_render_body_eq!(
        r#"<div>
        <h1 v-bind:[id="id">h1 elem</h1>
        <h2 :value]="value">h2 elem</h2>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <h1 [id="title">h1 elem</h1>
        <h2 value]="333">h2 elem</h2>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_key_lowercase() {
    assert_render_body_eq!(
        r#"<div>
        <h1>{{ dynamicKey }}</h1>
        <h2>{{ dynamic-key }}</h2>
        <h3>{{ value }}</h3>
        <h4 :[dynamicKey]="value">link</h4>
        <h5 :[dynamic-key]="value">link</h5>
    </div>"#,
        json!({
            "dynamicKey": "data-id",
            "value": 333,
        }),
        r#"<div>
        <h1>data-id</h1>
        <h2></h2>
        <h3>333</h3>
        <h4>link</h4>
        <h5>link</h5>
    </div>"#,
    );
}

// === Expression Values ===

#[test]
fn bind_expression() {
    assert_render_body_eq!(
        r#"<div>
        <h1 :format="`hello ${id}`">h1 elem</h1>
        <h2 :calc="value * 2">h2 elem</h2>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <h1 format="hello title">h1 elem</h1>
        <h2 calc="666">h2 elem</h2>
    </div>"#,
    );
}

#[test]
fn bind_statement() {
    assert_render_body_eq!(
        r#"<div>
        <h1 :format="let x = 1; x + 1">h1 elem</h1>
        <h2 :calc="let y = 2; y * 2">h2 elem</h2>
    </div>"#,
        json!({}),
        r#"<div>
        <h1 format="2">h1 elem</h1>
        <h2 calc="4">h2 elem</h2>
    </div>"#,
    );
}

// === Null / False ===

#[test]
fn bind_null_undefined() {
    assert_render_body_eq!(
        r#"<div>
        <h1 :foo="null">h1 elem</h1>
        <h2 :bar="undefined">h2 elem</h2>
    </div>"#,
        json!({}),
        r#"<div>
        <h1>h1 elem</h1>
        <h2>h2 elem</h2>
    </div>"#,
    );
}

#[test]
fn bind_false_kept() {
    assert_render_body_eq!(
        r#"<div>
        <h1 :foo="false">h1 elem</h1>
    </div>"#,
        json!({}),
        r#"<div>
        <h1 foo="false">h1 elem</h1>
    </div>"#,
    );
}

// === Object Syntax ===

#[test]
fn bind_object() {
    assert_render_body_eq!(
        r#"<div>
        <span v-bind="attrs"></span>
    </div>"#,
        json!({
            "attrs": {
                "str": "hello",
                "num": 123,
                "truthy": true,
                "falsy": false,
                "nullish": null,
            },
        }),
        r#"<div>
        <span str="hello" num="123" truthy="true" falsy="false"></span>
    </div>"#,
    );
}

#[test]
fn bind_object_overrides_existing_attr() {
    assert_render_body_eq!(
        r#"<div>
        <span str="old" v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "str": "hello",
                "num": 123,
                "truthy": true,
                "falsy": false,
                "nullish": null,
            },
        }),
        r#"<div>
        <span str="hello" num="123" truthy="true" falsy="false">elem</span>
    </div>"#,
    );
}

#[test]
fn bind_object_literal() {
    assert_render_body_eq!(
        r#"<div>
        <span v-bind="{ id, value: value * 2, hidden: null, skip: undefined, ok: false }">elem</span>
    </div>"#,
        json!({
            "id": "title",
            "value": 333,
        }),
        r#"<div>
        <span id="title" value="666" ok="false">elem</span>
    </div>"#,
    );
}

// === Class / Style Normalization ===

#[test]
fn bind_class_merges_static_and_object() {
    assert_render_body_eq!(
        r#"<div>
        <p class="base" :class="{ active: true, hidden: false, titled: id }">elem</p>
    </div>"#,
        json!({
            "id": "title",
        }),
        r#"<div>
        <p class="base active titled">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_class_array() {
    assert_render_body_eq!(
        r#"<div>
        <p :class="['btn', [id, { active: true, hidden: false }], null]">elem</p>
    </div>"#,
        json!({
            "id": "title",
        }),
        r#"<div>
        <p class="btn title active">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_object_class_merges_static() {
    assert_render_body_eq!(
        r#"<div>
        <p class="base" v-bind="{ class: ['from-bind', { active: true }], id }">elem</p>
    </div>"#,
        json!({
            "id": "title",
        }),
        r#"<div>
        <p class="base from-bind active" id="title">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_merges_static_and_object() {
    assert_render_body_eq!(
        r#"<div>
        <p style="color: red" :style="{ fontSize: '12px', 'line-height': 1.5, '--gap': '4px', display: null }">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="color: red; font-size: 12px; line-height: 1.5; --gap: 4px;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_array() {
    assert_render_body_eq!(
        r#"<div>
        <p :style="['color: red', { fontSize: '12px' }, { marginTop: value + 'px' }]">elem</p>
    </div>"#,
        json!({
            "value": 333,
        }),
        r#"<div>
        <p style="color: red; font-size: 12px; margin-top: 333px;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_object_style_merges_static() {
    assert_render_body_eq!(
        r#"<div>
        <p style="color: red;" v-bind="{ style: [{ fontSize: '12px' }, 'background: blue'], id }">elem</p>
    </div>"#,
        json!({
            "id": "title",
        }),
        r#"<div>
        <p style="color: red; font-size: 12px; background: blue" id="title">elem</p>
    </div>"#,
    );
}
