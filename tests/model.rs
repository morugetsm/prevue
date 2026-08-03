mod helper;

use helper::assert_render_body_eq;
use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::json;

// === Text Inputs ===

#[test]
fn model_fills_the_value() {
    assert_render_body_eq!(
        r#"<div><input v-model="name"></div>"#,
        json!({ "name": "Ada" }),
        r#"<div><input value="Ada"></div>"#,
    );
}

#[test]
fn model_without_a_value_writes_nothing() {
    assert_render_body_eq!(
        r#"<div><input v-model="missing"></div>"#,
        json!({ "missing": null }),
        r#"<div><input></div>"#,
    );
}

#[test]
fn model_overrides_a_static_value() {
    // Vue writes `value` twice here; folding by name leaves the model's.
    assert_render_body_eq!(
        r#"<div><input value="static" v-model="name"></div>"#,
        json!({ "name": "Ada" }),
        r#"<div><input value="Ada"></div>"#,
    );
}

#[test]
fn model_writes_a_number() {
    assert_render_body_eq!(
        r#"<div><input v-model="size"></div>"#,
        json!({ "size": 3.5 }),
        r#"<div><input value="3.5"></div>"#,
    );
}

// === Radio ===

#[test]
fn model_radio_checks_the_matching_value() {
    assert_render_body_eq!(
        r#"<div>
        <input type="radio" value="A" v-model="pick">
        <input type="radio" value="B" v-model="pick">
    </div>"#,
        json!({ "pick": "A" }),
        r#"<div>
        <input type="radio" value="A" checked="">
        <input type="radio" value="B">
    </div>"#,
    );
}

#[test]
fn model_radio_without_a_value_compares_against_null() {
    assert_render_body_eq!(
        r#"<div><input type="radio" v-model="pick"></div>"#,
        json!({ "pick": null }),
        r#"<div><input type="radio" checked=""></div>"#,
    );
}

#[test]
fn model_radio_reads_a_bound_value() {
    assert_render_body_eq!(
        r#"<div><input type="radio" :value="pick" v-model="pick"></div>"#,
        json!({ "pick": "A" }),
        r#"<div><input type="radio" value="A" checked=""></div>"#,
    );
}

// === Checkbox ===

#[test]
fn model_checkbox_uses_truthiness() {
    assert_render_body_eq!(
        r#"<div>
        <input type="checkbox" v-model="on">
        <input type="checkbox" v-model="off">
    </div>"#,
        json!({ "on": true, "off": false }),
        r#"<div>
        <input type="checkbox" checked="">
        <input type="checkbox">
    </div>"#,
    );
}

#[test]
fn model_checkbox_looks_inside_an_array() {
    assert_render_body_eq!(
        r#"<div>
        <input type="checkbox" value="p" v-model="picks">
        <input type="checkbox" value="z" v-model="picks">
    </div>"#,
        json!({ "picks": ["p", "q"] }),
        r#"<div>
        <input type="checkbox" value="p" checked="">
        <input type="checkbox" value="z">
    </div>"#,
    );
}

#[test]
fn model_checkbox_compares_against_true_value() {
    // Vue reads `true-value` and `false-value` off the element and never
    // writes them back.
    assert_render_body_eq!(
        r#"<div>
        <input type="checkbox" true-value="A" false-value="n" v-model="pick">
        <input type="checkbox" true-value="Z" false-value="n" v-model="pick">
    </div>"#,
        json!({ "pick": "A" }),
        r#"<div>
        <input type="checkbox" checked="">
        <input type="checkbox">
    </div>"#,
    );
}

// === Loose Comparison ===
//
// An attribute value is always a string, so Vue's `looseEqual` falls through to
// comparing both sides as strings.

#[test]
fn model_compares_a_number_against_the_attribute_string() {
    assert_render_body_eq!(
        r#"<div>
        <input type="radio" value="1" v-model="pick">
        <input type="radio" value="2" v-model="pick">
    </div>"#,
        json!({ "pick": 1 }),
        r#"<div>
        <input type="radio" value="1" checked="">
        <input type="radio" value="2">
    </div>"#,
    );
}

#[test]
fn model_compares_a_boolean_against_the_attribute_string() {
    assert_render_body_eq!(
        r#"<div><input type="radio" value="true" v-model="flag"></div>"#,
        json!({ "flag": true }),
        r#"<div><input type="radio" value="true" checked=""></div>"#,
    );
}

#[test]
fn model_compares_array_items_loosely() {
    assert_render_body_eq!(
        r#"<div>
        <input type="checkbox" value="1" v-model="picks">
        <input type="checkbox" value="9" v-model="picks">
    </div>"#,
        json!({ "picks": [1, 2] }),
        r#"<div>
        <input type="checkbox" value="1" checked="">
        <input type="checkbox" value="9">
    </div>"#,
    );
}

#[test]
fn model_select_compares_loosely() {
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><option value="1">a</option><option value="2">b</option></select></div>"#,
        json!({ "pick": 1 }),
        r#"<div><select><option value="1" selected="">a</option><option value="2">b</option></select></div>"#,
    );
}

// === Bound Type ===

#[test]
fn model_reads_the_resolved_type() {
    assert_render_body_eq!(
        r#"<div><input :type="kind" value="A" v-model="pick"></div>"#,
        json!({ "kind": "radio", "pick": "A" }),
        r#"<div><input type="radio" value="A" checked=""></div>"#,
    );
}

// === Textarea ===

#[test]
fn model_textarea_becomes_content() {
    assert_render_body_eq!(
        r#"<div><textarea v-model="note">original</textarea></div>"#,
        json!({ "note": "Ada" }),
        r#"<div><textarea>Ada</textarea></div>"#,
    );
}

// === Select ===

#[test]
fn model_select_marks_the_matching_option() {
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><option value="A">a</option><option value="B">b</option></select></div>"#,
        json!({ "pick": "A" }),
        r#"<div><select><option value="A" selected="">a</option><option value="B">b</option></select></div>"#,
    );
}

#[test]
fn model_select_looks_inside_an_array() {
    // The `multiple` attribute is not consulted; the model's type decides.
    assert_render_body_eq!(
        r#"<div><select multiple v-model="picks"><option value="p">a</option><option value="z">b</option></select></div>"#,
        json!({ "picks": ["p", "q"] }),
        r#"<div><select multiple=""><option value="p" selected="">a</option><option value="z">b</option></select></div>"#,
    );
}

#[test]
fn model_select_descends_through_optgroup() {
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><optgroup label="g"><option value="A">a</option></optgroup></select></div>"#,
        json!({ "pick": "A" }),
        r#"<div><select><optgroup label="g"><option value="A" selected="">a</option></optgroup></select></div>"#,
    );
}

#[test]
fn model_select_stops_at_any_other_element() {
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><div><option value="A">a</option></div></select></div>"#,
        json!({ "pick": "A" }),
        r#"<div><select><div><option value="A">a</option></div></select></div>"#,
    );
}

#[test]
fn model_select_keeps_an_existing_selection() {
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><option value="B" selected>a</option><option value="A">b</option></select></div>"#,
        json!({ "pick": "A" }),
        r#"<div><select><option value="B" selected="">a</option><option value="A" selected="">b</option></select></div>"#,
    );
}

#[test]
fn model_select_needs_a_value_attribute() {
    // A browser falls back to the option's text; Vue does not.
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><option>A</option></select></div>"#,
        json!({ "pick": "A" }),
        r#"<div><select><option>A</option></select></div>"#,
    );
}

#[test]
fn model_select_sees_options_from_a_loop() {
    assert_render_body_eq!(
        r#"<div><select v-model="pick"><option v-for="o in opts" :value="o">{{ o }}</option></select></div>"#,
        json!({ "pick": "A", "opts": ["A", "B"] }),
        r#"<div><select><option value="A" selected="">A</option><option value="B">B</option></select></div>"#,
    );
}

// === Modifiers ===

#[test]
fn model_ignores_input_modifiers() {
    // All three describe how input is read back, which the server never does.
    assert_render_body_eq!(
        r#"<div><input v-model.trim.number.lazy="name"></div>"#,
        json!({ "name": "Ada" }),
        r#"<div><input value="Ada"></div>"#,
    );
}

// === Errors ===

#[test]
fn model_without_an_expression_errors() {
    let err = render("<input v-model>", json!({})).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Model,
            kind: DirectiveErrorKind::MissingExpression,
            ..
        }
    ));
}

#[test]
fn model_on_a_plain_element_errors() {
    let err = render(r#"<div v-model="name"></div>"#, json!({ "name": "Ada" })).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Model,
            kind: DirectiveErrorKind::UnsupportedElement,
            ..
        }
    ));
}

#[test]
fn model_on_a_file_input_errors() {
    let err = render(
        r#"<input type="file" v-model="name">"#,
        json!({ "name": "Ada" }),
    )
    .unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Model,
            kind: DirectiveErrorKind::UnsupportedElement,
            ..
        }
    ));
}

#[test]
fn model_unknown_modifier_errors() {
    let err = render(r#"<input v-model.sync="name">"#, json!({ "name": "Ada" })).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Model,
            kind: DirectiveErrorKind::UnknownModifier,
            expression: Some(name),
        } if name == "sync"
    ));
}
