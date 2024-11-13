mod loading;
mod popover;
mod text;
mod util;
mod word;
mod ya_text;
mod ya_word;

use std::collections::{BTreeSet, HashMap};

use leptos::*;
use leptos_use::{
    signal_debounced, use_event_listener, use_raf_fn, use_window, UseRafFnCallbackArgs,
};
use text::{TextMark, TextPermanentTrigger};
use uuid::Uuid;
use web_sys::CaretPosition;
use word::{WordMark, WordPermanentTrigger};
use ya_text::YaTextPopover;
use ya_word::YaWordPopover;

pub const BEFORE_TRIGGER_TIMER: f64 = 200.0;
pub const TRIGGER_ANIMATED_TIMER: f64 = 1600.0;
pub const MARK_ROOT_ATTRIBUTE: &str = "data-ya-ya-mark-root";
pub const TRIGGER_ATTRIBUTE_WORD: &str = "data-ya-ya-trigger-word";
pub const TRIGGER_ATTRIBUTE_TEXT: &str = "data-ya-ya-trigger-text";
pub const PENDING_ATTRIBUTE_WORD: &str = "data-ya-ya-pending-word";
pub const PENDING_ATTRIBUTE_TEXT: &str = "data-ya-ya-pending-text";
pub const BRAND_COLOR: [u8; 3] = [239, 207, 227];

#[derive(Debug, Clone, Copy, PartialEq)]
enum PermanentTrigger {
    Word(RwSignal<WordPermanentTrigger>),
    Text(RwSignal<TextPermanentTrigger>),
}

impl PermanentTrigger {
    fn unmount(&self) -> Result<(), wasm_bindgen::JsValue> {
        match self {
            Self::Word(wd) => wd.get_untracked().unmount(),
            Self::Text(tx) => tx.get_untracked().unmount(),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let extension_root = create_node_ref::<html::Div>();

    let (data, set_data) = create_signal(HashMap::<Uuid, PermanentTrigger>::new());
    let (show_ya, set_show_ya) = create_signal(BTreeSet::<Uuid>::new());

    let (word_mark, set_word_mark) = create_signal(Option::<WordMark>::None);
    let (text_mark, set_text_mark) = create_signal(Option::<TextMark>::None);
    let (caret, set_caret) = create_signal(Option::<CaretPosition>::None);
    let caret = signal_debounced(caret, BEFORE_TRIGGER_TIMER);
    let (pointer, set_pointer) = create_signal(false);

    let clear_mouse_move_listener = use_event_listener(use_window(), ev::mousemove, move |evt| {
        let x = evt.client_x() as f32;
        let y = evt.client_y() as f32;
        let win = web_sys::window().unwrap();
        let doc = win.document().unwrap();
        let car = doc.caret_position_from_point(x, y);

        if !pointer.get() {
            if let Some(root) = extension_root.get().as_deref() {
                set_caret.set(car.filter(|car| !root.contains(car.offset_node().as_ref())));
            }
        } else {
            set_word_mark.update(|c| {
                if let Some(old_mark) = c.take() {
                    log::debug!("app.rs :: Removing old WordMark");
                    old_mark.unmount().unwrap();
                }
            });
            set_text_mark.update(|c| {
                if let Some(old_mark) = c.take() {
                    log::debug!("app.rs :: Removing old TextMark");
                    old_mark.unmount().unwrap();
                }
            });
        }
    });

    let clear_pointer_down_listener =
        use_event_listener(use_window(), ev::pointerdown, move |ev| {
            set_pointer.set(ev.pointer_type().as_str() != "touch");
        });

    let clear_pointer_up_listener = use_event_listener(use_window(), ev::pointerup, move |_| {
        set_pointer.set(false);
        if let Some(selection_rng) = web_sys::window()
            .unwrap()
            .get_selection()
            .ok()
            .flatten()
            .filter(|s| !s.is_collapsed())
            .map(|s| s.get_range_at(0).ok())
            .flatten()
        {
            if let Some(new_tx_mark) = TextMark::mount_on_text(selection_rng) {
                log::debug!("app.rs :: Mounting new TextMark");
                set_text_mark.update(|c| {
                    if let Some(old_mark) = c.replace(new_tx_mark) {
                        log::debug!("app.rs :: Unmounting old TextMark");
                        old_mark.unmount().unwrap();
                    }
                });
            }
        }
    });

    create_effect(move |_| {
        if let Some(id) = caret
            .get()
            .map(|car| {
                car.offset_node()
                    .and_then(|node| WordPermanentTrigger::id(node))
            })
            .flatten()
        {
            log::debug!("app.rs :: Found WordPermanentTrigger with ID: {:?}", id);
            set_show_ya.update(|d| {
                _ = d.insert(id);
            });
        } else if let Some(new_wd_mark) = caret
            .get()
            .map(|car| {
                car.offset_node()
                    .and_then(|node| WordMark::mount_on_text(node, car.offset()))
            })
            .flatten()
        {
            log::debug!("app.rs :: Mounting new WordMark");
            set_word_mark.update(|c| {
                if let Some(old_mark) = c.replace(new_wd_mark) {
                    log::debug!("app.rs :: Unmounting old WordMark");
                    old_mark.unmount().unwrap();
                }
            });
        } else if caret
            .get()
            .map(|car| {
                car.offset_node().and_then(|node| {
                    word_mark
                        .with_untracked(|wd| wd.as_ref().map(|wd| !wd.is_same(node, car.offset())))
                })
            })
            .flatten()
            .unwrap_or(true)
        {
            set_word_mark.update(|c| {
                if let Some(old_mark) = c.take() {
                    log::debug!("app.rs :: Removing old WordMark");
                    old_mark.unmount().unwrap();
                }
            });
        }
    });

    _ = use_raf_fn(move |UseRafFnCallbackArgs { delta, .. }| {
        if word_mark.get_untracked().is_some() {
            set_word_mark.update(|set_word_mark| {
                let wd = set_word_mark.as_mut().unwrap();
                log::debug!("app.rs :: Starting tick_timer for WordMark");
                let ended = wd.tick_timer(delta);
                if ended {
                    log::debug!("app.rs :: tick_timer ended, converting WordMark to permanent");
                    let id = Uuid::new_v4();
                    let permanent = wd.into_permanent(id).unwrap();
                    *set_word_mark = None;
                    set_data.update(|set_data| {
                        log::debug!(
                            "app.rs :: Inserting permanent WordMark into data with ID: {:?}",
                            id
                        );
                        _ = set_data.insert(id, PermanentTrigger::Word(RwSignal::new(permanent)));
                    });
                    set_show_ya.update(|set_show_ya| {
                        log::debug!("app.rs :: Inserting ID into show_ya: {:?}", id);
                        _ = set_show_ya.insert(id);
                    });
                }
            });
        }
    });

    let clear_mouse_out_listener = use_event_listener(use_window(), ev::mouseleave, move |_| {
        set_word_mark.update(|c| {
            log::debug!("app.rs :: Handling mouseleave event");
            if let Some(old_mark) = c.take() {
                log::debug!("app.rs :: Unmounting old WordMark due to mouseleave");
                old_mark.unmount().unwrap();
            }
        });
    });

    on_cleanup(move || {
        clear_mouse_move_listener();
        clear_mouse_out_listener();
        clear_pointer_down_listener();
        clear_pointer_up_listener();
    });

    let visible_annotations = create_memo(move |_| {
        let data = data.get();

        show_ya
            .get()
            .into_iter()
            .filter_map(|id| data.get(&id).map(|d| (*d, id)))
            .collect::<Vec<_>>()
    });

    let close_cb = Callback::new(move |(id, quality): (Uuid, Option<bool>)| {
        set_show_ya.update(|s| {
            _ = s.remove(&id);
        });

        if quality.is_none() || quality == Some(false) {
            set_data.update(|d| {
                let an = d.get(&id).unwrap();
                an.unmount().unwrap();
                _ = d.remove(&id);
            });
        }

        if let Some(good) = quality {}
    });

    let regenerate_cb = Callback::new(move |_id: Uuid| {});

    view! {
        <div node_ref=extension_root>
            <For each=move || visible_annotations.get()
                key=|wd| wd.1
                let:word
            >
                {match word.0 {
                    PermanentTrigger::Word(wd) => view!{
                        <YaWordPopover word=wd close_cb regenerate_cb/>
                    }.into_view(),
                    PermanentTrigger::Text(tx) => view!{
                        <YaTextPopover text=tx close_cb regenerate_cb/>
                    }.into_view(),
                }}
            </For>
        </div>
    }
}
