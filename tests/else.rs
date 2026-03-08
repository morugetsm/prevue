use prevue::render;
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
    let output = render(input.to_string(), data()).unwrap();

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
    let output = render(input.to_string(), data()).unwrap();

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
    let output = render(input.to_string(), data()).unwrap();

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
    let output = render(input.to_string(), data()).unwrap();

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
    let output = render(input.to_string(), data()).unwrap();

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
    // v-else and v-else-if without preceding v-if: should render normally without errors
    let input = r#"
    <div>
        <div>Normal</div>
        <div v-else>ELSE</div>

        <div>Normal</div>
        <div v-else-if="false">ELSE-IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>Normal</div>
        <div>ELSE</div>

        <div>Normal</div>
        <div>ELSE-IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}

#[test]
fn test_else_if_after_else() {
    // v-else breaks the chain, so a subsequent v-else-if is treated as a standalone (not hitting any chain state)
    let input = r#"
    <div>
        <div v-if="false">IF</div>
        <div v-else>ELSE</div>
        <div v-else-if="true">ELSE-IF</div>
    </div>
    "#;
    let output = render(input.to_string(), data()).unwrap();

    let expected = r#"<html><head></head><body><div>
        <div>ELSE</div>
        <div>ELSE-IF</div>
    </div>
    </body></html>"#;
    assert_eq!(output, expected);
}
