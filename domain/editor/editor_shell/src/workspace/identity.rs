//! File: domain/editor/editor_shell/src/workspace/identity.rs
//! Purpose: Typed workspace identity and allocation contracts.

use id_macros::id;

#[id]
pub struct WorkspaceId;

#[id]
pub struct PanelHostId;

#[id]
pub struct PanelInstanceId;

#[id]
pub struct ToolSurfaceInstanceId;

#[id]
pub struct TabStackId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceIdentitySeed {
    pub next_workspace_id: u64,
    pub next_panel_host_id: u64,
    pub next_panel_instance_id: u64,
    pub next_tool_surface_instance_id: u64,
    pub next_tab_stack_id: u64,
}

impl Default for WorkspaceIdentitySeed {
    fn default() -> Self {
        Self {
            next_workspace_id: 1,
            next_panel_host_id: 1,
            next_panel_instance_id: 1,
            next_tool_surface_instance_id: 1,
            next_tab_stack_id: 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WorkspaceIdentityAllocator {
    workspace_ids: WorkspaceIdSequence,
    panel_host_ids: PanelHostIdSequence,
    panel_instance_ids: PanelInstanceIdSequence,
    tool_surface_instance_ids: ToolSurfaceInstanceIdSequence,
    tab_stack_ids: TabStackIdSequence,
}

impl WorkspaceIdentityAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_seed(seed: WorkspaceIdentitySeed) -> Self {
        Self {
            workspace_ids: WorkspaceIdSequence::new(seed.next_workspace_id),
            panel_host_ids: PanelHostIdSequence::new(seed.next_panel_host_id),
            panel_instance_ids: PanelInstanceIdSequence::new(seed.next_panel_instance_id),
            tool_surface_instance_ids: ToolSurfaceInstanceIdSequence::new(
                seed.next_tool_surface_instance_id,
            ),
            tab_stack_ids: TabStackIdSequence::new(seed.next_tab_stack_id),
        }
    }

    pub fn seed_snapshot(&self) -> WorkspaceIdentitySeed {
        WorkspaceIdentitySeed {
            next_workspace_id: self.workspace_ids.next_raw(),
            next_panel_host_id: self.panel_host_ids.next_raw(),
            next_panel_instance_id: self.panel_instance_ids.next_raw(),
            next_tool_surface_instance_id: self.tool_surface_instance_ids.next_raw(),
            next_tab_stack_id: self.tab_stack_ids.next_raw(),
        }
    }

    pub fn allocate_workspace_id(&mut self) -> WorkspaceId {
        self.workspace_ids.allocate().into()
    }

    pub fn allocate_panel_host_id(&mut self) -> PanelHostId {
        self.panel_host_ids.allocate().into()
    }

    pub fn allocate_panel_instance_id(&mut self) -> PanelInstanceId {
        self.panel_instance_ids.allocate().into()
    }

    pub fn allocate_tool_surface_instance_id(&mut self) -> ToolSurfaceInstanceId {
        self.tool_surface_instance_ids.allocate().into()
    }

    pub fn allocate_tab_stack_id(&mut self) -> TabStackId {
        self.tab_stack_ids.allocate().into()
    }
}

impl Default for WorkspaceIdentityAllocator {
    fn default() -> Self {
        Self::from_seed(WorkspaceIdentitySeed::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_allocates_monotonic_ids_per_family() {
        let mut allocator = WorkspaceIdentityAllocator::new();

        let workspace_a = allocator.allocate_workspace_id();
        let workspace_b = allocator.allocate_workspace_id();
        assert_eq!(workspace_a.raw(), 1);
        assert_eq!(workspace_b.raw(), 2);

        let host_a = allocator.allocate_panel_host_id();
        let host_b = allocator.allocate_panel_host_id();
        assert_eq!(host_a.raw(), 1);
        assert_eq!(host_b.raw(), 2);

        let panel_a = allocator.allocate_panel_instance_id();
        let panel_b = allocator.allocate_panel_instance_id();
        assert_eq!(panel_a.raw(), 1);
        assert_eq!(panel_b.raw(), 2);

        let surface_a = allocator.allocate_tool_surface_instance_id();
        let surface_b = allocator.allocate_tool_surface_instance_id();
        assert_eq!(surface_a.raw(), 1);
        assert_eq!(surface_b.raw(), 2);

        let stack_a = allocator.allocate_tab_stack_id();
        let stack_b = allocator.allocate_tab_stack_id();
        assert_eq!(stack_a.raw(), 1);
        assert_eq!(stack_b.raw(), 2);
    }

    #[test]
    fn allocator_families_are_independent() {
        let mut allocator = WorkspaceIdentityAllocator::new();

        let _ = allocator.allocate_workspace_id();
        let _ = allocator.allocate_workspace_id();
        let _ = allocator.allocate_workspace_id();
        let host = allocator.allocate_panel_host_id();
        let panel = allocator.allocate_panel_instance_id();
        let surface = allocator.allocate_tool_surface_instance_id();
        let stack = allocator.allocate_tab_stack_id();

        assert_eq!(host.raw(), 1);
        assert_eq!(panel.raw(), 1);
        assert_eq!(surface.raw(), 1);
        assert_eq!(stack.raw(), 1);
    }

    #[test]
    fn allocator_seed_snapshot_roundtrip_is_deterministic() {
        let seed = WorkspaceIdentitySeed {
            next_workspace_id: 11,
            next_panel_host_id: 21,
            next_panel_instance_id: 31,
            next_tool_surface_instance_id: 41,
            next_tab_stack_id: 51,
        };

        let mut allocator = WorkspaceIdentityAllocator::from_seed(seed);
        assert_eq!(allocator.allocate_workspace_id().raw(), 11);
        assert_eq!(allocator.allocate_panel_host_id().raw(), 21);
        assert_eq!(allocator.allocate_panel_instance_id().raw(), 31);
        assert_eq!(allocator.allocate_tool_surface_instance_id().raw(), 41);
        assert_eq!(allocator.allocate_tab_stack_id().raw(), 51);

        let snapshot = allocator.seed_snapshot();
        assert_eq!(
            snapshot,
            WorkspaceIdentitySeed {
                next_workspace_id: 12,
                next_panel_host_id: 22,
                next_panel_instance_id: 32,
                next_tool_surface_instance_id: 42,
                next_tab_stack_id: 52,
            }
        );

        let mut restored = WorkspaceIdentityAllocator::from_seed(snapshot);
        assert_eq!(restored.allocate_workspace_id().raw(), 12);
        assert_eq!(restored.allocate_panel_host_id().raw(), 22);
        assert_eq!(restored.allocate_panel_instance_id().raw(), 32);
        assert_eq!(restored.allocate_tool_surface_instance_id().raw(), 42);
        assert_eq!(restored.allocate_tab_stack_id().raw(), 52);
    }
}
