/// What is left of a directive attribute once every render branch has passed
/// on it.
pub(crate) enum Unhandled {
    /// A Vue directive with no server-side rendering. Vue compiles it away, so
    /// it is not an attribute either.
    Unrendered,
    /// Not a Vue directive at all.
    Unknown,
}

pub(crate) fn classify(local: &str) -> Option<Unhandled> {
    let name = directive_name(local)?;

    Some(match is_builtin(name) {
        true => Unhandled::Unrendered,
        false => Unhandled::Unknown,
    })
}

/// Vue's parser reads `v-name:arg.modifier`, and each shorthand stands in for
/// the directive it names.
fn directive_name(local: &str) -> Option<&str> {
    let rest = match local.as_bytes().first()? {
        b'@' => return Some("on"),
        b'#' => return Some("slot"),
        b':' | b'.' => return Some("bind"),
        _ => local.strip_prefix("v-")?,
    };

    rest.split([':', '.']).next()
}

/// Vue's `isBuiltInDirective`.
fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        "bind"
            | "cloak"
            | "else-if"
            | "else"
            | "for"
            | "html"
            | "if"
            | "model"
            | "on"
            | "once"
            | "pre"
            | "show"
            | "slot"
            | "text"
            | "memo"
    )
}
