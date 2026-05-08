use editor_shell::{
    ActiveTabDragVisualState, ActiveTabStackPopupMenu, BODY_CONSOLE_SPLIT_WIDGET_ID,
    CENTER_RIGHT_SPLIT_WIDGET_ID, DockDropCandidate, DockSplitSide, DockingInteractionVisualState,
    DockingPreviewDropTarget, LEFT_RIGHT_SPLIT_WIDGET_ID, MODELLING_WORKSPACE_PROFILE_ID,
    PanelHostId, PanelInstanceId, SCENE_WORKSPACE_PROFILE_ID, ShellProjectionArtifacts, TabStackId,
    TabStackPopupMenuKind, ToolSurfaceInstanceId, ToolSurfaceKind, ToolbarMenuKind, UiRuntime,
    UiTree, WidgetId, WorkspaceId, WorkspaceIdentityAllocator, WorkspaceMutation,
    WorkspaceProfileId, WorkspaceSplitAxis, WorkspaceState, WorkspaceStateError,
    default_workspace_profile_registry, reduce_workspace,
};
use ui_math::{UiPoint, UiRect};

use crate::shell::{
    ActiveEditorDefinitionCatalogs, SelfAuthoringWorkspaceState,
    load_checked_in_editor_ui_definitions,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSplitKind {
    BodyConsole,
    LeftRight,
    CenterRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SplitResizeSession {
    split_host_id: PanelHostId,
    split_widget_id: WidgetId,
    axis: WorkspaceSplitAxis,
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

#[derive(Debug)]
pub struct RunenwerkEditorShellState {
    runtime: UiRuntime,
    last_tree: Option<UiTree>,
    last_bounds: Option<UiRect>,
    last_projection_artifacts: Option<ShellProjectionArtifacts>,
    projection_epoch: u64,
    identity_allocator: WorkspaceIdentityAllocator,
    active_workspace_profile_id: WorkspaceProfileId,
    open_workspace_profile_ids: Vec<WorkspaceProfileId>,
    active_toolbar_menu: Option<ToolbarMenuKind>,
    active_tab_stack_popup_menu: Option<ActiveTabStackPopupMenu>,
    workspace_state: WorkspaceState,
    self_authoring: SelfAuthoringWorkspaceState,
    active_editor_definitions: ActiveEditorDefinitionCatalogs,
    tab_drag_session: Option<TabDragSession>,
    split_resize_session: Option<SplitResizeSession>,
    corner_split_resize_session: Option<CornerSplitResizeSession>,
    corner_area_split_session: Option<CornerAreaSplitSession>,
    docking_visual_state: DockingInteractionVisualState,
}

impl Default for RunenwerkEditorShellState {
    fn default() -> Self {
        let mut identity_allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = identity_allocator.allocate_workspace_id();
        let profile_registry = default_workspace_profile_registry();
        let active_workspace_profile_id = profile_registry.default_profile_id();
        let workspace_state = profile_registry
            .default_profile()
            .expect("default workspace profile should exist")
            .build_default_workspace_state(workspace_id, &mut identity_allocator);
        debug_assert!(workspace_state.validate_integrity().is_ok());
        let mut active_editor_definitions = ActiveEditorDefinitionCatalogs::default();
        let checked_in_definitions = load_checked_in_editor_ui_definitions()
            .expect("checked-in editor UI definitions should load");
        for template in checked_in_definitions.templates.into_values() {
            active_editor_definitions.install_template(template);
        }
        active_editor_definitions
            .install_editor_bindings(checked_in_definitions.bindings)
            .expect("checked-in editor bindings should activate");

        Self {
            runtime: UiRuntime::new(),
            last_tree: None,
            last_bounds: None,
            last_projection_artifacts: None,
            projection_epoch: 0,
            identity_allocator,
            active_workspace_profile_id,
            open_workspace_profile_ids: vec![
                SCENE_WORKSPACE_PROFILE_ID,
                MODELLING_WORKSPACE_PROFILE_ID,
            ],
            active_toolbar_menu: None,
            active_tab_stack_popup_menu: None,
            workspace_state,
            self_authoring: SelfAuthoringWorkspaceState::from_checked_in_fixtures()
                .expect("checked-in self-authoring fixtures should load"),
            active_editor_definitions,
            tab_drag_session: None,
            split_resize_session: None,
            corner_split_resize_session: None,
            corner_area_split_session: None,
            docking_visual_state: DockingInteractionVisualState::default(),
        }
    }
}

impl RunenwerkEditorShellState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn runtime(&self) -> &UiRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut UiRuntime {
        &mut self.runtime
    }

    pub fn last_tree(&self) -> Option<&UiTree> {
        self.last_tree.as_ref()
    }

    pub fn set_last_tree(&mut self, tree: UiTree) {
        self.last_tree = Some(tree);
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

    pub fn last_bounds(&self) -> Option<UiRect> {
        self.last_bounds
    }

    pub fn set_last_bounds(&mut self, bounds: UiRect) {
        self.last_bounds = Some(bounds);
    }

    pub fn workspace_id(&self) -> WorkspaceId {
        self.workspace_state.workspace_id()
    }

    pub fn active_workspace_profile_id(&self) -> WorkspaceProfileId {
        self.active_workspace_profile_id
    }

    pub fn open_workspace_profile_ids(&self) -> &[WorkspaceProfileId] {
        &self.open_workspace_profile_ids
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

    pub fn workspace_state(&self) -> &WorkspaceState {
        &self.workspace_state
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

    pub fn replace_workspace_state(&mut self, workspace_state: WorkspaceState) {
        self.identity_allocator =
            WorkspaceIdentityAllocator::from_seed(workspace_state.next_identity_seed());
        self.workspace_state = workspace_state;
        self.clear_split_resize();
        self.clear_cached_projection();
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
        self.last_projection_artifacts
            .as_ref()
            .map(|artifacts| artifacts.projection_epoch == projection_epoch)
            .unwrap_or(false)
    }

    pub fn apply_workspace_mutation(
        &mut self,
        op: WorkspaceMutation,
    ) -> Result<(), WorkspaceStateError> {
        self.workspace_state = reduce_workspace(&self.workspace_state, op)?;
        self.clear_cached_projection();
        Ok(())
    }

    pub fn apply_workspace_mutation_with_allocations<T>(
        &mut self,
        build: impl FnOnce(&mut WorkspaceIdentityAllocator) -> (WorkspaceMutation, T),
    ) -> Result<T, WorkspaceStateError> {
        let mut allocator =
            WorkspaceIdentityAllocator::from_seed(self.identity_allocator.seed_snapshot());
        let (op, value) = build(&mut allocator);
        let next_workspace_state = reduce_workspace(&self.workspace_state, op)?;
        self.identity_allocator = allocator;
        self.workspace_state = next_workspace_state;
        self.clear_cached_projection();
        Ok(value)
    }

    pub fn switch_panel_tool_surface_kind(
        &mut self,
        panel_instance_id: PanelInstanceId,
        tool_surface_kind: ToolSurfaceKind,
    ) -> Result<ToolSurfaceInstanceId, WorkspaceStateError> {
        self.apply_workspace_mutation_with_allocations(|allocator| {
            let tool_surface_id = allocator.allocate_tool_surface_instance_id();
            (
                WorkspaceMutation::ReplacePanelToolSurfaceKind {
                    panel_id: panel_instance_id,
                    tool_surface_id,
                    tool_surface_kind,
                },
                tool_surface_id,
            )
        })
    }

    pub fn docking_visual_state(&self) -> DockingInteractionVisualState {
        self.docking_visual_state.clone()
    }

    pub fn begin_tab_drag_candidate(
        &mut self,
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
        pointer_down: UiPoint,
        projection_epoch: u64,
    ) {
        self.tab_drag_session = Some(TabDragSession {
            panel_instance_id,
            source_tab_stack_id,
            pointer_down,
            projection_epoch,
            active: false,
            drop_candidate_cycle_index: 0,
            drop_candidate_cycle_side: None,
        });
        self.docking_visual_state.active_tab_drag = None;
    }

    pub fn update_tab_drag_pointer(
        &mut self,
        pointer: UiPoint,
        current_projection_epoch: u64,
    ) -> bool {
        let Some(mut session) = self.tab_drag_session else {
            return false;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if session.active {
            self.tab_drag_session = Some(session);
            return true;
        }

        let delta_x = pointer.x - session.pointer_down.x;
        let delta_y = pointer.y - session.pointer_down.y;
        let distance_squared = delta_x * delta_x + delta_y * delta_y;
        if distance_squared < TAB_DRAG_THRESHOLD_PX * TAB_DRAG_THRESHOLD_PX {
            return false;
        }

        session.active = true;
        self.tab_drag_session = Some(session);
        self.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id: session.panel_instance_id,
            source_tab_stack_id: session.source_tab_stack_id,
            preview_target: None,
            preview_candidates: Vec::new(),
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
        let Some(mut session) = self.tab_drag_session else {
            return;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
        }
        if session.drop_candidate_cycle_side != active_side {
            session.drop_candidate_cycle_side = active_side;
        }
        if !session.active {
            self.tab_drag_session = Some(session);
            return;
        }
        self.tab_drag_session = Some(session);
        self.docking_visual_state.active_tab_drag = Some(ActiveTabDragVisualState {
            panel_instance_id: session.panel_instance_id,
            source_tab_stack_id: session.source_tab_stack_id,
            preview_target: target,
            preview_candidates: candidates,
        });
    }

    pub fn tab_drag_drop_candidate_cycle(&self) -> (usize, Option<DockSplitSide>) {
        self.tab_drag_session
            .map(|session| {
                (
                    session.drop_candidate_cycle_index,
                    session.drop_candidate_cycle_side,
                )
            })
            .unwrap_or((0, None))
    }

    pub fn cycle_active_tab_drag_preview_candidate(&mut self) -> bool {
        let Some(session) = self.tab_drag_session.as_mut() else {
            return false;
        };
        if !session.active {
            return false;
        }
        let Some(drag) = self.docking_visual_state.active_tab_drag.as_mut() else {
            return false;
        };
        if drag.preview_candidates.is_empty() {
            return false;
        }
        let active_side = drag
            .preview_candidates
            .iter()
            .find(|candidate| candidate.active)
            .map(|candidate| candidate.side)
            .or(session.drop_candidate_cycle_side)
            .unwrap_or(drag.preview_candidates[0].side);
        let same_side_indices = drag
            .preview_candidates
            .iter()
            .enumerate()
            .filter_map(|(index, candidate)| (candidate.side == active_side).then_some(index))
            .collect::<Vec<_>>();
        if same_side_indices.is_empty() {
            return false;
        }
        session.drop_candidate_cycle_index = session.drop_candidate_cycle_index.saturating_add(1);
        session.drop_candidate_cycle_side = Some(active_side);
        let active_index =
            same_side_indices[session.drop_candidate_cycle_index % same_side_indices.len()];
        for (index, candidate) in drag.preview_candidates.iter_mut().enumerate() {
            candidate.active = index == active_index;
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
        let session = self.tab_drag_session?;
        let preview_target = self
            .docking_visual_state
            .active_tab_drag
            .as_ref()
            .and_then(|drag| drag.preview_target);
        self.clear_tab_drag();
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
        self.tab_drag_session.map(|session| {
            (
                session.panel_instance_id,
                session.source_tab_stack_id,
                session.projection_epoch,
                session.active,
            )
        })
    }

    pub fn clear_tab_drag(&mut self) {
        self.tab_drag_session = None;
        self.docking_visual_state.active_tab_drag = None;
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
        self.corner_split_resize_session = None;
        self.split_resize_session = Some(SplitResizeSession {
            split_host_id,
            split_widget_id,
            axis,
        });
        self.docking_visual_state.active_split_border_widget = Some(split_widget_id);
    }

    pub fn begin_workspace_corner_split_resize(&mut self, session: CornerSplitResizeSession) {
        self.split_resize_session = None;
        self.corner_area_split_session = None;
        self.corner_split_resize_session = Some(session);
        self.docking_visual_state.active_split_border_widget =
            Some(session.horizontal_split_widget_id);
    }

    pub fn begin_corner_area_split(&mut self, session: CornerAreaSplitSession) {
        self.split_resize_session = None;
        self.corner_split_resize_session = None;
        self.corner_area_split_session = Some(session);
    }

    pub fn active_corner_area_split_session(&self) -> Option<CornerAreaSplitSession> {
        self.corner_area_split_session
    }

    pub fn clear_corner_area_split(&mut self) {
        self.corner_area_split_session = None;
    }

    pub fn active_split_resize_kind(&self) -> Option<WorkspaceSplitKind> {
        let session = self.split_resize_session?;
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
        self.split_resize_session
            .map(|session| (session.split_host_id, session.split_widget_id, session.axis))
    }

    pub fn active_corner_split_resize_session(&self) -> Option<CornerSplitResizeSession> {
        self.corner_split_resize_session
    }

    pub fn clear_split_resize(&mut self) {
        self.split_resize_session = None;
        self.corner_split_resize_session = None;
        self.corner_area_split_session = None;
        self.docking_visual_state.active_split_border_widget = None;
    }

    pub fn set_workspace_split_fraction(
        &mut self,
        split_kind: WorkspaceSplitKind,
        fraction: f32,
    ) -> Result<(), WorkspaceStateError> {
        let Some(split_host_id) = self.resolve_split_host_id(split_kind) else {
            return Err(WorkspaceStateError::ProjectionShapeMismatch(
                "split host path unavailable in workspace graph",
            ));
        };
        self.apply_workspace_mutation(WorkspaceMutation::SetSplitHostFraction {
            split_host_id,
            fraction,
        })
    }

    pub fn set_workspace_split_host_fraction(
        &mut self,
        split_host_id: PanelHostId,
        fraction: f32,
    ) -> Result<(), WorkspaceStateError> {
        self.apply_workspace_mutation(WorkspaceMutation::SetSplitHostFraction {
            split_host_id,
            fraction,
        })
    }

    pub fn clear_cached_projection(&mut self) {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.last_tree = None;
        self.last_bounds = None;
        self.last_projection_artifacts = None;
        self.clear_tab_drag();
    }

    fn resolve_split_host_id(&self, split_kind: WorkspaceSplitKind) -> Option<PanelHostId> {
        let root_host_id = self.workspace_state.root_host_id();
        let root = self.workspace_state.host(root_host_id)?;
        let editor_shell::PanelHostKind::SplitHost(root_split) = root.kind else {
            return None;
        };
        match split_kind {
            WorkspaceSplitKind::BodyConsole => Some(root_host_id),
            WorkspaceSplitKind::LeftRight => Some(root_split.first_child),
            WorkspaceSplitKind::CenterRight => {
                let left_right = self.workspace_state.host(root_split.first_child)?;
                let editor_shell::PanelHostKind::SplitHost(left_right_split) = left_right.kind
                else {
                    return None;
                };
                Some(left_right_split.second_child)
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
