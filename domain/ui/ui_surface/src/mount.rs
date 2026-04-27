//! File: domain/ui/ui_surface/src/mount.rs
//! Purpose: Mounted-surface instance lifecycle and containment contracts.

use std::collections::BTreeMap;

use crate::SurfaceDefinitionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceInstanceId(u64);

impl SurfaceInstanceId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceHostInstanceId(u64);

impl SurfaceHostInstanceId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MountedSurfaceInstance {
    pub surface_instance_id: SurfaceInstanceId,
    pub definition_id: SurfaceDefinitionId,
    pub host_instance_id: SurfaceHostInstanceId,
    pub generation: u64,
}

impl MountedSurfaceInstance {
    pub const fn new(
        surface_instance_id: SurfaceInstanceId,
        definition_id: SurfaceDefinitionId,
        host_instance_id: SurfaceHostInstanceId,
    ) -> Self {
        Self {
            surface_instance_id,
            definition_id,
            host_instance_id,
            generation: 0,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MountedSurfaceRegistry {
    generation: u64,
    mounted_by_surface_id: BTreeMap<SurfaceInstanceId, MountedSurfaceInstance>,
}

impl MountedSurfaceRegistry {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn mounted_surface(
        &self,
        surface_instance_id: SurfaceInstanceId,
    ) -> Option<MountedSurfaceInstance> {
        self.mounted_by_surface_id
            .get(&surface_instance_id)
            .copied()
    }

    pub fn mounted_surfaces(&self) -> impl Iterator<Item = MountedSurfaceInstance> + '_ {
        self.mounted_by_surface_id.values().copied()
    }

    pub fn is_empty(&self) -> bool {
        self.mounted_by_surface_id.is_empty()
    }

    pub fn rebuild(&mut self, mounted_surfaces: impl IntoIterator<Item = MountedSurfaceInstance>) {
        self.generation = self.generation.saturating_add(1);
        self.mounted_by_surface_id.clear();
        for mut mounted in mounted_surfaces {
            mounted.generation = self.generation;
            self.mounted_by_surface_id
                .insert(mounted.surface_instance_id, mounted);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mounted_registry_rebuild_replaces_membership_with_new_generation() {
        let mut registry = MountedSurfaceRegistry::default();
        registry.rebuild([MountedSurfaceInstance::new(
            SurfaceInstanceId::new(10),
            SurfaceDefinitionId::new(1),
            SurfaceHostInstanceId::new(100),
        )]);

        let first = registry
            .mounted_surface(SurfaceInstanceId::new(10))
            .expect("surface should be mounted after first rebuild");
        assert_eq!(first.generation, 1);

        registry.rebuild([MountedSurfaceInstance::new(
            SurfaceInstanceId::new(11),
            SurfaceDefinitionId::new(2),
            SurfaceHostInstanceId::new(101),
        )]);
        assert!(
            registry
                .mounted_surface(SurfaceInstanceId::new(10))
                .is_none()
        );

        let second = registry
            .mounted_surface(SurfaceInstanceId::new(11))
            .expect("surface should be mounted after second rebuild");
        assert_eq!(second.generation, 2);
    }
}
