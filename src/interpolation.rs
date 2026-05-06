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
    let mut iter = content[start..].char_indices().peekable();

    while let Some((offset, ch)) = iter.next() {
        let pos = start + offset;

        match state {
            JsScanState::Code => match ch {
                '\'' | '"' | '`' => {
                    state = JsScanState::String(ch);
                    escaped = false;
                }
                '/' if iter.peek().is_some_and(|(_, next)| *next == '/') => {
                    iter.next();
                    state = JsScanState::LineComment;
                }
                '/' if iter.peek().is_some_and(|(_, next)| *next == '*') => {
                    iter.next();
                    state = JsScanState::BlockComment;
                }
                '<' if should_stop(pos) => return None,
                '}' if iter.peek().is_some_and(|(_, next)| *next == '}') => return Some(pos),
                _ => {}
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
