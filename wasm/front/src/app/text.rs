use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;

use super::{
    util::text_node, BRAND_COLOR, MARK_ROOT_ATTRIBUTE, PENDING_ATTRIBUTE_TEXT,
    TRIGGER_ANIMATED_TIMER, TRIGGER_ATTRIBUTE_TEXT,
};

use leptos::document;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use web_sys::{Element, Node, NodeFilter, Range};

#[derive(Debug, Clone, PartialEq)]
pub struct TextMark {
    pub start: usize,
    pub end: usize,
    pub root: Element,
    pub start_el: Element,
    pub end_el: Option<Element>,
    pub marks: Vec<Element>,
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPermanentTrigger {
    pub id: Uuid,
    pub start: usize,
    pub end: usize,
    pub root: Element,
    pub start_el: Element,
    pub end_el: Option<Element>,
    pub marks: Vec<Element>,
    pub annotation: Option<String>,
}

impl TextMark {
    pub fn mount_on_text(range: Range) -> Option<Self> {
        log::debug!("text.rs :: Fetching common ancestor container from the range");
        let root = range.common_ancestor_container().ok()?;

        log::debug!("text.rs :: Casting common ancestor container to an Element");
        let root_element = root.dyn_ref::<Element>().cloned()?;

        log::debug!("text.rs :: Fetching start container from the range");
        let start_container = range.start_container().ok()?;
        log::debug!("text.rs :: Fetching start offset from the range");
        let start_offset = range.start_offset().ok()? as usize;
        log::debug!("text.rs :: Fetching end container from the range");
        let end_container = range.end_container().ok()?;
        log::debug!("text.rs :: Fetching end offset from the range");
        let end_offset = range.end_offset().ok()? as usize;

        let same_container = end_container == start_container;

        log::debug!("text.rs :: Wrapping text nodes within the specified range");

        let mut marks = vec![];
        let mut start_encountered = false;
        let mut end_encountered = false;

        let iter = document()
            .create_node_iterator_with_what_to_show(
                &root,
                0x4, // NodeFilter.SHOW_TEXT
            )
            .ok()?;

        let mut start_el = None;
        let mut end_el = None;

        let mut current_node = iter.next()?.dyn_ref::<Element>().cloned()?;

        while let Some(element) = current_node {
            let node_type = element.node_type();

            if node_type == 3 { // NodeFilter.SHOW_TEXT
                let node_offset = element.node_value().unwrap_or_default().len();

                if start_encountered && !end_encountered {
                    if node_offset < start_offset || node_offset >= end_offset {
                        let (before, after, start_text, end_text) = if node_offset < start_offset {
                            (String::new(), element.text_content().unwrap_or_default().to_string(), text_offset_to_text(&range, start_offset - node_offset), String::new())
                        } else {
                            (text_offset_to_text(&range, start_offset), element.text_content().unwrap_or_default().to_string(), "".to_string(), text_offset_to_text(&range, end_offset - node_offset - 1))
                        };

                        let mut el = document().create_element("span").ok()?;
                        el.append_child(&document().create_text_node(&before).ok()?)?;
                        let mark = document().create_element("mark").ok()?;
                        mark.set_attribute(PENDING_ATTRIBUTE_TEXT, "0").ok()?;
                        mark.set_attribute(
                            "style",
                            format!(
                                "background-color: rgba({}, {}, {}, 0.75);",
                                BRAND_COLOR[0], BRAND_COLOR[1], BRAND_COLOR[2]
                            )
                            .as_str(),
                        )?;
                        el.append_child(&mark)?;
                        el.append_child(&document().create_text_node(&end_text).ok()?)?;
                        el.set_attribute("style", "display: inline;").ok()?;
                        el.set_attribute(MARK_ROOT_ATTRIBUTE, "").ok()?;
                        root_element.append_child(&el)?;

                        marks.push(mark);
                    }
                } else if !start_encountered && node_offset == start_offset {
                    start_encountered = true;
                    start_el = Some(element);
                    element.split_text(start_offset).ok()?;
                    if !same_container {
                        let (content, next_sibling) = if let Some(next) = element.next_sibling() {
                            (element.text_content().unwrap_or_default().to_string(), next)
                        } else {
                            (String::new(), document().create_text_node("").ok()?)
                        };

                        let mut el = document().create_element("span").ok()?;
                        el.append_child(&document().create_text_node(&content).ok()?)?;
                        el.append_child(&document().create_text_node(&next_sibling.text_content().unwrap_or_default()).ok()?)?;
                        root_element.replace_child(&el, element)?;
                        element = next_sibling;
                    }
                } else if start_encountered && node_offset == end_offset {
                    end_encountered = true;
                    end_el = Some(element);
                    element.split_text(1).ok()?;
                    if end_offset == end_container.start_offset() as usize && same_container {
                        let text = if let Some(prev) = element.previous_sibling() {
                            prev.text_content().unwrap_or_default().to_string() + &element.text_content().unwrap_or_default()
                        } else {
                            String::new()
                        };

                        let mut el = document().create_element("span").ok()?;
                        el.append_child(&document().create_text_node(&text).ok()?)?;
                        root_element.replace_child(&el, element)?;
                        element = el;
                    } else {
                        element.split_text(1).ok()?;
                        let (content, prev): (String, Option<Element>) = if let Some(prev) = element.previous_sibling() {
                            (element.text_content().unwrap_or_default().to_string(), Some(prev))
                        } else {
                            (String::new(), None)
                        };

                        let mut el = document().create_element("span").ok()?;
                        el.append_child(&document().create_text_node(&content).ok()?)?;
                        el.append_child(&document().create_text_node(&element.text_content().unwrap_or_default()).ok()?)?;
                        root_element.replace_child(&el, element)?;

                        for (i, n) in el.child_nodes().iter().enumerate() {
                            if let Some(mark) = n.dyn_ref::<Element>().and_then(|e| marks.iter().position(|m| e.text_content().unwrap_or_default() == m.text_content().unwrap_or_default())) {
                                if i > 0 && let Some(prev_el) = el[child_nodes].get(i - 1).and_deref().dyn_ref::<Element>() {
                                    prev_el.append_child(&matches!(prev, Some(e) => e, None => document().create_text_node("").ok()).ok()?)?;
                                    document().remove_child(&el)?;
                                    element = prev_el;
                                    break;
                                }
                            }
                        }
                    }
                }

                current_node = iter.next()?.dyn_ref::<Element>().cloned()?;
            } else if node_type == 1 && element.has_attribute(PENDING_ATTRIBUTE_TEXT) {
                if start_encountered && !end_encountered {
                    let (before, after, start_text, end_text) = if element.offset_within(&element).0 < start_offset as u32 {
                        (String::new(), element.text_content().unwrap_or_default().to_string(), text_offset_to_text(&range, start_offset - element.offset_within(&element).0 as usize - 1), String::new())
                    } else {
                        (text_offset_to_text(&range, element.offset_within(&element).0 as usize), element.text_content().unwrap_or_default().to_string(), "".to_string(), text_offset_to_text(&range, end_offset - element.offset_within(&element).0 as usize - 1))
                    };

                    let mut el = document().create_element("span").ok()?;
                    el.append_child(&document().create_text_node(&before).ok()?)?;
                    let mark = document().create_element("mark").ok()?;
                    mark.set_attribute(PENDING_ATTRIBUTE_TEXT, "0").ok()?;
                    mark.set_attribute(
                        "style",
                        format!(
                            "background-color: rgba({}, {}, {}, 0.75);",
                            BRAND_COLOR[0], BRAND_COLOR[1], BRAND_COLOR[2]
                        )
                        .as_str(),
                    )?;
                    el.append_child(&mark)?;
                    el.append_child(&document().create_text_node(&end_text).ok()?)?;
                    el.set_attribute("style", "display: inline;").ok()?;
                    el.set_attribute(MARK_ROOT_ATTRIBUTE, "").ok()?;
                    root_element.append_child(&el)?;

                    marks.push(mark);
                }
            } else if !element.has_attribute(PENDING_ATTRIBUTE_TEXT) {
                current_node = Some(document().create_text_node("").ok()?.cast::<Element>()?);
            } else {
                current_node = element.next_sibling()?.dyn_ref::<Element>().cloned()?;
            }
        }

        log::debug!("text.rs :: Setting mark root attribute on the root element");
        root_element.set_attribute(MARK_ROOT_ATTRIBUTE, "").ok()?;

        log::debug!("text.rs :: Returning the constructed TextMark instance");
        Some(Self {
            start: start_offset,
            end: end_offset,
            root: root_element,
            start_el: start_el?,
            marks,
            end_el,
            time: 0.0,
        })
    }

    pub fn unmount(&self) -> Result<(), JsValue> {
        log::debug!("text.rs :: Unmounting TextMark");

        for mark in &self.marks {
            if let Some(par) = mark.parent_node() {
                // Restore the original text content
                if let Some(text_content) = mark.text_content() {
                    let text_node = document().create_text_node(&text_content);
                    par.replace_child(&text_node, mark)?;
                }
            }
        }

        // Remove mark root attribute from the root element
        self.root.remove_attribute(MARK_ROOT_ATTRIBUTE)?;

        Ok(())
    }

    pub fn tick_timer(&mut self, delta: f64) -> bool {
        if self.time == 0.0 {
            for mark in &self.marks {
                mark.set_attribute(PENDING_ATTRIBUTE_TEXT, "1").unwrap();
            }
        }

        self.time += delta;
        self.time >= TRIGGER_ANIMATED_TIMER
    }

    pub fn make_permanent(&self, id: Uuid) -> Result<TextPermanentTrigger, JsValue> {
        let mut marks = Vec::new();

        for (at, mark) in self.marks.iter().enumerate() {
            mark.remove_attribute(PENDING_ATTRIBUTE_TEXT)?;
            mark.set_attribute(TRIGGER_ATTRIBUTE_TEXT, id.to_string().as_str())?;
            mark.set_attribute("id", format!("mark--{at}--{id}").as_str())?;
            mark.set_attribute(
                "style",
                format!(
                    "background-color: rgba({}, {}, {}, 1);",
                    BRAND_COLOR[0], BRAND_COLOR[1], BRAND_COLOR[2]
                )
                .as_str(),
            )?;
            marks.push(mark.clone());
        }

        Ok(TextPermanentTrigger {
            marks,
            id,
            start: self.start,
            end: self.end,
            start_el: self.start_el.clone(),
            end_el: self.end_el.clone(),
            root: self.root.clone(),
            annotation: None,
        })
    }
}

impl TextPermanentTrigger {
    pub fn unmount(&self) -> Result<(), JsValue> {
        for mark in &self.marks {
            if let Some(par) = mark.parent_node() {
                // Restore the original text content
                if let Some(text_content) = mark.text_content() {
                    let text_node = document().create_text_node(&text_content);
                    par.replace_child(&text_node, mark)?;
                }
            }
        }

        // Remove mark root attribute from the root element
        self.root.remove_attribute(MARK_ROOT_ATTRIBUTE)?;

        Ok(())
    }

    pub fn id(node: Node) -> Option<Uuid> {
        let mut current_node = node;

        while let Some(element) = current_node.dyn_ref::<Element>().cloned().or_else(|| {
            current_node
                .parent_node()
                .and_then(|p| p.dyn_ref::<Element>().cloned())
        }) {
            if let Some(id) = element.get_attribute(TRIGGER_ATTRIBUTE_TEXT) {
                return Uuid::from_str(&id).ok();
            } else if let Some(up) = element.parent_node() {
                current_node = up;
            } else {
                break;
            }
        }

        None
    }
}
