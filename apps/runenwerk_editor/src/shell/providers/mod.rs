use std::collections::{BTreeMap, BTreeSet};

use editor_core::{DocumentKind, EditorMutationError};
use editor_shell::{
    ConsoleViewModel, ENTITY_TABLE_LIST_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    EditorDomainMutation, EditorShellFrameModel, EntityTableSortKey, ResolvedSurfaceFrame,
    SurfaceCommandProposal, SurfaceDocumentContext, SurfaceLocalAction, SurfaceLocalRoute,
    SurfacePresentationArtifact, SurfacePresentationArtifactKind, SurfaceProviderAvailability,
    SurfaceProviderDescriptor, SurfaceProviderDiagnostic, SurfaceProviderId,
    SurfaceProviderPriority, SurfaceProviderRequest, SurfaceRouteTable, SurfaceSessionMutation,
    ToolSurfaceInstanceId, ToolSurfaceKind, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID, build_console_panel,
    build_entity_table_panel, build_inspector_panel, build_outliner_panel, build_viewport_panel,
    editor_domain_proposal, entity_table_sort_button_widget_id, inspector_field_focus_widget_id,
    inspector_field_widget_id, outliner_row_widget_id, surface_session_proposal,
    tool_surface_definition_id, viewport_product_button_widget_id,
};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{EntityTablePanelPresenter, InspectorPanelPresenter};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
};
use crate::shell::console_adapter::build_console_view_model;
use crate::shell::entity_table_adapter::build_entity_table_view_model;
use crate::shell::inspector_adapter::{
    build_inspector_observation_frame, build_inspector_view_model,
};
use crate::shell::outliner_adapter::{build_outliner_observation_frame, build_outliner_view_model};
use crate::shell::toolbar_adapter::{build_toolbar_observation_frame, build_toolbar_view_model};
use crate::shell::viewport_adapter::{build_viewport_observation_frame, build_viewport_view_model};
use crate::shell::{RunenwerkEditorShellState, SurfaceSessionState};

const SCENE_OUTLINER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(1);
const SCENE_ENTITY_TABLE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(2);
const SCENE_VIEWPORT_PROVIDER_ID: SurfaceProviderId = surface_provider_id(3);
const SCENE_INSPECTOR_PROVIDER_ID: SurfaceProviderId = surface_provider_id(4);
const CONSOLE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(5);

const fn surface_provider_id(raw: u64) -> SurfaceProviderId {
    match SurfaceProviderId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("surface provider id constants must be non-zero"),
    }
}

pub struct SurfaceProviderBuildContext<'a> {
    pub app: &'a RunenwerkEditorApp,
    pub shell_state: &'a RunenwerkEditorShellState,
    pub theme: &'a ThemeTokens,
    pub viewport_observations: Option<&'a ViewportArtifactObservationResource>,
    pub tool_surface_bindings: Option<&'a ToolSurfaceRuntimeBindingRegistryResource>,
}

pub struct SurfaceProviderDispatchContext<'a> {
    pub app: &'a RunenwerkEditorApp,
    pub shell_state: &'a RunenwerkEditorShellState,
    pub projection_epoch: u64,
}

pub trait EditorSurfaceProvider: Send + Sync {
    fn descriptor(&self) -> SurfaceProviderDescriptor;
    fn supports(&self, request: &SurfaceProviderRequest) -> bool;
    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic>;
    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic>;
}

#[derive(Debug)]
pub enum SurfaceProviderRegistryError {
    DuplicateProviderId(SurfaceProviderId),
}

pub struct ProviderSurfaceFrame {
    pub title: String,
    pub artifact: SurfacePresentationArtifact,
    pub routes: SurfaceRouteTable,
}

pub struct EditorSurfaceProviderRegistry {
    providers: Vec<Box<dyn EditorSurfaceProvider>>,
}

impl EditorSurfaceProviderRegistry {
    pub fn new(
        providers: Vec<Box<dyn EditorSurfaceProvider>>,
    ) -> Result<Self, SurfaceProviderRegistryError> {
        let mut ids = BTreeSet::new();
        for provider in &providers {
            let id = provider.descriptor().id;
            if !ids.insert(id) {
                return Err(SurfaceProviderRegistryError::DuplicateProviderId(id));
            }
        }
        Ok(Self { providers })
    }

    pub fn runenwerk_default() -> Self {
        Self::new(vec![
            Box::new(SceneOutlinerProvider),
            Box::new(SceneEntityTableProvider),
            Box::new(SceneViewportProvider),
            Box::new(SceneInspectorProvider),
            Box::new(ConsoleProvider),
        ])
        .expect("default surface providers must have unique ids")
    }

    pub fn resolve_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> ResolvedSurfaceFrame {
        if !workspace_allows_document(context.shell_state, request) {
            return unsupported_frame(
                request,
                "Unsupported Document",
                "editor.surface.unsupported_document",
                "workspace profile does not allow the active document kind",
            );
        }

        let supported = self
            .providers
            .iter()
            .map(Box::as_ref)
            .filter(|provider| provider.supports(request))
            .collect::<Vec<_>>();
        if supported.is_empty() {
            return unsupported_frame(
                request,
                "Unsupported Surface",
                "editor.surface.unsupported_provider",
                "no provider supports this surface request",
            );
        }
        let Some(provider) = deterministic_provider(supported) else {
            return diagnostic_frame(
                request,
                "Ambiguous Surface",
                SurfaceProviderAvailability::Ambiguous,
                SurfacePresentationArtifactKind::Ambiguous,
                SurfaceProviderDiagnostic::new(
                    "editor.surface.ambiguous_provider",
                    "multiple providers support this request at the same priority",
                ),
            );
        };
        let descriptor = provider.descriptor();
        match provider.build_frame(context, request, session) {
            Ok(frame) => ResolvedSurfaceFrame {
                surface_instance_id: request.tool_surface_instance_id,
                panel_instance_id: request.panel_instance_id,
                tab_stack_id: request.tab_stack_id,
                surface_kind: request.tool_surface_kind,
                surface_definition_id: request.surface_definition_id,
                provider_id: Some(descriptor.id),
                title: frame.title,
                artifact: frame.artifact,
                routes: frame.routes,
                availability: SurfaceProviderAvailability::Available,
            },
            Err(diagnostic) => diagnostic_frame(
                request,
                descriptor.label,
                SurfaceProviderAvailability::Error,
                SurfacePresentationArtifactKind::Error,
                diagnostic,
            ),
        }
    }

    pub fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        provider_id: SurfaceProviderId,
        action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, EditorMutationError> {
        let provider = self
            .providers
            .iter()
            .find(|provider| provider.descriptor().id == provider_id)
            .ok_or(EditorMutationError::session_rejected(
                "surface provider id is not registered",
            ))?;
        if !provider.supports(request) {
            return Err(EditorMutationError::session_rejected(
                "surface provider id does not support target surface",
            ));
        }
        provider
            .map_action(context, request, action)
            .map_err(|_| EditorMutationError::session_rejected("surface provider action failed"))
    }
}

pub fn build_editor_shell_frame_model(
    app: &RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
    registry: &EditorSurfaceProviderRegistry,
    theme: &ThemeTokens,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
) -> EditorShellFrameModel {
    let scene_version = app.runtime().current_scene_reality_version();
    let session = app.runtime().session_reality();
    let history = session.history();
    let toolbar_frame = build_toolbar_observation_frame(
        session.active_tool(),
        history.can_undo(),
        history.can_redo(),
        app.debug_logs_enabled(),
        scene_version,
    );

    let context = SurfaceProviderBuildContext {
        app,
        shell_state,
        theme,
        viewport_observations,
        tool_surface_bindings,
    };
    let document_context = active_document_context(app);
    let mut surfaces = BTreeMap::new();
    for request in mounted_surface_requests(shell_state, document_context) {
        let session = app
            .surface_sessions()
            .session_or_default(request.tool_surface_instance_id);
        let frame = registry.resolve_frame(&context, &request, &session);
        surfaces.insert(request.tool_surface_instance_id, frame);
    }

    EditorShellFrameModel::new(build_toolbar_view_model(&toolbar_frame), surfaces)
}

pub fn mounted_surface_requests(
    shell_state: &RunenwerkEditorShellState,
    document_context: SurfaceDocumentContext,
) -> Vec<SurfaceProviderRequest> {
    shell_state
        .workspace_state()
        .panels()
        .filter_map(|panel| {
            let surface_id = panel.active_tool_surface?;
            let surface = shell_state.workspace_state().tool_surface(surface_id)?;
            let tab_stack_id = shell_state
                .workspace_state()
                .tab_stacks()
                .find(|stack| stack.ordered_panels.contains(&panel.id))
                .map(|stack| stack.id)?;
            Some(SurfaceProviderRequest {
                workspace_profile_id: shell_state.active_workspace_profile_id(),
                document_context: document_context.clone(),
                panel_instance_id: panel.id,
                tab_stack_id,
                tool_surface_instance_id: surface.id,
                tool_surface_kind: surface.tool_surface_kind,
                surface_definition_id: tool_surface_definition_id(surface.tool_surface_kind),
            })
        })
        .collect()
}

pub fn active_document_context(app: &RunenwerkEditorApp) -> SurfaceDocumentContext {
    let active_document = app.runtime().session().active_document();
    if let Some(document_id) = active_document {
        if let Some(document) = app.runtime().session().document(document_id) {
            return SurfaceDocumentContext::Resolved {
                document_id,
                document_kind: document.kind.clone(),
            };
        }
        return SurfaceDocumentContext::Unresolved { document_id };
    }
    SurfaceDocumentContext::NoActiveDocument
}

fn workspace_allows_document(
    shell_state: &RunenwerkEditorShellState,
    request: &SurfaceProviderRequest,
) -> bool {
    let registry = editor_shell::default_workspace_profile_registry();
    if request.tool_surface_kind == ToolSurfaceKind::Console {
        return true;
    }
    let Some(document_kind) = request.document_context.resolved_document_kind() else {
        return false;
    };
    registry
        .profile(shell_state.active_workspace_profile_id())
        .map(|profile| profile.document_kind_filters.contains(document_kind))
        .unwrap_or(false)
}

fn deterministic_provider(
    mut supported: Vec<&dyn EditorSurfaceProvider>,
) -> Option<&dyn EditorSurfaceProvider> {
    supported.sort_by_key(|provider| provider.descriptor().priority);
    let first = supported[0];
    if supported.len() == 1 {
        return Some(first);
    }
    if supported[0].descriptor().priority == supported[1].descriptor().priority {
        return None;
    }
    Some(first)
}

fn unsupported_frame(
    request: &SurfaceProviderRequest,
    title: &str,
    code: &'static str,
    message: &str,
) -> ResolvedSurfaceFrame {
    diagnostic_frame(
        request,
        title,
        SurfaceProviderAvailability::Unsupported,
        SurfacePresentationArtifactKind::Unsupported,
        SurfaceProviderDiagnostic::new(code, message),
    )
}

fn diagnostic_frame(
    request: &SurfaceProviderRequest,
    title: impl Into<String>,
    availability: SurfaceProviderAvailability,
    kind: SurfacePresentationArtifactKind,
    diagnostic: SurfaceProviderDiagnostic,
) -> ResolvedSurfaceFrame {
    let root = diagnostic_surface_node(request, &diagnostic);
    ResolvedSurfaceFrame::diagnostic(
        request,
        title,
        availability,
        SurfacePresentationArtifact::diagnostic(kind, root, diagnostic),
    )
}

fn diagnostic_surface_node(
    request: &SurfaceProviderRequest,
    diagnostic: &SurfaceProviderDiagnostic,
) -> editor_shell::UiNode {
    let root_id = surface_scoped_widget_id(request.tool_surface_instance_id, 900_000);
    let label_id = surface_scoped_widget_id(request.tool_surface_instance_id, 900_001);
    editor_shell::panel(
        root_id,
        ThemeTokens::default(),
        vec![editor_shell::label(
            label_id,
            format!("Unsupported surface: {}", diagnostic.message),
            ThemeTokens::default().body_small_text_style(FontId(1)),
        )],
    )
}

fn surface_scoped_widget_id(
    surface_id: ToolSurfaceInstanceId,
    base: u64,
) -> editor_shell::WidgetId {
    if surface_id.raw() <= 5 {
        editor_shell::WidgetId(base)
    } else {
        editor_shell::WidgetId(surface_id.raw().saturating_mul(100_000_000) + base)
    }
}

fn remap_surface_node_ids(
    mut node: editor_shell::UiNode,
    surface_id: ToolSurfaceInstanceId,
) -> editor_shell::UiNode {
    if surface_id.raw() <= 5 {
        return node;
    }
    remap_node_recursive(&mut node, surface_id);
    node
}

fn remap_node_recursive(node: &mut editor_shell::UiNode, surface_id: ToolSurfaceInstanceId) {
    node.id = surface_scoped_widget_id(surface_id, node.id.0);
    for child in &mut node.children {
        remap_node_recursive(child, surface_id);
    }
}

fn remap_widget_id(
    surface_id: ToolSurfaceInstanceId,
    widget_id: editor_shell::WidgetId,
) -> editor_shell::WidgetId {
    surface_scoped_widget_id(surface_id, widget_id.0)
}

struct SceneOutlinerProvider;
struct SceneEntityTableProvider;
struct SceneViewportProvider;
struct SceneInspectorProvider;
struct ConsoleProvider;

impl EditorSurfaceProvider for SceneOutlinerProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_OUTLINER_PROVIDER_ID,
            "Scene Outliner",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::Outliner
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let state = context.app.outliner_state();
        let frame = build_outliner_observation_frame(
            &state,
            context.app.runtime().current_scene_reality_version(),
        );
        let view_model = build_outliner_view_model(&frame);
        let root = remap_surface_node_ids(
            build_outliner_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        for (index, row) in view_model.rows.iter().enumerate() {
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    outliner_row_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::SelectOutlinerEntity {
                    entity: row.entity,
                }),
            );
        }
        Ok(ProviderSurfaceFrame {
            title: "Outliner".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic> {
        match action {
            SurfaceLocalAction::SelectOutlinerEntity { entity } => Ok(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::SelectOutlinerEntity { entity },
            )),
            _ => Ok(SurfaceCommandProposal::NoOp),
        }
    }
}

impl EditorSurfaceProvider for SceneEntityTableProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_ENTITY_TABLE_PROVIDER_ID,
            "Scene Entity Table",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::EntityTable
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let state = EntityTablePanelPresenter::build_state(
            context.app.runtime(),
            &session.entity_table_ui_state,
        );
        let view_model = build_entity_table_view_model(&state);
        let root = remap_surface_node_ids(
            build_entity_table_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_LIST_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::SelectEntityTableRow {
                entities: view_model.rows.iter().map(|row| row.entity).collect(),
            }),
        );
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                ENTITY_TABLE_SEARCH_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::AppendEntityTableSearchText {
                text: String::new(),
            }),
        );
        for (index, sort_key) in [
            EntityTableSortKey::EntityId,
            EntityTableSortKey::DisplayName,
            EntityTableSortKey::Parent,
            EntityTableSortKey::ComponentCount,
        ]
        .into_iter()
        .enumerate()
        {
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    entity_table_sort_button_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::ToggleEntityTableSort { sort_key }),
            );
        }
        Ok(ProviderSurfaceFrame {
            title: "Entities".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic> {
        let projection_epoch = context.projection_epoch;
        match action {
            SurfaceLocalAction::SelectEntityTableEntity { entity } => Ok(editor_domain_proposal(
                request,
                projection_epoch,
                EditorDomainMutation::SelectEntityTableRow {
                    entities: vec![entity],
                },
            )),
            SurfaceLocalAction::SelectEntityTableRow { entities } => Ok(editor_domain_proposal(
                request,
                projection_epoch,
                EditorDomainMutation::SelectEntityTableRow { entities },
            )),
            SurfaceLocalAction::AppendEntityTableSearchText { text } => {
                Ok(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::AppendEntityTableSearchText { text },
                ))
            }
            SurfaceLocalAction::BackspaceEntityTableSearch => Ok(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::BackspaceEntityTableSearch,
            )),
            SurfaceLocalAction::ToggleEntityTableSort { sort_key } => Ok(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::ToggleEntityTableSort { sort_key },
            )),
            _ => Ok(SurfaceCommandProposal::NoOp),
        }
    }
}

impl EditorSurfaceProvider for SceneViewportProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_VIEWPORT_PROVIDER_ID,
            "Scene Viewport",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::Viewport
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let products = context
            .tool_surface_bindings
            .and_then(|bindings| {
                bindings.binding_for_tool_surface(request.tool_surface_instance_id)
            })
            .and_then(|binding| {
                context
                    .viewport_observations
                    .and_then(|observations| observations.frame_for(binding.viewport_id))
            });
        let tool_state = context.app.viewport_tool_state();
        let frame = build_viewport_observation_frame(
            products,
            session.viewport_details_visible,
            context.app.runtime().selected_entity(),
            session.viewport_interaction_state.drag_in_progress(),
            tool_state,
            context.app.runtime().current_scene_reality_version(),
        );
        let view_model = build_viewport_view_model(&frame);
        let root = remap_surface_node_ids(
            build_viewport_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        routes.insert(
            remap_widget_id(
                request.tool_surface_instance_id,
                VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
            ),
            SurfaceLocalRoute::new(SurfaceLocalAction::ToggleViewportDetails),
        );
        for (index, choice) in view_model.product_choices.iter().enumerate() {
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    viewport_product_button_widget_id(index),
                ),
                SurfaceLocalRoute::new(SurfaceLocalAction::SelectViewportProduct {
                    viewport_id: choice.viewport_id,
                    product_id: choice.product_id,
                    enabled: choice.enabled,
                }),
            );
        }
        Ok(ProviderSurfaceFrame {
            title: "Viewport".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic> {
        match action {
            SurfaceLocalAction::SelectViewportProduct {
                viewport_id,
                product_id,
                enabled,
            } if enabled => Ok(editor_domain_proposal(
                request,
                context.projection_epoch,
                EditorDomainMutation::SelectViewportProduct {
                    viewport_id,
                    product_id,
                },
            )),
            SurfaceLocalAction::ToggleViewportDetails => Ok(surface_session_proposal(
                request,
                context.projection_epoch,
                SurfaceSessionMutation::ToggleViewportDetails,
            )),
            _ => Ok(SurfaceCommandProposal::NoOp),
        }
    }
}

impl EditorSurfaceProvider for SceneInspectorProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SCENE_INSPECTOR_PROVIDER_ID,
            "Scene Inspector",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        matches!(
            request.document_context.resolved_document_kind(),
            Some(DocumentKind::Scene)
        ) && request.tool_surface_kind == ToolSurfaceKind::Inspector
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let panel_model = InspectorPanelPresenter::build_view_model(
            context.app.runtime(),
            &session.inspector_ui_state,
        );
        let frame = build_inspector_observation_frame(
            &panel_model,
            context.app.runtime().current_scene_reality_version(),
        );
        let view_model = build_inspector_view_model(&frame);
        let root = remap_surface_node_ids(
            build_inspector_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        let mut routes = SurfaceRouteTable::empty();
        for (index, field) in view_model.fields.iter().enumerate() {
            let action = if field.editable {
                SurfaceLocalAction::EditInspectorFieldText {
                    index,
                    text: String::new(),
                }
            } else {
                SurfaceLocalAction::ActivateInspectorField { index }
            };
            routes.insert(
                remap_widget_id(
                    request.tool_surface_instance_id,
                    inspector_field_widget_id(index),
                ),
                SurfaceLocalRoute::new(action),
            );
            if field.editable {
                routes.insert(
                    remap_widget_id(
                        request.tool_surface_instance_id,
                        inspector_field_focus_widget_id(index),
                    ),
                    SurfaceLocalRoute::new(SurfaceLocalAction::FocusInspectorField { index }),
                );
            }
        }
        Ok(ProviderSurfaceFrame {
            title: "Inspector".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic> {
        let projection_epoch = context.projection_epoch;
        match action {
            SurfaceLocalAction::ActivateInspectorField { index } => Ok(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::ActivateInspectorField { index },
            )),
            SurfaceLocalAction::FocusInspectorField { index } => Ok(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::FocusInspectorField { index },
            )),
            SurfaceLocalAction::EditInspectorFieldText { index, text } => {
                Ok(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::AppendInspectorFieldText { index, text },
                ))
            }
            SurfaceLocalAction::BackspaceInspectorFieldText { index } => {
                Ok(surface_session_proposal(
                    request,
                    projection_epoch,
                    SurfaceSessionMutation::BackspaceInspectorFieldText { index },
                ))
            }
            SurfaceLocalAction::CommitInspectorFieldText { index } => Ok(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::CommitInspectorFieldText { index },
            )),
            SurfaceLocalAction::CancelInspectorFieldText { index } => Ok(surface_session_proposal(
                request,
                projection_epoch,
                SurfaceSessionMutation::CancelInspectorFieldText { index },
            )),
            _ => Ok(SurfaceCommandProposal::NoOp),
        }
    }
}

impl EditorSurfaceProvider for ConsoleProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            CONSOLE_PROVIDER_ID,
            "Console",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::Console
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let view_model: ConsoleViewModel = build_console_view_model(context.app.console_lines());
        let root = remap_surface_node_ids(
            build_console_panel(
                &view_model,
                context.theme,
                request.panel_instance_id,
                Some(request.tool_surface_instance_id),
            ),
            request.tool_surface_instance_id,
        );
        Ok(ProviderSurfaceFrame {
            title: "Console".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes: SurfaceRouteTable::empty(),
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _action: SurfaceLocalAction,
    ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic> {
        Ok(SurfaceCommandProposal::NoOp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{
        LAYOUT_WORKSPACE_PROFILE_ID, PanelInstanceId, TabStackId, ToolSurfaceInstanceId,
        VIEWPORT_SURFACE_DEFINITION_ID, WidgetId,
    };

    struct DummyProvider {
        descriptor: SurfaceProviderDescriptor,
        supports: bool,
        fail: bool,
    }

    impl EditorSurfaceProvider for DummyProvider {
        fn descriptor(&self) -> SurfaceProviderDescriptor {
            self.descriptor.clone()
        }

        fn supports(&self, _request: &SurfaceProviderRequest) -> bool {
            self.supports
        }

        fn build_frame(
            &self,
            _context: &SurfaceProviderBuildContext<'_>,
            _request: &SurfaceProviderRequest,
            _session: &SurfaceSessionState,
        ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
            if self.fail {
                return Err(SurfaceProviderDiagnostic::new(
                    "test.provider.failed",
                    "provider failed",
                ));
            }
            Ok(ProviderSurfaceFrame {
                title: self.descriptor.label.clone(),
                artifact: SurfacePresentationArtifact::provider(editor_shell::label(
                    WidgetId(99),
                    "ok",
                    ThemeTokens::default().body_small_text_style(FontId(1)),
                )),
                routes: SurfaceRouteTable::empty(),
            })
        }

        fn map_action(
            &self,
            _context: &SurfaceProviderDispatchContext<'_>,
            _request: &SurfaceProviderRequest,
            _action: SurfaceLocalAction,
        ) -> Result<SurfaceCommandProposal, SurfaceProviderDiagnostic> {
            Ok(SurfaceCommandProposal::NoOp)
        }
    }

    fn dummy(id: u64, priority: u16, supports: bool) -> Box<dyn EditorSurfaceProvider> {
        Box::new(DummyProvider {
            descriptor: SurfaceProviderDescriptor::new(
                SurfaceProviderId::try_from_raw(id).unwrap(),
                format!("provider-{id}"),
                SurfaceProviderPriority(priority),
            ),
            supports,
            fail: false,
        })
    }

    fn failing(id: u64) -> Box<dyn EditorSurfaceProvider> {
        Box::new(DummyProvider {
            descriptor: SurfaceProviderDescriptor::new(
                SurfaceProviderId::try_from_raw(id).unwrap(),
                "failing",
                SurfaceProviderPriority::DEFAULT,
            ),
            supports: true,
            fail: true,
        })
    }

    fn request() -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: LAYOUT_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(1),
                document_kind: DocumentKind::Scene,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(3).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(3).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(3).unwrap(),
            tool_surface_kind: ToolSurfaceKind::Viewport,
            surface_definition_id: VIEWPORT_SURFACE_DEFINITION_ID,
        }
    }

    fn request_with_document_context(
        document_context: SurfaceDocumentContext,
        tool_surface_kind: ToolSurfaceKind,
    ) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            document_context,
            tool_surface_kind,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            ..request()
        }
    }

    fn context<'a>(
        app: &'a RunenwerkEditorApp,
        shell_state: &'a RunenwerkEditorShellState,
        theme: &'a ThemeTokens,
    ) -> SurfaceProviderBuildContext<'a> {
        SurfaceProviderBuildContext {
            app,
            shell_state,
            theme,
            viewport_observations: None,
            tool_surface_bindings: None,
        }
    }

    #[test]
    fn duplicate_provider_id_is_rejected() {
        let error =
            match EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true), dummy(1, 90, true)])
            {
                Ok(_) => panic!("duplicate ids should be rejected"),
                Err(error) => error,
            };

        assert!(matches!(
            error,
            SurfaceProviderRegistryError::DuplicateProviderId(id) if id == SurfaceProviderId::try_from_raw(1).unwrap()
        ));
    }

    #[test]
    fn ambiguous_provider_support_fails_closed_with_zero_routes() {
        let registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true), dummy(2, 100, true)])
                .expect("ids are unique");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request(),
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Ambiguous);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn explicit_priority_resolves_deterministically() {
        let registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true), dummy(2, 10, true)])
                .expect("ids are unique");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request(),
            &Default::default(),
        );

        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(2).unwrap())
        );
        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    }

    #[test]
    fn provider_error_artifact_has_diagnostic_and_zero_routes() {
        let registry = EditorSurfaceProviderRegistry::new(vec![failing(1)]).expect("id is unique");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request(),
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Error);
        assert!(!frame.artifact.diagnostics.is_empty());
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn unsupported_provider_artifact_has_zero_routes() {
        let registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, false)]).expect("id is unique");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request(),
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn no_active_document_does_not_resolve_scene_provider() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = request_with_document_context(
            SurfaceDocumentContext::NoActiveDocument,
            ToolSurfaceKind::Viewport,
        );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn unresolved_document_returns_diagnostic_without_routes() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = request_with_document_context(
            SurfaceDocumentContext::Unresolved {
                document_id: editor_core::DocumentId(99),
            },
            ToolSurfaceKind::Inspector,
        );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert!(!frame.artifact.diagnostics.is_empty());
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn console_provider_resolves_without_active_scene_document() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = request_with_document_context(
            SurfaceDocumentContext::NoActiveDocument,
            ToolSurfaceKind::Console,
        );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
    }
}
