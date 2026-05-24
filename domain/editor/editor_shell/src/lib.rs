//! File: domain/editor/editor_shell/src/lib.rs
//! Crate: editor_shell

pub mod commands;
pub mod composition;
pub mod expression;
pub mod ids;
pub mod observation;
pub mod runtime;
pub mod surface_provider;
pub mod surfaces;
pub mod tool_suite;
pub mod view_models;
pub mod workspace;

#[cfg(test)]
mod tests;

pub use commands::*;
pub use composition::*;
pub use expression::*;
pub use ids::*;
pub use observation::*;
pub use runtime::*;
pub use surface_provider::*;
pub use surfaces::{
    AssetBrowserRowViewModel, AssetBrowserViewModel, AssetDetailViewModel, AssetSurfaceAction,
    EditorDefinitionSurfaceAction, EditorLabActionViewModel, EditorLabCanvasPreviewViewModel,
    EditorLabConsoleLineViewModel, EditorLabConsoleViewModel,
    EditorLabDefinitionHierarchyViewModel, EditorLabDefinitionRowViewModel,
    EditorLabDegradedViewModel, EditorLabDiagnosticViewModel, EditorLabDiagnosticsViewModel,
    EditorLabInspectorFieldViewModel, EditorLabInspectorViewModel,
    EditorLabPaletteCategoryViewModel, EditorLabPaletteViewModel, EditorLabReviewViewModel,
    EditorLabSurfaceViewModel, EditorLabTextFieldViewModel, EntityTableComponentFilter,
    EntityTableComponentFilterItem, EntityTableDomainMutation, EntityTableFilter,
    EntityTableHierarchyFilter, EntityTableQuery, EntityTableSessionMutation, EntityTableSort,
    EntityTableSurfaceAction, ImportInspectorViewModel, InspectorFieldControlKind,
    InspectorFieldEditIntent, InspectorSessionMutation, InspectorSurfaceAction,
    MaterialDiagnosticRowViewModel, MaterialDiagnosticSeverity, MaterialGraphCanvasViewModel,
    MaterialGraphEdgeViewModel, MaterialGraphEditorViewModel, MaterialGraphGroupViewModel,
    MaterialGraphNodeViewModel, MaterialGraphPortViewModel, MaterialGraphPropertyViewModel,
    MaterialGraphResourceBindingViewModel, MaterialGraphShortcutViewModel,
    MaterialGraphSourceDetailViewModel, MaterialGraphSourceRowViewModel,
    MaterialGraphToolbarViewModel, MaterialGraphValidationOverlayViewModel,
    MaterialGraphValidationSeverity, MaterialInspectorViewModel,
    MaterialModelMeshPreviewRegionViewModel, MaterialModelMeshPreviewStatusKind,
    MaterialModelMeshPreviewViewModel, MaterialModelMeshRegionBindingViewModel,
    MaterialNodePaletteCategoryViewModel, MaterialNodePaletteItemViewModel,
    MaterialNodePaletteViewModel, MaterialNodePickerViewModel,
    MaterialPreviewProductSurfaceViewModel, MaterialPreviewPublicationStatusKind,
    MaterialPreviewStatusKind, MaterialPreviewStatusViewModel, MaterialPreviewViewModel,
    MaterialResourceBindingDiagnosticViewModel, MaterialResourceBindingStatusKind,
    MaterialSceneMaterialSlotOptionViewModel, MaterialSdfPrimitiveBindingViewModel,
    MaterialShortcutAction, MaterialSurfaceAction, MaterialTextureResourceOptionViewModel,
    MaterialTextureResourcePickerViewModel, MaterialUndoRedoViewModel, OutlinerDomainMutation,
    OutlinerSurfaceAction, SdfOperationDomainMutation, SdfOperationSessionMutation,
    SdfOperationSurfaceAction, TexturePreviewChannelSelection, TextureSurfaceAction,
    TextureViewerSurfaceKind, ViewportDomainMutation, ViewportSessionMutation,
    ViewportSurfaceAction,
};
pub use tool_suite::{
    CommandCapabilityKey, EditorToolSuite, HostCapabilityPolicy, HostCapabilityRequirements,
    LegacyToolSurfaceResolution, LegacyToolSurfaceStableKeyCandidate, ProductCapabilityKey,
    ProductCapabilityNeed, ProfileRef, ProviderBundle, ProviderBundleError,
    ProviderFamilyDefinition, ProviderFamilyId, ProviderFamilyProviderAssignment,
    ProviderFamilyProviderMap, ProviderFamilyProviderMapError, ResourceCapabilityKey,
    SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES, SuiteRef, SurfaceRef, ToolServiceKey,
    ToolServiceNeed, ToolSuiteCapabilityDeclaration, ToolSuiteId, ToolSuiteIdentityError,
    ToolSuiteProfileDefinition, ToolSuiteRegistry, ToolSuiteRegistryError,
    ToolSurfaceCreationPolicy, ToolSurfaceDefinition, ToolSurfacePersistence, ToolSurfaceRegistry,
    ToolSurfaceResolution, ToolSurfaceRole, ToolSurfaceRoute, ToolSurfaceStableKey,
    ToolSurfaceTargetProfileCompatibility, WorkbenchComposition, WorkbenchCompositionBuildError,
    WorkbenchCompositionBuilder, resolve_legacy_tool_surface_kind,
    saveable_tool_surface_stable_key_candidates, stable_key_candidate_for_key,
    stable_key_candidate_for_kind, stable_key_candidate_for_persisted_kind,
    stable_key_for_persisted_tool_surface_kind_v2, stable_key_for_tool_surface_kind,
    tool_surface_kind_for_stable_key,
};
pub use view_models::*;
pub use workspace::{
    ANIMATION_GRAPH_CANVAS_SURFACE_DEFINITION_ID, ANIMATION_WORKSPACE_PROFILE_ID,
    ASSET_BROWSER_SURFACE_DEFINITION_ID, AuthoredToolSurfaceResolution,
    BINDINGS_SURFACE_DEFINITION_ID, COMMAND_DIFF_SURFACE_DEFINITION_ID,
    CONSOLE_SURFACE_DEFINITION_ID, CURVE_EDITOR_SURFACE_DEFINITION_ID,
    DEFINITION_VALIDATION_SURFACE_DEFINITION_ID, DIAGNOSTICS_SURFACE_DEFINITION_ID,
    DOCK_LAYOUT_PREVIEW_SURFACE_DEFINITION_ID, DockSplitSide,
    EDITOR_DESIGN_OUTLINER_SURFACE_DEFINITION_ID, EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
    ENTITY_TABLE_SURFACE_DEFINITION_ID, EditorWindowId, EditorWindowLifecycleState,
    EditorWindowPlacement, EditorWindowRecord, EditorWindowRegistry, EditorWindowTitlePolicy,
    FIELD_LAYER_STACK_SURFACE_DEFINITION_ID, FIELD_PRODUCT_VIEWER_SURFACE_DEFINITION_ID,
    FIELD_WORLD_WORKSPACE_PROFILE_ID, FloatingHostBounds, FloatingHostPlaceholderState,
    GAMEPLAY_COMPILER_DIAGNOSTICS_SURFACE_DEFINITION_ID,
    GAMEPLAY_GRAPH_CANVAS_SURFACE_DEFINITION_ID, GAMEPLAY_WORKSPACE_PROFILE_ID,
    GRAPH_CANVAS_SURFACE_DEFINITION_ID, GRAPH_WORKSPACE_PROFILE_ID,
    IMPORT_INSPECTOR_SURFACE_DEFINITION_ID, INSPECTOR_SURFACE_DEFINITION_ID,
    LAYOUT_WORKSPACE_PROFILE_ID, MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
    MATERIAL_INSPECTOR_SURFACE_DEFINITION_ID, MATERIAL_PREVIEW_SURFACE_DEFINITION_ID,
    MATERIAL_WORKSPACE_PROFILE_ID, MENU_EDITOR_SURFACE_DEFINITION_ID,
    MODELLING_WORKSPACE_PROFILE_ID, OUTLINER_SURFACE_DEFINITION_ID,
    PARTICLE_GRAPH_CANVAS_SURFACE_DEFINITION_ID, PARTICLE_PREVIEW_SURFACE_DEFINITION_ID,
    PARTICLE_WORKSPACE_PROFILE_ID, PERSISTED_WORKSPACE_STATE_VERSION_V1,
    PERSISTED_WORKSPACE_STATE_VERSION_V2, PERSISTED_WORKSPACE_STATE_VERSION_V3,
    PERSISTED_WORKSPACE_STATE_VERSION_V4, PERSISTED_WORKSPACE_STATE_VERSION_V5,
    PHYSICS_AUTHORING_SURFACE_DEFINITION_ID, PHYSICS_DEBUG_SURFACE_DEFINITION_ID,
    PHYSICS_WORKSPACE_PROFILE_ID, PLACEHOLDER_SURFACE_DEFINITION_ID,
    PROCGEN_GRAPH_CANVAS_SURFACE_DEFINITION_ID, PROCGEN_PREVIEW_SURFACE_DEFINITION_ID,
    PROCGEN_WORKSPACE_PROFILE_ID, PanelHostId, PanelHostKind, PanelHostNode, PanelInstanceId,
    PanelInstanceState, PanelKind, PersistedPanelHostKindV1, PersistedPanelHostNodeV1,
    PersistedPanelInstanceStateV1, PersistedPanelInstanceStateV2, PersistedPanelKindV1,
    PersistedPanelKindV2, PersistedTabStackStateV1, PersistedTabStackStateV5,
    PersistedToolSurfaceKindV1, PersistedToolSurfaceKindV2, PersistedToolSurfaceMountV1,
    PersistedToolSurfaceStateV1, PersistedToolSurfaceStateV2, PersistedToolSurfaceStateV3,
    PersistedToolSurfaceStateV5, PersistedWorkspaceSplitAxisV1, PersistedWorkspaceStateV1,
    PersistedWorkspaceStateV2, PersistedWorkspaceStateV3, PersistedWorkspaceStateV4,
    PersistedWorkspaceStateV5, ProjectedFloatingHostSlot, ProjectedPanelSlot, ProjectedTabButton,
    ProjectedTabButtonRoute, ProjectedTabDropRoute, ProjectedTabDropSlot, ProjectedTabDropTarget,
    ProjectedTabStackSlot, ProjectedWorkspaceHostSlot, RUNTIME_DEBUG_SURFACE_DEFINITION_ID,
    RUNTIME_DEBUG_WORKSPACE_PROFILE_ID, SCENE_WORKSPACE_PROFILE_ID,
    SDF_BRUSH_BROWSER_SURFACE_DEFINITION_ID, SDF_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
    SHORTCUT_EDITOR_SURFACE_DEFINITION_ID, SIMULATION_DIAGNOSTICS_SURFACE_DEFINITION_ID,
    SIMULATION_PREVIEW_SURFACE_DEFINITION_ID, SIMULATION_WORKSPACE_PROFILE_ID,
    STYLE_INSPECTOR_SURFACE_DEFINITION_ID, SplitHostState, StructuralWidgetRoutingContext,
    TEXTURE_VIEWER_SURFACE_DEFINITION_ID, TEXTURE_WORKSPACE_PROFILE_ID,
    THEME_EDITOR_SURFACE_DEFINITION_ID, TIMELINE_SURFACE_DEFINITION_ID, TabStackHostState,
    TabStackId, TabStackState, ToolSurfaceDisplayMetadata, ToolSurfaceDisplayMetadataSource,
    ToolSurfaceInstanceId, ToolSurfaceKind, ToolSurfaceMount, ToolSurfaceState,
    UI_CANVAS_SURFACE_DEFINITION_ID, UI_HIERARCHY_SURFACE_DEFINITION_ID,
    VIEWPORT_SURFACE_DEFINITION_ID, VOLUME_TEXTURE_VIEWER_SURFACE_DEFINITION_ID,
    WorkspaceDefaultToolSurface, WorkspaceDefinitionFormationError, WorkspaceId,
    WorkspaceIdentityAllocator, WorkspaceIdentitySeed, WorkspaceLayoutTemplate, WorkspaceMutation,
    WorkspaceProfile, WorkspaceProfileId, WorkspaceProfileRegistry,
    WorkspaceProfileRegistryBackedBuildError, WorkspaceProfileToolSurfaceCompatibilityReport,
    WorkspaceProfileToolSurfaceCompatibleSurface, WorkspaceProfileToolSurfaceLegacySurface,
    WorkspaceProfileToolSurfaceUnmappedLegacySurface, WorkspaceProjectionArtifact,
    WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError, WorkspaceSurfaceIdentityError,
    WorkspaceToolSurfaceRegistryCompatibilityReport, WorkspaceToolSurfaceRegistryCompatibleSurface,
    WorkspaceToolSurfaceRegistryIncompatibleSurface, WorkspaceToolSurfaceRegistryLegacySurface,
    WorkspaceToolSurfaceRegistryUnknownStableKey,
    WorkspaceToolSurfaceRegistryUnmappedLegacySurface, compact_empty_tab_stack_areas,
    default_workspace_profile_registry, editor_surface_definitions,
    form_workspace_state_from_definition, form_workspace_state_from_definition_with_registry,
    mounted_surface_instance, mounted_surface_instances, panel_kind_definition_key,
    panel_kind_for_tool_surface_kind, project_workspace_for_shell, projected_host_tab_stacks,
    reduce_workspace, resolve_authored_tool_surface_reference,
    tool_surface_capabilities_from_registry_or_legacy, tool_surface_capability_set,
    tool_surface_definition_id, tool_surface_display_metadata_from_registry_or_legacy,
    tool_surface_kind_definition_key, tool_surface_kind_from_definition_key,
    tool_surface_retention_class_from_registry_or_legacy, tool_surface_session_retention_class,
    viewport_embed_slot_for,
};
