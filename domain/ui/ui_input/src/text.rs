//! Normalized editable-text input facts.
//!
//! These facts describe text-editing intent at the input seam. They do not
//! decide whether a control is editable or own app document changes.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEditFact {
    pub target_id: Option<String>,
    pub intent: TextEditIntent,
    pub text: String,
    pub host_owned_source: Option<String>,
}

impl TextEditFact {
    pub fn insert_text(text: impl Into<String>) -> Self {
        Self {
            target_id: None,
            intent: TextEditIntent::InsertText,
            text: text.into(),
            host_owned_source: None,
        }
    }

    pub fn source_insert(source: impl Into<String>) -> Self {
        Self {
            target_id: None,
            intent: TextEditIntent::SourceInsert,
            text: String::new(),
            host_owned_source: Some(source.into()),
        }
    }

    pub fn with_target(mut self, target_id: impl Into<String>) -> Self {
        self.target_id = Some(target_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TextEditIntent {
    InsertText,
    DeleteBackward,
    DeleteForward,
    ReplaceSelection,
    MoveCaret,
    ExtendSelection,
    Submit,
    Cancel,
    SourceInsert,
    Copy,
    Cut,
}

impl TextEditIntent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InsertText => "insert-text",
            Self::DeleteBackward => "delete-backward",
            Self::DeleteForward => "delete-forward",
            Self::ReplaceSelection => "replace-selection",
            Self::MoveCaret => "move-caret",
            Self::ExtendSelection => "extend-selection",
            Self::Submit => "submit",
            Self::Cancel => "cancel",
            Self::SourceInsert => "source-insert",
            Self::Copy => "copy",
            Self::Cut => "cut",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_insert_is_host_owned_intent() {
        let fact = TextEditFact::source_insert("host.text-source.plain");
        assert_eq!(fact.intent.as_str(), "source-insert");
        assert_eq!(fact.host_owned_source.as_deref(), Some("host.text-source.plain"));
        assert!(fact.text.is_empty());
    }
}
