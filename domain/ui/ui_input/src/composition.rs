//! Normalized text composition facts.

use crate::selection::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCompositionFact {
    pub target_id: Option<String>,
    pub kind: TextCompositionKind,
    pub text: String,
    pub range: Option<TextRange>,
}

impl TextCompositionFact {
    pub fn start(text: impl Into<String>) -> Self {
        Self::new(TextCompositionKind::Start, text)
    }

    pub fn update(text: impl Into<String>) -> Self {
        Self::new(TextCompositionKind::Update, text)
    }

    pub fn accept(text: impl Into<String>) -> Self {
        Self::new(TextCompositionKind::Accept, text)
    }

    pub fn cancel() -> Self {
        Self::new(TextCompositionKind::Cancel, "")
    }

    pub fn new(kind: TextCompositionKind, text: impl Into<String>) -> Self {
        Self {
            target_id: None,
            kind,
            text: text.into(),
            range: None,
        }
    }

    pub fn with_target(mut self, target_id: impl Into<String>) -> Self {
        self.target_id = Some(target_id.into());
        self
    }

    pub fn with_range(mut self, range: TextRange) -> Self {
        self.range = Some(range);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextCompositionKind {
    Start,
    Update,
    Accept,
    Cancel,
}

impl TextCompositionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "composition-start",
            Self::Update => "composition-update",
            Self::Accept => "composition-accept",
            Self::Cancel => "composition-cancel",
        }
    }
}
