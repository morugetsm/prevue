use std::str::FromStr;

use html5ever::{
    driver::ParseOpts,
    parse_document,
    tendril::{StrTendril, TendrilSink},
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::{Error, Result, interpolation};

pub(crate) fn parse(template: &str) -> Result<RcDom> {
    let (template, mask) = mask_mustaches(template);
    let dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut template.as_bytes())
        .map_err(|source| Error::TemplateParse { source })?;
    restore_mustaches(&dom.document, &mask);
    Ok(dom)
}

struct Mask {
    prefix: String,
    entries: Vec<String>,
}

const MASK_SUFFIX: &str = "\u{E001}";

fn mask_mustaches(template: &str) -> (String, Mask) {
    let prefix = unique_mask_prefix(template);
    let mut masked = String::with_capacity(template.len());
    let mut entries = Vec::new();
    let mut cursor = 0;

    while cursor < template.len() {
        if template[cursor..].starts_with("{{")
            && let Some(close) =
                interpolation::find_closing_delimiter(template, cursor + 2, |pos| {
                    html_before_mustache_close(template, pos)
                })
        {
            let placeholder = format!("{prefix}{}{MASK_SUFFIX}", entries.len());
            masked.push_str(&placeholder);
            entries.push(template[cursor..close + 2].to_string());
            cursor = close + 2;
            continue;
        }

        if template[cursor..].starts_with("<!--") {
            let end = template[cursor + 4..]
                .find("-->")
                .map(|offset| cursor + 4 + offset + 3)
                .unwrap_or(template.len());
            masked.push_str(&template[cursor..end]);
            cursor = end;
            continue;
        }

        if let Some(tag_name) = raw_text_tag(template, cursor)
            && let Some(open_end) = html_tag_end(template, cursor)
        {
            let end = raw_text_close_end(template, open_end, tag_name).unwrap_or(template.len());
            masked.push_str(&template[cursor..end]);
            cursor = end;
            continue;
        }

        if looks_like_tag(template, cursor)
            && let Some(end) = html_tag_end(template, cursor)
        {
            masked.push_str(&template[cursor..end]);
            cursor = end;
            continue;
        }

        let Some(ch) = template[cursor..].chars().next() else {
            break;
        };
        masked.push(ch);
        cursor += ch.len_utf8();
    }

    (masked, Mask { prefix, entries })
}

fn unique_mask_prefix(template: &str) -> String {
    for salt in 0.. {
        let prefix = format!("\u{E000}TEMPLATE_INTERPOLATION_{salt}_");
        if !template.contains(&prefix) {
            return prefix;
        }
    }

    unreachable!("unbounded placeholder salt search should always return")
}

fn restore_mustaches(handle: &Handle, mask: &Mask) {
    if mask.entries.is_empty() {
        return;
    }

    if let NodeData::Text { contents } = &handle.data {
        let restored = {
            let text = contents.borrow();
            restore_text(&text, mask)
        };
        if let Some(restored) = restored {
            contents.replace(StrTendril::from_str(&restored).unwrap());
        }
    }

    if let NodeData::Element {
        template_contents, ..
    } = &handle.data
        && let Some(template_contents) = template_contents.borrow().as_ref()
    {
        restore_mustaches(template_contents, mask);
    }

    for child in handle.children.borrow().iter() {
        restore_mustaches(child, mask);
    }
}

fn restore_text(text: &str, mask: &Mask) -> Option<String> {
    let mut restored = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut changed = false;

    while let Some(offset) = text[cursor..].find(&mask.prefix) {
        let start = cursor + offset;
        let index_start = start + mask.prefix.len();

        restored.push_str(&text[cursor..start]);

        let Some(suffix_offset) = text[index_start..].find(MASK_SUFFIX) else {
            restored.push_str(&text[start..]);
            cursor = text.len();
            break;
        };
        let suffix_start = index_start + suffix_offset;
        let suffix_end = suffix_start + MASK_SUFFIX.len();
        let index = text[index_start..suffix_start].parse::<usize>().ok();

        if let Some(source) = index.and_then(|index| mask.entries.get(index)) {
            restored.push_str(source);
            changed = true;
        } else {
            restored.push_str(&text[start..suffix_end]);
        }
        cursor = suffix_end;
    }

    restored.push_str(&text[cursor..]);
    changed.then_some(restored)
}

fn raw_text_tag(input: &str, start: usize) -> Option<&'static str> {
    let rest = input.get(start..)?.strip_prefix('<')?;
    if rest.starts_with('/') {
        return None;
    }

    if starts_tag_name(rest, "script") {
        Some("script")
    } else if starts_tag_name(rest, "style") {
        Some("style")
    } else {
        None
    }
}

fn starts_tag_name(input: &str, name: &str) -> bool {
    let Some(head) = input.get(..name.len()) else {
        return false;
    };
    head.eq_ignore_ascii_case(name)
        && input[name.len()..]
            .chars()
            .next()
            .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '>' | '/'))
}

fn raw_text_close_end(input: &str, from: usize, tag_name: &str) -> Option<usize> {
    let needle = format!("</{tag_name}");
    let mut cursor = from;

    while let Some(offset) = find_ci(&input[cursor..], &needle) {
        let close_start = cursor + offset;
        let after_name = close_start + needle.len();
        let valid_end = input[after_name..]
            .chars()
            .next()
            .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '>' | '/'));
        if valid_end {
            return html_tag_end(input, close_start).or(Some(input.len()));
        }
        cursor = after_name;
    }

    None
}

fn find_ci(input: &str, needle: &str) -> Option<usize> {
    input.char_indices().find_map(|(idx, _)| {
        input[idx..]
            .get(..needle.len())
            .filter(|candidate| candidate.eq_ignore_ascii_case(needle))
            .map(|_| idx)
    })
}

fn looks_like_tag(input: &str, start: usize) -> bool {
    let Some(rest) = input.get(start..).and_then(|value| value.strip_prefix('<')) else {
        return false;
    };
    let Some(ch) = rest.chars().next() else {
        return false;
    };

    ch.is_ascii_alphabetic()
        || matches!(ch, '!' | '?')
        || (ch == '/'
            && rest[1..]
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic()))
}

fn html_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut quote: Option<char> = None;

    for (offset, ch) in input[start + 1..].char_indices() {
        if let Some(current_quote) = quote {
            if ch == current_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '>' => return Some(start + 1 + offset + 1),
            _ => {}
        }
    }

    None
}

fn html_before_mustache_close(input: &str, start: usize) -> bool {
    if !looks_like_tag(input, start) {
        return false;
    }

    let Some(tag_end) = html_tag_end(input, start) else {
        return false;
    };
    !input[start..tag_end].contains("}}")
}
