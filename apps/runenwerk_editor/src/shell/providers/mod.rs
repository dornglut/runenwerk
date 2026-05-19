use std::collections::{BTreeMap, BTreeSet};

use editor_core::{DocumentKind, EditorMutationError, EntityId, RealityVersion};
use editor_inspector::InspectorValue;
use editor_shell::{
    AssetSurfaceAction, ConsoleViewModel, ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
    ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID, ENTITY_TABLE_LIST_WIDGET_ID,
    ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID, EditorDefinitionSurfaceAction,
    EditorDomainMutation, EditorShellFrameModel, EntityTableDomainMutation,
    EntityTableRowViewModel, EntityTableSessionMutation, EntityTableSortKey,
    EntityTableSurfaceAction, EntityTableViewModel, InspectorFieldControlKind,
    InspectorFieldViewModel, InspectorObservationFrame, InspectorObservedField,
    InspectorObservedTarget, InspectorSessionMutation, InspectorSurfaceAction,
    InspectorTargetViewModel, InspectorViewModel, MaterialDiagnosticRowViewModel,
    MaterialPreviewStatusViewModel, MaterialResourceBindingDiagnosticViewModel,
    MaterialSurfaceAction, OUTLINER_LIST_WIDGET_ID, ObservationConsumerKind,
    ObservationFrameMetadata, ObservationSourceReality, OutlinerDomainMutation,
    OutlinerObservationFrame, OutlinerObservedRow, OutlinerRowViewModel, OutlinerSurfaceAction,
    OutlinerViewModel, ProviderFamilyId, ProviderFamilyProviderAssignment,
    ProviderFamilyProviderMap, ResolvedSurfaceFrame, ShellCommand, SurfaceCommandProposal,
    SurfaceDocumentContext, SurfaceInteraction, SurfaceLocalAction, SurfaceLocalRoute,
    SurfacePresentationArtifact, SurfacePresentationArtifactKind, SurfaceProviderAvailability,
    SurfaceProviderDescriptor, SurfaceProviderDiagnostic, SurfaceProviderId,
    SurfaceProviderPriority, SurfaceProviderRequest, SurfaceProviderSupportMode, SurfaceRouteTable,
    SurfaceSessionMutation, TexturePreviewChannelSelection, TextureSurfaceAction,
    TextureViewerSurfaceKind, ToolSurfaceCreateCandidate, ToolSurfaceKind, ToolSurfaceRegistry,
    UiNode, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID, VIEWPORT_FIELD_SLICE_DECREMENT_WIDGET_ID,
    VIEWPORT_FIELD_SLICE_INCREMENT_WIDGET_ID, VIEWPORT_FIELD_SLICE_RESET_WIDGET_ID,
    VIEWPORT_OPTIONS_BUTTON_WIDGET_ID, VIEWPORT_RESET_CAMERA_WIDGET_ID,
    VIEWPORT_ROOT_OPAQUE_TOGGLE_WIDGET_ID, VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID,
    VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID, ViewportDomainMutation, ViewportObservationFrame,
    ViewportProductChoiceViewModel, ViewportProductObservation, ViewportSessionMutation,
    ViewportSurfaceAction, ViewportViewModel, WidgetId, build_console_panel,
    build_entity_table_panel, build_inspector_panel, build_material_graph_surface,
    build_outliner_panel, build_self_authoring_control_panel, build_viewport_panel,
    editor_domain_proposal, entity_table_sort_button_widget_id, inspector_field_focus_widget_id,
    inspector_field_widget_id, surface_session_proposal, surface_widget_id,
    tool_surface_capability_set, tool_surface_definition_id, tool_surface_kind_for_stable_key,
    viewport_debug_stage_button_widget_id, viewport_field_color_ramp_button_widget_id,
    viewport_field_component_button_widget_id, viewport_field_debug_mode_button_widget_id,
    viewport_product_button_widget_id, viewport_tool_radial_item_widget_id,
};
use editor_viewport::{
    ArtifactObservationFrame, ProducerHealth, ProductAvailabilityState,
    ViewportFieldVisualizerSettings,
};
use ui_text::FontId;
use ui_theme::ThemeTokens;

use crate::editor_app::{ConsoleMessage, ConsoleMessageKind, RunenwerkEditorApp};
use crate::editor_panels::{
    EntityTablePanelPresenter, EntityTablePanelState, InspectorPanelPresenter,
    InspectorPanelViewModel, InspectorWidgetField, OutlinerPanelState, ViewportToolState,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource,
};
use crate::shell::active_route_actions_by_target;
use crate::shell::tool_suites::{
    ASSET_BROWSER_SURFACE_KEY, DIAGNOSTICS_SURFACE_KEYS, EDITOR_CONSOLE_SURFACE_KEY,
    EDITOR_DESIGN_SURFACE_KEYS, FIELD_LAYER_STACK_SURFACE_KEY, FIELD_PRODUCT_VIEWER_SURFACE_KEY,
    IMPORT_INSPECTOR_SURFACE_KEY, MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
    MATERIAL_INSPECTOR_SURFACE_KEY, MATERIAL_PREVIEW_SURFACE_KEY, PROCGEN_GRAPH_CANVAS_SURFACE_KEY,
    PROCGEN_PREVIEW_SURFACE_KEY, SCENE_ENTITY_TABLE_SURFACE_KEY, SCENE_INSPECTOR_SURFACE_KEY,
    SCENE_OUTLINER_SURFACE_KEY, SCENE_VIEWPORT_SURFACE_KEY, SDF_BRUSH_BROWSER_SURFACE_KEY,
    SDF_GRAPH_CANVAS_SURFACE_KEY, TEXTURE_VIEWER_2D_SURFACE_KEY, TEXTURE_VIEWER_3D_SURFACE_KEY,
    TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
};
use crate::shell::toolbar_adapter::{
    build_toolbar_observation_frame, build_toolbar_view_model, toolbar_binding_with_active_menus,
};
use crate::shell::{RunenwerkEditorShellState, SurfaceSessionState};

const SCENE_OUTLINER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(1);
const SCENE_ENTITY_TABLE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(2);
const SCENE_VIEWPORT_PROVIDER_ID: SurfaceProviderId = surface_provider_id(3);
const SCENE_INSPECTOR_PROVIDER_ID: SurfaceProviderId = surface_provider_id(4);
const CONSOLE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(5);
const SELF_AUTHORING_PROVIDER_ID: SurfaceProviderId = surface_provider_id(6);
const ASSET_BROWSER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(7);
const IMPORT_INSPECTOR_PROVIDER_ID: SurfaceProviderId = surface_provider_id(8);
const FIELD_PRODUCT_VIEWER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(9);
const SDF_BRUSH_BROWSER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(10);
const M6_WORKSPACE_PROVIDER_ID: SurfaceProviderId = surface_provider_id(11);
const MATERIAL_GRAPH_CANVAS_PROVIDER_ID: SurfaceProviderId = surface_provider_id(12);
const MATERIAL_INSPECTOR_PROVIDER_ID: SurfaceProviderId = surface_provider_id(13);
const MATERIAL_PREVIEW_PROVIDER_ID: SurfaceProviderId = surface_provider_id(14);
const TEXTURE_VIEWER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(15);
const VOLUME_TEXTURE_VIEWER_PROVIDER_ID: SurfaceProviderId = surface_provider_id(16);
const FIELD_LAYER_STACK_PROVIDER_ID: SurfaceProviderId = surface_provider_id(17);
const SDF_GRAPH_CANVAS_PROVIDER_ID: SurfaceProviderId = surface_provider_id(18);
const PROCGEN_GRAPH_CANVAS_PROVIDER_ID: SurfaceProviderId = surface_provider_id(19);
const PROCGEN_PREVIEW_PROVIDER_ID: SurfaceProviderId = surface_provider_id(20);
const TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID: SurfaceProviderId = surface_provider_id(21);

const fn surface_provider_id(raw: u64) -> SurfaceProviderId {
    match SurfaceProviderId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("surface provider id constants must be non-zero"),
    }
}

pub(crate) fn runenwerk_provider_family_assignments() -> Vec<ProviderFamilyProviderAssignment> {
    vec![
        provider_family_assignment("runenwerk.scene", SCENE_OUTLINER_PROVIDER_ID),
        provider_family_assignment("runenwerk.scene", SCENE_ENTITY_TABLE_PROVIDER_ID),
        provider_family_assignment("runenwerk.scene", SCENE_VIEWPORT_PROVIDER_ID),
        provider_family_assignment("runenwerk.scene", SCENE_INSPECTOR_PROVIDER_ID),
        provider_family_assignment("runenwerk.editor", CONSOLE_PROVIDER_ID),
        provider_family_assignment("runenwerk.editor_design", SELF_AUTHORING_PROVIDER_ID),
        provider_family_assignment("runenwerk.assets", ASSET_BROWSER_PROVIDER_ID),
        provider_family_assignment("runenwerk.assets", IMPORT_INSPECTOR_PROVIDER_ID),
        provider_family_assignment("runenwerk.field_world", FIELD_PRODUCT_VIEWER_PROVIDER_ID),
        provider_family_assignment("runenwerk.field_world", SDF_BRUSH_BROWSER_PROVIDER_ID),
        provider_family_assignment("runenwerk.field_world", FIELD_LAYER_STACK_PROVIDER_ID),
        provider_family_assignment("runenwerk.field_world", SDF_GRAPH_CANVAS_PROVIDER_ID),
        provider_family_assignment("runenwerk.texture", TEXTURE_VIEWER_PROVIDER_ID),
        provider_family_assignment("runenwerk.texture", VOLUME_TEXTURE_VIEWER_PROVIDER_ID),
        provider_family_assignment("runenwerk.procgen", PROCGEN_GRAPH_CANVAS_PROVIDER_ID),
        provider_family_assignment("runenwerk.procgen", PROCGEN_PREVIEW_PROVIDER_ID),
        provider_family_assignment("runenwerk.diagnostics", M6_WORKSPACE_PROVIDER_ID),
        provider_family_assignment(
            "runenwerk.diagnostics",
            TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID,
        ),
        provider_family_assignment("runenwerk.material_lab", MATERIAL_GRAPH_CANVAS_PROVIDER_ID),
        provider_family_assignment("runenwerk.material_lab", MATERIAL_INSPECTOR_PROVIDER_ID),
        provider_family_assignment("runenwerk.material_lab", MATERIAL_PREVIEW_PROVIDER_ID),
    ]
}

fn provider_family_assignment(
    provider_family_id: &str,
    provider_id: SurfaceProviderId,
) -> ProviderFamilyProviderAssignment {
    ProviderFamilyProviderAssignment::new(
        ProviderFamilyId::new(provider_family_id)
            .expect("compiled-in provider family id should be valid"),
        provider_id,
    )
}

pub struct SurfaceProviderBuildContext<'a> {
    pub app: &'a RunenwerkEditorApp,
    pub shell_state: &'a RunenwerkEditorShellState,
    pub theme: &'a ThemeTokens,
    pub frame_metrics: Option<EditorShellFrameMetrics>,
    pub viewport_observations: Option<&'a ViewportArtifactObservationResource>,
    pub tool_surface_bindings: Option<&'a ToolSurfaceRuntimeBindingRegistryResource>,
    pub viewport_instances: Option<&'a ViewportInstanceRegistryResource>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EditorShellFrameMetrics {
    pub fps_ema: f32,
    pub frame_ms_ema: f32,
}

pub struct SurfaceProviderDispatchContext<'a> {
    pub projection_epoch: u64,
    pub _marker: std::marker::PhantomData<&'a ()>,
}

pub trait EditorSurfaceProvider: Send + Sync {
    fn descriptor(&self) -> SurfaceProviderDescriptor;
    fn supports(&self, request: &SurfaceProviderRequest) -> bool;
    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        if self.supports(request) {
            SurfaceProviderSupportMode::LegacyKind
        } else {
            SurfaceProviderSupportMode::Unsupported
        }
    }
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

    fn map_interaction(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _interaction: SurfaceInteraction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        Ok(None)
    }
}

fn stable_key_or_legacy_kind_support(
    request: &SurfaceProviderRequest,
    stable_key: &str,
    legacy_kind: ToolSurfaceKind,
) -> SurfaceProviderSupportMode {
    if request.matches_stable_key(stable_key) {
        SurfaceProviderSupportMode::StableKey
    } else if request.legacy_kind() == Some(legacy_kind) {
        SurfaceProviderSupportMode::LegacyKind
    } else {
        SurfaceProviderSupportMode::Unsupported
    }
}

fn stable_keys_or_legacy_kind_support(
    request: &SurfaceProviderRequest,
    stable_keys: &[&str],
    legacy_kind_predicate: impl Fn(ToolSurfaceKind) -> bool,
) -> SurfaceProviderSupportMode {
    if request.matches_any_stable_key(stable_keys) {
        SurfaceProviderSupportMode::StableKey
    } else if request.legacy_kind().is_some_and(legacy_kind_predicate) {
        SurfaceProviderSupportMode::LegacyKind
    } else {
        SurfaceProviderSupportMode::Unsupported
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfaceProviderSupportObservation {
    pub provider_id: SurfaceProviderId,
    pub support_mode: SurfaceProviderSupportMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurfaceProviderResolutionObservation {
    pub candidate_provider_ids: Vec<SurfaceProviderId>,
    pub support_modes: Vec<SurfaceProviderSupportObservation>,
    pub selected_provider_id: Option<SurfaceProviderId>,
    pub availability: SurfaceProviderAvailability,
    pub diagnostic: Option<SurfaceProviderDiagnostic>,
}

struct SupportedProvider<'a> {
    provider: &'a dyn EditorSurfaceProvider,
    support_mode: SurfaceProviderSupportMode,
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
            Box::new(AssetBrowserProvider),
            Box::new(ImportInspectorProvider),
            Box::new(FieldProductViewerProvider),
            Box::new(SdfBrushBrowserProvider),
            Box::new(FieldLayerStackProvider),
            Box::new(SdfGraphCanvasProvider),
            Box::new(MaterialGraphCanvasProvider),
            Box::new(MaterialInspectorProvider),
            Box::new(MaterialPreviewProvider),
            Box::new(TextureViewerProvider),
            Box::new(VolumeTextureViewerProvider),
            Box::new(ProcgenGraphCanvasProvider),
            Box::new(ProcgenPreviewProvider),
            Box::new(M6WorkspaceProvider),
            Box::new(ToolSuiteRegistryInspectorProvider),
        ])
        .expect("default surface providers must have unique ids")
    }

    pub fn provider_descriptors(&self) -> impl Iterator<Item = SurfaceProviderDescriptor> + '_ {
        self.providers.iter().map(|provider| provider.descriptor())
    }

    pub fn provider_ids(&self) -> impl Iterator<Item = SurfaceProviderId> + '_ {
        self.providers
            .iter()
            .map(|provider| provider.descriptor().id)
    }

    pub fn has_provider_id(&self, provider_id: SurfaceProviderId) -> bool {
        self.provider_ids().any(|id| id == provider_id)
    }

    pub(crate) fn observe_resolution_for_request(
        &self,
        request: &SurfaceProviderRequest,
        provider_family_map: Option<&ProviderFamilyProviderMap>,
    ) -> SurfaceProviderResolutionObservation {
        let candidate_providers =
            match self.candidate_providers_for_request(request, provider_family_map) {
                Ok(candidate_providers) => candidate_providers,
                Err(diagnostic) => {
                    return SurfaceProviderResolutionObservation {
                        candidate_provider_ids: Vec::new(),
                        support_modes: Vec::new(),
                        selected_provider_id: None,
                        availability: SurfaceProviderAvailability::Unsupported,
                        diagnostic: Some(diagnostic),
                    };
                }
            };
        let candidate_provider_ids = candidate_providers
            .iter()
            .map(|provider| provider.descriptor().id)
            .collect::<Vec<_>>();
        let support_modes = candidate_providers
            .iter()
            .map(|provider| SurfaceProviderSupportObservation {
                provider_id: provider.descriptor().id,
                support_mode: provider.support_mode(request),
            })
            .collect::<Vec<_>>();

        if !workspace_allows_document(request) {
            return SurfaceProviderResolutionObservation {
                candidate_provider_ids,
                support_modes,
                selected_provider_id: None,
                availability: SurfaceProviderAvailability::Unsupported,
                diagnostic: Some(SurfaceProviderDiagnostic::new(
                    "editor.surface.unsupported_document",
                    "workspace profile does not allow the active document kind",
                )),
            };
        }

        let supported = candidate_providers
            .into_iter()
            .filter_map(|provider| {
                let support_mode = provider.support_mode(request);
                support_mode.is_supported().then_some(SupportedProvider {
                    provider,
                    support_mode,
                })
            })
            .collect::<Vec<_>>();
        if supported.is_empty() {
            return SurfaceProviderResolutionObservation {
                candidate_provider_ids,
                support_modes,
                selected_provider_id: None,
                availability: SurfaceProviderAvailability::Unsupported,
                diagnostic: Some(SurfaceProviderDiagnostic::new(
                    "editor.surface.unsupported_provider",
                    "no provider supports this surface request",
                )),
            };
        }

        let preferred_support_mode = supported.iter().fold(
            SurfaceProviderSupportMode::Unsupported,
            |current, supported| current.preferred(supported.support_mode),
        );
        let supported = supported
            .into_iter()
            .filter(|supported| supported.support_mode == preferred_support_mode)
            .map(|supported| supported.provider)
            .collect::<Vec<_>>();
        let Some(provider) = deterministic_provider(supported) else {
            return SurfaceProviderResolutionObservation {
                candidate_provider_ids,
                support_modes,
                selected_provider_id: None,
                availability: SurfaceProviderAvailability::Ambiguous,
                diagnostic: Some(SurfaceProviderDiagnostic::new(
                    "editor.surface.ambiguous_provider",
                    "multiple providers support this request at the same priority",
                )),
            };
        };

        SurfaceProviderResolutionObservation {
            candidate_provider_ids,
            support_modes,
            selected_provider_id: Some(provider.descriptor().id),
            availability: SurfaceProviderAvailability::Available,
            diagnostic: None,
        }
    }

    pub fn resolve_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> ResolvedSurfaceFrame {
        self.resolve_frame_with_provider_family_map(context, request, session, None)
    }

    pub fn resolve_frame_with_provider_family_map(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
        provider_family_map: Option<&ProviderFamilyProviderMap>,
    ) -> ResolvedSurfaceFrame {
        if !workspace_allows_document(request) {
            return unsupported_frame(
                request,
                "Unsupported Document",
                "editor.surface.unsupported_document",
                "workspace profile does not allow the active document kind",
            );
        }

        let candidate_providers =
            match self.candidate_providers_for_request(request, provider_family_map) {
                Ok(candidate_providers) => candidate_providers,
                Err(diagnostic) => {
                    return diagnostic_frame(
                        request,
                        "Unsupported Surface",
                        SurfaceProviderAvailability::Unsupported,
                        SurfacePresentationArtifactKind::Unsupported,
                        diagnostic,
                    );
                }
            };

        let supported = candidate_providers
            .into_iter()
            .filter_map(|provider| {
                let support_mode = provider.support_mode(request);
                support_mode.is_supported().then_some(SupportedProvider {
                    provider,
                    support_mode,
                })
            })
            .collect::<Vec<_>>();
        if supported.is_empty() {
            return unsupported_frame(
                request,
                "Unsupported Surface",
                "editor.surface.unsupported_provider",
                "no provider supports this surface request",
            );
        }
        let preferred_support_mode = supported.iter().fold(
            SurfaceProviderSupportMode::Unsupported,
            |current, supported| current.preferred(supported.support_mode),
        );
        let supported = supported
            .into_iter()
            .filter(|supported| supported.support_mode == preferred_support_mode)
            .map(|supported| supported.provider)
            .collect::<Vec<_>>();
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
                surface_kind: request.legacy_kind(),
                stable_surface_key: request.stable_surface_key.clone(),
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

    fn candidate_providers_for_request(
        &self,
        request: &SurfaceProviderRequest,
        provider_family_map: Option<&ProviderFamilyProviderMap>,
    ) -> Result<Vec<&dyn EditorSurfaceProvider>, SurfaceProviderDiagnostic> {
        let Some(provider_family_map) = provider_family_map else {
            return Ok(self.providers.iter().map(Box::as_ref).collect());
        };

        let Some(provider_family_id) = request.provider_family_id.as_ref() else {
            return Ok(self.providers.iter().map(Box::as_ref).collect());
        };

        let provider_ids = provider_family_map
            .providers_for(provider_family_id)
            .collect::<BTreeSet<_>>();
        if provider_ids.is_empty() {
            return Err(SurfaceProviderDiagnostic::new(
                "editor.surface.unassigned_provider_family",
                format!(
                    "no providers are assigned to provider family `{}`",
                    provider_family_id.as_str()
                ),
            ));
        }

        Ok(self
            .providers
            .iter()
            .map(Box::as_ref)
            .filter(|provider| provider_ids.contains(&provider.descriptor().id))
            .collect())
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

    pub fn map_interaction(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        provider_id: SurfaceProviderId,
        interaction: SurfaceInteraction,
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
            .map_interaction(context, request, interaction)
            .map_err(|_| {
                EditorMutationError::session_rejected("surface provider interaction failed")
            })
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
    build_editor_shell_frame_model_with_frame_metrics(
        app,
        shell_state,
        registry,
        theme,
        None,
        viewport_observations,
        tool_surface_bindings,
        viewport_instances,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_editor_shell_frame_model_with_frame_metrics(
    app: &RunenwerkEditorApp,
    shell_state: &RunenwerkEditorShellState,
    registry: &EditorSurfaceProviderRegistry,
    theme: &ThemeTokens,
    frame_metrics: Option<EditorShellFrameMetrics>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
) -> EditorShellFrameModel {
    let scene_version = app.runtime().current_scene_reality_version();
    let session = app.runtime().session_reality();
    let history = session.history();
    let active_definitions = shell_state.active_editor_definitions();
    let toolbar_frame = build_toolbar_observation_frame(
        session.active_tool(),
        history.can_undo(),
        history.can_redo(),
        app.debug_logs_enabled(),
        shell_state.active_toolbar_menu(),
        shell_state.active_workspace_profile_id(),
        shell_state.open_workspace_profile_ids(),
        scene_version,
        active_definitions.menus(),
    );

    let context = SurfaceProviderBuildContext {
        app,
        shell_state,
        theme,
        frame_metrics,
        viewport_observations,
        tool_surface_bindings,
        viewport_instances,
    };
    let document_context = active_document_context(app);
    let mut surfaces = BTreeMap::new();
    for request in mounted_surface_requests_with_registry(
        shell_state,
        document_context,
        Some(app.workbench_host().tool_surface_registry()),
    ) {
        let session = app
            .surface_sessions()
            .session_or_default(request.tool_surface_instance_id);
        let frame = if request.provider_family_id.is_none() {
            diagnostic_frame(
                &request,
                "Unsupported Surface",
                SurfaceProviderAvailability::Unsupported,
                SurfacePresentationArtifactKind::Unsupported,
                SurfaceProviderDiagnostic::new(
                    "editor.surface.unresolved_provider_family",
                    "stable surface metadata did not resolve to a provider family",
                ),
            )
        } else {
            registry.resolve_frame_with_provider_family_map(
                &context,
                &request,
                &session,
                Some(app.workbench_host().provider_family_provider_map()),
            )
        };
        surfaces.insert(request.tool_surface_instance_id, frame);
    }

    let active_bindings = active_definitions.editor_bindings();
    let toolbar_template = active_bindings.and_then(|bindings| {
        active_definitions
            .templates()
            .get(&bindings.toolbar.template)
            .cloned()
    });
    let toolbar_binding = toolbar_binding_with_active_menus(
        active_bindings.map(|bindings| bindings.toolbar.clone()),
        active_definitions.menus(),
    );
    let shell_chrome_template = active_bindings.and_then(|bindings| {
        active_definitions
            .templates()
            .get(&bindings.shell_chrome_template)
            .cloned()
    });
    let route_actions =
        active_route_actions_by_target(active_definitions, history.can_undo(), history.can_redo());
    let available_panel_kinds = active_definitions.available_panel_kinds();
    let available_tool_surface_kinds = active_definitions.available_tool_surface_kinds();
    let available_tool_surface_create_candidates = build_tool_surface_create_candidates(
        &available_tool_surface_kinds,
        app.workbench_host().tool_surface_registry(),
    );

    EditorShellFrameModel::new(build_toolbar_view_model(&toolbar_frame), surfaces)
        .with_route_actions(route_actions)
        .with_available_panel_kinds(available_panel_kinds)
        .with_available_tool_surface_kinds(available_tool_surface_kinds)
        .with_available_tool_surface_create_candidates(available_tool_surface_create_candidates)
        .with_active_ui_definitions(toolbar_template, toolbar_binding, shell_chrome_template)
        .with_active_tab_stack_popup_menu(shell_state.active_tab_stack_popup_menu())
}

fn build_tool_surface_create_candidates(
    available_tool_surface_kinds: &[ToolSurfaceKind],
    tool_surface_registry: &ToolSurfaceRegistry,
) -> Vec<ToolSurfaceCreateCandidate> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    let legacy_kinds = if available_tool_surface_kinds.is_empty() {
        tool_surface_registry
            .iter()
            .filter_map(|surface| tool_surface_kind_for_stable_key(&surface.key))
            .collect::<Vec<_>>()
    } else {
        available_tool_surface_kinds.to_vec()
    };

    for kind in legacy_kinds {
        let Some(stable_surface_key) = editor_shell::stable_key_for_tool_surface_kind(kind) else {
            continue;
        };
        let Some(surface) = tool_surface_registry.get(&stable_surface_key) else {
            continue;
        };
        if seen.insert(stable_surface_key.clone()) {
            candidates.push(ToolSurfaceCreateCandidate::new(
                stable_surface_key,
                surface.label.clone(),
                surface.panel_kind,
                Some(kind),
            ));
        }
    }

    for surface in tool_surface_registry.iter() {
        if tool_surface_kind_for_stable_key(&surface.key).is_none()
            && seen.insert(surface.key.clone())
        {
            candidates.push(ToolSurfaceCreateCandidate::new(
                surface.key.clone(),
                surface.label.clone(),
                surface.panel_kind,
                None,
            ));
        }
    }

    candidates
}

pub fn mounted_surface_requests(
    shell_state: &RunenwerkEditorShellState,
    document_context: SurfaceDocumentContext,
) -> Vec<SurfaceProviderRequest> {
    mounted_surface_requests_with_registry(shell_state, document_context, None)
}

pub fn mounted_surface_requests_with_registry(
    shell_state: &RunenwerkEditorShellState,
    document_context: SurfaceDocumentContext,
    tool_surface_registry: Option<&ToolSurfaceRegistry>,
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
            let stable_surface_key = surface.stable_surface_key().clone();
            let legacy_tool_surface_kind = surface.legacy_tool_surface_kind();
            let registered_surface =
                tool_surface_registry.and_then(|registry| registry.get(&stable_surface_key));
            let surface_definition_id = legacy_tool_surface_kind
                .map(tool_surface_definition_id)
                .unwrap_or(editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID);
            let capabilities = legacy_tool_surface_kind
                .map(tool_surface_capability_set)
                .unwrap_or_default();
            Some(SurfaceProviderRequest {
                workspace_profile_id: shell_state.active_workspace_profile_id(),
                document_context: document_context.clone(),
                panel_instance_id: panel.id,
                tab_stack_id,
                tool_surface_instance_id: surface.id,
                stable_surface_key,
                legacy_tool_surface_kind,
                provider_family_id: registered_surface
                    .map(|definition| definition.provider_family.clone()),
                surface_route: registered_surface.map(|definition| definition.route),
                surface_definition_id,
                capabilities,
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

fn build_console_view_model(lines: &[ConsoleMessage]) -> ConsoleViewModel {
    ConsoleViewModel {
        lines: lines
            .iter()
            .map(|line| {
                editor_shell::ConsoleLineViewModel::new(
                    console_line_kind(line.kind),
                    line.text.clone(),
                )
            })
            .collect(),
    }
}

fn console_line_kind(kind: ConsoleMessageKind) -> editor_shell::ConsoleLineKind {
    match kind {
        ConsoleMessageKind::Input => editor_shell::ConsoleLineKind::Input,
        ConsoleMessageKind::Error => editor_shell::ConsoleLineKind::Error,
        ConsoleMessageKind::Warning => editor_shell::ConsoleLineKind::Warning,
        ConsoleMessageKind::Info => editor_shell::ConsoleLineKind::Info,
        ConsoleMessageKind::Debug => editor_shell::ConsoleLineKind::Debug,
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
    tools_menu_open: bool,
    tool_radial_anchor_position: Option<ui_math::UiPoint>,
    debug_stage: editor_viewport::ViewportDebugStage,
    root_background_opaque: bool,
    field_visualizer_settings: ViewportFieldVisualizerSettings,
    selected_entity: Option<EntityId>,
    drag_in_progress: bool,
    tool_state: ViewportToolState,
    source_version: RealityVersion,
    fallback_viewport_id: Option<editor_viewport::ViewportId>,
    frame_metrics: Option<EditorShellFrameMetrics>,
    overlay_status_lines: Vec<String>,
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
        tools_menu_open,
        tool_radial_anchor_position,
        debug_stage,
        root_background_opaque,
        field_visualizer_settings,
        selected_entity,
        hovered_entity: tool_state.hovered_entity,
        drag_in_progress,
        preview_active: tool_state.active_preview.is_some(),
        frame_rate_fps: frame_metrics
            .map(|metrics| metrics.fps_ema)
            .filter(|value| value.is_finite() && *value > 0.0),
        frame_time_ms: frame_metrics
            .map(|metrics| metrics.frame_ms_ema)
            .filter(|value| value.is_finite() && *value > 0.0),
        overlay_status_lines,
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
        tools_menu_open: frame.tools_menu_open,
        tool_radial_anchor_position: frame.tool_radial_anchor_position,
        debug_stage: frame.debug_stage,
        root_background_opaque: frame.root_background_opaque,
        field_visualizer_settings: frame.field_visualizer_settings,
        selected_entity: frame.selected_entity,
        hovered_entity: frame.hovered_entity,
        drag_in_progress: frame.drag_in_progress,
        preview_active: frame.preview_active,
        frame_rate_fps: frame.frame_rate_fps,
        frame_time_ms: frame.frame_time_ms,
        overlay_status_lines: frame.overlay_status_lines.clone(),
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
            | InspectorFieldControlKind::EnumSelect { .. }
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
        InspectorValue::Enum { current, options } => {
            let current = field
                .draft_value
                .as_ref()
                .and_then(|draft| match draft {
                    editor_inspector::InspectorEditValue::EnumSymbol(value) => Some(value.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| current.clone());
            InspectorFieldControlKind::EnumSelect {
                selected_index: options.iter().position(|option| option == &current),
                current,
                options: options.clone(),
            }
        }
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
    if request.matches_stable_key(EDITOR_CONSOLE_SURFACE_KEY)
        || request.matches_any_stable_key(EDITOR_DESIGN_SURFACE_KEYS)
        || request.matches_any_stable_key(&[
            ASSET_BROWSER_SURFACE_KEY,
            IMPORT_INSPECTOR_SURFACE_KEY,
            FIELD_PRODUCT_VIEWER_SURFACE_KEY,
            SDF_BRUSH_BROWSER_SURFACE_KEY,
        ])
        || request.matches_any_stable_key(DIAGNOSTICS_SURFACE_KEYS)
        || request.matches_stable_key(TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY)
        || request.legacy_kind().is_some_and(|kind| {
            kind == ToolSurfaceKind::Console
                || is_self_authoring_surface(kind)
                || is_asset_surface(kind)
                || is_m6_global_diagnostic_surface(kind)
        })
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

fn legacy_surface_kind_or_diagnostic(
    request: &SurfaceProviderRequest,
) -> Result<ToolSurfaceKind, SurfaceProviderDiagnostic> {
    request.legacy_kind().ok_or_else(|| {
        SurfaceProviderDiagnostic::new(
            "editor.surface.missing_legacy_kind",
            format!(
                "surface `{}` is stable-key authoritative and has no legacy compatibility kind for this provider path",
                request.stable_key().as_str()
            ),
        )
    })
}

fn surface_document_context_line(document_context: &SurfaceDocumentContext) -> String {
    match document_context {
        SurfaceDocumentContext::Resolved {
            document_id,
            document_kind,
        } => format!(
            "active document: {} #{}",
            document_kind.stable_name(),
            document_id.0
        ),
        SurfaceDocumentContext::Unresolved { document_id } => {
            format!("active document: unresolved #{}", document_id.0)
        }
        SurfaceDocumentContext::NoActiveDocument => "active document: none".to_string(),
    }
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

fn is_asset_surface(kind: ToolSurfaceKind) -> bool {
    matches!(
        kind,
        ToolSurfaceKind::AssetBrowser
            | ToolSurfaceKind::ImportInspector
            | ToolSurfaceKind::FieldProductViewer
            | ToolSurfaceKind::SdfBrushBrowser
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
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_keys_or_legacy_kind_support(
            request,
            EDITOR_DESIGN_SURFACE_KEYS,
            is_self_authoring_surface,
        )
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let legacy_tool_surface_kind = legacy_surface_kind_or_diagnostic(request)?;
        let title = self_authoring_title(legacy_tool_surface_kind).to_string();
        let (root, routes) = match legacy_tool_surface_kind {
            ToolSurfaceKind::UiCanvas => context
                .shell_state
                .self_authoring()
                .formed_selected_preview_with_scope(
                    context.theme,
                    Some(editor_shell::surface_widget_scope_base(
                        request.tool_surface_instance_id,
                    )),
                )
                .map(|product| (product.root, SurfaceRouteTable::empty()))
                .unwrap_or_else(|| {
                    build_self_authoring_control_panel(
                        context.theme,
                        request.tool_surface_instance_id,
                        vec!["No retained preview available".to_string()],
                        Vec::new(),
                    )
                }),
            _ => build_self_authoring_control_panel(
                context.theme,
                request.tool_surface_instance_id,
                self_authoring_lines(context, legacy_tool_surface_kind),
                self_authoring_actions(context, legacy_tool_surface_kind),
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
    let root_id = surface_widget_id(
        request.tool_surface_instance_id,
        editor_shell::WidgetId(900_000),
    );
    let label_id = surface_widget_id(
        request.tool_surface_instance_id,
        editor_shell::WidgetId(900_001),
    );
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

pub mod asset_browser;
pub mod console;
pub mod field_layer_stack;
pub mod field_product_viewer;
pub mod import_inspector;
pub mod m6_workspace;
pub mod material_graph_canvas;
pub mod material_inspector;
pub mod material_preview;
pub mod procgen_graph_canvas;
pub mod procgen_preview;
pub mod scene;
pub mod sdf_brush_browser;
pub mod sdf_graph_canvas;
pub mod texture_viewer;
pub mod tool_suite_registry_inspector;
pub mod volume_texture_viewer;

use asset_browser::AssetBrowserProvider;
use console::ConsoleProvider;
use field_layer_stack::FieldLayerStackProvider;
use field_product_viewer::FieldProductViewerProvider;
use import_inspector::ImportInspectorProvider;
use m6_workspace::{M6WorkspaceProvider, is_m6_global_diagnostic_surface};
use material_graph_canvas::MaterialGraphCanvasProvider;
use material_inspector::MaterialInspectorProvider;
use material_preview::MaterialPreviewProvider;
use procgen_graph_canvas::ProcgenGraphCanvasProvider;
use procgen_preview::ProcgenPreviewProvider;
use scene::{
    SceneEntityTableProvider, SceneInspectorProvider, SceneOutlinerProvider, SceneViewportProvider,
};
use sdf_brush_browser::SdfBrushBrowserProvider;
use sdf_graph_canvas::SdfGraphCanvasProvider;
use texture_viewer::TextureViewerProvider;
use tool_suite_registry_inspector::ToolSuiteRegistryInspectorProvider;
use volume_texture_viewer::VolumeTextureViewerProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetDiagnosticCode,
        AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetKind, AssetRecord,
        AssetSourceDescriptor, SourceHash, asset_artifact_id, asset_id, asset_source_id,
    };
    use editor_shell::{
        EditorToolSuite, LAYOUT_WORKSPACE_PROFILE_ID, PanelInstanceId, PanelKind,
        ProviderFamilyDefinition, ProviderFamilyId, RUNTIME_DEBUG_WORKSPACE_PROFILE_ID, TabStackId,
        ToolSuiteId, ToolSuiteRegistry, ToolSurfaceDefinition, ToolSurfaceInstanceId,
        ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute, ToolSurfaceStableKey,
        UiNodeKind, VIEWPORT_SURFACE_DEFINITION_ID, WidgetId,
    };
    use graph::{CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId};
    use texture::{
        Ktx2TextureMetadata, TextureDescriptor, TextureDimension, TextureExtent,
        TexturePixelFormat, TextureProductId,
    };

    fn texture_descriptor(
        product_id: u64,
        dimension: TextureDimension,
        extent: TextureExtent,
    ) -> TextureDescriptor {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(product_id),
            format!("texture.{product_id}"),
            dimension,
            extent,
        );
        let mip_count = descriptor.mip_count;
        let descriptor_hash = descriptor.descriptor_hash().to_string();
        descriptor.with_ktx2_metadata(
            Ktx2TextureMetadata::new(
                TexturePixelFormat::Rgba8Unorm,
                mip_count,
                descriptor_hash,
                "1",
            )
            .with_byte_layout(128, [64]),
        )
    }

    fn texture_payload(descriptor: TextureDescriptor) -> ArtifactPayloadKind {
        ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: None,
        }
    }

    fn texture_payload_with_uri(
        descriptor: TextureDescriptor,
        artifact_uri: impl Into<String>,
    ) -> ArtifactPayloadKind {
        ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: Some(artifact_uri.into()),
        }
    }

    fn generated_texture_payload_with_uri(
        descriptor: TextureDescriptor,
        artifact_uri: impl Into<String>,
    ) -> ArtifactPayloadKind {
        ArtifactPayloadKind::GeneratedTextureProduct {
            descriptor_hash: descriptor.descriptor_hash().to_string(),
            descriptor,
            artifact_uri: Some(artifact_uri.into()),
        }
    }

    fn texture_payload_with_hash(
        descriptor: TextureDescriptor,
        descriptor_hash: impl Into<String>,
        artifact_uri: Option<String>,
    ) -> ArtifactPayloadKind {
        ArtifactPayloadKind::TextureProduct {
            descriptor_hash: descriptor_hash.into(),
            descriptor,
            artifact_uri,
        }
    }

    fn texture_descriptor_with_byte_length(
        product_id: u64,
        dimension: TextureDimension,
        extent: TextureExtent,
        byte_length: u64,
    ) -> TextureDescriptor {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(product_id),
            format!("texture.{product_id}"),
            dimension,
            extent,
        );
        let mip_count = descriptor.mip_count;
        let descriptor_hash = descriptor.descriptor_hash().to_string();
        descriptor.with_ktx2_metadata(
            Ktx2TextureMetadata::new(
                TexturePixelFormat::Rgba8Unorm,
                mip_count,
                descriptor_hash,
                "1",
            )
            .with_byte_layout(
                byte_length,
                [extent.width as u64 * extent.height as u64 * extent.depth as u64 * 4],
            ),
        )
    }

    fn texture_descriptor_with_mip_count_and_byte_length(
        product_id: u64,
        dimension: TextureDimension,
        extent: TextureExtent,
        mip_count: u32,
        byte_length: u64,
    ) -> TextureDescriptor {
        let descriptor = TextureDescriptor::new(
            TextureProductId::new(product_id),
            format!("texture.{product_id}"),
            dimension,
            extent,
        )
        .with_mip_count(mip_count);
        let descriptor_hash = descriptor.descriptor_hash().to_string();
        descriptor.with_ktx2_metadata(
            Ktx2TextureMetadata::new(
                TexturePixelFormat::Rgba8Unorm,
                mip_count,
                descriptor_hash,
                "1",
            )
            .with_byte_layout(
                byte_length,
                [extent.width as u64 * extent.height as u64 * extent.depth as u64 * 4],
            ),
        )
    }

    struct DummyProvider {
        descriptor: SurfaceProviderDescriptor,
        supports: bool,
        support_mode: Option<SurfaceProviderSupportMode>,
        fail: bool,
    }

    impl EditorSurfaceProvider for DummyProvider {
        fn descriptor(&self) -> SurfaceProviderDescriptor {
            self.descriptor.clone()
        }

        fn supports(&self, _request: &SurfaceProviderRequest) -> bool {
            self.support_mode
                .map(SurfaceProviderSupportMode::is_supported)
                .unwrap_or(self.supports)
        }

        fn support_mode(&self, _request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
            self.support_mode.unwrap_or_else(|| {
                if self.supports {
                    SurfaceProviderSupportMode::LegacyKind
                } else {
                    SurfaceProviderSupportMode::Unsupported
                }
            })
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
            support_mode: None,
            fail: false,
        })
    }

    fn dummy_with_support_mode(
        id: u64,
        priority: u16,
        support_mode: SurfaceProviderSupportMode,
    ) -> Box<dyn EditorSurfaceProvider> {
        Box::new(DummyProvider {
            descriptor: SurfaceProviderDescriptor::new(
                SurfaceProviderId::try_from_raw(id).unwrap(),
                format!("provider-{id}"),
                SurfaceProviderPriority(priority),
            ),
            supports: support_mode.is_supported(),
            support_mode: Some(support_mode),
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
            support_mode: None,
            fail: true,
        })
    }

    fn request() -> SurfaceProviderRequest {
        let tool_surface_kind = ToolSurfaceKind::Viewport;
        SurfaceProviderRequest {
            workspace_profile_id: LAYOUT_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(1),
                document_kind: DocumentKind::Scene,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(3).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(3).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(3).unwrap(),
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: VIEWPORT_SURFACE_DEFINITION_ID,
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn request_with_document_context(
        document_context: SurfaceDocumentContext,
        tool_surface_kind: ToolSurfaceKind,
    ) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            document_context,
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
            ..request()
        }
    }

    fn request_with_stable_key(
        stable_surface_key: &str,
        tool_surface_kind: ToolSurfaceKind,
    ) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            stable_surface_key: ToolSurfaceStableKey::new(stable_surface_key).unwrap(),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
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
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn m6_material_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::MATERIAL_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(6),
                document_kind: DocumentKind::MaterialGraph,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(20).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(20).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(20).unwrap(),
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn stable_key_only_material_request(
        stable_key: &str,
        route: ToolSurfaceRoute,
    ) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::MATERIAL_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(6),
                document_kind: DocumentKind::MaterialGraph,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(20).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(20).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(20).unwrap(),
            stable_surface_key: ToolSurfaceStableKey::new(stable_key).unwrap(),
            legacy_tool_surface_kind: None,
            provider_family_id: Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap()),
            surface_route: Some(route),
            surface_definition_id: editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID,
            capabilities: Default::default(),
        }
    }

    fn m6_texture_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
        let document_kind = match tool_surface_kind {
            ToolSurfaceKind::VolumeTextureViewer => DocumentKind::VolumeTexture,
            _ => DocumentKind::ProceduralTexture,
        };
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::TEXTURE_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(7),
                document_kind,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(21).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(21).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(21).unwrap(),
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn m6_procgen_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::PROCGEN_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(9),
                document_kind: DocumentKind::ProceduralGenerationGraph,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(23).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(23).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(23).unwrap(),
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn m6_sdf_request(
        tool_surface_kind: ToolSurfaceKind,
        document_kind: DocumentKind,
    ) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::FIELD_WORLD_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(8),
                document_kind,
            },
            panel_instance_id: PanelInstanceId::try_from_raw(22).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(22).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(22).unwrap(),
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn asset_request(tool_surface_kind: ToolSurfaceKind) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: editor_shell::FIELD_WORLD_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::NoActiveDocument,
            panel_instance_id: PanelInstanceId::try_from_raw(30).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(30).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(30).unwrap(),
            stable_surface_key: stable_key_for_test(tool_surface_kind),
            legacy_tool_surface_kind: Some(tool_surface_kind),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: tool_surface_definition_id(tool_surface_kind),
            capabilities: tool_surface_capability_set(tool_surface_kind),
        }
    }

    fn inspector_request() -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            workspace_profile_id: RUNTIME_DEBUG_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::NoActiveDocument,
            panel_instance_id: PanelInstanceId::try_from_raw(31).unwrap(),
            tab_stack_id: TabStackId::try_from_raw(31).unwrap(),
            tool_surface_instance_id: ToolSurfaceInstanceId::try_from_raw(31).unwrap(),
            stable_surface_key: ToolSurfaceStableKey::new(
                TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
            )
            .unwrap(),
            legacy_tool_surface_kind: None,
            provider_family_id: Some(ProviderFamilyId::new("runenwerk.diagnostics").unwrap()),
            surface_route: Some(ToolSurfaceRoute::ProviderOwnedLocal),
            surface_definition_id: editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID,
            capabilities: Default::default(),
        }
    }

    fn stable_key_for_test(tool_surface_kind: ToolSurfaceKind) -> ToolSurfaceStableKey {
        editor_shell::stable_key_for_tool_surface_kind(tool_surface_kind)
            .expect("provider test fixture surface should have a stable key")
    }

    fn scene_viewport_tool_suite_registry() -> ToolSuiteRegistry {
        let provider_family_id = ProviderFamilyId::new("runenwerk.scene").unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new("runenwerk.scene").unwrap(),
            label: "Scene".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family_id.clone(),
                label: "Scene".to_string(),
            }],
            surfaces: vec![ToolSurfaceDefinition {
                key: ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap(),
                label: "Viewport".to_string(),
                role: ToolSurfaceRole::Primary,
                panel_kind: PanelKind::Viewport,
                provider_family: provider_family_id,
                route: ToolSurfaceRoute::ProviderOwnedLocal,
                persistence: ToolSurfacePersistence::StableKey,
            }],
        }])
        .expect("scene viewport fixture should be valid")
    }

    fn tool_suite_registry_for_provider_family(provider_family_id: &str) -> ToolSuiteRegistry {
        let provider_family_id = ProviderFamilyId::new(provider_family_id).unwrap();
        ToolSuiteRegistry::new(vec![EditorToolSuite {
            suite_id: ToolSuiteId::new(provider_family_id.as_str()).unwrap(),
            label: "Fixture".to_string(),
            provider_families: vec![ProviderFamilyDefinition {
                id: provider_family_id.clone(),
                label: "Fixture".to_string(),
            }],
            surfaces: vec![ToolSurfaceDefinition {
                key: ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap(),
                label: "Viewport".to_string(),
                role: ToolSurfaceRole::Primary,
                panel_kind: PanelKind::Viewport,
                provider_family: provider_family_id,
                route: ToolSurfaceRoute::ProviderOwnedLocal,
                persistence: ToolSurfacePersistence::StableKey,
            }],
        }])
        .expect("provider family fixture should be valid")
    }

    fn provider_family_map(
        registry: &ToolSuiteRegistry,
        provider_family_id: &str,
        provider_ids: &[u64],
    ) -> ProviderFamilyProviderMap {
        let provider_family_id = ProviderFamilyId::new(provider_family_id).unwrap();
        ProviderFamilyProviderMap::new(
            registry,
            provider_ids
                .iter()
                .copied()
                .map(|id| {
                    ProviderFamilyProviderAssignment::new(
                        provider_family_id.clone(),
                        SurfaceProviderId::try_from_raw(id).unwrap(),
                    )
                })
                .collect(),
        )
        .expect("provider family map fixture should be valid")
    }

    fn request_for_provider_family(provider_family_id: &str) -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            provider_family_id: Some(ProviderFamilyId::new(provider_family_id).unwrap()),
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
            frame_metrics: None,
            viewport_observations: None,
            tool_surface_bindings: None,
            viewport_instances: None,
        }
    }

    fn provider_frame_text(frame: &ResolvedSurfaceFrame) -> String {
        format!("{:?}", frame.artifact.root)
    }

    fn frame_has_product_surface(frame: &ResolvedSurfaceFrame) -> bool {
        fn walk(node: &editor_shell::UiNode) -> bool {
            matches!(node.kind, UiNodeKind::ProductSurface(_)) || node.children.iter().any(walk)
        }
        walk(&frame.artifact.root)
    }

    fn build_rgba8_ktx2(
        width: u32,
        height: u32,
        depth: u32,
        slice0_texel: [u8; 4],
        slice1_texel: [u8; 4],
    ) -> Vec<u8> {
        let format = ktx2::Format::R8G8B8A8_UNORM;
        let (basic, type_size) =
            ktx2::dfd::Basic::from_format(format).expect("rgba8 dfd should build");
        let dfd_block = ktx2::dfd::Block::Basic(basic);
        let dfd_block_bytes = dfd_block.to_vec();
        let dfd_total_size = 4 + dfd_block_bytes.len();
        let level_index_offset = ktx2::Header::LENGTH;
        let dfd_offset = level_index_offset + ktx2::LevelIndex::LENGTH;
        let after_dfd = dfd_offset + dfd_total_size;
        let level_data_offset = (after_dfd + 3) / 4 * 4;
        let texel_count = width as usize * height as usize * depth.max(1) as usize;
        let level_data_size = texel_count * 4;
        let mut bytes = vec![0u8; level_data_offset + level_data_size];

        let header = ktx2::Header {
            format: Some(format),
            type_size,
            pixel_width: width,
            pixel_height: height,
            pixel_depth: if depth > 1 { depth } else { 0 },
            layer_count: 0,
            face_count: 1,
            level_count: 1,
            supercompression_scheme: None,
            index: ktx2::Index {
                dfd_byte_offset: dfd_offset as u32,
                dfd_byte_length: dfd_total_size as u32,
                kvd_byte_offset: 0,
                kvd_byte_length: 0,
                sgd_byte_offset: 0,
                sgd_byte_length: 0,
            },
        };
        bytes[..ktx2::Header::LENGTH].copy_from_slice(&header.as_bytes());
        let index = ktx2::LevelIndex {
            byte_offset: level_data_offset as u64,
            byte_length: level_data_size as u64,
            uncompressed_byte_length: level_data_size as u64,
        };
        bytes[level_index_offset..level_index_offset + ktx2::LevelIndex::LENGTH]
            .copy_from_slice(&index.as_bytes());
        bytes[dfd_offset..dfd_offset + 4].copy_from_slice(&(dfd_total_size as u32).to_le_bytes());
        bytes[dfd_offset + 4..dfd_offset + 4 + dfd_block_bytes.len()]
            .copy_from_slice(&dfd_block_bytes);
        let data = &mut bytes[level_data_offset..level_data_offset + level_data_size];
        let texels_per_slice = width as usize * height as usize;
        for (index, pixel) in data.chunks_exact_mut(4).enumerate() {
            let texel = if index / texels_per_slice == 0 {
                slice0_texel
            } else {
                slice1_texel
            };
            pixel.copy_from_slice(&texel);
        }
        bytes
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
    fn mounted_surface_request_includes_advisory_stable_key_when_available() {
        let shell_state = RunenwerkEditorShellState::new();
        let requests =
            mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);

        let viewport_request = requests
            .iter()
            .find(|request| request.legacy_kind() == Some(ToolSurfaceKind::Viewport))
            .expect("default workspace should mount viewport");

        assert_eq!(
            viewport_request.stable_key().as_str(),
            "runenwerk.scene.viewport"
        );
        assert_eq!(viewport_request.provider_family_id, None);
        assert_eq!(viewport_request.surface_route, None);
    }

    #[test]
    fn live_mounted_surface_requests_use_stable_key_authority() {
        let shell_state = RunenwerkEditorShellState::new();
        let requests =
            mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);

        let viewport_request = requests
            .iter()
            .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
            .expect("default workspace should mount viewport by stable key");

        assert_eq!(
            viewport_request.stable_key().as_str(),
            SCENE_VIEWPORT_SURFACE_KEY
        );
    }

    #[test]
    fn live_mounted_surface_requests_include_legacy_kind_only_as_metadata() {
        let shell_state = RunenwerkEditorShellState::new();
        let requests =
            mounted_surface_requests(&shell_state, SurfaceDocumentContext::NoActiveDocument);

        let viewport_request = requests
            .iter()
            .find(|request| request.matches_stable_key(SCENE_VIEWPORT_SURFACE_KEY))
            .expect("default workspace should mount viewport by stable key");

        assert_eq!(
            viewport_request.legacy_kind(),
            Some(ToolSurfaceKind::Viewport)
        );
    }

    #[test]
    fn mounted_surface_request_enrichment_adds_provider_family_and_route_when_registry_resolves() {
        let shell_state = RunenwerkEditorShellState::new();
        let registry = scene_viewport_tool_suite_registry();

        let requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(registry.surfaces()),
        );

        let viewport_request = requests
            .iter()
            .find(|request| request.legacy_kind() == Some(ToolSurfaceKind::Viewport))
            .expect("default workspace should mount viewport");

        assert_eq!(
            viewport_request.stable_key().as_str(),
            "runenwerk.scene.viewport"
        );
        assert_eq!(
            viewport_request
                .provider_family_id
                .as_ref()
                .map(ProviderFamilyId::as_str),
            Some("runenwerk.scene")
        );
        assert_eq!(
            viewport_request.surface_route,
            Some(ToolSurfaceRoute::ProviderOwnedLocal)
        );
    }

    #[test]
    fn mounted_surface_request_without_registry_matches_legacy_behavior() {
        let shell_state = RunenwerkEditorShellState::new();
        let document_context = SurfaceDocumentContext::NoActiveDocument;

        let legacy_requests = mounted_surface_requests(&shell_state, document_context.clone());
        let explicit_requests =
            mounted_surface_requests_with_registry(&shell_state, document_context, None);

        assert_eq!(legacy_requests, explicit_requests);
        assert!(
            legacy_requests
                .iter()
                .all(|request| request.provider_family_id.is_none()
                    && request.surface_route.is_none())
        );
    }

    #[test]
    fn provider_resolution_unchanged_when_metadata_is_present_but_providers_ignore_it() {
        let registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true), dummy(2, 10, false)])
                .expect("ids are unique");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let mut enriched_request = request();
        enriched_request.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.scene.viewport").unwrap();
        enriched_request.provider_family_id =
            Some(ProviderFamilyId::new("runenwerk.scene").unwrap());
        enriched_request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedLocal);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &enriched_request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(1).unwrap())
        );
    }

    #[test]
    fn provider_family_filtering_limits_candidates_before_supports() {
        let provider_registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 1, true), dummy(2, 200, true)])
                .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request_for_provider_family("runenwerk.scene"),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(2).unwrap())
        );
    }

    #[test]
    fn provider_family_filtering_still_runs_before_stable_key_matching() {
        let provider_registry = EditorSurfaceProviderRegistry::new(vec![
            dummy_with_support_mode(1, 1, SurfaceProviderSupportMode::StableKey),
            dummy_with_support_mode(2, 10, SurfaceProviderSupportMode::StableKey),
        ])
        .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request_for_provider_family("runenwerk.scene"),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(2).unwrap())
        );
    }

    #[test]
    fn provider_family_filtering_preserves_existing_priority_resolution() {
        let provider_registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true), dummy(2, 10, true)])
                .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request_for_provider_family("runenwerk.scene"),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(2).unwrap())
        );
    }

    #[test]
    fn missing_provider_family_assignment_fails_closed() {
        let provider_registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 1, true)]).expect("ids are unique");
        let suite_registry = tool_suite_registry_for_provider_family("runenwerk.scene");
        let provider_family_map =
            ProviderFamilyProviderMap::new(&suite_registry, Vec::new()).unwrap();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request_for_provider_family("runenwerk.scene"),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert_eq!(frame.provider_id, None);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn request_without_provider_family_id_keeps_legacy_full_candidate_behavior() {
        let provider_registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 1, true), dummy(2, 200, true)])
                .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request(),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(1).unwrap())
        );
    }

    #[test]
    fn provider_family_filtering_does_not_change_material_graph_provider_resolution() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
        request.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.material_lab.graph_canvas").unwrap();
        request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap());
        request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas);

        let legacy_frame = host.provider_registry().resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );
        let filtered_frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &Default::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(filtered_frame.availability, legacy_frame.availability);
        assert_eq!(filtered_frame.provider_id, legacy_frame.provider_id);
        assert_eq!(
            filtered_frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(12).unwrap())
        );
    }

    #[test]
    fn equal_priority_ambiguity_still_fails_closed_after_family_filtering() {
        let provider_registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true), dummy(2, 100, true)])
                .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request_for_provider_family("runenwerk.scene"),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Ambiguous);
        assert_eq!(frame.provider_id, None);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn provider_family_filtering_keeps_provider_supports_enum_backed() {
        let provider_registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, false)]).expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request_for_provider_family("runenwerk.scene"),
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert_eq!(frame.provider_id, None);
    }

    #[test]
    fn material_graph_provider_supports_stable_key_first() {
        let provider = MaterialGraphCanvasProvider;
        let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
        request.stable_surface_key =
            ToolSurfaceStableKey::new(MATERIAL_GRAPH_CANVAS_SURFACE_KEY).unwrap();

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::StableKey
        );
    }

    #[test]
    fn material_graph_provider_legacy_support_still_works_when_stable_key_does_not_match() {
        let provider = MaterialGraphCanvasProvider;
        let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
        request.stable_surface_key = ToolSurfaceStableKey::new("runenwerk.fixture.other").unwrap();

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::LegacyKind
        );
    }

    #[test]
    fn material_inspector_provider_supports_stable_key_first() {
        let provider = MaterialInspectorProvider;
        let mut request = m6_material_request(ToolSurfaceKind::MaterialInspector);
        request.stable_surface_key =
            ToolSurfaceStableKey::new(MATERIAL_INSPECTOR_SURFACE_KEY).unwrap();

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::StableKey
        );
    }

    #[test]
    fn material_preview_provider_supports_stable_key_first() {
        let provider = MaterialPreviewProvider;
        let mut request = m6_material_request(ToolSurfaceKind::MaterialPreview);
        request.stable_surface_key =
            ToolSurfaceStableKey::new(MATERIAL_PREVIEW_SURFACE_KEY).unwrap();

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::StableKey
        );
    }

    #[test]
    fn material_lab_providers_do_not_require_legacy_tool_surface_kind() {
        let cases = [
            (
                &MaterialGraphCanvasProvider as &dyn EditorSurfaceProvider,
                stable_key_only_material_request(
                    MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
                    ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                ),
            ),
            (
                &MaterialInspectorProvider,
                stable_key_only_material_request(
                    MATERIAL_INSPECTOR_SURFACE_KEY,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
            ),
            (
                &MaterialPreviewProvider,
                stable_key_only_material_request(
                    MATERIAL_PREVIEW_SURFACE_KEY,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
            ),
        ];

        for (provider, request) in cases {
            assert_eq!(request.legacy_kind(), None);
            assert_eq!(
                provider.support_mode(&request),
                SurfaceProviderSupportMode::StableKey
            );
            assert!(provider.supports(&request));
        }
    }

    #[test]
    fn material_lab_provider_resolution_uses_stable_keys() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let cases = [
            (
                stable_key_only_material_request(
                    MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
                    ToolSurfaceRoute::ProviderOwnedGraphCanvas,
                ),
                MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
            ),
            (
                stable_key_only_material_request(
                    MATERIAL_INSPECTOR_SURFACE_KEY,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
                MATERIAL_INSPECTOR_PROVIDER_ID,
            ),
            (
                stable_key_only_material_request(
                    MATERIAL_PREVIEW_SURFACE_KEY,
                    ToolSurfaceRoute::ProviderOwnedLocal,
                ),
                MATERIAL_PREVIEW_PROVIDER_ID,
            ),
        ];

        for (request, expected_provider_id) in cases {
            let frame = host
                .provider_registry()
                .resolve_frame_with_provider_family_map(
                    &context(&app, &shell_state, &theme),
                    &request,
                    &SurfaceSessionState::default(),
                    Some(host.provider_family_provider_map()),
                );

            assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
            assert_eq!(frame.provider_id, Some(expected_provider_id));
            assert_eq!(frame.stable_surface_key, request.stable_surface_key);
            assert_eq!(frame.surface_kind, None);
        }
    }

    #[test]
    fn texture_providers_support_registered_stable_keys() {
        let texture_provider = TextureViewerProvider;
        let mut texture_request = m6_texture_request(ToolSurfaceKind::TextureViewer);
        texture_request.stable_surface_key =
            ToolSurfaceStableKey::new(TEXTURE_VIEWER_2D_SURFACE_KEY).unwrap();

        let volume_provider = VolumeTextureViewerProvider;
        let mut volume_request = m6_texture_request(ToolSurfaceKind::VolumeTextureViewer);
        volume_request.stable_surface_key =
            ToolSurfaceStableKey::new(TEXTURE_VIEWER_3D_SURFACE_KEY).unwrap();

        assert_eq!(
            texture_provider.support_mode(&texture_request),
            SurfaceProviderSupportMode::StableKey
        );
        assert_eq!(
            volume_provider.support_mode(&volume_request),
            SurfaceProviderSupportMode::StableKey
        );
    }

    #[test]
    fn asset_providers_support_registered_stable_keys() {
        let asset_provider = AssetBrowserProvider;
        let mut asset_browser_request = asset_request(ToolSurfaceKind::AssetBrowser);
        asset_browser_request.stable_surface_key =
            ToolSurfaceStableKey::new(ASSET_BROWSER_SURFACE_KEY).unwrap();

        let import_provider = ImportInspectorProvider;
        let mut import_request = asset_request(ToolSurfaceKind::ImportInspector);
        import_request.stable_surface_key =
            ToolSurfaceStableKey::new(IMPORT_INSPECTOR_SURFACE_KEY).unwrap();

        assert_eq!(
            asset_provider.support_mode(&asset_browser_request),
            SurfaceProviderSupportMode::StableKey
        );
        assert_eq!(
            import_provider.support_mode(&import_request),
            SurfaceProviderSupportMode::StableKey
        );
    }

    #[test]
    fn console_and_editor_design_providers_support_registered_stable_keys() {
        let console_provider = ConsoleProvider;
        let console_request =
            request_with_stable_key(EDITOR_CONSOLE_SURFACE_KEY, ToolSurfaceKind::Console);

        let self_authoring_provider = SelfAuthoringProvider;
        let mut definition_validation_request =
            self_authoring_request(ToolSurfaceKind::DefinitionValidation);
        definition_validation_request.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.editor_design.definition_validation").unwrap();

        assert_eq!(
            console_provider.support_mode(&console_request),
            SurfaceProviderSupportMode::StableKey
        );
        assert_eq!(
            self_authoring_provider.support_mode(&definition_validation_request),
            SurfaceProviderSupportMode::StableKey
        );
    }

    #[test]
    fn field_and_procgen_providers_support_registered_stable_keys() {
        let field_product_provider = FieldProductViewerProvider;
        let mut field_product_request = asset_request(ToolSurfaceKind::FieldProductViewer);
        field_product_request.stable_surface_key =
            ToolSurfaceStableKey::new(FIELD_PRODUCT_VIEWER_SURFACE_KEY).unwrap();

        let sdf_brush_provider = SdfBrushBrowserProvider;
        let mut sdf_brush_request = asset_request(ToolSurfaceKind::SdfBrushBrowser);
        sdf_brush_request.stable_surface_key =
            ToolSurfaceStableKey::new(SDF_BRUSH_BROWSER_SURFACE_KEY).unwrap();

        let field_layer_provider = FieldLayerStackProvider;
        let mut field_layer_request =
            m6_sdf_request(ToolSurfaceKind::FieldLayerStack, DocumentKind::SdfGraph);
        field_layer_request.stable_surface_key =
            ToolSurfaceStableKey::new(FIELD_LAYER_STACK_SURFACE_KEY).unwrap();

        let sdf_graph_provider = SdfGraphCanvasProvider;
        let mut sdf_graph_request =
            m6_sdf_request(ToolSurfaceKind::SdfGraphCanvas, DocumentKind::SdfGraph);
        sdf_graph_request.stable_surface_key =
            ToolSurfaceStableKey::new(SDF_GRAPH_CANVAS_SURFACE_KEY).unwrap();

        let procgen_graph_provider = ProcgenGraphCanvasProvider;
        let mut procgen_graph_request = m6_procgen_request(ToolSurfaceKind::ProcgenGraphCanvas);
        procgen_graph_request.stable_surface_key =
            ToolSurfaceStableKey::new(PROCGEN_GRAPH_CANVAS_SURFACE_KEY).unwrap();

        let procgen_preview_provider = ProcgenPreviewProvider;
        let mut procgen_preview_request = m6_procgen_request(ToolSurfaceKind::ProcgenPreview);
        procgen_preview_request.stable_surface_key =
            ToolSurfaceStableKey::new(PROCGEN_PREVIEW_SURFACE_KEY).unwrap();

        for (provider, request) in [
            (
                &field_product_provider as &dyn EditorSurfaceProvider,
                &field_product_request,
            ),
            (&sdf_brush_provider, &sdf_brush_request),
            (&field_layer_provider, &field_layer_request),
            (&sdf_graph_provider, &sdf_graph_request),
            (&procgen_graph_provider, &procgen_graph_request),
            (&procgen_preview_provider, &procgen_preview_request),
        ] {
            assert_eq!(
                provider.support_mode(request),
                SurfaceProviderSupportMode::StableKey
            );
        }
    }

    #[test]
    fn scene_core_providers_support_registered_stable_keys() {
        let providers = [
            (
                Box::new(SceneOutlinerProvider) as Box<dyn EditorSurfaceProvider>,
                SCENE_OUTLINER_SURFACE_KEY,
                ToolSurfaceKind::Outliner,
            ),
            (
                Box::new(SceneEntityTableProvider) as Box<dyn EditorSurfaceProvider>,
                SCENE_ENTITY_TABLE_SURFACE_KEY,
                ToolSurfaceKind::EntityTable,
            ),
            (
                Box::new(SceneViewportProvider) as Box<dyn EditorSurfaceProvider>,
                SCENE_VIEWPORT_SURFACE_KEY,
                ToolSurfaceKind::Viewport,
            ),
            (
                Box::new(SceneInspectorProvider) as Box<dyn EditorSurfaceProvider>,
                SCENE_INSPECTOR_SURFACE_KEY,
                ToolSurfaceKind::Inspector,
            ),
        ];

        for (provider, stable_key, kind) in providers {
            let request = request_with_stable_key(stable_key, kind);
            assert_eq!(
                provider.support_mode(&request),
                SurfaceProviderSupportMode::StableKey
            );
        }
    }

    #[test]
    fn provider_matching_constants_match_registered_suite_keys() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let constants = [
            &[SCENE_OUTLINER_SURFACE_KEY][..],
            &[SCENE_ENTITY_TABLE_SURFACE_KEY],
            &[SCENE_VIEWPORT_SURFACE_KEY],
            &[SCENE_INSPECTOR_SURFACE_KEY],
            &[EDITOR_CONSOLE_SURFACE_KEY],
            EDITOR_DESIGN_SURFACE_KEYS,
            &[ASSET_BROWSER_SURFACE_KEY],
            &[IMPORT_INSPECTOR_SURFACE_KEY],
            &[FIELD_PRODUCT_VIEWER_SURFACE_KEY],
            &[SDF_BRUSH_BROWSER_SURFACE_KEY],
            &[FIELD_LAYER_STACK_SURFACE_KEY],
            &[SDF_GRAPH_CANVAS_SURFACE_KEY],
            DIAGNOSTICS_SURFACE_KEYS,
            &[MATERIAL_GRAPH_CANVAS_SURFACE_KEY],
            &[MATERIAL_INSPECTOR_SURFACE_KEY],
            &[MATERIAL_PREVIEW_SURFACE_KEY],
            &[TEXTURE_VIEWER_2D_SURFACE_KEY],
            &[TEXTURE_VIEWER_3D_SURFACE_KEY],
            &[PROCGEN_GRAPH_CANVAS_SURFACE_KEY],
            &[PROCGEN_PREVIEW_SURFACE_KEY],
            &[TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY],
        ];

        for stable_key in constants.into_iter().flatten() {
            let stable_key = ToolSurfaceStableKey::new(*stable_key).unwrap();
            assert!(
                host.tool_surface_registry().get(&stable_key).is_some(),
                "provider matching constant should be registered: {}",
                stable_key.as_str()
            );
        }
    }

    #[test]
    fn tool_suite_registry_inspector_provider_is_registered() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();

        assert!(registry.has_provider_id(TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID));
        assert!(
            registry
                .provider_descriptors()
                .any(
                    |descriptor| descriptor.id == TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID
                        && descriptor.label == "Tool Suite Registry Inspector"
                )
        );
    }

    #[test]
    fn inspector_provider_supports_stable_key_only() {
        let provider = ToolSuiteRegistryInspectorProvider;
        let request = inspector_request();

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::StableKey
        );
        assert!(provider.supports(&request));
    }

    #[test]
    fn inspector_provider_does_not_support_legacy_kind() {
        let provider = ToolSuiteRegistryInspectorProvider;
        let mut request = inspector_request();
        request.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.diagnostics.diagnostics").unwrap();
        request.legacy_tool_surface_kind = Some(ToolSurfaceKind::Diagnostics);

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::Unsupported
        );
        assert!(!provider.supports(&request));
    }

    #[test]
    fn inspector_surface_can_be_resolved_by_stable_key() {
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = inspector_request();

        let frame = app
            .workbench_host()
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &SurfaceSessionState::default(),
                Some(app.workbench_host().provider_family_provider_map()),
            );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID)
        );
        assert_eq!(frame.title, "Tool Suite Registry Inspector");
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn inspector_resolution_observation_matches_provider_resolution_for_material_lab() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
        request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.material_lab").unwrap());
        request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedGraphCanvas);

        let observation = host
            .provider_registry()
            .observe_resolution_for_request(&request, Some(host.provider_family_provider_map()));
        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &SurfaceSessionState::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(observation.availability, frame.availability);
        assert_eq!(observation.selected_provider_id, frame.provider_id);
        assert!(
            observation
                .candidate_provider_ids
                .contains(&MATERIAL_GRAPH_CANVAS_PROVIDER_ID)
        );
        assert!(observation.support_modes.iter().any(|row| {
            row.provider_id == MATERIAL_GRAPH_CANVAS_PROVIDER_ID
                && row.support_mode == SurfaceProviderSupportMode::StableKey
        }));
    }

    #[test]
    fn inspector_resolution_observation_matches_provider_resolution_for_diagnostics_inspector() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = inspector_request();

        let observation = host
            .provider_registry()
            .observe_resolution_for_request(&request, Some(host.provider_family_provider_map()));
        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &SurfaceSessionState::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(observation.availability, frame.availability);
        assert_eq!(observation.selected_provider_id, frame.provider_id);
        assert_eq!(
            observation.selected_provider_id,
            Some(TOOL_SUITE_REGISTRY_INSPECTOR_PROVIDER_ID)
        );
    }

    #[test]
    fn unresolved_mounted_surface_reports_diagnostic_without_mutation() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let provider_count_before = host.provider_registry().provider_ids().count();
        let assignment_count_before = host.provider_family_provider_map().assignments().len();
        let mut request = request();
        request.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.gameplay.graph_canvas").unwrap();
        request.legacy_tool_surface_kind = None;
        request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.gameplay").unwrap());
        request.surface_route = Some(ToolSurfaceRoute::ProviderOwnedLocal);

        let observation = host
            .provider_registry()
            .observe_resolution_for_request(&request, Some(host.provider_family_provider_map()));

        assert_eq!(
            observation.availability,
            SurfaceProviderAvailability::Unsupported
        );
        assert_eq!(observation.selected_provider_id, None);
        assert!(observation.diagnostic.is_some_and(|diagnostic| {
            diagnostic.code == "editor.surface.unassigned_provider_family"
        }));
        assert_eq!(
            host.provider_registry().provider_ids().count(),
            provider_count_before
        );
        assert_eq!(
            host.provider_family_provider_map().assignments().len(),
            assignment_count_before
        );
    }

    #[test]
    fn no_new_tool_surface_kind_for_inspector() {
        let state_source =
            include_str!("../../../../../domain/editor/editor_shell/src/workspace/state.rs");
        let shell_source = include_str!(
            "../../../../../domain/editor/editor_shell/src/composition/build_editor_shell.rs"
        );

        assert!(!state_source.contains("ToolSuiteRegistryInspector"));
        assert!(!shell_source.contains(TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY));
    }

    #[test]
    fn no_dynamic_plugin_behavior_introduced() {
        let inspector_source = include_str!("tool_suite_registry_inspector.rs");
        let diagnostics_suite_source = include_str!("../tool_suites/diagnostics_tool_suite.rs");

        for source in [inspector_source, diagnostics_suite_source] {
            assert!(!source.contains("runtime::plugin"));
            assert!(!source.contains("dynamic plugin"));
            assert!(!source.contains("PluginMarketplace"));
        }
    }

    #[test]
    fn provider_resolution_prefers_stable_key_support() {
        let provider_registry = EditorSurfaceProviderRegistry::new(vec![
            dummy_with_support_mode(1, 1, SurfaceProviderSupportMode::LegacyKind),
            dummy_with_support_mode(2, 200, SurfaceProviderSupportMode::StableKey),
        ])
        .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = SurfaceProviderRequest {
            stable_surface_key: ToolSurfaceStableKey::new(SCENE_VIEWPORT_SURFACE_KEY).unwrap(),
            provider_family_id: Some(ProviderFamilyId::new("runenwerk.scene").unwrap()),
            ..request()
        };

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(2).unwrap())
        );
    }

    #[test]
    fn provider_resolution_preserves_legacy_support_when_stable_key_does_not_match() {
        let provider_registry = EditorSurfaceProviderRegistry::new(vec![
            dummy_with_support_mode(1, 200, SurfaceProviderSupportMode::LegacyKind),
            dummy_with_support_mode(2, 10, SurfaceProviderSupportMode::LegacyKind),
        ])
        .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map = provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let mut request = request_for_provider_family("runenwerk.scene");
        request.stable_surface_key = ToolSurfaceStableKey::new("runenwerk.fixture.other").unwrap();

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(
            frame.provider_id,
            Some(SurfaceProviderId::try_from_raw(2).unwrap())
        );
    }

    #[test]
    fn provider_resolution_works_without_legacy_kind_for_stable_key_supported_surface() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let mut request =
            request_with_stable_key(SCENE_VIEWPORT_SURFACE_KEY, ToolSurfaceKind::Viewport);
        request.legacy_tool_surface_kind = None;
        request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.scene").unwrap());

        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &Default::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(SCENE_VIEWPORT_PROVIDER_ID));
    }

    #[test]
    fn provider_resolution_falls_back_to_legacy_only_when_legacy_kind_present() {
        let provider = MaterialGraphCanvasProvider;
        let mut request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
        request.stable_surface_key = ToolSurfaceStableKey::new("runenwerk.fixture.other").unwrap();

        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::LegacyKind
        );

        request.legacy_tool_surface_kind = None;
        assert_eq!(
            provider.support_mode(&request),
            SurfaceProviderSupportMode::Unsupported
        );
    }

    #[test]
    fn stable_key_ambiguity_fails_closed() {
        let provider_registry = EditorSurfaceProviderRegistry::new(vec![
            dummy_with_support_mode(1, 100, SurfaceProviderSupportMode::StableKey),
            dummy_with_support_mode(2, 100, SurfaceProviderSupportMode::StableKey),
            dummy_with_support_mode(3, 1, SurfaceProviderSupportMode::LegacyKind),
        ])
        .expect("ids are unique");
        let suite_registry = scene_viewport_tool_suite_registry();
        let provider_family_map =
            provider_family_map(&suite_registry, "runenwerk.scene", &[1, 2, 3]);
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = SurfaceProviderRequest {
            stable_surface_key: ToolSurfaceStableKey::new(SCENE_VIEWPORT_SURFACE_KEY).unwrap(),
            provider_family_id: Some(ProviderFamilyId::new("runenwerk.scene").unwrap()),
            ..request()
        };

        let frame = provider_registry.resolve_frame_with_provider_family_map(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
            Some(&provider_family_map),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Ambiguous);
        assert_eq!(frame.provider_id, None);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn future_placeholder_families_do_not_gain_provider_support() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let mut request = m6_procgen_request(ToolSurfaceKind::GameplayGraphCanvas);
        request.stable_surface_key =
            ToolSurfaceStableKey::new("runenwerk.gameplay.graph_canvas").unwrap();
        request.provider_family_id = Some(ProviderFamilyId::new("runenwerk.gameplay").unwrap());

        let frame = host
            .provider_registry()
            .resolve_frame_with_provider_family_map(
                &context(&app, &shell_state, &theme),
                &request,
                &Default::default(),
                Some(host.provider_family_provider_map()),
            );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert_eq!(frame.provider_id, None);
        assert!(frame.routes.is_empty());
    }

    #[test]
    fn live_mounted_surface_requests_include_provider_family_when_registry_resolves() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let shell_state = RunenwerkEditorShellState::new();
        let requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(host.tool_surface_registry()),
        );

        let viewport_request = requests
            .iter()
            .find(|request| request.legacy_kind() == Some(ToolSurfaceKind::Viewport))
            .expect("default workspace should mount viewport");

        assert_eq!(
            viewport_request
                .provider_family_id
                .as_ref()
                .map(ProviderFamilyId::as_str),
            Some("runenwerk.scene")
        );
        assert_eq!(
            viewport_request.surface_route,
            Some(ToolSurfaceRoute::ProviderOwnedLocal)
        );
    }

    #[test]
    fn unresolved_registry_surface_request_reports_diagnostic_in_live_frame_path() {
        let app = RunenwerkEditorApp::with_surface_provider_registry(
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 100, true)]).expect("ids are unique"),
        );
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame_model = build_editor_shell_frame_model(
            &app,
            &shell_state,
            app.surface_provider_registry(),
            &theme,
            None,
            None,
            None,
        );

        let viewport_frame = frame_model
            .surfaces
            .values()
            .find(|frame| frame.surface_kind == Some(ToolSurfaceKind::Viewport))
            .expect("default workspace should include viewport frame");

        assert_eq!(
            viewport_frame.availability,
            SurfaceProviderAvailability::Unsupported
        );
        assert_eq!(viewport_frame.provider_id, None);
        assert!(viewport_frame.routes.is_empty());
    }

    #[test]
    fn request_still_contains_legacy_tool_surface_kind_and_capabilities() {
        let host = crate::shell::RunenwerkWorkbenchHost::new().expect("host should build");
        let shell_state = RunenwerkEditorShellState::new();
        let requests = mounted_surface_requests_with_registry(
            &shell_state,
            SurfaceDocumentContext::NoActiveDocument,
            Some(host.tool_surface_registry()),
        );
        let request = requests
            .iter()
            .find(|request| request.legacy_kind() == Some(ToolSurfaceKind::Viewport))
            .expect("default workspace should mount viewport");

        assert_eq!(request.legacy_kind(), Some(ToolSurfaceKind::Viewport));
        assert_eq!(
            request.surface_definition_id,
            tool_surface_definition_id(ToolSurfaceKind::Viewport)
        );
        assert_eq!(
            request.capabilities,
            tool_surface_capability_set(ToolSurfaceKind::Viewport)
        );
    }

    #[test]
    fn non_material_providers_ignore_graph_canvas_interaction_by_default() {
        let registry =
            EditorSurfaceProviderRegistry::new(vec![dummy(1, 200, true)]).expect("ids are unique");
        let request = request();
        let proposal = registry
            .map_interaction(
                &SurfaceProviderDispatchContext {
                    projection_epoch: 61,
                    _marker: std::marker::PhantomData,
                },
                &request,
                SurfaceProviderId::try_from_raw(1).unwrap(),
                SurfaceInteraction::GraphCanvasAction(
                    ui_graph_editor::GraphCanvasAction::ClearSelection,
                ),
            )
            .expect("default provider interaction mapper should not fail");

        assert_eq!(proposal, None);
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

    #[test]
    fn material_graph_canvas_provider_resolves_descriptor_surface_with_material_routes() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(MATERIAL_GRAPH_CANVAS_PROVIDER_ID));
        assert_eq!(frame.title, "Material Graph Canvas");
        assert!(
            provider_frame_text(&frame).contains("domain/material_graph remains material truth")
        );
        assert!(!frame.routes.is_empty());
    }

    #[test]
    fn material_graph_canvas_view_model_exposes_structured_diagnostics() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        app.material_lab_runtime_mut().record_diagnostic(
            AssetDiagnosticRecord::new(
                AssetDiagnosticCode::RatificationRejected,
                AssetDiagnosticSeverity::Warning,
                "base color input is disconnected",
            )
            .with_subject("material_graph.node:7"),
        );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_material_request(ToolSurfaceKind::MaterialGraphCanvas),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("material diagnostic [Warning] asset.ratification.rejected"));
        assert!(text.contains("subject=material_graph.node:7"));
        assert!(text.contains("base color input is disconnected"));
    }

    #[test]
    fn material_preview_view_model_reports_preview_status() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_material_request(ToolSurfaceKind::MaterialPreview),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("material preview status [NoSelection]"));
        assert!(text.contains("No material asset selected"));
        assert!(text.contains("last good material preview available: false"));
    }

    #[test]
    fn material_preview_provider_renders_preview_product_status() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        app.material_lab_runtime_mut()
            .set_active_preview(test_material_preview_product(asset_id(202)));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_material_request(ToolSurfaceKind::MaterialPreview),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(
            text.contains("material preview product status: active material preview product ready")
        );
        assert!(text.contains("active material product label: material product 30"));
        assert!(text.contains("material preview artifact label: material artifact 32"));
        assert!(text.contains("material preview shader artifact label: shader artifact 33"));
        assert!(
            text.contains("material preview scene shader artifact label: scene shader artifact 34")
        );
        assert!(text.contains("material preview viewport product label: viewport product 10030"));
    }

    #[test]
    fn material_inspector_renders_resource_binding_diagnostics() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        app.material_lab_runtime_mut()
            .set_active_source_document(asset_id(241), material_texture_binding_source_document());

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_material_request(ToolSurfaceKind::MaterialInspector),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("Texture / Resource Bindings"));
        assert!(text.contains("material.resource.unresolved_binding"));
        assert!(text.contains("status=Unresolved"));
    }

    #[test]
    fn material_preview_renders_resource_binding_diagnostics() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        app.material_lab_runtime_mut()
            .set_active_source_document(asset_id(242), material_texture_binding_source_document());

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_material_request(ToolSurfaceKind::MaterialPreview),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("Texture / Resource Bindings"));
        assert!(text.contains("material.resource.unresolved_binding"));
        assert!(text.contains("status=Unresolved"));
    }

    #[test]
    fn provider_string_lines_remain_compatible_during_ml_a() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_material_request(ToolSurfaceKind::MaterialPreview),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("material diagnostics: none (structured)"));
        assert!(text.contains("No active material preview product"));
        assert!(text.contains("No material diagnostics"));
    }

    fn test_material_preview_product(
        asset_id: asset::AssetId,
    ) -> crate::material_lab::EditorMaterialPreviewProduct {
        let product = material_graph::FormedMaterialProduct::new(
            material_graph::MaterialProductId::new(30),
            material_graph::MaterialGraphDocumentId::new(31),
            material_graph::MaterialOutputTarget::RenderMaterial,
            material_graph::MaterialCacheKey::new("material-preview-cache"),
        );
        crate::material_lab::EditorMaterialPreviewProduct::new(
            asset_id,
            asset_source_id(22),
            asset_artifact_id(32),
            ArtifactCacheKey::new("artifact-cache"),
            product,
            crate::material_lab::MaterialRendererParameterProfile::RenderMaterial,
            asset_artifact_id(33),
            ArtifactCacheKey::new("shader-cache"),
            ".runenwerk/artifacts/material.wgsl",
            "material-shader",
            asset_artifact_id(34),
            ArtifactCacheKey::new("scene-shader-cache"),
            ".runenwerk/artifacts/scene-material.wgsl",
            "scene-material-shader",
            [],
        )
    }

    fn material_texture_binding_source_document() -> material_graph::MaterialGraphDocument {
        material_graph::MaterialGraphDocument::new(
            material_graph::MaterialGraphDocumentId::new(2401),
            "material-texture-binding",
            GraphDefinition::new(
                GraphId::new(2401),
                "material-texture-binding",
                CyclePolicy::RejectDirectedCycles,
                [NodeDefinition::new(
                    NodeId::new(24),
                    "texture.sample_2d",
                    [],
                )],
                [],
            ),
            material_graph::MaterialOutputTarget::RenderMaterial,
        )
    }

    #[test]
    fn material_provider_actions_map_to_epoch_carrying_shell_commands() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let asset_id = asset_id(101);
        let request = m6_material_request(ToolSurfaceKind::MaterialGraphCanvas);
        let dispatch_context = SurfaceProviderDispatchContext {
            projection_epoch: 77,
            _marker: std::marker::PhantomData,
        };

        let proposal = registry
            .map_action(
                &dispatch_context,
                &request,
                MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
                SurfaceLocalAction::Material(MaterialSurfaceAction::BuildMaterialPreview {
                    asset_id,
                }),
            )
            .expect("material action should map")
            .expect("material action should produce shell command");

        assert!(matches!(
            proposal,
            SurfaceCommandProposal::Shell(ShellCommand::BuildMaterialPreview {
                asset_id: mapped,
                projection_epoch: 77,
            }) if mapped == asset_id
        ));
    }

    #[test]
    fn procgen_providers_resolve_directly_with_visible_preview_lines() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        assert!(!m6_workspace::is_m6_workspace_surface(
            ToolSurfaceKind::ProcgenGraphCanvas
        ));
        assert!(!m6_workspace::is_m6_workspace_surface(
            ToolSurfaceKind::ProcgenPreview
        ));

        let graph_frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_procgen_request(ToolSurfaceKind::ProcgenGraphCanvas),
            &Default::default(),
        );
        assert_eq!(
            graph_frame.availability,
            SurfaceProviderAvailability::Available
        );
        assert_eq!(
            graph_frame.provider_id,
            Some(PROCGEN_GRAPH_CANVAS_PROVIDER_ID)
        );
        assert!(
            provider_frame_text(&graph_frame)
                .contains("domain-backed Phase 6D bake-capable CPU preview")
        );

        let preview_frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_procgen_request(ToolSurfaceKind::ProcgenPreview),
            &Default::default(),
        );
        assert_eq!(
            preview_frame.availability,
            SurfaceProviderAvailability::Available
        );
        assert_eq!(preview_frame.provider_id, Some(PROCGEN_PREVIEW_PROVIDER_ID));
        let text = provider_frame_text(&preview_frame);
        assert!(text.contains("concrete terrain/material CPU preview"));
        assert!(text.contains("changed_regions=2"));
        assert!(text.contains("procgen field preview products: 0"));
    }

    #[test]
    fn texture_viewer_rejects_descriptor_only_completion() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(60);
        let artifact_id = asset_artifact_id(61);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "albedo",
                "Albedo",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload(texture_descriptor(
                    42,
                    TextureDimension::Texture2D,
                    TextureExtent::new(512, 512, 1),
                )),
                ArtifactCacheKey::new("texture-42"),
            ));
        let request = m6_texture_request(ToolSurfaceKind::TextureViewer);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(TEXTURE_VIEWER_PROVIDER_ID));
        let text = provider_frame_text(&frame);
        assert!(text.contains("preview descriptor: product=42"));
        assert!(text.contains("MissingArtifactUri"));
        assert!(
            !frame_has_product_surface(&frame),
            "descriptor-only texture data must not emit product-surface proof"
        );
        assert!(!frame.routes.is_empty());
    }

    #[test]
    fn texture_viewer_gpu_preview_uses_catalog_residency() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(62);
        let artifact_id = asset_artifact_id(63);
        let bytes = build_rgba8_ktx2(2, 2, 1, [12, 34, 56, 255], [12, 34, 56, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-texture-viewer-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        let descriptor = texture_descriptor_with_byte_length(
            44,
            TextureDimension::Texture2D,
            TextureExtent::new(2, 2, 1),
            bytes.len() as u64,
        );
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "albedo_gpu",
                "Albedo GPU",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(descriptor, path_string.clone()),
                    ArtifactCacheKey::new("texture-44"),
                )
                .with_artifact_path(path_string.clone()),
            );
        let request = m6_texture_request(ToolSurfaceKind::TextureViewer);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(TEXTURE_VIEWER_PROVIDER_ID));
        let text = provider_frame_text(&frame);
        assert!(frame_has_product_surface(&frame));
        assert!(text.contains("texture viewer: rendered GPU product-surface preview"));
        assert!(text.contains("residency class: engine.material_ktx2_upload"));
        assert!(text.contains("bind group identity: engine_ui_product_surface_bind_group"));
        assert!(text.contains("artifact URI:"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn texture_viewer_gpu_proof_uses_provider_product_surface_path() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(162);
        let artifact_id = asset_artifact_id(163);
        let bytes = build_rgba8_ktx2(2, 2, 1, [108, 210, 162, 255], [108, 210, 162, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-texture-viewer-provider-proof-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "wr028_texture_viewer_provider_proof",
                "WR-028 Texture Viewer Provider Proof",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            9028,
                            TextureDimension::Texture2D,
                            TextureExtent::new(2, 2, 1),
                            bytes.len() as u64,
                        ),
                        "docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-texture-viewer-2d.ktx2",
                    ),
                    ArtifactCacheKey::new("wr028-texture-viewer-provider-proof"),
                )
                .with_artifact_path(path_string.clone()),
            );
        app.asset_catalog_runtime_mut().select_asset(Some(asset_id));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::TextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(TEXTURE_VIEWER_PROVIDER_ID));
        assert!(frame_has_product_surface(&frame));
        assert!(text.contains("preview descriptor: product=9028"));
        assert!(text.contains(
            "preview target: runenwerk.editor.texture_preview:texture2d.product9028.mip0.slice0.all"
        ));
        assert!(text.contains("residency class: engine.material_ktx2_upload"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn volume_texture_viewer_gpu_proof_uses_provider_product_surface_path() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(164);
        let artifact_id = asset_artifact_id(165);
        let bytes = build_rgba8_ktx2(2, 2, 2, [153, 103, 173, 255], [153, 103, 173, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-volume-viewer-provider-proof-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test volume texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "wr028_volume_viewer_provider_proof",
                "WR-028 Volume Viewer Provider Proof",
                AssetKind::Texture3DVolume,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture3DVolume,
                    generated_texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            9029,
                            TextureDimension::Texture3DVolume,
                            TextureExtent::new(2, 2, 2),
                            bytes.len() as u64,
                        ),
                        "docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-volume-texture-viewer-3d.ktx2",
                    ),
                    ArtifactCacheKey::new("wr028-volume-viewer-provider-proof"),
                )
                .with_artifact_path(path_string.clone()),
            );
        app.asset_catalog_runtime_mut().select_asset(Some(asset_id));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(VOLUME_TEXTURE_VIEWER_PROVIDER_ID));
        assert!(frame_has_product_surface(&frame));
        assert!(text.contains("preview descriptor: product=9029"));
        assert!(text.contains(
            "preview target: runenwerk.editor.texture_preview:texture3d.product9029.mip0.slice0.all"
        ));
        assert!(text.contains("residency class: engine.material_ktx2_upload"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn texture_preview_records_bind_group_identity() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(72);
        let artifact_id = asset_artifact_id(73);
        let bytes = build_rgba8_ktx2(2, 2, 1, [21, 43, 65, 255], [21, 43, 65, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-texture-bind-group-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        let descriptor = texture_descriptor_with_byte_length(
            78,
            TextureDimension::Texture2D,
            TextureExtent::new(2, 2, 1),
            bytes.len() as u64,
        );
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "bind_group_texture",
                "Bind Group Texture",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(descriptor, path_string.clone()),
                    ArtifactCacheKey::new("texture-78"),
                )
                .with_artifact_path(path_string.clone()),
            );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::TextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(frame_has_product_surface(&frame));
        assert!(text.contains("sampler identity: min="));
        assert!(text.contains("bind group identity: engine_ui_product_surface_bind_group"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn texture_preview_proof_metadata_has_concrete_descriptor_hash() {
        let mut catalog = asset::AssetCatalog::new();
        let asset_id = asset_id(74);
        let artifact_id = asset_artifact_id(75);
        let bytes = build_rgba8_ktx2(2, 2, 1, [31, 47, 59, 255], [31, 47, 59, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-texture-proof-hash-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        let descriptor = texture_descriptor_with_byte_length(
            86,
            TextureDimension::Texture2D,
            TextureExtent::new(2, 2, 1),
            bytes.len() as u64,
        );
        let descriptor_hash = descriptor.descriptor_hash().to_string();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            "proof_hash_texture",
            "Proof Hash Texture",
            AssetKind::Texture2D,
        ));
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(descriptor, path_string.clone()),
                ArtifactCacheKey::new("texture-86"),
            )
            .with_artifact_path(path_string.clone()),
        );

        let prepared = crate::texture_preview::prepare_texture_preview(
            &catalog,
            Some(asset_id),
            &crate::texture_preview::TexturePreviewRuntime::default(),
            TextureViewerSurfaceKind::Texture2D,
        )
        .expect("texture preview proof should prepare");

        assert_eq!(prepared.proof.texture_product_id, 86);
        assert_eq!(prepared.proof.descriptor_hash, descriptor_hash);
        assert_eq!(prepared.proof.artifact_uri, path_string);
        assert!(!prepared.proof.descriptor_hash.contains("TextureDescriptor"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn texture_preview_proof_metadata_has_concrete_bind_group_identity() {
        let mut catalog = asset::AssetCatalog::new();
        let asset_id = asset_id(76);
        let artifact_id = asset_artifact_id(77);
        let bytes = build_rgba8_ktx2(2, 2, 1, [41, 67, 83, 255], [41, 67, 83, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-texture-proof-bind-group-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            "proof_bind_group_texture",
            "Proof Bind Group Texture",
            AssetKind::Texture2D,
        ));
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload_with_uri(
                    texture_descriptor_with_byte_length(
                        87,
                        TextureDimension::Texture2D,
                        TextureExtent::new(2, 2, 1),
                        bytes.len() as u64,
                    ),
                    path_string.clone(),
                ),
                ArtifactCacheKey::new("texture-87"),
            )
            .with_artifact_path(path_string.clone()),
        );

        let prepared = crate::texture_preview::prepare_texture_preview(
            &catalog,
            Some(asset_id),
            &crate::texture_preview::TexturePreviewRuntime::default(),
            TextureViewerSurfaceKind::Texture2D,
        )
        .expect("texture preview proof should prepare");

        assert_eq!(
            prepared.proof.bind_group_identity,
            "engine_ui_product_surface_bind_group:runenwerk.editor.texture_preview:texture2d.product87.mip0.slice0.all"
        );
        assert_eq!(
            prepared.proof.target_key.label(),
            "runenwerk.editor.texture_preview:texture2d.product87.mip0.slice0.all"
        );
        assert_eq!(
            prepared.proof.residency_class,
            "engine.material_ktx2_upload"
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn wr028_proof_manifest_rejects_texture_metadata_placeholders() {
        let manifest = include_str!(
            "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
        );

        for forbidden in [
            "descriptor_hash_source",
            "<dynamic texture preview target>",
            "manual GPU smoke temp KTX2 artifact",
            "verified inside viewport_gpu_truth_smoke readback assertions",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "WR-028 proof manifest still contains placeholder texture proof metadata: {forbidden}"
            );
        }
        assert!(manifest.contains("texture_product_id: 9028"));
        assert!(manifest.contains("descriptor_hash: \""));
        assert!(manifest.contains("bind_group_identity: \"engine_ui_product_surface_bind_group:"));
        assert!(manifest.contains("residency_class: \"engine.material_ktx2_upload\""));
    }

    #[test]
    fn wr028_proof_manifest_rejects_temp_artifact_paths() {
        let manifest = include_str!(
            "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
        );

        for forbidden in [
            "AppData/Local/Temp",
            "std::env::temp_dir",
            "temp://runenwerk",
            "runenwerk-wr021-gpu-proof",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "WR-028 proof manifest must not depend on temp-only proof paths: {forbidden}"
            );
        }
    }

    #[test]
    fn wr028_proof_manifest_links_durable_texture_preview_artifacts() {
        let manifest = include_str!(
            "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
        );

        for required in [
            "artifacts/fixtures/wr028-texture-viewer-2d.ktx2",
            "artifacts/fixtures/wr028-volume-texture-viewer-3d.ktx2",
            "artifacts/metadata/wr028-texture2d-proof.ron",
            "artifacts/metadata/wr028-texture3d-proof.ron",
            "texture_viewer_provider_product_surface_path: true",
            "volume_texture_viewer_provider_product_surface_path: true",
        ] {
            assert!(
                manifest.contains(required),
                "WR-028 proof manifest must link durable texture viewer proof artifact: {required}"
            );
        }
    }

    #[test]
    fn texture_preview_records_concrete_catalog_metadata() {
        let manifest = include_str!(
            "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
        );

        for required in [
            "texture_product_id: 9028",
            "texture_product_id: 9029",
            "artifact_id: 29028",
            "artifact_id: 29029",
            "descriptor_hash: \"70726f647563745f69643d343a39303238",
            "descriptor_hash: \"70726f647563745f69643d343a39303239",
            "artifact_uri: \"docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-texture-viewer-2d.ktx2\"",
            "artifact_uri: \"docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/artifacts/fixtures/wr028-volume-texture-viewer-3d.ktx2\"",
            "preview_target_key: \"runenwerk.editor.texture_preview:texture2d.product9028.mip0.slice0.all\"",
            "preview_target_key: \"runenwerk.editor.texture_preview:texture3d.product9029.mip0.slice0.all\"",
            "capture_hash: \"blake3:483a40eff929a29193ca839ec96a069cc666764b2065c31a4986285bdec97eab\"",
            "capture_hash: \"blake3:917eb702a699db47d62821de87e240a75907e8470e8d5547966e983adcfe8dde\"",
        ] {
            assert!(
                manifest.contains(required),
                "WR-028 proof manifest must record concrete catalog texture metadata: {required}"
            );
        }
    }

    #[test]
    fn texture_viewer_gpu_proof_rejects_direct_temp_resource_bypass() {
        let manifest = include_str!(
            "../../../../../docs-site/src/content/docs/reports/closeouts/wr-028-perfectionist-material-lab-texture-views-and-scene-material-binding/proof-manifest.ron"
        );

        for forbidden in [
            "ResolvedMaterialResource",
            "gpu-truth-texture-70",
            "gpu-truth-texture-71",
            "wr021-material-texture-2d.ktx2",
            "wr021-material-texture-3d.ktx2",
        ] {
            assert!(
                !manifest.contains(forbidden),
                "WR-028 texture viewer proof must not cite the direct material resource bypass: {forbidden}"
            );
        }
    }

    #[test]
    fn texture_preview_uses_selected_catalog_texture_product() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let unselected_asset_id = asset_id(82);
        let selected_asset_id = asset_id(84);
        let bytes = build_rgba8_ktx2(2, 2, 1, [3, 5, 7, 255], [3, 5, 7, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-selected-texture-preview-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                unselected_asset_id,
                "unselected_texture",
                "Unselected Texture",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    asset_artifact_id(83),
                    unselected_asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            79,
                            TextureDimension::Texture2D,
                            TextureExtent::new(2, 2, 1),
                            bytes.len() as u64,
                        ),
                        path_string.clone(),
                    ),
                    ArtifactCacheKey::new("texture-79"),
                )
                .with_artifact_path(path_string.clone()),
            );
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                selected_asset_id,
                "selected_texture",
                "Selected Texture",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    asset_artifact_id(85),
                    selected_asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            80,
                            TextureDimension::Texture2D,
                            TextureExtent::new(2, 2, 1),
                            bytes.len() as u64,
                        ),
                        path_string.clone(),
                    ),
                    ArtifactCacheKey::new("texture-80"),
                )
                .with_artifact_path(path_string.clone()),
            );
        app.asset_catalog_runtime_mut()
            .select_asset(Some(selected_asset_id));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::TextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(frame_has_product_surface(&frame));
        assert!(text.contains("preview descriptor: product=80"));
        assert!(!text.contains("preview descriptor: product=79"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn texture_preview_invalid_selected_asset_does_not_fallback() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let valid_asset_id = asset_id(86);
        let selected_asset_id = asset_id(88);
        let bytes = build_rgba8_ktx2(2, 2, 1, [13, 17, 19, 255], [13, 17, 19, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-invalid-selected-texture-preview-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                valid_asset_id,
                "valid_fallback_texture",
                "Valid Fallback Texture",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    asset_artifact_id(87),
                    valid_asset_id,
                    AssetKind::Texture2D,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            81,
                            TextureDimension::Texture2D,
                            TextureExtent::new(2, 2, 1),
                            bytes.len() as u64,
                        ),
                        path_string.clone(),
                    ),
                    ArtifactCacheKey::new("texture-81"),
                )
                .with_artifact_path(path_string.clone()),
            );
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                selected_asset_id,
                "invalid_selected_texture",
                "Invalid Selected Texture",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(AssetArtifactDescriptor::new(
                asset_artifact_id(89),
                selected_asset_id,
                AssetKind::Texture2D,
                texture_payload(texture_descriptor(
                    82,
                    TextureDimension::Texture2D,
                    TextureExtent::new(2, 2, 1),
                )),
                ArtifactCacheKey::new("texture-82"),
            ));
        app.asset_catalog_runtime_mut()
            .select_asset(Some(selected_asset_id));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::TextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("preview descriptor: product=82"));
        assert!(text.contains("MissingArtifactUri"));
        assert!(!text.contains("preview descriptor: product=81"));
        assert!(!frame_has_product_surface(&frame));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn texture_preview_reports_missing_artifact_uri() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(64);
        let artifact_id = asset_artifact_id(65);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "missing_uri",
                "Missing URI",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload(texture_descriptor(
                    45,
                    TextureDimension::Texture2D,
                    TextureExtent::new(2, 2, 1),
                )),
                ArtifactCacheKey::new("texture-45"),
            ));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::TextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("MissingArtifactUri"));
        assert!(!frame_has_product_surface(&frame));
    }

    #[test]
    fn texture_preview_reports_invalid_descriptor_hash() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(66);
        let artifact_id = asset_artifact_id(67);
        let descriptor =
            texture_descriptor(46, TextureDimension::Texture2D, TextureExtent::new(2, 2, 1));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "bad_hash",
                "Bad Hash",
                AssetKind::Texture2D,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(AssetArtifactDescriptor::new(
                artifact_id,
                asset_id,
                AssetKind::Texture2D,
                texture_payload_with_hash(
                    descriptor,
                    "not-the-descriptor-hash",
                    Some("mem://bad.ktx2".to_string()),
                ),
                ArtifactCacheKey::new("texture-46"),
            ));

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::TextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("InvalidDescriptorHash"));
        assert!(!frame_has_product_surface(&frame));
    }

    #[test]
    fn asset_browser_projects_typed_rows_and_epoch_routes() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(80);
        let source_id = asset_source_id(81);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(
                AssetRecord::new(asset_id, "field", "Field", AssetKind::SdfGraph)
                    .with_primary_source(source_id),
            );
        app.asset_catalog_runtime_mut().catalog_mut().insert_source(
            AssetSourceDescriptor::new(
                source_id,
                asset_id,
                AssetKind::SdfGraph,
                "assets/field.ron",
            )
            .with_hash(SourceHash::new("sha256", "abc")),
        );
        let request = asset_request(ToolSurfaceKind::AssetBrowser);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(ASSET_BROWSER_PROVIDER_ID));
        let text = provider_frame_text(&frame);
        assert!(text.contains("asset 80 Field"));
        assert!(!frame.routes.is_empty());

        let proposal = registry
            .map_action(
                &SurfaceProviderDispatchContext {
                    projection_epoch: 55,
                    _marker: std::marker::PhantomData,
                },
                &request,
                ASSET_BROWSER_PROVIDER_ID,
                SurfaceLocalAction::Asset(AssetSurfaceAction::SelectAsset { asset_id }),
            )
            .expect("asset action should map")
            .expect("asset action should produce shell command");
        assert!(matches!(
            proposal,
            SurfaceCommandProposal::Shell(ShellCommand::SelectAsset {
                asset_id: mapped,
                projection_epoch: 55,
            }) if mapped == asset_id
        ));
    }

    #[test]
    fn import_inspector_surfaces_prior_valid_and_routes_reimport() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(90);
        let artifact_id = asset_artifact_id(91);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "field",
                "Field",
                AssetKind::FormedFieldProduct,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::FormedFieldProduct,
                    ArtifactPayloadKind::FormedFieldProduct {
                        product_id: "field".to_string(),
                    },
                    ArtifactCacheKey::new("field"),
                )
                .with_artifact_path(".runenwerk/artifacts/field.ron")
                .with_validity(asset::ArtifactValidity::FailedPreserved)
                .with_diagnostic(asset::AssetDiagnosticRecord::error(
                    asset::AssetDiagnosticCode::SourceMissing,
                    "source missing",
                )),
            );
        app.asset_catalog_runtime_mut().select_asset(Some(asset_id));
        app.asset_catalog_runtime_mut().mark_asset_dirty(asset_id);
        let request = asset_request(ToolSurfaceKind::ImportInspector);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.provider_id, Some(IMPORT_INSPECTOR_PROVIDER_ID));
        let text = provider_frame_text(&frame);
        assert!(text.contains("preserved artifact 91"));
        assert!(!frame.routes.is_empty());
    }

    #[test]
    fn volume_texture_viewer_slice_mip_channel_controls_affect_preview_request() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(70);
        let artifact_id = asset_artifact_id(71);
        let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-volume-texture-viewer-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test volume texture should write");
        let path_string = path.to_string_lossy().to_string();
        let descriptor = texture_descriptor_with_byte_length(
            77,
            TextureDimension::Texture3DVolume,
            TextureExtent::new(2, 2, 2),
            bytes.len() as u64,
        );
        app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewSlice {
            surface: TextureViewerSurfaceKind::VolumeTexture3D,
            slice_index: 1,
        });
        app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewChannel {
            surface: TextureViewerSurfaceKind::VolumeTexture3D,
            channel: TexturePreviewChannelSelection::G,
        });
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "density_volume",
                "Density Volume",
                AssetKind::Texture3DVolume,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    artifact_id,
                    asset_id,
                    AssetKind::Texture3DVolume,
                    texture_payload_with_uri(descriptor, path_string.clone()),
                    ArtifactCacheKey::new("volume-77"),
                )
                .with_artifact_path(path_string.clone()),
            );
        let request = m6_texture_request(ToolSurfaceKind::VolumeTextureViewer);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(VOLUME_TEXTURE_VIEWER_PROVIDER_ID));
        let text = provider_frame_text(&frame);
        assert!(text.contains("preview descriptor: product=77"));
        assert!(text.contains("selected slice: 1"));
        assert!(text.contains("selected channel: g"));
        assert!(text.contains(
            "preview target: runenwerk.editor.texture_preview:texture3d.product77.mip0.slice1.g"
        ));
        assert!(frame_has_product_surface(&frame));
        assert!(!frame.routes.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn volume_texture_viewer_slice_changes_preview_request() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(92);
        let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-volume-slice-preview-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test volume texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewSlice {
            surface: TextureViewerSurfaceKind::VolumeTexture3D,
            slice_index: 1,
        });
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "volume_slice",
                "Volume Slice",
                AssetKind::Texture3DVolume,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    asset_artifact_id(93),
                    asset_id,
                    AssetKind::Texture3DVolume,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            83,
                            TextureDimension::Texture3DVolume,
                            TextureExtent::new(2, 2, 2),
                            bytes.len() as u64,
                        ),
                        path_string.clone(),
                    ),
                    ArtifactCacheKey::new("volume-83"),
                )
                .with_artifact_path(path_string.clone()),
            );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("selected slice: 1"));
        assert!(text.contains("texture3d.product83.mip0.slice1.all"));
        assert!(frame_has_product_surface(&frame));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn volume_texture_viewer_channel_changes_preview_request() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(94);
        let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-volume-channel-preview-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test volume texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewChannel {
            surface: TextureViewerSurfaceKind::VolumeTexture3D,
            channel: TexturePreviewChannelSelection::B,
        });
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "volume_channel",
                "Volume Channel",
                AssetKind::Texture3DVolume,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    asset_artifact_id(95),
                    asset_id,
                    AssetKind::Texture3DVolume,
                    texture_payload_with_uri(
                        texture_descriptor_with_byte_length(
                            84,
                            TextureDimension::Texture3DVolume,
                            TextureExtent::new(2, 2, 2),
                            bytes.len() as u64,
                        ),
                        path_string.clone(),
                    ),
                    ArtifactCacheKey::new("volume-84"),
                )
                .with_artifact_path(path_string.clone()),
            );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("selected channel: b"));
        assert!(text.contains("texture3d.product84.mip0.slice0.b"));
        assert!(frame_has_product_surface(&frame));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn volume_texture_viewer_mip_request_is_diagnosed_when_unresident() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let asset_id = asset_id(96);
        let bytes = build_rgba8_ktx2(2, 2, 2, [1, 2, 3, 255], [8, 9, 10, 255]);
        let path = std::env::temp_dir().join(format!(
            "runenwerk-volume-mip-preview-{}.ktx2",
            std::process::id()
        ));
        std::fs::write(&path, &bytes).expect("test volume texture should write");
        let path_string = path.to_string_lossy().to_string();
        app.apply_texture_surface_action(TextureSurfaceAction::SetPreviewMip {
            surface: TextureViewerSurfaceKind::VolumeTexture3D,
            mip_level: 1,
        });
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(AssetRecord::new(
                asset_id,
                "volume_mip",
                "Volume Mip",
                AssetKind::Texture3DVolume,
            ));
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_artifact(
                AssetArtifactDescriptor::new(
                    asset_artifact_id(97),
                    asset_id,
                    AssetKind::Texture3DVolume,
                    texture_payload_with_uri(
                        texture_descriptor_with_mip_count_and_byte_length(
                            85,
                            TextureDimension::Texture3DVolume,
                            TextureExtent::new(2, 2, 2),
                            2,
                            bytes.len() as u64,
                        ),
                        path_string.clone(),
                    ),
                    ArtifactCacheKey::new("volume-85"),
                )
                .with_artifact_path(path_string.clone()),
            );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &m6_texture_request(ToolSurfaceKind::VolumeTextureViewer),
            &Default::default(),
        );

        let text = provider_frame_text(&frame);
        assert!(text.contains("preview descriptor: product=85 mip=1"));
        assert!(text.contains("FailedUpload"));
        assert!(text.contains("selected mip 1 is unsupported"));
        assert!(!frame_has_product_surface(&frame));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sdf_field_layer_stack_provider_resolves_before_m6_fallback_with_routes() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let mut app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let layer_id = app.sdf_operation_workspace().document().layers()[0].id;
        app.sdf_operation_workspace_mut()
            .apply_command(
                editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Sphere Add".to_string(),
                    primitive: editor_scene::SdfPrimitiveSpec::new(
                        editor_scene::SdfPrimitiveKind::Sphere,
                        editor_scene::SdfBooleanIntent::Add,
                    ),
                    material_channel: 2,
                },
            )
            .expect("SDF command should apply");
        let request = m6_sdf_request(ToolSurfaceKind::FieldLayerStack, DocumentKind::SdfGraph);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(FIELD_LAYER_STACK_PROVIDER_ID));
        let text = provider_frame_text(&frame);
        assert!(text.contains("lowered world_ops records: 1"));
        assert!(text.contains("commit eligible: true"));
        assert!(!frame.routes.is_empty());
    }

    #[test]
    fn field_layer_stack_actions_map_to_sdf_domain_proposals() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let layer_id = app.sdf_operation_workspace().document().layers()[0].id;
        let request = m6_sdf_request(ToolSurfaceKind::FieldLayerStack, DocumentKind::SdfGraph);
        let dispatch_context = SurfaceProviderDispatchContext {
            projection_epoch: 44,
            _marker: std::marker::PhantomData,
        };

        let proposal = registry
            .map_action(
                &dispatch_context,
                &request,
                FIELD_LAYER_STACK_PROVIDER_ID,
                SurfaceLocalAction::SdfOperation(
                    editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                        intent: editor_scene::SdfOperationCommandIntent::SetLayerEnabled {
                            layer_id,
                            enabled: false,
                        },
                    },
                ),
            )
            .expect("provider should map action")
            .expect("action should produce proposal");

        match proposal {
            SurfaceCommandProposal::EditorDomain(proposal) => {
                assert_eq!(proposal.projection_epoch, 44);
                assert!(matches!(
                    proposal.mutation,
                    EditorDomainMutation::SdfOperation(
                        editor_shell::SdfOperationDomainMutation::ApplyCommand { .. }
                    )
                ));
            }
            _ => panic!("SDF field action should map to an editor domain proposal"),
        }
    }

    #[test]
    fn sdf_graph_canvas_provider_is_descriptor_first_and_command_backed() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = m6_sdf_request(ToolSurfaceKind::SdfGraphCanvas, DocumentKind::SdfGraph);

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Available);
        assert_eq!(frame.provider_id, Some(SDF_GRAPH_CANVAS_PROVIDER_ID));
        assert!(
            provider_frame_text(&frame).contains("canvas/session state is not SDF graph truth")
        );
        assert!(provider_frame_text(&frame).contains("graph can lower: false"));
        assert!(!frame.routes.is_empty());
    }

    #[test]
    fn sdf_surfaces_fail_closed_for_incompatible_document_kind() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let app = RunenwerkEditorApp::new();
        let shell_state = RunenwerkEditorShellState::new();
        let theme = ThemeTokens::default();
        let request = m6_sdf_request(
            ToolSurfaceKind::FieldLayerStack,
            DocumentKind::MaterialGraph,
        );

        let frame = registry.resolve_frame(
            &context(&app, &shell_state, &theme),
            &request,
            &Default::default(),
        );

        assert_eq!(frame.availability, SurfaceProviderAvailability::Unsupported);
        assert!(frame.routes.is_empty());
    }
}
