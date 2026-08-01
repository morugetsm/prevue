mod helper;

use helper::assert_render_body_eq;
use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::json;

// === Basic Behavior ===

#[test]
fn else_basic() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="true">IF1</div>
        <div v-else>ELSE1</div>

        <div v-if="false">IF2</div>
        <div v-else>ELSE2</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>IF1</div>

        <div>ELSE2</div>
    </div>"#,
    );
}

#[test]
fn else_if_basic() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="true">IF1</div>
        <div v-else-if="true">ELSE-IF1</div>
        <div v-else>ELSE1</div>
        
        <div v-if="true">IF2</div>
        <div v-else-if="false">ELSE-IF2</div>
        <div v-else>ELSE2</div>

        <div v-if="false">IF3</div>
        <div v-else-if="true">ELSE-IF3</div>
        <div v-else>ELSE3</div>
        
        <div v-if="false">IF4</div>
        <div v-else-if="false">ELSE-IF4</div>
        <div v-else>ELSE4</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>IF1</div>
        
        <div>IF2</div>

        <div>ELSE-IF3</div>
        
        <div>ELSE4</div>
    </div>"#,
    );
}

#[test]
fn else_if_empty_string_falsy() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="false">IF</div>
        <div v-else-if="''">ELSE-IF</div>
        <div v-else>ELSE</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>ELSE</div>
    </div>"#,
    );
}

#[test]
fn else_if_undeclared_identifier_falsy() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="false">IF1</div>
        <div v-else-if="notexist">MISSING1</div>
        <div v-else-if="true">NEXT</div>
        <div v-else>ELSE1</div>

        <div v-if="false">IF2</div>
        <div v-else-if="notexist">MISSING2</div>
        <div v-else>ELSE2</div>
    </div>"#,
        json!({}),
        r#"<div>
        <div>NEXT</div>

        <div>ELSE2</div>
    </div>"#,
    );
}

// === Expressions ===

#[test]
fn else_if_expressions() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="score >= 90">A</div>
        <div v-else-if="score >= 80">B</div>
        <div v-else-if="score >= 70">C</div>
        <div v-else>F</div>

        <div v-if="status === 'pending'">Pending</div>
        <div v-else-if="status === 'success'">Success</div>
        <div v-else>Failed</div>
    </div>"#,
        json!({
            "score": 85,
            "status": "success",
        }),
        r#"<div>
        <div>B</div>

        <div>Success</div>
    </div>"#,
    );
}

// === Chain Evaluations ===

#[test]
fn else_if_chain_evaluations() {
    assert_render_body_eq!(
        r#"<div>
        <!-- first else-if hits -->
        <div v-if="false">IF</div>
        <div v-else-if="true">ELSE-IF1</div>
        <div v-else-if="true">ELSE-IF2</div>
        <div v-else>ELSE</div>

        <!-- first else-if hits, second misses -->
        <div v-if="false">IF</div>
        <div v-else-if="true">ELSE-IF1</div>
        <div v-else-if="false">ELSE-IF2</div>
        <div v-else>ELSE</div>

        <!-- second else-if hits -->
        <div v-if="false">IF</div>
        <div v-else-if="false">ELSE-IF1</div>
        <div v-else-if="true">ELSE-IF2</div>
        <div v-else>ELSE</div>

        <!-- none hits, falls to else -->
        <div v-if="false">IF</div>
        <div v-else-if="false">ELSE-IF1</div>
        <div v-else-if="false">ELSE-IF2</div>
        <div v-else>ELSE</div>
    </div>"#,
        json!({}),
        r#"<div>
        <!-- first else-if hits -->
        <div>ELSE-IF1</div>

        <!-- first else-if hits, second misses -->
        <div>ELSE-IF1</div>

        <!-- second else-if hits -->
        <div>ELSE-IF2</div>

        <!-- none hits, falls to else -->
        <div>ELSE</div>
    </div>"#,
    );
}

// === Multiple Chains ===

#[test]
fn multiple_chains_adjacent() {
    assert_render_body_eq!(
        r#"<div>
        <!-- Chain 1 -->
        <div v-if="true">IF1</div>
        <div v-else>ELSE1</div>

        <!-- Chain 2 -->
        <div v-if="false">IF2</div>
        <div v-else>ELSE2</div>

        <!-- Chain 3 -->
        <div v-if="true">IF3</div>
        <div v-else-if="false">ELSE-IF3</div>

        <!-- Chain 4 -->
        <div v-if="false">IF4</div>
        <div v-else-if="true">ELSE-IF4</div>
        <div v-else>ELSE4</div>
    </div>"#,
        json!({}),
        r#"<div>
        <!-- Chain 1 -->
        <div>IF1</div>

        <!-- Chain 2 -->
        <div>ELSE2</div>

        <!-- Chain 3 -->
        <div>IF3</div>

        <!-- Chain 4 -->
        <div>ELSE-IF4</div>
    </div>"#,
    );
}

// === Malformed Chains ===

#[test]
fn standalone_else() {
    let input = r#"
    <div>
        <div>Normal</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Else,
            kind: DirectiveErrorKind::MissingAdjacentConditional,
            expression: None
        }
    ));
}

#[test]
fn standalone_else_if() {
    let input = r#"
    <div>
        <div>Normal</div>
        <div v-else-if="false">ELSE-IF</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::ElseIf, kind: DirectiveErrorKind::MissingAdjacentConditional, expression: Some(expr) }
            if expr == "false")
    );
}

#[test]
fn else_if_after_else() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else>ELSE</div>
        <div v-else-if="true">ELSE-IF</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::ElseIf, kind: DirectiveErrorKind::MissingAdjacentConditional, expression: Some(expr) }
            if expr == "true")
    );
}

#[test]
fn else_if_empty_expression() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else-if="">empty</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::ElseIf, kind: DirectiveErrorKind::MissingExpression, expression: Some(expr) }
            if expr.is_empty())
    );
}

#[test]
fn else_with_expression_error() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else="ok">ELSE</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::Else, kind: DirectiveErrorKind::UnexpectedExpression, expression: Some(expr) }
            if expr == "ok")
    );
}

#[test]
fn else_allows_whitespace_comments() {
    assert_render_body_eq!(
        r#"<div>
        <div v-if="false">IF</div>
        <!-- comment keeps the chain adjacent -->
        <div v-else>ELSE</div>
    </div>"#,
        json!({}),
        r#"<div>
        <!-- comment keeps the chain adjacent -->
        <div>ELSE</div>
    </div>"#,
    );
}

#[test]
fn else_rejects_text_between_branches() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        text
        <div v-else>ELSE</div>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Else,
            kind: DirectiveErrorKind::MissingAdjacentConditional,
            expression: None
        }
    ));
}

#[test]
fn else_inside_pre_preserved() {
    assert_render_body_eq!(
        r#"<div v-pre>
        <p v-else>ELSE</p>
        <p v-else-if="ok">ELSE-IF</p>
    </div>"#,
        json!({}),
        r#"<div>
        <p v-else="">ELSE</p>
        <p v-else-if="ok">ELSE-IF</p>
    </div>"#,
    );
}

#[test]
fn else_skipped_branch_ignored() {
    assert_render_body_eq!(
        r#"<div>
        <template v-if="false">
            <p v-else>ELSE</p>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
    </div>"#,
    );
}

#[test]
fn else_rendered_branch_validated() {
    let input = r#"
    <div>
        <template v-if="true">
            <p v-else>ELSE</p>
        </template>
    </div>
    "#;
    let err = render(input, json!({})).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Else,
            kind: DirectiveErrorKind::MissingAdjacentConditional,
            expression: None
        }
    ));
}

#[test]
fn else_chain_rendered_branch() {
    assert_render_body_eq!(
        r#"<div>
        <template v-if="true">
            <p v-if="false">IF</p>
            <p v-else>ELSE</p>
        </template>
    </div>"#,
        json!({}),
        r#"<div>
        <p>ELSE</p>
    </div>"#,
    );
}
