mod helper;

use helper::assert_render_body_eq;
use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::json;

// === Basic Binding ===

#[test]
fn bind_args() {
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
fn bind_shorthand() {
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
fn bind_dynamic_arg() {
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
fn bind_dynamic_arg_empty() {
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
fn bind_unclosed_arg() {
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
fn bind_dynamic_arg_lowercase() {
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
fn bind_expr() {
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
fn bind_nullish_removed() {
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
fn bind_spread() {
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
fn bind_spread_overrides() {
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
fn bind_spread_literal() {
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

#[test]
fn bind_spread_drops_non_primitive() {
    assert_render_body_eq!(
        r#"<div>
        <span v-bind="{ attrs: { key: 'val' }, list: [1, 2] }">elem</span>
    </div>"#,
        json!({}),
        r#"<div>
        <span>elem</span>
    </div>"#,
    );
}

#[test]
fn bind_arg_drops_non_primitive() {
    assert_render_body_eq!(
        r#"<div>
        <span :attrs="{ key: 'val' }" :list="[1, 2]">elem</span>
    </div>"#,
        json!({}),
        r#"<div>
        <span>elem</span>
    </div>"#,
    );
}

// === Class / Style Normalization ===

#[test]
fn bind_class_static_object() {
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
fn bind_spread_class() {
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
fn bind_style_static_object() {
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
fn bind_spread_style() {
    assert_render_body_eq!(
        r#"<div>
        <p style="color: red;" v-bind="{ style: [{ fontSize: '12px' }, 'background: blue'], id }">elem</p>
    </div>"#,
        json!({
            "id": "title",
        }),
        r#"<div>
        <p style="color: red; font-size: 12px; background: blue;" id="title">elem</p>
    </div>"#,
    );
}

// === Attribute Name Validation ===

#[test]
fn bind_dynamic_empty_name() {
    assert_render_body_eq!(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "",
            "value": "ok",
        }),
        r#"<div>
        <p>elem</p>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_single_space() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": " ",
            "value": "ok",
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == " "));
}

#[test]
fn bind_dynamic_space_error() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "space name",
            "value": "ok",
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "space name"));
}

#[test]
fn bind_dynamic_quote_name() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "quote\"name",
            "value": "ok",
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "quote\"name"));
}

#[test]
fn bind_dynamic_slash_error() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "slash/name",
            "value": "ok",
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "slash/name"));
}

#[test]
fn bind_dynamic_lt_name() {
    assert_render_body_eq!(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "<",
            "value": "ok",
        }),
        r#"<div>
        <p <="ok">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_lt_in_name() {
    assert_render_body_eq!(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "less<name",
            "value": "ok",
        }),
        r#"<div>
        <p less<name="ok">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_gt_error() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": ">",
            "value": "ok",
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == ">"));
}

#[test]
fn bind_dynamic_gt_in_name() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({
            "name": "greater>name",
            "value": "ok",
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "greater>name"));
}

#[test]
fn bind_spread_empty_name() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name.is_empty()));
}

#[test]
fn bind_spread_single_space() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                " ": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == " "));
}

#[test]
fn bind_spread_space_error() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "space name": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "space name"));
}

#[test]
fn bind_spread_quote_name() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "quote\"name": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "quote\"name"));
}

#[test]
fn bind_spread_slash_error() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "slash/name": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "slash/name"));
}

#[test]
fn bind_spread_lt_name() {
    assert_render_body_eq!(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "<": "value",
            },
        }),
        r#"<div>
        <span <="value">elem</span>
    </div>"#,
    );
}

#[test]
fn bind_spread_lt_in_name() {
    assert_render_body_eq!(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "less<name": "value",
            },
        }),
        r#"<div>
        <span less<name="value">elem</span>
    </div>"#,
    );
}

#[test]
fn bind_spread_gt_error() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                ">": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == ">"));
}

#[test]
fn bind_spread_gt_in_name() {
    let err = render(
        r#"<div>
        <span v-bind="attrs">elem</span>
    </div>"#,
        json!({
            "attrs": {
                "greater>name": "value",
            },
        }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "greater>name"));
}

// === Boolean Attributes ===

#[test]
fn bind_boolean_falsy_removed() {
    assert_render_body_eq!(
        r#"<div>
        <button :disabled="no" :checked="nothing" :hidden="zero">b</button>
    </div>"#,
        json!({ "no": false, "nothing": null, "zero": 0 }),
        r#"<div>
        <button>b</button>
    </div>"#,
    );
}

#[test]
fn bind_boolean_truthy_present() {
    // The empty string counts as present, which is the `readonly=""` idiom.
    assert_render_body_eq!(
        r#"<div>
        <button :disabled="yes" :hidden="text" :multiple="one" :readonly="blank">b</button>
    </div>"#,
        json!({ "yes": true, "text": "no", "one": 1, "blank": "" }),
        r#"<div>
        <button disabled="" hidden="" multiple="" readonly="">b</button>
    </div>"#,
    );
}

#[test]
fn bind_boolean_non_primitive_removed() {
    // Truthy in JavaScript, but rejected before the boolean rule runs.
    assert_render_body_eq!(
        r#"<div>
        <button :disabled="list">b</button>
    </div>"#,
        json!({ "list": [] }),
        r#"<div>
        <button>b</button>
    </div>"#,
    );
}

#[test]
fn bind_boolean_via_spread() {
    assert_render_body_eq!(
        r#"<div>
        <button v-bind="{ disabled: false, required: true }">b</button>
    </div>"#,
        json!({}),
        r#"<div>
        <button required="">b</button>
    </div>"#,
    );
}

#[test]
fn bind_boolean_via_dynamic_arg() {
    assert_render_body_eq!(
        r#"<div>
        <button :[key]="no">b</button>
    </div>"#,
        json!({ "key": "disabled", "no": false }),
        r#"<div>
        <button>b</button>
    </div>"#,
    );
}

// === Non-Primitive Values ===

#[test]
fn bind_non_primitive_removed() {
    assert_render_body_eq!(
        r#"<div>
        <span :a="obj" :b="list" :c="sym" :d="big" :e="fun">elem</span>
    </div>"#,
        json!({ "obj": { "k": 1 }, "list": [1, 2] }),
        r#"<div>
        <span>elem</span>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_arg_non_primitive_removed() {
    assert_render_body_eq!(
        r#"<div>
        <span :[key]="obj">elem</span>
    </div>"#,
        json!({ "key": "data-x", "obj": { "k": 1 } }),
        r#"<div>
        <span>elem</span>
    </div>"#,
    );
}

#[test]
fn bind_dynamic_arg_normalizes_class() {
    assert_render_body_eq!(
        r#"<div>
        <p class="base" :[key]="{ active: true, hidden: false }">elem</p>
    </div>"#,
        json!({ "key": "class" }),
        r#"<div>
        <p class="base active">elem</p>
    </div>"#,
    );
}

// === Modifiers ===

#[test]
fn bind_camel_modifier() {
    assert_render_body_eq!(
        r#"<div>
        <svg :view-box.camel="box"></svg>
    </div>"#,
        json!({ "box": "0 0 9 9" }),
        r#"<div>
        <svg viewBox="0 0 9 9"></svg>
    </div>"#,
    );
}

#[test]
fn bind_camel_modifier_on_dynamic_arg() {
    assert_render_body_eq!(
        r#"<div>
        <svg :[key].camel="box"></svg>
    </div>"#,
        json!({ "key": "view-box", "box": "0 0 9 9" }),
        r#"<div>
        <svg viewBox="0 0 9 9"></svg>
    </div>"#,
    );
}

#[test]
fn bind_unknown_modifier_errors() {
    let err = render(
        r#"<div>
        <span :foo.sync="value">elem</span>
    </div>"#,
        json!({ "value": "x" }),
    )
    .unwrap_err();

    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Bind,
            kind: DirectiveErrorKind::UnknownModifier,
            expression: Some(name),
        } if name == "sync"
    ));
}

#[test]
fn bind_prop_modifier_renders_a_plain_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <span :foo.prop="value">elem</span>
    </div>"#,
        json!({ "value": "x" }),
        r#"<div>
        <span foo="x">elem</span>
    </div>"#,
    );
}

#[test]
fn bind_attr_modifier_renders_a_plain_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <span :foo.attr="value">elem</span>
    </div>"#,
        json!({ "value": "x" }),
        r#"<div>
        <span foo="x">elem</span>
    </div>"#,
    );
}

// === Static / Dynamic Collision ===

#[test]
fn bind_arg_shadows_static_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <a href="/fallback" :href="url">link</a>
    </div>"#,
        json!({ "url": "/real" }),
        r#"<div>
        <a href="/real">link</a>
    </div>"#,
    );
}

#[test]
fn bind_arg_shadows_static_attribute_that_precedes_it() {
    assert_render_body_eq!(
        r#"<div>
        <a href="/before" :href="url" title="keep">link</a>
    </div>"#,
        json!({ "url": "/real" }),
        r#"<div>
        <a href="/real" title="keep">link</a>
    </div>"#,
    );
}

#[test]
fn bind_merges_and_overrides_in_one_pass() {
    // A merged name and an overridden one both keep their first position.
    assert_render_body_eq!(
        r#"<div>
        <a href="/before" class="base" :class="{ on: true }" :href="url">link</a>
    </div>"#,
        json!({ "url": "/real" }),
        r#"<div>
        <a href="/real" class="base on">link</a>
    </div>"#,
    );
}

#[test]
fn bind_camel_modifier_shadows_static_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <svg viewBox="0 0 1 1" :view-box.camel="box"></svg>
    </div>"#,
        json!({ "box": "0 0 9 9" }),
        r#"<div>
        <svg viewBox="0 0 9 9"></svg>
    </div>"#,
    );
}

#[test]
fn bind_removed_binding_leaves_static_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <a href="/fallback" :href="missing">link</a>
    </div>"#,
        json!({ "missing": null }),
        r#"<div>
        <a href="/fallback">link</a>
    </div>"#,
    );
}

#[test]
fn later_binding_wins_over_earlier_binding() {
    // Neither may delete the other outright, or no href would survive.
    assert_render_body_eq!(
        r#"<div>
        <a :href="first" v-bind:href="second">link</a>
    </div>"#,
        json!({ "first": "/first", "second": "/second" }),
        r#"<div>
        <a href="/second">link</a>
    </div>"#,
    );
}

#[test]
fn later_binding_wins_across_argument_forms() {
    assert_render_body_eq!(
        r#"<div>
        <a href="/static" :[key]="dynamic" :href="literal" v-bind:href="last">link</a>
    </div>"#,
        json!({
            "key": "href",
            "dynamic": "/dynamic",
            "literal": "/literal",
            "last": "/last",
        }),
        r#"<div>
        <a href="/last">link</a>
    </div>"#,
    );
}

#[test]
fn dropped_later_binding_leaves_the_earlier_one() {
    assert_render_body_eq!(
        r#"<div>
        <a :href="url" v-bind:href="missing">link</a>
    </div>"#,
        json!({ "url": "/real", "missing": null }),
        r#"<div>
        <a href="/real">link</a>
    </div>"#,
    );
}

#[test]
fn bind_spread_loses_to_a_later_binding() {
    assert_render_body_eq!(
        r#"<div>
        <a v-bind="{ href: '/spread' }" :href="url">a</a>
        <a :href="url" v-bind="{ href: '/spread' }">b</a>
    </div>"#,
        json!({ "url": "/real" }),
        r#"<div>
        <a href="/real">a</a>
        <a href="/spread">b</a>
    </div>"#,
    );
}

#[test]
fn bind_spread_loses_to_a_later_static_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <a v-bind="{ href: '/spread' }" href="/static">link</a>
    </div>"#,
        json!({}),
        r#"<div>
        <a href="/static">link</a>
    </div>"#,
    );
}

#[test]
fn bind_class_and_style_merge_in_source_order() {
    assert_render_body_eq!(
        r#"<div>
        <p :class="'b'" class="a" :style="{ color: 'blue' }" style="color: red">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p class="b a" style="color: red;">x</p>
    </div>"#,
    );
}

// === Style Merging ===
//
// Vue folds a static `style`, every array item and every object into one
// declaration list, so a repeated property keeps its place but takes the newer
// value instead of being emitted twice.

#[test]
fn bind_style_array_merges_duplicate_keys() {
    assert_render_body_eq!(
        r#"<div>
        <p :style="[{ color: 'red' }, { color: 'blue' }]">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="color: blue;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_overrides_the_static_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <p style="color: red; margin: 0" :style="{ color: 'blue' }">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="color: blue; margin: 0;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_parses_string_items() {
    assert_render_body_eq!(
        r#"<div>
        <p :style="['color: red; margin: 0', { color: 'blue' }]">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="color: blue; margin: 0;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_keeps_semicolons_inside_parentheses() {
    assert_render_body_eq!(
        r#"<div>
        <p :style="['background: url(a;b); color: red']">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="background: url(a;b); color: red;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_drops_comments_and_non_primitives() {
    assert_render_body_eq!(
        r#"<div>
        <p :style="['/* note */ color: red', { margin: [1, 2] }]">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="color: red;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_hyphenates_without_a_leading_dash() {
    // Vue's `\B([A-Z])` leaves the first character alone.
    assert_render_body_eq!(
        r#"<div>
        <p :style="{ MozTransform: 'none', fontSize: '1px', '--gap': '2px' }">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="moz-transform: none; font-size: 1px; --gap: 2px;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_hyphenates_a_key_parsed_from_css() {
    assert_render_body_eq!(
        r#"<div>
        <p :style="['backgroundColor: red']">a</p>
        <p style="backgroundColor: red" :style="{ color: 'x' }">b</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="background-color: red;">a</p>
        <p style="background-color: red; color: x;">b</p>
    </div>"#,
    );
}

#[test]
fn bind_style_normalizes_a_static_attribute() {
    assert_render_body_eq!(
        r#"<div>
        <p style="color: red">a</p>
        <p style="marginTop: 1px">b</p>
        <p style="color: a; color: b">c</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="color: red;">a</p>
        <p style="margin-top: 1px;">b</p>
        <p style="color: b;">c</p>
    </div>"#,
    );
}

#[test]
fn bind_style_keeps_an_unparseable_static_attribute() {
    // Vue empties it instead; prevue never rewrites what it could not read.
    assert_render_body_eq!(
        r#"<div>
        <p style="">a</p>
        <p style>b</p>
        <p style="???">c</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="">a</p>
        <p style="">b</p>
        <p style="???">c</p>
    </div>"#,
    );
}

#[test]
fn bind_style_hyphenates_only_after_a_word_character() {
    // Vue's `\B` fails right after `-`, `.` or a space, so no hyphen is added
    // there.
    assert_render_body_eq!(
        r#"<div>
        <p :style="{ 'a-Bc': 1, 'a.B': 1, 'a B': 1, '-Webkit-x': 1 }">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="a-bc: 1; a.b: 1; a b: 1; -webkit-x: 1;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_lowercases_the_whole_key() {
    // Vue ends with `toLowerCase()`, which reaches past ASCII.
    assert_render_body_eq!(
        r#"<div>
        <p :style="{ 'Ä': 1, fooBAR: 'x', a_B: 'y', x1Y: 'w' }">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="ä: 1; foo-b-a-r: x; a_-b: y; x1-y: w;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_keeps_custom_property_case() {
    // The `--` exemption belongs to the stringify step, not to `hyphenate`.
    assert_render_body_eq!(
        r#"<div>
        <p :style="{ '--Gap': 1, '--gap': 2 }">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="--Gap: 1; --gap: 2;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_style_hyphenates_late_enough_to_keep_spellings_apart() {
    // Vue merges into a JavaScript object first, where the two are distinct
    // keys, and only hyphenates on the way out.
    assert_render_body_eq!(
        r#"<div>
        <p :style="['background-color: a', { backgroundColor: 'b' }]">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p style="background-color: a; background-color: b;">elem</p>
    </div>"#,
    );
}

#[test]
fn bind_apostrophe_in_attribute_name_errors() {
    let err = render(
        r#"<div>
        <p :[name]="value">elem</p>
    </div>"#,
        json!({ "name": "a'b", "value": "x" }),
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAttributeName { name } if name == "a'b"));
}

// === Props Vue Never Writes ===

#[test]
fn static_vue_internal_props_are_dropped() {
    // Vue filters these by name, however they were written.
    assert_render_body_eq!(
        r#"<div>
        <p key="k" ref="r" ref_key="rk" ref_for="rf" id="keep">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p id="keep">x</p>
    </div>"#,
    );
}

#[test]
fn bind_spread_skips_vue_internal_props() {
    assert_render_body_eq!(
        r#"<div>
        <p v-bind="{ key: 1, ref: 'r', innerHTML: 'h', textContent: 't', ref_key: 'k', ref_for: 'f', onClick: 'c' }">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>x</p>
    </div>"#,
    );
}

#[test]
fn bind_spread_keeps_a_lowercase_handler_attribute() {
    // `isOn` is /^on[^a-z]/, so plain `onclick` is a real attribute.
    assert_render_body_eq!(
        r#"<div>
        <p v-bind="{ onclick: 'go()' }">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p onclick="go()">x</p>
    </div>"#,
    );
}

#[test]
fn bind_arg_skips_vue_internal_props() {
    assert_render_body_eq!(
        r#"<div>
        <p :key="1" :ref="'r'">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>x</p>
    </div>"#,
    );
}

// === Directives With Nothing to Render ===

#[test]
fn unrendered_vue_directives_are_removed() {
    // A directive is never an attribute, and the template a client framework
    // would need is already gone by the time these reach the output.
    assert_render_body_eq!(
        r#"<div>
        <button @click="a">b</button>
        <button v-on:click.prevent="a">c</button>
        <p v-once>{{ a }}</p>
        <p v-memo="[a]">e</p>
        <p v-cloak>f</p>
        <p v-slot:x>g</p>
        <p #y>h</p>
    </div>"#,
        json!({ "a": "A" }),
        r#"<div>
        <button>b</button>
        <button>c</button>
        <p>A</p>
        <p>e</p>
        <p>f</p>
        <p>g</p>
        <p>h</p>
    </div>"#,
    );
}

#[test]
fn a_removed_directive_leaves_other_attributes_alone() {
    assert_render_body_eq!(
        r#"<div>
        <button id="a" @click="a" :class="'c'" type="button">x</button>
    </div>"#,
        json!({ "a": "A" }),
        r#"<div>
        <button id="a" class="c" type="button">x</button>
    </div>"#,
    );
}

#[test]
fn an_attribute_merely_containing_v_dash_is_not_a_directive() {
    assert_render_body_eq!(
        r#"<div>
        <p data-v-foo="x" av-bar="y">elem</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p data-v-foo="x" av-bar="y">elem</p>
    </div>"#,
    );
}

// === Non-Object v-bind ===

#[test]
fn bind_non_object_leaves_no_directive() {
    for expr in ["'str'", "null", "1", "undefined", "nope"] {
        let out = render(format!(r#"<p v-bind="{expr}">x</p>"#), json!({})).unwrap();
        assert!(!out.contains("v-bind"), "{expr}: {out}");
    }
}

// === Attribute Name Normalization ===

#[test]
fn bind_spread_maps_prop_names() {
    assert_render_body_eq!(
        r#"<div>
        <p v-bind="{ className: 'c', htmlFor: 'f', httpEquiv: 'e', acceptCharset: 'utf-8' }">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p class="c" for="f" http-equiv="e" accept-charset="utf-8">x</p>
    </div>"#,
    );
}

#[test]
fn bind_spread_class_name_merges_with_static_class() {
    assert_render_body_eq!(
        r#"<div>
        <p class="base" v-bind="{ className: 'extra' }">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p class="base extra">x</p>
    </div>"#,
    );
}

#[test]
fn bind_spread_lowercases_on_html_elements() {
    assert_render_body_eq!(
        r#"<div>
        <button v-bind="{ Disabled: false, viewBox: 'x' }">b</button>
    </div>"#,
        json!({}),
        r#"<div>
        <button viewbox="x">b</button>
    </div>"#,
    );
}

#[test]
fn bind_spread_preserves_case_on_svg_and_custom_elements() {
    assert_render_body_eq!(
        r#"<div>
        <svg v-bind="{ viewBox: '0 0 9 9' }"></svg>
        <my-el v-bind="{ fooBar: '1' }"></my-el>
    </div>"#,
        json!({}),
        r#"<div>
        <svg viewBox="0 0 9 9"></svg>
        <my-el fooBar="1"></my-el>
    </div>"#,
    );
}

#[test]
fn bind_camel_is_undone_outside_svg() {
    // `.camel` exists for SVG; on an HTML element Vue folds the name back down.
    assert_render_body_eq!(
        r#"<div>
        <div :view-box.camel="b"></div>
    </div>"#,
        json!({ "b": "x" }),
        r#"<div>
        <div viewbox="x"></div>
    </div>"#,
    );
}

// === Textarea ===

#[test]
fn bind_textarea_value_becomes_content() {
    assert_render_body_eq!(
        r#"<div>
        <textarea :value="v"></textarea>
        <textarea v-bind="{ value: v }"></textarea>
    </div>"#,
        json!({ "v": "hello" }),
        r#"<div>
        <textarea>hello</textarea>
        <textarea>hello</textarea>
    </div>"#,
    );
}

#[test]
fn bind_textarea_value_takes_a_non_primitive() {
    // The value becomes content, so the rule about what may go in an attribute
    // does not reach it.
    assert_render_body_eq!(
        r#"<div>
        <textarea :value="v"></textarea>
        <textarea v-bind="{ value: v }"></textarea>
    </div>"#,
        json!({ "v": { "a": 1 } }),
        r#"<div>
        <textarea>[object Object]</textarea>
        <textarea>[object Object]</textarea>
    </div>"#,
    );
}

#[test]
fn bind_textarea_value_is_not_interpolated_again() {
    assert_render_body_eq!(
        r#"<div>
        <textarea :value="v"></textarea>
    </div>"#,
        json!({ "v": "{{ 1 + 1 }}" }),
        r#"<div>
        <textarea>{{ 1 + 1 }}</textarea>
    </div>"#,
    );
}

#[test]
fn bind_spread_skips_dom_property_keys() {
    // `.foo` names a DOM property, which has no HTML spelling.
    assert_render_body_eq!(
        r#"<div>
        <p v-bind="{ '.foo': 1 }">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>x</p>
    </div>"#,
    );
}

#[test]
fn bind_spread_drops_the_forced_attribute_marker() {
    assert_render_body_eq!(
        r#"<div>
        <p v-bind="{ '^foo': 1 }">x</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p foo="1">x</p>
    </div>"#,
    );
}

#[test]
fn bind_camel_matches_the_regex_on_repeated_hyphens() {
    // Vue's /-(\w)/g retries at the next position when a match fails, so the
    // second hyphen of `a--b` is the one that pairs with `b`.
    assert_render_body_eq!(
        r#"<div>
        <svg :a--b.camel="v" :c-.camel="v" :-d.camel="v"></svg>
    </div>"#,
        json!({ "v": "x" }),
        r#"<div>
        <svg a-B="x" c-="x" D="x"></svg>
    </div>"#,
    );
}
