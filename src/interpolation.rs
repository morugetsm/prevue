use crate::engine::Engine;

pub(crate) fn render_text(content: &str, engine: &mut Engine) -> Option<String> {
    let mut rendered = String::new();
    let mut cursor = 0;
    let mut changed = false;

    while let Some(open_offset) = content[cursor..].find("{{") {
        let open = cursor + open_offset;
        let expr_start = open + 2;
        let Some(close) = find_closing_delimiter(content, expr_start, |_| false) else {
            break;
        };

        rendered.push_str(&content[cursor..open]);
        let expr = content[expr_start..close].trim();
        rendered.push_str(&engine.eval_fmt(expr).unwrap_or_default());
        cursor = close + 2;
        changed = true;
    }

    if changed {
        rendered.push_str(&content[cursor..]);
        Some(rendered)
    } else {
        None
    }
}

#[derive(Clone, Copy)]
enum JsScanState {
    Code,
    String(char),
    Template,
    Regex { in_class: bool },
    LineComment,
    BlockComment,
}

pub(crate) fn find_closing_delimiter<F>(
    content: &str,
    start: usize,
    mut should_stop: F,
) -> Option<usize>
where
    F: FnMut(usize) -> bool,
{
    let mut state = JsScanState::Code;
    let mut escaped = false;
    let mut can_start_regex = true;
    let mut after_property_dot = false;
    let mut template_expr_brace_depths: Vec<usize> = Vec::new();
    let mut iter = content[start..].char_indices().peekable();

    while let Some((offset, ch)) = iter.next() {
        let pos = start + offset;

        match state {
            JsScanState::Code => match ch {
                ch if ch.is_whitespace() => {}
                '\'' | '"' => {
                    state = JsScanState::String(ch);
                    escaped = false;
                    can_start_regex = false;
                    after_property_dot = false;
                }
                '`' => {
                    state = JsScanState::Template;
                    escaped = false;
                    can_start_regex = false;
                    after_property_dot = false;
                }
                '/' if iter.peek().is_some_and(|(_, next)| *next == '/') => {
                    iter.next();
                    state = JsScanState::LineComment;
                }
                '/' if iter.peek().is_some_and(|(_, next)| *next == '*') => {
                    iter.next();
                    state = JsScanState::BlockComment;
                }
                '/' if can_start_regex => {
                    state = JsScanState::Regex { in_class: false };
                    escaped = false;
                    can_start_regex = false;
                    after_property_dot = false;
                }
                '<' if template_expr_brace_depths.is_empty() && should_stop(pos) => return None,
                '}' if !template_expr_brace_depths.is_empty() => {
                    let depth = template_expr_brace_depths
                        .last_mut()
                        .expect("non-empty template expression stack");
                    if *depth == 0 {
                        template_expr_brace_depths.pop();
                        state = JsScanState::Template;
                    } else {
                        *depth -= 1;
                    }
                    can_start_regex = false;
                    after_property_dot = false;
                }
                '}' if iter.peek().is_some_and(|(_, next)| *next == '}') => return Some(pos),
                ch if is_js_identifier_start(ch) => {
                    let mut end = pos + ch.len_utf8();
                    while let Some((next_offset, next_ch)) = iter.peek().copied() {
                        if !is_js_identifier_continue(next_ch) {
                            break;
                        }
                        iter.next();
                        end = start + next_offset + next_ch.len_utf8();
                    }

                    let word = &content[pos..end];
                    can_start_regex = !after_property_dot && keyword_allows_regex_after(word);
                    after_property_dot = false;
                }
                ch if ch.is_ascii_digit() => {
                    while let Some((_, next_ch)) = iter.peek().copied() {
                        if !(next_ch.is_ascii_alphanumeric() || matches!(next_ch, '.' | '_')) {
                            break;
                        }
                        iter.next();
                    }
                    can_start_regex = false;
                    after_property_dot = false;
                }
                '.' => {
                    can_start_regex = false;
                    after_property_dot = true;
                }
                '+' | '-' if iter.peek().is_some_and(|(_, next)| *next == ch) => {
                    iter.next();
                    after_property_dot = false;
                }
                '(' | '[' | '{' | ',' | ';' | ':' | '?' => {
                    if ch == '{'
                        && let Some(depth) = template_expr_brace_depths.last_mut()
                    {
                        *depth += 1;
                    }
                    can_start_regex = true;
                    after_property_dot = false;
                }
                ')' | ']' | '}' => {
                    can_start_regex = false;
                    after_property_dot = false;
                }
                '/' | '=' | '!' | '+' | '-' | '*' | '%' | '&' | '|' | '^' | '~' | '<' | '>' => {
                    can_start_regex = true;
                    after_property_dot = false;
                }
                _ => {
                    after_property_dot = false;
                }
            },
            JsScanState::String(quote) => {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == quote {
                    state = JsScanState::Code;
                }
            }
            JsScanState::Template => {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '`' {
                    state = JsScanState::Code;
                    can_start_regex = false;
                    after_property_dot = false;
                } else if ch == '$' && iter.peek().is_some_and(|(_, next)| *next == '{') {
                    iter.next();
                    template_expr_brace_depths.push(0);
                    state = JsScanState::Code;
                    can_start_regex = true;
                    after_property_dot = false;
                }
            }
            JsScanState::Regex { in_class } => {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if in_class {
                    if ch == ']' {
                        state = JsScanState::Regex { in_class: false };
                    }
                } else if ch == '[' {
                    state = JsScanState::Regex { in_class: true };
                } else if ch == '/' {
                    state = JsScanState::Code;
                    can_start_regex = false;
                }
            }
            JsScanState::LineComment => {
                if ch == '\n' {
                    state = JsScanState::Code;
                }
            }
            JsScanState::BlockComment => {
                if ch == '*' && iter.peek().is_some_and(|(_, next)| *next == '/') {
                    iter.next();
                    state = JsScanState::Code;
                }
            }
        }
    }

    None
}

fn is_js_identifier_start(ch: char) -> bool {
    ch == '$' || ch == '_' || ch.is_ascii_alphabetic()
}

fn is_js_identifier_continue(ch: char) -> bool {
    is_js_identifier_start(ch) || ch.is_ascii_digit()
}

fn keyword_allows_regex_after(word: &str) -> bool {
    matches!(
        word,
        "return"
            | "throw"
            | "typeof"
            | "void"
            | "delete"
            | "new"
            | "await"
            | "yield"
            | "case"
            | "in"
            | "of"
            | "instanceof"
    )
}
