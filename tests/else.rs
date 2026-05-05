use prevue::{Directive, DirectiveErrorKind, Error, render};
use serde_json::{Value, json};

fn data() -> Value {
    json!({
        "status": "success",
        "score": 85,
    })
}

// === Basic Behavior ===

#[test]
fn test_else_basic() {
    let input = r#"
    <div>
        <div v-if="true">IF1</div>
        <div v-else>ELSE1</div>

        <div v-if="false">IF2</div>
        <div v-else>ELSE2</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF1</div>

        <div>ELSE2</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_if_basic() {
    let input = r#"
    <div>
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
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>IF1</div>
        
        <div>IF2</div>

        <div>ELSE-IF3</div>
        
        <div>ELSE4</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Expressions ===

#[test]
fn test_else_if_expressions() {
    let input = r#"
    <div>
        <div v-if="score >= 90">A</div>
        <div v-else-if="score >= 80">B</div>
        <div v-else-if="score >= 70">C</div>
        <div v-else>F</div>

        <div v-if="status === 'pending'">Pending</div>
        <div v-else-if="status === 'success'">Success</div>
        <div v-else>Failed</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>B</div>

        <div>Success</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Chain Evaluations ===

#[test]
fn test_else_if_chain_evaluations() {
    // tests evaluating multiple v-else-if in a row
    let input = r#"
    <div>
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
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <!-- first else-if hits -->
        <div>ELSE-IF1</div>

        <!-- first else-if hits, second misses -->
        <div>ELSE-IF1</div>

        <!-- second else-if hits -->
        <div>ELSE-IF2</div>

        <!-- none hits, falls to else -->
        <div>ELSE</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Multiple Chains ===

#[test]
fn test_multiple_chains_adjacent() {
    // testing adjacent if-else chains to ensure their states do not leak
    let input = r#"
    <div>
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
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <!-- Chain 1 -->
        <div>IF1</div>

        <!-- Chain 2 -->
        <div>ELSE2</div>

        <!-- Chain 3 -->
        <div>IF3</div>

        <!-- Chain 4 -->
        <div>ELSE-IF4</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

// === Malformed Chains ===

#[test]
fn test_standalone_else_and_else_if() {
    let else_input = r#"
    <div>
        <div>Normal</div>
        <div v-else>ELSE</div>
    </div>
    "#;
    let err = render(else_input, data()).unwrap_err();
    assert!(matches!(
        err,
        Error::InvalidDirective {
            directive: Directive::Else,
            kind: DirectiveErrorKind::MissingAdjacentConditional,
            expression: None
        }
    ));

    let else_if_input = r#"
    <div>
        <div>Normal</div>
        <div v-else-if="false">ELSE-IF</div>
    </div>
    "#;
    let err = render(else_if_input, data()).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::ElseIf, kind: DirectiveErrorKind::MissingAdjacentConditional, expression: Some(expr) }
            if expr == "false")
    );
}

#[test]
fn test_else_if_after_else() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else>ELSE</div>
        <div v-else-if="true">ELSE-IF</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
    assert!(
        matches!(err, Error::InvalidDirective { directive: Directive::ElseIf, kind: DirectiveErrorKind::MissingAdjacentConditional, expression: Some(expr) }
            if expr == "true")
    );
}

#[test]
fn test_else_chain_allows_whitespace_and_comments() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <!-- comment keeps the chain adjacent -->
        <div v-else>ELSE</div>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <!-- comment keeps the chain adjacent -->
        <div>ELSE</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_chain_rejects_non_whitespace_text_between_branches() {
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        text
        <div v-else>ELSE</div>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
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
fn test_else_inside_pre_is_preserved() {
    let input = r#"
    <div v-pre>
        <p v-else>ELSE</p>
        <p v-else-if="ok">ELSE-IF</p>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p v-else="">ELSE</p>
        <p v-else-if="ok">ELSE-IF</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_inside_skipped_structural_branch_is_not_validated() {
    let input = r#"
    <div>
        <template v-if="false">
            <p v-else>ELSE</p>
        </template>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_inside_rendered_structural_branch_is_validated() {
    let input = r#"
    <div>
        <template v-if="true">
            <p v-else>ELSE</p>
        </template>
    </div>
    "#;
    let err = render(input, data()).unwrap_err();
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
fn test_else_chain_inside_rendered_structural_branch() {
    let input = r#"
    <div>
        <template v-if="true">
            <p v-if="false">IF</p>
            <p v-else>ELSE</p>
        </template>
    </div>
    "#;
    let output = render(input, data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <p>ELSE</p>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
