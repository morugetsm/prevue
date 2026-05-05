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


## Features

| Syntax | Status | Notes |
|---|---|---|
| `{{ }}` | ✅ | Text interpolation |
| `<template>` | ✅ | Structural wrapper |
| `<script type="prevue">` | ✅ | Render-order setup script |
| `v-bind`, `:attr` | ✅ | Attribute binding |
| `v-if` | ✅ | Conditional rendering |
| `v-else`, `v-else-if` | ✅ | Conditional branches |
| `v-for` | ✅ | List rendering |
| `v-text` | ✅ | Text replacement |
| `v-html` | ✅ | Raw HTML replacement; inserted HTML is not compiled |
| `v-pre` | ✅ | Skip rendering logic |


## Behavior Notes

### HTML5 Parsing

This library uses [html5ever](https://github.com/servo/html5ever), which follows HTML5 spec strictly:

- Attribute names are lowercased (for example, `:MyAttr` becomes `:myattr`).
- Dynamic binding names are lowercased, so `:[dynamicKey]` looks up `dynamickey`.
- Output is serialized as a complete HTML document with `<html>`, `<head>`, and `<body>`.

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
- `render` returns `prevue::Result<String>`, so parser, data, setup script, and structural directive errors can be handled explicitly.
- `this` is not Vue-compatible and may expose internal scope objects. Avoid using `this` in templates.


## License

MIT
