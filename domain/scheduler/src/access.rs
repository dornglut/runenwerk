use std::any::TypeId;
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AccessDomain {
    Component,
    OrphanedComponent,
    Resource,
    Structural,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AccessKey {
    domain: AccessDomain,
    type_id: Option<TypeId>,
    name: &'static str,
}

impl AccessKey {
    pub fn component<T: 'static>(name: &'static str) -> Self {
        Self::component_by_id(TypeId::of::<T>(), name)
    }

    pub fn component_by_id(type_id: TypeId, name: &'static str) -> Self {
        Self {
            domain: AccessDomain::Component,
            type_id: Some(type_id),
            name,
        }
    }

    pub fn resource<T: 'static>(name: &'static str) -> Self {
        Self::resource_by_id(TypeId::of::<T>(), name)
    }

    pub fn orphaned_component<T: 'static>(name: &'static str) -> Self {
        Self::orphaned_component_by_id(TypeId::of::<T>(), name)
    }

    pub fn orphaned_component_by_id(type_id: TypeId, name: &'static str) -> Self {
        Self {
            domain: AccessDomain::OrphanedComponent,
            type_id: Some(type_id),
            name,
        }
    }

    pub fn resource_by_id(type_id: TypeId, name: &'static str) -> Self {
        Self {
            domain: AccessDomain::Resource,
            type_id: Some(type_id),
            name,
        }
    }

    pub fn structural(name: &'static str) -> Self {
        Self {
            domain: AccessDomain::Structural,
            type_id: None,
            name,
        }
    }

    pub fn domain(&self) -> AccessDomain {
        self.domain
    }

    pub fn type_id(&self) -> Option<TypeId> {
        self.type_id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    ReadWrite,
    WriteWrite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessConflict {
    pub key: AccessKey,
    pub kind: ConflictKind,
}

#[derive(Debug, Clone, Default)]
pub struct SystemAccess {
    reads: HashSet<AccessKey>,
    writes: HashSet<AccessKey>,
}

impl SystemAccess {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reads(&self) -> &HashSet<AccessKey> {
        &self.reads
    }

    pub fn writes(&self) -> &HashSet<AccessKey> {
        &self.writes
    }

    pub fn add_read(&mut self, key: AccessKey) {
        self.reads.insert(key);
    }

    pub fn add_write(&mut self, key: AccessKey) {
        self.writes.insert(key);
    }

    pub fn with_read(mut self, key: AccessKey) -> Self {
        self.add_read(key);
        self
    }

    pub fn with_write(mut self, key: AccessKey) -> Self {
        self.add_write(key);
        self
    }

    pub fn conflicts_with(&self, other: &Self) -> Vec<AccessConflict> {
        let mut conflicts = Vec::new();

        for key in self.writes.intersection(&other.writes) {
            if key.domain() == AccessDomain::Structural {
                // Deferred structural mutation is merged at stage end; multiple producers can
                // coexist in a stage and are serialized by deterministic system order.
                continue;
            }
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::WriteWrite,
            });
        }

        for key in self.writes.intersection(&other.reads) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadWrite,
            });
        }

        for key in self.reads.intersection(&other.writes) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadWrite,
            });
        }

        conflicts
    }

    pub fn validate_internal(&self) -> Result<(), AccessConflict> {
        for key in self.reads.intersection(&self.writes) {
            return Err(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadWrite,
            });
        }
        Ok(())
    }
}
