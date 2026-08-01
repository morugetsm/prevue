use prevue::{Renderer, Template, render};
use serde_json::json;

// === Data isolation ===

#[test]
fn data_is_replaced_between_renders() {
    let mut renderer = Renderer::new().unwrap();
    let template = "<p>[{{ first }}][{{ second }}]</p>";

    let one = renderer.render(template, json!({ "first": "A" })).unwrap();
    let two = renderer.render(template, json!({ "second": "B" })).unwrap();

    assert!(one.contains("[A][]"), "{one}");
    assert!(two.contains("[][B]"), "{two}");
}

#[test]
fn data_alias_is_replaced_between_renders() {
    let mut renderer = Renderer::new().unwrap();
    let template = "<p>[{{ $.first }}]</p>";

    let one = renderer.render(template, json!({ "first": "A" })).unwrap();
    let two = renderer.render(template, json!({ "second": "B" })).unwrap();

    assert!(one.contains("[A]"), "{one}");
    assert!(two.contains("[]"), "{two}");
}

#[test]
fn nested_for_bindings_do_not_survive_a_render() {
    let mut renderer = Renderer::new().unwrap();

    renderer
        .render(r#"<p v-for="item in [1, 2]">{{ item }}</p>"#, json!({}))
        .unwrap();
    let after = renderer
        .render("<p>{{ typeof item }}</p>", json!({}))
        .unwrap();

    assert!(after.contains("undefined"), "{after}");
}

// === Setup script isolation ===

#[test]
fn setup_lexical_declarations_do_not_leak() {
    let mut renderer = Renderer::new().unwrap();

    renderer
        .render(
            r#"<script type="prevue">const helper = () => 'x'; class K {}</script>"#,
            json!({}),
        )
        .unwrap();
    let after = renderer
        .render("<p>{{ typeof helper }}/{{ typeof K }}</p>", json!({}))
        .unwrap();

    assert!(after.contains("undefined/undefined"), "{after}");
}

#[test]
fn setup_var_and_function_declarations_do_not_leak() {
    // `var` and (Annex B) function declarations would otherwise hoist onto
    // `globalThis` and outlive the render; `eval_setup` scopes them.
    let mut renderer = Renderer::new().unwrap();

    renderer
        .render(
            r#"<script type="prevue">var v = 1; function f() { return 2; }</script>"#,
            json!({}),
        )
        .unwrap();
    let after = renderer
        .render("<p>{{ typeof v }}/{{ typeof f }}</p>", json!({}))
        .unwrap();

    assert!(after.contains("undefined/undefined"), "{after}");
}

#[test]
fn setup_script_still_works_within_one_render() {
    let mut renderer = Renderer::new().unwrap();
    let template = r#"<script type="prevue">
        var prefix = 'hi';
        function greet(name) { return `${prefix}, ${name}`; }
    </script><p>{{ greet(user) }}</p>"#;

    let one = renderer.render(template, json!({ "user": "Ada" })).unwrap();
    let two = renderer
        .render(template, json!({ "user": "Grace" }))
        .unwrap();

    assert!(one.contains("hi, Ada"), "{one}");
    assert!(two.contains("hi, Grace"), "{two}");
}

// === Recovery ===

#[test]
fn renderer_recovers_after_error() {
    let mut renderer = Renderer::new().unwrap();

    // Fails while a `v-for` scope is open.
    let failed = renderer.render(
        r#"<div v-for="i in [1, 2]"><p v-if="">x</p></div>"#,
        json!({}),
    );
    assert!(failed.is_err());

    let after = renderer
        .render(r#"<p v-for="i in [1, 2]">{{ i }}</p>"#, json!({}))
        .unwrap();
    assert!(after.contains("<p>1</p><p>2</p>"), "{after}");
}

// === Equivalence with the free function ===

#[test]
fn renderer_matches_free_function() {
    let template = r#"<div>
        <a :id="id">link</a>
        <p v-if="user.age >= 18">{{ user.name }} is adult</p>
        <ul>
            <li v-for="item in list">{{ item }}</li>
        </ul>
    </div>"#;
    let data = json!({
        "id": "link-id",
        "user": { "name": "James", "age": 28 },
        "list": ["a", "b", "c"],
    });

    let mut renderer = Renderer::new().unwrap();
    assert_eq!(
        renderer.render(template, &data).unwrap(),
        render(template, &data).unwrap()
    );
}

#[test]
fn repeated_renders_are_stable() {
    let mut renderer = Renderer::new().unwrap();
    let template = r#"<ul><li v-for="n in list" :class="{ first: n === 1 }">{{ n }}</li></ul>"#;
    let data = json!({ "list": [1, 2, 3] });

    let first = renderer.render(template, &data).unwrap();
    for _ in 0..5 {
        assert_eq!(renderer.render(template, &data).unwrap(), first);
    }
}

// === Precompiled templates ===
//
// A render strips directives and rewrites children, so these all check that it
// worked on a copy and left the stored tree alone.

/// Covers every kind of mutation a render makes: attribute removal, attribute
/// rewriting, child replacement and text interpolation.
const MIXED: &str = r#"<div>
        <a :id="id" v-text="label">placeholder</a>
        <p v-if="user.age >= 18">{{ user.name }} is adult</p>
        <p v-else>{{ user.name }} is a minor</p>
        <ul>
            <li v-for="item, i in list" :class="{ first: i === 0 }">{{ item }}</li>
        </ul>
    </div>"#;

fn mixed_data(name: &str, age: u8) -> serde_json::Value {
    json!({
        "id": "link-id",
        "label": "link",
        "user": { "name": name, "age": age },
        "list": ["a", "b", "c"],
    })
}

#[test]
fn template_returns_to_an_earlier_result() {
    // Interleaved on purpose: repeating one input passes even on a frozen tree.
    let mut renderer = Renderer::new().unwrap();
    let template = Template::new(MIXED);
    let adult = mixed_data("James", 28);
    let minor = mixed_data("Annie", 12);

    let first = renderer.render_template(&template, &adult).unwrap();
    let second = renderer.render_template(&template, &minor).unwrap();
    let third = renderer.render_template(&template, &adult).unwrap();

    assert_ne!(first, second);
    assert_eq!(first, third);
}

#[test]
fn template_matches_render() {
    let data = mixed_data("James", 28);
    let mut renderer = Renderer::new().unwrap();

    assert_eq!(
        renderer
            .render_template(&Template::new(MIXED), &data)
            .unwrap(),
        render(MIXED, &data).unwrap()
    );
}

#[test]
fn template_takes_a_different_branch_on_new_data() {
    // A leaked `v-if` removal would strand the second render on the first branch.
    let mut renderer = Renderer::new().unwrap();
    let template = Template::new(MIXED);

    let adult = renderer
        .render_template(&template, mixed_data("James", 28))
        .unwrap();
    let minor = renderer
        .render_template(&template, mixed_data("Annie", 12))
        .unwrap();

    assert!(adult.contains("James is adult"), "{adult}");
    assert!(!adult.contains("minor"), "{adult}");
    assert!(minor.contains("Annie is a minor"), "{minor}");
    assert!(!minor.contains("adult"), "{minor}");
}

#[test]
fn cloned_template_is_independent() {
    // A clone shares the stored tree, so each side must still see its own data.
    let mut renderer = Renderer::new().unwrap();
    let template = Template::new(MIXED);
    let clone = template.clone();
    let adult = mixed_data("James", 28);
    let minor = mixed_data("Annie", 12);

    let from_clone = renderer.render_template(&clone, &adult).unwrap();
    let from_original = renderer.render_template(&template, &minor).unwrap();

    assert_eq!(from_clone, render(MIXED, &adult).unwrap());
    assert_eq!(from_original, render(MIXED, &minor).unwrap());
}

#[test]
fn template_element_survives_precompile() {
    // `template_contents` is the one branch of the copy that is not a subtree.
    let source = r#"<template v-for="n in list"><b>{{ n }}</b><i>x</i></template>"#;
    let mut renderer = Renderer::new().unwrap();
    let template = Template::new(source);

    let two = renderer
        .render_template(&template, json!({ "list": [1, 2] }))
        .unwrap();
    let one = renderer
        .render_template(&template, json!({ "list": [9] }))
        .unwrap();

    assert!(two.contains("<b>1</b><i>x</i><b>2</b><i>x</i>"), "{two}");
    assert!(one.contains("<b>9</b><i>x</i>"), "{one}");
    assert!(!one.contains("<b>1</b>"), "{one}");
}

#[test]
fn doctype_survives_precompile() {
    let source = "<!DOCTYPE html><html><body><p>{{ n }}</p></body></html>";
    let mut renderer = Renderer::new().unwrap();
    let template = Template::new(source);

    let first = renderer
        .render_template(&template, json!({ "n": 1 }))
        .unwrap();
    let second = renderer
        .render_template(&template, json!({ "n": 2 }))
        .unwrap();

    assert!(first.starts_with("<!DOCTYPE html>"), "{first}");
    assert!(second.starts_with("<!DOCTYPE html>"), "{second}");
    assert!(first.contains("<p>1</p>"), "{first}");
    assert!(second.contains("<p>2</p>"), "{second}");
}

#[test]
fn setup_script_runs_on_every_template_render() {
    // The script is removed from the copy, so the stored tree must still have it.
    let source = r#"<script type="prevue">function greet(n) { return `hi, ${n}`; }</script><p>{{ greet(user) }}</p>"#;
    let mut renderer = Renderer::new().unwrap();
    let template = Template::new(source);

    let one = renderer
        .render_template(&template, json!({ "user": "Ada" }))
        .unwrap();
    let two = renderer
        .render_template(&template, json!({ "user": "Grace" }))
        .unwrap();

    assert!(one.contains("hi, Ada"), "{one}");
    assert!(two.contains("hi, Grace"), "{two}");
    assert!(!two.contains("<script"), "{two}");
}

// === Documented limitation ===

#[test]
fn mustache_var_leaks_between_renders() {
    // Mustaches are evaluated for their completion value, so they cannot be
    // wrapped in a function the way setup scripts are. A `var` declared inside
    // one therefore becomes a real global and outlives the render.
    let mut renderer = Renderer::new().unwrap();

    renderer
        .render("<p>{{ var leaked = 7; leaked }}</p>", json!({}))
        .unwrap();
    let after = renderer
        .render("<p>{{ typeof leaked }}</p>", json!({}))
        .unwrap();

    assert!(after.contains("number"), "{after}");
}
