//! Reusable editable-text declarations for control packages.
//!
//! `ui_controls` may describe that a reusable control exposes editable text
//! behavior and which generic policy it needs. It does not own runtime caret
//! state, OS clipboard integration, product/editor/game mutation, authored UI
//! editing, rich text editing, code editor semantics, or product undo stacks.

use serde::{Deserialize, Serialize};

use crate::package::ids::ControlKindId;

/// Editable text modes that can be declared by reusable control packages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlEditableTextMode {
    SingleLine,
    MultiLine,
    ReadOnlySelectable,
    SearchField,
    CommandInput,
    InspectorField,
}

impl ControlEditableTextMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleLine => "single-line",
            Self::MultiLine => "multi-line",
            Self::ReadOnlySelectable => "read-only-selectable",
            Self::SearchField => "search-field",
            Self::CommandInput => "command-input",
            Self::InspectorField => "inspector-field",
        }
    }
}

/// Generic edit intents that may be requested through normalized input facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlEditableTextIntent {
    InsertText,
    DeleteBackward,
    DeleteForward,
    ReplaceSelection,
    MoveCaret,
    ExtendSelection,
    Submit,
    Cancel,
    Paste,
    Copy,
    Cut,
    CompositionStart,
    CompositionUpdate,
    CompositionCommit,
    CompositionCancel,
}

impl ControlEditableTextIntent {
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
            Self::Paste => "paste",
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::CompositionStart => "composition-start",
            Self::CompositionUpdate => "composition-update",
            Self::CompositionCommit => "composition-commit",
            Self::CompositionCancel => "composition-cancel",
        }
    }

    pub const fn mutates_transient_text(self) -> bool {
        matches!(
            self,
            Self::InsertText
                | Self::DeleteBackward
                | Self::DeleteForward
                | Self::ReplaceSelection
                | Self::Paste
                | Self::Cut
                | Self::CompositionCommit
        )
    }

    pub const fn requires_selection(self) -> bool {
        matches!(self, Self::ReplaceSelection | Self::ExtendSelection | Self::Copy | Self::Cut)
    }

    pub const fn requires_composition(self) -> bool {
        matches!(
            self,
            Self::CompositionStart
                | Self::CompositionUpdate
                | Self::CompositionCommit
                | Self::CompositionCancel
        )
    }
}

/// Focus handling requested by a reusable editable-text declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlEditableTextFocusPolicy {
    Focusable,
    RequiresFocus,
    HostOwned,
}

impl ControlEditableTextFocusPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focusable => "focusable",
            Self::RequiresFocus => "requires-focus",
            Self::HostOwned => "host-owned",
        }
    }
}

/// Selection behavior requested by a reusable editable-text declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlEditableTextSelectionPolicy {
    None,
    CaretOnly,
    RangeSelection,
}

impl ControlEditableTextSelectionPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::CaretOnly => "caret-only",
            Self::RangeSelection => "range-selection",
        }
    }

    pub const fn supports_ranges(self) -> bool {
        matches!(self, Self::RangeSelection)
    }
}

/// IME/composition handling requested by a reusable editable-text declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlEditableTextCompositionPolicy {
    None,
    Preedit,
}

impl ControlEditableTextCompositionPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Preedit => "preedit",
        }
    }

    pub const fn supports_preedit(self) -> bool {
        matches!(self, Self::Preedit)
    }
}

/// Public text position units avoid exposing Rust byte offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ControlTextPositionUnit {
    Opaque,
    Grapheme,
    LineColumn,
}

impl ControlTextPositionUnit {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Opaque => "opaque",
            Self::Grapheme => "grapheme",
            Self::LineColumn => "line-column",
        }
    }
}

/// Domain-shaped text position. This is intentionally not a raw byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ControlTextPosition {
    pub unit: ControlTextPositionUnit,
    pub ordinal: u32,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl ControlTextPosition {
    pub const fn opaque(ordinal: u32) -> Self {
        Self {
            unit: ControlTextPositionUnit::Opaque,
            ordinal,
            line: None,
            column: None,
        }
    }

    pub const fn grapheme(ordinal: u32) -> Self {
        Self {
            unit: ControlTextPositionUnit::Grapheme,
            ordinal,
            line: None,
            column: None,
        }
    }

    pub const fn line_column(line: u32, column: u32) -> Self {
        Self {
            unit: ControlTextPositionUnit::LineColumn,
            ordinal: column,
            line: Some(line),
            column: Some(column),
        }
    }
}

/// Domain-shaped range with explicit anchor and extent positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ControlTextRange {
    pub anchor: ControlTextPosition,
    pub extent: ControlTextPosition,
}

impl ControlTextRange {
    pub const fn collapsed(position: ControlTextPosition) -> Self {
        Self {
            anchor: position,
            extent: position,
        }
    }

    pub const fn new(anchor: ControlTextPosition, extent: ControlTextPosition) -> Self {
        Self { anchor, extent }
    }

    pub const fn is_collapsed(&self) -> bool {
        self.anchor.ordinal == self.extent.ordinal
            && self.anchor.line.is_none() == self.extent.line.is_none()
            && self.anchor.column.is_none() == self.extent.column.is_none()
    }
}

/// Package-owned editable-text declaration for one reusable control kind.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlEditableTextDescriptor {
    pub control_kind_id: ControlKindId,
    pub mode: ControlEditableTextMode,
    pub focus_policy: ControlEditableTextFocusPolicy,
    pub selection_policy: ControlEditableTextSelectionPolicy,
    pub composition_policy: ControlEditableTextCompositionPolicy,
    #[serde(default)]
    pub supported_intents: Vec<ControlEditableTextIntent>,
    #[serde(default)]
    pub label_role: Option<String>,
    #[serde(default)]
    pub placeholder_role: Option<String>,
    #[serde(default = "default_true")]
    pub suppresses_when_disabled: bool,
    #[serde(default = "default_true")]
    pub suppresses_when_read_only: bool,
    #[serde(default = "default_true")]
    pub host_owned_clipboard: bool,
    #[serde(default = "default_true")]
    pub host_owned_mutation: bool,
    #[serde(default = "default_true")]
    pub proof_required: bool,
}

impl ControlEditableTextDescriptor {
    pub fn new(control_kind_id: ControlKindId, mode: ControlEditableTextMode) -> Self {
        Self {
            control_kind_id,
            mode,
            focus_policy: ControlEditableTextFocusPolicy::RequiresFocus,
            selection_policy: ControlEditableTextSelectionPolicy::CaretOnly,
            composition_policy: ControlEditableTextCompositionPolicy::Preedit,
            supported_intents: default_mutating_intents(),
            label_role: None,
            placeholder_role: None,
            suppresses_when_disabled: true,
            suppresses_when_read_only: true,
            host_owned_clipboard: true,
            host_owned_mutation: true,
            proof_required: true,
        }
    }

    pub fn single_line(control_kind_id: ControlKindId) -> Self {
        Self::new(control_kind_id, ControlEditableTextMode::SingleLine)
    }

    pub fn multi_line(control_kind_id: ControlKindId) -> Self {
        Self::new(control_kind_id, ControlEditableTextMode::MultiLine)
            .with_intent(ControlEditableTextIntent::MoveCaret)
            .with_intent(ControlEditableTextIntent::ExtendSelection)
    }

    pub fn read_only_selectable(control_kind_id: ControlKindId) -> Self {
        Self {
            supported_intents: vec![
                ControlEditableTextIntent::MoveCaret,
                ControlEditableTextIntent::ExtendSelection,
                ControlEditableTextIntent::Copy,
            ],
            selection_policy: ControlEditableTextSelectionPolicy::RangeSelection,
            composition_policy: ControlEditableTextCompositionPolicy::None,
            ..Self::new(control_kind_id, ControlEditableTextMode::ReadOnlySelectable)
        }
    }

    pub fn search_field(control_kind_id: ControlKindId) -> Self {
        Self::new(control_kind_id, ControlEditableTextMode::SearchField)
            .with_intent(ControlEditableTextIntent::Submit)
    }

    pub fn command_input(control_kind_id: ControlKindId) -> Self {
        Self::new(control_kind_id, ControlEditableTextMode::CommandInput)
            .with_intent(ControlEditableTextIntent::Submit)
            .with_intent(ControlEditableTextIntent::Cancel)
    }

    pub fn inspector_field_input(control_kind_id: ControlKindId) -> Self {
        Self::new(control_kind_id, ControlEditableTextMode::InspectorField)
            .with_intent(ControlEditableTextIntent::Submit)
            .with_intent(ControlEditableTextIntent::Cancel)
            .with_label_role("label.inspector-field")
            .with_placeholder_role("placeholder.inspector-field")
    }

    pub fn with_focus_policy(mut self, value: ControlEditableTextFocusPolicy) -> Self {
        self.focus_policy = value;
        self
    }

    pub fn with_selection_policy(mut self, value: ControlEditableTextSelectionPolicy) -> Self {
        self.selection_policy = value;
        self
    }

    pub fn with_composition_policy(mut self, value: ControlEditableTextCompositionPolicy) -> Self {
        self.composition_policy = value;
        self
    }

    pub fn with_intent(mut self, intent: ControlEditableTextIntent) -> Self {
        if !self.supported_intents.contains(&intent) {
            self.supported_intents.push(intent);
            self.supported_intents.sort();
        }
        self
    }

    pub fn with_supported_intents(
        mut self,
        intents: impl IntoIterator<Item = ControlEditableTextIntent>,
    ) -> Self {
        self.supported_intents = intents.into_iter().collect();
        self.supported_intents.sort();
        self.supported_intents.dedup();
        self
    }

    pub fn with_label_role(mut self, value: impl Into<String>) -> Self {
        self.label_role = Some(value.into());
        self
    }

    pub fn with_placeholder_role(mut self, value: impl Into<String>) -> Self {
        self.placeholder_role = Some(value.into());
        self
    }

    pub fn without_composition(mut self) -> Self {
        self.composition_policy = ControlEditableTextCompositionPolicy::None;
        self.supported_intents
            .retain(|intent| !intent.requires_composition());
        self
    }

    pub fn summary(&self) -> ControlEditableTextSupportSummary {
        ControlEditableTextSupportSummary::from_descriptor(self)
    }
}

/// Read-only catalog/inspection summary for editable text support.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlEditableTextSupportSummary {
    pub control_kind_id: ControlKindId,
    pub modes: Vec<String>,
    pub edit_intents: Vec<String>,
    pub focus_policy: String,
    pub selection_policy: String,
    pub composition_policy: String,
    pub editable_text_supported: bool,
    pub caret_supported: bool,
    pub range_selection_supported: bool,
    pub composition_supported: bool,
    pub suppresses_when_disabled: bool,
    pub suppresses_when_read_only: bool,
    pub host_owned_clipboard: bool,
    pub host_owned_mutation: bool,
    pub proof_required: bool,
    pub executes_host_commands: bool,
    pub mutates_product_state: bool,
    pub authored_ui_edits: bool,
    pub product_undo_redo: bool,
}

impl ControlEditableTextSupportSummary {
    pub fn from_descriptor(descriptor: &ControlEditableTextDescriptor) -> Self {
        let mut edit_intents = descriptor
            .supported_intents
            .iter()
            .map(|intent| intent.as_str().to_owned())
            .collect::<Vec<_>>();
        edit_intents.sort();
        edit_intents.dedup();
        Self {
            control_kind_id: descriptor.control_kind_id.clone(),
            modes: vec![descriptor.mode.as_str().to_owned()],
            edit_intents,
            focus_policy: descriptor.focus_policy.as_str().to_owned(),
            selection_policy: descriptor.selection_policy.as_str().to_owned(),
            composition_policy: descriptor.composition_policy.as_str().to_owned(),
            editable_text_supported: !descriptor.supported_intents.is_empty(),
            caret_supported: descriptor.selection_policy != ControlEditableTextSelectionPolicy::None,
            range_selection_supported: descriptor.selection_policy.supports_ranges(),
            composition_supported: descriptor.composition_policy.supports_preedit(),
            suppresses_when_disabled: descriptor.suppresses_when_disabled,
            suppresses_when_read_only: descriptor.suppresses_when_read_only,
            host_owned_clipboard: descriptor.host_owned_clipboard,
            host_owned_mutation: descriptor.host_owned_mutation,
            proof_required: descriptor.proof_required,
            executes_host_commands: false,
            mutates_product_state: false,
            authored_ui_edits: false,
            product_undo_redo: false,
        }
    }

    pub fn inspection_facts(&self) -> Vec<ControlEditableTextInspectionFact> {
        vec![
            ControlEditableTextInspectionFact::new("text_editing.modes", self.modes.join(",")),
            ControlEditableTextInspectionFact::new(
                "text_editing.edit_intents",
                self.edit_intents.join(","),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.focus_policy",
                self.focus_policy.clone(),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.selection_policy",
                self.selection_policy.clone(),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.composition_policy",
                self.composition_policy.clone(),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.supported",
                bool_string(self.editable_text_supported),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.caret_supported",
                bool_string(self.caret_supported),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.range_selection_supported",
                bool_string(self.range_selection_supported),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.composition_supported",
                bool_string(self.composition_supported),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.host_owned_clipboard",
                bool_string(self.host_owned_clipboard),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.host_owned_mutation",
                bool_string(self.host_owned_mutation),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.executes_host_commands",
                bool_string(self.executes_host_commands),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.mutates_product_state",
                bool_string(self.mutates_product_state),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.authored_ui_edits",
                bool_string(self.authored_ui_edits),
            ),
            ControlEditableTextInspectionFact::new(
                "text_editing.product_undo_redo",
                bool_string(self.product_undo_redo),
            ),
        ]
    }
}

/// One read-only editable-text inspection fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlEditableTextInspectionFact {
    pub key: String,
    pub value: String,
}

impl ControlEditableTextInspectionFact {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

fn default_mutating_intents() -> Vec<ControlEditableTextIntent> {
    vec![
        ControlEditableTextIntent::InsertText,
        ControlEditableTextIntent::DeleteBackward,
        ControlEditableTextIntent::DeleteForward,
        ControlEditableTextIntent::ReplaceSelection,
        ControlEditableTextIntent::MoveCaret,
        ControlEditableTextIntent::ExtendSelection,
        ControlEditableTextIntent::Paste,
        ControlEditableTextIntent::CompositionStart,
        ControlEditableTextIntent::CompositionUpdate,
        ControlEditableTextIntent::CompositionCommit,
        ControlEditableTextIntent::CompositionCancel,
    ]
}

fn default_true() -> bool {
    true
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editable_text_builders_preserve_no_product_behavior() {
        let descriptor = ControlEditableTextDescriptor::inspector_field_input(ControlKindId::new(
            "runenwerk.ui.inspector_field",
        ));
        let summary = descriptor.summary();
        assert_eq!(summary.modes, vec!["inspector-field"]);
        assert!(summary.editable_text_supported);
        assert!(summary.caret_supported);
        assert!(summary.composition_supported);
        assert!(!summary.executes_host_commands);
        assert!(!summary.mutates_product_state);
        assert!(!summary.authored_ui_edits);
        assert!(!summary.product_undo_redo);
    }

    #[test]
    fn public_text_position_is_not_a_byte_offset() {
        let range = ControlTextRange::collapsed(ControlTextPosition::grapheme(2));
        assert!(range.is_collapsed());
        assert_eq!(range.anchor.unit.as_str(), "grapheme");
    }
}
