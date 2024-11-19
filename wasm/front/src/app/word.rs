use std::str::FromStr;

use super::{
    annotation::Annotation, error::YaYaError, util::*, BRAND_COLOR, MARK_ROOT_ATTRIBUTE,
    PENDING_ATTRIBUTE_WORD, TRIGGER_ANIMATED_TIMER, TRIGGER_ATTRIBUTE_WORD,
};

use leptos::document;
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Node};

#[derive(Debug, Clone, PartialEq)]
pub struct WordMark {
    pub start: usize,
    pub end: usize,
    pub word_pos: usize,
    pub root: Element,
    pub mark: Element,
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordPermanentTrigger {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
    pub word_pos: usize,
    pub root: Element,
    pub mark: Element,
    pub anchors: (Element, Element),
    pub annotation: Option<Result<Annotation, YaYaError>>,
    pub feedback: bool,
}

impl WordMark {
    pub fn is_same(&self, node: Node, pos: u32) -> bool {
        if !self.root.contains(Some(&node)) {
            log::debug!("word.rs :: node is not contained in root");
            return false;
        }

        let text_node = if let Some(tn) = text_node(node.clone()) {
            tn
        } else {
            log::debug!("word.rs :: Failed to get text node");
            return false;
        };

        log::debug!("word.rs :: Fetching text content from the node");
        let text = if let Some(text) = text_node.text_content() {
            text
        } else {
            log::debug!("word.rs :: Failed to get text content");
            return false;
        };

        let char_at = pos as usize;

        log::debug!("word.rs :: Creating a map of words from the text content");
        let words_map = words_map(&text);

        log::debug!("word.rs :: Extracting word details based on the character position");
        let (wd, start, end, word_pos) = if let Some(word_pos) = words_map
            .iter()
            .position(|(start, end, _)| *start <= char_at && *end > char_at)
        {
            let (start, end, wd) = &words_map[word_pos];
            (wd.clone(), *start, *end, word_pos)
        } else {
            log::debug!("word.rs :: Failed to find word position");
            return false;
        };

        self.start == start
            && self.end == end
            && self.word_pos == word_pos
            && self
                .mark
                .text_content()
                .map(|t| t == wd)
                .unwrap_or_default()
    }

    pub fn mount_on_text(node: Node, pos: u32) -> Option<Self> {
        log::debug!("word.rs :: Fetching text node from the provided node");
        let text_node = text_node(node.clone())?;

        log::debug!("word.rs :: Checking if the text node is already mounted");
        if is_al_mounted(&text_node) {
            log::debug!("word.rs :: Text node is already mounted, returning None");
            return None;
        }

        log::debug!("word.rs :: Fetching text content from the node");
        let text = node.text_content()?;
        let char_at = pos as usize;

        log::debug!("word.rs :: Creating a map of words from the text content");
        let words_map = words_map(&text);

        log::debug!("word.rs :: Extracting word details based on the character position");
        let (text_before, wd, text_after, start, end, word_pos) = {
            let word_pos = words_map
                .iter()
                .position(|(start, end, _)| *start <= char_at && *end > char_at)?;
            let (start, end, wd) = &words_map[word_pos];
            let text_before = text.chars().take(*start).collect::<String>();
            let text_after = text.chars().skip(*end).collect::<String>();
            (text_before, wd.clone(), text_after, *start, *end, word_pos)
        };

        log::debug!("word.rs :: Checking if the extracted word is empty after trimming");
        if wd.trim().is_empty() {
            log::debug!("word.rs :: Word is empty, returning None");
            return None;
        }

        log::debug!("word.rs :: Creating text nodes for before and after the word");
        let text_node_before = document().create_text_node(&text_before);
        let text_node_after = document().create_text_node(&text_after);

        log::debug!("word.rs :: Creating a mark element for the word");
        let mark = document().create_element("mark").ok()?;

        log::debug!("word.rs :: Setting pending attribute on the mark element");
        mark.set_attribute(PENDING_ATTRIBUTE_WORD, "0").ok()?;

        log::debug!("word.rs :: Setting style attributes on the mark element");
        mark.set_attribute(
            "style",
            format!("--pending-animation-duration: {TRIGGER_ANIMATED_TIMER}ms; --mark-background-color: rgba({r}, {g}, {b}, 0.75)",
                r=BRAND_COLOR[0],
                g=BRAND_COLOR[1],
                b=BRAND_COLOR[2]
            ).as_str(),
        ).ok()?;

        log::debug!("word.rs :: Setting text content on the mark element");
        mark.set_text_content(Some(&wd));

        log::debug!("word.rs :: Creating a root span element");
        let root = document().create_element("span").ok()?;

        log::debug!("word.rs :: Setting style and mark root attributes on the root element");
        root.set_attribute("style", "display: inline;").ok()?;
        root.set_attribute(MARK_ROOT_ATTRIBUTE, "").ok()?;

        log::debug!("word.rs :: Appending text nodes and mark element to the root element");
        root.append_child(&text_node_before).ok()?;
        root.append_child(&mark).ok()?;
        root.append_child(&text_node_after).ok()?;

        log::debug!("word.rs :: Replacing the original text node with the root element");
        if let Some(par) = text_node.parent_node() {
            par.replace_child(&root.clone().into(), &text_node).ok()?;
        } else {
            document()
                .replace_child(&root.clone().into(), &text_node)
                .ok()?;
        }

        log::debug!("word.rs :: Returning the constructed WordMark instance");
        Some(Self {
            start,
            end,
            word_pos,
            root,
            mark,
            time: 0.0,
        })
    }

    pub fn unmount(&self) -> Result<(), JsValue> {
        log::debug!("word.rs :: Fetching text content from the root element");
        let text = self
            .root
            .text_content()
            .ok_or_else(|| JsValue::from_str("no root text"))?;

        log::debug!("word.rs :: Creating a text node with the fetched text content");
        let text_node = document().create_text_node(&text);

        log::debug!("word.rs :: Checking if the root element has a parent node");
        if let Some(par) = self.root.parent_node() {
            log::debug!(
                "word.rs :: Replacing the root element with the text node in the parent node"
            );
            par.replace_child(&text_node, &self.root.clone().into())?;
        } else {
            log::debug!("word.rs :: Replacing the root element with the text node in the document");
            document().replace_child(&text_node, &self.root.clone().into())?;
        }

        Ok(())
    }

    pub fn tick_timer(&mut self, delta: f64) -> bool {
        if self.time == 0.0 {
            self.mark
                .set_attribute(PENDING_ATTRIBUTE_WORD, "1")
                .unwrap();
        }

        self.time += delta;
        self.time >= TRIGGER_ANIMATED_TIMER
    }
}

impl WordPermanentTrigger {
    pub fn make_permanent(pending: &WordMark, id: Uuid) -> Result<WordPermanentTrigger, JsValue> {
        let mark = pending.mark.clone();

        mark.remove_attribute(PENDING_ATTRIBUTE_WORD)?;
        mark.set_attribute(TRIGGER_ATTRIBUTE_WORD, id.to_string().as_str())?;
        mark.set_attribute("id", format!("mark-{id}").as_str())?;

        let anchor_before = document().create_element("span")?;
        anchor_before.set_attribute("class", "ya-ya-anchor")?;
        mark.prepend_with_node_1(&anchor_before.clone().into())?;

        let anchor_after = document().create_element("span")?;
        anchor_after.set_attribute("class", "ya-ya-anchor")?;
        mark.append_with_node_1(&anchor_after.clone().into())?;

        Ok(WordPermanentTrigger {
            mark,
            id,
            anchors: (anchor_before, anchor_after),
            start: pending.start,
            end: pending.end,
            word_pos: pending.word_pos,
            root: pending.root.clone(),
            annotation: None,
            feedback: false,
        })
    }

    pub fn unmount(&self) -> Result<(), JsValue> {
        log::debug!("word.rs :: Fetching text content from the root element");
        let text = self
            .root
            .text_content()
            .ok_or_else(|| JsValue::from_str("no root text"))?;

        log::debug!("word.rs :: Creating a text node with the fetched text content");
        let text_node = document().create_text_node(&text);

        log::debug!("word.rs :: Checking if the root element has a parent node");
        if let Some(par) = self.root.parent_node() {
            log::debug!(
                "word.rs :: Replacing the root element with the text node in the parent node"
            );
            par.replace_child(&text_node, &self.root.clone().into())?;
        } else {
            log::debug!("word.rs :: Replacing the root element with the text node in the document");
            document().replace_child(&text_node, &self.root.clone().into())?;
        }

        Ok(())
    }

    pub fn id(node: Node) -> Option<Uuid> {
        let text_node = text_node(node)?;
        let text = Some(text_node.text_content()?);

        let mut current_node = text_node;

        while let Some(element) = current_node.dyn_ref::<Element>().cloned().or_else(|| {
            current_node
                .parent_node()
                .and_then(|p| p.dyn_ref::<Element>().cloned())
        }) {
            if let Some(id) = element.get_attribute(TRIGGER_ATTRIBUTE_WORD) {
                return Uuid::from_str(&id).ok();
            } else if text != element.text_content() {
                break;
            } else if let Some(up) = element.parent_node() {
                current_node = up
            } else {
                break;
            }
        }

        None
    }

    pub fn word(&self) -> String {
        self.mark
            .text_content()
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    pub fn context(&self) -> String {
        let mut current_node: Node = self.root.clone().into();
        let word = self.word();

        while let Some(element) = current_node.parent_node() {
            if let Some(parent_text) = element.text_content() {
                let parent_text_without_spaces: String = parent_text.split_whitespace().collect();
                if parent_text_without_spaces.contains(&word) {
                    let words: Vec<&str> = parent_text.split_whitespace().collect();
                    if let Some(word_pos) = words.iter().position(|&w| w == word) {
                        let start = word_pos.saturating_sub(3);
                        let end = (word_pos + 4).min(words.len());
                        return words[start..end].join(" ");
                    }
                }
            }
            current_node = element;
        }

        // Fallback to the original root if no parent contains the word
        let text = self.root.text_content().unwrap_or_default();
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_pos = words.iter().position(|&w| w == word).unwrap_or(0);

        let start = word_pos.saturating_sub(3);
        let end = (word_pos + 4).min(words.len());

        words[start..end].join(" ")
    }
}
