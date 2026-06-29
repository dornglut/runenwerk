//! File: domain/ui/ui_input/src/facts.rs
//! Crate: ui_input

use crate::{
    FocusChange, FocusDirection, FocusTargetId, Key, KeyState, Modifiers, PointerButton,
    PointerDelta, PointerEventKind, PointerPacket, PointerPosition, SemanticActionEvent,
};

#[derive(Debug, Clone, PartialEq)]
pub enum NormalizedInputFact {
    Pointer(PointerInputFact),
    Keyboard(KeyboardInputFact),
    Focus(FocusInputFact),
    Semantic(SemanticInputFact),
    TextIntent(TextIntentFact),
}

impl NormalizedInputFact {
    pub const fn kind(&self) -> NormalizedInputFactKind {
        match self {
            Self::Pointer(_) => NormalizedInputFactKind::Pointer,
            Self::Keyboard(_) => NormalizedInputFactKind::Keyboard,
            Self::Focus(_) => NormalizedInputFactKind::Focus,
            Self::Semantic(_) => NormalizedInputFactKind::Semantic,
            Self::TextIntent(_) => NormalizedInputFactKind::TextIntent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NormalizedInputFactKind {
    Pointer,
    Keyboard,
    Focus,
    Semantic,
    TextIntent,
}

impl NormalizedInputFactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pointer => "pointer",
            Self::Keyboard => "keyboard",
            Self::Focus => "focus",
            Self::Semantic => "semantic",
            Self::TextIntent => "text-intent",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointerInputFact {
    pub kind: PointerEventKind,
    pub position: PointerPosition,
    pub delta: PointerDelta,
    pub button: Option<PointerButton>,
    pub modifiers: Modifiers,
    pub click_count: u8,
    pub packet: PointerPacket,
}

impl PointerInputFact {
    pub fn new(kind: PointerEventKind, position: PointerPosition) -> Self {
        Self {
            kind,
            position,
            delta: PointerDelta::ZERO,
            button: None,
            modifiers: Modifiers::default(),
            click_count: 0,
            packet: PointerPacket::default(),
        }
    }

    pub fn with_button(mut self, button: PointerButton) -> Self {
        self.button = Some(button);
        self
    }

    pub fn with_click_count(mut self, click_count: u8) -> Self {
        self.click_count = click_count;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardInputFact {
    pub key: Key,
    pub state: KeyState,
    pub modifiers: Modifiers,
}

impl KeyboardInputFact {
    pub fn new(key: Key, state: KeyState) -> Self {
        Self {
            key,
            state,
            modifiers: Modifiers::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusInputFact {
    pub change: FocusChange,
    pub direction: Option<FocusDirection>,
    pub focus_visible: bool,
}

impl FocusInputFact {
    pub const fn change(change: FocusChange) -> Self {
        Self {
            change,
            direction: None,
            focus_visible: false,
        }
    }

    pub const fn traversal(direction: FocusDirection) -> Self {
        Self {
            change: FocusChange::None,
            direction: Some(direction),
            focus_visible: true,
        }
    }

    pub const fn target(target: FocusTargetId) -> Self {
        Self {
            change: FocusChange::Set(target),
            direction: None,
            focus_visible: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticInputFact {
    pub event: SemanticActionEvent,
}

impl SemanticInputFact {
    pub const fn new(event: SemanticActionEvent) -> Self {
        Self { event }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextIntentFact {
    pub kind: TextIntentKind,
    pub text: String,
}

impl TextIntentFact {
    pub fn insert_text(text: impl Into<String>) -> Self {
        Self {
            kind: TextIntentKind::InsertText,
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextIntentKind {
    InsertText,
    Commit,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedInputSample {
    pub sample_id: String,
    pub facts: Vec<NormalizedInputFact>,
}

impl NormalizedInputSample {
    pub fn new(sample_id: impl Into<String>) -> Self {
        Self {
            sample_id: sample_id.into(),
            facts: Vec::new(),
        }
    }

    pub fn with_fact(mut self, fact: NormalizedInputFact) -> Self {
        self.facts.push(fact);
        self
    }

    pub fn fact_kinds(&self) -> Vec<&'static str> {
        self.facts.iter().map(|fact| fact.kind().as_str()).collect()
    }
}
