use editor_shell::{
    PanelHostId, PanelInstanceId, ShellProjectionArtifacts, TabStackId, UiRuntime, UiTree,
    WorkspaceId, WorkspaceIdentityAllocator, WorkspaceMutation, WorkspaceState,
    WorkspaceStateError, reduce_workspace,
};
use ui_math::UiRect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellTabDragState {
    pub panel_instance_id: PanelInstanceId,
    pub source_tab_stack_id: TabStackId,
    pub hovered_tab_stack_id: Option<TabStackId>,
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
    tab_drag_state: Option<ShellTabDragState>,
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
            tab_drag_state: None,
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

    pub fn identity_allocator(&self) -> &WorkspaceIdentityAllocator {
        &self.identity_allocator
    }

    pub fn allocate_panel_host_id(&mut self) -> PanelHostId {
        self.identity_allocator.allocate_panel_host_id()
    }

    pub fn allocate_tab_stack_id(&mut self) -> TabStackId {
        self.identity_allocator.allocate_tab_stack_id()
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
        self.identity_allocator =
            WorkspaceIdentityAllocator::from_seed(self.workspace_state.next_identity_seed());
        self.clear_cached_projection();
        Ok(())
    }

    pub fn apply_workspace_mutations(
        &mut self,
        ops: impl IntoIterator<Item = WorkspaceMutation>,
    ) -> Result<(), WorkspaceStateError> {
        let mut next = self.workspace_state.clone();
        for op in ops {
            next = reduce_workspace(&next, op)?;
        }
        self.workspace_state = next;
        self.identity_allocator =
            WorkspaceIdentityAllocator::from_seed(self.workspace_state.next_identity_seed());
        self.clear_cached_projection();
        Ok(())
    }

    pub fn replace_workspace_state(
        &mut self,
        workspace_state: WorkspaceState,
    ) -> Result<(), WorkspaceStateError> {
        workspace_state.validate_integrity()?;
        self.workspace_state = workspace_state;
        self.identity_allocator =
            WorkspaceIdentityAllocator::from_seed(self.workspace_state.next_identity_seed());
        self.clear_cached_projection();
        Ok(())
    }

    pub fn tab_drag_state(&self) -> Option<ShellTabDragState> {
        self.tab_drag_state
    }

    pub fn begin_tab_drag(
        &mut self,
        panel_instance_id: PanelInstanceId,
        source_tab_stack_id: TabStackId,
    ) {
        self.tab_drag_state = Some(ShellTabDragState {
            panel_instance_id,
            source_tab_stack_id,
            hovered_tab_stack_id: None,
        });
    }

    pub fn set_tab_drag_hover_target(&mut self, tab_stack_id: Option<TabStackId>) {
        if let Some(state) = &mut self.tab_drag_state {
            state.hovered_tab_stack_id = tab_stack_id;
        }
    }

    pub fn clear_tab_drag(&mut self) {
        self.tab_drag_state = None;
    }

    pub fn end_tab_drag(&mut self) -> Option<ShellTabDragState> {
        self.tab_drag_state.take()
    }

    pub fn clear_cached_projection(&mut self) {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.last_tree = None;
        self.last_bounds = None;
        self.last_projection_artifacts = None;
        self.tab_drag_state = None;
    }
}
