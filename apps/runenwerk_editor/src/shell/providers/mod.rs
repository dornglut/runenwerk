use std::collections::{BTreeMap, BTreeSet};

use editor_core::{DocumentKind, EditorMutationError, EntityId, RealityVersion};
use editor_inspector::InspectorValue;
use editor_shell::{
    AssetSurfaceAction, ConsoleViewModel, ENTITY_TABLE_CLEAR_SEARCH_WIDGET_ID,
    ENTITY_TABLE_COMPONENT_FILTER_SELECT_WIDGET_ID, ENTITY_TABLE_LIST_WIDGET_ID,
    ENTITY_TABLE_ROOTS_ONLY_TOGGLE_WIDGET_ID, ENTITY_TABLE_SEARCH_WIDGET_ID,
    ENTITY_TABLE_SELECTED_ONLY_TOGGLE_WIDGET_ID, EditorDefinitionSurfaceAction,
    EditorDomainMutation, EditorLabActionViewModel, EditorLabCanvasPreviewViewModel,
    EditorLabCatalogItemDetails, EditorLabCatalogItemViewModel,
    EditorLabDefinitionHierarchyViewModel, EditorLabDefinitionRowViewModel,
    EditorLabDegradedViewModel, EditorLabDiagnosticViewModel, EditorLabDiagnosticsViewModel,
    EditorLabInspectorFieldViewModel, EditorLabInspectorViewModel,
    EditorLabPaletteCategoryViewModel, EditorLabPaletteViewModel, EditorLabReviewViewModel,
    EditorLabSurfaceViewModel, EditorLabTextFieldViewModel, EditorShellFrameModel,
    EntityTableDomainMutation, EntityTableRowViewModel, EntityTableSessionMutation,
    EntityTableSortKey, EntityTableSurfaceAction, EntityTableViewModel, InspectorFieldControlKind,
    InspectorFieldViewModel, InspectorObservationFrame, InspectorObservedField,
    InspectorObservedTarget, InspectorSessionMutation, InspectorSurfaceAction,
    InspectorTargetViewModel, InspectorViewModel, MaterialDiagnosticRowViewModel,
    MaterialPreviewStatusViewModel, MaterialPreviewViewModel,
    MaterialResourceBindingDiagnosticViewModel, MaterialSdfPrimitiveBindingViewModel,
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
    TextureViewerSurfaceKind, ToolSurfaceCreateCandidate, ToolSurfaceKind, ToolSurfaceReadiness,
    ToolSurfaceRegistry, ToolSurfaceStableKey, UI_DESIGNER_WORKBENCH_TARGET_PROFILE,
    UiDesignerWorkbenchPaneKind, UiDesignerWorkbenchPaneViewModel,
    UiDesignerWorkbenchReadinessStatus, UiDesignerWorkbenchReadinessViewModel,
    UiDesignerWorkbenchViewModel, UiNode, VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
    VIEWPORT_FIELD_SLICE_DECREMENT_WIDGET_ID, VIEWPORT_FIELD_SLICE_INCREMENT_WIDGET_ID,
    VIEWPORT_FIELD_SLICE_RESET_WIDGET_ID, VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
    VIEWPORT_RESET_CAMERA_WIDGET_ID, VIEWPORT_ROOT_OPAQUE_TOGGLE_WIDGET_ID,
    VIEWPORT_STATISTICS_TOGGLE_WIDGET_ID, VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID,
    ViewportDomainMutation, ViewportObservationFrame, ViewportProductChoiceViewModel,
    ViewportProductObservation, ViewportSessionMutation, ViewportSurfaceAction, ViewportViewModel,
    WidgetId, WorkspaceProfileRegistry, build_console_panel, build_editor_lab_surface,
    build_entity_table_panel, build_inspector_panel, build_material_graph_surface,
    build_outliner_panel, build_self_authoring_control_panel, build_viewport_panel,
    editor_domain_proposal, entity_table_sort_button_widget_id, inspector_field_focus_widget_id,
    inspector_field_widget_id, surface_session_proposal, surface_widget_id,
    tool_surface_capabilities_from_registry_or_legacy, tool_surface_definition_id,
    tool_surface_kind_for_stable_key, viewport_debug_stage_button_widget_id,
    viewport_field_color_ramp_button_widget_id, viewport_field_component_button_widget_id,
    viewport_field_debug_mode_button_widget_id, viewport_product_button_widget_id,
    viewport_tool_radial_item_widget_id,
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
            SurfaceProviderSupportMode::StableKey
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

fn stable_key_support(
    request: &SurfaceProviderRequest,
    stable_key: &str,
) -> SurfaceProviderSupportMode {
    if request.matches_stable_key(stable_key) {
        SurfaceProviderSupportMode::StableKey
    } else {
        SurfaceProviderSupportMode::Unsupported
    }
}

fn stable_keys_support(
    request: &SurfaceProviderRequest,
    stable_keys: &[&str],
) -> SurfaceProviderSupportMode {
    if request.matches_any_stable_key(stable_keys) {
        SurfaceProviderSupportMode::StableKey
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

    pub fn runenwerk_material_lab_workbench() -> Self {
        Self::new(vec![
            Box::new(ConsoleProvider),
            Box::new(AssetBrowserProvider),
            Box::new(ImportInspectorProvider),
            Box::new(M6WorkspaceProvider),
            Box::new(ToolSuiteRegistryInspectorProvider),
            Box::new(TextureViewerProvider),
            Box::new(VolumeTextureViewerProvider),
            Box::new(MaterialGraphCanvasProvider),
            Box::new(MaterialInspectorProvider),
            Box::new(MaterialPreviewProvider),
        ])
        .expect("Material Lab workbench surface providers must have unique ids")
    }

    pub fn runenwerk_ui_designer_workbench() -> Self {
        Self::new(vec![
            Box::new(ConsoleProvider),
            Box::new(SelfAuthoringProvider),
        ])
        .expect("UI Designer workbench surface providers must have unique ids")
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
        workspace_profile_registry: &WorkspaceProfileRegistry,
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

        if !workspace_allows_document(request, workspace_profile_registry) {
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

    pub(crate) fn assigned_provider_supports_request(
        &self,
        request: &SurfaceProviderRequest,
        provider_family_map: &ProviderFamilyProviderMap,
    ) -> Result<bool, SurfaceProviderDiagnostic> {
        Ok(self
            .candidate_providers_for_request(request, Some(provider_family_map))?
            .into_iter()
            .any(|provider| provider.support_mode(request).is_supported()))
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
        if !workspace_allows_document(
            request,
            context.app.workbench_host().workspace_profile_registry(),
        ) {
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
    let available_tool_surface_keys = active_definitions.available_tool_surface_keys();
    let available_tool_surface_create_candidates = build_tool_surface_create_candidates(
        &available_tool_surface_keys,
        app.workbench_host().tool_surface_registry(),
    );

    EditorShellFrameModel::new(build_toolbar_view_model(&toolbar_frame), surfaces)
        .with_route_actions(route_actions)
        .with_available_panel_kinds(available_panel_kinds)
        .with_available_tool_surface_create_candidates(available_tool_surface_create_candidates)
        .with_active_ui_definitions(toolbar_template, toolbar_binding, shell_chrome_template)
        .with_active_tab_stack_popup_menu(shell_state.active_tab_stack_popup_menu())
}

fn build_tool_surface_create_candidates(
    available_tool_surface_keys: &[ToolSurfaceStableKey],
    tool_surface_registry: &ToolSurfaceRegistry,
) -> Vec<ToolSurfaceCreateCandidate> {
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();

    if available_tool_surface_keys.is_empty() {
        for surface in tool_surface_registry.iter() {
            if seen.insert(surface.key.clone()) {
                candidates.push(ToolSurfaceCreateCandidate::new(
                    surface.key.clone(),
                    surface.label.clone(),
                    surface.panel_kind,
                ));
            }
        }

        return candidates;
    }

    for key in available_tool_surface_keys {
        let Some(surface) = tool_surface_registry.get(key) else {
            continue;
        };
        if seen.insert(surface.key.clone()) {
            candidates.push(ToolSurfaceCreateCandidate::new(
                surface.key.clone(),
                surface.label.clone(),
                surface.panel_kind,
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
            let registered_surface =
                tool_surface_registry.and_then(|registry| registry.get(&stable_surface_key));
            let stable_key_kind = tool_surface_kind_for_stable_key(&stable_surface_key);
            let surface_definition_id = stable_key_kind
                .map(tool_surface_definition_id)
                .unwrap_or(editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID);
            let capabilities = stable_key_kind
                .map(|kind| {
                    tool_surface_capabilities_from_registry_or_legacy(
                        kind,
                        Some(&stable_surface_key),
                        tool_surface_registry,
                    )
                })
                .or_else(|| registered_surface.map(|definition| definition.capabilities))
                .unwrap_or_default();
            Some(SurfaceProviderRequest {
                workspace_profile_id: shell_state.active_workspace_profile_id(),
                document_context: document_context.clone(),
                panel_instance_id: panel.id,
                tab_stack_id,
                tool_surface_instance_id: surface.id,
                stable_surface_key,
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

fn workspace_allows_document(
    request: &SurfaceProviderRequest,
    registry: &WorkspaceProfileRegistry,
) -> bool {
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
use m6_workspace::M6WorkspaceProvider;
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

mod common;
mod self_authoring;

#[cfg(test)]
mod tests;

use common::{deterministic_provider, diagnostic_frame, unsupported_frame};
use self_authoring::SelfAuthoringProvider;
