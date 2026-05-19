//! File: domain/editor/editor_shell/src/surface_provider.rs
//! Purpose: App-neutral editor surface provider request, artifact, and route DTOs.

use std::collections::BTreeMap;

use editor_core::{DocumentId, DocumentKind};
use editor_definition::EditorToolbarBinding;
use id_macros::id;
use ui_definition::NormalizedUiTemplate;
use ui_surface::{SurfaceCapability, SurfaceCapabilitySet, SurfaceDefinitionId};

use crate::{
    AssetSurfaceAction, EditorDefinitionSurfaceAction, EntityTableDomainMutation,
    EntityTableSessionMutation, EntityTableSurfaceAction, InspectorSessionMutation,
    InspectorSurfaceAction, MaterialSurfaceAction, OutlinerDomainMutation, OutlinerSurfaceAction,
    PanelInstanceId, PanelKind, ProviderFamilyId, RoutedShellAction, SdfOperationDomainMutation,
    SdfOperationSessionMutation, SdfOperationSurfaceAction, StructuralCommandTarget, TabStackId,
    TextureSurfaceAction, ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceRoute,
    ToolSurfaceStableKey, ToolbarViewModel, UiNode, ViewportDomainMutation,
    ViewportSessionMutation, ViewportSurfaceAction, WidgetId, WorkspaceSurfaceIdentityError,
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
    pub stable_surface_key: ToolSurfaceStableKey,
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    pub provider_family_id: Option<ProviderFamilyId>,
    pub surface_route: Option<ToolSurfaceRoute>,
    pub surface_definition_id: SurfaceDefinitionId,
    pub capabilities: SurfaceCapabilitySet,
}

impl SurfaceProviderRequest {
    pub fn has_capability(&self, capability: SurfaceCapability) -> bool {
        self.capabilities.allows(capability)
    }

    pub fn stable_key(&self) -> &ToolSurfaceStableKey {
        &self.stable_surface_key
    }

    pub const fn legacy_kind(&self) -> Option<ToolSurfaceKind> {
        self.legacy_tool_surface_kind
    }

    pub fn legacy_kind_or_error(&self) -> Result<ToolSurfaceKind, WorkspaceSurfaceIdentityError> {
        self.legacy_tool_surface_kind.ok_or_else(|| {
            WorkspaceSurfaceIdentityError::MissingLegacyCompatibilityKind {
                stable_surface_key: self.stable_surface_key.clone(),
            }
        })
    }

    pub fn matches_stable_key(&self, expected: &str) -> bool {
        let Ok(expected) = ToolSurfaceStableKey::new(expected) else {
            return false;
        };
        self.stable_surface_key == expected
    }

    pub fn matches_any_stable_key(&self, expected: &[&str]) -> bool {
        expected
            .iter()
            .any(|expected| self.matches_stable_key(expected))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceProviderSupportMode {
    StableKey,
    LegacyKind,
    Unsupported,
}

impl SurfaceProviderSupportMode {
    pub const fn is_supported(self) -> bool {
        matches!(self, Self::StableKey | Self::LegacyKind)
    }

    pub const fn preferred(self, other: Self) -> Self {
        match (self, other) {
            (Self::StableKey, _) | (_, Self::StableKey) => Self::StableKey,
            (Self::LegacyKind, _) | (_, Self::LegacyKind) => Self::LegacyKind,
            (Self::Unsupported, Self::Unsupported) => Self::Unsupported,
        }
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

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceLocalAction {
    Asset(AssetSurfaceAction),
    Material(MaterialSurfaceAction),
    Outliner(OutlinerSurfaceAction),
    EntityTable(EntityTableSurfaceAction),
    Inspector(InspectorSurfaceAction),
    Viewport(ViewportSurfaceAction),
    EditorDefinition(EditorDefinitionSurfaceAction),
    SdfOperation(SdfOperationSurfaceAction),
    Texture(TextureSurfaceAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceLocalRoute {
    Static { action: SurfaceLocalAction },
    ProviderOwnedGraphCanvas,
}

impl SurfaceLocalRoute {
    pub fn new(action: SurfaceLocalAction) -> Self {
        Self::Static { action }
    }

    pub const fn provider_owned_graph_canvas() -> Self {
        Self::ProviderOwnedGraphCanvas
    }

    pub const fn action(&self) -> Option<&SurfaceLocalAction> {
        match self {
            Self::Static { action } => Some(action),
            Self::ProviderOwnedGraphCanvas => None,
        }
    }

    pub const fn is_provider_owned_graph_canvas(&self) -> bool {
        matches!(self, Self::ProviderOwnedGraphCanvas)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceInteraction {
    GraphCanvasAction(ui_graph_editor::GraphCanvasAction),
}

#[derive(Debug, Clone, PartialEq, Default)]
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
    /// C6C shell UI compatibility boundary pending final cleanup: frame
    /// artifacts keep the legacy kind only for enum-backed labels and commands
    /// that have not been retired yet. `stable_surface_key` is the surface
    /// identity.
    pub surface_kind: Option<ToolSurfaceKind>,
    pub stable_surface_key: ToolSurfaceStableKey,
    pub surface_definition_id: SurfaceDefinitionId,
    pub provider_id: Option<SurfaceProviderId>,
    pub title: String,
    pub artifact: SurfacePresentationArtifact,
    pub routes: SurfaceRouteTable,
    pub availability: SurfaceProviderAvailability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSurfaceCreateCandidate {
    pub stable_surface_key: ToolSurfaceStableKey,
    pub label: String,
    pub panel_kind: PanelKind,
    pub legacy_tool_surface_kind: Option<ToolSurfaceKind>,
}

impl ToolSurfaceCreateCandidate {
    pub fn new(
        stable_surface_key: ToolSurfaceStableKey,
        label: impl Into<String>,
        panel_kind: PanelKind,
        legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    ) -> Self {
        Self {
            stable_surface_key,
            label: label.into(),
            panel_kind,
            legacy_tool_surface_kind,
        }
    }
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
            surface_kind: request.legacy_tool_surface_kind,
            stable_surface_key: request.stable_surface_key.clone(),
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
pub enum TabStackPopupMenuKind {
    AreaActions,
    SurfaceKinds,
    CreateSurface,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveTabStackPopupMenu {
    pub kind: TabStackPopupMenuKind,
    pub tab_stack_id: TabStackId,
    pub anchor_widget_id: crate::WidgetId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorShellFrameModel {
    pub toolbar: ToolbarViewModel,
    pub surfaces: BTreeMap<ToolSurfaceInstanceId, ResolvedSurfaceFrame>,
    pub active_tab_stack_popup_menu: Option<ActiveTabStackPopupMenu>,
    pub route_actions_by_route_target: BTreeMap<String, RoutedShellAction>,
    pub available_panel_kinds: Vec<PanelKind>,
    /// C6C shell UI compatibility boundary pending final cleanup:
    /// the switch-type menu still exposes enum-backed choices. Normal creation
    /// and provider request identity use stable keys.
    pub available_tool_surface_kinds: Vec<ToolSurfaceKind>,
    pub available_tool_surface_create_candidates: Vec<ToolSurfaceCreateCandidate>,
    pub active_toolbar_template: Option<NormalizedUiTemplate>,
    pub active_toolbar_binding: Option<EditorToolbarBinding>,
    pub active_shell_chrome_template: Option<NormalizedUiTemplate>,
}

impl EditorShellFrameModel {
    pub fn new(
        toolbar: ToolbarViewModel,
        surfaces: BTreeMap<ToolSurfaceInstanceId, ResolvedSurfaceFrame>,
    ) -> Self {
        Self {
            toolbar,
            surfaces,
            active_tab_stack_popup_menu: None,
            route_actions_by_route_target: BTreeMap::new(),
            available_panel_kinds: Vec::new(),
            available_tool_surface_kinds: Vec::new(),
            available_tool_surface_create_candidates: Vec::new(),
            active_toolbar_template: None,
            active_toolbar_binding: None,
            active_shell_chrome_template: None,
        }
    }

    pub fn with_active_tab_stack_popup_menu(
        mut self,
        popup_menu: Option<ActiveTabStackPopupMenu>,
    ) -> Self {
        self.active_tab_stack_popup_menu = popup_menu;
        self
    }

    /// C6B legacy boundary for enum-backed shell surface menus pending final
    /// cleanup. Normal provider request identity is `ToolSurfaceStableKey`.
    pub fn with_available_tool_surface_kinds(mut self, kinds: Vec<ToolSurfaceKind>) -> Self {
        self.available_tool_surface_kinds = kinds;
        self
    }

    pub fn with_available_tool_surface_create_candidates(
        mut self,
        candidates: Vec<ToolSurfaceCreateCandidate>,
    ) -> Self {
        self.available_tool_surface_create_candidates = candidates;
        self
    }

    pub fn with_available_panel_kinds(mut self, kinds: Vec<PanelKind>) -> Self {
        self.available_panel_kinds = kinds;
        self
    }

    pub fn with_route_actions(mut self, actions: BTreeMap<String, RoutedShellAction>) -> Self {
        self.route_actions_by_route_target = actions;
        self
    }

    pub fn with_active_ui_definitions(
        mut self,
        toolbar_template: Option<NormalizedUiTemplate>,
        toolbar_binding: Option<EditorToolbarBinding>,
        shell_chrome_template: Option<NormalizedUiTemplate>,
    ) -> Self {
        self.active_toolbar_template = toolbar_template;
        self.active_toolbar_binding = toolbar_binding;
        self.active_shell_chrome_template = shell_chrome_template;
        self
    }

    pub fn surface(&self, surface_id: ToolSurfaceInstanceId) -> Option<&ResolvedSurfaceFrame> {
        self.surfaces.get(&surface_id)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceCommandProposal {
    SurfaceSession(SurfaceSessionMutationProposal),
    EditorDomain(EditorDomainProposal),
    Shell(crate::ShellCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceSessionMutationProposal {
    pub target: StructuralCommandTarget,
    pub projection_epoch: u64,
    pub mutation: SurfaceSessionMutation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceSessionMutation {
    EntityTable(EntityTableSessionMutation),
    Inspector(InspectorSessionMutation),
    Viewport(ViewportSessionMutation),
    SdfOperation(SdfOperationSessionMutation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorDomainProposal {
    pub target: StructuralCommandTarget,
    pub projection_epoch: u64,
    pub mutation: EditorDomainMutation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorDomainMutation {
    Outliner(OutlinerDomainMutation),
    EntityTable(EntityTableDomainMutation),
    Viewport(ViewportDomainMutation),
    SdfOperation(SdfOperationDomainMutation),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID, MATERIAL_WORKSPACE_PROFILE_ID,
        ProviderFamilyId, ToolSurfaceRoute, ToolSurfaceStableKey,
    };

    #[test]
    fn surface_provider_request_can_carry_stable_key_metadata_without_losing_legacy_kind() {
        let stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap();
        let provider_family_id = ProviderFamilyId::new("runenwerk.material_lab").unwrap();

        let request = SurfaceProviderRequest {
            workspace_profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: DocumentId(1),
                document_kind: DocumentKind::MaterialGraph,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(1).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(1).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
            stable_surface_key: stable_surface_key.clone(),
            legacy_tool_surface_kind: Some(ToolSurfaceKind::MaterialGraphCanvas),
            provider_family_id: Some(provider_family_id.clone()),
            surface_route: Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas),
            surface_definition_id: MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            capabilities: SurfaceCapabilitySet::new(true, true, true, false),
        };

        assert_eq!(
            request.legacy_kind(),
            Some(ToolSurfaceKind::MaterialGraphCanvas)
        );
        assert_eq!(request.stable_key(), &stable_surface_key);
        assert_eq!(
            request.provider_family_id.as_ref(),
            Some(&provider_family_id)
        );
        assert_eq!(
            request.surface_route,
            Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas)
        );
        assert!(request.has_capability(SurfaceCapability::RequestMutation));
    }

    #[test]
    fn surface_provider_request_matches_stable_key() {
        let request = material_graph_request_with_stable_key(
            "runenwerk.material_lab.graph_canvas",
            Some(ToolSurfaceKind::MaterialGraphCanvas),
        );

        assert!(request.matches_stable_key("runenwerk.material_lab.graph_canvas"));
        assert!(request.matches_any_stable_key(&[
            "runenwerk.material_lab.inspector",
            "runenwerk.material_lab.graph_canvas"
        ]));
        assert_eq!(
            request.stable_key().as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
    }

    #[test]
    fn mounted_surface_request_requires_stable_key() {
        let request = material_graph_request_with_stable_key(
            "runenwerk.material_lab.graph_canvas",
            Some(ToolSurfaceKind::MaterialGraphCanvas),
        );

        assert_eq!(
            request.stable_surface_key.as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
    }

    #[test]
    fn provider_request_stable_key_helper_returns_authority() {
        let request = material_graph_request_with_stable_key(
            "runenwerk.material_lab.graph_canvas",
            Some(ToolSurfaceKind::MaterialGraphCanvas),
        );

        assert_eq!(
            request.stable_key().as_str(),
            "runenwerk.material_lab.graph_canvas"
        );
    }

    #[test]
    fn surface_provider_request_legacy_kind_is_optional() {
        let request =
            material_graph_request_with_stable_key("runenwerk.material_lab.graph_canvas", None);

        assert!(request.matches_stable_key("runenwerk.material_lab.graph_canvas"));
        assert_eq!(request.legacy_kind(), None);
        assert!(request.legacy_kind_or_error().is_err());
    }

    #[test]
    fn provider_request_legacy_kind_absent_does_not_panic() {
        let request =
            material_graph_request_with_stable_key("runenwerk.material_lab.graph_canvas", None);

        assert_eq!(request.legacy_kind(), None);
        assert!(!request.matches_stable_key("not a valid stable key"));
    }

    #[test]
    fn surface_provider_request_does_not_require_tool_surface_kind_authority() {
        let source = include_str!("surface_provider.rs");
        let request_struct = source
            .split("pub struct SurfaceProviderRequest")
            .nth(1)
            .and_then(|rest| rest.split("impl SurfaceProviderRequest").next())
            .expect("SurfaceProviderRequest source block should exist");

        assert!(request_struct.contains("pub stable_surface_key: ToolSurfaceStableKey"));
        assert!(!request_struct.contains("pub tool_surface_kind: ToolSurfaceKind"));
    }

    #[test]
    fn support_mode_prefers_stable_key_over_legacy() {
        assert_eq!(
            SurfaceProviderSupportMode::LegacyKind.preferred(SurfaceProviderSupportMode::StableKey),
            SurfaceProviderSupportMode::StableKey
        );
        assert_eq!(
            SurfaceProviderSupportMode::Unsupported
                .preferred(SurfaceProviderSupportMode::LegacyKind),
            SurfaceProviderSupportMode::LegacyKind
        );
        assert!(SurfaceProviderSupportMode::StableKey.is_supported());
        assert!(!SurfaceProviderSupportMode::Unsupported.is_supported());
    }

    fn material_graph_request_with_stable_key(
        stable_key: &str,
        legacy_tool_surface_kind: Option<ToolSurfaceKind>,
    ) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: MATERIAL_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: DocumentId(1),
                document_kind: DocumentKind::MaterialGraph,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(1).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(1).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(1).unwrap(),
            stable_surface_key: ToolSurfaceStableKey::new(stable_key).unwrap(),
            legacy_tool_surface_kind,
            provider_family_id: Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap()),
            surface_route: Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas),
            surface_definition_id: MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            capabilities: SurfaceCapabilitySet::new(true, true, true, false),
        }
    }
}
