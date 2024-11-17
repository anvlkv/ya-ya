use leptos::*;
use uuid::Uuid;

use super::loading::Loading;
use super::popover::YaYaPopover;
use super::word::WordPermanentTrigger;

#[component]
pub fn YaWordPopover(
    #[prop(into)] word: MaybeSignal<WordPermanentTrigger>,
    #[prop(into)] close_cb: Callback<(Uuid, Option<bool>)>,
    #[prop(into)] regenerate_cb: Callback<Uuid>,
) -> impl IntoView {
    let word = Signal::derive(move || word.get());
    let mark_el = Signal::derive(move || word.get().mark);

    let content = create_memo(move |_| {
        word.get()
            .annotation
            .as_ref()
            .map(|a| markdown::to_html(a.as_str()))
    });

    let on_close = move |_| {
        close_cb.call((
            word.get().id,
            word.get().annotation.map(|_| Some(false)).unwrap_or(None),
        ));
    };

    view! {
        <YaYaPopover
            attr:aria-labelledby=move || format!("mark-{}", word.get().id)
            attr:aria-describedby=move || format!("mark-{}", word.get().id)
            mark_el=mark_el
            close_cb=on_close
        >
            <Show
                when={move ||content.get().is_some()}
                fallback={move || view! {
                    <h3>{move || word.get().mark.text_content()}</h3>
                    <Loading/>
                }}
            >
                <div style:display="contents" inner_html=content />
                <div class="ya-ya-water-mark">
                    <hr/>
                    <p>Ответ создан языковой моделью и может содержать ошибки.</p>
                    <hr/>
                </div>
                <div class="ya-ya-footer">
                    <button
                        class="ya-ya-button"
                        on:click=move |_| {
                            regenerate_cb.call(word.get().id);
                        }
                    >
                        "↺ Не понятно"
                    </button>
                    <button
                        class="ya-ya-button-cta"
                        on:click=move |_| {
                            close_cb.call((word.get().id, Some(true)));
                        }
                    >
                        "✔︎ Ясно"
                    </button>
                </div>
            </Show>
        </YaYaPopover>
    }
}
