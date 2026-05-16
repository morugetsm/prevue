use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Directive {
    If,
    ElseIf,
    Else,
    For,
    Text,
    Html,
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
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectiveErrorKind {
    MissingExpression,
    UnexpectedExpression,
    MissingAdjacentConditional,
    InvalidExpression,
}

impl fmt::Display for DirectiveErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingExpression => f.write_str("missing expression"),
            Self::UnexpectedExpression => f.write_str("unexpected expression"),
            Self::MissingAdjacentConditional => f.write_str("missing adjacent conditional"),
            Self::InvalidExpression => f.write_str("invalid expression"),
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse template: {source}")]
    ParseTemplate {
        #[source]
        source: std::io::Error,
    },

    #[error("failed to serialize HTML: {source}")]
    SerializeHtml {
        #[source]
        source: std::io::Error,
    },

    #[error("failed to convert rendered HTML to UTF-8: {source}")]
    OutputUtf8 {
        #[source]
        source: std::string::FromUtf8Error,
    },

    #[error("failed to serialize render data: {source}")]
    DataSerialize {
        #[source]
        source: serde_json::Error,
    },

    #[error(
        "failed to convert render data{field} to JavaScript: {message}",
        field = format_field(field)
    )]
    DataToJs {
        field: Option<String>,
        message: String,
    },

    #[error(
        "failed to inject render data{field}: {message}",
        field = format_field(field)
    )]
    DataInject {
        field: Option<String>,
        message: String,
    },

    #[error("failed to manage JavaScript scope: {message}")]
    Scope { message: String },

    #[error("failed to execute <script type=\"prevue\">: {message}")]
    SetupScript { message: String },

    #[error(
        "invalid {directive}: {kind}{expression}",
        expression = format_expression(expression)
    )]
    InvalidDirective {
        directive: Directive,
        kind: DirectiveErrorKind,
        expression: Option<String>,
    },

    #[error(
        "conflicting directives: {directives}",
        directives = format_directives(directives)
    )]
    ConflictingDirectives { directives: Vec<Directive> },

    #[error("invalid attribute name {name:?}")]
    InvalidAttributeName { name: String },
}

pub type Result<T> = std::result::Result<T, Error>;
