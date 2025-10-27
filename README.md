# prevue

An HTML templating engine using [Vue](https://github.com/vuejs/core)'s [template syntax](https://vuejs.org/guide/essentials/template-syntax). It parses HTML, evaluates inline JavaScript expressions, and returns rendered HTML.


## Installation

```toml
[dependencies]
prevue = "0.0.1"
```


## API

```rust
pub fn render(html: String, data: impl Serialize) -> Result<String, anyhow::Error>
```


## Example

```rust
use prevue::render;
use serde_json::json;

let html = r#"
<div>
    <p v-if="user.age >= 18">{{ user.name }} is adult</p>
    <ul>
        <li v-for="item in list">{{ item }}</li>
    </ul>
    <a :[attribute]="value">link</a>
</div>
"#.to_string();

let data = json!({
    "user": { "name": "James", "age": 28 },
    "list": ["a", "b", "c"],
    "attribute": "link-id",
    "value": 123,
});

let output = render(html, data).unwrap();

// <html><head></head><body><div>
//         <p>James is adult</p>
//         <ul>
//             <li>a</li>
//             <li>b</li>
//             <li>c</li>
//         </ul>
//         <a link-id="123">link</a>
//     </div>
//     </body></html>
```


## Features

| Syntax | Status | Notes |
|---|---|---|
| `{{ }}` | 🟡 | Array/Object stringify without spacing (e.g., `[1,2,3]` not `[ 1, 2, 3 ]`) |
| `<template>` | ✅ |  |
| `v-bind`, `:attr` | 🟡 | No class/style object binding |
| `v-if` | ✅ |  |
| `v-else`, `v-else-if` | ❌ |  |
| `v-for` | 🟡 | Array and Object only |
| `v-text`, `v-html` | ❌ | Planned |
| `v-pre` | ❌ |  |


## Important Notes

### HTML5 Parsing

This library uses [html5ever](https://github.com/servo/html5ever), which follows HTML5 spec strictly:
- Attribute names are **lowercased** (e.g., `:MyAttr` → `:myattr`)
- Dynamic bindings are **lowercased**: `:[dynamicKey]` looks up `dynamickey` variable
- Outputs complete HTML document with `<html>`, `<head>`, `<body>` tags

### JavaScript Evaluation

This library uses a [Boa](https://github.com/boa-dev/boa) JavaScript engine to evaluate expressions.

- ⚠️ **Security:** Never use untrusted templates or data
- **Evaluation Behavior:** Unlike Vue which restricts each binding to a single expression, prevue currently allows both expressions and statements in all binding contexts (e.g., `{{ let x = 1; x + 1 }}` → `2`). This may change in future versions to match Vue's behavior
- **Variable Access:** ⚠️ Accessing undefined variables will cause the entire expression evaluation to fail, rather than returning `undefined`. Always ensure that variables exist in the provided data
- **`this` Context:** While `this` is accessible in the JavaScript engine context, its behavior may vary due to internal optimizations, and access is restricted in the template engine context. Therefore, using `this` is not recommended


## License

MIT
