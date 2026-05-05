use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Directive {
    If,
    ElseIf,
    Else,
    For,
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::If => f.write_str("v-if"),
            Self::ElseIf => f.write_str("v-else-if"),
            Self::Else => f.write_str("v-else"),
            Self::For => f.write_str("v-for"),
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

    #[error("failed to convert render data to JavaScript: {message}")]
    DataToJs {
        field: Option<String>,
        message: String,
    },

    #[error("failed to inject render data: {message}")]
    DataInject {
        field: Option<String>,
        message: String,
    },

    #[error("failed to manage JavaScript scope: {message}")]
    Scope { message: String },

    #[error("failed to execute prevue script: {message}")]
    SetupScript { message: String },

    #[error("invalid {directive}: {kind}")]
    InvalidDirective {
        directive: Directive,
        kind: DirectiveErrorKind,
        expression: Option<String>,
    },

    #[error("conflicting conditional directives: {directives:?}")]
    ConflictingDirectives { directives: Vec<Directive> },
}

pub type Result<T> = std::result::Result<T, Error>;
