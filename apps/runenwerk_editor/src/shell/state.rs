use editor_shell::{
    ActiveTabDragVisualState, DockingInteractionVisualState, DockingPreviewDropTarget, PanelHostId,
    PanelInstanceId, ShellProjectionArtifacts, TabStackId, UiRuntime, UiTree, WorkspaceId,
    WorkspaceIdentityAllocator, WorkspaceMutation, WorkspaceState, WorkspaceStateError,
    reduce_workspace,
};
use ui_math::{UiPoint, UiRect};

const TAB_DRAG_THRESHOLD_PX: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct TabDragSession {
    panel_instance_id: PanelInstanceId,
    source_tab_stack_id: TabStackId,
    pointer_down: UiPoint,
    projection_epoch: u64,
    active: bool,
}

#[derive(Debug)]
pub struct RunenwerkEditorShellState {
    runtime: UiRuntime,
    last_tree: Option<UiTree>,
    last_bounds: Option<UiRect>,
    last_projection_artifacts: Option<ShellProjectionArtifacts>,
    projection_epoch: u64,
    identity_allocator: WorkspaceIdentityAllocator,
    workspace_state: WorkspaceState,
    tab_drag_session: Option<TabDragSession>,
    docking_visual_state: DockingInteractionVisualState,
}

impl Default for RunenwerkEditorShellState {
    fn default() -> Self {
        let mut identity_allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = identity_allocator.allocate_workspace_id();
        let workspace_state =
            WorkspaceState::bootstrap_current_layout(workspace_id, &mut identity_allocator);
        debug_assert!(workspace_state.validate_integrity().is_ok());
        Self {
            runtime: UiRuntime::new(),
            last_tree: None,
            last_bounds: None,
            last_projection_artifacts: None,
            projection_epoch: 0,
            identity_allocator,
            workspace_state,
            tab_drag_session: None,
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

    pub fn workspace_state(&self) -> &WorkspaceState {
        &self.workspace_state
    }

    pub fn replace_workspace_state(&mut self, workspace_state: WorkspaceState) {
        self.identity_allocator =
            WorkspaceIdentityAllocator::from_seed(workspace_state.next_identity_seed());
        self.workspace_state = workspace_state;
        self.clear_cached_projection();
    }

    pub fn allocate_panel_host_id(&mut self) -> PanelHostId {
        self.identity_allocator.allocate_panel_host_id()
    }

    pub fn allocate_tab_stack_id(&mut self) -> TabStackId {
        self.identity_allocator.allocate_tab_stack_id()
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

    pub fn docking_visual_state(&self) -> DockingInteractionVisualState {
        self.docking_visual_state
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
        });
        true
    }

    pub fn set_tab_drag_preview_target(
        &mut self,
        target: Option<DockingPreviewDropTarget>,
        current_projection_epoch: u64,
    ) {
        let Some(mut session) = self.tab_drag_session else {
            return;
        };
        if session.projection_epoch != current_projection_epoch {
            session.projection_epoch = current_projection_epoch;
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
        });
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

    pub fn clear_tab_drag(&mut self) {
        self.tab_drag_session = None;
        self.docking_visual_state.active_tab_drag = None;
    }

    pub fn clear_cached_projection(&mut self) {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.last_tree = None;
        self.last_bounds = None;
        self.last_projection_artifacts = None;
        self.clear_tab_drag();
    }
}
