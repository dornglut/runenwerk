use super::model::{
    OwnerId, OwnerRole, OwnerState, OwnershipTarget, OwnershipTransferRecord, ResourceOwnerKey,
    ResourceOwnershipDescriptor,
};
use crate::component::Resource;
use crate::entity::Entity;
use crate::world::World;
use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Default)]
pub(crate) struct OwnershipRegistry {
    next_owner_id: u64,
    next_resource_key: u64,
    next_transfer_sequence: u64,
    owner_roles: BTreeMap<OwnerId, OwnerRole>,
    entity_owner: BTreeMap<Entity, OwnerState>,
    resource_owner_by_key: BTreeMap<ResourceOwnerKey, OwnerState>,
    resource_key_by_type: HashMap<TypeId, ResourceOwnerKey>,
    resource_name_by_key: BTreeMap<ResourceOwnerKey, &'static str>,
    targets_by_owner: BTreeMap<OwnerId, BTreeSet<OwnershipTarget>>,
    transfer_log: Vec<OwnershipTransferRecord>,
}

impl OwnershipRegistry {
    pub(super) fn create_owner(&mut self, role: OwnerRole) -> OwnerId {
        self.next_owner_id = self.next_owner_id.saturating_add(1);
        let id = OwnerId::from_raw(self.next_owner_id);
        self.owner_roles.insert(id, role);
        self.targets_by_owner.entry(id).or_default();
        id
    }

    pub(super) fn owner_role(&self, owner: OwnerId) -> Option<OwnerRole> {
        self.owner_roles.get(&owner).copied()
    }

    pub(super) fn set_owner_role(&mut self, owner: OwnerId, role: OwnerRole) -> bool {
        match self.owner_roles.get_mut(&owner) {
            Some(current) => {
                if *current == role {
                    false
                } else {
                    *current = role;
                    true
                }
            }
            None => {
                self.owner_roles.insert(owner, role);
                self.targets_by_owner.entry(owner).or_default();
                true
            }
        }
    }

    pub(super) fn ensure_resource_key(
        &mut self,
        resource_type: TypeId,
        resource_name: &'static str,
    ) -> ResourceOwnerKey {
        if let Some(existing) = self.resource_key_by_type.get(&resource_type).copied() {
            return existing;
        }

        self.next_resource_key = self.next_resource_key.saturating_add(1);
        let key = ResourceOwnerKey::from_raw(self.next_resource_key);
        self.resource_key_by_type.insert(resource_type, key);
        self.resource_name_by_key.insert(key, resource_name);
        key
    }

    pub(super) fn resource_key(&self, resource_type: TypeId) -> Option<ResourceOwnerKey> {
        self.resource_key_by_type.get(&resource_type).copied()
    }

    pub(super) fn resource_descriptor(
        &self,
        key: ResourceOwnerKey,
    ) -> Option<ResourceOwnershipDescriptor> {
        let resource_name = self.resource_name_by_key.get(&key).copied()?;
        Some(ResourceOwnershipDescriptor { key, resource_name })
    }

    pub(super) fn entity_owner(&self, entity: Entity) -> OwnerState {
        self.entity_owner
            .get(&entity)
            .copied()
            .unwrap_or(OwnerState::Unowned)
    }

    pub(super) fn resource_owner_by_key(&self, key: ResourceOwnerKey) -> OwnerState {
        self.resource_owner_by_key
            .get(&key)
            .copied()
            .unwrap_or(OwnerState::Unowned)
    }

    pub(super) fn assign_entity_owner(&mut self, entity: Entity, next: OwnerState) -> bool {
        let target = OwnershipTarget::Entity(entity);
        self.assign_target_owner(target, next)
    }

    pub(super) fn assign_resource_owner_by_key(
        &mut self,
        key: ResourceOwnerKey,
        next: OwnerState,
    ) -> bool {
        let target = OwnershipTarget::Resource(key);
        self.assign_target_owner(target, next)
    }

    pub(super) fn owned_targets(&self, owner: OwnerId) -> Vec<OwnershipTarget> {
        self.targets_by_owner
            .get(&owner)
            .map(|targets| targets.iter().copied().collect())
            .unwrap_or_default()
    }

    pub(super) fn owned_entities(&self, owner: OwnerId) -> Vec<Entity> {
        self.owned_targets(owner)
            .into_iter()
            .filter_map(|target| match target {
                OwnershipTarget::Entity(entity) => Some(entity),
                OwnershipTarget::Resource(_) => None,
            })
            .collect()
    }

    pub(super) fn owned_resources(&self, owner: OwnerId) -> Vec<ResourceOwnerKey> {
        self.owned_targets(owner)
            .into_iter()
            .filter_map(|target| match target {
                OwnershipTarget::Entity(_) => None,
                OwnershipTarget::Resource(key) => Some(key),
            })
            .collect()
    }

    pub(super) fn transfer_log_since(&self, sequence: u64) -> Vec<OwnershipTransferRecord> {
        self.transfer_log
            .iter()
            .copied()
            .filter(|record| record.sequence > sequence)
            .collect()
    }

    pub(super) fn current_transfer_sequence(&self) -> u64 {
        self.next_transfer_sequence
    }

    fn assign_target_owner(&mut self, target: OwnershipTarget, next: OwnerState) -> bool {
        let previous = match target {
            OwnershipTarget::Entity(entity) => self.entity_owner(entity),
            OwnershipTarget::Resource(key) => self.resource_owner_by_key(key),
        };

        if previous == next {
            return false;
        }

        self.remove_owned_target(previous, target);

        match target {
            OwnershipTarget::Entity(entity) => {
                if next == OwnerState::Unowned {
                    self.entity_owner.remove(&entity);
                } else {
                    self.entity_owner.insert(entity, next);
                }
            }
            OwnershipTarget::Resource(key) => {
                if next == OwnerState::Unowned {
                    self.resource_owner_by_key.remove(&key);
                } else {
                    self.resource_owner_by_key.insert(key, next);
                }
            }
        }

        self.add_owned_target(next, target);

        self.next_transfer_sequence = self.next_transfer_sequence.saturating_add(1);
        self.transfer_log.push(OwnershipTransferRecord {
            sequence: self.next_transfer_sequence,
            target,
            previous,
            next,
        });
        true
    }

    fn remove_owned_target(&mut self, owner: OwnerState, target: OwnershipTarget) {
        let OwnerState::OwnedBy(owner_id) = owner else {
            return;
        };

        if let Some(targets) = self.targets_by_owner.get_mut(&owner_id) {
            targets.remove(&target);
        }
    }

    fn add_owned_target(&mut self, owner: OwnerState, target: OwnershipTarget) {
        let OwnerState::OwnedBy(owner_id) = owner else {
            return;
        };

        self.targets_by_owner
            .entry(owner_id)
            .or_default()
            .insert(target);
    }
}

impl World {
    pub fn create_owner(&mut self, role: OwnerRole) -> OwnerId {
        self.ownership.create_owner(role)
    }

    pub fn set_owner_role(&mut self, owner: OwnerId, role: OwnerRole) -> bool {
        self.ownership.set_owner_role(owner, role)
    }

    pub fn owner_role(&self, owner: OwnerId) -> Option<OwnerRole> {
        self.ownership.owner_role(owner)
    }

    pub fn entity_owner(&self, entity: Entity) -> OwnerState {
        self.ownership.entity_owner(entity)
    }

    pub fn assign_entity_owner(&mut self, entity: Entity, owner: OwnerState) -> bool {
        self.ownership.assign_entity_owner(entity, owner)
    }

    pub fn transfer_entity_owner(&mut self, entity: Entity, owner: OwnerState) -> bool {
        self.assign_entity_owner(entity, owner)
    }

    pub fn resource_owner_key<R: Resource>(&self) -> Option<ResourceOwnerKey> {
        self.ownership.resource_key(TypeId::of::<R>())
    }

    pub fn ensure_resource_owner_key<R: Resource>(&mut self) -> ResourceOwnerKey {
        self.ownership
            .ensure_resource_key(TypeId::of::<R>(), R::resource_name())
    }

    pub fn resource_ownership_descriptor(
        &self,
        key: ResourceOwnerKey,
    ) -> Option<ResourceOwnershipDescriptor> {
        self.ownership.resource_descriptor(key)
    }

    pub fn resource_owner<R: Resource>(&self) -> OwnerState {
        let Some(key) = self.resource_owner_key::<R>() else {
            return OwnerState::Unowned;
        };
        self.ownership.resource_owner_by_key(key)
    }

    pub fn resource_owner_by_key(&self, key: ResourceOwnerKey) -> OwnerState {
        self.ownership.resource_owner_by_key(key)
    }

    pub fn resource_owner_by_type_id(&self, resource_type: TypeId) -> OwnerState {
        let Some(key) = self.ownership.resource_key(resource_type) else {
            return OwnerState::Unowned;
        };
        self.ownership.resource_owner_by_key(key)
    }

    pub fn assign_resource_owner<R: Resource>(&mut self, owner: OwnerState) -> bool {
        let key = self.ensure_resource_owner_key::<R>();
        self.ownership.assign_resource_owner_by_key(key, owner)
    }

    pub fn transfer_resource_owner<R: Resource>(&mut self, owner: OwnerState) -> bool {
        self.assign_resource_owner::<R>(owner)
    }

    pub fn owned_targets(&self, owner: OwnerId) -> Vec<OwnershipTarget> {
        self.ownership.owned_targets(owner)
    }

    pub fn owned_entities(&self, owner: OwnerId) -> Vec<Entity> {
        self.ownership.owned_entities(owner)
    }

    pub fn owned_resources(&self, owner: OwnerId) -> Vec<ResourceOwnerKey> {
        self.ownership.owned_resources(owner)
    }

    pub fn transfer_owned_targets_to_world(&mut self, owner: OwnerId) -> usize {
        let targets = self.ownership.owned_targets(owner);
        let mut transferred = 0usize;
        for target in targets {
            let changed = match target {
                OwnershipTarget::Entity(entity) => self
                    .ownership
                    .assign_entity_owner(entity, OwnerState::WorldOwned),
                OwnershipTarget::Resource(resource) => self
                    .ownership
                    .assign_resource_owner_by_key(resource, OwnerState::WorldOwned),
            };
            if changed {
                transferred = transferred.saturating_add(1);
            }
        }
        transferred
    }

    pub fn ownership_transfer_sequence(&self) -> u64 {
        self.ownership.current_transfer_sequence()
    }

    pub fn ownership_transfers_since(&self, sequence: u64) -> Vec<OwnershipTransferRecord> {
        self.ownership.transfer_log_since(sequence)
    }
}
