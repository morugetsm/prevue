use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "list": [1, 2, 3],
        "user": {
            "name": "Alice",
            "age": 21
        },
    })
}

// === Array ===

#[test]
fn test_for_array() {
    let input = r#"
    <div>
        <h1>{{ notclosed }</h1>
        <h2>{{ item }}</h2>
        <h3 v-for="item in list">{{ item }}</h3>
        <h4>{{ item }}</h4>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>{{ notclosed }</h1>
        <h2></h2>
        <h3>1</h3>
        <h3>2</h3>
        <h3>3</h3>
        <h4></h4>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_array_of() {
    let input = r#"
    <div>
        <div v-for="item of list">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_array_literal() {
    let input = r#"
    <div>
        <div v-for="item, index in [10, 20, 30]">{{ `${index}: ${item}` }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>0: 10</div>
        <div>1: 20</div>
        <div>2: 30</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_array_excess_arguments() {
    // Array iteration supports at most `item, index`. Third argument resolves to empty/undefined.
    let input = r#"
    <div>
        <div v-for="item, index, third in list">
            <h1>{{ item }}</h1>
            <h2>{{ index }}</h2>
            <h3>{{ third }}</h3>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
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
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_nested() {
    let input = r#"
    <div>
        <div v-for="item in list">
            <h1>{{ item }}</h1>
            <h2 v-for="item in list">{{ item }}</h2>
            <h3>{{ item }}</h3>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
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
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Object ===

#[test]
fn test_for_object() {
    let input = r#"
    <div>
        <h1 v-for="value, key in user">{{ `${key}: ${value}` }}</h1>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>name: Alice</h1>
        <h1>age: 21</h1>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_object_three_arguments() {
    // Object iteration supports up to 3 arguments: `value, key, index`
    let input = r#"
    <div>
        <div v-for="value, key, index in user">
            <h1>{{ value }}</h1>
            <h2>{{ key }}</h2>
            <h3>{{ index }}</h3>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
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
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Number ===

#[test]
fn test_for_number_literal() {
    let input = r#"
    <div>
        <div v-for="item in 5">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
        <div>4</div>
        <div>5</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_number_variable() {
    let input = r#"
    <div>
        <div v-for="item in user.age">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
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
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_number_with_index() {
    let input = r#"
    <div>
        <div v-for="item, index in 3">{{ `${index}: ${item}` }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>0: 1</div>
        <div>1: 2</div>
        <div>2: 3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_number_zero() {
    let input = r#"
    <div>
        <div v-for="item in 0">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === String ===

#[test]
fn test_for_string_literal() {
    let input = r#"
    <div>
        <div v-for="char in 'abc'">{{ char }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>a</div>
        <div>b</div>
        <div>c</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_string_variable() {
    let input = r#"
    <div>
        <div v-for="char in user.name">{{ char }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>A</div>
        <div>l</div>
        <div>i</div>
        <div>c</div>
        <div>e</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_string_with_index() {
    let input = r#"
    <div>
        <div v-for="char, index in 'xyz'">{{ `${index}: ${char}` }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>0: x</div>
        <div>1: y</div>
        <div>2: z</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_string_empty() {
    let input = r#"
    <div>
        <div v-for="char in ''">{{ char }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Expressions & Special Types ===

#[test]
fn test_for_function_call() {
    let input = r#"
    <div>
        <div v-for="key in Object.keys(user)">{{ key }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>name</div>
        <div>age</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_method_chaining() {
    let input = r#"
    <div>
        <div v-for="item in list.filter(x => x > 1).map(x => x * 2)">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>4</div>
        <div>6</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_expression() {
    let input = r#"
    <div>
        <div v-for="n in Array(3).fill(0).map((_, i) => i + 1)">{{ n }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_special_char_variables() {
    // Valid JS identifier characters like $ and _ can be used
    let input = r#"
    <div>
        <div v-for="$, _ in list">{{ `${_}: ${$}` }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>0: 1</div>
        <div>1: 2</div>
        <div>2: 3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Edge Cases & Whitespace ===

#[test]
fn test_for_with_comment() {
    let input = r#"
    <div>
        <!-- comment --><div v-for="item in list">a{{ item }}</div>
        <div v-for="item in list">b{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <!-- comment --><div>a1</div><div>a2</div><div>a3</div>
        <div>b1</div>
        <div>b2</div>
        <div>b3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_with_leading_empty_line() {
    let input = r#"
    <div>
        
        <div v-for="item in list">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_with_trailing_empty_line() {
    let input = r#"
    <div>
        <div v-for="item in list">{{ item }}</div>
        
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
        
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_empty() {
    let input = r#"
    <div>
        <div v-for="item in []">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_with_leading_whitespace() {
    let input = r#"
    <div> hi
        <div v-for="item in list">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div> hi
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_with_leading_polluted() {
    let input = r#"
    <div> hi
    hi  <div v-for="item in list">{{ item }}</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div> hi
    hi  <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Syntax Errors ===

#[test]
fn test_for_syntax_error() {
    // Malformed v-for expression should safely do nothing (render 0 items)
    let input = r#"
    <div>
        <div v-for="Hello, world!">Hello, world!</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
