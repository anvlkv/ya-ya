use wasm_bindgen::JsCast;
use web_sys::{Element, Node};

use super::{
    PENDING_ATTRIBUTE_TEXT, PENDING_ATTRIBUTE_WORD, TRIGGER_ATTRIBUTE_TEXT, TRIGGER_ATTRIBUTE_WORD,
};

pub fn words_map(text: &str) -> Vec<(usize, usize, String)> {
    text.chars()
        .enumerate()
        .fold(Vec::<(usize, usize, String)>::new(), |mut acc, (at, ch)| {
            if let Some(entry) = acc.last_mut().filter(|(_, _, w)| {
                w.chars()
                    .next_back()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or_default()
                    && ch.is_alphabetic()
            }) {
                entry.1 += 1;
                entry.2.push(ch);
            } else {
                acc.push((at, at + 1, String::from(ch)));
            }
            acc
        })
}

pub fn is_al_mounted(node: &Node) -> bool {
    let mut current_node = node.clone();
    while let Some(element) = current_node.dyn_ref::<Element>().cloned().or_else(|| {
        current_node
            .parent_node()
            .map(|p| p.dyn_ref::<Element>().cloned())
            .flatten()
    }) {
        if element.has_attribute(PENDING_ATTRIBUTE_WORD)
            || element.has_attribute(TRIGGER_ATTRIBUTE_WORD)
            || element.has_attribute(PENDING_ATTRIBUTE_TEXT)
            || element.has_attribute(TRIGGER_ATTRIBUTE_TEXT)
        {
            return true;
        }
        if let Some(up) = element.parent_node() {
            current_node = up
        } else {
            break;
        }
    }
    false
}

pub fn text_node(node: Node) -> Option<Node> {
    match node.node_type() {
        1 => {
            let text = node.text_content()?;
            let children = node.child_nodes();

            for n in 0..children.length() {
                let nth = children.get(n)?;
                if let Some(n_text) = nth.text_content() {
                    if n_text == text {
                        return text_node(nth);
                    }
                }
            }
            None
        }
        3 => Some(node),
        _ => None,
    }
}
