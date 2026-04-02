// Owner: ecs World - Change Tracking Types
use crate::entity::Entity;
use std::any::TypeId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ComponentChangeKind {
    Added,
    Modified,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentChangeRecord {
    pub tick: u64,
    pub entity: Entity,
    pub component_type: TypeId,
    pub component_name: &'static str,
    pub kind: ComponentChangeKind,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) struct RemovedComponentRecord {
    pub(super) tick: u64,
    pub(super) entity: Entity,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResourceChangeKind {
    Inserted,
    Modified,
    Removed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceChangeRecord {
    pub tick: u64,
    pub resource_type: TypeId,
    pub resource_name: &'static str,
    pub kind: ResourceChangeKind,
}

#[derive(Debug)]
pub(super) struct ComponentMeta {
    pub(super) _id: u32,
    pub(super) name: &'static str,
}
