use prevue::render;
use serde_json::{Value, json};

fn payload() -> Value {
    json!({
        "list": [1, 2, 3],
        "number": 9999,
        "user": {
            "label": "User",
            "value": "Morrison",
            "age": 28
        },
        "complex": [{
            "foo": "hi",
            "bar": "hello",
        }, {
            "foo": "bow",
            "bar": "wow",
        }],
    })
}

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
    let output = render(input.to_string(), payload()).unwrap();

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
fn test_for_array_literal() {
    let input = r#"
    <div>
        <div v-for="item, index in [10, 20, 30]">{{ `${index}: ${item}` }}</div>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>0: 10</div>
        <div>1: 20</div>
        <div>2: 30</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_array_with_third() {
    let input = r#"
    <div>
        <div v-for="item, index, third in list">
            <h1>{{ item }}</h1>
            <h2>{{ index }}</h2>
            <h3>{{ third }}</h3>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

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
    let output = render(input.to_string(), payload()).unwrap();

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

#[test]
fn test_for_object() {
    let input = r#"
    <div>
        <h1 v-for="value, key in user">{{ `${key}: ${value}` }}</h1>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>label: User</h1>
        <h1>value: Morrison</h1>
        <h1>age: 28</h1>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_object_with_index() {
    let input = r#"
    <div>
        <div v-for="value, key, index in user">
            <h1>{{ value }}</h1>
            <h2>{{ key }}</h2>
            <h3>{{ index }}</h3>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            <h1>User</h1>
            <h2>label</h2>
            <h3>0</h3>
        </div>
        <div>
            <h1>Morrison</h1>
            <h2>value</h2>
            <h3>1</h3>
        </div>
        <div>
            <h1>28</h1>
            <h2>age</h2>
            <h3>2</h3>
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_complex() {
    let input = r#"
    <div>
        <template v-for="item, index in complex">
            <h1>{{ index + ': ' + item }}</h1>
            <template v-for="value, key in item">
                <h2>{{ key + ': ' + value }}</h2>
            </template>
        </template>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <h1>0: [object Object]</h1>
        <h2>foo: hi</h2>
        <h2>bar: hello</h2>
        <h1>1: [object Object]</h1>
        <h2>foo: bow</h2>
        <h2>bar: wow</h2>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_for_function_call() {
    let input = r#"
    <div>
        <div v-for="key in Object.keys(user)">{{ key }}</div>
    </div>
    "#;
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>label</div>
        <div>value</div>
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
    let output = render(input.to_string(), payload()).unwrap();

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
    let output = render(input.to_string(), payload()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>1</div>
        <div>2</div>
        <div>3</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
