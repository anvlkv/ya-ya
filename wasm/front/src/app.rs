use std::collections::BTreeMap;
use std::collections::HashSet;

use leptos::*;
use leptos_use::{use_event_listener, use_window};
use unicode_segmentation::UnicodeSegmentation;
use wasm_bindgen::JsValue;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, Node};

const BEFORE_TRIGGER_TIMER: i32 = 300;
const TRIGGER_ANIMATED_TIMER: i32 = 800;
const TRIGGER_ATTRIBUTE: &str = "data-ya-ya-trigger-word";
const PENDING_ATTRIBUTE: &str = "data-ya-ya-pending-word";

#[derive(Debug, Clone, PartialEq)]
enum Phase {
    None,
    Pending,
}

#[derive(Debug, Clone, PartialEq)]
struct Word {
    pub text: String,
    pub start: u32,
    pub end: u32,
    pub word_pos: usize,
    pub node: Node,
}

impl Word {
    fn carret_trigger(pos: &web_sys::CaretPosition) -> Option<u64> {
        let node = pos.offset_node()?;

        let element: Option<&Element> = node.dyn_ref::<Element>();

        element
            .map(|e| e.get_attribute(TRIGGER_ATTRIBUTE).map(|s| s.parse().ok()))
            .flatten()
            .flatten()
    }

    fn carret_word(pos: &web_sys::CaretPosition) -> Option<Self> {
        let node = pos.offset_node()?;

        if node
            .dyn_ref::<Element>()
            .ok_or_else(|| node.parent_element())
            .map(|e| e.has_attribute(TRIGGER_ATTRIBUTE))
            .unwrap_or_default()
        {
            return None;
        }

        let text = node.text_content()?;
        let char_at = pos.offset();
        let mut char_caret: u32 = 0;
        let mut words = text.as_str().unicode_words().enumerate();
        while let Some((index, word)) = words.next() {
            let word_start = char_caret;
            let word_end = char_caret + word.graphemes(true).count() as u32;
            if word_end >= char_at && word_start <= char_at {
                return Some(Word {
                    start: word_start,
                    end: word_end,
                    word_pos: index,
                    text: word.to_string(),
                    node,
                });
            }
            char_caret = word_end + 1;
        }
        None
    }

    fn animate_mount(&self) -> Result<(), JsValue> {
        let text = self.node.text_content().unwrap();

        let text_before = text
            .graphemes(true)
            .take(self.start as usize)
            .collect::<String>();
        let text_after = text
            .graphemes(true)
            .skip(self.end as usize)
            .collect::<String>();

        let wd = self.text.clone();

        log::info!("text_before: {text_before}");
        log::info!("wd: {wd}");
        log::info!("text_after: {text_after}");

        let text_node_before = document().create_text_node(&text_before);
        let text_node_after = document().create_text_node(&text_after);
        let wd_node = document().create_element("mark")?;

        wd_node.set_attribute(PENDING_ATTRIBUTE, "0")?;

        wd_node.set_text_content(Some(&wd));

        let replace_node = document().create_element("span")?;

        replace_node.append_child(&text_node_before)?;
        replace_node.append_child(&wd_node)?;
        replace_node.append_child(&text_node_after)?;

        if let Some(par) = self.node.parent_node() {
            par.replace_child(&replace_node.into(), &self.node)?;
        } else {
            document().replace_child(&replace_node.into(), &self.node)?;
        }

        Ok(())
    }

    fn animate_tick(&self) -> Result<(), String> {
        Ok(())
    }

    fn revert_animate(&self) -> Result<(), String> {
        Ok(())
    }

    fn mount_trigger(&self, key: u64) -> Result<(), String> {
        Ok(())
    }
}

type MouseWordTimeout = (Word, i32, Closure<dyn FnMut()>);
type RepeatWord = (Word, Option<String>, Closure<dyn FnMut()>);

#[component]
pub fn App() -> impl IntoView {
    let show_ya = create_rw_signal(HashSet::<u64>::new());
    let data = create_rw_signal(BTreeMap::<u64, RepeatWord>::new());
    let current_word_trigger = create_rw_signal(Option::<MouseWordTimeout>::None);

    let on_trigger_repeat = Callback::new(move |key: u64| {});

    let on_trigger_complete = Callback::new({
        move |word: Word| {
            log::info!("complete: {}", word.text);
            current_word_trigger.set(None);

            let key = data.with(|d| d.keys().last().map(|k| k + 1).unwrap_or(0));

            let closure = {
                let key = key;

                Closure::wrap(Box::new(move || {
                    on_trigger_repeat.call(key);
                }) as Box<dyn FnMut()>)
            };

            data.update(|d| {
                _ = d.insert(key, (word, None, closure));
            });
        }
    });

    let on_trigger_word_animate = Callback::new({
        move |word: Word| {
            log::info!("timer: {}", word.text);
            let win = web_sys::window().unwrap();

            let closure = {
                let word = word.clone();

                Closure::wrap(Box::new(move || {
                    on_trigger_complete.call(word.clone());
                }) as Box<dyn FnMut()>)
            };

            let timer = win
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    TRIGGER_ANIMATED_TIMER,
                )
                .expect("set timer");

            word.animate_mount().unwrap();
            current_word_trigger.update(|running| {
                if let Some((_, old_timer, _)) = running.take() {
                    win.clear_timeout_with_handle(old_timer);
                }
                *running = Some((word, timer, closure));
            });
        }
    });

    let clear_mouse_move_listener = use_event_listener(use_window(), ev::mousemove, move |evt| {
        let x = evt.client_x() as f32;
        let y = evt.client_y() as f32;
        let win = web_sys::window().unwrap();
        let doc = win.document().unwrap();

        if let Some(key) = doc
            .caret_position_from_point(x, y)
            .map(|car| Word::carret_trigger(&car))
            .flatten()
            .filter(|k| data.with(|d| d.contains_key(&k)))
        {
            show_ya.update(|d| _ = d.insert(key));
        } else if let Some(word) = doc
            .caret_position_from_point(x, y)
            .map(|car| Word::carret_word(&car))
            .flatten()
        {
            if current_word_trigger.with(|t| {
                t.as_ref()
                    .map(|(current_word, _, _)| current_word != &word)
                    .unwrap_or(true)
            }) {
                let closure = {
                    let word = word.clone();

                    Closure::wrap(Box::new(move || {
                        on_trigger_word_animate.call(word.clone());
                    }) as Box<dyn FnMut()>)
                };

                let timer = win
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        BEFORE_TRIGGER_TIMER,
                    )
                    .expect("set timer");
                current_word_trigger.update(|running| {
                    if let Some((_, old_timer, _)) = running.take() {
                        win.clear_timeout_with_handle(old_timer);
                    }
                    *running = Some((word, timer, closure));
                });
            }
        } else {
            current_word_trigger.update(|running| {
                if let Some((_, old_timer, _)) = running.take() {
                    win.clear_timeout_with_handle(old_timer);
                }
                *running = None;
            });
        }
    });

    let clear_mouse_out_listener = use_event_listener(use_window(), ev::mouseleave, move |evt| {
        let win = web_sys::window().unwrap();

        current_word_trigger.update(|running| {
            if let Some((_, old_timer, _)) = running.take() {
                win.clear_timeout_with_handle(old_timer);
            }
            *running = None;
        });
    });

    on_cleanup(move || {
        clear_mouse_move_listener();
        clear_mouse_out_listener();
    });

    view! {
        <div style:color="red">
            TEST
        </div>
    }
}
