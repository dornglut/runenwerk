//! File: domain/editor/editor_shell/src/surface_provider.rs
//! Purpose: App-neutral editor surface provider request, artifact, and route DTOs.

use std::collections::BTreeMap;

use editor_core::{DocumentId, DocumentKind, EntityId};
use editor_viewport::{ExpressionProductId, ViewportId};
use id_macros::id;
use ui_surface::{SurfaceCapability, SurfaceCapabilitySet, SurfaceDefinitionId};

use crate::{
    EntityTableSortKey, PanelInstanceId, StructuralCommandTarget, TabStackId,
    ToolSurfaceInstanceId, ToolSurfaceKind, ToolbarViewModel, UiNode, WidgetId,
};

#[id]
pub struct SurfaceProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceProviderPriority(pub u16);

impl SurfaceProviderPriority {
    pub const DEFAULT: Self = Self(100);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceProviderDescriptor {
    pub id: SurfaceProviderId,
    pub label: String,
    pub priority: SurfaceProviderPriority,
}

impl SurfaceProviderDescriptor {
    pub fn new(
        id: SurfaceProviderId,
        label: impl Into<String>,
        priority: SurfaceProviderPriority,
    ) -> Self {
        Self {
            id,
            label: label.into(),
            priority,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceDocumentContext {
    Resolved {
        document_id: DocumentId,
        document_kind: DocumentKind,
    },
    NoActiveDocument,
    Unresolved {
        document_id: DocumentId,
    },
}

impl SurfaceDocumentContext {
    pub fn resolved_document_id(&self) -> Option<DocumentId> {
        match self {
            Self::Resolved { document_id, .. } | Self::Unresolved { document_id } => {
                Some(*document_id)
            }
            Self::NoActiveDocument => None,
        }
    }

    pub fn resolved_document_kind(&self) -> Option<&DocumentKind> {
        match self {
            Self::Resolved { document_kind, .. } => Some(document_kind),
            Self::NoActiveDocument | Self::Unresolved { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceProviderRequest {
    pub workspace_profile_id: crate::WorkspaceProfileId,
    pub document_context: SurfaceDocumentContext,
    pub panel_instance_id: PanelInstanceId,
    pub tab_stack_id: TabStackId,
    pub tool_surface_instance_id: ToolSurfaceInstanceId,
    pub tool_surface_kind: ToolSurfaceKind,
    pub surface_definition_id: SurfaceDefinitionId,
    pub capabilities: SurfaceCapabilitySet,
}

impl SurfaceProviderRequest {
    pub fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceProviderAvailability {
    Available,
    Unsupported,
    Ambiguous,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceProviderDiagnostic {
    pub code: &'static str,
    pub message: String,
}

impl SurfaceProviderDiagnostic {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfacePresentationArtifactKind {
    Provider,
    Unsupported,
    Ambiguous,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfacePresentationArtifact {
    pub kind: SurfacePresentationArtifactKind,
    pub root: UiNode,
    pub diagnostics: Vec<SurfaceProviderDiagnostic>,
}

impl SurfacePresentationArtifact {
    pub fn provider(root: UiNode) -> Self {
        Self {
            kind: SurfacePresentationArtifactKind::Provider,
            root,
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostic(
        kind: SurfacePresentationArtifactKind,
        root: UiNode,
        diagnostic: SurfaceProviderDiagnostic,
    ) -> Self {
        Self {
            kind,
            root,
            diagnostics: vec![diagnostic],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceLocalAction {
    SelectOutlinerEntity {
        entity: EntityId,
    },
    SelectEntityTableEntity {
        entity: EntityId,
    },
    SelectEntityTableRow {
        entities: Vec<EntityId>,
    },
    AppendEntityTableSearchText {
        text: String,
    },
    BackspaceEntityTableSearch,
    ToggleEntityTableSort {
        sort_key: EntityTableSortKey,
    },
    SelectViewportProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
        enabled: bool,
    },
    ToggleViewportDetails,
    ActivateInspectorField {
        index: usize,
    },
    FocusInspectorField {
        index: usize,
    },
    EditInspectorFieldText {
        index: usize,
        text: String,
    },
    BackspaceInspectorFieldText {
        index: usize,
    },
    CommitInspectorFieldText {
        index: usize,
    },
    CancelInspectorFieldText {
        index: usize,
    },
    SelectEditorDefinitionDocument {
        document_id: String,
    },
    DuplicateSelectedEditorDefinition,
    RenameSelectedEditorDefinition {
        display_name: String,
    },
    DeleteSelectedEditorDefinition,
    ExportSelectedEditorDefinition,
    ApplySelectedEditorDefinition,
    RollbackSelectedEditorDefinition,
    SelectEditorDefinitionUiNode {
        node_id: String,
    },
    SetSelectedEditorDefinitionUiNodeText {
        node_id: String,
        text: String,
    },
    SetSelectedEditorThemeColor {
        token: String,
        value: String,
    },
    AddSelectedEditorWorkspaceLayoutTab {
        label: String,
        tool_surface: String,
    },
    SplitSelectedEditorWorkspaceLayoutRoot {
        axis: String,
    },
    CloseSelectedEditorWorkspaceLayoutLastTab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceLocalRoute {
    pub action: SurfaceLocalAction,
}

impl SurfaceLocalRoute {
    pub fn new(action: SurfaceLocalAction) -> Self {
        Self { action }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SurfaceRouteTable {
    routes_by_widget_id: BTreeMap<WidgetId, SurfaceLocalRoute>,
}

impl SurfaceRouteTable {
    pub fn new(routes_by_widget_id: BTreeMap<WidgetId, SurfaceLocalRoute>) -> Self {
        Self {
            routes_by_widget_id,
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, widget_id: WidgetId, route: SurfaceLocalRoute) {
        self.routes_by_widget_id.insert(widget_id, route);
    }

    pub fn get(&self, widget_id: &WidgetId) -> Option<&SurfaceLocalRoute> {
        self.routes_by_widget_id.get(widget_id)
    }

    pub fn is_empty(&self) -> bool {
        self.routes_by_widget_id.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&WidgetId, &SurfaceLocalRoute)> {
        self.routes_by_widget_id.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSurfaceFrame {
    pub surface_instance_id: ToolSurfaceInstanceId,
    pub panel_instance_id: PanelInstanceId,
    pub tab_stack_id: TabStackId,
    pub surface_kind: ToolSurfaceKind,
    pub surface_definition_id: SurfaceDefinitionId,
    pub provider_id: Option<SurfaceProviderId>,
    pub title: String,
    pub artifact: SurfacePresentationArtifact,
    pub routes: SurfaceRouteTable,
    pub availability: SurfaceProviderAvailability,
}

impl ResolvedSurfaceFrame {
    pub fn diagnostic(
        request: &SurfaceProviderRequest,
        title: impl Into<String>,
        availability: SurfaceProviderAvailability,
        artifact: SurfacePresentationArtifact,
    ) -> Self {
        Self {
            surface_instance_id: request.tool_surface_instance_id,
            panel_instance_id: request.panel_instance_id,
            tab_stack_id: request.tab_stack_id,
            surface_kind: request.tool_surface_kind,
            surface_definition_id: request.surface_definition_id,
            provider_id: None,
            title: title.into(),
            artifact,
            routes: SurfaceRouteTable::empty(),
            availability,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorShellFrameModel {
    pub toolbar: ToolbarViewModel,
    pub surfaces: BTreeMap<ToolSurfaceInstanceId, ResolvedSurfaceFrame>,
}

impl EditorShellFrameModel {
    pub fn new(
        toolbar: ToolbarViewModel,
        surfaces: BTreeMap<ToolSurfaceInstanceId, ResolvedSurfaceFrame>,
    ) -> Self {
        Self { toolbar, surfaces }
    }

    pub fn surface(&self, surface_id: ToolSurfaceInstanceId) -> Option<&ResolvedSurfaceFrame> {
        self.surfaces.get(&surface_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceCommandProposal {
    SurfaceSession(SurfaceSessionMutationProposal),
    EditorDomain(EditorDomainProposal),
    Shell(crate::ShellCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceSessionMutationProposal {
    pub target: StructuralCommandTarget,
    pub projection_epoch: u64,
    pub mutation: SurfaceSessionMutation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceSessionMutation {
    AppendEntityTableSearchText { text: String },
    BackspaceEntityTableSearch,
    ToggleEntityTableSort { sort_key: EntityTableSortKey },
    ToggleViewportDetails,
    ActivateInspectorField { index: usize },
    FocusInspectorField { index: usize },
    AppendInspectorFieldText { index: usize, text: String },
    BackspaceInspectorFieldText { index: usize },
    CommitInspectorFieldText { index: usize },
    CancelInspectorFieldText { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorDomainProposal {
    pub target: StructuralCommandTarget,
    pub projection_epoch: u64,
    pub mutation: EditorDomainMutation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorDomainMutation {
    SelectOutlinerEntity {
        entity: EntityId,
    },
    SelectEntityTableRow {
        entities: Vec<EntityId>,
    },
    SelectViewportProduct {
        viewport_id: ViewportId,
        product_id: ExpressionProductId,
    },
}

pub fn surface_session_proposal(
    request: &SurfaceProviderRequest,
    projection_epoch: u64,
    mutation: SurfaceSessionMutation,
) -> SurfaceCommandProposal {
    SurfaceCommandProposal::SurfaceSession(SurfaceSessionMutationProposal {
        target: surface_command_target(request),
        projection_epoch,
        mutation,
    })
}

pub fn editor_domain_proposal(
    request: &SurfaceProviderRequest,
    projection_epoch: u64,
    mutation: EditorDomainMutation,
) -> SurfaceCommandProposal {
    SurfaceCommandProposal::EditorDomain(EditorDomainProposal {
        target: surface_command_target(request),
        projection_epoch,
        mutation,
    })
}

pub fn surface_command_target(request: &SurfaceProviderRequest) -> StructuralCommandTarget {
    StructuralCommandTarget {
        panel_instance_id: request.panel_instance_id,
        active_tool_surface: Some(request.tool_surface_instance_id),
        tab_stack_id: request.tab_stack_id,
    }
}
