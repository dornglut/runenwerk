use editor_core::{EntityId, SelectionTarget};
use scene::Vec3Value;

use crate::editor_runtime::{TransformPreviewSession, TransformToolKind};
use crate::editor_tools_state::TranslateAxis;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct EditorToolRuntimeState {
    hovered_entity: Option<EntityId>,
    preview: Option<TransformPreviewSession>,
    translate_axis: Option<TranslateAxis>,
}

impl EditorToolRuntimeState {
    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: new
    pub fn new() -> Self {
        Self::default()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: hovered_entity
    pub fn hovered_entity(&self) -> Option<EntityId> {
        self.hovered_entity
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: set_hovered_entity
    pub fn set_hovered_entity(&mut self, entity: Option<EntityId>) {
        self.hovered_entity = entity;
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: preview
    pub fn preview(&self) -> Option<&TransformPreviewSession> {
        self.preview.as_ref()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: preview_active
    pub fn preview_active(&self) -> bool {
        self.preview.is_some()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: translate_axis
    pub fn translate_axis(&self) -> Option<TranslateAxis> {
        self.translate_axis
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: set_translate_axis
    pub fn set_translate_axis(&mut self, axis: Option<TranslateAxis>) -> Result<(), &'static str> {
        if axis.is_some() && self.preview.is_none() {
            return Err("cannot set translate axis without active preview");
        }

        self.translate_axis = axis;
        Ok(())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: begin_preview
    pub fn begin_preview(
        &mut self,
        selection: SelectionTarget,
        tool: TransformToolKind,
    ) -> Result<(), &'static str> {
        let entity = match selection {
            SelectionTarget::Entity(entity) => entity,
            SelectionTarget::Component { entity, .. } => entity,
            _ => return Err("selection target is not previewable"),
        };

        self.preview = Some(TransformPreviewSession::new(entity, tool, selection));
        self.translate_axis = None;
        Ok(())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: update_translation_preview
    pub fn update_translation_preview(&mut self, delta: Vec3Value) -> Result<(), &'static str> {
        let preview = self.preview.as_mut().ok_or("no active preview session")?;

        preview.translation_delta = delta;
        Ok(())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: update_preview
    pub fn update_preview(&mut self) -> Result<(), &'static str> {
        if self.preview.is_none() {
            return Err("no active preview session");
        }

        Ok(())
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: commit_preview
    pub fn commit_preview(&mut self) -> Option<TransformPreviewSession> {
        self.translate_axis = None;
        self.preview.take()
    }

    /// File: apps/runenwerk_editor/src/editor_runtime/tool_state.rs
    /// Method: cancel_preview
    pub fn cancel_preview(&mut self) -> Option<TransformPreviewSession> {
        self.translate_axis = None;
        self.preview.take()
    }
}
