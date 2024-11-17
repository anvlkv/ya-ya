mod loading;
mod popover;
mod util;
mod word;
mod ya_word;

use std::collections::{BTreeSet, HashMap};

use leptos::*;
use leptos_use::{
    signal_debounced, use_event_listener, use_raf_fn, use_window, UseRafFnCallbackArgs,
};
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::CaretPosition;
use word::{WordMark, WordPermanentTrigger};
use ya_word::YaWordPopover;

pub const BEFORE_TRIGGER_TIMER: f64 = 200.0;
pub const TRIGGER_ANIMATED_TIMER: f64 = 1800.0;
pub const MARK_ROOT_ATTRIBUTE: &str = "data-ya-ya-mark-root";
pub const TRIGGER_ATTRIBUTE_WORD: &str = "data-ya-ya-trigger-word";
pub const PENDING_ATTRIBUTE_WORD: &str = "data-ya-ya-pending-word";
pub const BRAND_COLOR: [u8; 3] = [239, 207, 227];

#[derive(Debug, Clone, Copy, PartialEq)]
enum PermanentTrigger {
    Word(RwSignal<WordPermanentTrigger>),
}

impl PermanentTrigger {
    fn unmount(&self) -> Result<(), wasm_bindgen::JsValue> {
        match self {
            Self::Word(wd) => wd.get_untracked().unmount(),
        }
    }

    fn word(&self) -> String {
        match self {
            Self::Word(wd) => wd
                .get_untracked()
                .mark
                .text_content()
                .unwrap_or_default()
                .trim()
                .to_string(),
        }
    }

    fn context(&self) -> String {
        match self {
            Self::Word(wd) => wd
                .get_untracked()
                .root
                .parent_element()
                .map(|par| par.text_content())
                .flatten()
                .unwrap_or_default(),
        }
    }

    fn annotate(&self, value: Option<String>) {
        match self {
            Self::Word(wd) => wd.update(|wd| wd.annotation = value),
        }
    }

    fn annotation(&self) -> Option<String> {
        match self {
            Self::Word(wd) => wd.get_untracked().annotation.clone(),
        }
    }

    fn feedback(&self, val: bool) {
        match self {
            Self::Word(wd) => wd.update_untracked(|wd| wd.feedback = val),
        }
    }

    fn skip_feedback(&self) -> bool {
        match self {
            Self::Word(wd) => wd.get_untracked().feedback,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum PendingMark {
    Word(WordMark),
}

impl PendingMark {
    fn unmount(&self) -> Result<(), wasm_bindgen::JsValue> {
        match self {
            Self::Word(wd) => wd.unmount(),
        }
    }

    fn tick_timer(&mut self, delta: f64) -> bool {
        match self {
            Self::Word(wd) => wd.tick_timer(delta),
        }
    }

    fn make_permanent(&self, id: Uuid) -> Result<PermanentTrigger, JsValue> {
        match self {
            Self::Word(wd) => wd
                .make_permanent(id)
                .map(|d| PermanentTrigger::Word(RwSignal::new(d))),
        }
    }

    fn is_same(&self, node: web_sys::Node, pos: u32) -> bool {
        match self {
            Self::Word(wd) => wd.is_same(node, pos),
        }
    }
}

async fn translate_word(word: String, context: String, previous: Option<String>) -> String {
    let client = reqwest::Client::new();

    let body = json::object! {
        word: word,
        context: context,
        previous: previous
    };

    client
        .post(format!("{}/translate-word", crate::env::EXTENSION_TRANSLATE_URL).as_str())
        .body(json::stringify(body))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}

async fn success_record(word: String, context: String, translation: String, result: bool) {
    let client = reqwest::Client::new();

    let body = json::object! {
        word: word,
        context: context,
        translation: translation,
        result: result
    };

    client
        .post(format!("{}/success-record", crate::env::EXTENSION_TRANSLATE_URL).as_str())
        .body(json::stringify(body))
        .send()
        .await
        .unwrap();
}

#[component]
pub fn App() -> impl IntoView {
    let extension_root = create_node_ref::<html::Div>();

    let (data, set_data) = create_signal(HashMap::<Uuid, PermanentTrigger>::new());
    let (show_ya, set_show_ya) = create_signal(Vec::<Uuid>::new());

    let (pending_mark, set_pending_mark) = create_signal(Option::<PendingMark>::None);

    let (caret, set_caret) = create_signal(Option::<CaretPosition>::None);
    let caret = signal_debounced(caret, BEFORE_TRIGGER_TIMER);
    let (pointer, set_pointer) = create_signal(false);

    let translate_action = create_action(
        |(id, ctx, word, prev): &(Uuid, String, String, Option<String>)| {
            let ctx = ctx.clone();
            let word = word.clone();
            let prev = prev.clone();
            let id = *id;

            async move {
                let translation = translate_word(word, ctx, prev).await;

                (id, translation)
            }
        },
    );

    let success_record_action = create_action(
        |(word, ctx, annotation, result): &(String, String, String, bool)| {
            let ctx = ctx.clone();
            let word = word.clone();
            let annotation = annotation.clone();
            let result = *result;

            async move {
                success_record(word, ctx, annotation, result).await;
            }
        },
    );

    let replace_pending = Callback::new(move |mark: Option<PendingMark>| {
        set_pending_mark.update(|c| {
            if let Some(old_mark) = c.take() {
                log::debug!("app.rs :: Removing old PendingMark");
                old_mark.unmount().unwrap();
            }
            *c = mark;
        });
    });

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
            replace_pending.call(None)
        }
    });

    let clear_pointer_down_listener =
        use_event_listener(use_window(), ev::pointerdown, move |ev| {
            set_pointer.set(ev.pointer_type().as_str() != "touch");
        });

    let clear_pointer_up_listener = use_event_listener(use_window(), ev::pointerup, move |_| {
        set_pointer.set(false);
    });

    create_effect(move |_| {
        let no_selection = web_sys::window()
            .unwrap()
            .get_selection()
            .ok()
            .flatten()
            .map(|s| s.is_collapsed())
            .unwrap_or(true);

        if let Some(id) = caret.get().and_then(|car| {
            car.offset_node()
                .and_then(|node| WordPermanentTrigger::id(node.clone()))
        }) {
            log::debug!("app.rs :: Found PermanentTrigger with ID: {:?}", id);
            set_show_ya.update(|d| {
                if !d.contains(&id) {
                    d.push(id);
                }
            });
        } else if let Some(new_wd_mark) = caret.get().filter(|_| no_selection).and_then(|car| {
            car.offset_node()
                .and_then(|node| WordMark::mount_on_text(node, car.offset()))
        }) {
            log::debug!("app.rs :: Mounting new WordMark");
            replace_pending.call(Some(PendingMark::Word(new_wd_mark)));
        } else if caret
            .get()
            .filter(|_| no_selection)
            .and_then(|car| {
                car.offset_node().and_then(|node| {
                    pending_mark
                        .with_untracked(|wd| wd.as_ref().map(|wd| !wd.is_same(node, car.offset())))
                })
            })
            .unwrap_or(no_selection)
        {
            log::debug!("clear pendning mark on carret move");
            replace_pending.call(None);
        }
    });

    _ = use_raf_fn(move |UseRafFnCallbackArgs { delta, .. }| {
        if pending_mark.get_untracked().is_some() {
            set_pending_mark.update(|set_pending_mark| {
                let wd = set_pending_mark.as_mut().unwrap();
                log::debug!("app.rs :: Starting tick_timer for WordMark");
                let ended = wd.tick_timer(delta);
                if ended {
                    log::debug!("app.rs :: tick_timer ended, converting WordMark to permanent");
                    let id = Uuid::new_v4();
                    let permanent = wd.make_permanent(id).unwrap();
                    *set_pending_mark = None;
                    translate_action.dispatch((id, permanent.word(), permanent.context(), None));
                    set_data.update(|set_data| {
                        log::debug!(
                            "app.rs :: Inserting permanent WordMark into data with ID: {:?}",
                            id
                        );
                        _ = set_data.insert(id, permanent);
                    });
                    set_show_ya.update(|d| {
                        log::debug!("app.rs :: Inserting ID into show_ya: {:?}", id);
                        if !d.contains(&id) {
                            d.push(id);
                        }
                    });
                }
            });
        }
    });

    let clear_mouse_out_listener = use_event_listener(use_window(), ev::mouseleave, move |_| {
        log::debug!("clear pendning mark on ev::mouseleave");
        replace_pending.call(None);
    });

    let clear_win_blur_listener = use_event_listener(use_window(), ev::blur, move |_| {
        log::debug!("clear pendning mark on ev::blur");
        replace_pending.call(None);
    });

    on_cleanup(move || {
        clear_mouse_move_listener();
        clear_mouse_out_listener();
        clear_pointer_down_listener();
        clear_pointer_up_listener();
        clear_win_blur_listener();
    });

    let visible_annotations = create_memo(move |_| {
        let data = data.get();

        show_ya
            .get()
            .into_iter()
            .filter_map(|id| data.get(&id).map(|d| (*d, id)))
            .collect::<Vec<_>>()
    });

    let translate_value = translate_action.value();
    create_effect(move |_| {
        if let Some((id, translation)) = translate_value.get() {
            let data = data.get_untracked();
            if let Some(wd) = data.get(&id) {
                wd.annotate(Some(translation));
            } else {
                log::error!("no entry for translation id {id}");
            }
        }
    });

    let close_cb = Callback::new(move |(id, quality): (Uuid, Option<bool>)| {
        set_show_ya.update(|s| {
            _ = s.retain(|v| v != &id);
        });

        if let Some(good) = quality.as_ref() {
            let data = data.get_untracked();
            let wd = data.get(&id).unwrap();
            if !wd.skip_feedback() {
                let word = wd.word();
                let context = wd.context();
                let annotation = wd.annotation().unwrap();

                success_record_action.dispatch((word, context, annotation, *good));
                wd.feedback(true);
            }
        }

        if quality.is_none() || quality == Some(false) {
            set_data.update(|d| {
                let an = d.get(&id).unwrap();
                an.unmount().unwrap();
                _ = d.remove(&id);
            });
        }
    });

    let regenerate_cb = Callback::new(move |id: Uuid| {
        let data = data.get();
        let entry = data.get(&id).unwrap();
        entry.feedback(false);
        let word = entry.word();
        let context = entry.context();
        let annotation = entry.annotation();
        translate_action.dispatch((id, word.clone(), context.clone(), annotation.clone()));
        entry.annotate(None);
        if let Some(annotation) = annotation {
            success_record_action.dispatch((word, context, annotation, false));
        }
    });

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
                }}
            </For>
        </div>
    }
}
