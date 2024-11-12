use super::{BRAND_COLOR, PENDING_ATTRIBUTE, TRIGGER_ANIMATED_TIMER, TRIGGER_ATTRIBUTE};

use leptos::document;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CaretPosition, Element, Node};

#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub word_pos: usize,
    pub node: Node,
}

impl PartialEq for Word {
    fn eq(&self, other: &Word) -> bool {
        self.node == other.node && self.start == other.start && self.end == other.end
    }
}

impl Word {
    pub fn carret_trigger(pos: &CaretPosition) -> Option<u64> {
        let mut current_node = pos.offset_node().and_then(Word::text_node)?;

        while let Some(element) = current_node.dyn_ref::<Element>() {
            if let Some(attr) = element.get_attribute(TRIGGER_ATTRIBUTE) {
                return attr.parse().ok();
            }
            current_node = element.parent_node()?;
        }

        None
    }

    pub fn carret_animation(pos: &CaretPosition) -> bool {
        pos.offset_node()
            .and_then(Word::text_node)
            .map(|node| {
                let mut current_node = node;
                while let Some(element) = current_node.dyn_ref::<Element>() {
                    if element.has_attribute(PENDING_ATTRIBUTE) {
                        return true;
                    }
                    if let Some(up) = element.parent_node() {
                        current_node = up
                    } else {
                        break;
                    }
                }
                false
            })
            .unwrap_or_default()
    }

    fn text_node(node: Node) -> Option<Node> {
        match node.node_type() {
            1 => {
                let text = node.text_content()?;
                let children = node.child_nodes();

                for n in 0..children.length() {
                    let nth = children.get(n)?;
                    if let Some(n_text) = nth.text_content() {
                        if n_text == text {
                            return Word::text_node(nth);
                        }
                    }
                }
                None
            }
            3 => Some(node),
            _ => None,
        }
    }

    pub fn carret_word(pos: &CaretPosition) -> Option<Self> {
        let node = pos.offset_node()?;

        if node
            .dyn_ref::<Element>()
            .ok_or_else(|| node.parent_element())
            .map(|e| e.has_attribute(TRIGGER_ATTRIBUTE) || e.has_attribute(PENDING_ATTRIBUTE))
            .unwrap_or_default()
        {
            return None;
        }

        let node = Word::text_node(node)?;

        let text = node.text_content()?;
        let char_at = pos.offset() as usize;
        let words_map = text.chars().enumerate().fold(
            Vec::<(usize, usize, String)>::new(),
            |mut acc, (at, ch)| {
                if let Some(entry) = acc.last_mut().filter(|(_, _, w)| {
                    w.chars().next_back()
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
            },
        );

        for (word_pos, (start, end, text)) in words_map.into_iter().enumerate() {
            if start <= char_at && end > char_at {
                if text.chars().all(|c| c.is_alphabetic()) {
                    return Some(Word {
                        text,
                        start,
                        end,
                        word_pos,
                        node,
                    });
                } else {
                    break;
                }
            }
        }

        None
    }

    pub fn animate_mount(&mut self) -> Result<(), JsValue> {
        let text = self.node.text_content().unwrap();

        let text_before = text.chars().take(self.start).collect::<String>();
        let text_after = text.chars().skip(self.end).collect::<String>();

        let wd = self.text.clone();

        let text_node_before = document().create_text_node(&text_before);
        let text_node_after = document().create_text_node(&text_after);
        let wd_node = document().create_element("mark")?;

        wd_node.set_attribute(PENDING_ATTRIBUTE, "1")?;
        wd_node.set_attribute(
            "style",
            format!("--pending-animation-duration: {TRIGGER_ANIMATED_TIMER}ms; --mark-background-color: rgba({r}, {g}, {b}, 0.75)",
                r=BRAND_COLOR[0],
                g=BRAND_COLOR[1],
                b=BRAND_COLOR[2]
            ).as_str(),
        )?;

        wd_node.set_text_content(Some(&wd));

        let replace_node = document().create_element("span")?;

        replace_node.set_attribute("style", "display: inline;")?;
        replace_node.append_child(&text_node_before)?;
        replace_node.append_child(&wd_node)?;
        replace_node.append_child(&text_node_after)?;

        if let Some(par) = self.node.parent_node() {
            par.replace_child(&replace_node.clone().into(), &self.node)?;
        } else {
            document().replace_child(&replace_node.clone().into(), &self.node)?;
        }

        self.node = replace_node.into();

        Ok(())
    }

    pub fn revert_animate(&mut self) -> Result<(), JsValue> {
        let text = self.node.text_content().unwrap();
        let text_node = document().create_text_node(&text);

        if let Some(par) = self.node.parent_node() {
            par.replace_child(&text_node.clone().into(), &self.node)?;
        } else {
            document().replace_child(&text_node.clone().into(), &self.node)?;
        }

        self.node = text_node.into();

        Ok(())
    }

    pub fn mount_trigger(&mut self, key: u64) -> Result<(), JsValue> {
        let text = self.node.text_content().unwrap();

        let text_before = text.chars().take(self.start).collect::<String>();
        let text_after = text.chars().skip(self.end).collect::<String>();

        let wd = self.text.clone();

        let text_node_before = document().create_text_node(&text_before);
        let text_node_after = document().create_text_node(&text_after);
        let wd_node = document().create_element("mark")?;

        wd_node.set_attribute(TRIGGER_ATTRIBUTE, key.to_string().as_str())?;
        wd_node.set_attribute(
            "style",
            format!("--mark-background-color: rgba({r}, {g}, {b}, 0.75)",
                r=BRAND_COLOR[0],
                g=BRAND_COLOR[1],
                b=BRAND_COLOR[2]
            ).as_str(),
        )?;

        wd_node.set_text_content(Some(&wd));

        let replace_node = document().create_element("span")?;

        replace_node.set_attribute("style", "display: inline;")?;
        replace_node.append_child(&text_node_before)?;
        replace_node.append_child(&wd_node)?;
        replace_node.append_child(&text_node_after)?;

        if let Some(par) = self.node.parent_node() {
            par.replace_child(&replace_node.clone().into(), &self.node)?;
        } else {
            document().replace_child(&replace_node.clone().into(), &self.node)?;
        }

        self.node = replace_node.into();

        Ok(())
    }
}
