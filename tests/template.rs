mod helper;

use helper::assert_render_body_eq;
use serde_json::json;

// === Basic Behavior ===

#[test]
fn template_basic() {
    assert_render_body_eq!(
        r#"<div>
        <template>Hello</template>
    </div>"#,
        json!({}),
        r#"<div>
        <template></template>
    </div>"#,
    );
}

// === v-if ===

#[test]
fn template_if() {
    assert_render_body_eq!(
        r#"<div>
        <template v-if="true">Hello</template>
    </div>"#,
        json!({}),
        r#"<div>
        Hello
    </div>"#,
    );
}

#[test]
fn template_if_chain() {
    assert_render_body_eq!(
        r#"<div>
        <template v-if="false">A</template>
        <template v-else-if="true">B</template>
        <template v-else>C</template>
    </div>"#,
        json!({}),
        r#"<div>
        B
    </div>"#,
    );
}

// === v-for ===

#[test]
fn template_for() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="item in list">{{ item }}</template>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        1
        2
        3
    </div>"#,
    );
}

#[test]
fn template_for_destructuring() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="{ foo } in list">{{ foo }}</template>
    </div>"#,
        json!({
            "list": [
                { "foo": "a" },
                { "foo": "b" },
            ],
        }),
        r#"<div>
        a
        b
    </div>"#,
    );
}

#[test]
fn template_for_empty() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="item in list"></template>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
    </div>"#,
    );
}

#[test]
fn template_for_element() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="item in list">
            <div>{{ item }}</div>
        </template>
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
fn template_for_element_linebreak() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="item in list">
            <div>
                {{ item }}
            </div>
        </template>
    </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
        <div>
            1
        </div>
        <div>
            2
        </div>
        <div>
            3
        </div>
    </div>"#,
    );
}

#[test]
fn template_for_element_linebreak_with_less_indent() {
    assert_render_body_eq!(
        r#"<div>
    <template v-for="item in list">
      <div>
        {{ item }}
      </div>
    </template>
  </div>"#,
        json!({ "list": [1, 2, 3] }),
        r#"<div>
    <div>
      1
    </div>
    <div>
      2
    </div>
    <div>
      3
    </div>
  </div>"#,
    );
}

#[test]
fn template_expansion_preserves_pre_text_indentation() {
    assert_render_body_eq!(
        r#"<div>
        <template v-if="true">
            <pre>
                keep
            </pre>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <pre>                keep
        </pre>
    </div>"#,
    );
}

#[test]
fn template_expansion_preserves_v_pre_subtree_indentation() {
    assert_render_body_eq!(
        r#"<div>
        <template v-if="true">
            <div v-pre>
                {{ message }}
            </div>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <div>
            {{ message }}
        </div>
    </div>"#,
    );
}

#[test]
fn template_for_complex() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="item, index in complex">
            <h1>{{ index + ': ' + item }}</h1>
            <template v-for="value, key in item">
                <h2>{{ key + ': ' + value }}</h2>
            </template>
        </template>
    </div>"#,
        json!({
            "complex": [{
                "foo": "hi",
                "bar": "hello",
            }, {
                "foo": "bow",
                "bar": "wow",
            }],
        }),
        r#"<div>
        <h1>0: [object Object]</h1>
        <h2>foo: hi</h2>
        <h2>bar: hello</h2>
        <h1>1: [object Object]</h1>
        <h2>foo: bow</h2>
        <h2>bar: wow</h2>
    </div>"#,
    );
}

#[test]
fn template_for_object_with_key_index() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="val, key, idx in { a: 1, b: 2 }">
            <p>{{ `[${idx}] ${key}: ${val}` }}</p>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <p>[0] a: 1</p>
        <p>[1] b: 2</p>
    </div>"#,
    );
}

#[test]
fn template_for_with_inner_if() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="n in [1, 2, 3]">
            <template v-if="n % 2 === 1">
                <span>{{ n }}</span>
            </template>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <span>1</span>
        <span>3</span>
    </div>"#,
    );
}

#[test]
fn template_for_trims_whitespace_children() {
    assert_render_body_eq!(
        r#"<div>
        <template v-for="i in [1,2]">
            
            
            <em>{{ i }}</em>
            
            
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <em>1</em>
        <em>2</em>
    </div>"#,
    );
}

// === v-pre ===

#[test]
fn template_pre() {
    assert_render_body_eq!(
        r#"<div>
        <template v-pre>
            <div>DIV</div>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <template></template>
    </div>"#,
    );
}

#[test]
fn template_pre_inner() {
    assert_render_body_eq!(
        r#"<div>
        <div v-pre>
            <template>TEMPLATE</template>
        </div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>
            <template></template>
        </div>
    </div>"#,
    );
}

#[test]
fn template_pre_with_if() {
    assert_render_body_eq!(
        r#"<div>
        <template v-pre v-if="false">Hello</template>
    </div>"#,
        json!({}),
        r#"<div>
        <template v-if="false"></template>
    </div>"#,
    );
}

// === Attributes ===

#[test]
fn template_no_directive_with_attrs() {
    assert_render_body_eq!(
        r#"<div>
        <template data-x="y">IGNORED</template>
    </div>"#,
        json!({}),
        r#"<div>
        <template data-x="y"></template>
    </div>"#,
    );
}
