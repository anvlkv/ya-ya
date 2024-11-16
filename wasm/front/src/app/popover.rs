use leptos::*;
use leptos_use::{
    use_element_bounding, use_window_scroll, use_window_size, UseElementBoundingReturn,
    UseWindowSizeReturn,
};
use web_sys::Element;

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
pub fn YaYaPopover(
    #[prop(attrs)] attrs: Vec<(&'static str, Attribute)>,
    #[prop(into)] mark_el: MaybeSignal<Element>,
    #[prop(into)] close_cb: Callback<()>,
    children: ChildrenFn,
) -> impl IntoView {
    let mark_el = Signal::derive(move || mark_el.get());
    let popover_el = create_node_ref::<html::Div>();
    let UseElementBoundingReturn {
        x: mark_x,
        y: mark_y,
        width: mark_width,
        height: mark_height,
        ..
    } = use_element_bounding(mark_el);
    let UseElementBoundingReturn {
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
        let top_space = mark_y.get();
        let bottom_space = win_height.get() - mark_y.get() - mark_height.get();
        let left_space = mark_x.get();
        let right_space = win_width.get() - mark_x.get() - mark_width.get();

        let popover_height = popover_height.get();
        let popover_width = popover_width.get();

        log::debug!(
            r#"
            top_space: {top_space}
            bottom_space: {bottom_space}
            left_space: {left_space}
            right_space: {right_space}
            popover_height: {popover_height}
            popover_width: {popover_width}
            "#
        );

        if let Some(prev) = prev.filter(|side| match side {
            CalloutSide::Top => {
                let half_width = popover_width / 2.0;
                bottom_space >= popover_height
                    && half_width < left_space
                    && half_width < right_space
            }
            CalloutSide::Left => {
                let half_height = popover_height / 2.0;
                right_space >= popover_width
                    && half_height < top_space
                    && half_height < bottom_space
            }
            CalloutSide::Bottom => {
                let half_width = popover_width / 2.0;
                top_space >= popover_height && half_width < left_space && half_width < right_space
            }
            CalloutSide::Right => {
                let half_height = popover_height / 2.0;
                left_space >= popover_width && half_height < top_space && half_height < bottom_space
            }
            CalloutSide::TopLeft => bottom_space >= popover_height && right_space >= popover_width,
            CalloutSide::TopRight => bottom_space >= popover_height && left_space >= popover_width,
            CalloutSide::BottomLeft => top_space >= popover_height && right_space >= popover_width,
            CalloutSide::BottomRight => top_space >= popover_height && left_space >= popover_width,
        }) {
            *prev
        } else {
            let prefer_bottom = top_space < bottom_space;
            let prefer_left = right_space < left_space;

            let prefer_sideways = left_space.max(right_space) > top_space.max(bottom_space);

            let prefer_corner = if prefer_sideways {
                let half_height = popover_height / 2.0;
                top_space < half_height || bottom_space < half_height
            } else {
                let half_width = popover_width / 2.0;
                left_space < half_width || right_space < half_width
            };

            match (prefer_corner, prefer_sideways, prefer_bottom, prefer_left) {
                (true, _, true, true) => CalloutSide::TopRight,
                (true, _, true, false) => CalloutSide::TopLeft,
                (true, _, false, true) => CalloutSide::BottomRight,
                (true, _, false, false) => CalloutSide::BottomLeft,
                (false, true, _, true) => CalloutSide::Right,
                (false, true, _, false) => CalloutSide::Left,
                (false, false, true, _) => CalloutSide::Top,
                (false, false, false, _) => CalloutSide::Bottom,
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

    let on_close = move |_| {
        close_cb.call(());
    };

    view! {
        <div
            {..attrs}
            class=class
            style=pos_style
            aria-live="polite"
            role="dialog"
            node_ref=popover_el
        >
            <div class="ya-ya-popover-inner">
                <button
                    class="ya-ya-close-button"
                    on:click=on_close
                    title="Закрыть"
                >
                    "×"
                </button>
                <div class="ya-ya-content">
                    {children}
                </div>
            </div>
        </div>
    }
}
