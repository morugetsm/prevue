mod helper;

use helper::assert_render_body_eq;
use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::json;

// === Array ===

#[test]
fn for_array() {
    assert_render_body_eq!(
        r#"<div>
        <h1>{{ notclosed }</h1>
        <h2>{{ item }}</h2>
        <h3 v-for="item in list">{{ item }}</h3>
        <h4>{{ item }}</h4>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <h1>{{ notclosed }</h1>
        <h2></h2>
        <h3>1</h3>
        <h3>2</h3>
        <h3>3</h3>
        <h4></h4>
    </div>"#,
    );
}

#[test]
fn for_array_of() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item of list">{{ item }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>"#,
    );
}

#[test]
fn for_array_literal() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item, index in [10, 20, 30]">{{ `${index}: ${item}` }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>0: 10</div>
        <div>1: 20</div>
        <div>2: 30</div>
    </div>"#,
    );
}

#[test]
fn for_sparse_array_uses_length() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item, index in let list = []; list[2] = 'C'; list">{{ `${index}:${item}` }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>0:undefined</div>
        <div>1:undefined</div>
        <div>2:C</div>
    </div>"#,
    );
}

#[test]
fn for_empty_slots_length() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item, index in Array(3)">{{ `${index}:${item}` }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>0:undefined</div>
        <div>1:undefined</div>
        <div>2:undefined</div>
    </div>"#,
    );
}

#[test]
fn for_array_excess_arguments() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item, index, third in list">
            <h1>{{ item }}</h1>
            <h2>{{ index }}</h2>
            <h3>{{ third }}</h3>
        </div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>
            <h1>1</h1>
            <h2>0</h2>
            <h3></h3>
        </div>
        <div>
            <h1>2</h1>
            <h2>1</h2>
            <h3></h3>
        </div>
        <div>
            <h1>3</h1>
            <h2>2</h2>
            <h3></h3>
        </div>
    </div>"#,
    );
}

#[test]
fn for_object_destructuring_alias() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="{ foo, bar } in items">{{ `${foo}:${bar}` }}</p>
    </div>"#,
        json!({
            "items": [
                { "foo": "a", "bar": 1 },
                { "foo": "b", "bar": 2 },
            ],
        }),
        r#"<div>
        <p>a:1</p>
        <p>b:2</p>
    </div>"#,
    );
}

#[test]
fn for_nested_default_rest() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="{ foo: label, nested: { count = 0 }, ...rest } in items">
            {{ `${label}:${count}:${rest.extra}` }}
        </p>
    </div>"#,
        json!({
            "items": [
                { "foo": "a", "nested": { "count": 7 }, "extra": "x" },
                { "foo": "b", "nested": {}, "extra": "y" },
            ],
        }),
        r#"<div>
        <p>
            a:7:x
        </p>
        <p>
            b:0:y
        </p>
    </div>"#,
    );
}

#[test]
fn for_array_destructuring_alias() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="[first, second, ...rest] in rows">{{ `${first}:${second}:${rest.length}` }}</p>
    </div>"#,
        json!({
            "rows": [
                [1, 2, 3, 4],
                [5, 6],
            ],
        }),
        r#"<div>
        <p>1:2:2</p>
        <p>5:6:0</p>
    </div>"#,
    );
}

#[test]
fn for_destructuring_with_index() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="{ name }, index in users">{{ `${index}:${name}` }}</p>
    </div>"#,
        json!({
            "users": [
                { "name": "Alice" },
                { "name": "Bob" },
            ],
        }),
        r#"<div>
        <p>0:Alice</p>
        <p>1:Bob</p>
    </div>"#,
    );
}

#[test]
fn for_nested() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in list">
            <h1>{{ item }}</h1>
            <h2 v-for="item in list">{{ item }}</h2>
            <h3>{{ item }}</h3>
        </div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>
            <h1>1</h1>
            <h2>1</h2>
            <h2>2</h2>
            <h2>3</h2>
            <h3>1</h3>
        </div>
        <div>
            <h1>2</h1>
            <h2>1</h2>
            <h2>2</h2>
            <h2>3</h2>
            <h3>2</h3>
        </div>
        <div>
            <h1>3</h1>
            <h2>1</h2>
            <h2>2</h2>
            <h2>3</h2>
            <h3>3</h3>
        </div>
    </div>"#,
    );
}

// === Object ===

#[test]
fn for_object() {
    assert_render_body_eq!(
        r#"<div>
        <h1 v-for="value, key in user">{{ `${key}: ${value}` }}</h1>
    </div>"#,
        json!({
            "user": {
                "name": "Alice",
                "age": 21,
            },
        }),
        r#"<div>
        <h1>name: Alice</h1>
        <h1>age: 21</h1>
    </div>"#,
    );
}

#[test]
fn for_object_three_arguments() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="value, key, index in user">
            <h1>{{ value }}</h1>
            <h2>{{ key }}</h2>
            <h3>{{ index }}</h3>
        </div>
    </div>"#,
        json!({
            "user": {
                "name": "Alice",
                "age": 21,
            },
        }),
        r#"<div>
        <div>
            <h1>Alice</h1>
            <h2>name</h2>
            <h3>0</h3>
        </div>
        <div>
            <h1>21</h1>
            <h2>age</h2>
            <h3>1</h3>
        </div>
    </div>"#,
    );
}

#[test]
fn for_object_destructuring_key_index() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="{ age }, key, index in users">{{ `${key}:${index}:${age}` }}</p>
    </div>"#,
        json!({
            "users": {
                "alice": { "age": 21 },
                "bob": { "age": 22 },
            },
        }),
        r#"<div>
        <p>alice:0:21</p>
        <p>bob:1:22</p>
    </div>"#,
    );
}

#[test]
fn for_enumerable_keys() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="value, key in let obj = { visible: 'yes' }; Object.defineProperty(obj, 'hidden', { value: 'no', enumerable: false }); obj">{{ `${key}:${value}` }}</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>visible:yes</p>
    </div>"#,
    );
}

#[test]
fn for_object_skips_symbol_keys() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="value in let sym = Symbol('secret'); let obj = { visible: 'yes' }; obj[sym] = 'hidden'; obj">{{ value }}</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>yes</p>
    </div>"#,
    );
}

// === Number ===

#[test]
fn for_number_literal() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in 5">{{ item }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
        <div>4</div>
        <div>5</div>
    </div>"#,
    );
}

#[test]
fn for_number_variable() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in user.age">{{ item }}</div>
    </div>"#,
        json!({
            "user": {
                "age": 21,
            },
        }),
        r#"<div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
        <div>4</div>
        <div>5</div>
        <div>6</div>
        <div>7</div>
        <div>8</div>
        <div>9</div>
        <div>10</div>
        <div>11</div>
        <div>12</div>
        <div>13</div>
        <div>14</div>
        <div>15</div>
        <div>16</div>
        <div>17</div>
        <div>18</div>
        <div>19</div>
        <div>20</div>
        <div>21</div>
    </div>"#,
    );
}

#[test]
fn for_number_with_index() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item, index in 3">{{ `${index}: ${item}` }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>0: 1</div>
        <div>1: 2</div>
        <div>2: 3</div>
    </div>"#,
    );
}

#[test]
fn for_number_zero() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in 0">{{ item }}</div>
    </div>"#,
        json!({}),
        r#"<div>
    </div>"#,
    );
}

// === String ===

#[test]
fn for_string_literal() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="char in 'abc'">{{ char }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>a</div>
        <div>b</div>
        <div>c</div>
    </div>"#,
    );
}

#[test]
fn for_string_variable() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="char in user.name">{{ char }}</div>
    </div>"#,
        json!({
            "user": {
                "name": "Alice",
            },
        }),
        r#"<div>
        <div>A</div>
        <div>l</div>
        <div>i</div>
        <div>c</div>
        <div>e</div>
    </div>"#,
    );
}

#[test]
fn for_string_with_index() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="char, index in 'xyz'">{{ `${index}: ${char}` }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>0: x</div>
        <div>1: y</div>
        <div>2: z</div>
    </div>"#,
    );
}

#[test]
fn for_string_empty() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="char in ''">{{ char }}</div>
    </div>"#,
        json!({}),
        r#"<div>
    </div>"#,
    );
}

// === Expressions & Special Types ===

#[test]
fn for_function_call() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="key in Object.keys(user)">{{ key }}</div>
    </div>"#,
        json!({
            "user": {
                "name": "Alice",
                "age": 21,
            },
        }),
        r#"<div>
        <div>name</div>
        <div>age</div>
    </div>"#,
    );
}

#[test]
fn for_method_chaining() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in list.filter(x => x > 1).map(x => x * 2)">{{ item }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>4</div>
        <div>6</div>
    </div>"#,
    );
}

#[test]
fn for_expression() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="n in Array(3).fill(0).map((_, i) => i + 1)">{{ n }}</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>"#,
    );
}

#[test]
fn for_special_char_variables() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="$, _ in list">{{ `${_}: ${$}` }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>0: 1</div>
        <div>1: 2</div>
        <div>2: 3</div>
    </div>"#,
    );
}

#[test]
fn for_set_iterable() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="item in new Set(['a', 'b'])">{{ item }}</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>a</p>
        <p>b</p>
    </div>"#,
    );
}

#[test]
fn for_map_iterable() {
    assert_render_body_eq!(
        r#"<div>
        <p v-for="[key, value] in new Map([['x', 1], ['y', 2]])">{{ `${key}:${value}` }}</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p>x:1</p>
        <p>y:2</p>
    </div>"#,
    );
}

// === Edge Cases & Whitespace ===

#[test]
fn for_with_comment() {
    assert_render_body_eq!(
        r#"<div>
        <!-- comment --><div v-for="item in list">a{{ item }}</div>
        <div v-for="item in list">b{{ item }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <!-- comment --><div>a1</div><div>a2</div><div>a3</div>
        <div>b1</div>
        <div>b2</div>
        <div>b3</div>
    </div>"#,
    );
}

#[test]
fn for_with_leading_empty_line() {
    assert_render_body_eq!(
        r#"<div>
        
        <div v-for="item in list">{{ item }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>"#,
    );
}

#[test]
fn for_with_trailing_empty_line() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in list">{{ item }}</div>
        
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
        
    </div>"#,
    );
}

#[test]
fn for_empty() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in []">{{ item }}</div>
    </div>"#,
        json!({}),
        r#"<div>
    </div>"#,
    );
}

#[test]
fn for_with_leading_whitespace() {
    assert_render_body_eq!(
        r#"<div> hi
        <div v-for="item in list">{{ item }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div> hi
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>"#,
    );
}

#[test]
fn for_with_leading_polluted() {
    assert_render_body_eq!(
        r#"<div> hi
    hi  <div v-for="item in list">{{ item }}</div>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div> hi
    hi  <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>"#,
    );
}

#[test]
fn for_sibling_loops_do_not_share_scope() {
    // Sibling loops reuse the same scope depth; the first loop's binding must
    // not still be visible inside the second.
    assert_render_body_eq!(
        r#"<div><p v-for="a in [1]">{{ a }}</p><p v-for="b in [2]">{{ typeof a }}</p></div>"#,
        json!({}),
        r#"<div><p>1</p><p>undefined</p></div>"#,
    );
}

#[test]
fn for_nested_loop_does_not_leak_to_sibling() {
    assert_render_body_eq!(
        r#"<div><p v-for="outer in [1]"><b v-for="inner in [9]">{{ inner }}</b></p><p v-for="other in [2]">{{ typeof inner }}</p></div>"#,
        json!({}),
        r#"<div><p><b>9</b></p><p>undefined</p></div>"#,
    );
}

#[test]
fn for_missing_iterable_is_empty() {
    assert_render_body_eq!(
        r#"<div>
        <div v-for="item in missing">{{ item }}</div>
    </div>"#,
        json!({}),
        r#"<div>
    </div>"#,
    );
}

// === Syntax Errors ===

#[test]
fn for_syntax_error() {
    let input = r#"
    <div>
        <div v-for="Hello, world!">Hello, world!</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::For, kind: DirectiveErrorKind::InvalidExpression, expression: Some(expr) }
            if expr == "Hello, world!")
    );
}

#[test]
fn for_destructuring_syntax_error() {
    let input = r#"
    <div>
        <div v-for="{ foo: } in list">Hello, world!</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::For, kind: DirectiveErrorKind::InvalidExpression, expression: Some(expr) }
            if expr == "{ foo: } in list")
    );
}
