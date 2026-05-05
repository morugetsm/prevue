# prevue

[![CI](https://github.com/morugetsm/prevue/actions/workflows/ci.yml/badge.svg)](https://github.com/morugetsm/prevue/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/prevue.svg)](https://crates.io/crates/prevue)

An HTML templating engine that uses [Vue](https://github.com/vuejs/core)'s [template syntax](https://vuejs.org/guide/essentials/template-syntax). Parses HTML, evaluates inline JavaScript expressions, and returns rendered output.


## Installation

```bash
cargo add prevue
```


## API

```rust
pub fn render(template: impl AsRef<str>, data: impl Serialize) -> Result<String, anyhow::Error>
```


## Example

```rust
use prevue::render;
use serde_json::json;

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

let output = render(template, data).unwrap();
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
```


## Features

| Syntax | Status | Notes |
|---|---|---|
| `{{ }}` | ✅ |  |
| `<template>` | ✅ |  |
| `<script type="prevue">` | ✅ | Server-side setup script |
| `v-bind`, `:attr` | ✅ |  |
| `v-if` | ✅ |  |
| `v-else` | ✅ |  |
| `v-else-if` | ✅ |  |
| `v-for` | ✅ |  |
| `v-text` | ✅ |  |
| `v-html` | ❌ |  |
| `v-pre` | ✅ |  |


## Important Notes

### HTML5 Parsing

This library uses [html5ever](https://github.com/servo/html5ever), which follows HTML5 spec strictly:
- Attribute names are **lowercased** (e.g., `:MyAttr` → `:myattr`)
- Dynamic bindings are **lowercased**: `:[dynamicKey]` looks up `dynamickey` variable
- Outputs complete HTML document with `<html>`, `<head>`, `<body>` tags

### JavaScript Evaluation

This library uses a [Boa](https://github.com/boa-dev/boa) JavaScript engine to evaluate expressions.

- ⚠️ **Security:** Never use untrusted templates or data.
- **Evaluation Behavior:** Unlike Vue, which restricts each binding to a single expression, prevue currently allows both expressions and statements in all binding contexts (e.g., `:x="let n = 1; n + 1"` and `{{ let n = 1; n + 1 }}` → `2`). This may change in future versions to match Vue's behavior.
- **Data Alias:** Top-level object fields are available directly, and the original data is also available as `$` (e.g., `{{ user.name }}` and `{{ $.user.name }}`). `$` is a reserved alias for the full data value; if your data contains a top-level `"$"` field, access it with `$["$"]`.
- **Setup Script:** `<script type="prevue">` runs when rendering reaches it and can define helpers for following template expressions. Only `type="prevue"` scripts are executed; regular `<script>` tags are preserved. Executed setup scripts are removed from the rendered HTML. Setup scripts share the same scope as template expressions, including top-level data fields and `$`.
- **Variable Access:** Accessing undeclared identifiers will cause the entire expression evaluation to fail, rather than returning `undefined`. Always ensure that variables exist in the provided data.
- **`this` Context:** `this` is not Vue-compatible and may expose internal scope objects. Using `this` in templates is not recommended.


## License

MIT
