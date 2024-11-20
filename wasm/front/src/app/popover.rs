use leptos::*;
use leptos_use::{
    use_element_bounding, use_window, use_window_scroll, use_window_size, UseElementBoundingReturn,
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
    #[prop(into)] elements: MaybeSignal<(Element, Element, Element)>,
    #[prop(into)] close_cb: Callback<()>,
    children: ChildrenFn,
) -> impl IntoView {
    let elements = Signal::derive(move || elements.get());
    let before_el = Signal::derive(move || elements.get().0);
    let mark_el = Signal::derive(move || elements.get().1);
    let after_el = Signal::derive(move || elements.get().2);

    let popover_el = create_node_ref::<html::Div>();

    let UseElementBoundingReturn {
        y: mark_y,
        height: mark_height,
        ..
    } = use_element_bounding(mark_el);

    let UseElementBoundingReturn {
        x: before_x,
        y: before_y,
        ..
    } = use_element_bounding(before_el);

    let UseElementBoundingReturn {
        x: after_x,
        y: after_y,
        ..
    } = use_element_bounding(after_el);

    let is_multiline = Signal::derive(move || before_y.get() != after_y.get());

    let (scroll_x, scroll_y) = use_window_scroll();

    let UseWindowSizeReturn {
        width: win_width,
        height: win_height,
    } = use_window_size();

    let space_above = move || mark_y.get() - scroll_y.get();
    let space_left_before = move || before_x.get() - scroll_x.get();

    let space_below =
        move || win_height.get() - (mark_y.get() + mark_height.get()) + scroll_y.get();
    let space_right_after = move || win_width.get() - (after_x.get() + scroll_x.get());

    let side = create_memo(move |_| {
        let space_above = space_above();
        let space_below = space_below();
        let space_right_after = space_right_after();
        let space_left_before = space_left_before();
        let win_width = win_width.get();
        let win_height = win_height.get();

        let prefer_top = space_above > space_below;
        let prefer_left = space_left_before > space_right_after;

        let prefer_sideways = win_width > win_height || is_multiline.get();
        let prefer_corner = if prefer_sideways {
            space_above.max(space_below) - space_below.min(space_above) > win_height * 0.5
        } else {
            space_left_before.max(space_right_after) - space_right_after.min(space_left_before)
                > win_width * 0.5
        };

        let side = match (prefer_corner, prefer_sideways, prefer_left, prefer_top) {
            (true, _, true, true) => CalloutSide::BottomRight,
            (true, _, false, true) => CalloutSide::BottomLeft,
            (true, _, true, false) => CalloutSide::TopRight,
            (true, _, false, false) => CalloutSide::TopLeft,
            (false, false, _, true) => CalloutSide::Bottom,
            (false, false, _, false) => CalloutSide::Top,
            (false, true, true, _) => CalloutSide::Right,
            (false, true, false, _) => CalloutSide::Left,
        };

        log::debug!(
            r#"
side: {side:?}
space_above: {space_above}
space_below: {space_below}
space_right_after: {space_right_after}
space_left_before: {space_left_before}
win_width: {win_width}
win_height: {win_height}
            "#
        );

        side
    });

    let padding = move || {
        let value = popover_el
            .get()
            .zip(use_window().as_ref())
            .and_then(|(el, win)| win.get_computed_style(&el).ok().flatten())
            .and_then(|s| s.get_property_value("padding-left").ok())
            .and_then(|s| s.trim_end_matches("px").parse::<f64>().ok())
            .unwrap_or_default();

        log::debug!("padding: {value}");

        value
    };

    let content_max_width = move || {
        let space_right_after = space_right_after();
        let space_left_before = space_left_before();
        let side = side.get();
        let padding = padding();

        match side {
            CalloutSide::Top | CalloutSide::Bottom => "auto".to_string(),
            CalloutSide::TopLeft | CalloutSide::BottomLeft | CalloutSide::Left => {
                format!("{}px", space_right_after - padding * 3.0)
            }
            CalloutSide::TopRight | CalloutSide::BottomRight | CalloutSide::Right => {
                format!("{}px", space_left_before - padding * 3.0)
            }
        }
    };

    let content_max_height = move || {
        let space_above = space_above();
        let space_below = space_below();
        let side = side.get();
        let padding = padding();

        match side {
            CalloutSide::Left | CalloutSide::Right => "auto".to_string(),
            CalloutSide::Top | CalloutSide::TopLeft | CalloutSide::TopRight => {
                format!("{}px", space_below - padding * 3.0)
            }
            CalloutSide::Bottom | CalloutSide::BottomLeft | CalloutSide::BottomRight => {
                format!("{}px", space_above - padding * 3.0)
            }
        }
    };

    let UseElementBoundingReturn {
        width: popover_width,
        height: popover_height,
        ..
    } = use_element_bounding(popover_el);

    let pos_style = create_memo(move |_| {
        let side = side.get();

        let mark_y = mark_y.get();
        let mark_height = mark_height.get();
        let popover_height = popover_height.get();
        let scroll_y = scroll_y.get();

        let after_x = after_x.get();
        let before_x = before_x.get();
        let popover_width = popover_width.get();
        let scroll_x = scroll_x.get();

        let top = match side {
            CalloutSide::Top | CalloutSide::TopLeft | CalloutSide::TopRight => {
                mark_y + mark_height + scroll_y
            }
            CalloutSide::Bottom | CalloutSide::BottomLeft | CalloutSide::BottomRight => {
                mark_y - popover_height + scroll_y
            }
            CalloutSide::Left | CalloutSide::Right => {
                mark_y + mark_height / 2.0 + scroll_y - popover_height / 2.0
            }
        };

        let left = match side {
            CalloutSide::Left | CalloutSide::TopLeft | CalloutSide::BottomLeft => {
                after_x + scroll_x
            }
            CalloutSide::Right | CalloutSide::TopRight | CalloutSide::BottomRight => {
                before_x - popover_width + scroll_x
            }
            CalloutSide::Top | CalloutSide::Bottom => {
                before_x + (after_x - before_x) / 2.0 - popover_width / 2.0 + scroll_x
            }
        };

        format!("top: {}px; left: {}px;", top, left)
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
                <div class="ya-ya-content"
                    style:max-width=content_max_width
                    style:max-height=content_max_height
                >
                    {children}
                </div>
            </div>
        </div>
    }
}
