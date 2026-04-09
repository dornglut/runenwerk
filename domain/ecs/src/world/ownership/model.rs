use crate::entity::Entity;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OwnerId(u64);

impl OwnerId {
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OwnerRole {
    Active,
    Observer,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum OwnerState {
    Unowned,
    WorldOwned,
    OwnedBy(OwnerId),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceOwnerKey(u64);

impl ResourceOwnerKey {
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    pub const fn as_raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OwnershipTarget {
    Entity(Entity),
    Resource(ResourceOwnerKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOwnershipDescriptor {
    pub key: ResourceOwnerKey,
    pub resource_name: &'static str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct OwnershipTransferRecord {
    pub sequence: u64,
    pub target: OwnershipTarget,
    pub previous: OwnerState,
    pub next: OwnerState,
}
