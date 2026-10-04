use editor_shell::{
    ActiveTabStackPopupMenu, EditorCompositionIdentityAllocator,
    EditorCompositionProjectionArtifact, EditorCompositionRuntime, EditorDockingIntent,
    EditorFreshTargetRequest, EditorStructuralEditPlan, EditorWindowId, EditorWindowRegistry,
    MODELLING_WORKSPACE_PROFILE_ID, PanelHostId, PanelInstanceId, PreparedEditorCompositionCommit,
    ProfileRef, SCENE_WORKSPACE_PROFILE_ID, ShellProjectionArtifacts, TabStackId,
    TabStackPopupMenuKind,
    ToolSurfaceInstanceId, ToolSurfaceRegistry, ToolbarMenuKind, UiRuntime, UiTree, WidgetId,
    WorkspaceId, WorkspaceIdentityAllocator, WorkspaceProfileId, WorkspaceProfileRegistry,
    WorkspaceProfileRegistryBackedBuildError,
    form_editor_profile_layout_source, form_editor_profile_layout_source_with_identities,
    project_editor_composition,
};
use engine::plugins::render::backend::RenderSurfaceId;
use engine::runtime::NativeWindowId;
use std::collections::BTreeMap;
use ui_composition::{CompositionPolicies, MountedUnitId, PresentationTargetId, RegionId};
use ui_math::UiRect;

use crate::shell::{
    ActiveEditorDefinitionCatalogs, EditorCompositionTargetBindingRegistry,
    SelfAuthoringWorkspaceState, load_checked_in_editor_ui_definitions,
};

mod interaction;

pub use interaction::{CornerAreaSplitSession, CornerSplitResizeSession, WorkspaceSplitKind};
use interaction::TargetInteractionState;

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
    pending_fresh_target_request: Option<EditorFreshTargetRequest>,
    pending_composition_restore: Option<EditorCompositionRuntime>,
    composition_coordination_pending: bool,
    composition_projection: EditorCompositionProjectionArtifact,
    self_authoring: SelfAuthoringWorkspaceState,
    active_editor_definitions: ActiveEditorDefinitionCatalogs,
    interaction_by_target: BTreeMap<PresentationTargetId, TargetInteractionState>,
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
        let profile = profile_registry
            .default_profile()
            .expect("default workspace profile should exist");
        profile.require_tool_surface_registry_compatibility(registry)?;
        let composition_runtime =
            form_editor_profile_layout_source(profile.id, &profile.layout_source, registry)
                .map_err(|error| {
                    WorkspaceProfileRegistryBackedBuildError::CompositionFormation {
                        profile_id: profile.id,
                        error: Box::new(error),
                    }
                })?;
        Self::from_bootstrapped_composition_with_open_profiles(
            identity_allocator,
            workspace_id,
            active_workspace_profile_id,
            composition_runtime,
            vec![SCENE_WORKSPACE_PROFILE_ID, MODELLING_WORKSPACE_PROFILE_ID],
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
        profile.require_tool_surface_registry_compatibility(registry)?;
        let composition_runtime =
            form_editor_profile_layout_source(profile.id, &profile.layout_source, registry)
                .map_err(|error| {
                    WorkspaceProfileRegistryBackedBuildError::CompositionFormation {
                        profile_id: profile.id,
                        error: Box::new(error),
                    }
                })?;
        Self::from_bootstrapped_composition_with_open_profiles(
            identity_allocator,
            workspace_id,
            profile_id,
            composition_runtime,
            vec![profile_id],
        )
    }

    fn from_bootstrapped_composition_with_open_profiles(
        mut identity_allocator: WorkspaceIdentityAllocator,
        workspace_id: WorkspaceId,
        active_workspace_profile_id: WorkspaceProfileId,
        composition_runtime: EditorCompositionRuntime,
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

        let composition_projection =
            project_editor_composition(&composition_runtime).map_err(|error| {
                WorkspaceProfileRegistryBackedBuildError::CompositionFormation {
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
            pending_fresh_target_request: None,
            pending_composition_restore: None,
            composition_coordination_pending: false,
            composition_projection,
            self_authoring: SelfAuthoringWorkspaceState::from_checked_in_fixtures()
                .expect("checked-in self-authoring fixtures should load"),
            active_editor_definitions,
            interaction_by_target,
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

    pub fn queue_composition_restore(
        &mut self,
        runtime: EditorCompositionRuntime,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        if self.composition_coordination_pending || self.pending_composition_restore.is_some() {
            return Err(editor_shell::EditorCompositionRejection::single(
                editor_shell::EditorCompositionDiagnosticRecord::error(
                    editor_shell::EditorCompositionDiagnosticCode::CoordinationPending,
                    editor_shell::EditorCompositionDiagnosticStage::Policy,
                    editor_shell::EditorCompositionDiagnosticSubject::General(
                        "composition-restore".to_owned(),
                    ),
                    "Wait for the pending composition transition to commit or roll back before loading another layout.",
                ),
            ));
        }
        project_editor_composition(&runtime)?;
        WorkspaceProfileId::try_from_raw(runtime.extension().workspace_profile_raw()).map_err(
            |_| {
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
            },
        )?;
        self.pending_composition_restore = Some(runtime);
        self.composition_coordination_pending = true;
        Ok(())
    }

    pub fn has_pending_composition_restore(&self) -> bool {
        self.pending_composition_restore.is_some()
    }

    pub fn take_pending_composition_restore(&mut self) -> Option<EditorCompositionRuntime> {
        self.pending_composition_restore.take()
    }

    pub fn queue_fresh_target_request(
        &mut self,
        request: EditorFreshTargetRequest,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        if self.composition_coordination_pending
            || self.pending_fresh_target_request.is_some()
            || self.pending_composition_restore.is_some()
        {
            return Err(editor_shell::EditorCompositionRejection::single(
                editor_shell::EditorCompositionDiagnosticRecord::error(
                    editor_shell::EditorCompositionDiagnosticCode::CoordinationPending,
                    editor_shell::EditorCompositionDiagnosticStage::Policy,
                    editor_shell::EditorCompositionDiagnosticSubject::General(
                        "fresh-presentation-target".to_owned(),
                    ),
                    "Wait for the pending composition transition to commit or roll back before opening another window.",
                ),
            ));
        }
        self.pending_fresh_target_request = Some(request);
        self.composition_coordination_pending = true;
        Ok(())
    }

    pub fn has_pending_fresh_target_request(&self) -> bool {
        self.pending_fresh_target_request.is_some()
    }

    pub fn take_pending_fresh_target_request(&mut self) -> Option<EditorFreshTargetRequest> {
        self.pending_fresh_target_request.take()
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
        self.pending_editor_window_presentations
            .retain(|pending| *pending != editor_window_id);
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
        profile.require_tool_surface_registry_compatibility(registry)?;
        let mut allocator =
            WorkspaceIdentityAllocator::from_seed(self.identity_allocator.seed_snapshot());
        let workspace_id = allocator.allocate_workspace_id();
        let seed = allocator.seed_snapshot();
        let primary_target_id = self.primary_composition_target_id();
        let (composition_runtime, composition_identities) =
            form_editor_profile_layout_source_with_identities(
                profile.id,
                &profile.layout_source,
                registry,
                primary_target_id,
                self.composition_identity_allocator,
            )
            .map_err(|error| {
                WorkspaceProfileRegistryBackedBuildError::CompositionFormation {
                    profile_id: profile.id,
                    error: Box::new(error),
                }
            })?;
        let composition_projection =
            project_editor_composition(&composition_runtime).map_err(|error| {
                WorkspaceProfileRegistryBackedBuildError::CompositionFormation {
                    profile_id: profile.id,
                    error: Box::new(error),
                }
            })?;
        self.identity_allocator = WorkspaceIdentityAllocator::from_seed(seed);
        self.workspace_id = workspace_id;
        self.composition_runtime = composition_runtime;
        self.composition_identity_allocator = composition_identities;
        self.composition_projection = composition_projection;
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
        self.install_composition_runtime_with_target_bindings(runtime, composition_target_bindings)
    }

    pub(crate) fn install_composition_runtime_with_target_bindings(
        &mut self,
        runtime: EditorCompositionRuntime,
        composition_target_bindings: EditorCompositionTargetBindingRegistry<
            EditorWindowPresentationBinding,
        >,
    ) -> Result<(), editor_shell::EditorCompositionRejection> {
        let projection = project_editor_composition(&runtime)?;
        let Some(primary_target) = runtime.composition().definition().targets().first() else {
            return Err(target_binding_rejection(
                "editor composition requires at least one presentation target",
            ));
        };
        let primary_binding = self
            .editor_window_binding(self.editor_windows.primary_window_id())
            .ok_or_else(|| {
                target_binding_rejection("primary editor window has no presentation binding")
            })?;
        if composition_target_bindings
            .binding(primary_target.id)
            .copied()
            != Some(primary_binding)
        {
            return Err(target_binding_rejection(
                "primary composition target must bind to the current primary editor presentation",
            ));
        }
        for target in runtime.composition().definition().targets() {
            let binding = composition_target_bindings
                .binding(target.id)
                .copied()
                .ok_or_else(|| {
                    target_binding_rejection(
                        "every composition target requires an explicit editor window binding",
                    )
                })?;
            if target.id != primary_target.id && self.editor_window_for_binding(binding).is_none() {
                return Err(target_binding_rejection(
                    "secondary composition target binding must reference a live editor window presentation",
                ));
            }
        }
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

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_activation_preserves_target_and_advances_composition_identity_space() {
        let host = crate::shell::RunenwerkWorkbenchHost::new()
            .expect("default workbench host composition must build");
        let mut shell_state =
            RunenwerkEditorShellState::new_with_workspace_profile_registry_and_tool_surface_registry(
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("default workspace profile should form");

        let primary_target = shell_state.primary_composition_target_id();
        let previous_max_unit = shell_state
            .composition_runtime()
            .composition()
            .definition()
            .mounted_units()
            .iter()
            .map(|unit| unit.id.raw())
            .max()
            .expect("default profile should mount content");
        let previous_max_panel = shell_state
            .composition_runtime()
            .extension()
            .mounted_units()
            .iter()
            .map(|unit| unit.panel_instance_raw)
            .max()
            .expect("default profile should expose panel compatibility identities");
        let previous_max_surface = shell_state
            .composition_runtime()
            .extension()
            .mounted_units()
            .iter()
            .map(|unit| unit.compatibility_surface_raw)
            .max()
            .expect("default profile should expose surface compatibility identities");

        let profile = host
            .workspace_profile_registry()
            .profile(editor_shell::MATERIAL_WORKSPACE_PROFILE_ID)
            .expect("material profile should be installed");
        shell_state
            .activate_workspace_profile_ref_with_registry(
                &profile.profile_ref,
                host.workspace_profile_registry(),
                host.tool_surface_registry(),
            )
            .expect("material profile should activate");

        assert_eq!(shell_state.primary_composition_target_id(), primary_target);
        assert!(
            shell_state
                .composition_runtime()
                .composition()
                .definition()
                .mounted_units()
                .iter()
                .all(|unit| unit.id.raw() > previous_max_unit)
        );
        assert!(
            shell_state
                .composition_runtime()
                .extension()
                .mounted_units()
                .iter()
                .all(|unit| unit.panel_instance_raw > previous_max_panel)
        );
        assert!(
            shell_state
                .composition_runtime()
                .extension()
                .mounted_units()
                .iter()
                .all(|unit| unit.compatibility_surface_raw > previous_max_surface)
        );
    }
}
