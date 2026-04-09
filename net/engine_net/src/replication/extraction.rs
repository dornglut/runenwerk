use ecs::{
    ChangeExtractionFilter, ChangeExtractionWindow, ComponentTypeKey, ControllerId, OwnerState,
    ResourceTypeKey, StructuralDeltaBatch,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default)]
pub struct ReplicationExtractionFilter {
    pub component_keys: Option<BTreeSet<ComponentTypeKey>>,
    pub resource_keys: Option<BTreeSet<ResourceTypeKey>>,
    pub include_no_owner: bool,
    pub include_server_owned: bool,
    pub allowed_controllers: Option<BTreeSet<ControllerId>>,
}

impl ReplicationExtractionFilter {
    pub fn allows_owner(&self, owner: OwnerState) -> bool {
        match owner {
            OwnerState::NoOwner => self.include_no_owner,
            OwnerState::ServerOwned => self.include_server_owned,
            OwnerState::ControllerOwned(controller) => self
                .allowed_controllers
                .as_ref()
                .is_none_or(|controllers| controllers.contains(&controller)),
        }
    }
}

pub fn extract_replication_deltas(
    world: &ecs::World,
    window: ChangeExtractionWindow,
    filter: &ReplicationExtractionFilter,
) -> StructuralDeltaBatch {
    let component_keys = filter.component_keys.as_ref();
    let resource_keys = filter.resource_keys.as_ref();

    let component_key_filter = |key: ComponentTypeKey| {
        component_keys
            .map(|keys| keys.contains(&key))
            .unwrap_or(true)
    };
    let resource_key_filter = |key: ResourceTypeKey| {
        resource_keys
            .map(|keys| keys.contains(&key))
            .unwrap_or(true)
    };
    let component_ownership_filter =
        |_: ecs::Entity, owner: OwnerState, _: ComponentTypeKey| filter.allows_owner(owner);
    let resource_ownership_filter =
        |_: ResourceTypeKey, owner: OwnerState| filter.allows_owner(owner);

    world.extract_structural_deltas(
        window,
        ChangeExtractionFilter {
            component_key_filter: Some(&component_key_filter),
            resource_key_filter: Some(&resource_key_filter),
            component_ownership_filter: Some(&component_ownership_filter),
            resource_ownership_filter: Some(&resource_ownership_filter),
            interest_filter: None,
        },
    )
}
