use super::model::{
    ControllerId, ControllerRole, OwnerState, OwnershipTarget, OwnershipTransferRecord,
    ResourceOwnerKey, ResourceOwnershipDescriptor,
};
use crate::component::Resource;
use crate::entity::Entity;
use crate::world::world::World;
use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Default)]
pub(crate) struct OwnershipRegistry {
    next_controller_id: u64,
    next_resource_key: u64,
    next_transfer_sequence: u64,
    controller_roles: BTreeMap<ControllerId, ControllerRole>,
    entity_owner: BTreeMap<Entity, OwnerState>,
    resource_owner_by_key: BTreeMap<ResourceOwnerKey, OwnerState>,
    resource_key_by_type: HashMap<TypeId, ResourceOwnerKey>,
    resource_name_by_key: BTreeMap<ResourceOwnerKey, &'static str>,
    targets_by_controller: BTreeMap<ControllerId, BTreeSet<OwnershipTarget>>,
    transfer_log: Vec<OwnershipTransferRecord>,
}

impl OwnershipRegistry {
    pub(super) fn create_controller(&mut self, role: ControllerRole) -> ControllerId {
        self.next_controller_id = self.next_controller_id.saturating_add(1);
        let id = ControllerId::from_raw(self.next_controller_id);
        self.controller_roles.insert(id, role);
        self.targets_by_controller.entry(id).or_default();
        id
    }

    pub(super) fn controller_role(&self, controller: ControllerId) -> Option<ControllerRole> {
        self.controller_roles.get(&controller).copied()
    }

    pub(super) fn set_controller_role(
        &mut self,
        controller: ControllerId,
        role: ControllerRole,
    ) -> bool {
        match self.controller_roles.get_mut(&controller) {
            Some(current) => {
                if *current == role {
                    false
                } else {
                    *current = role;
                    true
                }
            }
            None => {
                self.controller_roles.insert(controller, role);
                self.targets_by_controller.entry(controller).or_default();
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
            .unwrap_or(OwnerState::NoOwner)
    }

    pub(super) fn resource_owner_by_key(&self, key: ResourceOwnerKey) -> OwnerState {
        self.resource_owner_by_key
            .get(&key)
            .copied()
            .unwrap_or(OwnerState::NoOwner)
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

    pub(super) fn owned_targets(&self, controller: ControllerId) -> Vec<OwnershipTarget> {
        self.targets_by_controller
            .get(&controller)
            .map(|targets| targets.iter().copied().collect())
            .unwrap_or_default()
    }

    pub(super) fn owned_entities(&self, controller: ControllerId) -> Vec<Entity> {
        self.owned_targets(controller)
            .into_iter()
            .filter_map(|target| match target {
                OwnershipTarget::Entity(entity) => Some(entity),
                OwnershipTarget::Resource(_) => None,
            })
            .collect()
    }

    pub(super) fn owned_resources(&self, controller: ControllerId) -> Vec<ResourceOwnerKey> {
        self.owned_targets(controller)
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

        self.remove_controller_target(previous, target);

        match target {
            OwnershipTarget::Entity(entity) => {
                if next == OwnerState::NoOwner {
                    self.entity_owner.remove(&entity);
                } else {
                    self.entity_owner.insert(entity, next);
                }
            }
            OwnershipTarget::Resource(key) => {
                if next == OwnerState::NoOwner {
                    self.resource_owner_by_key.remove(&key);
                } else {
                    self.resource_owner_by_key.insert(key, next);
                }
            }
        }

        self.add_controller_target(next, target);

        self.next_transfer_sequence = self.next_transfer_sequence.saturating_add(1);
        self.transfer_log.push(OwnershipTransferRecord {
            sequence: self.next_transfer_sequence,
            target,
            previous,
            next,
        });
        true
    }

    fn remove_controller_target(&mut self, owner: OwnerState, target: OwnershipTarget) {
        let OwnerState::ControllerOwned(controller) = owner else {
            return;
        };

        if let Some(targets) = self.targets_by_controller.get_mut(&controller) {
            targets.remove(&target);
        }
    }

    fn add_controller_target(&mut self, owner: OwnerState, target: OwnershipTarget) {
        let OwnerState::ControllerOwned(controller) = owner else {
            return;
        };

        self.targets_by_controller
            .entry(controller)
            .or_default()
            .insert(target);
    }
}

impl World {
    pub fn create_controller(&mut self, role: ControllerRole) -> ControllerId {
        self.ownership.create_controller(role)
    }

    pub fn set_controller_role(&mut self, controller: ControllerId, role: ControllerRole) -> bool {
        self.ownership.set_controller_role(controller, role)
    }

    pub fn controller_role(&self, controller: ControllerId) -> Option<ControllerRole> {
        self.ownership.controller_role(controller)
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
            return OwnerState::NoOwner;
        };
        self.ownership.resource_owner_by_key(key)
    }

    pub fn resource_owner_by_key(&self, key: ResourceOwnerKey) -> OwnerState {
        self.ownership.resource_owner_by_key(key)
    }

    pub fn resource_owner_by_type_id(&self, resource_type: TypeId) -> OwnerState {
        let Some(key) = self.ownership.resource_key(resource_type) else {
            return OwnerState::NoOwner;
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

    pub fn owned_targets(&self, controller: ControllerId) -> Vec<OwnershipTarget> {
        self.ownership.owned_targets(controller)
    }

    pub fn owned_entities(&self, controller: ControllerId) -> Vec<Entity> {
        self.ownership.owned_entities(controller)
    }

    pub fn owned_resources(&self, controller: ControllerId) -> Vec<ResourceOwnerKey> {
        self.ownership.owned_resources(controller)
    }

    pub fn transfer_controller_targets_to_server(&mut self, controller: ControllerId) -> usize {
        let targets = self.ownership.owned_targets(controller);
        let mut transferred = 0usize;
        for target in targets {
            let changed = match target {
                OwnershipTarget::Entity(entity) => self
                    .ownership
                    .assign_entity_owner(entity, OwnerState::ServerOwned),
                OwnershipTarget::Resource(resource) => self
                    .ownership
                    .assign_resource_owner_by_key(resource, OwnerState::ServerOwned),
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
