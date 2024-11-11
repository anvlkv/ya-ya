use std::collections::BTreeMap;
use std::collections::HashSet;

use leptos::*;
use leptos_use::{use_event_listener, use_window};
use unicode_segmentation::UnicodeSegmentation;
use wasm_bindgen::JsValue;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, Node};

const BEFORE_TRIGGER_TIMER: i32 = 400;
const TRIGGER_ANIMATED_TIMER: i32 = 1200;
const TRIGGER_ATTRIBUTE: &str = "data-ya-ya-trigger-word";
const PENDING_ATTRIBUTE: &str = "data-ya-ya-pending-word";
const BRAND_COLOR: [u8; 3] = [239, 207, 227];

#[derive(Debug, Clone, PartialEq)]
struct Word {
    pub text: String,
    pub start: usize,
    pub end: usize,
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

    fn carret_animation(pos: &web_sys::CaretPosition) -> bool {
        pos.offset_node()
            .map(|node| {
                node.dyn_ref::<Element>()
                    .ok_or_else(|| node.parent_element())
                    .map(|e| e.has_attribute(PENDING_ATTRIBUTE))
                    .ok()
            })
            .flatten()
            .unwrap_or_default()
    }

    fn carret_word(pos: &web_sys::CaretPosition) -> Option<Self> {
        let mut node = pos.offset_node()?;

        if node
            .dyn_ref::<Element>()
            .ok_or_else(|| node.parent_element())
            .map(|e| e.has_attribute(TRIGGER_ATTRIBUTE))
            .unwrap_or_default()
        {
            return None;
        }

        match node.node_type() {
            1 => {
                if node.child_nodes().length() == 1 {
                    node = node.last_child().unwrap();
                } else {
                    return None;
                }
            }
            3 => {}
            _ => return None,
        }

        let text = node.text_content()?;
        let char_at = pos.offset() as usize;
        let words_map = text.chars().enumerate().fold(
            Vec::<(usize, usize, String)>::new(),
            |mut acc, (at, ch)| {
                if let Some(entry) = acc.last_mut().filter(|(_, _, w)| {
                    w.chars()
                        .rev()
                        .next()
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

    fn animate_mount(&mut self) -> Result<(), JsValue> {
        let text = self.node.text_content().unwrap();

        let text_before = text.chars().take(self.start).collect::<String>();
        let text_after = text.chars().skip(self.end).collect::<String>();

        let wd = self.text.clone();

        let text_node_before = document().create_text_node(&text_before);
        let text_node_after = document().create_text_node(&text_after);
        let wd_node = document().create_element("mark")?;

        wd_node.set_attribute(PENDING_ATTRIBUTE, "0")?;
        wd_node.set_attribute(
            "style",
            format!("--pending-animation-duration: {TRIGGER_ANIMATED_TIMER}ms; --mark-background-color: rgba({r}, {g}, {b}, 1)",
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

    fn revert_animate(&mut self) -> Result<(), JsValue> {
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

    fn mount_trigger(&self, key: u64) -> Result<(), JsValue> {
        let wd_node = self
            .node
            .dyn_ref::<Element>()
            .map(|e| {
                e.query_selector(format!("mark[{PENDING_ATTRIBUTE}]").as_str())
                    .ok()
            })
            .flatten()
            .flatten()
            .ok_or_else(|| JsValue::from_str("No mark element"))?;

        let mnt_node = document().create_element("mark")?;

        mnt_node.set_attribute(TRIGGER_ATTRIBUTE, key.to_string().as_str())?;
        mnt_node.set_attribute(
            "style",
            format!(
                "--mark-background-color: rgba({r}, {g}, {b}, 1)",
                r = BRAND_COLOR[0],
                g = BRAND_COLOR[1],
                b = BRAND_COLOR[2]
            )
            .as_str(),
        )?;

        mnt_node.set_text_content(Some(&self.text));

        self.node.replace_child(&mnt_node.into(), &wd_node.into())?;

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
        move |mut word: Word| {
            log::info!("complete: {}", word.text);
            current_word_trigger.set(None);

            let key = data.with(|d| d.keys().last().map(|k| k + 1).unwrap_or(0));

            let closure = {
                let key = key;

                Closure::wrap(Box::new(move || {
                    on_trigger_repeat.call(key);
                }) as Box<dyn FnMut()>)
            };

            word.mount_trigger(key).unwrap();

            data.update(|d| {
                _ = d.insert(key, (word, None, closure));
            });
        }
    });

    let on_trigger_word_animate = Callback::new({
        move |mut word: Word| {
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
            log::debug!("on_trigger_word_animate");
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
        let pos = doc.caret_position_from_point(x, y);
        if pos
            .as_ref()
            .map(|car| Word::carret_animation(car))
            .unwrap_or_default()
        {
            return;
        } else if let Some(key) = pos
            .as_ref()
            .map(|car| Word::carret_trigger(car))
            .flatten()
            .filter(|k| data.with(|d| d.contains_key(&k)))
        {
            show_ya.update(|d| _ = d.insert(key));
        } else if let Some(word) = pos.as_ref().map(|car| Word::carret_word(car)).flatten() {
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

                log::debug!("mousemove new word");
                current_word_trigger.update(|running| {
                    if let Some((mut old_word, old_timer, _)) = running.take() {
                        win.clear_timeout_with_handle(old_timer);
                        _ = old_word.revert_animate();
                    }
                    *running = Some((word, timer, closure));
                });
            }
        } else {
            log::debug!("mousemove clear");
            current_word_trigger.update(|running| {
                if let Some((_, old_timer, _)) = running.take() {
                    win.clear_timeout_with_handle(old_timer);
                }
                *running = None;
            });
        }
    });

    let clear_mouse_out_listener = use_event_listener(use_window(), ev::mouseleave, move |_| {
        let win = web_sys::window().unwrap();
        log::debug!("mouseleave");
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
