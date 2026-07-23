use editor_shell::{
    ActiveTabDragVisualState, ActiveTabStackPopupMenu, BODY_CONSOLE_SPLIT_WIDGET_ID,
    CENTER_RIGHT_SPLIT_WIDGET_ID, DockDropCandidate, DockDropCandidateState, DockSplitSide,
    DockingInteractionVisualState, DockingPreviewDropTarget, EditorCompositionIdentityAllocator,
    EditorCompositionProjectionArtifact, EditorCompositionRuntime, EditorDockingIntent,
    EditorStructuralEditPlan, EditorWindowId, EditorWindowRegistry, LEFT_RIGHT_SPLIT_WIDGET_ID,
    MODELLING_WORKSPACE_PROFILE_ID, PanelHostId, PanelInstanceId, PreparedEditorCompositionCommit,
    ProfileRef, RegionCompassViewModel, SCENE_WORKSPACE_PROFILE_ID, ShellProjectionArtifacts,
    TabStackId, TabStackPopupMenuKind, ToolSurfaceInstanceId, ToolSurfaceRegistry, ToolbarMenuKind,
    UiRuntime, UiTree, WidgetId, WorkspaceId, WorkspaceIdentityAllocator, WorkspaceProfileId,
    WorkspaceProfileRegistry, WorkspaceProfileRegistryBackedBuildError, WorkspaceSplitAxis,
    WorkspaceState, import_legacy_workspace, project_editor_composition,
};
#[cfg(test)]
use editor_shell::{WorkspaceMutation, reduce_workspace};
use engine::plugins::render::backend::RenderSurfaceId;
use engine::runtime::NativeWindowId;
use std::collections::BTreeMap;
use ui_composition::{
    CompositionPolicies, MountedUnitId, PresentationTargetId, RegionId, SplitFraction,
    StateRevision,
};
use ui_math::{UiPoint, UiRect};

use crate::shell::{
    ActiveEditorDefinitionCatalogs, EditorCompositionTargetBindingRegistry,
    SelfAuthoringWorkspaceState, load_checked_in_editor_ui_definitions,
};

const TAB_DRAG_THRESHOLD_PX: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct TabDragSession {
    panel_instance_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    pointer_down: UiPoint,
    projection_epoch: u64,
    active: bool,
    drop_candidate_cycle_index: usize,
    drop_candidate_cycle_side: Option<DockSplitSide>,
}

#[derive(Debug, Clone, Default)]
struct TargetInteractionState {
    tab_drag_session: Option<TabDragSession>,
    split_resize_session: Option<SplitResizeSession>,
    corner_split_resize_session: Option<CornerSplitResizeSession>,
    corner_area_split_session: Option<CornerAreaSplitSession>,
    docking_visual_state: DockingInteractionVisualState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSplitKind {
    BodyConsole,
    LeftRight,
    CenterRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SplitResizeSession {
    split_region_id: RegionId,
    split_host_id: PanelHostId,
    split_widget_id: WidgetId,
    axis: WorkspaceSplitAxis,
    source_revision: StateRevision,
    preview_fraction: SplitFraction,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerSplitResizeSession {
    pub horizontal_split_host_id: PanelHostId,
    pub horizontal_split_widget_id: WidgetId,
    pub vertical_split_host_id: PanelHostId,
    pub vertical_split_widget_id: WidgetId,
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerAreaSplitSession {
    pub tab_stack_id: TabStackId,
    pub pointer_down: UiPoint,
    pub projection_epoch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorWindowPresentationBinding {
    pub native_window_id: NativeWindowId,
    pub render_surface_id: RenderSurfaceId,
}

impl EditorWindowPresentationBinding {
    pub fn primary() -> Self {
        Self {
            native_window_id: NativeWindowId::primary(),
            render_surface_id: RenderSurfaceId::primary(),
        }
    }
}

#[derive(Debug)]
pub struct RunenwerkEditorShellState {
    target_runtimes: BTreeMap<PresentationTargetId, UiRuntime>,
    last_tree: Option<UiTree>,
    last_tree_by_target: BTreeMap<PresentationTargetId, UiTree>,
    last_bounds: Option<UiRect>,
    last_bounds_by_target: BTreeMap<PresentationTargetId, UiRect>,
    last_projection_artifacts: Option<ShellProjectionArtifacts>,
    last_projection_artifacts_by_target: BTreeMap<PresentationTargetId, ShellProjectionArtifacts>,
    projection_epoch: u64,
    identity_allocator: WorkspaceIdentityAllocator,
    active_workspace_profile_id: WorkspaceProfileId,
    open_workspace_profile_ids: Vec<WorkspaceProfileId>,
    editor_windows: EditorWindowRegistry,
    editor_window_bindings: BTreeMap<EditorWindowId, EditorWindowPresentationBinding>,
    composition_target_bindings:
        EditorCompositionTargetBindingRegistry<EditorWindowPresentationBinding>,
    pending_editor_window_presentations: Vec<EditorWindowId>,
    active_toolbar_menu: Option<ToolbarMenuKind>,
    active_tab_stack_popup_menu: Option<ActiveTabStackPopupMenu>,
    workspace_id: WorkspaceId,
    composition_runtime: EditorCompositionRuntime,
    composition_identity_allocator: EditorCompositionIdentityAllocator,
    pending_docking_intents: Vec<EditorDockingIntent>,
    composition_coordination_pending: bool,
    composition_projection: EditorCompositionProjectionArtifact,
    self_authoring: SelfAuthoringWorkspaceState,
    active_editor_definitions: ActiveEditorDefinitionCatalogs,
    interaction_by_target: BTreeMap<PresentationTargetId, TargetInteractionState>,
    #[cfg(test)]
    legacy_workspace_snapshot: WorkspaceState,
}

fn reconcile_composition_target_bindings(
    runtime: &EditorCompositionRuntime,
    existing: &EditorCompositionTargetBindingRegistry<EditorWindowPresentationBinding>,
    primary_binding: EditorWindowPresentationBinding,
    created_binding: Option<(PresentationTargetId, EditorWindowPresentationBinding)>,
) -> Result<
    EditorCompositionTargetBindingRegistry<EditorWindowPresentationBinding>,
    editor_shell::EditorCompositionRejection,
> {
    let Some(primary_target) = runtime.composition().definition().targets().first() else {
        return Err(target_binding_rejection(
            "editor composition requires at least one presentation target",
        ));
    };
    let mut bindings = EditorCompositionTargetBindingRegistry::default();
    for target in runtime.composition().definition().targets() {
        let binding = created_binding
            .filter(|(target_id, _)| *target_id == target.id)
            .map(|(_, binding)| binding)
            .or_else(|| existing.binding(target.id).copied())
            .or_else(|| (target.id == primary_target.id).then_some(primary_binding))
            .ok_or_else(|| {
                target_binding_rejection(
                    "every non-primary composition target requires an editor window binding",
                )
            })?;
        bindings.bind(target.id, binding);
    }
    Ok(bindings)
}

fn target_binding_rejection(message: &'static str) -> editor_shell::EditorCompositionRejection {
    editor_shell::EditorCompositionRejection::single(
        editor_shell::EditorCompositionDiagnosticRecord::error(
            editor_shell::EditorCompositionDiagnosticCode::TargetBindingMismatch,
            editor_shell::EditorCompositionDiagnosticStage::Projection,
            editor_shell::EditorCompositionDiagnosticSubject::General(
                "editor-static-target-binding".to_owned(),
            ),
            message,
        ),
    )
}

fn require_same_history_targets(
    runtime: &EditorCompositionRuntime,
    expected: &[PresentationTargetId],
    operation: &'static str,
) -> Result<(), editor_shell::EditorCompositionRejection> {
    let actual = runtime
        .composition()
        .definition()
        .targets()
        .iter()
        .map(|target| target.id)
        .collect::<Vec<_>>();
    if actual == expected {
        return Ok(());
    }
    Err(editor_shell::EditorCompositionRejection::single(
        editor_shell::EditorCompositionDiagnosticRecord::error(
            editor_shell::EditorCompositionDiagnosticCode::HistoryTargetCoordinationRequired,
            editor_shell::EditorCompositionDiagnosticStage::Policy,
            editor_shell::EditorCompositionDiagnosticSubject::General(format!(
                "composition-history-{operation}"
            )),
            "Use the window-coordination history path when undo or redo changes presentation targets.",
        ),
    ))
}

impl Default for RunenwerkEditorShellState {
    fn default() -> Self {
        Self::new()
    }
}

impl RunenwerkEditorShellState {
    pub fn new() -> Self {
        let host = crate::shell::RunenwerkWorkbenchHost::new()
            .expect("default workbench host composition must build");
        Self::new_with_workspace_profile_registry_and_tool_surface_registry(
            host.workspace_profile_registry(),
            host.tool_surface_registry(),
        )
        .expect("default workspace should build from the default workbench host registry")
    }

    pub fn new_with_tool_surface_registry(
        registry: &ToolSurfaceRegistry,
    ) -> Result<Self, WorkspaceProfileRegistryBackedBuildError> {
        let host = crate::shell::RunenwerkWorkbenchHost::new()
            .expect("default workbench host composition must build");
        Self::new_with_workspace_profile_registry_and_tool_surface_registry(
            host.workspace_profile_registry(),
            registry,
        )
    }

    pub fn new_with_workspace_profile_registry_and_tool_surface_registry(
        profile_registry: &WorkspaceProfileRegistry,
        registry: &ToolSurfaceRegistry,
    ) -> Result<Self, WorkspaceProfileRegistryBackedBuildError> {
        let mut identity_allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = identity_allocator.allocate_workspace_id();
        let active_workspace_profile_id = profile_registry.default_profile_id();
        let workspace_state = profile_registry
            .default_profile()
            .expect("default workspace profile should exist")
            .build_default_workspace_state_with_registry(
                workspace_id,
                &mut identity_allocator,
                registry,
            )?;
        debug_assert!(workspace_state.validate_integrity().is_ok());
        Self::from_bootstrapped_workspace(
            identity_allocator,
            active_workspace_profile_id,
            workspace_state,
        )
    }

    pub fn new_for_workspace_profile_with_tool_surface_registry(
        profile_id: WorkspaceProfileId,
        registry: &ToolSurfaceRegistry,
    ) -> Result<Self, WorkspaceProfileRegistryBackedBuildError> {
        let host = crate::shell::RunenwerkWorkbenchHost::new()
            .expect("default workbench host composition must build");
        Self::new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
            profile_id,
            host.workspace_profile_registry(),
            registry,
        )
    }

    pub fn new_for_workspace_profile_with_workspace_profile_registry_and_tool_surface_registry(
        profile_id: WorkspaceProfileId,
        profile_registry: &WorkspaceProfileRegistry,
        registry: &ToolSurfaceRegistry,
    ) -> Result<Self, WorkspaceProfileRegistryBackedBuildError> {
        let mut identity_allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = identity_allocator.allocate_workspace_id();
        let profile = profile_registry.profile(profile_id).ok_or(
            WorkspaceProfileRegistryBackedBuildError::UnknownWorkspaceProfile { profile_id },
        )?;
        let workspace_state = profile.build_default_workspace_state_with_registry(
            workspace_id,
            &mut identity_allocator,
            registry,
        )?;
        debug_assert!(workspace_state.validate_integrity().is_ok());
        Self::from_bootstrapped_workspace_with_open_profiles(
            identity_allocator,
            profile_id,
            workspace_state,
            vec![profile_id],
        )
    }

    fn from_bootstrapped_workspace(
        identity_allocator: WorkspaceIdentityAllocator,
        active_workspace_profile_id: WorkspaceProfileId,
        workspace_state: WorkspaceState,
    ) -> Result<Self, WorkspaceProfileRegistryBackedBuildError> {
        Self::from_bootstrapped_workspace_with_open_profiles(
            identity_allocator,
            active_workspace_profile_id,
            workspace_state,
            vec![SCENE_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID],
        )
    }

    fn from_bootstrapped_workspace_with_open_profiles(
        mut identity_allocator: WorkspaceIdentityAllocator,
        active_workspace_profile_id: WorkspaceProfileId,
        workspace_state: WorkspaceState,
        open_workspace_profile_ids: Vec<WorkspaceProfileId>,
    ) -> Result<Self, WorkspaceProfileRegistryBackedBuildError> {
        let mut active_editor_definitions = ActiveEditorDefinitionCatalogs::default();
        let checked_in_definitions = load_checked_in_editor_ui_definitions()
            .expect("checked-in editor UI definitions should load");
        for template in checked_in_definitions.templates.into_values() {
            active_editor_definitions.install_template(template);
        }
        active_editor_definitions
            .install_editor_bindings(checked_in_definitions.bindings)
            .expect("checked-in editor bindings should activate");

        let workspace_id = workspace_state.workspace_id();
        let composition_runtime =
            import_legacy_workspace(active_workspace_profile_id, &workspace_state).map_err(
                |error| WorkspaceProfileRegistryBackedBuildError::CompositionImport {
                    profile_id: active_workspace_profile_id,
                    error: Box::new(error),
                },
            )?;
        let composition_projection =
            project_editor_composition(&composition_runtime).map_err(|error| {
                WorkspaceProfileRegistryBackedBuildError::CompositionImport {
                    profile_id: active_workspace_profile_id,
                    error: Box::new(error),
                }
            })?;
        let composition_identity_allocator =
            EditorCompositionIdentityAllocator::from_runtime(&composition_runtime);

        let primary_editor_window_id = identity_allocator.allocate_editor_window_id();
        let editor_windows = EditorWindowRegistry::new(primary_editor_window_id, workspace_id);
        let editor_window_bindings = BTreeMap::from([(
            primary_editor_window_id,
            EditorWindowPresentationBinding::primary(),
        )]);
        let composition_target_bindings = reconcile_composition_target_bindings(
            &composition_runtime,
            &EditorCompositionTargetBindingRegistry::default(),
            EditorWindowPresentationBinding::primary(),
            None,
        )
        .expect("built-in editor composition must have one bindable presentation target");
        let target_runtimes = composition_runtime
            .composition()
            .definition()
            .targets()
            .iter()
            .map(|target| (target.id, UiRuntime::new()))
            .collect();
        let interaction_by_target = composition_runtime
            .composition()
            .definition()
            .targets()
            .iter()
            .map(|target| (target.id, TargetInteractionState::default()))
            .collect();

        Ok(Self {
            target_runtimes,
            last_tree: None,
            last_tree_by_target: BTreeMap::new(),
            last_bounds: None,
            last_bounds_by_target: BTreeMap::new(),
            last_projection_artifacts: None,
            last_projection_artifacts_by_target: BTreeMap::new(),
            projection_epoch: 0,
            identity_allocator,
            active_workspace_profile_id,
            open_workspace_profile_ids,
            editor_windows,
            editor_window_bindings,
            composition_target_bindings,
            pending_editor_window_presentations: Vec::new(),
            active_toolbar_menu: None,
            active_tab_stack_popup_menu: None,
            workspace_id,
            composition_runtime,
            composition_identity_allocator,
            pending_docking_intents: Vec::new(),
            composition_coordination_pending: false,
            composition_projection,
            self_authoring: SelfAuthoringWorkspaceState::from_checked_in_fixtures()
                .expect("checked-in self-authoring fixtures should load"),
            active_editor_definitions,
            interaction_by_target,
            #[cfg(test)]
            legacy_workspace_snapshot: workspace_state,
        })
    }

    pub fn runtime(&self) -> &UiRuntime {
        self.runtime_for_target(self.primary_composition_target_id())
            .expect("ratified composition targets always own a UI runtime")
    }

    pub fn runtime_mut(&mut self) -> &mut UiRuntime {
        self.runtime_for_target_mut(self.primary_composition_target_id())
    }

    pub fn primary_composition_target_id(&self) -> PresentationTargetId {
        self.composition_runtime
            .composition()
            .definition()
            .targets()
            .first()
            .expect("ratified editor composition requires a presentation target")
            .id
    }

    fn interaction_state(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<&TargetInteractionState> {
        self.interaction_by_target.get(&target_id)
    }

    fn interaction_state_mut(
        &mut self,
        target_id: PresentationTargetId,
    ) -> &mut TargetInteractionState {
        self.interaction_by_target.entry(target_id).or_default()
    }

    pub fn runtime_for_target(&self, target_id: PresentationTargetId) -> Option<&UiRuntime> {
        self.target_runtimes.get(&target_id)
    }

    pub fn runtime_for_target_mut(&mut self, target_id: PresentationTargetId) -> &mut UiRuntime {
        self.target_runtimes.entry(target_id).or_default()
    }

    pub fn last_tree(&self) -> Option<&UiTree> {
        self.last_tree.as_ref()
    }

    pub fn set_last_tree(&mut self, tree: UiTree) {
        self.last_tree = Some(tree);
    }

    pub fn last_tree_for_target(&self, target_id: PresentationTargetId) -> Option<&UiTree> {
        self.last_tree_by_target.get(&target_id)
    }

    pub fn set_last_tree_for_target(&mut self, target_id: PresentationTargetId, tree: UiTree) {
        self.last_tree_by_target.insert(target_id, tree);
    }

    pub fn last_projection_artifacts(&self) -> Option<&ShellProjectionArtifacts> {
        self.last_projection_artifacts.as_ref()
    }

    pub fn set_last_projection_artifacts(&mut self, artifacts: ShellProjectionArtifacts) {
        let _ = self.cache_projection_artifacts(artifacts);
    }

    pub fn cache_projection_artifacts(
        &mut self,
        mut artifacts: ShellProjectionArtifacts,
    ) -> ShellProjectionArtifacts {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        artifacts.projection_epoch = self.projection_epoch;
        self.last_projection_artifacts = Some(artifacts.clone());
        artifacts
    }

    pub fn cache_projection_artifacts_for_target(
        &mut self,
        target_id: PresentationTargetId,
        mut artifacts: ShellProjectionArtifacts,
    ) -> ShellProjectionArtifacts {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        artifacts.projection_epoch = self.projection_epoch;
        self.last_projection_artifacts_by_target
            .insert(target_id, artifacts.clone());
        artifacts
    }

    pub fn last_projection_artifacts_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<&ShellProjectionArtifacts> {
        self.last_projection_artifacts_by_target.get(&target_id)
    }

    pub fn last_bounds(&self) -> Option<UiRect> {
        self.last_bounds
    }

    pub fn set_last_bounds(&mut self, bounds: UiRect) {
        self.last_bounds = Some(bounds);
    }

    pub fn last_bounds_for_target(&self, target_id: PresentationTargetId) -> Option<UiRect> {
        self.last_bounds_by_target.get(&target_id).copied()
    }

    pub fn set_last_bounds_for_target(&mut self, target_id: PresentationTargetId, bounds: UiRect) {
        self.last_bounds_by_target.insert(target_id, bounds);
    }

    pub fn workspace_id(&self) -> WorkspaceId {
        self.workspace_id
    }

    #[cfg(test)]
    pub(crate) fn workspace_state(&self) -> &WorkspaceState {
        &self.legacy_workspace_snapshot
    }

    #[cfg(test)]
    pub(crate) fn replace_workspace_state(&mut self, workspace: WorkspaceState) {
        let runtime = import_legacy_workspace(self.active_workspace_profile_id, &workspace)
            .expect("test workspace replacement should import as editor composition");
        self.install_composition_runtime(runtime)
            .expect("test workspace replacement should project as editor composition");
        self.legacy_workspace_snapshot = workspace;
    }

    #[cfg(test)]
    pub(crate) fn apply_workspace_mutation(
        &mut self,
        op: WorkspaceMutation,
    ) -> Result<(), editor_shell::WorkspaceStateError> {
        let workspace = reduce_workspace(&self.legacy_workspace_snapshot, op)?;
        self.replace_workspace_state(workspace);
        Ok(())
    }

    pub fn composition_runtime(&self) -> &EditorCompositionRuntime {
        &self.composition_runtime
    }

    pub fn composition_identity_allocator(&self) -> EditorCompositionIdentityAllocator {
        self.composition_identity_allocator
    }

    pub fn replace_composition_identity_allocator(
        &mut self,
        allocator: EditorCompositionIdentityAllocator,
    ) {
        self.composition_identity_allocator = allocator;
    }

    pub fn composition_projection(&self) -> &EditorCompositionProjectionArtifact {
        &self.composition_projection
    }

    pub fn mounted_unit_id_for_tool_surface(
        &self,
        tool_surface_id: ToolSurfaceInstanceId,
    ) -> Option<MountedUnitId> {
        self.composition_runtime
            .extension()
            .mounted_units()
            .iter()
            .find(|record| record.compatibility_surface_raw == tool_surface_id.raw())
            .map(|record| record.mounted_unit_id)
    }

    pub fn tool_surface_id_for_mounted_unit(
        &self,
        mounted_unit_id: MountedUnitId,
    ) -> Option<ToolSurfaceInstanceId> {
        self.composition_runtime
            .extension()
            .mounted_unit(mounted_unit_id)
            .and_then(|record| {
                ToolSurfaceInstanceId::try_from_raw(record.compatibility_surface_raw).ok()
            })
    }

    pub fn mounted_unit_id_for_panel(
        &self,
        panel_instance_id: PanelInstanceId,
    ) -> Option<MountedUnitId> {
        self.composition_runtime
            .extension()
            .mounted_units()
            .iter()
            .find(|record| record.panel_instance_raw == panel_instance_id.raw())
            .map(|record| record.mounted_unit_id)
    }

    pub fn structural_command_target_for_mounted_unit(
        &self,
        mounted_unit_id: MountedUnitId,
    ) -> Option<editor_shell::StructuralCommandTarget> {
        let unit = self
            .composition_runtime
            .extension()
            .mounted_unit(mounted_unit_id)?;
        let region = self
            .composition_runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.kind.mounted_units().contains(&mounted_unit_id))?;
        let region_extension = self.composition_runtime.extension().region(region.id)?;
        Some(editor_shell::StructuralCommandTarget {
            mounted_unit_id: Some(mounted_unit_id),
            panel_instance_id: PanelInstanceId::try_from_raw(unit.panel_instance_raw).ok()?,
            active_tool_surface: ToolSurfaceInstanceId::try_from_raw(
                unit.compatibility_surface_raw,
            )
            .ok(),
            tab_stack_id: TabStackId::try_from_raw(region_extension.tab_stack_raw?).ok()?,
        })
    }

    pub fn region_id_for_tab_stack(
        &self,
        tab_stack_id: TabStackId,
    ) -> Option<ui_composition::RegionId> {
        self.composition_runtime
            .extension()
            .regions()
            .iter()
            .find(|record| record.tab_stack_raw == Some(tab_stack_id.raw()))
            .map(|record| record.region_id)
    }

    pub fn tab_stack_id_for_region(
        &self,
        region_id: ui_composition::RegionId,
    ) -> Option<TabStackId> {
        self.composition_runtime
            .extension()
            .region(region_id)
            .and_then(|record| record.tab_stack_raw)
            .and_then(|raw| TabStackId::try_from_raw(raw).ok())
    }

    pub fn stack_region_for_host(&self, host_id: PanelHostId) -> Option<ui_composition::RegionId> {
        let region = self
            .composition_runtime
            .extension()
            .regions()
            .iter()
            .find(|record| record.compatibility_host_raw == host_id.raw())
            .map(|record| record.region_id)?;
        self.first_stack_region(region)
    }

    pub fn region_id_for_host(&self, host_id: PanelHostId) -> Option<RegionId> {
        self.composition_runtime
            .extension()
            .regions()
            .iter()
            .find(|record| record.compatibility_host_raw == host_id.raw())
            .map(|record| record.region_id)
    }

    pub fn primary_stack_region(&self) -> Option<ui_composition::RegionId> {
        let root = self
            .composition_runtime
            .composition()
            .definition()
            .roots()
            .iter()
            .find(|root| root.primary)?;
        self.first_stack_region(root.region)
    }

    fn first_stack_region(
        &self,
        region_id: ui_composition::RegionId,
    ) -> Option<ui_composition::RegionId> {
        let region = self
            .composition_runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.id == region_id)?;
        match &region.kind {
            ui_composition::RegionKind::Stack { .. } => Some(region_id),
            ui_composition::RegionKind::Split { first, second, .. } => self
                .first_stack_region(*first)
                .or_else(|| self.first_stack_region(*second)),
            ui_composition::RegionKind::Overlay {
                base,
                ordered_overlays,
            } => self.first_stack_region(*base).or_else(|| {
                ordered_overlays
                    .iter()
                    .find_map(|region| self.first_stack_region(*region))
            }),
            ui_composition::RegionKind::MountPoint { .. } => None,
        }
    }

    pub fn queue_docking_intent(&mut self, intent: EditorDockingIntent) {
        self.pending_docking_intents.push(intent);
        self.composition_coordination_pending = true;
    }

    pub fn drain_docking_intents(&mut self) -> Vec<EditorDockingIntent> {
        std::mem::take(&mut self.pending_docking_intents)
    }

    pub fn composition_coordination_pending(&self) -> bool {
        self.composition_coordination_pending
    }

    pub fn set_composition_coordination_pending(&mut self, pending: bool) {
        self.composition_coordination_pending = pending;
    }

    pub fn active_workspace_profile_id(&self) -> WorkspaceProfileId {
        self.active_workspace_profile_id
    }

    pub fn open_workspace_profile_ids(&self) -> &[WorkspaceProfileId] {
        &self.open_workspace_profile_ids
    }

    pub fn editor_windows(&self) -> &EditorWindowRegistry {
        &self.editor_windows
    }

    pub fn editor_window_binding(
        &self,
        editor_window_id: EditorWindowId,
    ) -> Option<EditorWindowPresentationBinding> {
        self.editor_window_bindings.get(&editor_window_id).copied()
    }

    pub fn editor_window_for_binding(
        &self,
        binding: EditorWindowPresentationBinding,
    ) -> Option<EditorWindowId> {
        self.editor_window_bindings
            .iter()
            .find_map(|(window_id, candidate)| (*candidate == binding).then_some(*window_id))
    }

    pub fn remove_editor_window_presentation(&mut self, editor_window_id: EditorWindowId) -> bool {
        if self
            .editor_windows
            .remove_window(editor_window_id)
            .is_none()
        {
            return false;
        }
        self.editor_window_bindings.remove(&editor_window_id);
        true
    }

    pub fn composition_target_binding(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<EditorWindowPresentationBinding> {
        self.composition_target_bindings.binding(target_id).copied()
    }

    pub fn composition_target_bindings(
        &self,
    ) -> impl Iterator<
        Item = crate::shell::EditorCompositionTargetBinding<EditorWindowPresentationBinding>,
    > {
        self.composition_target_bindings.iter().map(|entry| {
            crate::shell::EditorCompositionTargetBinding {
                target_id: entry.target_id,
                binding: *entry.binding,
            }
        })
    }

    pub fn bind_editor_window_presentation(
        &mut self,
        editor_window_id: EditorWindowId,
        binding: EditorWindowPresentationBinding,
    ) -> bool {
        if self.editor_windows.record(editor_window_id).is_none() {
            return false;
        }
        self.editor_window_bindings
            .insert(editor_window_id, binding);
        if editor_window_id == self.editor_windows.primary_window_id() {
            let target_ids = self
                .composition_target_bindings
                .iter()
                .map(|entry| entry.target_id)
                .collect::<Vec<_>>();
            for target_id in target_ids {
                self.composition_target_bindings.bind(target_id, binding);
            }
        }
        true
    }

    pub fn bind_composition_target_presentation(
        &mut self,
        target_id: PresentationTargetId,
        editor_window_id: EditorWindowId,
        binding: EditorWindowPresentationBinding,
    ) -> bool {
        if self.editor_windows.record(editor_window_id).is_none()
            || self
                .composition_runtime
                .composition()
                .definition()
                .targets()
                .iter()
                .all(|target| target.id != target_id)
        {
            return false;
        }
        self.editor_window_bindings
            .insert(editor_window_id, binding);
        self.composition_target_bindings.bind(target_id, binding);
        true
    }

    pub fn drain_pending_editor_window_presentations(&mut self) -> Vec<EditorWindowId> {
        std::mem::take(&mut self.pending_editor_window_presentations)
    }

    pub fn open_editor_window_for_active_workspace(&mut self) -> EditorWindowId {
        let editor_window_id = self.identity_allocator.allocate_editor_window_id();
        self.editor_windows
            .open_secondary_window(editor_window_id, self.workspace_id);
        self.pending_editor_window_presentations
            .push(editor_window_id);
        editor_window_id
    }

    pub fn active_toolbar_menu(&self) -> Option<ToolbarMenuKind> {
        self.active_toolbar_menu
    }

    pub fn toggle_toolbar_menu(&mut self, menu: ToolbarMenuKind) {
        self.active_toolbar_menu = if self.active_toolbar_menu == Some(menu) {
            None
        } else {
            Some(menu)
        };
        self.active_tab_stack_popup_menu = None;
        self.clear_cached_projection();
    }

    pub fn active_tab_stack_popup_menu(&self) -> Option<ActiveTabStackPopupMenu> {
        self.active_tab_stack_popup_menu.clone()
    }

    pub fn active_tab_stack_action_menu(&self) -> Option<TabStackId> {
        self.active_tab_stack_popup_menu
            .as_ref()
            .filter(|menu| menu.kind == TabStackPopupMenuKind::AreaActions)
            .map(|menu| menu.tab_stack_id)
    }

    pub fn open_tab_stack_popup_menu(
        &mut self,
        kind: TabStackPopupMenuKind,
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    ) {
        self.active_tab_stack_popup_menu = Some(ActiveTabStackPopupMenu {
            kind,
            tab_stack_id,
            anchor_widget_id,
        });
        self.active_toolbar_menu = None;
        self.clear_cached_projection();
    }

    pub fn toggle_tab_stack_popup_menu(
        &mut self,
        kind: TabStackPopupMenuKind,
        tab_stack_id: TabStackId,
        anchor_widget_id: WidgetId,
    ) {
        let next = ActiveTabStackPopupMenu {
            kind,
            tab_stack_id,
            anchor_widget_id,
        };
        self.active_tab_stack_popup_menu = if self.active_tab_stack_popup_menu == Some(next.clone())
        {
            None
        } else {
            Some(next)
        };
        self.active_toolbar_menu = None;
        self.clear_cached_projection();
    }

    pub fn close_tab_stack_popup_menu(&mut self) {
        if self.active_tab_stack_popup_menu.is_some() {
            self.active_tab_stack_popup_menu = None;
            self.clear_cached_projection();
        }
    }

    pub fn close_tab_stack_action_menu(&mut self) {
        self.close_tab_stack_popup_menu();
    }

    pub fn close_toolbar_menu(&mut self) {
        if self.active_toolbar_menu.is_some() {
            self.active_toolbar_menu = None;
            self.clear_cached_projection();
        }
    }

    pub fn set_active_workspace_profile_id(&mut self, profile_id: WorkspaceProfileId) {
        if !self.open_workspace_profile_ids.contains(&profile_id) {
            self.open_workspace_profile_ids.push(profile_id);
        }
        if self.active_workspace_profile_id == profile_id {
            return;
        }
        self.active_workspace_profile_id = profile_id;
        self.active_toolbar_menu = None;
        self.active_tab_stack_popup_menu = None;
        self.clear_cached_projection();
    }

    pub fn activate_workspace_profile_ref_with_registry(
        &mut self,
        profile_ref: &ProfileRef,
        profile_registry: &WorkspaceProfileRegistry,
        registry: &ToolSurfaceRegistry,
    ) -> Result<WorkspaceProfileId, WorkspaceProfileRegistryBackedBuildError> {
        let profile = profile_registry.profile_by_ref(profile_ref).ok_or(
            WorkspaceProfileRegistryBackedBuildError::UnknownWorkspaceProfile {
                profile_id: profile_registry.default_profile_id(),
            },
        )?;
        let mut allocator =
            WorkspaceIdentityAllocator::from_seed(self.identity_allocator.seed_snapshot());
        let workspace_id = allocator.allocate_workspace_id();
        let workspace_state = profile.build_default_workspace_state_with_registry(
            workspace_id,
            &mut allocator,
            registry,
        )?;
        let mut seed = workspace_state.next_identity_seed();
        seed.next_editor_window_id = self
            .identity_allocator
            .seed_snapshot()
            .next_editor_window_id;
        let composition_runtime =
            import_legacy_workspace(profile.id, &workspace_state).map_err(|error| {
                WorkspaceProfileRegistryBackedBuildError::CompositionImport {
                    profile_id: profile.id,
                    error: Box::new(error),
                }
            })?;
        let composition_projection =
            project_editor_composition(&composition_runtime).map_err(|error| {
                WorkspaceProfileRegistryBackedBuildError::CompositionImport {
                    profile_id: profile.id,
                    error: Box::new(error),
                }
            })?;
        self.identity_allocator = WorkspaceIdentityAllocator::from_seed(seed);
        self.workspace_id = workspace_id;
        self.composition_runtime = composition_runtime;
        self.composition_identity_allocator =
            EditorCompositionIdentityAllocator::from_runtime(&self.composition_runtime);
        self.composition_projection = composition_projection;
        #[cfg(test)]
        {
            self.legacy_workspace_snapshot = workspace_state;
        }
        self.active_workspace_profile_id = profile.id;
        if !self.open_workspace_profile_ids.contains(&profile.id) {
            self.open_workspace_profile_ids.push(profile.id);
        }
        self.active_toolbar_menu = None;
        self.active_tab_stack_popup_menu = None;
        self.clear_cached_projection();
        Ok(profile.id)
    }

    pub fn close_workspace_profile_id(
        &mut self,
        profile_id: WorkspaceProfileId,
    ) -> Option<WorkspaceProfileId> {
        let close_index = self
            .open_workspace_profile_ids
            .iter()
            .position(|open_profile_id| *open_profile_id == profile_id)?;
        if self.open_workspace_profile_ids.len() <= 1 {
            return None;
        }

        let active_before_close = self.active_workspace_profile_id;
        let fallback_profile_id = if active_before_close == profile_id {
            let fallback_index = if close_index + 1 < self.open_workspace_profile_ids.len() {
                close_index + 1
            } else {
                close_index.saturating_sub(1)
            };
            self.open_workspace_profile_ids[fallback_index]
        } else {
            active_before_close
        };

        self.open_workspace_profile_ids.remove(close_index);
        if active_before_close == profile_id {
            self.active_workspace_profile_id = fallback_profile_id;
            self.active_toolbar_menu = None;
            self.active_tab_stack_popup_menu = None;
        }
        self.clear_cached_projection();
        Some(fallback_profile_id)
    }

    pub fn self_authoring(&self) -> &SelfAuthoringWorkspaceState {
        &self.self_authoring
    }

    pub fn self_authoring_mut(&mut self) -> &mut SelfAuthoringWorkspaceState {
        &mut self.self_authoring
    }

    pub fn active_editor_definitions(&self) -> &ActiveEditorDefinitionCatalogs {
        &self.active_editor_definitions
    }

    pub fn active_editor_definitions_mut(&mut self) -> &mut ActiveEditorDefinitionCatalogs {
        &mut self.active_editor_definitions
    }

    pub fn install_composition_runtime(
        &mut self,
        runtime: EditorCompositionRuntime,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        let projection = project_editor_composition(&runtime)?;
        let primary_binding = self
            .editor_window_binding(self.editor_windows.primary_window_id())
            .ok_or_else(|| {
                target_binding_rejection("primary editor window has no presentation binding")
            })?;
        let composition_target_bindings = reconcile_composition_target_bindings(
            &runtime,
            &self.composition_target_bindings,
            primary_binding,
            None,
        )?;
        let profile_id = WorkspaceProfileId::try_from_raw(
            runtime.extension().workspace_profile_raw(),
        )
        .map_err(|_| {
            editor_shell::EditorCompositionRejection::single(
                editor_shell::EditorCompositionDiagnosticRecord::error(
                    editor_shell::EditorCompositionDiagnosticCode::ExtensionCoreMismatch,
                    editor_shell::EditorCompositionDiagnosticStage::Extension,
                    editor_shell::EditorCompositionDiagnosticSubject::Profile(
                        runtime.extension().workspace_profile_raw().to_string(),
                    ),
                    "Use a valid non-zero editor workspace profile compatibility identity.",
                ),
            )
        })?;
        self.composition_runtime = runtime;
        self.composition_identity_allocator =
            EditorCompositionIdentityAllocator::from_runtime(&self.composition_runtime);
        self.composition_projection = projection;
        self.composition_target_bindings = composition_target_bindings;
        self.reconcile_target_runtimes();
        self.active_workspace_profile_id = profile_id;
        self.clear_split_resize();
        self.clear_cached_projection();
        Ok(())
    }

    pub fn commit_prepared_composition(
        &mut self,
        prepared: PreparedEditorCompositionCommit,
        created_binding: Option<(PresentationTargetId, EditorWindowPresentationBinding)>,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        let primary_binding = self
            .editor_window_binding(self.editor_windows.primary_window_id())
            .ok_or_else(|| {
                target_binding_rejection("primary editor window has no presentation binding")
            })?;
        let next_bindings = reconcile_composition_target_bindings(
            prepared.candidate_runtime(),
            &self.composition_target_bindings,
            primary_binding,
            created_binding,
        )?;
        let projection = self.composition_runtime.commit_prepared(prepared)?;
        self.composition_projection = projection;
        self.composition_target_bindings = next_bindings;
        self.reconcile_target_runtimes();
        self.clear_split_resize();
        self.clear_cached_projection();
        Ok(())
    }

    pub fn apply_structural_edit_plan(
        &mut self,
        plan: EditorStructuralEditPlan,
        policies: CompositionPolicies<'_>,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        let prepared = self
            .composition_runtime
            .prepare_change(plan.change, policies)?;
        self.commit_prepared_composition(prepared, None)?;
        self.composition_identity_allocator = plan.identities;
        Ok(())
    }

    pub fn undo_structural_composition(
        &mut self,
        policies: CompositionPolicies<'_>,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        let mut allocator = self.composition_identity_allocator;
        let transaction_id = allocator.allocate_transaction()?;
        let current_targets = self
            .composition_runtime
            .composition()
            .definition()
            .targets()
            .iter()
            .map(|target| target.id)
            .collect::<Vec<_>>();
        let mut candidate = self.composition_runtime.clone();
        candidate.undo_structural(transaction_id, policies)?;
        require_same_history_targets(&candidate, &current_targets, "undo")?;
        self.install_composition_runtime(candidate)
    }

    pub fn redo_structural_composition(
        &mut self,
        policies: CompositionPolicies<'_>,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        let mut allocator = self.composition_identity_allocator;
        let transaction_id = allocator.allocate_transaction()?;
        let current_targets = self
            .composition_runtime
            .composition()
            .definition()
            .targets()
            .iter()
            .map(|target| target.id)
            .collect::<Vec<_>>();
        let mut candidate = self.composition_runtime.clone();
        candidate.redo_structural(transaction_id, policies)?;
        require_same_history_targets(&candidate, &current_targets, "redo")?;
        self.install_composition_runtime(candidate)
    }

    fn reconcile_target_runtimes(&mut self) {
        let target_ids = self
            .composition_runtime
            .composition()
            .definition()
            .targets()
            .iter()
            .map(|target| target.id)
            .collect::<Vec<_>>();
        self.target_runtimes
            .retain(|target_id, _| target_ids.contains(target_id));
        self.interaction_by_target
            .retain(|target_id, _| target_ids.contains(target_id));
        for target_id in target_ids {
            self.target_runtimes.entry(target_id).or_default();
            self.interaction_by_target.entry(target_id).or_default();
        }
    }

    pub fn allocate_panel_host_id(&mut self) -> PanelHostId {
        self.identity_allocator.allocate_panel_host_id()
    }

    pub fn allocate_tab_stack_id(&mut self) -> TabStackId {
        self.identity_allocator.allocate_tab_stack_id()
    }

    pub fn allocate_panel_instance_id(&mut self) -> PanelInstanceId {
        self.identity_allocator.allocate_panel_instance_id()
    }

    pub fn allocate_tool_surface_instance_id(&mut self) -> ToolSurfaceInstanceId {
        self.identity_allocator.allocate_tool_surface_instance_id()
    }

    pub fn identity_allocator(&self) -> &WorkspaceIdentityAllocator {
        &self.identity_allocator
    }

    pub fn current_projection_epoch(&self) -> u64 {
        self.projection_epoch
    }

    pub fn is_projection_epoch_current(&self, projection_epoch: u64) -> bool {
        projection_epoch == self.projection_epoch
    }

    pub fn docking_visual_state(&self) -> DockingInteractionVisualState {
        self.docking_visual_state_for_target(self.primary_composition_target_id())
    }

    pub fn docking_visual_state_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> DockingInteractionVisualState {
        self.interaction_by_target
            .get(&target_id)
            .map(|state| state.docking_visual_state.clone())
            .unwrap_or_default()
    }

    pub fn begin_tab_drag_candidate(
        &mut self,
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        pointer_down: UiPoint,
        projection_epoch: u64,
    ) {
        self.begin_tab_drag_candidate_for_target(
            self.primary_composition_target_id(),
            panel_instance_id,
            source_tab_stack_id,
            pointer_down,
            projection_epoch,
        );
    }

    pub fn begin_tab_drag_candidate_for_target(
        &mut self,
        target_id: PresentationTargetId,
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        pointer_down: UiPoint,
        projection_epoch: u64,
    ) {
        let state = self.interaction_state_mut(target_id);
        state.tab_drag_session = Some(TabDragSession {
            panel_instance_id,
            source_tab_stack_id,
            pointer_down,
            projection_epoch,
            active: false,
            drop_candidate_cycle_index: 0,
            drop_candidate_cycle_side: None,
        });
        state.docking_visual_state.active_tab_drag = None;
    }

    pub fn update_tab_drag_pointer(
        &mut self,
        pointer: UiPoint,
        current_projection_epoch: u64,
    ) -> bool {
        self.update_tab_drag_pointer_for_target(
            self.primary_composition_target_id(),
            pointer,
            current_projection_epoch,
        )
    }

    pub fn update_tab_drag_pointer_for_target(
        &mut self,
        target_id: PresentationTargetId,
        pointer: UiPoint,
        current_projection_epoch: u64,
    ) -> bool {
        let state = self.interaction_state_mut(target_id);
        let Some(mut session) = state.tab_drag_session else {
            return false;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if session.active {
            state.tab_drag_session = Some(session);
            return true;
        }

        let delta_x = pointer.x - session.pointer_down.x;
        let delta_y = pointer.y - session.pointer_down.y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        if distance_squared < TAB_DRAG_THRESHOLD_PX * TAB_DRAG_THRESHOLD_PX {
            return false;
        }

        session.active = true;
        state.tab_drag_session = Some(session);
        state.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id: session.panel_instance_id,
            source_tab_stack_id: session.source_tab_stack_id,
            preview_target: None,
            preview_candidates: Vec::new(),
            region_compass_anchor: None,
            region_compass: None,
        });
        true
    }

    pub fn set_tab_drag_preview(
        &mut self,
        target: Option<DockingPreviewDropTarget>,
        candidates: Vec<DockDropCandidate>,
        active_side: Option<DockSplitSide>,
        current_projection_epoch: u64,
    ) {
        self.set_tab_drag_preview_for_target(
            self.primary_composition_target_id(),
            target,
            candidates,
            active_side,
            current_projection_epoch,
        );
    }

    pub fn set_tab_drag_preview_for_target(
        &mut self,
        target_id: PresentationTargetId,
        target: Option<DockingPreviewDropTarget>,
        candidates: Vec<DockDropCandidate>,
        active_side: Option<DockSplitSide>,
        current_projection_epoch: u64,
    ) {
        let state = self.interaction_state_mut(target_id);
        let Some(mut session) = state.tab_drag_session else {
            return;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if session.drop_candidate_cycle_side != active_side {
            session.drop_candidate_cycle_side = active_side;
        }
        if !session.active {
            state.tab_drag_session = Some(session);
            return;
        }
        state.tab_drag_session = Some(session);
        let previous = state.docking_visual_state.active_tab_drag.as_ref();
        let region_compass_anchor = previous.and_then(|drag| drag.region_compass_anchor);
        let region_compass = previous.and_then(|drag| drag.region_compass.clone());
        state.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id: session.panel_instance_id,
            source_tab_stack_id: session.source_tab_stack_id,
            preview_target: target,
            preview_candidates: candidates,
            region_compass_anchor,
            region_compass,
        });
    }

    pub fn set_region_compass_for_target(
        &mut self,
        target_id: PresentationTargetId,
        anchor: WidgetId,
        compass: RegionCompassViewModel,
        current_projection_epoch: u64,
    ) {
        let state = self.interaction_state_mut(target_id);
        let Some(mut session) = state.tab_drag_session else {
            return;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if !session.active {
            state.tab_drag_session = Some(session);
            return;
        }
        state.tab_drag_session = Some(session);
        if let Some(drag) = state.docking_visual_state.active_tab_drag.as_mut() {
            drag.region_compass_anchor = Some(anchor);
            drag.region_compass = Some(compass);
        }
    }

    pub fn clear_region_compass_for_target(&mut self, target_id: PresentationTargetId) {
        if let Some(drag) = self
            .interaction_by_target
            .get_mut(&target_id)
            .and_then(|state| state.docking_visual_state.active_tab_drag.as_mut())
        {
            drag.region_compass_anchor = None;
            drag.region_compass = None;
            drag.preview_target = None;
            drag.preview_candidates.clear();
        }
    }

    pub fn focus_region_compass_detach_for_target(
        &mut self,
        target_id: PresentationTargetId,
    ) -> bool {
        let Some(compass) = self
            .interaction_by_target
            .get_mut(&target_id)
            .and_then(|state| state.docking_visual_state.active_tab_drag.as_mut())
            .and_then(|drag| drag.region_compass.as_mut())
        else {
            return false;
        };
        compass.focus_detach();
        true
    }

    pub fn region_compass_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<&RegionCompassViewModel> {
        self.interaction_by_target
            .get(&target_id)
            .and_then(|state| state.docking_visual_state.active_tab_drag.as_ref())
            .and_then(|drag| drag.region_compass.as_ref())
    }

    pub fn finish_region_compass_for_target(
        &mut self,
        target_id: PresentationTargetId,
        current_projection_epoch: u64,
    ) -> Option<EditorDockingIntent> {
        let state = self.interaction_by_target.get(&target_id)?;
        let session = state.tab_drag_session?;
        let compass = state
            .docking_visual_state
            .active_tab_drag
            .as_ref()?
            .region_compass
            .clone()?;
        self.clear_tab_drag_for_target(target_id);
        if !session.active || session.projection_epoch != current_projection_epoch {
            return None;
        }
        let source_revision = self.composition_runtime.composition().revision();
        if compass.session == editor_shell::RegionCompassSessionState::DetachFocused {
            return Some(EditorDockingIntent::detach_to_new_target(
                source_revision,
                compass.unit,
            ));
        }
        let zone = compass.focused_zone()?;
        Some(EditorDockingIntent {
            source_revision,
            unit: compass.unit,
            destination: editor_shell::EditorDockingDestination::Region {
                target_region: compass.region,
                ordinal: compass.ordinal,
                zone,
            },
        })
    }

    pub fn tab_drag_drop_candidate_cycle(&self) -> (usize, Option<DockSplitSide>) {
        self.tab_drag_drop_candidate_cycle_for_target(self.primary_composition_target_id())
    }

    pub fn tab_drag_drop_candidate_cycle_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> (usize, Option<DockSplitSide>) {
        self.interaction_by_target
            .get(&target_id)
            .and_then(|state| state.tab_drag_session)
            .map(|session| {
                (
                    session.drop_candidate_cycle_index,
                    session.drop_candidate_cycle_side,
                )
            })
            .unwrap_or((0, None))
    }

    pub fn cycle_active_tab_drag_preview_candidate(&mut self) -> bool {
        self.cycle_active_tab_drag_preview_candidate_for_target(
            self.primary_composition_target_id(),
        )
    }

    pub fn cycle_active_tab_drag_preview_candidate_for_target(
        &mut self,
        target_id: PresentationTargetId,
    ) -> bool {
        let state = self.interaction_state_mut(target_id);
        let Some(session) = state.tab_drag_session.as_mut() else {
            return false;
        };
        if !session.active {
            return false;
        }
        let Some(drag) = state.docking_visual_state.active_tab_drag.as_mut() else {
            return false;
        };
        if drag.preview_candidates.is_empty() {
            return false;
        }
        let active_side = drag
            .preview_candidates
            .iter()
            .find(|candidate| candidate.state.is_active())
            .map(|candidate| candidate.side)
            .or(session.drop_candidate_cycle_side)
            .or_else(|| {
                drag.preview_candidates
                    .iter()
                    .find(|candidate| candidate.state.is_selectable())
                    .map(|candidate| candidate.side)
            });
        let Some(active_side) = active_side else {
            return false;
        };
        let same_side_indices = drag
            .preview_candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| {
                (candidate.state.is_selectable() && candidate.side == active_side).then_some(index)
            })
            .collect::<Vec<_>>();
        if same_side_indices.is_empty() {
            return false;
        }
        session.drop_candidate_cycle_index = session.drop_candidate_cycle_index.saturating_add(1);
        session.drop_candidate_cycle_side = Some(active_side);
        let active_index =
            same_side_indices[session.drop_candidate_cycle_index % same_side_indices.len()];
        for (index, candidate) in drag.preview_candidates.iter_mut().enumerate() {
            if candidate.state.is_selectable() {
                candidate.state = DockDropCandidateState::selectable(index == active_index);
            }
        }
        drag.preview_target = Some(drag.preview_candidates[active_index].target);
        true
    }

    pub fn finish_tab_drag(
        &mut self,
        current_projection_epoch: u64,
    ) -> Option<(
        PanelInstanceId,
        TabStackId,
        Option<DockingPreviewDropTarget>,
        u64,
    )> {
        self.finish_tab_drag_for_target(
            self.primary_composition_target_id(),
            current_projection_epoch,
        )
    }

    pub fn finish_tab_drag_for_target(
        &mut self,
        target_id: PresentationTargetId,
        current_projection_epoch: u64,
    ) -> Option<(
        PanelInstanceId,
        TabStackId,
        Option<DockingPreviewDropTarget>,
        u64,
    )> {
        let state = self.interaction_by_target.get(&target_id)?;
        let session = state.tab_drag_session?;
        let preview_target = state
            .docking_visual_state
            .active_tab_drag
            .as_ref()
            .and_then(|drag| drag.preview_target);
        self.clear_tab_drag_for_target(target_id);
        if !session.active {
            return None;
        }
        Some((
            session.panel_instance_id,
            session.source_tab_stack_id,
            preview_target,
            current_projection_epoch,
        ))
    }

    pub fn tab_drag_candidate(&self) -> Option<(PanelInstanceId, TabStackId, u64, bool)> {
        self.tab_drag_candidate_for_target(self.primary_composition_target_id())
    }

    pub fn tab_drag_candidate_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<(PanelInstanceId, TabStackId, u64, bool)> {
        self.interaction_by_target
            .get(&target_id)
            .and_then(|state| state.tab_drag_session)
            .map(|session| {
                (
                    session.panel_instance_id,
                    session.source_tab_stack_id,
                    session.projection_epoch,
                    session.active,
                )
            })
    }

    pub fn clear_tab_drag(&mut self) {
        self.clear_tab_drag_for_target(self.primary_composition_target_id());
    }

    pub fn clear_tab_drag_for_target(&mut self, target_id: PresentationTargetId) {
        let state = self.interaction_state_mut(target_id);
        state.tab_drag_session = None;
        state.docking_visual_state.active_tab_drag = None;
    }

    pub fn clear_all_tab_drags(&mut self) {
        for state in self.interaction_by_target.values_mut() {
            state.tab_drag_session = None;
            state.docking_visual_state.active_tab_drag = None;
        }
    }

    pub fn begin_split_resize(&mut self, split_kind: WorkspaceSplitKind) {
        if let Some(split_host_id) = self.resolve_split_host_id(split_kind) {
            self.begin_workspace_split_resize(
                split_host_id,
                split_kind_widget_id(split_kind),
                split_kind_axis(split_kind),
            );
        }
    }

    pub fn begin_workspace_split_resize(
        &mut self,
        split_host_id: PanelHostId,
        split_widget_id: WidgetId,
        axis: WorkspaceSplitAxis,
    ) {
        self.begin_workspace_split_resize_for_target(
            self.primary_composition_target_id(),
            split_host_id,
            split_widget_id,
            axis,
        );
    }

    pub fn begin_workspace_split_resize_for_target(
        &mut self,
        target_id: PresentationTargetId,
        split_host_id: PanelHostId,
        split_widget_id: WidgetId,
        axis: WorkspaceSplitAxis,
    ) {
        let Some(split_region_id) = self.region_id_for_host(split_host_id) else {
            return;
        };
        let Some(initial_fraction) = self
            .composition_runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.id == split_region_id)
            .and_then(|region| match &region.kind {
                ui_composition::RegionKind::Split { fraction, .. } => Some(*fraction),
                _ => None,
            })
        else {
            return;
        };
        let source_revision = self.composition_runtime.composition().revision();
        let state = self.interaction_state_mut(target_id);
        state.corner_split_resize_session = None;
        state.split_resize_session = Some(SplitResizeSession {
            split_region_id,
            split_host_id,
            split_widget_id,
            axis,
            source_revision,
            preview_fraction: initial_fraction,
        });
        state.docking_visual_state.active_split_border_widget = Some(split_widget_id);
        state.docking_visual_state.active_split_preview_fraction = Some((
            split_widget_id,
            initial_fraction.basis_points() as f32 / 10_000.0,
        ));
    }

    pub fn update_split_resize_preview_for_target(
        &mut self,
        target_id: PresentationTargetId,
        fraction: SplitFraction,
    ) -> bool {
        let state = self.interaction_state_mut(target_id);
        let Some(session) = state.split_resize_session.as_mut() else {
            return false;
        };
        session.preview_fraction = fraction;
        state.docking_visual_state.active_split_preview_fraction = Some((
            session.split_widget_id,
            fraction.basis_points() as f32 / 10_000.0,
        ));
        true
    }

    pub fn finish_split_resize_for_target(
        &mut self,
        target_id: PresentationTargetId,
    ) -> Option<(RegionId, SplitFraction, StateRevision)> {
        let state = self.interaction_by_target.get_mut(&target_id)?;
        let session = state.split_resize_session.take()?;
        state.docking_visual_state.active_split_border_widget = None;
        state.docking_visual_state.active_split_preview_fraction = None;
        Some((
            session.split_region_id,
            session.preview_fraction,
            session.source_revision,
        ))
    }

    pub fn begin_workspace_corner_split_resize(&mut self, session: CornerSplitResizeSession) {
        let state = self.interaction_state_mut(self.primary_composition_target_id());
        state.split_resize_session = None;
        state.corner_area_split_session = None;
        state.corner_split_resize_session = Some(session);
        state.docking_visual_state.active_split_border_widget =
            Some(session.horizontal_split_widget_id);
    }

    pub fn begin_corner_area_split(&mut self, session: CornerAreaSplitSession) {
        let state = self.interaction_state_mut(self.primary_composition_target_id());
        state.split_resize_session = None;
        state.corner_split_resize_session = None;
        state.corner_area_split_session = Some(session);
    }

    pub fn active_corner_area_split_session(&self) -> Option<CornerAreaSplitSession> {
        self.interaction_state(self.primary_composition_target_id())
            .and_then(|state| state.corner_area_split_session)
    }

    pub fn clear_corner_area_split(&mut self) {
        self.interaction_state_mut(self.primary_composition_target_id())
            .corner_area_split_session = None;
    }

    pub fn active_split_resize_kind(&self) -> Option<WorkspaceSplitKind> {
        let session = self
            .interaction_state(self.primary_composition_target_id())?
            .split_resize_session?;
        [
            WorkspaceSplitKind::BodyConsole,
            WorkspaceSplitKind::LeftRight,
            WorkspaceSplitKind::CenterRight,
        ]
        .into_iter()
        .find(|kind| self.resolve_split_host_id(*kind) == Some(session.split_host_id))
    }

    pub fn active_split_resize_session(
        &self,
    ) -> Option<(PanelHostId, WidgetId, WorkspaceSplitAxis)> {
        self.active_split_resize_session_for_target(self.primary_composition_target_id())
    }

    pub fn active_split_resize_session_for_target(
        &self,
        target_id: PresentationTargetId,
    ) -> Option<(PanelHostId, WidgetId, WorkspaceSplitAxis)> {
        self.interaction_state(target_id)
            .and_then(|state| state.split_resize_session)
            .map(|session| (session.split_host_id, session.split_widget_id, session.axis))
    }

    pub fn active_corner_split_resize_session(&self) -> Option<CornerSplitResizeSession> {
        self.interaction_state(self.primary_composition_target_id())
            .and_then(|state| state.corner_split_resize_session)
    }

    pub fn clear_split_resize(&mut self) {
        for state in self.interaction_by_target.values_mut() {
            state.split_resize_session = None;
            state.corner_split_resize_session = None;
            state.corner_area_split_session = None;
            state.docking_visual_state.active_split_border_widget = None;
            state.docking_visual_state.active_split_preview_fraction = None;
        }
    }

    pub fn clear_split_resize_for_target(&mut self, target_id: PresentationTargetId) {
        if let Some(state) = self.interaction_by_target.get_mut(&target_id) {
            state.split_resize_session = None;
            state.corner_split_resize_session = None;
            state.corner_area_split_session = None;
            state.docking_visual_state.active_split_border_widget = None;
            state.docking_visual_state.active_split_preview_fraction = None;
        }
    }

    pub fn clear_cached_projection(&mut self) {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.last_tree = None;
        self.last_tree_by_target.clear();
        self.last_bounds = None;
        self.last_bounds_by_target.clear();
        self.last_projection_artifacts = None;
        self.last_projection_artifacts_by_target.clear();
        self.clear_all_tab_drags();
    }

    fn resolve_split_host_id(&self, split_kind: WorkspaceSplitKind) -> Option<PanelHostId> {
        let root = self
            .composition_projection
            .roots
            .iter()
            .find(|root| root.primary)?;
        let root_region = self
            .composition_runtime
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.id == root.region)?;
        let ui_composition::RegionKind::Split { first, .. } = &root_region.kind else {
            return None;
        };
        let host_for_region = |region_id| {
            self.composition_runtime
                .extension()
                .region(region_id)
                .and_then(|record| PanelHostId::try_from_raw(record.compatibility_host_raw).ok())
        };
        match split_kind {
            WorkspaceSplitKind::BodyConsole => host_for_region(root.region),
            WorkspaceSplitKind::LeftRight => host_for_region(*first),
            WorkspaceSplitKind::CenterRight => {
                let left_right = self
                    .composition_runtime
                    .composition()
                    .definition()
                    .regions()
                    .iter()
                    .find(|region| region.id == *first)?;
                let ui_composition::RegionKind::Split { second, .. } = &left_right.kind else {
                    return None;
                };
                host_for_region(*second)
            }
        }
    }
}

fn split_kind_widget_id(kind: WorkspaceSplitKind) -> WidgetId {
    match kind {
        WorkspaceSplitKind::BodyConsole => BODY_CONSOLE_SPLIT_WIDGET_ID,
        WorkspaceSplitKind::LeftRight => LEFT_RIGHT_SPLIT_WIDGET_ID,
        WorkspaceSplitKind::CenterRight => CENTER_RIGHT_SPLIT_WIDGET_ID,
    }
}

fn split_kind_axis(kind: WorkspaceSplitKind) -> WorkspaceSplitAxis {
    match kind {
        WorkspaceSplitKind::BodyConsole => WorkspaceSplitAxis::Vertical,
        WorkspaceSplitKind::LeftRight => WorkspaceSplitAxis::Horizontal,
        WorkspaceSplitKind::CenterRight => WorkspaceSplitAxis::Vertical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_compass_finish_emits_typed_revision_bound_docking_intent() {
        let mut shell_state = RunenwerkEditorShellState::new();
        let extension = shell_state
            .composition_runtime()
            .extension()
            .mounted_units()
            .first()
            .cloned()
            .expect("default composition should mount editor content");
        let unit = extension.mounted_unit_id;
        let panel = PanelInstanceId::try_from_raw(extension.panel_instance_raw).unwrap();
        let target_region = shell_state
            .composition_runtime()
            .composition()
            .definition()
            .regions()
            .iter()
            .find(|region| region.kind.mounted_units().contains(&unit))
            .map(|region| region.id)
            .unwrap();
        let stack = shell_state.tab_stack_id_for_region(target_region).unwrap();
        let target = shell_state.primary_composition_target_id();
        let epoch = shell_state.current_projection_epoch();
        shell_state.begin_tab_drag_candidate_for_target(
            target,
            panel,
            stack,
            UiPoint::new(10.0, 10.0),
            epoch,
        );
        assert!(shell_state.update_tab_drag_pointer_for_target(
            target,
            UiPoint::new(18.0, 10.0),
            epoch,
        ));
        shell_state.set_region_compass_for_target(
            target,
            WidgetId(91),
            RegionCompassViewModel::active(
                target,
                target_region,
                unit,
                ui_adaptive_composition::DockZone::Center,
                "content",
                "region",
                editor_shell::RegionCompassAccessibility::default(),
            )
            .with_ordinal(1),
            epoch,
        );

        let intent = shell_state
            .finish_region_compass_for_target(target, epoch)
            .expect("typed Region Compass intent");
        assert_eq!(intent.unit, unit);
        assert_eq!(
            intent.source_revision,
            shell_state.composition_runtime().composition().revision()
        );
        assert_eq!(
            intent.destination,
            editor_shell::EditorDockingDestination::Region {
                target_region,
                ordinal: 1,
                zone: ui_adaptive_composition::DockZone::Center,
            }
        );
    }

    #[test]
    fn tab_drag_candidate_cycling_skips_invalid_drop_candidates() {
        let mut shell_state = RunenwerkEditorShellState::new();
        let panel_instance_id = PanelInstanceId::try_from_raw(1).unwrap();
        let source_tab_stack_id = TabStackId::try_from_raw(2).unwrap();
        let active_target = DockingPreviewDropTarget::SplitIntoRoot {
            side: DockSplitSide::Left,
        };
        let next_target = DockingPreviewDropTarget::SplitIntoRoot {
            side: DockSplitSide::Right,
        };

        let target_id = shell_state.primary_composition_target_id();
        let interaction = shell_state.interaction_state_mut(target_id);
        interaction.tab_drag_session = Some(TabDragSession {
            panel_instance_id,
            source_tab_stack_id,
            pointer_down: UiPoint::new(10.0, 10.0),
            projection_epoch: 1,
            active: true,
            drop_candidate_cycle_index: 0,
            drop_candidate_cycle_side: Some(DockSplitSide::Left),
        });
        interaction.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id,
            source_tab_stack_id,
            preview_target: Some(active_target),
            preview_candidates: vec![
                DockDropCandidate {
                    target: DockingPreviewDropTarget::SplitIntoArea {
                        target_tab_stack_id: source_tab_stack_id,
                        side: DockSplitSide::Left,
                    },
                    scope: editor_shell::DockDropScope::Area,
                    side: DockSplitSide::Left,
                    anchor_widget_id: WidgetId(10),
                    state: DockDropCandidateState::Invalid {
                        reason: editor_shell::DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea,
                    },
                },
                DockDropCandidate {
                    target: active_target,
                    scope: editor_shell::DockDropScope::Workspace,
                    side: DockSplitSide::Left,
                    anchor_widget_id: WidgetId(11),
                    state: DockDropCandidateState::Active,
                },
                DockDropCandidate {
                    target: next_target,
                    scope: editor_shell::DockDropScope::Workspace,
                    side: DockSplitSide::Left,
                    anchor_widget_id: WidgetId(12),
                    state: DockDropCandidateState::Candidate,
                },
            ],
            region_compass_anchor: None,
            region_compass: None,
        });

        assert!(shell_state.cycle_active_tab_drag_preview_candidate());

        let visual = shell_state.docking_visual_state_for_target(target_id);
        let drag = visual
            .active_tab_drag
            .as_ref()
            .expect("tab drag visual state should remain active");
        assert_eq!(drag.preview_target, Some(next_target));
        assert!(matches!(
            drag.preview_candidates[0].state,
            DockDropCandidateState::Invalid {
                reason: editor_shell::DockDropInvalidTargetReason::SourceOnlyTabCannotSplitOwnArea
            }
        ));
        assert_eq!(
            drag.preview_candidates[1].state,
            DockDropCandidateState::Candidate
        );
        assert_eq!(
            drag.preview_candidates[2].state,
            DockDropCandidateState::Active
        );
    }

    #[test]
    fn tab_drag_sessions_are_isolated_by_presentation_target() {
        let mut shell_state = RunenwerkEditorShellState::new();
        let primary = shell_state.primary_composition_target_id();
        let secondary = PresentationTargetId::new(primary.raw() + 100);
        let panel = PanelInstanceId::try_from_raw(1).unwrap();
        let stack = TabStackId::try_from_raw(2).unwrap();

        shell_state.begin_tab_drag_candidate_for_target(
            primary,
            panel,
            stack,
            UiPoint::new(10.0, 10.0),
            1,
        );
        shell_state.begin_tab_drag_candidate_for_target(
            secondary,
            panel,
            stack,
            UiPoint::new(50.0, 50.0),
            1,
        );
        assert!(shell_state.update_tab_drag_pointer_for_target(
            primary,
            UiPoint::new(17.0, 10.0),
            1,
        ));

        assert!(
            shell_state
                .docking_visual_state_for_target(primary)
                .active_tab_drag
                .is_some()
        );
        assert!(
            shell_state
                .docking_visual_state_for_target(secondary)
                .active_tab_drag
                .is_none()
        );
        assert_eq!(
            shell_state.tab_drag_candidate_for_target(secondary),
            Some((panel, stack, 1, false))
        );
    }
}
