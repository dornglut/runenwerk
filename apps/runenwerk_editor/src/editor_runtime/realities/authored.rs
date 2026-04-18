//! File: apps/runenwerk_editor/src/editor_runtime/realities/authored.rs
//! Purpose: Read-only authored-reality boundary for scene document state.

use editor_core::EntityId;
use editor_scene::SceneEntitySnapshot;

use crate::editor_runtime::SceneDocumentState;

#[derive(Debug, Clone, Copy)]
pub struct AuthoredSceneReality<'a> {
    state: &'a SceneDocumentState,
}

impl<'a> AuthoredSceneReality<'a> {
    pub fn new(state: &'a SceneDocumentState) -> Self {
        Self { state }
    }

    pub fn state(&self) -> &'a SceneDocumentState {
        self.state
    }

    pub fn entity_ids(&self) -> impl Iterator<Item = EntityId> + 'a {
        self.state.entity_ids()
    }

    pub fn entity_snapshot(&self, entity: EntityId) -> Option<SceneEntitySnapshot> {
        self.state.entity_snapshot(entity)
    }

    pub fn entity_display_name(&self, entity: EntityId) -> Option<&'a str> {
        self.state.entity_display_name(entity)
    }
}
