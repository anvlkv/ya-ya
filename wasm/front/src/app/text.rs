use std::str::FromStr;

use super::{
    util::text_node, BRAND_COLOR, MARK_ROOT_ATTRIBUTE, PENDING_ATTRIBUTE_TEXT, TRIGGER_ANIMATED_TIMER,
    TRIGGER_ATTRIBUTE_TEXT,
};

use leptos::document;
use uuid::Uuid;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Node, Range};

#[derive(Debug, Clone, PartialEq)]
pub struct TextMark {
    pub start: usize,
    pub end: usize,
    pub root: Element,
    pub marks: Vec<Element>,
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPermanentTrigger {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
    pub root: Element,
    pub marks: Vec<Element>,
    pub annotation: Option<String>,
}

impl TextMark {
    pub fn is_same(&self, range: Range, pos: u32) -> bool {
        todo!()
    }

    pub fn mount_on_text(range: Range) -> Option<Self> {
        log::debug!("text.rs :: Fetching common ancestor container from the range");
        let root = range.common_ancestor_container().ok()?;

        log::debug!("text.rs :: Casting common ancestor container to an Element");
        let root_element = root.dyn_ref::<Element>().cloned()?;

        log::debug!("text.rs :: Fetching start container from the range");
        let start_container = range.start_container().ok()?;
        log::debug!("text.rs :: Fetching start offset from the range");
        let start_offset = range.start_offset().ok()?;
        log::debug!("text.rs :: Fetching end container from the range");
        let end_container = range.end_container().ok()?;
        log::debug!("text.rs :: Fetching end offset from the range");
        let end_offset = range.end_offset().ok()?;

        log::debug!("text.rs :: Wrapping text nodes within the specified range");
        let (marks, start, end) = wrap_text_nodes(root, start_container, start_offset, end_container, end_offset)?;

        log::debug!("text.rs :: Setting mark root attribute on the root element");
        root_element.set_attribute(MARK_ROOT_ATTRIBUTE, "").ok()?;

        log::debug!("text.rs :: Returning the constructed TextMark instance");
        Some(Self {
            start,
            end,
            root: root_element,
            marks,
            time: 0.0,
        })
    }

    pub fn unmount(&self) -> Result<(), JsValue> {
        todo!()

        // Ok(())
    }

    pub fn tick_timer(&mut self, delta: f64) -> bool {
        if self.time == 0.0 {
            for mark in &self.marks {
                mark.set_attribute(PENDING_ATTRIBUTE_TEXT, "1").unwrap();
            }
        }

        self.time += delta;
        self.time >= TRIGGER_ANIMATED_TIMER as f64
    }

    pub fn into_permanent(&self, id: Uuid) -> Result<TextPermanentTrigger, JsValue> {
        let mut marks = Vec::new();

        for (at, mark) in self.marks.iter().enumerate() {
            mark.remove_attribute(PENDING_ATTRIBUTE_TEXT)?;
            mark.set_attribute(TRIGGER_ATTRIBUTE_TEXT, id.to_string().as_str())?;
            mark.set_attribute("id", format!("mark--{at}--{id}").as_str())?;
            marks.push(mark.clone());
        }

        Ok(TextPermanentTrigger {
            marks,
            id,
            start: self.start,
            end: self.end,
            root: self.root.clone(),
            annotation: None,
        })
    }
}

fn wrap_text_nodes(node: Node, start_container: Node, start_offset: u32, end_container: Node, end_offset: u32) -> Option<(Vec<Element>, usize, usize)> {
    let mut marks = Vec::new();
    let mut start = None;
    let mut end = None;
    let nodes = node.child_nodes();
    let count = nodes.length();

    log::debug!("text.rs :: Iterating over child nodes to find start and end containers");
    for i in 0..count {
        let child = nodes.get(i)?;
        if child == start_container {
            log::debug!("text.rs :: Found start container, attempting to wrap text nodes");
            if let Some((child_marks, child_start, child_end)) = wrap_text_nodes(child.clone(), start_container.clone(), start_offset, end_container.clone(), end_offset) {
                marks.extend(child_marks);
                start = Some(child_start);
                end = Some(child_end);
                break;
            }
        } else if child == end_container {
            log::debug!("text.rs :: Found end container, attempting to wrap text nodes");
            if let Some((child_marks, child_start, child_end)) = wrap_text_nodes(child.clone(), start_container.clone(), start_offset, end_container.clone(), end_offset) {
                marks.extend(child_marks);
                start = Some(child_start);
                end = Some(child_end);
                break;
            }
        } else {
            log::debug!("text.rs :: Wrapping text nodes for intermediate child node");
            if let Some((child_marks, child_start, child_end)) = wrap_text_nodes(child.clone(), start_container.clone(), start_offset, end_container.clone(), end_offset) {
                marks.extend(child_marks);
                if start.is_none() {
                    start = Some(child_start);
                }
                end = Some(child_end);
            }
        }
    }

    if node == start_container && node.node_type() == 3 {
        log::debug!("text.rs :: Node is a text node and matches start container, wrapping text");
        let text = node.text_content()?;
        let text_before = text.chars().take(start_offset as usize).collect::<String>();
        let text_marked = text.chars().skip(start_offset as usize).take((end_offset - start_offset) as usize).collect::<String>();
        let text_after = text.chars().skip(end_offset as usize).collect::<String>();

        let mark = document().create_element("mark").ok()?;
        log::debug!("text.rs :: Setting pending attribute on the mark element");
        mark.set_attribute(PENDING_ATTRIBUTE_TEXT, "0").ok()?;
        log::debug!("text.rs :: Setting text content on the mark element");
        mark.set_text_content(Some(&text_marked));
        marks.push(mark);

        let text_node_before = document().create_text_node(&text_before);
        let text_node_after = document().create_text_node(&text_after);

        log::debug!("text.rs :: Replacing original text node with text node before the marked text");
        node.parent_node()?.replace_child(&text_node_before, &node).ok()?;
        log::debug!("text.rs :: Inserting mark element before the text node after the marked text");
        node.parent_node()?.insert_before(&marks[0], Some(&text_node_after)).ok()?;
        log::debug!("text.rs :: Inserting text node after the marked text");
        node.parent_node()?.insert_before(&text_node_after, None).ok()?;

        start = Some(start_offset as usize);
        end = Some(end_offset as usize);
    }

    Some((marks, start?, end?))
}

impl TextPermanentTrigger {
    pub fn unmount(&self) -> Result<(), JsValue> {
        todo!()

        // Ok(())
    }

    pub fn id() -> Option<Uuid> {
        todo!()

        // None
    }
}
