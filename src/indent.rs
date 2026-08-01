use std::rc::{Rc, Weak};

use html5ever::tendril::StrTendril;
use markup5ever_rcdom::{Handle, NodeData};

pub(crate) fn get_indent(node: &Handle) -> Option<String> {
    let parent_weak = node.parent.take()?;
    node.parent.set(Some(Weak::clone(&parent_weak)));
    let parent = parent_weak.upgrade()?;

    let children = parent.children.borrow();
    let pos = children.iter().position(|c| Rc::ptr_eq(c, node))?;

    if pos == 0 {
        return None;
    }

    if let NodeData::Text { contents } = &children[pos - 1].data {
        let text = contents.borrow();
        if let Some(last_nl) = text.rfind('\n') {
            let indent_text = &text[last_nl..];
            return Some(
                indent_text
                    .chars()
                    .map(|c| if c == '\n' { '\n' } else { ' ' })
                    .collect(),
            );
        }
    }
    None
}

pub(crate) fn adjust_indent_in_subtree(node: &Handle, indent_adjustment: isize) {
    if let NodeData::Text { contents } = &node.data {
        let text = contents.borrow().to_string();
        let adjusted = adjust_text_indent(&text, indent_adjustment);
        contents.replace(StrTendril::from(adjusted.as_str()));
    }

    if let NodeData::Element {
        template_contents, ..
    } = &node.data
        && let Some(tc) = template_contents.borrow().as_ref()
    {
        for child in tc.children.borrow().iter() {
            adjust_indent_in_subtree(child, indent_adjustment);
        }
    } else {
        for child in node.children.borrow().iter() {
            adjust_indent_in_subtree(child, indent_adjustment);
        }
    }
}

fn adjust_text_indent(text: &str, adjustment: isize) -> String {
    if adjustment == 0 {
        return text.to_string();
    }

    let mut result = String::new();
    for (i, line) in text.split('\n').enumerate() {
        if i == 0 {
            result.push_str(line);
        } else {
            result.push('\n');

            let spaces = line.chars().take_while(|c| *c == ' ').count();
            let new_spaces = (spaces as isize + adjustment).max(0) as usize;
            let rest = &line[spaces..];
            result.push_str(&" ".repeat(new_spaces));
            result.push_str(rest);
        }
    }
    result
}
