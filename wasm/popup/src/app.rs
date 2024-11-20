use common::{
    annotation::{annotate_text, success_record},
    error::*,
    loading::Loading,
};
use leptos::*;

use super::send_message;

const STYLE: &str = include_str!("../../../style.css");
const ANIMATE_STYLE: &str = include_str!("../../../node_modules/animate.css/animate.min.css");

#[component]
pub fn App() -> impl IntoView {
    let selection_rsc = create_local_resource(
        || (),
        |_| async {
            let msg = js_sys::Object::new();
            js_sys::Reflect::set(&msg, &"action".into(), &"getSelectedText".into()).unwrap();

            let val = send_message(msg.into()).await;

            js_sys::Reflect::get(&val, &"selectedText".into())
                .ok()
                .zip(js_sys::Reflect::get(&val, &"origin".into()).ok())
                .map(|(text, origin)| text.as_string().zip(origin.as_string()))
                .flatten()
                .filter(|(t, _)| !t.is_empty())
        },
    );

    view! {
        <style inner_html={ANIMATE_STYLE}/>
        <style inner_html={STYLE}/>

        <div class="ya-ya-action">
            <h1>Пояснения</h1>
            {
                move || match selection_rsc.get().flatten() {
                    Some((text, origin)) => view!{
                        <AnnotateText text origin/>
                    }.into_view(),
                    None=> view!{
                        <p>
                            "Чтобы получить пояснение выделите текст на странице и нажмите кнопку ЯЯ."
                        </p>
                    }.into_view()
                }
            }
        </div>
    }
}

#[component]
pub fn AnnotateText(
    #[prop(into)] text: MaybeSignal<String>,
    #[prop(into)] origin: MaybeSignal<String>,
) -> impl IntoView {
    let (previous_annotation, set_previous_annotation) = create_signal(Option::<String>::None);
    let text = Signal::derive(move || text.get());
    let annotation = create_local_resource(
        move || (text.get(), origin.get(), previous_annotation.get()),
        |(text, origin, previous)| async move { annotate_text(text, origin, previous).await },
    );
    let success_action = create_action(|(id, quality): &(usize, bool)| {
        let id = *id;
        let quality = *quality;
        async move {
            _ = success_record(id, quality).await.unwrap();
            if quality {
                window().close().unwrap();
            }
        }
    });
    let annotation_cb = Callback::new(move |quality: bool| {
        let ann = annotation.get().unwrap().unwrap();

        success_action.dispatch((ann.id, quality));

        if !quality {
            set_previous_annotation.set(Some(ann.annotation));
        }
    });

    let disabled_btns =
        Signal::derive(move || success_action.pending().get() || annotation.loading().get());

    let annotation_view = move || {
        if let Some(annotation) = annotation.get() {
            let text = annotation?.annotation;
            let html_content = markdown::to_html(&text);
            let html_content = html_content
                .split(&['.', '?', '!', ':'])
                .map(|s| s.trim())
                .collect::<Vec<&str>>()
                .join(".\n");

            Result::<View, YaYaError>::Ok(
                view! {
                    <div>
                        <pre class="ya-ya-pre" inner_html=html_content/>
                    </div>
                    <div class="ya-ya-water-mark">
                        <hr/>
                        <p>Ответ создан языковой моделью и может содержать ошибки.</p>
                        <hr/>
                    </div>
                    <div class="ya-ya-footer">
                        <button
                            class="ya-ya-button"
                            on:click=move |_| {
                                annotation_cb.call(false);
                            }
                            disabled=disabled_btns
                        >
                            "↺ Не понятно"
                        </button>
                        <button
                            class="ya-ya-button-cta"
                            on:click=move |_| {
                                annotation_cb.call(true);
                            }
                            disabled=disabled_btns
                        >
                            "✔︎ Ясно"
                        </button>
                    </div>
                }
                .into_view(),
            )
        } else {
            Result::<View, YaYaError>::Ok(().into_view())
        }
    };

    view! {
        <div class="ya-ya-content">
            <blockquote class="ya-ya-text-original">
                {text}
            </blockquote>
            <Suspense fallback=Loading>
                <ErrorBoundary fallback=move |errors| view!{
                    <ErrorView errors=errors on_retry=move |_| {
                        annotation.refetch();
                    }/>
                }>
                    {annotation_view}
                </ErrorBoundary>
            </Suspense>
        </div>
    }
}
