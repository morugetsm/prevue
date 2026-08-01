use prevue::{Renderer, render};
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
