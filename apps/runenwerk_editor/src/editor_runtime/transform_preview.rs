use editor_core::{EntityId, SelectionTarget};
use scene::Vec3Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformToolKind {
    Translate,
    Rotate,
    Scale,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransformPreviewSession {
    pub entity: EntityId,
    pub tool: TransformToolKind,
    pub started_from_selection: SelectionTarget,
    pub translation_delta: Vec3Value,
}

impl TransformPreviewSession {
    pub fn new(
        entity: EntityId,
        tool: TransformToolKind,
        started_from_selection: SelectionTarget,
    ) -> Self {
        Self {
            entity,
            tool,
            started_from_selection,
            translation_delta: Vec3Value::zero(),
        }
    }

    pub fn with_translation_delta(mut self, delta: Vec3Value) -> Self {
        self.translation_delta = delta;
        self
    }
}
