use leptos::*;

use super::word::WordPermanentTrigger;

#[component]
pub fn YaWordPopover(#[prop(into)] word: MaybeSignal<WordPermanentTrigger>) -> impl IntoView {
    format!("wrd: {word:#?}")
}
