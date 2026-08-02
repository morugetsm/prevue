use std::fmt;

/// A Vue-style template directive recognized by `prevue`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Directive {
    /// `v-if`
    If,
    /// `v-else-if`
    ElseIf,
    /// `v-else`
    Else,
    /// `v-for`
    For,
    /// `v-text`
    Text,
    /// `v-html`
    Html,
    /// `v-bind`, including the `:` shorthand
    Bind,
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::If => f.write_str("v-if"),
            Self::ElseIf => f.write_str("v-else-if"),
            Self::Else => f.write_str("v-else"),
            Self::For => f.write_str("v-for"),
            Self::Text => f.write_str("v-text"),
            Self::Html => f.write_str("v-html"),
            Self::Bind => f.write_str("v-bind"),
        }
    }
}

/// The reason a directive failed validation before rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectiveErrorKind {
    /// The directive requires an expression, but the attribute value was empty.
    MissingExpression,
    /// The directive does not accept an expression, but one was provided.
    UnexpectedExpression,
    /// `v-else-if` or `v-else` was not adjacent to a preceding conditional branch.
    MissingAdjacentConditional,
    /// The directive expression could not be parsed as valid directive syntax.
    InvalidExpression,
    /// The directive was given a modifier it does not define.
    UnknownModifier,
}

impl fmt::Display for DirectiveErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExpression => f.write_str("missing expression"),
            Self::UnexpectedExpression => f.write_str("unexpected expression"),
            Self::MissingAdjacentConditional => f.write_str("missing adjacent conditional"),
            Self::InvalidExpression => f.write_str("invalid expression"),
            Self::UnknownModifier => f.write_str("unknown modifier"),
        }
    }
}

fn format_field(field: &Option<String>) -> String {
    match field {
        Some(field) => format!(" field {field:?}"),
        None => String::new(),
    }
}

fn format_expression(expression: &Option<String>) -> String {
    match expression {
        Some(expression) if !expression.is_empty() => format!(" {expression:?}"),
        _ => String::new(),
    }
}

fn format_directives(directives: &[Directive]) -> String {
    directives
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Errors returned while rendering a template.
///
/// Each variant describes the operation that failed, such as preparing caller
/// data, applying directives, installing render data, running setup code, or
/// writing the final HTML.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Render data could not be serialized to JSON.
    #[error("failed to serialize render data: {source}")]
    DataSerialize {
        /// The serialization error reported by `serde_json`.
        #[source]
        source: serde_json::Error,
    },

    /// The rendered document could not be written as final HTML output.
    #[error("failed to render HTML output: {message}")]
    RenderOutput {
        /// The output serialization or UTF-8 conversion failure.
        message: String,
    },

    /// A directive was malformed or appeared in an invalid position during traversal.
    #[error(
        "invalid {directive}: {kind}{expression}",
        expression = format_expression(expression)
    )]
    InvalidDirective {
        /// The directive that failed validation.
        directive: Directive,
        /// The validation failure kind.
        kind: DirectiveErrorKind,
        /// The directive expression, when one was present.
        expression: Option<String>,
    },

    /// Multiple directives were used together where only one can apply.
    #[error(
        "conflicting directives: {directives}",
        directives = format_directives(directives)
    )]
    ConflictingDirectives {
        /// The conflicting directives found on the same element.
        directives: Vec<Directive>,
    },

    /// A dynamic `v-bind` result produced an invalid HTML attribute name.
    #[error("invalid attribute name {name:?}")]
    InvalidAttributeName {
        /// The invalid attribute name.
        name: String,
    },

    /// An attribute spelled like a directive that Vue does not define.
    #[error("unknown directive {name:?}")]
    UnknownDirective {
        /// The attribute name as written.
        name: String,
    },

    /// Render data could not be installed into the JavaScript scope.
    ///
    /// `field` is `None` when initializing the root data alias `$`, and
    /// `Some(name)` when initializing a top-level data field.
    #[error(
        "failed to initialize render data{field}: {message}",
        field = format_field(field)
    )]
    DataInit {
        /// The data field being initialized, or `None` for the root alias `$`.
        field: Option<String>,
        /// The JavaScript engine error message.
        message: String,
    },

    /// A `<script type="prevue">` block failed to parse or execute.
    #[error("failed to execute <script type=\"prevue\">: {message}")]
    SetupScript {
        /// The JavaScript engine error message.
        message: String,
    },

    /// An unexpected failure occurred while managing render state.
    #[error("internal error: {message}")]
    Internal {
        /// The internal failure description.
        message: String,
    },
}

/// A `Result` alias using [`Error`] as the error type.
pub type Result<T> = std::result::Result<T, Error>;
