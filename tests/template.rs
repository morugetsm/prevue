use prevue::render;
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "list": [1, 2, 3],
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
fn test_template() {
    let input = r#"
    <div>
        <template>Hello</template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <template></template>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_if() {
    let input = r#"
    <div>
        <template v-if="true">Hello</template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        Hello
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for() {
    let input = r#"
    <div>
        <template v-for="item in list">{{ item }}</template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        1
        2
        3
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_empty() {
    let input = r#"
    <div>
        <template v-for="item in list"></template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_element() {
    let input = r#"
    <div>
        <template v-for="item in list">
            <div>{{ item }}</div>
        </template>
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
fn test_template_for_element_linebreak() {
    let input = r#"
    <div>
        <template v-for="item in list">
            <div>
                {{ item }}
            </div>
        </template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            1
        </div>
        <div>
            2
        </div>
        <div>
            3
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_element_linebreak_with_less_indent() {
    let input = r#"
  <div>
    <template v-for="item in list">
      <div>
        {{ item }}
      </div>
    </template>
  </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    <div>
      1
    </div>
    <div>
      2
    </div>
    <div>
      3
    </div>
  </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_complex() {
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
    let output = render(input.to_string(), data()).unwrap();

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
fn test_template_pre() {
    let input = r#"
    <div>
        <template v-pre>
            <div>DIV</div>
        </template>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <template></template>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_pre_inner() {
    let input = r#"
    <div>
        <div v-pre>
            <template>TEMPLATE</template>
        </div>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>
            <template></template>
        </div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_pre_with_if() {
    let input = r#"
    <div>
        <template v-pre v-if="false">Hello</template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <template v-if="false"></template>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_if_chain() {
    let input = r#"
    <div>
        <template v-if="false">A</template>
        <template v-else-if="true">B</template>
        <template v-else>C</template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        B
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_object_with_key_index() {
    let input = r#"
    <div>
        <template v-for="val, key, idx in { a: 1, b: 2 }">
            <p>{{ `[${idx}] ${key}: ${val}` }}</p>
        </template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>[0] a: 1</p>
        <p>[1] b: 2</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_with_inner_if() {
    let input = r#"
    <div>
        <template v-for="n in [1, 2, 3]">
            <template v-if="n % 2 === 1">
                <span>{{ n }}</span>
            </template>
        </template>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <span>1</span>
        <span>3</span>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_no_directive_with_attrs() {
    let input = r#"
    <div>
        <template data-x="y">IGNORED</template>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <template data-x="y"></template>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_template_for_trims_whitespace_children() {
    let input = r#"
    <div>
        <template v-for="i in [1,2]">
            
            
            <em>{{ i }}</em>
            
            
        </template>
    </div>
    "#;
    let output = render(input.to_string(), Value::Null).unwrap();

    let expected = r#"<html><head></head><body><div>
        <em>1</em>
        <em>2</em>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
