mod word;

use std::collections::BTreeMap;
use std::collections::HashSet;

use leptos::*;
use leptos_use::{use_event_listener, use_window};
use wasm_bindgen::{prelude::Closure, JsCast};
use word::Word;

pub const BEFORE_TRIGGER_TIMER: i32 = 400;
pub const TRIGGER_ANIMATED_TIMER: i32 = 1200;
pub const TRIGGER_ATTRIBUTE: &str = "data-ya-ya-trigger-word";
pub const PENDING_ATTRIBUTE: &str = "data-ya-ya-pending-word";
pub const BRAND_COLOR: [u8; 3] = [239, 207, 227];

type MouseWordTimeout = (Word, i32, Closure<dyn FnMut()>);
type RepeatWord = (Word, Option<String>, Closure<dyn FnMut()>);

#[component]
pub fn App() -> impl IntoView {
    let show_ya = create_rw_signal(HashSet::<u64>::new());
    let data = create_rw_signal(BTreeMap::<u64, RepeatWord>::new());
    let current_word_trigger = create_rw_signal(Option::<MouseWordTimeout>::None);

    let on_trigger_repeat = Callback::new(move |key: u64| {
        show_ya.update(|s| {
            s.insert(key);
        });
    });

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

            log::info!("mount: {word:#?}");

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
            .map(Word::carret_animation)
            .unwrap_or_default()
        {
        } else if let Some(key) = pos
            .as_ref()
            .and_then(Word::carret_trigger)
            .filter(|k| data.with(|d| d.contains_key(k)))
        {
            show_ya.update(|d| _ = d.insert(key));
        } else if let Some(word) = pos.as_ref().and_then(Word::carret_word) {
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
