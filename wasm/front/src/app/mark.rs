use leptos::*;
use uuid::Uuid;
use wasm_bindgen::JsValue;

use super::{
    annotation::Annotation,
    error::YaYaError,
    word::{WordMark, WordPermanentTrigger},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermanentTrigger {
    Word(RwSignal<WordPermanentTrigger>),
}

impl PermanentTrigger {
    pub fn unmount(&self) -> Result<(), wasm_bindgen::JsValue> {
        match self {
            Self::Word(wd) => wd.get_untracked().unmount(),
        }
    }

    pub fn content(&self) -> String {
        match self {
            Self::Word(wd) => wd.get_untracked().word(),
        }
    }

    pub fn context(&self) -> String {
        match self {
            Self::Word(wd) => wd.get_untracked().context(),
        }
    }

    pub fn annotate(&self, value: Option<Result<Annotation, YaYaError>>) {
        match self {
            Self::Word(wd) => wd.update(|wd| wd.annotation = value),
        }
    }

    pub fn annotation(&self) -> Option<Result<Annotation, YaYaError>> {
        match self {
            Self::Word(wd) => wd.get_untracked().annotation.clone(),
        }
    }

    pub fn feedback(&self, val: bool) {
        match self {
            Self::Word(wd) => wd.update_untracked(|wd| wd.feedback = val),
        }
    }

    pub fn skip_feedback(&self) -> bool {
        match self {
            Self::Word(wd) => wd.get_untracked().feedback,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingMark {
    Word(WordMark),
}

impl PendingMark {
    pub fn unmount(&self) -> Result<(), wasm_bindgen::JsValue> {
        match self {
            Self::Word(wd) => wd.unmount(),
        }
    }

    pub fn tick_timer(&mut self, delta: f64) -> bool {
        match self {
            Self::Word(wd) => wd.tick_timer(delta),
        }
    }

    pub fn make_permanent(&self, id: Uuid) -> Result<PermanentTrigger, JsValue> {
        match self {
            Self::Word(wd) => WordPermanentTrigger::make_permanent(&wd, id)
                .map(|d| PermanentTrigger::Word(RwSignal::new(d))),
        }
    }

    pub fn is_same(&self, node: web_sys::Node, pos: u32) -> bool {
        match self {
            Self::Word(wd) => wd.is_same(node, pos),
        }
    }
}
