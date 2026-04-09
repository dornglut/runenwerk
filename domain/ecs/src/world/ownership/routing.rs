use super::model::{OwnerId, OwnerRole, OwnershipTarget, ResourceOwnerKey};
use crate::entity::Entity;
use crate::world::world::World;

impl World {
    pub fn route_owner_entities(&self, owner: OwnerId) -> Vec<Entity> {
        if !matches!(self.owner_role(owner), Some(OwnerRole::Active)) {
            return Vec::new();
        }
        self.owned_entities(owner)
    }

    pub fn route_owner_resources(&self, owner: OwnerId) -> Vec<ResourceOwnerKey> {
        if !matches!(self.owner_role(owner), Some(OwnerRole::Active)) {
            return Vec::new();
        }
        self.owned_resources(owner)
    }

    pub fn route_owner_targets(&self, owner: OwnerId) -> Vec<OwnershipTarget> {
        if !matches!(self.owner_role(owner), Some(OwnerRole::Active)) {
            return Vec::new();
        }
        self.owned_targets(owner)
    }

    pub fn is_entity_owned_by_owner(&self, entity: Entity, owner: OwnerId) -> bool {
        self.route_owner_entities(owner).contains(&entity)
    }

    pub fn owner_owned_target_count(&self, owner: OwnerId) -> usize {
        self.route_owner_targets(owner).len()
    }
}
