//! File: apps/runenwerk_editor/src/runtime/viewport/surface_mounts.rs
//! Purpose: Runtime resources for ui_surface definitions and mounted surface lifecycle.

use editor_shell::{WorkspaceState, editor_surface_definitions, mounted_surface_instances};
use ui_surface::{
    MountedSurfaceInstance, MountedSurfaceRegistry, SurfaceDefinition, SurfaceDefinitionId,
    SurfaceDefinitionRegistry, SurfaceInstanceId,
};

#[derive(Debug, Clone, ecs::Component, ecs::Resource)]
pub struct SurfaceDefinitionRegistryResource {
    registry: SurfaceDefinitionRegistry,
}

impl Default for SurfaceDefinitionRegistryResource {
    fn default() -> Self {
        let mut registry = SurfaceDefinitionRegistry::default();
        for definition in editor_surface_definitions() {
            registry.register(definition);
        }
        Self { registry }
    }
}

impl SurfaceDefinitionRegistryResource {
    pub fn definition(&self, definition_id: SurfaceDefinitionId) -> Option<SurfaceDefinition> {
        self.registry.definition(definition_id)
    }

    pub fn definitions(&self) -> impl Iterator<Item = SurfaceDefinition> + '_ {
        self.registry.definitions()
    }

    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }
}

#[derive(Debug, Clone, ecs::Component, ecs::Resource, Default)]
pub struct MountedSurfaceRegistryResource {
    registry: MountedSurfaceRegistry,
}

impl MountedSurfaceRegistryResource {
    pub fn generation(&self) -> u64 {
        self.registry.generation()
    }

    pub fn mounted_surface(
        &self,
        surface_instance_id: SurfaceInstanceId,
    ) -> Option<MountedSurfaceInstance> {
        self.registry.mounted_surface(surface_instance_id)
    }

    pub fn mounted_surfaces(&self) -> impl Iterator<Item = MountedSurfaceInstance> + '_ {
        self.registry.mounted_surfaces()
    }

    pub fn is_empty(&self) -> bool {
        self.registry.is_empty()
    }

    pub fn sync_from_workspace_state(&mut self, workspace_state: &WorkspaceState) {
        self.registry
            .rebuild(mounted_surface_instances(workspace_state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{
        VIEWPORT_SURFACE_DEFINITION_ID, WorkspaceIdentityAllocator, WorkspaceState,
    };

    #[test]
    fn definition_registry_seeds_editor_surface_definitions() {
        let definitions = SurfaceDefinitionRegistryResource::default()
            .definitions()
            .collect::<Vec<_>>();

        assert!(!definitions.is_empty());
        assert!(
            definitions
                .iter()
                .any(|definition| definition.id == VIEWPORT_SURFACE_DEFINITION_ID)
        );
    }

    #[test]
    fn mounted_registry_syncs_from_workspace_mounts() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace_state =
            WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);

        let mut mounts = MountedSurfaceRegistryResource::default();
        mounts.sync_from_workspace_state(&workspace_state);

        assert_eq!(mounts.generation(), 1);
        assert_eq!(mounts.mounted_surfaces().count(), 4);
    }
}
