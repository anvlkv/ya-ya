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
                .map(|v| v.as_string())
                .ok()
                .flatten()
        },
    );

    view! {
        <style inner_html={ANIMATE_STYLE}/>
        <style inner_html={STYLE}/>

        <div class="ya-ya-action">
            <h1>Пояснения</h1>
            {
                match selection_rsc.get().flatten() {
                    Some(text) => view!{
                        <AnnotateText text/>
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
pub fn AnnotateText(#[prop(into)] text: MaybeSignal<String>) -> impl IntoView {
    view! {
        <blockquote>
            {text}
        </blockquote>
    }
}
