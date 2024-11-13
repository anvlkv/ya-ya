use leptos::*;
use leptos_use::{
    use_element_bounding, use_window_scroll, use_window_size, UseElementBoundingReturn,
    UseWindowSizeReturn,
};
use uuid::Uuid;

use super::loading::Loading;
use super::word::WordPermanentTrigger;

#[derive(Debug, Clone, Copy, PartialEq)]
enum CalloutSide {
    Top,
    Left,
    Bottom,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl CalloutSide {
    fn class_name(&self) -> &'static str {
        match self {
            CalloutSide::Top => "ya-ya-popover-side-top",
            CalloutSide::Left => "ya-ya-popover-side-left",
            CalloutSide::Bottom => "ya-ya-popover-side-bottom",
            CalloutSide::Right => "ya-ya-popover-side-right",
            CalloutSide::TopLeft => "ya-ya-popover-side-top-left",
            CalloutSide::TopRight => "ya-ya-popover-side-top-right",
            CalloutSide::BottomLeft => "ya-ya-popover-side-bottom-left",
            CalloutSide::BottomRight => "ya-ya-popover-side-bottom-right",
        }
    }

    fn animate(&self) -> &'static str {
        match self {
            CalloutSide::Top => "animate__fadeInBottom",
            CalloutSide::Left => "animate__fadeInRight",
            CalloutSide::Bottom => "animate__fadeInTop",
            CalloutSide::Right => "animate__fadeInLeft",
            CalloutSide::TopLeft => "animate__fadeInBottomRight",
            CalloutSide::TopRight => "animate__fadeInBottomLeft",
            CalloutSide::BottomLeft => "animate__fadeInTopRight",
            CalloutSide::BottomRight => "animate__fadeInTopLeft",
        }
    }
}

#[component]
pub fn YaWordPopover(
    #[prop(into)] word: MaybeSignal<WordPermanentTrigger>,
    #[prop(into)] close_cb: Callback<(Uuid, Option<bool>)>,
    #[prop(into)] regenerate_cb: Callback<Uuid>,
) -> impl IntoView {
    let word = Signal::derive(move || word.get());
    let mark_el = Signal::derive(move || word.get().mark);
    let popover_el = create_node_ref::<html::Div>();
    let UseElementBoundingReturn {
        x: mark_x,
        y: mark_y,
        top: mark_top,
        right: mark_right,
        bottom: mark_bottom,
        left: mark_left,
        width: mark_width,
        height: mark_height,
        ..
    } = use_element_bounding(mark_el);
    let UseElementBoundingReturn {
        x: popover_x,
        y: popover_y,
        top: popover_top,
        right: popover_right,
        bottom: popover_bottom,
        left: popover_left,
        width: popover_width,
        height: popover_height,
        ..
    } = use_element_bounding(popover_el);

    let (scroll_x, scroll_y) = use_window_scroll();

    let UseWindowSizeReturn {
        width: win_width,
        height: win_height,
    } = use_window_size();

    let side = create_memo(move |prev| {
        let available_top = mark_top.get();
        let available_bottom = mark_bottom.get();
        let available_left = mark_left.get();
        let available_right = mark_right.get();

        let required_height = popover_height.get();
        let required_width = popover_width.get();

        if let Some(prev) = prev.filter(|side| match side {
            CalloutSide::Top => {
                let h_w = required_width / 2.0;
                available_bottom >= required_height && h_w < available_left && h_w < available_right
            }
            CalloutSide::Left => {
                let h_h = required_height / 2.0;
                available_right >= required_width && h_h < available_top && h_h < available_bottom
            }
            CalloutSide::Bottom => {
                let h_w = required_width / 2.0;
                available_top >= required_height && h_w < available_left && h_w < available_right
            }
            CalloutSide::Right => {
                let h_h = required_height / 2.0;
                available_left >= required_width && h_h < available_top && h_h < available_bottom
            }
            CalloutSide::TopLeft => {
                available_bottom >= required_height && available_right >= required_width
            }
            CalloutSide::TopRight => {
                available_bottom >= required_height && available_left >= required_width
            }
            CalloutSide::BottomLeft => {
                available_top >= required_height && available_right >= required_width
            }
            CalloutSide::BottomRight => {
                available_top >= required_height && available_left >= required_width
            }
        }) {
            *prev
        } else {
            let prefer_bottom = available_top < available_bottom;
            let prefer_left = available_right > available_left;

            let prefer_sideways =
                available_left.max(available_right) > available_top.max(available_bottom);

            let prefer_corner = if prefer_sideways {
                let half = required_height / 2.0;
                available_top < half || available_bottom < half
            } else {
                let half = required_width / 2.0;
                available_left < half || available_right < half
            };

            if prefer_corner {
                if prefer_bottom {
                    if prefer_left {
                        CalloutSide::TopRight
                    } else {
                        CalloutSide::TopLeft
                    }
                } else {
                    if prefer_left {
                        CalloutSide::BottomRight
                    } else {
                        CalloutSide::BottomLeft
                    }
                }
            } else if prefer_sideways {
                if prefer_left {
                    CalloutSide::Right
                } else {
                    CalloutSide::Left
                }
            } else if prefer_bottom {
                CalloutSide::Top
            } else {
                CalloutSide::Bottom
            }
        }
    });

    let pos_style = create_memo(move |_| {
        let side_value = side.get();
        let mark_x_value = mark_x.get();
        let mark_y_value = mark_y.get();
        let mark_height_value = mark_height.get();
        let mark_width_value = mark_width.get();
        let popover_width_value = popover_width.get();
        let popover_height_value = popover_height.get();
        let win_height_value = win_height.get();
        let win_width_value = win_width.get();
        let scroll_x_value = scroll_x.get();
        let scroll_y_value = scroll_y.get();

        match side_value {
            CalloutSide::Top => format!(
                "top: {}px; left: {}px; max-height: {}px;",
                mark_y_value + mark_height_value + scroll_y_value,
                mark_x_value + mark_width_value / 2.0 + scroll_x_value - popover_width_value / 2.0,
                win_height_value - mark_y_value - mark_height_value - scroll_y_value
            ),
            CalloutSide::Left => format!(
                "top: {}px; left: {}px; max-width: {}px;",
                mark_y_value + mark_height_value / 2.0 + scroll_y_value
                    - popover_height_value / 2.0,
                mark_x_value + mark_width_value + scroll_x_value,
                win_width_value - mark_x_value - mark_width_value - scroll_x_value
            ),
            CalloutSide::Bottom => format!(
                "top: {}px; left: {}px; max-height: {}px;",
                mark_y_value - popover_height_value + scroll_y_value,
                mark_x_value + mark_width_value / 2.0 + scroll_x_value - popover_width_value / 2.0,
                mark_y_value - scroll_y_value
            ),
            CalloutSide::Right => format!(
                "top: {}px; left: {}px; max-width: {}px;",
                mark_y_value + mark_height_value / 2.0 + scroll_y_value
                    - popover_height_value / 2.0,
                mark_x_value - popover_width_value + scroll_x_value,
                mark_x_value - scroll_x_value
            ),
            CalloutSide::TopLeft => format!(
                "top: {}px; left: {}px; max-height: {}px; max-width: {}px;",
                mark_y_value + mark_height_value + scroll_y_value,
                mark_x_value + mark_width_value + scroll_x_value,
                win_height_value - mark_y_value - mark_height_value - scroll_y_value,
                win_width_value - mark_x_value - mark_width_value - scroll_x_value
            ),
            CalloutSide::TopRight => format!(
                "top: {}px; left: {}px; max-height: {}px; max-width: {}px;",
                mark_y_value + mark_height_value + scroll_y_value,
                mark_x_value - popover_width_value + scroll_x_value,
                win_height_value - mark_y_value - mark_height_value - scroll_y_value,
                mark_x_value - scroll_x_value
            ),
            CalloutSide::BottomLeft => format!(
                "top: {}px; left: {}px; max-height: {}px; max-width: {}px;",
                mark_y_value - popover_height_value + scroll_y_value,
                mark_x_value + mark_width_value + scroll_x_value,
                mark_y_value - scroll_y_value,
                win_width_value - mark_x_value - mark_width_value - scroll_x_value
            ),
            CalloutSide::BottomRight => format!(
                "top: {}px; left: {}px; max-height: {}px; max-width: {}px;",
                mark_y_value - popover_height_value + scroll_y_value,
                mark_x_value - popover_width_value + scroll_x_value,
                mark_y_value - scroll_y_value,
                mark_x_value - scroll_x_value
            ),
        }
    });

    let class = Signal::derive(move || {
        format!(
            "ya-ya-popover animate__animated {} {}",
            side.get().class_name(),
            side.get().animate()
        )
    });

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
        <div
            class=class
            style=pos_style
            aria-live="polite"
            role="dialog"
            aria-describedby=move || format!("mark-{}", word.get().id)
            node_ref=popover_el
        >
            <button
                class="ya-ya-close-button"
                on:click=on_close
                title="Закрыть"
            >
                "×"
            </button>
            <div class="ya-ya-content">
                <Show
                    when={move ||content.get().is_some()}
                    fallback={move || view! {
                        <h3>{move || word.get().mark.text_content()}</h3>
                        <Loading/>
                    }}
                >
                    <div style:display="contents" inner_html=content/>
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
            </div>
        </div>
    }
}
