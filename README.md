# prevue

[![CI](https://github.com/morugetsm/prevue/actions/workflows/ci.yml/badge.svg)](https://github.com/morugetsm/prevue/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/prevue.svg)](https://crates.io/crates/prevue)

An HTML templating engine that uses [Vue](https://github.com/vuejs/core)'s [template syntax](https://vuejs.org/guide/essentials/template-syntax). Parses HTML, evaluates JavaScript expressions, and returns rendered output.


## Quick Start

```bash
cargo add prevue
```

```rust
pub fn render(template: impl AsRef<str>, data: impl serde::Serialize) -> prevue::Result<String>
```

`data` can be any value that implements `Serialize`.

```rust
use prevue::render;
use serde_json::json;

fn main() -> prevue::Result<()> {
    let template = r#"
        <div>
            <a :id="id">link</a>
            <p v-if="user.age >= 18">{{ user.name }} is adult</p>
            <ul>
                <li v-for="item in list">{{ item }}</li>
            </ul>
        </div>
    "#;

    let data = json!({
        "id": "link-id",
        "user": { "name": "James", "age": 28 },
        "list": ["a", "b", "c"],
    });

    let output = render(template, data)?;

    // <html><head></head><body><div>
    //         <a id="link-id">link</a>
    //         <p>James is adult</p>
    //         <ul>
    //             <li>a</li>
    //             <li>b</li>
    //             <li>c</li>
    //         </ul>
    //     </div>
    //     </body></html>

    Ok(())
}
```


## Rendering repeatedly

`render` builds a fresh JavaScript engine on every call. `Renderer` keeps one
alive across renders and caches compiled expressions:

```rust
use prevue::Renderer;
use serde_json::json;

let mut renderer = Renderer::new()?;
let template = "<p>{{ name }}</p>";

for name in ["Ada", "Grace"] {
    println!("{}", renderer.render(template, json!({ "name": name }))?);
}
```

Small templates render 5-25x faster this way; large loop-heavy ones barely
change, since the engine setup was already a rounding error there.

Render data and setup script declarations do not carry over between renders, but
globals a template creates deliberately do — `var` inside `{{ }}`, an undeclared
assignment, a write to `globalThis`, a mutated built-in. Use a fresh `Renderer`
when you need a clean realm.

`Renderer` is not `Send`; use one per thread.


## Precompiled templates

The HTML is still parsed on every call above, and parsing is most of the cost of
a small render. `Template` parses once:

```rust
use prevue::{Renderer, Template};
use serde_json::json;

let mut renderer = Renderer::new()?;
let template = Template::new("<p>{{ name }}</p>");

for name in ["Ada", "Grace"] {
    println!("{}", renderer.render_template(&template, json!({ "name": name }))?);
}
```

That is roughly twice as fast; loop-heavy templates gain little, since
evaluation dominates them instead. `Template` is not `Send` either, but cloning
one is cheap.


## Features

| Syntax | Notes |
|---|---|
| `{{ }}` | Text interpolation |
| `v-bind`, `:attr` | Attribute binding |
| `v-if` | Conditional rendering |
| `v-else`, `v-else-if` | Conditional branches |
| `v-for` | List rendering |
| `v-text` | Text replacement |
| `v-show` | Hides the element with `display: none` |
| `v-html` | Raw HTML replacement; inserted HTML is not compiled |
| `v-pre` | Skip rendering logic |
| `<template>` | Structural wrapper |
| `<script type="prevue">` | Render-order setup script |


## Behavior Notes

### HTML5 Parsing

This library uses [html5ever](https://github.com/servo/html5ever), which follows HTML5 spec strictly:

- Attribute names are lowercased (for example, `:MyAttr` becomes `:myattr`). On SVG and MathML, where case matters, `.camel` restores it: `:view-box.camel` binds `viewBox`.
- Dynamic binding names are lowercased, so `:[dynamicKey]` looks up `dynamickey`.
- A set boolean attribute is written as `disabled=""` rather than bare `disabled`, since the serializer always writes a value — HTML checks presence either way.
- Output is serialized as a complete HTML document with `<html>`, `<head>`, and `<body>`.

### Directives

- A truthy `v-show` adds nothing. Vue's server renderer writes `style=""` there; its own browser runtime does not.
- Directives prevue does not implement — `v-model`, `v-on` and `@`, `v-once`, `v-memo`, `v-cloak`, `v-slot` and `#` — are removed, since a directive is never an attribute. Vue's server renderer turns `v-model` into a `value` or `checked` attribute; prevue only removes it.
- An attribute spelled like a directive Vue does not define, such as a misspelled `v-els`, is an error.

### Attribute Binding

- When two sources set the same attribute, the one written last wins, and `class` and `style` merge in that order. Vue keeps the first instead unless a `v-bind` object is involved.
- A static `style` attribute is parsed and rewritten like a binding, so `style="marginTop: 1px"` becomes `margin-top: 1px;`. One that yields no declarations is left exactly as written, where Vue empties it, and `v-pre` skips this along with everything else.

### Data Scope

Object data fields are available as top-level variables. The original data value is also available as `$`.

```html
{{ user.name }}
{{ $.user.name }}
```

`$` is reserved for the full data value. If your data contains a top-level `"$"` field, access it with `$["$"]`.

### Setup Scripts

`<script type="prevue">` runs when rendering reaches it. Helpers defined by a setup script are available to following template expressions, and executed setup scripts are removed from the rendered HTML.

```html
<script type="prevue">
function fullName(user) {
    return `${user.first} ${user.last}`;
}
</script>

<p>{{ fullName(user) }}</p>
```

Only `type="prevue"` scripts are executed by prevue. Regular `<script>` tags are preserved.

### JavaScript Evaluation

prevue uses [Boa](https://github.com/boa-dev/boa) to evaluate JavaScript expressions and setup scripts.

- Never use untrusted templates.
- Accessing undeclared identifiers fails expression evaluation instead of returning `undefined`.
- `this` is not Vue-compatible and may expose internal scope objects. Avoid using `this` in templates.


## License

MIT
