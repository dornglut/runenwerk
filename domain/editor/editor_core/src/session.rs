//! File: domain/editor/editor_core/src/session.rs
//! Purpose: Root editor session state.

use std::collections::BTreeMap;

use crate::{
    DocumentDescriptor, DocumentId, DocumentKind, EditorMutationError, HistoryStack, SelectionSet,
    ToolId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModeId(pub u64);

pub const EDIT_MODE_ID: ModeId = ModeId(1);
pub const PLAY_MODE_ID: ModeId = ModeId(2);
pub const SIMULATE_MODE_ID: ModeId = ModeId(3);
pub const PREVIEW_MODE_ID: ModeId = ModeId(4);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModeDescriptor {
    pub id: ModeId,
    pub stable_name: &'static str,
    pub display_name: String,
    compatible_document_kinds: Vec<DocumentKind>,
}

impl ModeDescriptor {
    pub fn new(
        id: ModeId,
        stable_name: &'static str,
        display_name: impl Into<String>,
        compatible_document_kinds: Vec<DocumentKind>,
    ) -> Self {
        Self {
            id,
            stable_name,
            display_name: display_name.into(),
            compatible_document_kinds,
        }
    }

    pub fn compatible_document_kinds(&self) -> &[DocumentKind] {
        &self.compatible_document_kinds
    }

    pub fn supports_document_kind(&self, document_kind: &DocumentKind) -> bool {
        self.compatible_document_kinds.is_empty()
            || self.compatible_document_kinds.contains(document_kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeRegistry {
    modes: BTreeMap<ModeId, ModeDescriptor>,
}

impl Default for ModeRegistry {
    fn default() -> Self {
        Self::new(default_mode_descriptors())
    }
}

impl ModeRegistry {
    pub fn new(modes: impl IntoIterator<Item = ModeDescriptor>) -> Self {
        Self {
            modes: modes.into_iter().map(|mode| (mode.id, mode)).collect(),
        }
    }

    pub fn default_registry() -> Self {
        Self::default()
    }

    pub fn mode(&self, mode_id: ModeId) -> Option<&ModeDescriptor> {
        self.modes.get(&mode_id)
    }

    pub fn modes(&self) -> impl Iterator<Item = &ModeDescriptor> {
        self.modes.values()
    }

    pub fn validate_activation(
        &self,
        mode_id: ModeId,
        context: &ModeActivationContext,
    ) -> Result<(), EditorMutationError> {
        if !context.workspace_mode_ids.contains(&mode_id) {
            return Err(EditorMutationError::session_rejected(
                "mode is not enabled for the active workspace",
            ));
        }

        let mode = self
            .mode(mode_id)
            .ok_or(EditorMutationError::session_rejected(
                "mode is not registered",
            ))?;

        let document_kind =
            context
                .document_kind
                .as_ref()
                .ok_or(EditorMutationError::session_rejected(
                    "mode activation requires an active document",
                ))?;

        if !mode.supports_document_kind(document_kind) {
            return Err(EditorMutationError::session_rejected(
                "mode is not compatible with the active document kind",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeActivationContext {
    pub workspace_mode_ids: Vec<ModeId>,
    pub document_kind: Option<DocumentKind>,
}

impl ModeActivationContext {
    pub fn new(
        workspace_mode_ids: impl IntoIterator<Item = ModeId>,
        document_kind: Option<DocumentKind>,
    ) -> Self {
        Self {
            workspace_mode_ids: workspace_mode_ids.into_iter().collect(),
            document_kind,
        }
    }
}

pub fn default_mode_descriptors() -> Vec<ModeDescriptor> {
    vec![
        ModeDescriptor::new(
            EDIT_MODE_ID,
            "edit",
            "Edit",
            vec![
                DocumentKind::Scene,
                DocumentKind::Prefab,
                DocumentKind::SdfGraph,
                DocumentKind::SdfBrushLayer,
                DocumentKind::FieldWorldDefinition,
                DocumentKind::MaterialGraph,
                DocumentKind::Material,
                DocumentKind::ProceduralTexture,
                DocumentKind::VolumeTexture,
                DocumentKind::ProceduralGenerationGraph,
                DocumentKind::GameplayGraph,
                DocumentKind::GameplayRuleTrigger,
                DocumentKind::Ability,
                DocumentKind::Quest,
                DocumentKind::ParticleGraph,
                DocumentKind::ParticleEmitter,
                DocumentKind::PhysicsScene,
                DocumentKind::PhysicsConfig,
                DocumentKind::AnimationClip,
                DocumentKind::AnimationGraph,
                DocumentKind::Timeline,
                DocumentKind::UiLayout,
                DocumentKind::Graph,
                DocumentKind::Script,
                DocumentKind::ForeignMeshReferenceImport,
                DocumentKind::AssetCatalog,
                DocumentKind::WorkspaceDefinition,
                DocumentKind::Theme,
                DocumentKind::Shortcut,
                DocumentKind::Menu,
                DocumentKind::CommandBinding,
                DocumentKind::PanelRegistry,
                DocumentKind::ToolSurfaceDefinition,
            ],
        ),
        ModeDescriptor::new(
            PREVIEW_MODE_ID,
            "preview",
            "Preview",
            vec![
                DocumentKind::Scene,
                DocumentKind::FieldProductPreview,
                DocumentKind::RuntimeDebug,
                DocumentKind::UiLayout,
            ],
        ),
        ModeDescriptor::new(
            PLAY_MODE_ID,
            "play",
            "Play",
            vec![DocumentKind::Scene, DocumentKind::RuntimeDebug],
        ),
        ModeDescriptor::new(
            SIMULATE_MODE_ID,
            "simulate",
            "Simulate",
            vec![
                DocumentKind::Scene,
                DocumentKind::PhysicsScene,
                DocumentKind::RuntimeDebug,
            ],
        ),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirtyDocumentClosePolicy {
    RejectDirty,
    DiscardDirty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentCompatibilityContext {
    pub allowed_document_kinds: Vec<DocumentKind>,
}

impl DocumentCompatibilityContext {
    pub fn new(allowed_document_kinds: impl IntoIterator<Item = DocumentKind>) -> Self {
        Self {
            allowed_document_kinds: allowed_document_kinds.into_iter().collect(),
        }
    }

    pub fn allows(&self, document_kind: &DocumentKind) -> bool {
        self.allowed_document_kinds.is_empty()
            || self.allowed_document_kinds.contains(document_kind)
    }
}

#[derive(Debug, Default)]
pub struct EditorSession {
    documents: BTreeMap<DocumentId, DocumentDescriptor>,
    document_tabs: Vec<DocumentId>,
    active_document: Option<DocumentId>,
    active_tool: Option<ToolId>,
    active_mode: ModeId,
    selection: SelectionSet,
    history: HistoryStack,
}

impl Default for ModeId {
    fn default() -> Self {
        EDIT_MODE_ID
    }
}

impl EditorSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> ModeId {
        self.active_mode
    }

    pub fn active_mode(&self) -> ModeId {
        self.active_mode
    }

    pub fn set_mode(&mut self, mode: ModeId) {
        self.active_mode = mode;
    }

    pub fn activate_mode(
        &mut self,
        mode_id: ModeId,
        registry: &ModeRegistry,
        context: &ModeActivationContext,
    ) -> Result<(), EditorMutationError> {
        registry.validate_activation(mode_id, context)?;
        self.active_mode = mode_id;
        Ok(())
    }

    pub fn active_document(&self) -> Option<DocumentId> {
        self.active_document
    }

    pub fn set_active_document(
        &mut self,
        document_id: Option<DocumentId>,
    ) -> Result<(), EditorMutationError> {
        if let Some(document_id) = document_id
            && !self.documents.contains_key(&document_id)
        {
            return Err(EditorMutationError::session_rejected("document not found"));
        }

        self.active_document = document_id;
        Ok(())
    }

    pub fn activate_document(
        &mut self,
        document_id: DocumentId,
    ) -> Result<(), EditorMutationError> {
        self.set_active_document(Some(document_id))
    }

    pub fn activate_document_with_compatibility(
        &mut self,
        document_id: DocumentId,
        compatibility: &DocumentCompatibilityContext,
    ) -> Result<(), EditorMutationError> {
        let document = self
            .document(document_id)
            .ok_or(EditorMutationError::session_rejected("document not found"))?;

        if !compatibility.allows(&document.kind) {
            return Err(EditorMutationError::session_rejected(
                "document kind is not compatible with the active workspace",
            ));
        }

        self.active_document = Some(document_id);
        Ok(())
    }

    pub fn active_document_descriptor(&self) -> Option<&DocumentDescriptor> {
        self.active_document
            .and_then(|document_id| self.document(document_id))
    }

    pub fn active_tool(&self) -> Option<ToolId> {
        self.active_tool
    }

    pub fn set_active_tool(&mut self, tool_id: Option<ToolId>) {
        self.active_tool = tool_id;
    }

    pub fn documents(&self) -> impl Iterator<Item = &DocumentDescriptor> {
        self.documents.values()
    }

    pub fn document_tabs(&self) -> &[DocumentId] {
        &self.document_tabs
    }

    pub fn document_tab_descriptors(&self) -> impl Iterator<Item = &DocumentDescriptor> {
        self.document_tabs
            .iter()
            .filter_map(|document_id| self.documents.get(document_id))
    }

    pub fn document(&self, document_id: DocumentId) -> Option<&DocumentDescriptor> {
        self.documents.get(&document_id)
    }

    pub fn document_mut(&mut self, document_id: DocumentId) -> Option<&mut DocumentDescriptor> {
        self.documents.get_mut(&document_id)
    }

    pub fn upsert_document(
        &mut self,
        descriptor: DocumentDescriptor,
    ) -> Option<DocumentDescriptor> {
        let document_id = descriptor.id;
        let previous = self.documents.insert(document_id, descriptor);
        if previous.is_none() && !self.document_tabs.contains(&document_id) {
            self.document_tabs.push(document_id);
        }
        if self.active_document.is_none() {
            self.active_document = Some(document_id);
        }
        previous
    }

    pub fn remove_document(&mut self, document_id: DocumentId) -> Option<DocumentDescriptor> {
        self.document_tabs.retain(|tab_id| *tab_id != document_id);
        if self.active_document == Some(document_id) {
            self.active_document = None;
        }

        self.documents.remove(&document_id)
    }

    pub fn close_document(
        &mut self,
        document_id: DocumentId,
        policy: DirtyDocumentClosePolicy,
    ) -> Result<DocumentDescriptor, EditorMutationError> {
        let document = self
            .document(document_id)
            .ok_or(EditorMutationError::session_rejected("document not found"))?;

        if document.is_dirty && policy == DirtyDocumentClosePolicy::RejectDirty {
            return Err(EditorMutationError::session_rejected(
                "dirty document close rejected",
            ));
        }

        let removed_index = self
            .document_tabs
            .iter()
            .position(|tab_id| *tab_id == document_id);
        let removed = self
            .remove_document(document_id)
            .ok_or(EditorMutationError::session_rejected("document not found"))?;

        if self.active_document.is_none() {
            self.active_document = removed_index.and_then(|index| {
                self.document_tabs
                    .get(index)
                    .or_else(|| {
                        index
                            .checked_sub(1)
                            .and_then(|prev| self.document_tabs.get(prev))
                    })
                    .copied()
            });
        }

        Ok(removed)
    }

    pub fn selection(&self) -> &SelectionSet {
        &self.selection
    }

    pub fn selection_mut(&mut self) -> &mut SelectionSet {
        &mut self.selection
    }

    pub fn history(&self) -> &HistoryStack {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut HistoryStack {
        &mut self.history
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    pub fn select_single(&mut self, target: crate::SelectionTarget) {
        self.selection.set_single(target);
    }

    pub fn add_selection(&mut self, target: crate::SelectionTarget) {
        self.selection.add(target);
    }

    pub fn set_document_dirty(
        &mut self,
        document_id: crate::DocumentId,
        is_dirty: bool,
    ) -> Result<(), EditorMutationError> {
        let document = self
            .documents
            .get_mut(&document_id)
            .ok_or(EditorMutationError::session_rejected("document not found"))?;

        document.is_dirty = is_dirty;
        Ok(())
    }

    pub fn mark_document_dirty(
        &mut self,
        document_id: crate::DocumentId,
    ) -> Result<(), EditorMutationError> {
        self.set_document_dirty(document_id, true)
    }

    pub fn mark_document_saved(
        &mut self,
        document_id: crate::DocumentId,
    ) -> Result<(), EditorMutationError> {
        self.set_document_dirty(document_id, false)
    }
}

impl crate::CommandContext for EditorSession {
    type Error = EditorMutationError;

    fn mark_document_dirty(
        &mut self,
        document_id: crate::DocumentId,
        is_dirty: bool,
    ) -> Result<(), Self::Error> {
        self.set_document_dirty(document_id, is_dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(id: u64, kind: DocumentKind) -> DocumentDescriptor {
        DocumentDescriptor::new(DocumentId(id), kind, format!("Document {id}"))
    }

    #[test]
    fn upsert_document_opens_ordered_tabs_and_sets_first_active_document() {
        let mut session = EditorSession::new();

        session.upsert_document(descriptor(1, DocumentKind::Scene));
        session.upsert_document(descriptor(2, DocumentKind::MaterialGraph));

        assert_eq!(session.document_tabs(), &[DocumentId(1), DocumentId(2)]);
        assert_eq!(session.active_document(), Some(DocumentId(1)));
    }

    #[test]
    fn close_document_rejects_dirty_document_without_discard_policy() {
        let mut session = EditorSession::new();
        session.upsert_document(descriptor(1, DocumentKind::Scene).with_dirty(true));

        let result = session.close_document(DocumentId(1), DirtyDocumentClosePolicy::RejectDirty);

        assert!(result.is_err());
        assert_eq!(session.document_tabs(), &[DocumentId(1)]);
    }

    #[test]
    fn close_active_document_activates_neighbor_tab() {
        let mut session = EditorSession::new();
        session.upsert_document(descriptor(1, DocumentKind::Scene));
        session.upsert_document(descriptor(2, DocumentKind::Material));
        session
            .activate_document(DocumentId(2))
            .expect("document should exist");

        session
            .close_document(DocumentId(2), DirtyDocumentClosePolicy::RejectDirty)
            .expect("clean document should close");

        assert_eq!(session.document_tabs(), &[DocumentId(1)]);
        assert_eq!(session.active_document(), Some(DocumentId(1)));
    }

    #[test]
    fn mode_activation_validates_workspace_and_document_compatibility() {
        let registry = ModeRegistry::default_registry();
        let context = ModeActivationContext::new([EDIT_MODE_ID], Some(DocumentKind::MaterialGraph));
        let mut session = EditorSession::new();

        session
            .activate_mode(EDIT_MODE_ID, &registry, &context)
            .expect("edit mode should support material graphs in an edit workspace");

        let rejected = session.activate_mode(PLAY_MODE_ID, &registry, &context);
        assert!(rejected.is_err());
    }

    #[test]
    fn preview_mode_fails_closed_for_non_preview_document_kinds() {
        let registry = ModeRegistry::default_registry();
        let mut session = EditorSession::new();

        session
            .activate_mode(
                PREVIEW_MODE_ID,
                &registry,
                &ModeActivationContext::new([PREVIEW_MODE_ID], Some(DocumentKind::Scene)),
            )
            .expect("scene preview should be compatible");

        let rejected = session.activate_mode(
            PREVIEW_MODE_ID,
            &registry,
            &ModeActivationContext::new([PREVIEW_MODE_ID], Some(DocumentKind::Theme)),
        );

        assert!(rejected.is_err());
    }

    #[test]
    fn active_document_switch_validates_document_compatibility() {
        let mut session = EditorSession::new();
        session.upsert_document(descriptor(1, DocumentKind::Scene));
        session.upsert_document(descriptor(2, DocumentKind::Theme));
        let compatibility = DocumentCompatibilityContext::new([DocumentKind::Scene]);

        let rejected = session.activate_document_with_compatibility(DocumentId(2), &compatibility);

        assert!(rejected.is_err());
        assert_eq!(session.active_document(), Some(DocumentId(1)));
    }
}
