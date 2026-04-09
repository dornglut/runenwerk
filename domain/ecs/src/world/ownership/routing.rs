use super::model::{ControllerId, ControllerRole, OwnershipTarget, ResourceOwnerKey};
use crate::entity::Entity;
use crate::world::world::World;

impl World {
    pub fn route_controller_entities(&self, controller: ControllerId) -> Vec<Entity> {
        if !matches!(
            self.controller_role(controller),
            Some(ControllerRole::Controller)
        ) {
            return Vec::new();
        }
        self.owned_entities(controller)
    }

    pub fn route_controller_resources(&self, controller: ControllerId) -> Vec<ResourceOwnerKey> {
        if !matches!(
            self.controller_role(controller),
            Some(ControllerRole::Controller)
        ) {
            return Vec::new();
        }
        self.owned_resources(controller)
    }

    pub fn route_controller_targets(&self, controller: ControllerId) -> Vec<OwnershipTarget> {
        if !matches!(
            self.controller_role(controller),
            Some(ControllerRole::Controller)
        ) {
            return Vec::new();
        }
        self.owned_targets(controller)
    }

    pub fn is_entity_owned_by_controller(&self, entity: Entity, controller: ControllerId) -> bool {
        self.route_controller_entities(controller).contains(&entity)
    }

    pub fn controller_owned_target_count(&self, controller: ControllerId) -> usize {
        self.route_controller_targets(controller).len()
    }
}
