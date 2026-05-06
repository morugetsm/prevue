use std::str::FromStr;

use html5ever::{
    driver::ParseOpts,
    parse_document,
    tendril::{StrTendril, TendrilSink},
};
use markup5ever_rcdom::{Handle, NodeData, RcDom};

use crate::{Error, Result, interpolation};

pub(crate) fn parse(template: &str) -> Result<RcDom> {
    let (template, mustache_mask) = mask_template_mustaches(template);
    let dom = parse_document(RcDom::default(), ParseOpts::default())
        .from_utf8()
        .read_from(&mut template.as_bytes())
        .map_err(|source| Error::ParseTemplate { source })?;
    restore_masked_mustaches(&dom.document, &mustache_mask);
    Ok(dom)
}

struct MustacheMask {
    placeholder_prefix: String,
    entries: Vec<String>,
}

const MUSTACHE_PLACEHOLDER_SUFFIX: &str = "\u{E001}";

fn mask_template_mustaches(template: &str) -> (String, MustacheMask) {
    let placeholder_prefix = unique_mustache_placeholder_prefix(template);
    let mut masked = String::with_capacity(template.len());
    let mut entries = Vec::new();
    let mut cursor = 0;

    while cursor < template.len() {
        if template[cursor..].starts_with("{{")
            && let Some(close) =
                interpolation::find_closing_delimiter(template, cursor + 2, |pos| {
                    html_boundary_before_mustache_close(template, pos)
                })
        {
            let placeholder = format!(
                "{placeholder_prefix}{}{MUSTACHE_PLACEHOLDER_SUFFIX}",
                entries.len()
            );
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

        if let Some(tag_name) = raw_text_open_tag_name(template, cursor)
            && let Some(open_end) = find_html_tag_end(template, cursor)
        {
            let end =
                find_raw_text_close_end(template, open_end, tag_name).unwrap_or(template.len());
            masked.push_str(&template[cursor..end]);
            cursor = end;
            continue;
        }

        if looks_like_html_tag_at(template, cursor)
            && let Some(end) = find_html_tag_end(template, cursor)
        {
            masked.push_str(&template[cursor..end]);
            cursor = end;
            continue;
        }

        let ch = template[cursor..]
            .chars()
            .next()
            .expect("cursor is within template");
        masked.push(ch);
        cursor += ch.len_utf8();
    }

    (
        masked,
        MustacheMask {
            placeholder_prefix,
            entries,
        },
    )
}

fn unique_mustache_placeholder_prefix(template: &str) -> String {
    for salt in 0.. {
        let prefix = format!("\u{E000}PREVUE_MUSTACHE_{salt}_");
        if !template.contains(&prefix) {
            return prefix;
        }
    }

    unreachable!("unbounded placeholder salt search should always return")
}

fn restore_masked_mustaches(handle: &Handle, mask: &MustacheMask) {
    if mask.entries.is_empty() {
        return;
    }

    if let NodeData::Text { contents } = &handle.data {
        let restored = {
            let text = contents.borrow();
            restore_masked_text(&text, mask)
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
        restore_masked_mustaches(template_contents, mask);
    }

    for child in handle.children.borrow().iter() {
        restore_masked_mustaches(child, mask);
    }
}

fn restore_masked_text(text: &str, mask: &MustacheMask) -> Option<String> {
    let first = text.find(&mask.placeholder_prefix)?;
    let mut restored = String::with_capacity(text.len());
    let mut cursor = 0;
    let mut changed = false;

    loop {
        let start = if cursor == 0 {
            first
        } else if let Some(offset) = text[cursor..].find(&mask.placeholder_prefix) {
            cursor + offset
        } else {
            break;
        };
        let index_start = start + mask.placeholder_prefix.len();

        restored.push_str(&text[cursor..start]);

        let Some(suffix_offset) = text[index_start..].find(MUSTACHE_PLACEHOLDER_SUFFIX) else {
            restored.push_str(&text[start..]);
            cursor = text.len();
            break;
        };
        let suffix_start = index_start + suffix_offset;
        let suffix_end = suffix_start + MUSTACHE_PLACEHOLDER_SUFFIX.len();
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

fn raw_text_open_tag_name(input: &str, start: usize) -> Option<&'static str> {
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

fn find_raw_text_close_end(input: &str, from: usize, tag_name: &str) -> Option<usize> {
    let needle = format!("</{tag_name}");
    let mut cursor = from;

    while let Some(offset) = find_ascii_case_insensitive(&input[cursor..], &needle) {
        let close_start = cursor + offset;
        let after_name = close_start + needle.len();
        let valid_end = input[after_name..]
            .chars()
            .next()
            .is_none_or(|ch| ch.is_whitespace() || matches!(ch, '>' | '/'));
        if valid_end {
            return find_html_tag_end(input, close_start).or(Some(input.len()));
        }
        cursor = after_name;
    }

    None
}

fn find_ascii_case_insensitive(input: &str, needle: &str) -> Option<usize> {
    input.char_indices().find_map(|(idx, _)| {
        input[idx..]
            .get(..needle.len())
            .filter(|candidate| candidate.eq_ignore_ascii_case(needle))
            .map(|_| idx)
    })
}

fn looks_like_html_tag_at(input: &str, start: usize) -> bool {
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

fn find_html_tag_end(input: &str, start: usize) -> Option<usize> {
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

fn html_boundary_before_mustache_close(input: &str, start: usize) -> bool {
    if !looks_like_html_tag_at(input, start) {
        return false;
    }

    let Some(tag_end) = find_html_tag_end(input, start) else {
        return false;
    };
    !input[start..tag_end].contains("}}")
}
