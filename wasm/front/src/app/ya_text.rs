use leptos::*;
use leptos_use::{
    use_element_bounding, use_window_scroll, use_window_size, UseElementBoundingReturn,
    UseWindowSizeReturn,
};
use uuid::Uuid;
use web_sys::Element;

use super::loading::Loading;
use super::popover::YaYaPopover;
use super::text::TextPermanentTrigger;

#[component]
pub fn YaTextPopover(
    #[prop(into)] text: MaybeSignal<TextPermanentTrigger>,
    #[prop(into)] close_cb: Callback<(Uuid, Option<bool>)>,
    #[prop(into)] regenerate_cb: Callback<Uuid>,
) -> impl IntoView {
    let text = Signal::derive(move || text.get());
    let mark_els = Signal::derive(move || text.get().marks);
    let (_, scroll_y) = use_window_scroll();

    let mark_el = create_memo(move |_| {
        let els = mark_els.get();
        let scroll_y_value = scroll_y.get();
        let mut closest_mark: Option<Element> = els.first().cloned();
        let mut closest_distance = std::f64::INFINITY;

        for mark in els.into_iter() {
            let rect = mark.get_bounding_client_rect();
            let y_value = rect.y();
            let distance = (y_value - scroll_y_value).abs();

            if distance < closest_distance {
                closest_distance = distance;
                closest_mark = Some(mark.clone());
            }
        }

        closest_mark.expect("mark element")
    });

    let content = create_memo(move |_| {
        text.get()
            .annotation
            .as_ref()
            .map(|a| markdown::to_html(a.as_str()))
    });

    let on_close = move |_| {
        close_cb.call((
            text.get().id,
            text.get().annotation.map(|_| Some(false)).unwrap_or(None),
        ));
    };

    view! {
        <YaYaPopover
            attr:aria-labelledby=move || format!("mark-{}", text.get().id)
            attr:aria-describedby=move || format!("mark-{}", text.get().id)
            mark_el=mark_el
            close_cb=on_close
        >
            <Show
                when={move ||content.get().is_some()}
                fallback={move || view! {
                    <Loading/>
                }}
            >
                <div style:display="contents" inner_html=content/>
                <div class="ya-ya-footer">
                    <button
                        class="ya-ya-button"
                        on:click=move |_| {
                            regenerate_cb.call(text.get().id);
                        }
                    >
                        "↺ Не понятно"
                    </button>
                    <button
                        class="ya-ya-button-cta"
                        on:click=move |_| {
                            close_cb.call((text.get().id, Some(true)));
                        }
                    >
                        "✔︎ Ясно"
                    </button>
                </div>
            </Show>
        </YaYaPopover>
    }
}
