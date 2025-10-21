use prevue::render;
use serde_json::{Value, json};

fn payload() -> Value {
    json!({
        "list": [1, 2, 3],
        "number": 9999,
        "user": {
            "label": "User",
            "value": "morugetsm",
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
fn test_for() {
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
