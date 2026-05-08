use std::collections::{BTreeMap, BTreeSet};

use editor_core::{DocumentKind, EditorMutationError, EntityId, RealityVersion};
use editor_inspector::InspectorValue;
use editor_shell::{
    ConsoleViewModel, ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
    ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID, ENTITY_TABLE_LIST_WIDGET_ID,
    ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID, EditorDefinitionSurfaceAction,
    EditorDomainMutation, EditorShellFrameModel, EntityTableDomainMutation,
    EntityTableRowViewModel, EntityTableSessionMutation, EntityTableSortKey,
    EntityTableSurfaceAction, EntityTableViewModel, InspectorFieldControlKind,
    InspectorFieldViewModel, InspectorObservationFrame, InspectorObservedField,
    InspectorObservedTarget, InspectorSessionMutation, InspectorSurfaceAction,
    InspectorTargetViewModel, InspectorViewModel, ObservationConsumerKind,
    ObservationFrameMetadata, ObservationSourceReality, OutlinerDomainMutation,
    OutlinerObservationFrame, OutlinerObservedRow, OutlinerRowViewModel, OutlinerSurfaceAction,
    OutlinerViewModel, ResolvedSurfaceFrame, ShellCommand, SurfaceCommandProposal,
    SurfaceDocumentContext, SurfaceLocalAction, SurfaceLocalRoute, SurfacePresentationArtifact,
    SurfacePresentationArtifactKind, SurfaceProviderAvailability, SurfaceProviderDescriptor,
    SurfaceProviderDiagnostic, SurfaceProviderId, SurfaceProviderPriority, SurfaceProviderRequest,
    SurfaceRouteTable, SurfaceSessionMutation, ToolSurfaceInstanceId, ToolSurfaceKind,
    VIEWPORT_DETAILS_TOGGLE_WIDGET_ID, VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
    VIEWPORT_RESET_CAMERA_WIDGET_ID, VIEWPORT_ROOT_OPAQUE_TOGGLE_WIDGET_ID,
    VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID, ViewportDomainMutation, ViewportObservationFrame,
    ViewportProductChoiceViewModel, ViewportProductObservation, ViewportSessionMutation,
    ViewportSurfaceAction, ViewportViewModel, build_console_panel, build_entity_table_panel,
    build_inspector_panel, build_outliner_panel, build_viewport_panel, editor_domain_proposal,
    entity_table_sort_button_widget_id, inspector_field_focus_widget_id, inspector_field_widget_id,
    outliner_row_widget_id, surface_session_proposal, tool_surface_capability_set,
    tool_surface_definition_id, viewport_debug_stage_button_widget_id,
    viewport_product_button_widget_id,
};
use editor_viewport::{ArtifactObservationFrame, ProducerHealth, ProductAvailabilityState};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::{
    EntityTablePanelPresenter, EntityTablePanelState, InspectorPanelPresenter,
    InspectorPanelViewModel, InspectorWidgetField, OutlinerPanelState, ViewportToolState,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource,
};
use crate::shell::toolbar_adapter::{build_toolbar_observation_frame, build_toolbar_view_model};
use crate::shell::{RunenwerkEditorShellState, SurfaceSessionState};

const SCENE_OUTLINER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(1);
const SCENE_ENTITY_TABLE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(2);
const SCENE_VIEWPORT_PROVIDER_ID: SurfaceProviderId = surface_provider_id(3);
const SCENE_INSPECTOR_PROVIDER_ID: SurfaceProviderId = surface_provider_id(4);
const CONSOLE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(5);
const SELF_AUTHORING_PROVIDER_ID: SurfaceProviderId = surface_provider_id(6);

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
    pub viewport_instances: Option<&'a ViewportInstanceRegistryResource>,
}

pub struct SurfaceProviderDispatchContext<'a> {
    pub projection_epoch: u64,
    pub _marker: std::marker::PhantomData<&'a ()>,
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
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic>;
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
            Box::new(SelfAuthoringProvider),
        ])
        .expect("default surface providers must have unique ids")
    }

    pub fn resolve_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> ResolvedSurfaceFrame {
        if !workspace_allows_document(request) {
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
    ) -> Result<Option<SurfaceCommandProposal>, EditorMutationError> {
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
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
) -> EditorShellFrameModel {
    let scene_version = app.runtime().current_scene_reality_version();
    let session = app.runtime().session_reality();
    let history = session.history();
    let toolbar_frame = build_toolbar_observation_frame(
        session.active_tool(),
        history.can_undo(),
        history.can_redo(),
        app.debug_logs_enabled(),
        shell_state.active_toolbar_menu(),
        shell_state.active_workspace_profile_id(),
        shell_state.open_workspace_profile_ids(),
        scene_version,
    );

    let context = SurfaceProviderBuildContext {
        app,
        shell_state,
        theme,
        viewport_observations,
        tool_surface_bindings,
        viewport_instances,
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
        .with_active_tab_stack_popup_menu(shell_state.active_tab_stack_popup_menu())
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
                capabilities: tool_surface_capability_set(surface.tool_surface_kind),
            })
        })
        .collect()
}

pub fn active_document_context(app: &RunenwerkEditorApp) -> SurfaceDocumentContext {
    let active_document = app.runtime().session().active_document();
    if let Some(document_id) = active_document {
        if let Some(document) = app.runtime().session().active_document_descriptor() {
            return SurfaceDocumentContext::Resolved {
                document_id,
                document_kind: document.kind.clone(),
            };
        }
        return SurfaceDocumentContext::Unresolved { document_id };
    }
    SurfaceDocumentContext::NoActiveDocument
}

fn build_console_view_model(lines: &[String]) -> ConsoleViewModel {
    ConsoleViewModel {
        lines: lines.to_vec(),
    }
}

fn build_outliner_observation_frame(
    state: &OutlinerPanelState,
    source_version: RealityVersion,
) -> OutlinerObservationFrame {
    OutlinerObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Outliner,
            source_version,
        ),
        rows: state
            .rows
            .iter()
            .map(|row| OutlinerObservedRow {
                entity: row.entity,
                display_name: row.display_name.clone(),
                depth: row.depth,
                has_children: row.has_children,
                is_selected: state.selected_entity == Some(row.entity),
            })
            .collect(),
    }
}

fn build_outliner_view_model(frame: &OutlinerObservationFrame) -> OutlinerViewModel {
    OutlinerViewModel {
        rows: frame
            .rows
            .iter()
            .map(|row| OutlinerRowViewModel {
                entity: row.entity,
                display_name: row.display_name.clone(),
                depth: row.depth,
                has_children: row.has_children,
                is_selected: row.is_selected,
            })
            .collect(),
    }
}

fn build_entity_table_view_model(state: &EntityTablePanelState) -> EntityTableViewModel {
    EntityTableViewModel {
        query: state.query.clone(),
        search_query: state.search_query.clone(),
        sort_key: state.sort_key,
        sort_ascending: state.sort_ascending,
        component_filters: state.component_filters.clone(),
        rows: state
            .rows
            .iter()
            .map(|row| EntityTableRowViewModel {
                entity: row.entity,
                entity_id_label: row.entity.0.to_string(),
                display_name: row.display_name.clone(),
                parent_label: row
                    .parent
                    .map(|parent| parent.0.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                component_count: row.component_count,
                is_selected: row.is_selected,
            })
            .collect(),
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "viewport observation projection keeps source provider state explicit at the boundary"
)]
fn build_viewport_observation_frame(
    products: Option<&ArtifactObservationFrame>,
    details_visible: bool,
    statistics_visible: bool,
    options_menu_open: bool,
    debug_stage: editor_viewport::ViewportDebugStage,
    root_background_opaque: bool,
    selected_entity: Option<EntityId>,
    drag_in_progress: bool,
    tool_state: ViewportToolState,
    source_version: RealityVersion,
    fallback_viewport_id: Option<editor_viewport::ViewportId>,
) -> ViewportObservationFrame {
    let viewport_id = products
        .map(|value| value.viewport_id)
        .or(fallback_viewport_id);
    let selected_primary_product_id = products.and_then(|value| value.selected_primary_product_id);
    let products = products
        .map(|value| {
            value
                .available_products
                .iter()
                .map(|descriptor| {
                    let availability = value
                        .availability_by_product
                        .get(&descriptor.id)
                        .copied()
                        .unwrap_or(ProductAvailabilityState::Unavailable);
                    let producer_health = value
                        .producer_health_by_product
                        .get(&descriptor.id)
                        .copied()
                        .unwrap_or(ProducerHealth::Unavailable);
                    ViewportProductObservation {
                        viewport_id: value.viewport_id,
                        product_id: descriptor.id,
                        product_kind: descriptor.kind,
                        label: format!("{:?}", descriptor.kind),
                        freshness: descriptor.freshness,
                        availability,
                        producer_health,
                        is_selected_primary: value.selected_primary_product_id
                            == Some(descriptor.id),
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ViewportObservationFrame {
        metadata: ObservationFrameMetadata::strict_current(
            ObservationSourceReality::ObservedScene,
            ObservationConsumerKind::Viewport,
            source_version,
        ),
        viewport_id,
        selected_primary_product_id,
        products,
        details_visible,
        statistics_visible,
        options_menu_open,
        debug_stage,
        root_background_opaque,
        selected_entity,
        hovered_entity: tool_state.hovered_entity,
        drag_in_progress,
        preview_active: tool_state.active_preview.is_some(),
    }
}

fn build_viewport_view_model(frame: &ViewportObservationFrame) -> ViewportViewModel {
    ViewportViewModel {
        viewport_id: frame.viewport_id,
        selected_primary_product_id: frame.selected_primary_product_id,
        product_choices: frame
            .products
            .iter()
            .map(|product| ViewportProductChoiceViewModel {
                viewport_id: product.viewport_id,
                product_id: product.product_id,
                label: format!(
                    "{:?} [{:?}/{:?}]",
                    product.product_kind, product.availability, product.producer_health
                ),
                selected: product.is_selected_primary,
                enabled: product.availability == ProductAvailabilityState::Available,
            })
            .collect::<Vec<_>>(),
        details_visible: frame.details_visible,
        statistics_visible: frame.statistics_visible,
        options_menu_open: frame.options_menu_open,
        debug_stage: frame.debug_stage,
        root_background_opaque: frame.root_background_opaque,
        selected_entity: frame.selected_entity,
        hovered_entity: frame.hovered_entity,
        drag_in_progress: frame.drag_in_progress,
        preview_active: frame.preview_active,
    }
}

fn build_inspector_observation_frame(
    view_model: &InspectorPanelViewModel,
    source_version: RealityVersion,
) -> InspectorObservationFrame {
    let metadata = ObservationFrameMetadata::strict_current(
        ObservationSourceReality::ObservedScene,
        ObservationConsumerKind::Inspector,
        source_version,
    );

    match view_model {
        InspectorPanelViewModel::Empty => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Empty,
            fields: Vec::new(),
        },
        InspectorPanelViewModel::Entity {
            display_name,
            components,
            available_component_types,
            ..
        } => {
            InspectorObservationFrame {
                metadata,
                target: InspectorObservedTarget::Entity {
                    display_name: display_name.clone(),
                },
                fields: components
                    .iter()
                    .map(|component| InspectorObservedField {
                        label: component.display_name.clone(),
                        path_key: None,
                        value_summary: if component.is_selected {
                            "selected".to_string()
                        } else {
                            "attached".to_string()
                        },
                        control: InspectorFieldControlKind::ReadOnly,
                        is_focused: false,
                        editable: false,
                    })
                    .chain(available_component_types.iter().map(|component| {
                        InspectorObservedField {
                            label: format!("+ {}", component.display_name),
                            path_key: None,
                            value_summary: if component.already_attached {
                                "already attached".to_string()
                            } else {
                                "available".to_string()
                            },
                            control: InspectorFieldControlKind::ReadOnly,
                            is_focused: false,
                            editable: false,
                        }
                    }))
                    .collect(),
            }
        }
        InspectorPanelViewModel::Component {
            entity_display_name,
            component_display_name,
            widget_fields,
            ..
        } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Component {
                entity_display_name: entity_display_name.clone(),
                component_display_name: component_display_name.clone(),
            },
            fields: widget_fields
                .iter()
                .map(build_inspector_observed_field)
                .collect(),
        },
        InspectorPanelViewModel::Resource { resource_type } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Resource {
                display_name: format!("Resource {}", resource_type.0),
            },
            fields: Vec::new(),
        },
        InspectorPanelViewModel::Unsupported { target } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Unsupported {
                label: target.clone(),
            },
            fields: Vec::new(),
        },
        InspectorPanelViewModel::Error { message } => InspectorObservationFrame {
            metadata,
            target: InspectorObservedTarget::Error {
                message: message.clone(),
            },
            fields: Vec::new(),
        },
    }
}

fn build_inspector_view_model(frame: &InspectorObservationFrame) -> InspectorViewModel {
    let target = match &frame.target {
        InspectorObservedTarget::Empty => InspectorTargetViewModel::Empty,
        InspectorObservedTarget::Entity { display_name } => InspectorTargetViewModel::Entity {
            display_name: display_name.clone(),
        },
        InspectorObservedTarget::Component {
            entity_display_name,
            component_display_name,
        } => InspectorTargetViewModel::Component {
            entity_display_name: entity_display_name.clone(),
            component_display_name: component_display_name.clone(),
        },
        InspectorObservedTarget::Resource { display_name } => InspectorTargetViewModel::Resource {
            display_name: display_name.clone(),
        },
        InspectorObservedTarget::Unsupported { label } => InspectorTargetViewModel::Unsupported {
            label: label.clone(),
        },
        InspectorObservedTarget::Error { message } => InspectorTargetViewModel::Error {
            message: message.clone(),
        },
    };

    InspectorViewModel {
        target,
        fields: frame
            .fields
            .iter()
            .map(|field| InspectorFieldViewModel {
                label: field.label.clone(),
                path_key: field.path_key.clone(),
                value_summary: field.value_summary.clone(),
                control: field.control.clone(),
                is_focused: field.is_focused,
                editable: field.editable,
            })
            .collect(),
    }
}

fn build_inspector_observed_field(field: &InspectorWidgetField) -> InspectorObservedField {
    let value_text = field
        .draft_text
        .clone()
        .unwrap_or_else(|| inspector_value_summary(&field.value));

    let control = inspector_field_control_kind(field);
    let editable = matches!(
        control,
        InspectorFieldControlKind::BoolToggle { .. }
            | InspectorFieldControlKind::IntegerInput { .. }
            | InspectorFieldControlKind::FloatInput { .. }
            | InspectorFieldControlKind::TextInput
    );

    InspectorObservedField {
        label: field.display_name.clone(),
        path_key: Some(field.path.stable_key()),
        value_summary: value_text,
        control,
        is_focused: field.is_focused,
        editable,
    }
}

fn inspector_field_control_kind(field: &InspectorWidgetField) -> InspectorFieldControlKind {
    match &field.value {
        InspectorValue::Bool(value) => InspectorFieldControlKind::BoolToggle {
            checked: field
                .draft_value
                .as_ref()
                .and_then(|draft| match draft {
                    editor_inspector::InspectorEditValue::Bool(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or(*value),
        },
        InspectorValue::Integer(value) => InspectorFieldControlKind::IntegerInput {
            value: field
                .draft_value
                .as_ref()
                .and_then(|draft| match draft {
                    editor_inspector::InspectorEditValue::Integer(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or(*value),
        },
        InspectorValue::Float(value) => InspectorFieldControlKind::FloatInput {
            value: field
                .draft_value
                .as_ref()
                .and_then(|draft| match draft {
                    editor_inspector::InspectorEditValue::Float(value) => Some(*value),
                    _ => None,
                })
                .unwrap_or(*value),
        },
        InspectorValue::Text(_) => InspectorFieldControlKind::TextInput,
        InspectorValue::Enum { current, options } => InspectorFieldControlKind::EnumSelect {
            current: current.clone(),
            options: options.clone(),
            selected_index: options.iter().position(|option| option == current),
        },
        InspectorValue::ReadOnlyText(_) => InspectorFieldControlKind::ReadOnly,
        InspectorValue::Group => InspectorFieldControlKind::Group,
        InspectorValue::Unsupported { .. } => InspectorFieldControlKind::Unsupported,
    }
}

fn inspector_value_summary(value: &InspectorValue) -> String {
    match value {
        InspectorValue::Bool(v) => v.to_string(),
        InspectorValue::Integer(v) => v.to_string(),
        InspectorValue::Float(v) => v.to_string(),
        InspectorValue::Text(v) => v.clone(),
        InspectorValue::Enum { current, .. } => current.clone(),
        InspectorValue::ReadOnlyText(v) => v.clone(),
        InspectorValue::Group => "group".to_string(),
        InspectorValue::Unsupported { type_name } => format!("unsupported<{type_name}>"),
    }
}

fn workspace_allows_document(request: &SurfaceProviderRequest) -> bool {
    let registry = editor_shell::default_workspace_profile_registry();
    if request.tool_surface_kind == ToolSurfaceKind::Console
        || is_self_authoring_surface(request.tool_surface_kind)
    {
        return true;
    }
    let Some(document_kind) = request.document_context.resolved_document_kind() else {
        return false;
    };
    registry
        .profile(request.workspace_profile_id)
        .map(|profile| profile.document_kind_filters.contains(document_kind))
        .unwrap_or(false)
}

fn is_self_authoring_surface(kind: ToolSurfaceKind) -> bool {
    matches!(
        kind,
        ToolSurfaceKind::EditorDesignOutliner
            | ToolSurfaceKind::UiHierarchy
            | ToolSurfaceKind::UiCanvas
            | ToolSurfaceKind::StyleInspector
            | ToolSurfaceKind::Bindings
            | ToolSurfaceKind::DockLayoutPreview
            | ToolSurfaceKind::ThemeEditor
            | ToolSurfaceKind::ShortcutEditor
            | ToolSurfaceKind::MenuEditor
            | ToolSurfaceKind::DefinitionValidation
            | ToolSurfaceKind::CommandDiff
    )
}

struct SelfAuthoringProvider;

impl EditorSurfaceProvider for SelfAuthoringProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SELF_AUTHORING_PROVIDER_ID,
            "Self-Authoring",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        is_self_authoring_surface(request.tool_surface_kind)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let title = self_authoring_title(request.tool_surface_kind).to_string();
        let (root, routes) = match request.tool_surface_kind {
            ToolSurfaceKind::UiCanvas => context
                .shell_state
                .self_authoring()
                .formed_selected_preview(context.theme)
                .map(|product| {
                    (
                        remap_surface_node_ids(product.root, request.tool_surface_instance_id),
                        SurfaceRouteTable::empty(),
                    )
                })
                .unwrap_or_else(|| {
                    build_self_authoring_control_panel(
                        context,
                        request.tool_surface_instance_id,
                        vec!["No retained preview available".to_string()],
                        Vec::new(),
                    )
                }),
            _ => build_self_authoring_control_panel(
                context,
                request.tool_surface_instance_id,
                self_authoring_lines(context, request.tool_surface_kind),
                self_authoring_actions(context, request.tool_surface_kind),
            ),
        };

        Ok(ProviderSurfaceFrame {
            title,
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let command = match action {
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SelectDocument { document_id },
            ) => ShellCommand::SelectEditorDefinitionDocument { document_id },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::DuplicateSelected,
            ) => ShellCommand::DuplicateSelectedEditorDefinition,
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::RenameSelected { display_name },
            ) => ShellCommand::RenameSelectedEditorDefinition { display_name },
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::DeleteSelected) => {
                ShellCommand::DeleteSelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ExportSelected) => {
                ShellCommand::ExportSelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected) => {
                ShellCommand::ApplySelectedEditorDefinition
            }
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::RollbackSelected,
            ) => ShellCommand::RollbackSelectedEditorDefinition,
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::SelectUiNode {
                node_id,
            }) => ShellCommand::SelectEditorDefinitionUiNode { node_id },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SetUiNodeText { node_id, text },
            ) => ShellCommand::SetSelectedEditorDefinitionUiNodeText { node_id, text },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SetThemeColor { token, value },
            ) => ShellCommand::SetSelectedEditorThemeColor { token, value },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::AddWorkspaceLayoutTab {
                    label,
                    tool_surface,
                },
            ) => ShellCommand::AddSelectedEditorWorkspaceLayoutTab {
                label,
                tool_surface,
            },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::SplitWorkspaceLayoutRoot { axis },
            ) => ShellCommand::SplitSelectedEditorWorkspaceLayoutRoot { axis },
            SurfaceLocalAction::EditorDefinition(
                EditorDefinitionSurfaceAction::CloseWorkspaceLayoutLastTab,
            ) => ShellCommand::CloseSelectedEditorWorkspaceLayoutLastTab,
            _ => return Ok(None),
        };
        Ok(Some(SurfaceCommandProposal::Shell(command)))
    }
}

fn self_authoring_title(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => "Definition Outliner",
        ToolSurfaceKind::UiHierarchy => "UI Hierarchy",
        ToolSurfaceKind::UiCanvas => "UI Canvas",
        ToolSurfaceKind::StyleInspector => "Style Inspector",
        ToolSurfaceKind::Bindings => "Bindings",
        ToolSurfaceKind::DockLayoutPreview => "Dock Layout Preview",
        ToolSurfaceKind::ThemeEditor => "Theme Editor",
        ToolSurfaceKind::ShortcutEditor => "Shortcut Editor",
        ToolSurfaceKind::MenuEditor => "Menu Editor",
        ToolSurfaceKind::DefinitionValidation => "Definition Validation",
        ToolSurfaceKind::CommandDiff => "Command Diff",
        _ => "Self-Authoring",
    }
}

fn self_authoring_lines(
    context: &SurfaceProviderBuildContext<'_>,
    kind: ToolSurfaceKind,
) -> Vec<String> {
    let state = context.shell_state.self_authoring();
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => state
            .draft_documents()
            .map(|document| {
                let marker = if Some(&document.id) == state.selected_document_id() {
                    "*"
                } else {
                    " "
                };
                format!("{marker} {} [{:?}]", document.display_name, document.kind)
            })
            .collect(),
        ToolSurfaceKind::UiHierarchy => state
            .selected_document()
            .map(|document| match &document.content {
                editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                    ui_node_hierarchy_lines(&template.root, 0, state.selected_ui_node_id())
                }
                _ => vec!["Selected definition is not a UI template".to_string()],
            })
            .unwrap_or_else(|| vec!["No definition selected".to_string()]),
        ToolSurfaceKind::StyleInspector => vec![
            "Theme tokens are editor-owned definition data".to_string(),
            "Retained preview uses the active ThemeTokens until a theme document is applied"
                .to_string(),
        ],
        ToolSurfaceKind::Bindings => state
            .selected_document()
            .map(|document| match &document.content {
                editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) => {
                    vec![
                        format!("template: {}", template.id),
                        format!("local templates: {}", template.templates.len()),
                        format!("menus: {}", template.menus.len()),
                    ]
                }
                editor_definition::EditorDefinitionDocumentContent::EditorBindings(bindings) => {
                    vec![
                        format!("toolbar: {}", bindings.toolbar.template),
                        format!("surface templates: {}", bindings.surface_templates.len()),
                    ]
                }
                _ => vec!["Selected editor definition has no UI slots".to_string()],
            })
            .unwrap_or_else(|| vec!["No definition selected".to_string()]),
        ToolSurfaceKind::DockLayoutPreview => {
            if let Some(document) = state.selected_document()
                && let editor_definition::EditorDefinitionDocumentContent::WorkspaceLayout(layout) =
                    &document.content
            {
                return vec![
                    format!("layout: {}", layout.label),
                    format!("root: {}", workspace_host_summary(&layout.root)),
                    format!("floating hosts: {}", layout.floating_hosts.len()),
                ];
            }
            let workspace = context.shell_state.workspace_state();
            vec![
                "Select an authored workspace layout to edit".to_string(),
                format!("active hosts: {}", workspace.hosts().count()),
                format!("active tab stacks: {}", workspace.tab_stacks().count()),
                format!("active panels: {}", workspace.panels().count()),
                format!(
                    "active tool surfaces: {}",
                    workspace.tool_surfaces().count()
                ),
            ]
        }
        ToolSurfaceKind::ThemeEditor => vec![
            "Theme documents validate in editor_definition".to_string(),
            "Apply keeps runtime state unchanged until explicit shell command".to_string(),
        ],
        ToolSurfaceKind::ShortcutEditor => vec![
            "Shortcut documents report duplicate chord diagnostics".to_string(),
            "Platform override execution remains app-owned".to_string(),
        ],
        ToolSurfaceKind::MenuEditor => vec![
            "Menu documents own labels, hierarchy, availability refs, and command refs".to_string(),
            "Command execution remains outside editor_definition".to_string(),
        ],
        ToolSurfaceKind::DefinitionValidation => {
            let diagnostics = state.selected_diagnostics();
            if diagnostics.is_empty() {
                return vec!["No blocking definition diagnostics".to_string()];
            }
            diagnostics
                .into_iter()
                .map(|diagnostic| {
                    format!(
                        "{:?} {}: {}",
                        diagnostic.severity, diagnostic.code, diagnostic.message
                    )
                })
                .collect()
        }
        ToolSurfaceKind::CommandDiff => state
            .build_apply_preview()
            .map(|preview| preview.summary)
            .unwrap_or_else(|| vec!["No definition selected".to_string()]),
        _ => vec!["Unsupported self-authoring surface".to_string()],
    }
}

fn self_authoring_actions(
    context: &SurfaceProviderBuildContext<'_>,
    kind: ToolSurfaceKind,
) -> Vec<(String, SurfaceLocalAction)> {
    let state = context.shell_state.self_authoring();
    match kind {
        ToolSurfaceKind::EditorDesignOutliner => {
            let mut actions = state
                .draft_documents()
                .map(|document| {
                    (
                        format!("Select {}", document.display_name),
                        SurfaceLocalAction::EditorDefinition(
                            EditorDefinitionSurfaceAction::SelectDocument {
                                document_id: document.id.as_str().to_string(),
                            },
                        ),
                    )
                })
                .collect::<Vec<_>>();
            actions.extend([
                (
                    "Duplicate".to_string(),
                    SurfaceLocalAction::EditorDefinition(
                        EditorDefinitionSurfaceAction::DuplicateSelected,
                    ),
                ),
                (
                    "Delete".to_string(),
                    SurfaceLocalAction::EditorDefinition(
                        EditorDefinitionSurfaceAction::DeleteSelected,
                    ),
                ),
                (
                    "Export".to_string(),
                    SurfaceLocalAction::EditorDefinition(
                        EditorDefinitionSurfaceAction::ExportSelected,
                    ),
                ),
            ]);
            actions
        }
        ToolSurfaceKind::UiHierarchy => selected_ui_node_actions(state),
        ToolSurfaceKind::StyleInspector => vec![(
            "Rename Draft".to_string(),
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::RenameSelected {
                display_name: "Retained draft".to_string(),
            }),
        )],
        ToolSurfaceKind::ThemeEditor => vec![
            (
                "Select Theme".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SelectDocument {
                        document_id: "runenwerk.editor.theme.default".to_string(),
                    },
                ),
            ),
            (
                "Set Accent".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SetThemeColor {
                        token: "accent".to_string(),
                        value: "#5f8cff".to_string(),
                    },
                ),
            ),
        ],
        ToolSurfaceKind::DefinitionValidation | ToolSurfaceKind::CommandDiff => vec![
            (
                "Apply".to_string(),
                SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected),
            ),
            (
                "Rollback".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::RollbackSelected,
                ),
            ),
        ],
        ToolSurfaceKind::DockLayoutPreview => vec![
            (
                "Select Layout".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SelectDocument {
                        document_id: "runenwerk.editor.layout.editor_design".to_string(),
                    },
                ),
            ),
            (
                "Add Tab".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::AddWorkspaceLayoutTab {
                        label: "Authored Tab".to_string(),
                        tool_surface: "definition_validation".to_string(),
                    },
                ),
            ),
            (
                "Split Root".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::SplitWorkspaceLayoutRoot {
                        axis: "horizontal".to_string(),
                    },
                ),
            ),
            (
                "Close Last Tab".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::CloseWorkspaceLayoutLastTab,
                ),
            ),
            (
                "Apply".to_string(),
                SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected),
            ),
            (
                "Rollback".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::RollbackSelected,
                ),
            ),
        ],
        ToolSurfaceKind::Bindings
        | ToolSurfaceKind::ShortcutEditor
        | ToolSurfaceKind::MenuEditor => vec![
            (
                "Duplicate".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::DuplicateSelected,
                ),
            ),
            (
                "Apply".to_string(),
                SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::ApplySelected),
            ),
            (
                "Rollback".to_string(),
                SurfaceLocalAction::EditorDefinition(
                    EditorDefinitionSurfaceAction::RollbackSelected,
                ),
            ),
        ],
        _ => Vec::new(),
    }
}

fn selected_ui_node_actions(
    state: &crate::shell::self_authoring::SelfAuthoringWorkspaceState,
) -> Vec<(String, SurfaceLocalAction)> {
    let Some(document) = state.selected_document() else {
        return Vec::new();
    };
    let editor_definition::EditorDefinitionDocumentContent::UiTemplate(template) =
        &document.content
    else {
        return Vec::new();
    };
    let mut actions = ui_node_selection_actions(&template.root);
    if let Some(node_id) = state.selected_ui_node_id() {
        actions.push((
            "Set Text".to_string(),
            SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::SetUiNodeText {
                node_id: node_id.to_string(),
                text: "Edited in self-authoring".to_string(),
            }),
        ));
    }
    actions
}

fn ui_node_selection_actions(
    node: &ui_definition::UiNodeDefinition,
) -> Vec<(String, SurfaceLocalAction)> {
    let mut actions = vec![(
        format!("Select {}", node.id()),
        SurfaceLocalAction::EditorDefinition(EditorDefinitionSurfaceAction::SelectUiNode {
            node_id: node.id().as_str().to_string(),
        }),
    )];
    for child in node.children() {
        actions.extend(ui_node_selection_actions(child));
    }
    actions
}

fn ui_node_hierarchy_lines(
    node: &ui_definition::UiNodeDefinition,
    depth: usize,
    selected_node_id: Option<&str>,
) -> Vec<String> {
    let marker = if selected_node_id == Some(node.id().as_str()) {
        "* "
    } else {
        "  "
    };
    let mut lines = vec![format!(
        "{}{}{}",
        "  ".repeat(depth),
        marker,
        node.id().as_str()
    )];
    for child in node.children() {
        lines.extend(ui_node_hierarchy_lines(child, depth + 1, selected_node_id));
    }
    lines
}

fn workspace_host_summary(host: &editor_definition::EditorWorkspaceHostDefinition) -> String {
    match host {
        editor_definition::EditorWorkspaceHostDefinition::TabStack { tabs, .. } => {
            format!("tab_stack tabs={}", tabs.len())
        }
        editor_definition::EditorWorkspaceHostDefinition::Split {
            axis,
            fraction,
            first,
            second,
            ..
        } => format!(
            "split {:?} {:.2} [{} | {}]",
            axis,
            fraction,
            workspace_host_summary(first),
            workspace_host_summary(second)
        ),
    }
}

fn build_self_authoring_control_panel(
    context: &SurfaceProviderBuildContext<'_>,
    surface_id: ToolSurfaceInstanceId,
    lines: Vec<String>,
    actions: Vec<(String, SurfaceLocalAction)>,
) -> (editor_shell::UiNode, SurfaceRouteTable) {
    let text_style = context.theme.body_small_text_style(FontId(1));
    let mut children = Vec::new();
    let mut routes = SurfaceRouteTable::empty();
    for (index, line) in lines.into_iter().enumerate() {
        children.push(editor_shell::label(
            editor_shell::WidgetId(30_000 + index as u64),
            line,
            text_style.clone(),
        ));
    }
    for (index, (label, action)) in actions.into_iter().enumerate() {
        let widget_id = editor_shell::WidgetId(40_000 + index as u64);
        children.push(editor_shell::button(
            widget_id,
            label,
            text_style.clone(),
            context.theme.clone(),
        ));
        routes.insert(
            remap_widget_id(surface_id, widget_id),
            SurfaceLocalRoute::new(action),
        );
    }
    (
        remap_surface_node_ids(
            editor_shell::panel(
                editor_shell::WidgetId(29_999),
                context.theme.clone(),
                children,
            ),
            surface_id,
        ),
        routes,
    )
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
    match &mut node.kind {
        editor_shell::UiNodeKind::Popup(popup) => {
            popup.anchor = remap_widget_id(surface_id, popup.anchor);
        }
        editor_shell::UiNodeKind::Button(button) => {
            button.reveal_on_hover_anchor = button
                .reveal_on_hover_anchor
                .map(|anchor| remap_widget_id(surface_id, anchor));
        }
        _ => {}
    }
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

pub mod console;
pub mod scene;

use console::ConsoleProvider;
use scene::{
    SceneEntityTableProvider, SceneInspectorProvider, SceneOutlinerProvider, SceneViewportProvider,
};

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
        ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
            Ok(None)
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
            capabilities: tool_surface_capability_set(ToolSurfaceKind::Viewport),
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

    fn self_authoring_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::NoActiveDocument,
            panel_instance_id: PanelInstanceId::try_from_raw(10).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(10).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(10).unwrap(),
            tool_surface_kind,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
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
            viewport_instances: None,
        }
    }

    fn find_node(
        node: &editor_shell::UiNode,
        widget_id: editor_shell::WidgetId,
    ) -> Option<&editor_shell::UiNode> {
        if node.id == widget_id {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| find_node(child, widget_id))
    }

    #[test]
    fn surface_node_remap_rewrites_popup_and_reveal_anchor_widget_ids() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(9).unwrap();
        let anchor_id = editor_shell::WidgetId(44);
        let popup_id = editor_shell::WidgetId(76);
        let button_id = editor_shell::WidgetId(1_500_000);
        let mut button = editor_shell::ButtonNode::new(
            "x",
            ThemeTokens::default().body_small_text_style(FontId(1)),
            ThemeTokens::default(),
        );
        button.reveal_on_hover_anchor = Some(anchor_id);
        let root = editor_shell::UiNode::with_children(
            editor_shell::WidgetId(40),
            editor_shell::UiNodeKind::Panel(editor_shell::PanelNode::new(ThemeTokens::default())),
            vec![
                editor_shell::UiNode::new(
                    anchor_id,
                    editor_shell::UiNodeKind::Panel(editor_shell::PanelNode::new(
                        ThemeTokens::default(),
                    )),
                ),
                editor_shell::UiNode::with_children(
                    popup_id,
                    editor_shell::UiNodeKind::Popup(editor_shell::PopupNode::anchored_top_start(
                        anchor_id,
                        ThemeTokens::default(),
                    )),
                    vec![editor_shell::UiNode::new(
                        button_id,
                        editor_shell::UiNodeKind::Button(button),
                    )],
                ),
            ],
        );

        let remapped = remap_surface_node_ids(root, surface_id);
        let remapped_anchor_id = remap_widget_id(surface_id, anchor_id);
        let remapped_popup_id = remap_widget_id(surface_id, popup_id);
        let remapped_button_id = remap_widget_id(surface_id, button_id);
        let popup = find_node(&remapped, remapped_popup_id).expect("popup should be remapped");
        let button = find_node(&remapped, remapped_button_id).expect("button should be remapped");

        assert!(find_node(&remapped, remapped_anchor_id).is_some());
        assert!(matches!(
            &popup.kind,
            editor_shell::UiNodeKind::Popup(popup) if popup.anchor == remapped_anchor_id
        ));
        assert!(matches!(
            &button.kind,
            editor_shell::UiNodeKind::Button(button)
                if button.reveal_on_hover_anchor == Some(remapped_anchor_id)
        ));
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
    fn self_authoring_provider_resolves_definition_validation_without_scene_document() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = self_authoring_request(ToolSurfaceKind::DefinitionValidation);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &SurfaceSessionState::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.title, "Definition Validation");
        assert!(!frame.routes.is_empty());
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
