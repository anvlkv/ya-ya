use common::error::{ErrorView, YaYaError};
use common::loading::Loading;
use leptos::*;
use uuid::Uuid;

use super::popover::YaYaPopover;
use super::word::WordPermanentTrigger;

#[component]
pub fn YaWordPopover(
    #[prop(into)] word: MaybeSignal<WordPermanentTrigger>,
    #[prop(into)] close_cb: Callback<(Uuid, Option<bool>)>,
    #[prop(into)] regenerate_cb: Callback<Uuid>,
) -> impl IntoView {
    let word = Signal::derive(move || word.get());
    let elemetns = Signal::derive(move || {
        let wd = word.get();
        (wd.anchors.0, wd.mark, wd.anchors.1)
    });

    let content = create_memo(move |_| {
        word.get().annotation.map(|res| {
            res.map(|a| {
                let html_content = markdown::to_html(&a.annotation);
                html_content
                    .split(&['.', '?', '!', ':'])
                    .map(|s| s.trim())
                    .collect::<Vec<&str>>()
                    .join(".\n")
            })
        })
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
            elements=elemetns
            close_cb=on_close
        >
            <Show
                when={move ||content.get().is_some()}
                fallback={move || view! {
                    <pre class="ya-ya-pre">
                        <h3>{move || word.get().mark.text_content()}</h3>
                        <Loading/>
                    </pre>
                }}
            >
                <ErrorBoundary fallback=move |errors| view!{
                    <ErrorView errors=errors on_retry=move |_| {
                        regenerate_cb.call(word.get().id);
                    }/>
                }>
                    {move || {
                        let content = content.get().ok_or(YaYaError::IntegrationError)??;
                        let id = word.get().id;
                        Result::<View, YaYaError>::Ok(view!{
                            <pre class="ya-ya-pre" inner_html=content />
                            <div class="ya-ya-water-mark">
                                <hr/>
                                <p>Ответ создан языковой моделью и может содержать ошибки.</p>
                                <hr/>
                            </div>
                            <div class="ya-ya-footer">
                                <button
                                    class="ya-ya-button"
                                    on:click=move |_| {
                                        regenerate_cb.call(id);
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
                        }.into_view())
                    }}
                </ErrorBoundary>
            </Show>
        </YaYaPopover>
    }
}
