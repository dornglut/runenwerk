use std::collections::BTreeMap;

use editor_core::{ComponentTypeId, EntityId};
use ui_input::{
    Key, KeyState, KeyboardEvent, Modifiers, PointerEvent, PointerEventKind, TextInputEvent,
    UiInputEvent,
};
use ui_math::{Axis, UiRect};
use ui_theme::ThemeTokens;

use crate::{
    ActiveTabDragVisualState, ActiveTabStackPopupMenu, AssetBrowserRowViewModel,
    AssetBrowserViewModel, AssetSurfaceAction, DockDropCandidate, DockDropCandidateState,
    DockDropInvalidTargetReason, DockDropScope, DockingInteractionVisualState,
    DockingPreviewDropTarget, ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID,
    ENTITY_TABLE_SEARCH_WIDGET_ID, EditorShellFrameModel, EntityTableComponentFilter,
    EntityTableHierarchyFilter, EntityTableSurfaceAction, ImportInspectorViewModel,
    InspectorSurfaceAction, MaterialGraphCanvasViewModel, MaterialGraphEditorViewModel,
    MaterialGraphSourceRowViewModel, MaterialGraphToolbarViewModel, MaterialNodePaletteViewModel,
    MaterialNodePickerViewModel, MaterialPreviewStatusViewModel, MaterialPreviewViewModel,
    MaterialSurfaceAction, MaterialTextureResourcePickerViewModel, MaterialUndoRedoViewModel,
    OutlinerSurfaceAction, PanelInstanceId, PanelKind, ResolvedSurfaceFrame, RoutedShellAction,
    ShellCommand,
    SurfaceInteraction, SurfaceLocalAction, SurfaceLocalRoute, SurfacePresentationArtifact,
    SurfaceProviderAvailability, SurfaceProviderId, SurfaceRouteTable, TabStackPopupMenuKind,
    ToolSurfaceKind, ToolbarButtonViewModel, ToolbarViewModel, UiInteraction, UiInteractionResults,
    ViewportSurfaceAction, ViewportViewModel, WidgetId, WorkspaceIdentityAllocator,
    WorkspaceMutation, WorkspaceSplitAxis, WorkspaceState, build_editor_shell_frame,
    build_editor_shell_frame_with_docking_visual_state, build_entity_table_panel,
    build_viewport_panel, dock_split_preview_label_widget_id, dock_split_preview_overlay_widget_id,
    dock_split_preview_panel_widget_id, label, map_interactions_to_shell_commands,
    panel_kind_definition_key, reduce_workspace, surface_widget_id, tab_active_indicator_widget_id,
    tab_chrome_widget_id, tab_close_button_widget_id, tab_drop_zone_widget_id,
    tab_stack_action_menu_popup_widget_id, tab_stack_container_widget_id,
    tab_stack_new_surface_menu_item_widget_id, tab_stack_new_surface_menu_popup_widget_id,
    tab_stack_new_tab_button_widget_id, tab_stack_split_horizontal_button_widget_id,
    tab_stack_surface_menu_list_widget_id, tab_stack_surface_menu_popup_widget_id,
    tab_stack_surface_menu_scroll_widget_id, tab_stack_surface_submenu_anchor_widget_id,
    tool_surface_definition_id, tool_surface_kind_definition_key,
    toolbar_workspace_active_indicator_widget_id, toolbar_workspace_chrome_widget_id,
    toolbar_workspace_close_widget_id, workspace_split_host_widget_id,
};

#[test]
fn shell_graph_routing_has_no_new_domain_specific_graph_dispatch_actions() {
    let sources = [
        ("surface_provider.rs", include_str!("surface_provider.rs")),
        (
            "composition/build_editor_shell.rs",
            include_str!("composition/build_editor_shell.rs"),
        ),
        (
            "commands/map_interactions.rs",
            include_str!("commands/map_interactions.rs"),
        ),
    ];
    let forbidden_dispatch_actions = [
        "SurfaceLocalRoute::MaterialGraphCanvas",
        "DispatchMaterialGraphCanvasAction",
        "DispatchProcgenGraphCanvasAction",
        "DispatchGameplayGraphCanvasAction",
        "DispatchAnimationGraphCanvasAction",
        "DispatchParticleGraphCanvasAction",
        "DispatchPhysicsGraphCanvasAction",
        "DispatchSdfGraphCanvasAction",
        "command_for_graph_canvas_action",
        "material_action_for_graph_canvas_action",
        "material_graph::",
    ];

    for (source_name, source) in sources {
        for forbidden in forbidden_dispatch_actions {
            assert!(
                !source.contains(forbidden),
                "{source_name} must not add domain-specific graph dispatch action `{forbidden}`; use provider-owned graph routing instead"
            );
        }
    }
}

#[test]
fn editor_shell_no_longer_exposes_material_graph_canvas_route() {
    let source = include_str!("surface_provider.rs");

    assert!(!source.contains("SurfaceLocalRoute::MaterialGraphCanvas"));
    assert!(!source.contains("pub const fn material_graph_canvas"));
    assert!(source.contains("ProviderOwnedGraphCanvas"));
}

#[test]
fn editor_shell_no_longer_exposes_dispatch_material_graph_canvas_action() {
    let source = include_str!("composition/build_editor_shell.rs");

    assert!(!source.contains("DispatchMaterialGraphCanvasAction"));
    assert!(source.contains("DispatchSurfaceInteraction"));
}

#[test]
fn editor_shell_does_not_import_material_graph_for_graph_action_mapping() {
    let source = include_str!("commands/map_interactions.rs");

    assert!(!source.contains("material_graph::"));
    assert!(!source.contains("material_action_for_graph_canvas_action"));
    assert!(source.contains("SurfaceInteraction::GraphCanvasAction"));
}

#[test]
fn material_resource_binding_diagnostics_are_app_neutral_view_models() {
    let source = include_str!("surfaces/material.rs");
    let block = source_block_between(
        source,
        "pub struct MaterialResourceBindingDiagnosticViewModel {",
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum MaterialResourceBindingStatusKind",
        "MaterialResourceBindingDiagnosticViewModel",
    );

    assert!(block.contains("pub status: MaterialResourceBindingStatusKind"));
    for forbidden in [
        "AssetCatalog",
        "AssetArtifactDescriptor",
        "ResolvedMaterialResource",
        "TextureDescriptor",
        "ProductPublicationRuntimeResource",
        "Renderer",
        "resource_resolution",
    ] {
        assert!(
            !block.contains(forbidden),
            "Material resource binding diagnostics must remain app-neutral presentation DTOs; found forbidden dependency `{forbidden}`"
        );
    }
}

#[test]
fn reducer_normal_mutations_do_not_reintroduce_tool_surface_kind_authority_fields() {
    let source = include_str!("workspace/reducer.rs");
    let enum_block = source
        .split("pub enum WorkspaceMutation {")
        .nth(1)
        .and_then(|tail| tail.split("impl WorkspaceMutation").next())
        .expect("WorkspaceMutation enum should be followed by impl block");

    for forbidden in [
        "tool_surface_kind: ToolSurfaceKind",
        "new_tool_surface_kind: ToolSurfaceKind",
        "locked_tool_surface_kind: Option<ToolSurfaceKind>",
        "ReplacePanelToolSurfaceKind",
        "LockTabStackAreaType",
    ] {
        assert!(
            !enum_block
                .lines()
                .map(str::trim)
                .any(|line| line == forbidden),
            "normal reducer mutations must not carry ToolSurfaceKind authority field `{forbidden}`; use stable keys plus named legacy wrappers"
        );
    }

    assert!(enum_block.contains("stable_surface_key: ToolSurfaceStableKey"));
    assert!(enum_block.contains("locked_stable_surface_key: Option<ToolSurfaceStableKey>"));
    assert!(source.contains("add_panel_tab_legacy"));
    assert!(source.contains("replace_panel_tool_surface_kind_legacy"));
}

#[test]
fn tool_surface_kind_usage_is_boundary_only_guard() {
    let legacy_source = include_str!("tool_suite/legacy.rs");
    assert!(legacy_source.contains("explicit compatibility boundary"));
    assert!(legacy_source.contains("Do not use it for new stable-key-first"));
    assert!(legacy_source.contains("code paths."));

    stable_key_authority_is_end_to_end_guard();
    tool_surface_kind_is_legacy_boundary_only_guard();
    panel_kind_is_structural_not_surface_identity_guard();
    surface_provider_request_requires_stable_key_guard();
    v5_persistence_uses_stable_key_primary_identity_guard();
    tool_surface_kind_declaration_is_legacy_boundary_guard();
    public_tool_surface_kind_apis_are_legacy_labeled_guard();
    normal_tool_surface_state_does_not_use_tool_surface_kind_authority_guard();
    normal_workspace_mutations_do_not_use_tool_surface_kind_authority_guard();
    profile_default_surfaces_do_not_use_tool_surface_kind_authority_guard();
    provider_request_does_not_require_tool_surface_kind_guard();
    shell_menu_enum_compatibility_is_marked_pending_final_cleanup();
    normal_surface_classifiers_use_stable_keys_when_available();
    app_command_surface_contract_lookup_is_legacy_named();
    no_unmarked_tool_surface_kind_usage_in_normal_path_guard();
}

#[test]
fn stable_key_authority_is_end_to_end_guard() {
    let state_source = include_str!("workspace/state.rs");
    let reducer_source = include_str!("workspace/reducer.rs");
    let profile_source = include_str!("workspace/profile.rs");
    let projection_source = include_str!("workspace/projection.rs");
    let app_provider_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/providers/mod.rs");
    let workbench_host_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/workbench_host.rs");

    let tool_surface_state = source_block_between(
        state_source,
        "pub struct ToolSurfaceState {",
        "impl ToolSurfaceState",
        "ToolSurfaceState",
    );
    assert!(tool_surface_state.contains("pub stable_surface_key: ToolSurfaceStableKey"));
    assert!(!tool_surface_state.contains("pub tool_surface_kind: ToolSurfaceKind"));

    let mutation_enum = source_block_between(
        reducer_source,
        "pub enum WorkspaceMutation {",
        "impl WorkspaceMutation",
        "WorkspaceMutation",
    );
    assert!(mutation_enum.contains("stable_surface_key: ToolSurfaceStableKey"));
    assert!(mutation_enum.contains("locked_stable_surface_key: Option<ToolSurfaceStableKey>"));
    assert!(!mutation_enum.contains("tool_surface_kind: ToolSurfaceKind"));

    let profile_struct = source_block_between(
        profile_source,
        "pub struct WorkspaceProfile {",
        "#[derive(Debug, Clone, PartialEq, Eq, Default)]",
        "WorkspaceProfile",
    );
    assert!(profile_struct.contains("pub default_surfaces: Vec<WorkspaceDefaultToolSurface>"));
    assert!(!profile_struct.contains("default_tool_surfaces: Vec<ToolSurfaceKind>"));

    assert!(
        projection_source.contains("pub active_stable_surface_key: Option<ToolSurfaceStableKey>")
    );
    assert!(projection_source.contains(".map(|surface| surface.stable_surface_key().clone())"));
    surface_provider_request_requires_stable_key_guard();
    v5_persistence_uses_stable_key_primary_identity_guard();

    assert!(app_provider_source.contains("mounted_surface_requests_with_registry"));
    assert!(
        app_provider_source
            .contains("let stable_surface_key = surface.stable_surface_key().clone();")
    );
    assert!(app_provider_source.contains("provider_family_provider_map"));
    assert!(app_provider_source.contains("resolve_frame_with_provider_family_map"));
    assert!(app_provider_source.contains("SurfaceProviderSupportMode::StableKey"));
    assert!(app_provider_source.contains("preferred_support_mode"));
    assert!(app_provider_source.contains(".preferred(supported.support_mode)"));

    assert!(
        workbench_host_source.contains("provider_family_provider_map: ProviderFamilyProviderMap")
    );
    assert!(workbench_host_source.contains("RunenwerkWorkbenchHost"));
}

#[test]
fn tool_surface_kind_is_legacy_boundary_only_guard() {
    let legacy_source = include_str!("tool_suite/legacy.rs");
    let persisted_source = include_str!("workspace/persisted.rs");
    let definition_form_source = include_str!("workspace/definition_form.rs");
    let reducer_source = include_str!("workspace/reducer.rs");
    let profile_source = include_str!("workspace/profile.rs");
    let controller_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/controller.rs");
    let dispatch_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/dispatch/mod.rs");

    assert!(legacy_source.contains("explicit compatibility boundary"));
    assert!(
        legacy_source.contains("The reverse stable-key to `ToolSurfaceKind` bridge exists only")
    );

    assert!(persisted_source.contains("pub fn from_persisted_v1"));
    assert!(persisted_source.contains("pub fn from_persisted_v2"));
    assert!(persisted_source.contains("pub fn from_persisted_v3"));
    assert!(persisted_source.contains("pub fn from_persisted_v4"));
    assert!(persisted_source.contains("pub fn from_persisted_v5"));
    assert!(persisted_source.contains("legacy_tool_surface_kind_for_legacy_persistence"));
    assert!(persisted_source.contains("persisted_v5_stable_surface_key_for_surface"));

    assert!(definition_form_source.contains("authored_legacy_surface_key_still_resolves"));
    assert!(definition_form_source.contains("ToolSurfaceState::new_legacy"));
    assert!(reducer_source.contains("add_panel_tab_legacy"));
    assert!(reducer_source.contains("replace_panel_tool_surface_kind_legacy"));
    assert!(profile_source.contains("pub fn new_legacy"));

    assert!(controller_source.contains("tab_stack_chrome_surface_target_pending_c6"));
    assert!(controller_source.contains("SplitTabStackAreaStableKey"));
    assert!(dispatch_source.contains("LegacySurfaceCommandContract"));
    assert!(dispatch_source.contains("resolve_legacy_surface_command_contract"));
}

#[test]
fn panel_kind_is_structural_not_surface_identity_guard() {
    let state_source = include_str!("workspace/state.rs");
    let docs = preceding_lines_for(state_source, "pub enum PanelKind", 10);

    assert!(docs.contains("Structural shell/layout grouping"));
    assert!(docs.contains("not tool-surface identity"));
    assert!(docs.contains("ToolSurfaceStableKey"));

    let panel_struct = source_block_between(
        state_source,
        "pub struct PanelInstanceState {",
        "#[derive(Debug, Clone, PartialEq)]",
        "PanelInstanceState",
    );
    assert!(panel_struct.contains("pub panel_kind: PanelKind"));
    assert!(panel_struct.contains("pub active_tool_surface: Option<ToolSurfaceInstanceId>"));
    normal_tool_surface_state_does_not_use_tool_surface_kind_authority_guard();
}

#[test]
fn surface_provider_request_requires_stable_key_guard() {
    let source = include_str!("surface_provider.rs");
    let request_struct = source_block_between(
        source,
        "pub struct SurfaceProviderRequest {",
        "impl SurfaceProviderRequest",
        "SurfaceProviderRequest",
    );

    assert!(request_struct.contains("pub stable_surface_key: ToolSurfaceStableKey"));
    assert!(request_struct.contains("pub legacy_tool_surface_kind: Option<ToolSurfaceKind>"));
    assert!(request_struct.contains("pub provider_family_id: Option<ProviderFamilyId>"));
    assert!(request_struct.contains("pub surface_route: Option<ToolSurfaceRoute>"));
    assert!(!request_struct.contains("pub tool_surface_kind: ToolSurfaceKind"));
    assert!(source.contains("pub fn stable_key(&self) -> &ToolSurfaceStableKey"));
}

#[test]
fn v5_persistence_uses_stable_key_primary_identity_guard() {
    let source = include_str!("workspace/persisted.rs");
    let persisted_surface = source_block_between(
        source,
        "pub struct PersistedToolSurfaceStateV5 {",
        "pub struct PersistedViewportSettingsV1",
        "PersistedToolSurfaceStateV5",
    );
    let persisted_tab_stack = source_block_between(
        source,
        "pub struct PersistedTabStackStateV5 {",
        "pub struct PersistedPanelInstanceStateV1",
        "PersistedTabStackStateV5",
    );
    let to_persisted_v5 = source_block_between(
        source,
        "pub fn to_persisted_v5(&self) -> Result<PersistedWorkspaceStateV5, WorkspaceStateError> {",
        "pub fn to_persisted_v4",
        "to_persisted_v5",
    );
    let from_persisted_v5 = source_block_between(
        source,
        "pub fn from_persisted_v5(",
        "fn persisted_v5_stable_surface_key_for_surface",
        "from_persisted_v5",
    );

    assert!(persisted_surface.contains("pub stable_surface_key: String"));
    assert!(
        persisted_surface
            .contains("pub legacy_tool_surface_kind: Option<PersistedToolSurfaceKindV2>")
    );
    assert!(!persisted_surface.contains("pub tool_surface_kind: PersistedToolSurfaceKindV2"));
    assert!(persisted_tab_stack.contains("pub locked_stable_surface_key: Option<String>"));
    assert!(
        persisted_tab_stack
            .contains("pub legacy_locked_tool_surface_kind: Option<PersistedToolSurfaceKindV2>")
    );
    assert!(!persisted_tab_stack.contains("pub locked_tool_surface_kind: Option"));
    assert!(to_persisted_v5.contains("persisted_v5_stable_surface_key_for_surface(surface)?"));
    assert!(to_persisted_v5.contains("stable_surface_key: stable_surface_key.to_string()"));
    assert!(to_persisted_v5.contains("locked_stable_surface_key: stack"));
    assert!(from_persisted_v5.contains("persisted_v5_tool_surface_identity"));
    assert!(from_persisted_v5.contains("persisted_v5_tab_stack_lock_identity"));
    assert!(from_persisted_v5.contains("ToolSurfaceState::new_with_stable_key"));
}

#[test]
fn tool_surface_kind_declaration_is_legacy_boundary_guard() {
    let source = include_str!("workspace/state.rs");
    let declaration_docs = preceding_lines_for(source, "pub enum ToolSurfaceKind", 10);

    assert!(declaration_docs.contains("Legacy boundary enum"));
    assert!(declaration_docs.contains("ToolSurfaceStableKey"));
    assert!(declaration_docs.contains("not live tool-surface identity"));
    assert!(declaration_docs.contains("New"));
    assert!(declaration_docs.contains("normal APIs should carry `ToolSurfaceStableKey`"));
}

#[test]
fn public_tool_surface_kind_apis_are_legacy_labeled_guard() {
    let surface_contract_source = include_str!("workspace/surface_contract.rs");
    for helper in [
        "pub fn stable_key_for_tool_surface_kind",
        "pub fn tool_surface_definition_id",
        "pub fn tool_surface_kind_definition_key",
        "pub fn tool_surface_kind_from_definition_key",
        "pub fn panel_kind_for_tool_surface_kind",
        "pub fn tool_surface_capability_set",
        "pub fn tool_surface_session_retention_class",
    ] {
        assert_legacy_boundary_doc(surface_contract_source, helper);
    }

    let surface_provider_source = include_str!("surface_provider.rs");
    assert_legacy_boundary_doc(
        surface_provider_source,
        "pub fn with_available_tool_surface_kinds",
    );
}

#[test]
fn normal_tool_surface_state_does_not_use_tool_surface_kind_authority_guard() {
    let source = include_str!("workspace/state.rs");
    let struct_block = source_block_between(
        source,
        "pub struct ToolSurfaceState {",
        "impl ToolSurfaceState",
        "ToolSurfaceState",
    );

    assert!(struct_block.contains("pub stable_surface_key: ToolSurfaceStableKey"));
    assert!(struct_block.contains("pub legacy_tool_surface_kind: Option<ToolSurfaceKind>"));
    assert!(
        !struct_block.contains("pub tool_surface_kind: ToolSurfaceKind"),
        "ToolSurfaceState must not reintroduce ToolSurfaceKind authority"
    );
    assert!(source.contains("pub fn new_with_stable_key"));
    assert!(source.contains("pub fn new_legacy"));
    assert!(source.contains("pub panel_kind: PanelKind"));
}

#[test]
fn normal_workspace_mutations_do_not_use_tool_surface_kind_authority_guard() {
    reducer_normal_mutations_do_not_reintroduce_tool_surface_kind_authority_fields();
}

#[test]
fn profile_default_surfaces_do_not_use_tool_surface_kind_authority_guard() {
    let source = include_str!("workspace/profile.rs");
    let profile_block = source_block_between(
        source,
        "pub struct WorkspaceProfile {",
        "#[derive(Debug, Clone, PartialEq, Eq, Default)]",
        "WorkspaceProfile",
    );
    let default_surface_block = source_block_between(
        include_str!("workspace/state.rs"),
        "pub struct WorkspaceDefaultToolSurface {",
        "impl WorkspaceDefaultToolSurface",
        "WorkspaceDefaultToolSurface",
    );

    assert!(profile_block.contains("pub default_surfaces: Vec<WorkspaceDefaultToolSurface>"));
    assert!(
        !profile_block.contains("default_tool_surfaces: Vec<ToolSurfaceKind>"),
        "WorkspaceProfile default surfaces must stay stable-key primary"
    );
    assert!(default_surface_block.contains("pub stable_surface_key: ToolSurfaceStableKey"));
    assert!(default_surface_block.contains("pub panel_kind: PanelKind"));
    assert!(
        default_surface_block.contains("pub legacy_tool_surface_kind: Option<ToolSurfaceKind>")
    );
    assert!(source.contains("pub fn new_legacy"));
}

#[test]
fn provider_request_does_not_require_tool_surface_kind_guard() {
    let source = include_str!("surface_provider.rs");
    let request_struct = source_block_between(
        source,
        "pub struct SurfaceProviderRequest {",
        "impl SurfaceProviderRequest",
        "SurfaceProviderRequest",
    );

    assert!(request_struct.contains("pub stable_surface_key: ToolSurfaceStableKey"));
    assert!(request_struct.contains("pub legacy_tool_surface_kind: Option<ToolSurfaceKind>"));
    assert!(
        !request_struct.contains("pub tool_surface_kind: ToolSurfaceKind"),
        "SurfaceProviderRequest must not require ToolSurfaceKind"
    );
}

#[test]
fn shell_menu_enum_compatibility_is_marked_pending_final_cleanup() {
    let surface_provider_source = include_str!("surface_provider.rs");
    let composition_source = include_str!("composition/build_editor_shell.rs");

    assert!(
        surface_provider_source
            .contains("C6C shell UI compatibility boundary pending final cleanup")
    );
    assert!(
        composition_source.contains("C6C shell UI compatibility boundary pending final cleanup")
    );
    assert!(composition_source.contains("RoutedShellAction::SwitchPanelToolSurfaceKindTo"));
    assert!(composition_source.contains("RoutedShellAction::CreatePanelTab"));
}

#[test]
fn normal_surface_classifiers_use_stable_keys_when_available() {
    let viewport_registry_source =
        include_str!("../../../../apps/runenwerk_editor/src/runtime/viewport/instance_registry.rs");
    let surface_session_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/surface_session.rs");

    assert!(viewport_registry_source.contains("surface.stable_surface_key().as_str()"));
    assert!(viewport_registry_source.contains("SCENE_VIEWPORT_SURFACE_KEY"));
    assert!(!viewport_registry_source.contains("legacy_tool_surface_kind()"));

    assert!(
        surface_session_source.contains("retains_live_session_key(surface.stable_surface_key())")
    );
    assert!(surface_session_source.contains("SCENE_VIEWPORT_SURFACE_KEY"));
    assert!(!surface_session_source.contains("legacy_tool_surface_kind()"));
}

#[test]
fn app_command_surface_contract_lookup_is_legacy_named() {
    let dispatch_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/dispatch/mod.rs");

    assert!(dispatch_source.contains("pub(crate) struct LegacySurfaceCommandContract"));
    assert!(dispatch_source.contains("pub(crate) fn resolve_legacy_surface_command_contract"));
    assert!(dispatch_source.contains("C6C command-dispatch compatibility boundary"));
    assert!(
        dispatch_source.contains("tool_surface_kind_for_stable_key(surface.stable_surface_key())")
    );
    assert!(dispatch_source.contains("or_else(|| surface.legacy_tool_surface_kind())"));
}

#[test]
fn no_unmarked_tool_surface_kind_usage_in_normal_path_guard() {
    let controller_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/controller.rs");
    let dispatch_source =
        include_str!("../../../../apps/runenwerk_editor/src/shell/dispatch/mod.rs");
    let surface_provider_source = include_str!("surface_provider.rs");
    let composition_source = include_str!("composition/build_editor_shell.rs");

    assert!(controller_source.contains("tab_stack_chrome_surface_target_pending_c6"));
    assert!(controller_source.contains("C6D repair"));
    assert!(controller_source.contains("stable_surface_key: surface.stable_surface_key().clone()"));
    assert!(!controller_source.contains("unwrap_or(ToolSurfaceKind::Viewport)"));

    assert!(dispatch_source.contains("LegacySurfaceCommandContract"));
    assert!(dispatch_source.contains("C6C legacy app-command compatibility boundary"));
    assert!(
        dispatch_source.contains("tool_surface_kind_for_stable_key(surface.stable_surface_key())")
    );

    let request_struct = source_block_between(
        surface_provider_source,
        "pub struct SurfaceProviderRequest {",
        "impl SurfaceProviderRequest",
        "SurfaceProviderRequest",
    );
    assert!(!request_struct.contains("pub tool_surface_kind: ToolSurfaceKind"));
    assert!(!composition_source.contains("C5 shell UI compatibility boundary pending C6"));
    assert!(
        composition_source.contains("C6C shell UI compatibility boundary pending final cleanup")
    );
}

fn source_block_between<'a>(source: &'a str, start: &str, end: &str, label: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|tail| tail.split(end).next())
        .unwrap_or_else(|| panic!("{label} source block should exist"))
}

fn assert_legacy_boundary_doc(source: &str, needle: &str) {
    let docs = preceding_lines_for(source, needle, 8);
    assert!(
        docs.contains("legacy boundary") || docs.contains("C6B legacy boundary"),
        "`{needle}` must be explicitly documented as a legacy boundary helper"
    );
}

fn preceding_lines_for(source: &str, needle: &str, count: usize) -> String {
    let prefix = source
        .split(needle)
        .next()
        .unwrap_or_else(|| panic!("source should contain `{needle}`"));
    let mut lines = prefix.lines().rev().take(count).collect::<Vec<_>>();
    lines.reverse();
    lines.join("\n")
}

#[test]
fn asset_surface_contracts_use_typed_asset_ids_and_epoch_commands() {
    let asset_id = asset::asset_id(11);
    let source_id = asset::asset_source_id(12);
    let artifact_id = asset::asset_artifact_id(13);
    let browser = AssetBrowserViewModel {
        rows: vec![AssetBrowserRowViewModel {
            asset_id,
            display_name: "Field".to_string(),
            stable_name: "field".to_string(),
            kind: asset::AssetKind::SdfGraph,
            source_id: Some(source_id),
            artifact_count: 1,
            is_selected: true,
            is_dirty: false,
            has_prior_valid_preservation: true,
        }],
        selected: Some(crate::AssetDetailViewModel {
            asset_id,
            display_name: "Field".to_string(),
            stable_name: "field".to_string(),
            kind: asset::AssetKind::SdfGraph,
            source_id: Some(source_id),
            artifact_ids: vec![artifact_id],
            source_lines: Vec::new(),
            artifact_lines: Vec::new(),
            dependency_lines: Vec::new(),
        }),
        catalog_status_lines: Vec::new(),
        dirty_asset_count: 0,
    };
    let inspector = ImportInspectorViewModel {
        selected_asset_id: Some(asset_id),
        pending_dirty_asset_ids: vec![asset_id],
        plan_lines: Vec::new(),
        diagnostic_lines: Vec::new(),
        prior_valid_lines: Vec::new(),
        catalog_status_lines: Vec::new(),
    };
    let command = ShellCommand::ReimportAsset {
        asset_id,
        projection_epoch: 7,
    };

    assert_eq!(browser.rows[0].source_id, Some(source_id));
    assert_eq!(inspector.pending_dirty_asset_ids, vec![asset_id]);
    assert_eq!(command.projection_epoch(), Some(7));
    assert_eq!(
        SurfaceLocalAction::Asset(AssetSurfaceAction::SelectAsset { asset_id }),
        SurfaceLocalAction::Asset(AssetSurfaceAction::SelectAsset { asset_id })
    );
}

#[test]
fn material_surface_contracts_use_typed_ids_and_epoch_commands() {
    let asset_id = asset::asset_id(21);
    let source_id = asset::asset_source_id(22);
    let artifact_id = asset::asset_artifact_id(23);
    let product_id = material_graph::MaterialProductId::new(24);
    let canvas = MaterialGraphCanvasViewModel {
        rows: vec![MaterialGraphSourceRowViewModel {
            asset_id,
            display_name: "Rock".to_string(),
            stable_name: "rock".to_string(),
            source_id: Some(source_id),
            artifact_count: 1,
            is_selected: true,
            has_prior_valid_preservation: false,
        }],
        selected: None,
        graph: MaterialGraphEditorViewModel::default(),
        palette: MaterialNodePaletteViewModel {
            search_query: String::new(),
            categories: Vec::new(),
        },
        texture_picker: MaterialTextureResourcePickerViewModel::default(),
        toolbar: MaterialGraphToolbarViewModel::default(),
        validation_overlays: Vec::new(),
        active_diagnostic_index: None,
        node_picker: MaterialNodePickerViewModel::default(),
        shortcuts: Vec::new(),
        undo_redo: MaterialUndoRedoViewModel::default(),
        catalog_status_lines: Vec::new(),
        diagnostic_rows: Vec::new(),
        resource_binding_diagnostics: Vec::new(),
        diagnostic_lines: Vec::new(),
    };
    let preview = MaterialPreviewViewModel {
        selected_asset_id: Some(asset_id),
        active_product_id: Some(product_id),
        artifact_id: Some(artifact_id),
        viewport_product_id: Some(editor_viewport::ExpressionProductId(25)),
        specialization_fragment: Some("material.first_slice.render_material".to_string()),
        prepared_parameter_payload_bytes: 16,
        preview_status: MaterialPreviewStatusViewModel::default(),
        diagnostic_rows: Vec::new(),
        resource_binding_diagnostics: Vec::new(),
        preview_status_lines: Vec::new(),
        diagnostic_lines: Vec::new(),
    };
    let command = ShellCommand::BuildMaterialPreview {
        asset_id,
        projection_epoch: 9,
    };

    assert_eq!(canvas.rows[0].source_id, Some(source_id));
    assert_eq!(preview.active_product_id, Some(product_id));
    assert_eq!(command.projection_epoch(), Some(9));
    assert_eq!(
        SurfaceLocalAction::Material(MaterialSurfaceAction::BuildMaterialPreview { asset_id }),
        SurfaceLocalAction::Material(MaterialSurfaceAction::BuildMaterialPreview { asset_id })
    );
}

#[test]
fn toolbar_omits_global_transform_tool_buttons() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_001),
                    stable_name: "menu_file",
                    label: "File".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_001),
                    stable_name: "workspace_scene",
                    label: "Scene".to_string(),
                    is_active: true,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    for removed_widget in [
        crate::TOOLBAR_SELECT_BUTTON_WIDGET_ID,
        crate::TOOLBAR_TRANSLATE_BUTTON_WIDGET_ID,
        crate::TOOLBAR_ROTATE_BUTTON_WIDGET_ID,
        crate::TOOLBAR_SCALE_BUTTON_WIDGET_ID,
    ] {
        assert!(
            build.tree.walk().all(|node| node.id != removed_widget),
            "global transform tool buttons should not be projected in the top toolbar",
        );
    }
}

#[test]
fn top_bar_menu_and_workspace_buttons_map_to_shell_commands() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_001),
                    stable_name: "menu_file",
                    label: "File".to_string(),
                    is_active: true,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_100),
                    stable_name: "file_save",
                    label: "Save".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_101),
                    stable_name: "file_save_as",
                    label: "Save As".to_string(),
                    is_active: false,
                    enabled: false,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_002),
                    stable_name: "workspace_modelling",
                    label: "Modelling".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_003),
                    stable_name: "workspace_editor_design",
                    label: "Editor Design".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_004),
                    stable_name: "workspace_plus",
                    label: "+".to_string(),
                    is_active: true,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_402),
                    stable_name: "workspace_menu_editor_design",
                    label: "Editor Design".to_string(),
                    is_active: false,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(crate::TOOLBAR_FILE_MENU_WIDGET_ID),
                UiInteraction::Activated(crate::toolbar_menu_item_widget_id(1)),
                UiInteraction::Activated(crate::toolbar_menu_item_widget_id(2)),
                UiInteraction::Activated(crate::TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID),
                UiInteraction::Activated(toolbar_workspace_close_widget_id(
                    crate::MODELLING_WORKSPACE_PROFILE_ID,
                )),
                UiInteraction::Activated(crate::TOOLBAR_EDITOR_DESIGN_WORKSPACE_WIDGET_ID),
                UiInteraction::Activated(crate::TOOLBAR_ADD_WORKSPACE_WIDGET_ID),
            ],
        },
        &build.projection_artifacts,
    );

    assert_eq!(
        commands,
        vec![
            ShellCommand::ToggleToolbarMenu {
                menu: crate::ToolbarMenuKind::File,
            },
            ShellCommand::RunToolbarCommand {
                command: crate::ToolbarCommandKind::SaveScene,
            },
            ShellCommand::NoOp,
            ShellCommand::SwitchWorkspaceProfile {
                profile_id: crate::MODELLING_WORKSPACE_PROFILE_ID,
            },
            ShellCommand::CloseWorkspaceProfile {
                profile_id: crate::MODELLING_WORKSPACE_PROFILE_ID,
            },
            ShellCommand::SwitchWorkspaceProfile {
                profile_id: crate::EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
            },
            ShellCommand::ToggleToolbarMenu {
                menu: crate::ToolbarMenuKind::Workspace,
            },
        ]
    );
    let modelling_chrome =
        toolbar_workspace_chrome_widget_id(crate::MODELLING_WORKSPACE_PROFILE_ID);
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        modelling_chrome,
        toolbar_workspace_close_widget_id(crate::MODELLING_WORKSPACE_PROFILE_ID),
        ui_definition::UiChromeSlotKindDefinition::CloseAffordance,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        modelling_chrome,
        crate::TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
        ui_definition::UiChromeSlotKindDefinition::Label,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        modelling_chrome,
        crate::TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
        ui_definition::UiChromeSlotKindDefinition::DragRegion,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        modelling_chrome,
        toolbar_workspace_active_indicator_widget_id(crate::MODELLING_WORKSPACE_PROFILE_ID),
        ui_definition::UiChromeSlotKindDefinition::ActiveIndicator,
    );
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    assert_horizontal_slot_order(
        &layouts,
        toolbar_workspace_close_widget_id(crate::MODELLING_WORKSPACE_PROFILE_ID),
        crate::TOOLBAR_MODELLING_WORKSPACE_WIDGET_ID,
        toolbar_workspace_active_indicator_widget_id(crate::MODELLING_WORKSPACE_PROFILE_ID),
    );

    let workspace_menu_frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(3_004),
                stable_name: "workspace_plus",
                label: "+".to_string(),
                is_active: true,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    );
    let workspace_menu_build = build_editor_shell_frame(
        &workspace_menu_frame_model,
        &ThemeTokens::default(),
        &workspace,
    );
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                crate::toolbar_menu_item_widget_id(2),
            )],
        },
        &workspace_menu_build.projection_artifacts,
    );
    assert_eq!(
        commands,
        vec![ShellCommand::SwitchWorkspaceProfile {
            profile_id: crate::EDITOR_DESIGN_WORKSPACE_PROFILE_ID,
        }]
    );
}

#[test]
fn toolbar_route_slots_use_app_supplied_route_actions_before_fallback() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(2_001),
                stable_name: "menu_file",
                label: "File".to_string(),
                is_active: true,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    )
    .with_active_ui_definitions(
        None,
        Some(editor_definition::EditorToolbarBinding {
            template: "unused.toolbar.template".into(),
            workspace_catalog: None,
            routes: Vec::new(),
            availability: Vec::new(),
            menus: Vec::new(),
            menu_items: vec![editor_definition::EditorToolbarMenuItemBinding {
                menu_id: "file".to_string(),
                item_id: "apply".to_string(),
                label: "Apply".to_string(),
                route: ui_definition::UiRouteSlotId::new("authored.apply-selected"),
                availability: None,
            }],
        }),
        None,
    )
    .with_route_actions(BTreeMap::from([(
        "authored.apply-selected".to_string(),
        RoutedShellAction::ApplySelectedEditorDefinition,
    )]));
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                crate::toolbar_menu_item_widget_id(0),
            )],
        },
        &build.projection_artifacts,
    );

    assert_eq!(commands, vec![ShellCommand::ApplySelectedEditorDefinition]);
}

#[test]
fn active_top_bar_menu_projects_as_popup_without_pushing_content_down() {
    let active_frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_001),
                    stable_name: "menu_file",
                    label: "File".to_string(),
                    is_active: true,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_100),
                    stable_name: "file_save",
                    label: "Save".to_string(),
                    is_active: false,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let inactive_frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![ToolbarButtonViewModel {
                id: editor_core::ToolId(2_001),
                stable_name: "menu_file",
                label: "File".to_string(),
                is_active: false,
                enabled: true,
            }],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let theme = ThemeTokens::default();
    let active = build_editor_shell_frame(&active_frame_model, &theme, &workspace);
    let inactive = build_editor_shell_frame(&inactive_frame_model, &theme, &workspace);
    let active_layouts = ui_runtime::compute_tree_layout(
        &active.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    let inactive_layouts = ui_runtime::compute_tree_layout(
        &inactive.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );

    assert!(
        active_layouts.contains_key(&crate::TOOLBAR_MENU_POPUP_WIDGET_ID),
        "active menu should project a retained popup"
    );
    assert!(
        !inactive_layouts.contains_key(&crate::TOOLBAR_MENU_POPUP_WIDGET_ID),
        "inactive menu should not project popup layout"
    );
    assert_eq!(
        active_layouts
            .get(&crate::BODY_ROOT_WIDGET_ID)
            .expect("active body root")
            .bounds,
        inactive_layouts
            .get(&crate::BODY_ROOT_WIDGET_ID)
            .expect("inactive body root")
            .bounds,
        "opening a menu must not add a second toolbar row or push content down"
    );
    assert!(
        active
            .projection_artifacts
            .interaction_model
            .menu_scopes
            .iter()
            .any(
                |scope| scope.popup_widget_id == crate::TOOLBAR_MENU_POPUP_WIDGET_ID
                    && scope.anchor_widget_id == crate::TOOLBAR_FILE_MENU_WIDGET_ID
                    && scope.parent_scope_id.is_none()
            ),
        "toolbar popup should expose a formed Interaction V2 menu-stack scope",
    );
    assert!(
        active
            .projection_artifacts
            .interaction_model
            .scroll_owners
            .iter()
            .any(|owner| owner.widget_id == crate::TOOLBAR_MENU_POPUP_SCROLL_WIDGET_ID),
        "toolbar popup scroll should expose a formed Interaction V2 scroll owner",
    );
    assert!(
        active
            .projection_artifacts
            .interaction_model
            .menu_sizing
            .iter()
            .any(|sizing| {
                sizing.popup_widget_id == crate::TOOLBAR_MENU_POPUP_WIDGET_ID
                    && sizing.list_widget_id == crate::TOOLBAR_MENU_POPUP_LIST_WIDGET_ID
                    && sizing.item_width
                        == ui_definition::UiMenuItemWidthDefinition::FillToMenuWidth
                    && sizing.overflow == ui_definition::UiMenuOverflowDefinition::ScrollWhenClamped
            }),
        "toolbar popup should expose formed Interaction V2 menu sizing",
    );
}

#[test]
fn entity_table_control_rail_overflows_and_scrolls_from_child_controls() {
    let theme = ThemeTokens::default();
    let tree = ui_tree::UiTree::new(build_entity_table_panel(
        &crate::EntityTableViewModel::default(),
        &theme,
        PanelInstanceId::try_from_raw(1).unwrap(),
        None,
    ));
    let bounds = ui_math::UiRect::new(0.0, 0.0, 260.0, 240.0);
    let mut runtime = ui_runtime::UiRuntime::new();
    let layouts = runtime.compute_layout(&tree, bounds);
    let max_offset = runtime
        .max_scroll_offset_for_layout_axis(
            &tree,
            &layouts,
            ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID,
            Axis::Horizontal,
        )
        .expect("entity controls rail should be a scroll node");

    assert!(
        max_offset > 0.0,
        "entity table controls must measure to content width and overflow when narrow"
    );

    let search_bounds = layouts
        .get(&ENTITY_TABLE_SEARCH_WIDGET_ID)
        .expect("entity search layout should exist")
        .bounds;
    let pointer = ui_math::UiPoint::new(
        search_bounds.x + search_bounds.width * 0.5,
        search_bounds.y + search_bounds.height * 0.5,
    );
    assert_eq!(
        ui_runtime::hit_test_widget(&tree, &layouts, pointer),
        Some(ENTITY_TABLE_SEARCH_WIDGET_ID),
        "the controls rail row must have stable identity so child controls win hit testing",
    );
    let _ = runtime.dispatch_input(
        &tree,
        &layouts,
        &UiInputEvent::Pointer(PointerEvent {
            kind: PointerEventKind::Scroll,
            position: pointer,
            delta: ui_math::UiVector::new(0.0, -8.0),
            button: None,
            modifiers: Modifiers::default(),
            click_count: 0,
            ..Default::default()
        }),
    );

    assert!(
        runtime.scroll_offset_for_axis(ENTITY_TABLE_CONTROLS_SCROLL_WIDGET_ID, Axis::Horizontal)
            > 0.0,
        "wheel input over the search child should scroll the controls rail owner"
    );
}

#[test]
fn toolbar_separator_projects_as_centered_visible_divider() {
    let frame_model = EditorShellFrameModel::new(
        ToolbarViewModel {
            buttons: vec![
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_001),
                    stable_name: "menu_file",
                    label: "File".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_002),
                    stable_name: "menu_edit",
                    label: "Edit".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(2_003),
                    stable_name: "menu_window",
                    label: "Window".to_string(),
                    is_active: false,
                    enabled: true,
                },
                ToolbarButtonViewModel {
                    id: editor_core::ToolId(3_001),
                    stable_name: "workspace_scene",
                    label: "Scene".to_string(),
                    is_active: true,
                    enabled: true,
                },
            ],
        },
        BTreeMap::new(),
    );
    let workspace = sample_workspace_state();
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    let separator_node = build
        .tree
        .walk()
        .find(|node| node.id == crate::TOOLBAR_SEPARATOR_WIDGET_ID)
        .expect("toolbar separator should exist");
    assert!(matches!(
        &separator_node.kind,
        crate::UiNodeKind::Divider(_)
    ));

    let row = layouts
        .get(&crate::TOOLBAR_ROW_WIDGET_ID)
        .expect("toolbar row should have layout");
    let separator = layouts
        .get(&crate::TOOLBAR_SEPARATOR_WIDGET_ID)
        .expect("toolbar separator should have layout");

    assert!((separator.bounds.width - 1.0).abs() < 0.001);
    assert!((separator.bounds.height - 14.0).abs() < 0.001);
    assert!(
        (separator.bounds.y - (row.bounds.y + (row.bounds.height - separator.bounds.height) * 0.5))
            .abs()
            < 0.001,
        "toolbar separator should be centered vertically in the row"
    );
}

#[test]
fn default_scene_workspace_uses_viewport_left_and_hierarchy_over_inspector_right() {
    let workspace = sample_workspace_state();
    let projection =
        crate::project_workspace_for_shell(&workspace).expect("default layout should project");
    let crate::ProjectedWorkspaceHostSlot::Split {
        axis: WorkspaceSplitAxis::Vertical,
        fraction: body_console_fraction,
        first_child: left_right,
        ..
    } = &projection.root_host
    else {
        panic!("default root host should be a vertical graph split");
    };
    let crate::ProjectedWorkspaceHostSlot::Split {
        axis: WorkspaceSplitAxis::Horizontal,
        fraction: left_right_fraction,
        first_child: viewport,
        second_child: right_sidebar,
        ..
    } = left_right.as_ref()
    else {
        panic!("default upper body should be a horizontal graph split");
    };
    let crate::ProjectedWorkspaceHostSlot::Split {
        axis: WorkspaceSplitAxis::Vertical,
        fraction: center_right_fraction,
        first_child: outliner,
        second_child: inspector,
        ..
    } = right_sidebar.as_ref()
    else {
        panic!("default right sidebar should be a vertical graph split");
    };
    let crate::ProjectedWorkspaceHostSlot::TabStack {
        tab_stack: viewport,
        ..
    } = viewport.as_ref()
    else {
        panic!("default viewport slot should be a tab stack");
    };
    let crate::ProjectedWorkspaceHostSlot::TabStack {
        tab_stack: outliner,
        ..
    } = outliner.as_ref()
    else {
        panic!("default outliner slot should be a tab stack");
    };
    let crate::ProjectedWorkspaceHostSlot::TabStack {
        tab_stack: inspector,
        ..
    } = inspector.as_ref()
    else {
        panic!("default inspector slot should be a tab stack");
    };

    assert_eq!(*body_console_fraction, 0.78);
    assert_eq!(*left_right_fraction, 0.72);
    assert_eq!(*center_right_fraction, 0.56);
    assert_eq!(
        viewport.active_panel.as_ref().map(|panel| panel.panel_kind),
        Some(PanelKind::Viewport)
    );
    assert_eq!(
        outliner.active_panel.as_ref().map(|panel| panel.panel_kind),
        Some(PanelKind::Outliner)
    );
    assert_eq!(
        inspector
            .active_panel
            .as_ref()
            .map(|panel| panel.panel_kind),
        Some(PanelKind::Inspector)
    );
}

#[test]
fn panel_and_tool_surface_definition_keys_share_workspace_vocabulary() {
    for (panel_kind, tool_surface_kind, expected_key) in [
        (PanelKind::Outliner, ToolSurfaceKind::Outliner, "outliner"),
        (
            PanelKind::EntityTable,
            ToolSurfaceKind::EntityTable,
            "entity_table",
        ),
        (PanelKind::Viewport, ToolSurfaceKind::Viewport, "viewport"),
        (
            PanelKind::Inspector,
            ToolSurfaceKind::Inspector,
            "inspector",
        ),
        (PanelKind::Console, ToolSurfaceKind::Console, "console"),
        (
            PanelKind::EditorDesignOutliner,
            ToolSurfaceKind::EditorDesignOutliner,
            "editor_design_outliner",
        ),
        (
            PanelKind::UiHierarchy,
            ToolSurfaceKind::UiHierarchy,
            "ui_hierarchy",
        ),
        (PanelKind::UiCanvas, ToolSurfaceKind::UiCanvas, "ui_canvas"),
        (
            PanelKind::StyleInspector,
            ToolSurfaceKind::StyleInspector,
            "style_inspector",
        ),
        (PanelKind::Bindings, ToolSurfaceKind::Bindings, "bindings"),
        (
            PanelKind::DockLayoutPreview,
            ToolSurfaceKind::DockLayoutPreview,
            "dock_layout_preview",
        ),
        (
            PanelKind::ThemeEditor,
            ToolSurfaceKind::ThemeEditor,
            "theme_editor",
        ),
        (
            PanelKind::ShortcutEditor,
            ToolSurfaceKind::ShortcutEditor,
            "shortcut_editor",
        ),
        (
            PanelKind::MenuEditor,
            ToolSurfaceKind::MenuEditor,
            "menu_editor",
        ),
        (
            PanelKind::DefinitionValidation,
            ToolSurfaceKind::DefinitionValidation,
            "definition_validation",
        ),
        (
            PanelKind::CommandDiff,
            ToolSurfaceKind::CommandDiff,
            "command_diff",
        ),
        (
            PanelKind::AssetBrowser,
            ToolSurfaceKind::AssetBrowser,
            "asset_browser",
        ),
        (
            PanelKind::ImportInspector,
            ToolSurfaceKind::ImportInspector,
            "import_inspector",
        ),
        (
            PanelKind::FieldProductViewer,
            ToolSurfaceKind::FieldProductViewer,
            "field_product_viewer",
        ),
        (
            PanelKind::SdfBrushBrowser,
            ToolSurfaceKind::SdfBrushBrowser,
            "sdf_brush_browser",
        ),
        (
            PanelKind::GraphCanvas,
            ToolSurfaceKind::GraphCanvas,
            "graph_canvas",
        ),
        (
            PanelKind::Diagnostics,
            ToolSurfaceKind::Diagnostics,
            "diagnostics",
        ),
        (
            PanelKind::RuntimeDebug,
            ToolSurfaceKind::RuntimeDebug,
            "runtime_debug",
        ),
        (
            PanelKind::FieldLayerStack,
            ToolSurfaceKind::FieldLayerStack,
            "field_layer_stack",
        ),
        (
            PanelKind::SdfGraphCanvas,
            ToolSurfaceKind::SdfGraphCanvas,
            "sdf_graph_canvas",
        ),
        (
            PanelKind::MaterialGraphCanvas,
            ToolSurfaceKind::MaterialGraphCanvas,
            "material_graph_canvas",
        ),
        (
            PanelKind::MaterialInspector,
            ToolSurfaceKind::MaterialInspector,
            "material_inspector",
        ),
        (
            PanelKind::MaterialPreview,
            ToolSurfaceKind::MaterialPreview,
            "material_preview",
        ),
        (
            PanelKind::TextureViewer,
            ToolSurfaceKind::TextureViewer,
            "texture_viewer",
        ),
        (
            PanelKind::VolumeTextureViewer,
            ToolSurfaceKind::VolumeTextureViewer,
            "volume_texture_viewer",
        ),
        (
            PanelKind::ProcgenGraphCanvas,
            ToolSurfaceKind::ProcgenGraphCanvas,
            "procgen_graph_canvas",
        ),
        (
            PanelKind::ProcgenPreview,
            ToolSurfaceKind::ProcgenPreview,
            "procgen_preview",
        ),
        (
            PanelKind::GameplayGraphCanvas,
            ToolSurfaceKind::GameplayGraphCanvas,
            "gameplay_graph_canvas",
        ),
        (
            PanelKind::GameplayCompilerDiagnostics,
            ToolSurfaceKind::GameplayCompilerDiagnostics,
            "gameplay_compiler_diagnostics",
        ),
        (
            PanelKind::ParticleGraphCanvas,
            ToolSurfaceKind::ParticleGraphCanvas,
            "particle_graph_canvas",
        ),
        (
            PanelKind::ParticlePreview,
            ToolSurfaceKind::ParticlePreview,
            "particle_preview",
        ),
        (
            PanelKind::PhysicsAuthoring,
            ToolSurfaceKind::PhysicsAuthoring,
            "physics_authoring",
        ),
        (
            PanelKind::PhysicsDebug,
            ToolSurfaceKind::PhysicsDebug,
            "physics_debug",
        ),
        (PanelKind::Timeline, ToolSurfaceKind::Timeline, "timeline"),
        (
            PanelKind::CurveEditor,
            ToolSurfaceKind::CurveEditor,
            "curve_editor",
        ),
        (
            PanelKind::AnimationGraphCanvas,
            ToolSurfaceKind::AnimationGraphCanvas,
            "animation_graph_canvas",
        ),
        (
            PanelKind::SimulationPreview,
            ToolSurfaceKind::SimulationPreview,
            "simulation_preview",
        ),
        (
            PanelKind::SimulationDiagnostics,
            ToolSurfaceKind::SimulationDiagnostics,
            "simulation_diagnostics",
        ),
        (
            PanelKind::Placeholder,
            ToolSurfaceKind::Placeholder,
            "placeholder",
        ),
    ] {
        assert_eq!(panel_kind_definition_key(panel_kind), expected_key);
        assert_eq!(
            tool_surface_kind_definition_key(tool_surface_kind),
            expected_key
        );
    }
}

#[test]
fn provider_route_activation_maps_to_surface_local_dispatch_command() {
    let workspace = sample_workspace_state();
    let (panel_id, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let frame_model = frame_model_with_surface_route(
        &workspace,
        surface_id,
        WidgetId(50_000),
        SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity {
            entity: EntityId(42),
        }),
    );

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(WidgetId(50_000))],
        },
        &build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [ShellCommand::DispatchSurfaceLocalAction {
            provider_id,
            tool_surface_instance_id,
            target,
            action: SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity { entity }),
            projection_epoch,
        }] if *provider_id == SurfaceProviderId::try_from_raw(77).unwrap()
            && *tool_surface_instance_id == surface_id
            && target.panel_instance_id == panel_id
            && target.active_tool_surface == Some(surface_id)
            && *entity == EntityId(42)
            && *projection_epoch == build.projection_artifacts.projection_epoch
    ));
}

#[test]
fn graph_canvas_interaction_maps_to_generic_surface_interaction() {
    let workspace = sample_workspace_state();
    let (panel_id, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let widget_id = WidgetId(50_010);
    let mut frame_model = frame_model_for_workspace(&workspace);
    let frame = frame_model
        .surfaces
        .get_mut(&surface_id)
        .expect("material graph canvas surface should exist in frame model");
    frame
        .routes
        .insert(widget_id, SurfaceLocalRoute::provider_owned_graph_canvas());
    frame.artifact.root = label(
        widget_id,
        frame.title.clone(),
        ThemeTokens::default().body_small_text_style(ui_text::FontId(1)),
    );

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::GraphCanvasAction {
                target: widget_id,
                action: ui_graph_editor::GraphCanvasAction::EndNodeDrag {
                    node: ui_graph_editor::GraphNodeKey(3),
                    delta: ui_graph_editor::GraphVector::new(12, -6),
                },
            }],
        },
        &build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [ShellCommand::DispatchSurfaceInteraction {
            provider_id,
            tool_surface_instance_id,
            target,
            interaction: SurfaceInteraction::GraphCanvasAction(
                ui_graph_editor::GraphCanvasAction::EndNodeDrag { node, delta }
            ),
            projection_epoch,
        }] if *provider_id == SurfaceProviderId::try_from_raw(77).unwrap()
            && *tool_surface_instance_id == surface_id
            && target.panel_instance_id == panel_id
            && target.active_tool_surface == Some(surface_id)
            && *node == ui_graph_editor::GraphNodeKey(3)
            && *delta == ui_graph_editor::GraphVector::new(12, -6)
            && *projection_epoch == build.projection_artifacts.projection_epoch
    ));
}

#[test]
fn graph_canvas_shortcut_actions_map_to_generic_surface_interactions() {
    let workspace = sample_workspace_state();
    let (panel_id, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let widget_id = WidgetId(50_011);
    let mut frame_model = frame_model_for_workspace(&workspace);
    let frame = frame_model
        .surfaces
        .get_mut(&surface_id)
        .expect("material graph canvas surface should exist in frame model");
    frame
        .routes
        .insert(widget_id, SurfaceLocalRoute::provider_owned_graph_canvas());
    frame.artifact.root = label(
        widget_id,
        frame.title.clone(),
        ThemeTokens::default().body_small_text_style(ui_text::FontId(1)),
    );

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::GraphCanvasAction {
                    target: widget_id,
                    action: ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                        ui_graph_editor::GraphShortcutAction::AddNode,
                    ),
                },
                UiInteraction::GraphCanvasAction {
                    target: widget_id,
                    action: ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                        ui_graph_editor::GraphShortcutAction::Undo,
                    ),
                },
            ],
        },
        &build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::DispatchSurfaceInteraction {
                provider_id,
                tool_surface_instance_id,
                target,
                interaction: SurfaceInteraction::GraphCanvasAction(
                    ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                        ui_graph_editor::GraphShortcutAction::AddNode
                    )
                ),
                projection_epoch,
            },
            ShellCommand::DispatchSurfaceInteraction {
                interaction: SurfaceInteraction::GraphCanvasAction(
                    ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                        ui_graph_editor::GraphShortcutAction::Undo
                    )
                ),
                ..
            },
        ] if *provider_id == SurfaceProviderId::try_from_raw(77).unwrap()
            && *tool_surface_instance_id == surface_id
            && target.panel_instance_id == panel_id
            && target.active_tool_surface == Some(surface_id)
            && *projection_epoch == build.projection_artifacts.projection_epoch
    ));
}

#[test]
fn provider_route_rejects_mismatched_structural_context() {
    let workspace = sample_workspace_state();
    let (_, surface_id) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let frame_model = frame_model_with_surface_route(
        &workspace,
        surface_id,
        WidgetId(50_001),
        SurfaceLocalAction::Outliner(OutlinerSurfaceAction::SelectEntity {
            entity: EntityId(42),
        }),
    );
    let mut build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    build
        .projection_artifacts
        .widget_structural_context_by_id
        .insert(
            WidgetId(50_001),
            crate::StructuralWidgetRoutingContext {
                panel_instance_id: PanelInstanceId::try_from_raw(999).unwrap(),
                active_tool_surface: None,
                tab_stack_id: crate::TabStackId::try_from_raw(999).unwrap(),
            },
        );

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(WidgetId(50_001))],
        },
        &build.projection_artifacts,
    );

    assert_eq!(commands, vec![ShellCommand::NoOp]);
}

#[test]
fn surface_text_and_keyboard_input_map_to_typed_entity_table_actions() {
    let widget_id = WidgetId(50_100);
    let actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        widget_id,
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText {
            text: String::new(),
        }),
        vec![
            UiInteraction::TextInput {
                target: widget_id,
                event: TextInputEvent {
                    text: "alpha".to_string(),
                },
            },
            UiInteraction::KeyboardInput {
                target: widget_id,
                event: keyboard_event(Key::Backspace),
            },
        ],
    );

    assert_eq!(
        actions,
        vec![
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::AppendSearchText {
                text: "alpha".to_string(),
            }),
            SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::BackspaceSearch),
        ]
    );
}

#[test]
fn material_graph_inspector_text_input_maps_to_source_backed_edit() {
    let actions = mapped_surface_actions_for_route(
        PanelKind::Outliner,
        WidgetId(50_105),
        SurfaceLocalAction::Material(MaterialSurfaceAction::SetNodeValue {
            node_id: graph::NodeId::new(3),
            key: "roughness".to_string(),
            value: String::new(),
        }),
        vec![UiInteraction::TextInput {
            target: WidgetId(50_105),
            event: TextInputEvent {
                text: "0.25".to_string(),
            },
        }],
    );

    assert_eq!(
        actions,
        vec![SurfaceLocalAction::Material(
            MaterialSurfaceAction::SetNodeValue {
                node_id: graph::NodeId::new(3),
                key: "roughness".to_string(),
                value: "0.25".to_string(),
            },
        )]
    );
}

#[test]
fn surface_toggle_select_and_table_row_input_map_to_typed_entity_table_actions() {
    let selected_only_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_110),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetSelectedOnly {
            selected_only: false,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_110),
            checked: true,
        }],
    );
    assert_eq!(
        selected_only_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetSelectedOnly {
                selected_only: true,
            }
        )]
    );

    let roots_only_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_111),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SetHierarchyFilter {
            filter: EntityTableHierarchyFilter::All,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_111),
            checked: true,
        }],
    );
    assert_eq!(
        roots_only_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetHierarchyFilter {
                filter: EntityTableHierarchyFilter::RootsOnly,
            }
        )]
    );

    let component_filter = EntityTableComponentFilter::Has(ComponentTypeId(9));
    let select_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_112),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectComponentFilter {
            filters: vec![EntityTableComponentFilter::All, component_filter],
        }),
        vec![UiInteraction::SelectChanged {
            target: WidgetId(50_112),
            index: 1,
        }],
    );
    assert_eq!(
        select_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SetComponentFilter {
                filter: component_filter,
            }
        )]
    );

    let row_actions = mapped_surface_actions_for_route(
        PanelKind::EntityTable,
        WidgetId(50_113),
        SurfaceLocalAction::EntityTable(EntityTableSurfaceAction::SelectRow {
            entities: vec![EntityId(7), EntityId(8)],
        }),
        vec![UiInteraction::TableRowSelected {
            target: WidgetId(50_113),
            row_index: 1,
        }],
    );
    assert_eq!(
        row_actions,
        vec![SurfaceLocalAction::EntityTable(
            EntityTableSurfaceAction::SelectEntity {
                entity: EntityId(8),
            }
        )]
    );
}

#[test]
fn surface_text_keyboard_toggle_and_numeric_input_map_to_typed_inspector_actions() {
    let text_widget_id = WidgetId(50_120);
    let text_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        text_widget_id,
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
            index: 3,
            text: String::new(),
        }),
        vec![
            UiInteraction::TextInput {
                target: text_widget_id,
                event: TextInputEvent {
                    text: "Beta".to_string(),
                },
            },
            UiInteraction::KeyboardInput {
                target: text_widget_id,
                event: keyboard_event(Key::Backspace),
            },
            UiInteraction::KeyboardInput {
                target: text_widget_id,
                event: keyboard_event(Key::Enter),
            },
            UiInteraction::KeyboardInput {
                target: text_widget_id,
                event: keyboard_event(Key::Escape),
            },
        ],
    );
    assert_eq!(
        text_actions,
        vec![
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::EditFieldText {
                index: 3,
                text: "Beta".to_string(),
            }),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::BackspaceFieldText { index: 3 }),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::CommitFieldText { index: 3 }),
            SurfaceLocalAction::Inspector(InspectorSurfaceAction::CancelFieldText { index: 3 }),
        ]
    );

    let bool_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        WidgetId(50_121),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldBool {
            index: 4,
            value: false,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_121),
            checked: true,
        }],
    );
    assert_eq!(
        bool_actions,
        vec![SurfaceLocalAction::Inspector(
            InspectorSurfaceAction::SetFieldBool {
                index: 4,
                value: true,
            }
        )]
    );

    let numeric_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        WidgetId(50_122),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SetFieldNumber {
            index: 5,
            value: 0.0,
        }),
        vec![UiInteraction::NumericStepped {
            target: WidgetId(50_122),
            value: 12.5,
        }],
    );
    assert_eq!(
        numeric_actions,
        vec![SurfaceLocalAction::Inspector(
            InspectorSurfaceAction::SetFieldNumber {
                index: 5,
                value: 12.5,
            }
        )]
    );

    let enum_actions = mapped_surface_actions_for_route(
        PanelKind::Inspector,
        WidgetId(50_123),
        SurfaceLocalAction::Inspector(InspectorSurfaceAction::SelectFieldEnum {
            index: 6,
            options: vec!["Nearest".to_string(), "Linear".to_string()],
        }),
        vec![UiInteraction::SelectChanged {
            target: WidgetId(50_123),
            index: 1,
        }],
    );
    assert_eq!(
        enum_actions,
        vec![SurfaceLocalAction::Inspector(
            InspectorSurfaceAction::SetFieldEnum {
                index: 6,
                value: "Linear".to_string(),
            }
        )]
    );

    let root_opaque_actions = mapped_surface_actions_for_route(
        PanelKind::Viewport,
        WidgetId(50_124),
        SurfaceLocalAction::Viewport(ViewportSurfaceAction::SetRootBackgroundOpaque {
            viewport_id: editor_viewport::ViewportId(4),
            enabled: false,
        }),
        vec![UiInteraction::Toggled {
            target: WidgetId(50_124),
            checked: true,
        }],
    );
    assert_eq!(
        root_opaque_actions,
        vec![SurfaceLocalAction::Viewport(
            ViewportSurfaceAction::SetRootBackgroundOpaque {
                viewport_id: editor_viewport::ViewportId(4),
                enabled: true,
            }
        )]
    );
}

#[test]
fn tab_chrome_maps_shell_owned_controls_to_structural_commands() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let projection_epoch = build.projection_artifacts.projection_epoch;

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(tab_stack_new_tab_button_widget_id(viewport_stack)),
                UiInteraction::Activated(tab_stack_split_horizontal_button_widget_id(
                    viewport_stack,
                )),
                UiInteraction::Activated(tab_close_button_widget_id(viewport_stack, 0)),
            ],
        },
        &build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::ToggleTabStackCreateSurfaceMenu {
                tab_stack_id: create_stack,
                anchor_widget_id,
            },
            ShellCommand::SplitTabStackArea {
                tab_stack_id: split_stack,
                ..
            },
            ShellCommand::ClosePanelTab {
                tab_stack_id: close_stack,
                panel_instance_id: close_panel,
                projection_epoch: close_epoch,
            },
        ] if *create_stack == viewport_stack
            && *anchor_widget_id == tab_stack_new_tab_button_widget_id(viewport_stack)
            && *split_stack == viewport_stack
            && *close_stack == viewport_stack
            && *close_panel == viewport_panel
            && *close_epoch == projection_epoch
    ));
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        tab_chrome_widget_id(viewport_stack, 0),
        tab_close_button_widget_id(viewport_stack, 0),
        ui_definition::UiChromeSlotKindDefinition::CloseAffordance,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        tab_chrome_widget_id(viewport_stack, 0),
        tab_close_button_widget_id(viewport_stack, 0),
        ui_definition::UiChromeSlotKindDefinition::CommandArea,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        tab_chrome_widget_id(viewport_stack, 0),
        crate::tab_button_widget_id(viewport_stack, 0),
        ui_definition::UiChromeSlotKindDefinition::Label,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        tab_chrome_widget_id(viewport_stack, 0),
        crate::tab_button_widget_id(viewport_stack, 0),
        ui_definition::UiChromeSlotKindDefinition::DragRegion,
    );
    assert_chrome_slot(
        &build.projection_artifacts.interaction_model,
        tab_chrome_widget_id(viewport_stack, 0),
        tab_active_indicator_widget_id(viewport_stack, 0),
        ui_definition::UiChromeSlotKindDefinition::ActiveIndicator,
    );
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    assert_horizontal_slot_order(
        &layouts,
        tab_close_button_widget_id(viewport_stack, 0),
        crate::tab_button_widget_id(viewport_stack, 0),
        tab_active_indicator_widget_id(viewport_stack, 0),
    );
}

#[test]
fn tab_stack_area_actions_are_projected_as_popup_menu() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let inactive_frame_model = frame_model_for_workspace(&workspace);
    let inactive_build =
        build_editor_shell_frame(&inactive_frame_model, &ThemeTokens::default(), &workspace);

    assert!(!ui_tree_contains_widget(
        &inactive_build.tree.root,
        tab_stack_action_menu_popup_widget_id(viewport_stack)
    ));
    assert!(!ui_tree_contains_widget(
        &inactive_build.tree.root,
        tab_stack_split_horizontal_button_widget_id(viewport_stack)
    ));

    let active_frame_model = frame_model_for_workspace(&workspace)
        .with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::AreaActions,
            tab_stack_id: viewport_stack,
            anchor_widget_id: WidgetId(99_001),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);

    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_action_menu_popup_widget_id(viewport_stack)
    ));
    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_split_horizontal_button_widget_id(viewport_stack)
    ));

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(tab_stack_surface_submenu_anchor_widget_id(
                    viewport_stack,
                )),
                UiInteraction::Activated(tab_stack_split_horizontal_button_widget_id(
                    viewport_stack,
                )),
            ],
        },
        &active_build.projection_artifacts,
    );

    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::ToggleTabStackSurfaceMenu {
                tab_stack_id,
                anchor_widget_id,
            },
            ShellCommand::SplitTabStackArea {
                tab_stack_id: split_stack,
                ..
            },
        ] if *tab_stack_id == viewport_stack
            && *anchor_widget_id == tab_stack_surface_submenu_anchor_widget_id(viewport_stack)
            && *split_stack == viewport_stack
    ));
}

#[test]
fn tab_stack_surface_submenu_keeps_parent_menu_stack_scope() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let active_frame_model = frame_model_for_workspace(&workspace)
        .with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::SurfaceKinds,
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_surface_submenu_anchor_widget_id(viewport_stack),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);

    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_action_menu_popup_widget_id(viewport_stack)
    ));
    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_surface_submenu_anchor_widget_id(viewport_stack)
    ));
    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_surface_menu_popup_widget_id(viewport_stack)
    ));

    let layouts = ui_runtime::compute_tree_layout(
        &active_build.tree,
        ui_math::UiRect::new(0.0, 0.0, 1024.0, 768.0),
        &ui_runtime::UiRuntimeState::default(),
    );
    let surface_popup = layouts
        .get(&tab_stack_surface_menu_popup_widget_id(viewport_stack))
        .expect("surface submenu popup should lay out from its parent menu anchor");
    assert!(
        surface_popup.bounds.width > 0.0 && surface_popup.bounds.height > 0.0,
        "surface submenu must not fall back to a zero-size unanchored layout",
    );

    let scopes = &active_build
        .projection_artifacts
        .interaction_model
        .menu_scopes;
    let parent_scope = scopes
        .iter()
        .find(|scope| {
            scope.popup_widget_id == tab_stack_action_menu_popup_widget_id(viewport_stack)
        })
        .expect("parent area-actions menu scope should be formed");
    let child_scope = scopes
        .iter()
        .find(|scope| {
            scope.popup_widget_id == tab_stack_surface_menu_popup_widget_id(viewport_stack)
        })
        .expect("surface submenu scope should be formed");
    assert_eq!(
        child_scope.parent_scope_id.as_deref(),
        Some(parent_scope.scope_id.as_str()),
        "surface submenu should declare the parent area-actions menu scope",
    );
    assert_eq!(
        child_scope.anchor_widget_id,
        tab_stack_surface_submenu_anchor_widget_id(viewport_stack),
    );
    assert!(
        active_build
            .projection_artifacts
            .interaction_model
            .scroll_owners
            .iter()
            .any(|owner| owner.widget_id == tab_stack_surface_menu_scroll_widget_id(viewport_stack)),
        "surface submenu scroll should be a formed Interaction V2 scroll owner",
    );
    assert!(
        active_build
            .projection_artifacts
            .interaction_model
            .menu_sizing
            .iter()
            .any(|sizing| {
                sizing.popup_widget_id == tab_stack_surface_menu_popup_widget_id(viewport_stack)
                    && sizing.list_widget_id
                        == tab_stack_surface_menu_list_widget_id(viewport_stack)
                    && sizing.item_width
                        == ui_definition::UiMenuItemWidthDefinition::FillToMenuWidth
                    && sizing.overflow == ui_definition::UiMenuOverflowDefinition::ScrollWhenClamped
            }),
        "surface submenu should expose formed Interaction V2 menu sizing",
    );
}

#[test]
fn tab_plus_projects_create_surface_menu_and_routes_selected_kind() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let available_kinds = vec![
        ToolSurfaceKind::Outliner,
        ToolSurfaceKind::EntityTable,
        ToolSurfaceKind::Viewport,
        ToolSurfaceKind::Inspector,
        ToolSurfaceKind::Console,
    ];
    let frame_model = frame_model_for_workspace(&workspace)
        .with_available_tool_surface_kinds(available_kinds.clone());
    let inactive_build =
        build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);

    assert!(!ui_tree_contains_widget(
        &inactive_build.tree.root,
        tab_stack_new_surface_menu_popup_widget_id(viewport_stack)
    ));

    let plus_commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                tab_stack_new_tab_button_widget_id(viewport_stack),
            )],
        },
        &inactive_build.projection_artifacts,
    );
    assert_eq!(
        plus_commands,
        vec![ShellCommand::ToggleTabStackCreateSurfaceMenu {
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(viewport_stack),
        }]
    );

    let active_frame_model =
        frame_model.with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::CreateSurface,
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(viewport_stack),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);
    assert!(ui_tree_contains_widget(
        &active_build.tree.root,
        tab_stack_new_surface_menu_popup_widget_id(viewport_stack)
    ));

    let inspector_index = available_kinds
        .iter()
        .position(|kind| *kind == ToolSurfaceKind::Inspector)
        .expect("inspector kind should be present");
    let create_commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![UiInteraction::Activated(
                tab_stack_new_surface_menu_item_widget_id(viewport_stack, inspector_index),
            )],
        },
        &active_build.projection_artifacts,
    );
    assert!(matches!(
        create_commands.as_slice(),
        [
            ShellCommand::CreatePanelTab {
                tab_stack_id,
                tool_surface_kind: ToolSurfaceKind::Inspector,
                ..
            }
        ] if *tab_stack_id == viewport_stack
    ));
}

#[test]
fn locked_tab_plus_menu_shows_all_kinds_but_routes_only_locked_kind() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let workspace = reduce_workspace(
        &workspace,
        WorkspaceMutation::lock_tab_stack_area_type_legacy(
            viewport_stack,
            Some(ToolSurfaceKind::Viewport),
        )
        .expect("legacy wrapper should create lock mutation"),
    )
    .expect("locking viewport tab stack should succeed");
    let available_kinds = vec![
        ToolSurfaceKind::Outliner,
        ToolSurfaceKind::EntityTable,
        ToolSurfaceKind::Viewport,
        ToolSurfaceKind::Inspector,
        ToolSurfaceKind::Console,
    ];
    let active_frame_model = frame_model_for_workspace(&workspace)
        .with_available_tool_surface_kinds(available_kinds.clone())
        .with_active_tab_stack_popup_menu(Some(ActiveTabStackPopupMenu {
            kind: TabStackPopupMenuKind::CreateSurface,
            tab_stack_id: viewport_stack,
            anchor_widget_id: tab_stack_new_tab_button_widget_id(viewport_stack),
        }));
    let active_build =
        build_editor_shell_frame(&active_frame_model, &ThemeTokens::default(), &workspace);
    let viewport_index = available_kinds
        .iter()
        .position(|kind| *kind == ToolSurfaceKind::Viewport)
        .expect("viewport kind should be present");
    let console_index = available_kinds
        .iter()
        .position(|kind| *kind == ToolSurfaceKind::Console)
        .expect("console kind should be present");

    assert!(button_enabled(
        &active_build.tree.root,
        tab_stack_new_surface_menu_item_widget_id(viewport_stack, viewport_index)
    ));
    assert!(!button_enabled(
        &active_build.tree.root,
        tab_stack_new_surface_menu_item_widget_id(viewport_stack, console_index)
    ));

    let commands = map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: vec![
                UiInteraction::Activated(tab_stack_new_surface_menu_item_widget_id(
                    viewport_stack,
                    console_index,
                )),
                UiInteraction::Activated(tab_stack_new_surface_menu_item_widget_id(
                    viewport_stack,
                    viewport_index,
                )),
            ],
        },
        &active_build.projection_artifacts,
    );
    assert!(matches!(
        commands.as_slice(),
        [
            ShellCommand::NoOp,
            ShellCommand::CreatePanelTab {
                tab_stack_id,
                tool_surface_kind: ToolSurfaceKind::Viewport,
                ..
            },
        ] if *tab_stack_id == viewport_stack
    ));
}

#[test]
fn tab_reorder_drop_slots_are_formed_with_higher_priority_than_split_previews() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(DockingPreviewDropTarget::TabStack {
                tab_stack_id: viewport_stack,
                insert_index: 1,
            }),
            preview_candidates: Vec::new(),
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &ThemeTokens::default(),
        &workspace,
        Some(&docking_visual_state),
    );
    let active_zone = tab_drop_zone_widget_id(viewport_stack, 1);

    assert!(ui_tree_contains_widget(&build.tree.root, active_zone));
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        active_zone,
        ui_definition::UiDockDropZoneKindDefinition::TabReorder,
        ui_definition::UiDockDropZoneStateDefinition::Active,
        0,
    );
    let formed = build
        .projection_artifacts
        .interaction_model
        .dock_drop_zones
        .iter()
        .find(|zone| zone.zone_widget_id == active_zone)
        .expect("active tab reorder drop zone should be formed");
    assert_eq!(
        formed.anchor_widget_id,
        crate::tab_strip_widget_id(viewport_stack)
    );
    assert_eq!(formed.scope, ui_definition::UiDockDropScopeDefinition::Area);
    assert_eq!(formed.side, None);
    assert!(!formed.preview_only);
}

#[test]
fn dock_split_preview_projects_side_slice_without_consuming_layout() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let theme = ThemeTokens::default();
    let anchor_widget_id = tab_stack_container_widget_id(viewport_stack);
    let preview_target = DockingPreviewDropTarget::SplitIntoArea {
        target_tab_stack_id: viewport_stack,
        side: crate::DockSplitSide::Left,
    };
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(preview_target),
            preview_candidates: vec![DockDropCandidate {
                target: preview_target,
                scope: DockDropScope::Area,
                side: crate::DockSplitSide::Left,
                anchor_widget_id,
                state: DockDropCandidateState::Active,
            }],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &theme,
        &workspace,
        Some(&docking_visual_state),
    );
    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_overlay_widget_id(anchor_widget_id)
    ));
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        dock_split_preview_overlay_widget_id(anchor_widget_id),
        ui_definition::UiDockDropZoneKindDefinition::SplitInsertion,
        ui_definition::UiDockDropZoneStateDefinition::Active,
        10,
    );
    let formed_preview = build
        .projection_artifacts
        .interaction_model
        .dock_drop_zones
        .iter()
        .find(|zone| zone.zone_widget_id == dock_split_preview_overlay_widget_id(anchor_widget_id))
        .expect("active split preview zone should be formed");
    assert_eq!(formed_preview.anchor_widget_id, anchor_widget_id);
    assert_eq!(
        formed_preview.scope,
        ui_definition::UiDockDropScopeDefinition::Area
    );
    assert_eq!(
        formed_preview.side,
        Some(ui_definition::UiDockDropSideDefinition::Left)
    );
    assert!(formed_preview.preview_only);

    let bounds = UiRect::new(0.0, 0.0, 960.0, 640.0);
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        bounds,
        &ui_runtime::UiRuntimeState::default(),
    );
    let anchor = layouts
        .get(&anchor_widget_id)
        .expect("target tab stack should have layout");
    let preview = layouts
        .get(&dock_split_preview_panel_widget_id(anchor_widget_id))
        .expect("preview slice should have layout");
    assert!((preview.bounds.x - anchor.bounds.x).abs() <= 0.001);
    assert!((preview.bounds.height - anchor.bounds.height).abs() <= 0.001);
    assert!(preview.bounds.width > 0.0 && preview.bounds.width <= 132.0);
}

#[test]
fn invalid_dock_split_preview_forms_invalid_drop_zone_without_active_target() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let anchor_widget_id = tab_stack_container_widget_id(viewport_stack);
    let preview_target = DockingPreviewDropTarget::SplitIntoArea {
        target_tab_stack_id: viewport_stack,
        side: crate::DockSplitSide::Left,
    };
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: None,
            preview_candidates: vec![DockDropCandidate {
                target: preview_target,
                scope: DockDropScope::Area,
                side: crate::DockSplitSide::Left,
                anchor_widget_id,
                state: DockDropCandidateState::Invalid {
                    reason: DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea,
                },
            }],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &ThemeTokens::default(),
        &workspace,
        Some(&docking_visual_state),
    );

    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_overlay_widget_id(anchor_widget_id)
    ));
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        dock_split_preview_overlay_widget_id(anchor_widget_id),
        ui_definition::UiDockDropZoneKindDefinition::SplitInsertion,
        ui_definition::UiDockDropZoneStateDefinition::Invalid,
        10,
    );
}

#[test]
fn dock_root_split_preview_spans_target_root_edge() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let theme = ThemeTokens::default();
    let anchor_widget_id = workspace_split_host_widget_id(workspace.root_host_id());
    let preview_target = DockingPreviewDropTarget::SplitIntoRoot {
        side: crate::DockSplitSide::Bottom,
    };
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(preview_target),
            preview_candidates: vec![DockDropCandidate {
                target: preview_target,
                scope: DockDropScope::Workspace,
                side: crate::DockSplitSide::Bottom,
                anchor_widget_id,
                state: DockDropCandidateState::Active,
            }],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &theme,
        &workspace,
        Some(&docking_visual_state),
    );
    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_overlay_widget_id(anchor_widget_id)
    ));

    let bounds = UiRect::new(0.0, 0.0, 960.0, 640.0);
    let layouts = ui_runtime::compute_tree_layout(
        &build.tree,
        bounds,
        &ui_runtime::UiRuntimeState::default(),
    );
    let anchor = layouts
        .get(&anchor_widget_id)
        .expect("root target should have layout");
    let preview = layouts
        .get(&dock_split_preview_panel_widget_id(anchor_widget_id))
        .expect("root preview slice should have layout");
    assert!(
        (preview.bounds.y + preview.bounds.height - (anchor.bounds.y + anchor.bounds.height)).abs()
            <= 0.001
    );
    assert!((preview.bounds.width - anchor.bounds.width).abs() <= 0.001);
    assert!(preview.bounds.height > 0.0 && preview.bounds.height <= 42.0);
}

#[test]
fn floating_host_drop_zone_is_formed_only_as_active_workspace_target() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(DockingPreviewDropTarget::NewFloatingHost),
            preview_candidates: Vec::new(),
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &ThemeTokens::default(),
        &workspace,
        Some(&docking_visual_state),
    );

    assert!(ui_tree_contains_widget(
        &build.tree.root,
        crate::FLOATING_DROP_ZONE_WIDGET_ID
    ));
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        crate::FLOATING_DROP_ZONE_WIDGET_ID,
        ui_definition::UiDockDropZoneKindDefinition::FloatingHost,
        ui_definition::UiDockDropZoneStateDefinition::Active,
        40,
    );
}

#[test]
fn dock_scope_previews_render_all_candidates_and_active_label() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let frame_model = frame_model_for_workspace(&workspace);
    let theme = ThemeTokens::default();
    let area_anchor = tab_stack_container_widget_id(viewport_stack);
    let group_anchor = crate::CENTER_RIGHT_SPLIT_WIDGET_ID;
    let workspace_anchor = crate::BODY_ROOT_WIDGET_ID;
    let docking_visual_state = DockingInteractionVisualState {
        active_tab_drag: Some(ActiveTabDragVisualState {
            panel_instance_id: viewport_panel,
            source_tab_stack_id: viewport_stack,
            preview_target: Some(DockingPreviewDropTarget::SplitIntoHost {
                target_host_id: workspace.root_host_id(),
                side: crate::DockSplitSide::Right,
            }),
            preview_candidates: vec![
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoArea {
                        target_tab_stack_id: viewport_stack,
                        side: crate::DockSplitSide::Right,
                    },
                    scope: DockDropScope::Area,
                    side: crate::DockSplitSide::Right,
                    anchor_widget_id: area_anchor,
                    state: DockDropCandidateState::Candidate,
                },
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoHost {
                        target_host_id: workspace.root_host_id(),
                        side: crate::DockSplitSide::Right,
                    },
                    scope: DockDropScope::Group,
                    side: crate::DockSplitSide::Right,
                    anchor_widget_id: group_anchor,
                    state: DockDropCandidateState::Active,
                },
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoRoot {
                        side: crate::DockSplitSide::Right,
                    },
                    scope: DockDropScope::Workspace,
                    side: crate::DockSplitSide::Right,
                    anchor_widget_id: workspace_anchor,
                    state: DockDropCandidateState::Candidate,
                },
            ],
        }),
        active_split_border_widget: None,
    };
    let build = build_editor_shell_frame_with_docking_visual_state(
        &frame_model,
        &theme,
        &workspace,
        Some(&docking_visual_state),
    );

    for anchor in [area_anchor, group_anchor, workspace_anchor] {
        assert!(ui_tree_contains_widget(
            &build.tree.root,
            dock_split_preview_overlay_widget_id(anchor)
        ));
    }
    assert!(ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_label_widget_id(group_anchor)
    ));
    assert!(!ui_tree_contains_widget(
        &build.tree.root,
        dock_split_preview_label_widget_id(area_anchor)
    ));
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        dock_split_preview_overlay_widget_id(area_anchor),
        ui_definition::UiDockDropZoneKindDefinition::SplitInsertion,
        ui_definition::UiDockDropZoneStateDefinition::Candidate,
        10,
    );
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        dock_split_preview_overlay_widget_id(group_anchor),
        ui_definition::UiDockDropZoneKindDefinition::SplitInsertion,
        ui_definition::UiDockDropZoneStateDefinition::Active,
        20,
    );
    assert_dock_drop_zone(
        &build.projection_artifacts.interaction_model,
        dock_split_preview_overlay_widget_id(workspace_anchor),
        ui_definition::UiDockDropZoneKindDefinition::SplitInsertion,
        ui_definition::UiDockDropZoneStateDefinition::Candidate,
        30,
    );
}

#[test]
fn viewport_status_region_forms_scroll_overflow_and_viewport_arbitration_policy() {
    let workspace = sample_workspace_state();
    let (viewport_panel, viewport_surface) =
        panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let viewport_state = workspace
        .tool_surface(viewport_surface)
        .expect("viewport surface should exist");
    let viewport_root = build_viewport_panel(
        &ViewportViewModel {
            viewport_id: Some(editor_viewport::ViewportId(7)),
            details_visible: true,
            statistics_visible: true,
            options_menu_open: true,
            tools_menu_open: true,
            frame_rate_fps: Some(60.0),
            frame_time_ms: Some(16.67),
            overlay_status_lines: vec!["Procgen overlay: 2 region(s)".to_string()],
            ..Default::default()
        },
        &ThemeTokens::default(),
        viewport_panel,
        Some(viewport_surface),
    );
    let mut frame_model = frame_model_for_workspace(&workspace);
    frame_model.surfaces.insert(
        viewport_surface,
        ResolvedSurfaceFrame {
            artifact: SurfacePresentationArtifact::provider(viewport_root),
            ..surface_frame(
                viewport_panel,
                viewport_stack,
                viewport_state,
                WidgetId(viewport_surface.raw() + 10_000),
            )
        },
    );

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    let interaction_model = &build.projection_artifacts.interaction_model;
    let status_widget_id = surface_widget_id(viewport_surface, crate::VIEWPORT_STATUS_WIDGET_ID);
    let region = interaction_model
        .viewport_status_regions
        .iter()
        .find(|region| region.status_widget_id == status_widget_id)
        .expect("viewport status region should be formed");

    assert_eq!(
        region.overflow,
        ui_definition::UiStatusOverflowPolicyDefinition::SingleRowHorizontalScroll
    );
    assert_eq!(
        region.input_arbitration,
        ui_definition::UiViewportInputArbitrationPolicyDefinition::UiOwnsStatusBeforeViewportFallback
    );
    assert_eq!(
        region.viewport_surface_widget_id,
        surface_widget_id(viewport_surface, crate::VIEWPORT_SURFACE_EMBED_WIDGET_ID)
    );
    assert!(region.metrics.iter().any(|metric| {
        metric.kind == ui_definition::UiViewportStatusMetricKindDefinition::FrameRate
            && metric.priority == ui_definition::UiViewportStatusMetricPriorityDefinition::Essential
    }));
    assert!(region.metrics.iter().any(|metric| {
        metric.kind == ui_definition::UiViewportStatusMetricKindDefinition::FrameTime
            && metric.priority == ui_definition::UiViewportStatusMetricPriorityDefinition::Essential
    }));
    assert_scroll_owner(interaction_model, status_widget_id, Axis::Horizontal);
    assert_viewport_popup_interaction(
        interaction_model,
        viewport_surface,
        crate::VIEWPORT_OPTIONS_POPUP_WIDGET_ID,
        crate::VIEWPORT_OPTIONS_BUTTON_WIDGET_ID,
        crate::VIEWPORT_OPTIONS_POPUP_SCROLL_WIDGET_ID,
        Axis::Vertical,
    );
    assert_viewport_popup_interaction(
        interaction_model,
        viewport_surface,
        crate::VIEWPORT_TOOLS_MENU_WIDGET_ID,
        crate::VIEWPORT_TOOL_RADIAL_BUTTON_WIDGET_ID,
        crate::VIEWPORT_TOOLS_MENU_SCROLL_WIDGET_ID,
        Axis::Vertical,
    );
}

#[test]
fn frame_model_surfaces_are_artifact_lookup_not_layout_authority() {
    let workspace = sample_workspace_state();
    let (_, outliner_surface) = panel_and_surface_by_kind(&workspace, PanelKind::Outliner);
    let (_, viewport_surface) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let frame_model = frame_model_with_only_surface(&workspace, viewport_surface);

    assert!(frame_model.surface(viewport_surface).is_some());
    assert!(frame_model.surface(outliner_surface).is_none());

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);
    assert!(
        build
            .projection_artifacts
            .workspace
            .widget_context_by_id
            .values()
            .any(|context| context.active_tool_surface == Some(outliner_surface)),
        "workspace projection still owns mounted surface layout even when the frame lookup lacks an artifact"
    );
}

#[test]
fn shell_frame_renders_dynamic_split_workspace_after_area_split() {
    let workspace = sample_workspace_state();
    let (viewport_panel, _) = panel_and_surface_by_kind(&workspace, PanelKind::Viewport);
    let viewport_stack = tab_stack_by_panel(&workspace, viewport_panel);
    let mut allocator = WorkspaceIdentityAllocator::from_seed(workspace.next_identity_seed());
    let split_host_id = allocator.allocate_panel_host_id();
    let first_child_host_id = allocator.allocate_panel_host_id();
    let second_child_host_id = allocator.allocate_panel_host_id();
    let new_tab_stack_id = allocator.allocate_tab_stack_id();
    let new_panel_id = allocator.allocate_panel_instance_id();
    let new_surface_id = allocator.allocate_tool_surface_instance_id();

    let split_workspace = reduce_workspace(
        &workspace,
        WorkspaceMutation::split_tab_stack_area_legacy(
            viewport_stack,
            WorkspaceSplitAxis::Horizontal,
            split_host_id,
            first_child_host_id,
            second_child_host_id,
            new_tab_stack_id,
            new_panel_id,
            PanelKind::Inspector,
            new_surface_id,
            ToolSurfaceKind::Inspector,
            0.5,
        )
        .expect("legacy wrapper should create split mutation"),
    )
    .expect("split area should produce a valid workspace graph");
    let frame_model = frame_model_for_workspace(&split_workspace);

    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &split_workspace);

    assert!(
        build
            .projection_artifacts
            .workspace
            .tab_button_route_by_widget_id
            .values()
            .any(|route| route.tab_stack_id == new_tab_stack_id
                && route.panel_instance_id == new_panel_id),
        "dynamic projection should route tabs in newly split areas",
    );
    assert!(
        ui_tree_contains_widget(
            &build.tree.root,
            workspace_split_host_widget_id(split_host_id)
        ),
        "dynamic composition should render the newly inserted split host",
    );
}

fn frame_model_with_surface_route(
    workspace: &WorkspaceState,
    routed_surface: crate::ToolSurfaceInstanceId,
    widget_id: WidgetId,
    action: SurfaceLocalAction,
) -> EditorShellFrameModel {
    let mut frame_model = frame_model_for_workspace(workspace);
    let frame = frame_model
        .surfaces
        .get_mut(&routed_surface)
        .expect("routed surface should exist in frame model");
    frame
        .routes
        .insert(widget_id, SurfaceLocalRoute::new(action));
    frame.artifact.root = label(
        widget_id,
        frame.title.clone(),
        ThemeTokens::default().body_small_text_style(ui_text::FontId(1)),
    );
    frame_model
}

fn mapped_surface_actions_for_route(
    panel_kind: PanelKind,
    widget_id: WidgetId,
    action: SurfaceLocalAction,
    interactions: Vec<UiInteraction>,
) -> Vec<SurfaceLocalAction> {
    let workspace = sample_workspace_state();
    let (_, surface_id) = panel_and_surface_by_kind(&workspace, panel_kind);
    let frame_model = frame_model_with_surface_route(&workspace, surface_id, widget_id, action);
    let build = build_editor_shell_frame(&frame_model, &ThemeTokens::default(), &workspace);

    map_interactions_to_shell_commands(
        &UiInteractionResults {
            items: interactions,
        },
        &build.projection_artifacts,
    )
    .into_iter()
    .map(|command| match command {
        ShellCommand::DispatchSurfaceLocalAction { action, .. } => action,
        other => panic!("expected surface dispatch command, got {other:?}"),
    })
    .collect()
}

fn keyboard_event(key: Key) -> KeyboardEvent {
    KeyboardEvent {
        key,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
    }
}

fn frame_model_with_only_surface(
    workspace: &WorkspaceState,
    surface_id: crate::ToolSurfaceInstanceId,
) -> EditorShellFrameModel {
    let mut frame_model = EditorShellFrameModel::new(ToolbarViewModel::default(), BTreeMap::new());
    let panel = workspace
        .panels()
        .find(|panel| panel.active_tool_surface == Some(surface_id))
        .expect("surface should be mounted");
    let tab_stack_id = workspace
        .tab_stacks()
        .find(|stack| stack.ordered_panels.contains(&panel.id))
        .map(|stack| stack.id)
        .expect("mounted panel should belong to a tab stack");
    let surface = workspace
        .tool_surface(surface_id)
        .expect("surface should exist");
    frame_model.surfaces.insert(
        surface_id,
        surface_frame(
            panel.id,
            tab_stack_id,
            surface,
            WidgetId(surface_id.raw() + 10_000),
        ),
    );
    frame_model
}

fn frame_model_for_workspace(workspace: &WorkspaceState) -> EditorShellFrameModel {
    let mut surfaces = BTreeMap::new();
    for panel in workspace.panels() {
        let Some(surface_id) = panel.active_tool_surface else {
            continue;
        };
        let Some(surface) = workspace.tool_surface(surface_id) else {
            continue;
        };
        let Some(tab_stack_id) = workspace
            .tab_stacks()
            .find(|stack| stack.ordered_panels.contains(&panel.id))
            .map(|stack| stack.id)
        else {
            continue;
        };
        surfaces.insert(
            surface_id,
            surface_frame(
                panel.id,
                tab_stack_id,
                surface,
                WidgetId(surface_id.raw() + 10_000),
            ),
        );
    }
    EditorShellFrameModel::new(ToolbarViewModel::default(), surfaces)
}

fn surface_frame(
    panel_instance_id: PanelInstanceId,
    tab_stack_id: crate::TabStackId,
    surface: &crate::ToolSurfaceState,
    root_widget_id: WidgetId,
) -> ResolvedSurfaceFrame {
    let legacy_tool_surface_kind = surface
        .legacy_tool_surface_kind_or_error()
        .expect("test shell frame fixture should retain legacy compatibility metadata");
    ResolvedSurfaceFrame {
        surface_instance_id: surface.id,
        panel_instance_id,
        tab_stack_id,
        surface_kind: Some(legacy_tool_surface_kind),
        stable_surface_key: surface.stable_surface_key().clone(),
        surface_definition_id: tool_surface_definition_id(legacy_tool_surface_kind),
        provider_id: Some(SurfaceProviderId::try_from_raw(77).unwrap()),
        title: format!("{:?}", legacy_tool_surface_kind),
        artifact: SurfacePresentationArtifact::provider(label(
            root_widget_id,
            "surface",
            ThemeTokens::default().body_small_text_style(ui_text::FontId(1)),
        )),
        routes: SurfaceRouteTable::empty(),
        availability: SurfaceProviderAvailability::Available,
    }
}

fn sample_workspace_state() -> WorkspaceState {
    let mut allocator = WorkspaceIdentityAllocator::new();
    let workspace_id = allocator.allocate_workspace_id();
    WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator)
}

fn panel_and_surface_by_kind(
    workspace: &WorkspaceState,
    panel_kind: PanelKind,
) -> (PanelInstanceId, crate::ToolSurfaceInstanceId) {
    workspace
        .panels()
        .find(|panel| panel.panel_kind == panel_kind)
        .and_then(|panel| {
            panel
                .active_tool_surface
                .map(|surface_id| (panel.id, surface_id))
        })
        .expect("expected mounted surface for panel kind")
}

fn tab_stack_by_panel(workspace: &WorkspaceState, panel_id: PanelInstanceId) -> crate::TabStackId {
    workspace
        .tab_stacks()
        .find(|stack| stack.ordered_panels.contains(&panel_id))
        .map(|stack| stack.id)
        .expect("panel should belong to a tab stack")
}

fn assert_chrome_slot(
    model: &ui_definition::FormedInteractionModel,
    host_widget_id: WidgetId,
    slot_widget_id: WidgetId,
    kind: ui_definition::UiChromeSlotKindDefinition,
) {
    assert!(
        model.chrome_slots.iter().any(|slot| {
            slot.host_widget_id == host_widget_id
                && slot.slot_widget_id == slot_widget_id
                && slot.kind == kind
        }),
        "expected chrome slot {kind:?} for host {host_widget_id:?} and slot {slot_widget_id:?}; slots: {:?}",
        model.chrome_slots,
    );
}

fn assert_horizontal_slot_order(
    layouts: &ui_runtime::ComputedLayoutMap,
    close_widget_id: WidgetId,
    label_widget_id: WidgetId,
    active_indicator_widget_id: WidgetId,
) {
    let close = layouts
        .get(&close_widget_id)
        .expect("close slot layout should exist")
        .bounds;
    let label = layouts
        .get(&label_widget_id)
        .expect("label slot layout should exist")
        .bounds;
    let active_indicator = layouts
        .get(&active_indicator_widget_id)
        .expect("active indicator slot layout should exist")
        .bounds;

    assert!(
        close.x + close.width <= label.x,
        "close slot should not overlap label slot: close={close:?}, label={label:?}",
    );
    assert!(
        label.x + label.width <= active_indicator.x,
        "label slot should not overlap active indicator slot: label={label:?}, active={active_indicator:?}",
    );
}

fn assert_dock_drop_zone(
    model: &ui_definition::FormedInteractionModel,
    zone_widget_id: WidgetId,
    kind: ui_definition::UiDockDropZoneKindDefinition,
    state: ui_definition::UiDockDropZoneStateDefinition,
    priority: u16,
) {
    assert!(
        model.dock_drop_zones.iter().any(|zone| {
            zone.zone_widget_id == zone_widget_id
                && zone.kind == kind
                && zone.state == state
                && zone.priority == priority
        }),
        "expected dock/drop zone {kind:?} {state:?} priority {priority} for {zone_widget_id:?}; zones: {:?}",
        model.dock_drop_zones,
    );
}

fn assert_scroll_owner(
    model: &ui_definition::FormedInteractionModel,
    widget_id: WidgetId,
    axis: Axis,
) {
    assert!(
        model.scroll_owners.iter().any(|owner| {
            owner.widget_id == widget_id
                && owner.axes.contains(&axis)
                && owner.boundary
                    == ui_definition::UiScrollBoundaryPolicyDefinition::ConsumeAtBoundary
        }),
        "expected scroll owner for {widget_id:?} on {axis:?}; owners: {:?}",
        model.scroll_owners,
    );
}

fn assert_viewport_popup_interaction(
    model: &ui_definition::FormedInteractionModel,
    surface_id: crate::ToolSurfaceInstanceId,
    popup_widget_id: WidgetId,
    anchor_widget_id: WidgetId,
    scroll_widget_id: WidgetId,
    axis: Axis,
) {
    let popup_widget_id = surface_widget_id(surface_id, popup_widget_id);
    let anchor_widget_id = surface_widget_id(surface_id, anchor_widget_id);
    assert!(
        model.menu_scopes.iter().any(|scope| {
            scope.popup_widget_id == popup_widget_id
                && scope.anchor_widget_id == anchor_widget_id
                && scope.focus_return == Some(anchor_widget_id)
        }),
        "expected viewport popup scope for {popup_widget_id:?}; scopes: {:?}",
        model.menu_scopes,
    );
    assert!(
        model
            .menu_sizing
            .iter()
            .any(|sizing| sizing.popup_widget_id == popup_widget_id),
        "expected viewport popup sizing for {popup_widget_id:?}; sizing: {:?}",
        model.menu_sizing,
    );
    assert_scroll_owner(model, surface_widget_id(surface_id, scroll_widget_id), axis);
}

fn ui_tree_contains_widget(node: &crate::UiNode, widget_id: WidgetId) -> bool {
    node.id == widget_id
        || node
            .children
            .iter()
            .any(|child| ui_tree_contains_widget(child, widget_id))
}

fn button_enabled(node: &crate::UiNode, widget_id: WidgetId) -> bool {
    if node.id == widget_id {
        if let crate::UiNodeKind::Button(button) = &node.kind {
            return button.enabled;
        }
        return false;
    }
    node.children
        .iter()
        .any(|child| button_enabled(child, widget_id))
}
