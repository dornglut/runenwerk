use editor_shell::{
    ShellProjectionArtifacts, UiRuntime, UiTree, WorkspaceId, WorkspaceIdentityAllocator,
    WorkspaceMutation, WorkspaceState, WorkspaceStateError, reduce_workspace,
};
use ui_math::UiRect;

#[derive(Debug)]
pub struct RunenwerkEditorShellState {
    runtime: UiRuntime,
    last_tree: Option<UiTree>,
    last_bounds: Option<UiRect>,
    last_projection_artifacts: Option<ShellProjectionArtifacts>,
    projection_epoch: u64,
    identity_allocator: WorkspaceIdentityAllocator,
    workspace_state: WorkspaceState,
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

    pub fn clear_cached_projection(&mut self) {
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.last_tree = None;
        self.last_bounds = None;
        self.last_projection_artifacts = None;
    }
}
