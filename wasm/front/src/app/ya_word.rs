use leptos::*;
use leptos_use::{
    use_element_bounding, use_window_size, UseElementBoundingReturn, UseWindowSizeReturn,
};

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
pub fn YaWordPopover(#[prop(into)] word: MaybeSignal<WordPermanentTrigger>) -> impl IntoView {
    let word = Signal::derive(move || word.get());
    let UseElementBoundingReturn {
        x,
        y,
        top,
        right,
        bottom,
        left,
        width,
        height,
        ..
    } = use_element_bounding(Signal::derive(move || word.get().mark));
    let UseWindowSizeReturn {
        width: win_width,
        height: win_height,
    } = use_window_size();

    let side = create_memo(move |_| {
        let mut side = CalloutSide::Top;

        let available_top = top.get();
        let available_bottom = win_height.get() - bottom.get();
        let available_left = left.get();
        let available_right = win_width.get() - right.get();

        if available_top >= height.get() {
            side = CalloutSide::Top;
        } else if available_bottom >= height.get() {
            side = CalloutSide::Bottom;
        } else if available_left >= width.get() {
            side = CalloutSide::Left;
        } else if available_right >= width.get() {
            side = CalloutSide::Right;
        } else {
            if available_top >= height.get() && available_left >= width.get() {
                side = CalloutSide::TopLeft;
            } else if available_top >= height.get() && available_right >= width.get() {
                side = CalloutSide::TopRight;
            } else if available_bottom >= height.get() && available_left >= width.get() {
                side = CalloutSide::BottomLeft;
            } else if available_bottom >= height.get() && available_right >= width.get() {
                side = CalloutSide::BottomRight;
            }
        }

        side
    });

    let pos_style = create_memo(move |_| {
        let side_value = side.get();
        let x_value = x.get();
        let y_value = y.get();
        let width_value = width.get();
        let height_value = height.get();

        match side_value {
            CalloutSide::Top => format!("top: {}px; left: {}px;", y_value - height_value, x_value),
            CalloutSide::Bottom => {
                format!("top: {}px; left: {}px;", y_value + height_value, x_value)
            }
            CalloutSide::Left => format!("top: {}px; left: {}px;", y_value, x_value - width_value),
            CalloutSide::Right => format!("top: {}px; left: {}px;", y_value, x_value + width_value),
            CalloutSide::TopLeft => format!(
                "top: {}px; left: {}px;",
                y_value - height_value,
                x_value - width_value
            ),
            CalloutSide::TopRight => format!(
                "top: {}px; left: {}px;",
                y_value - height_value,
                x_value + width_value
            ),
            CalloutSide::BottomLeft => format!(
                "top: {}px; left: {}px;",
                y_value + height_value,
                x_value - width_value
            ),
            CalloutSide::BottomRight => format!(
                "top: {}px; left: {}px;",
                y_value + height_value,
                x_value + width_value
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

    view! {
        <div
            class=class
            style=pos_style
        >
            {move || word.get().mark.text_content()}
        </div>
    }
}
