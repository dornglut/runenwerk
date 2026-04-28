use std::any::TypeId;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AccessDomain {
    Component,
    OrphanedComponent,
    Resource,
    BroadcastStream,
    WorkQueue,
    TickBuffer,
    Structural,
}

#[derive(Debug, Copy, Clone)]
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

    pub fn broadcast_stream<T: 'static>(name: &'static str) -> Self {
        Self::broadcast_stream_by_id(TypeId::of::<T>(), name)
    }

    pub fn broadcast_stream_by_id(type_id: TypeId, name: &'static str) -> Self {
        Self {
            domain: AccessDomain::BroadcastStream,
            type_id: Some(type_id),
            name,
        }
    }

    pub fn work_queue<T: 'static>(name: &'static str) -> Self {
        Self::work_queue_by_id(TypeId::of::<T>(), name)
    }

    pub fn work_queue_by_id(type_id: TypeId, name: &'static str) -> Self {
        Self {
            domain: AccessDomain::WorkQueue,
            type_id: Some(type_id),
            name,
        }
    }

    pub fn tick_buffer<T: 'static>(name: &'static str) -> Self {
        Self::tick_buffer_by_id(TypeId::of::<T>(), name)
    }

    pub fn tick_buffer_by_id(type_id: TypeId, name: &'static str) -> Self {
        Self {
            domain: AccessDomain::TickBuffer,
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

impl PartialEq for AccessKey {
    fn eq(&self, other: &Self) -> bool {
        if self.domain != other.domain {
            return false;
        }

        match self.domain {
            AccessDomain::Structural => self.name == other.name,
            _ => self.type_id == other.type_id,
        }
    }
}

impl Eq for AccessKey {}

impl Hash for AccessKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.domain.hash(state);
        match self.domain {
            AccessDomain::Structural => self.name.hash(state),
            _ => self.type_id.hash(state),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    ReadWrite,
    WriteWrite,
    ReadDrain,
    WriteDrain,
    DrainDrain,
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
    drains: HashSet<AccessKey>,
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

    pub fn drains(&self) -> &HashSet<AccessKey> {
        &self.drains
    }

    pub fn add_read(&mut self, key: AccessKey) {
        self.reads.insert(key);
    }

    pub fn add_write(&mut self, key: AccessKey) {
        self.writes.insert(key);
    }

    pub fn add_drain(&mut self, key: AccessKey) {
        self.drains.insert(key);
    }

    pub fn with_read(mut self, key: AccessKey) -> Self {
        self.add_read(key);
        self
    }

    pub fn with_write(mut self, key: AccessKey) -> Self {
        self.add_write(key);
        self
    }

    pub fn with_drain(mut self, key: AccessKey) -> Self {
        self.add_drain(key);
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

        for key in self.reads.intersection(&other.drains) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadDrain,
            });
        }

        for key in self.drains.intersection(&other.reads) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadDrain,
            });
        }

        for key in self.writes.intersection(&other.drains) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::WriteDrain,
            });
        }

        for key in self.drains.intersection(&other.writes) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::WriteDrain,
            });
        }

        for key in self.drains.intersection(&other.drains) {
            conflicts.push(AccessConflict {
                key: *key,
                kind: ConflictKind::DrainDrain,
            });
        }

        conflicts
    }

    pub fn validate_internal(&self) -> Result<(), AccessConflict> {
        if let Some(key) = self.reads.intersection(&self.writes).next() {
            return Err(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadWrite,
            });
        }

        if let Some(key) = self.reads.intersection(&self.drains).next() {
            return Err(AccessConflict {
                key: *key,
                kind: ConflictKind::ReadDrain,
            });
        }

        if let Some(key) = self.writes.intersection(&self.drains).next() {
            return Err(AccessConflict {
                key: *key,
                kind: ConflictKind::WriteDrain,
            });
        }

        Ok(())
    }
}
