use crate::replication::interest::InterestPolicy;
use crate::replication::profile::{ReplicationDirection, ReplicationProfilePreset};
use engine_sim::NetEntityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Component)]
pub struct Replicated;

pub trait Replicate:
    serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static
{
}

impl<T> Replicate for T where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static
{
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuthorityModel {
    Server,
    Client,
}

pub trait NetEntity: Send + Sync + 'static {}

pub trait NetComponentMetadata: Send + Sync + 'static {
    fn replication_descriptor() -> ReplicatedComponentDescriptor;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicatedComponentDescriptor {
    pub component_name: String,
    pub authority: AuthorityModel,
    pub direction: ReplicationDirection,
    pub profile: ReplicationProfilePreset,
    pub interest: InterestPolicy,
    pub owner_prediction: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ReplicationRegistry {
    by_component: BTreeMap<String, ReplicatedComponentDescriptor>,
}

impl ReplicationRegistry {
    pub fn register(&mut self, descriptor: ReplicatedComponentDescriptor) {
        self.by_component
            .insert(descriptor.component_name.clone(), descriptor);
    }

    pub fn register_component<T: NetComponentMetadata>(&mut self) {
        self.register(T::replication_descriptor());
    }

    pub fn descriptor(&self, component_name: &str) -> Option<&ReplicatedComponentDescriptor> {
        self.by_component.get(component_name)
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &ReplicatedComponentDescriptor> {
        self.by_component.values()
    }

    pub fn len(&self) -> usize {
        self.by_component.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_component.is_empty()
    }
}

static NEXT_NET_ENTITY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetEntityMapEvent {
    Assigned {
        ecs_entity: u64,
        net_entity: NetEntityId,
    },
    Removed {
        ecs_entity: u64,
        net_entity: NetEntityId,
    },
}

#[derive(Debug, Clone, Default)]
pub struct NetEntityMap {
    ecs_to_net: BTreeMap<u64, NetEntityId>,
    net_to_ecs: BTreeMap<NetEntityId, u64>,
}

impl NetEntityMap {
    pub fn get_or_assign(&mut self, ecs_entity: u64) -> NetEntityId {
        self.get_or_assign_with_event(ecs_entity).0
    }

    pub fn get_or_assign_with_event(
        &mut self,
        ecs_entity: u64,
    ) -> (NetEntityId, Option<NetEntityMapEvent>) {
        if let Some(existing) = self.ecs_to_net.get(&ecs_entity).copied() {
            return (existing, None);
        }
        let id = NetEntityId(NEXT_NET_ENTITY_ID.fetch_add(1, Ordering::Relaxed));
        self.ecs_to_net.insert(ecs_entity, id);
        self.net_to_ecs.insert(id, ecs_entity);
        (
            id,
            Some(NetEntityMapEvent::Assigned {
                ecs_entity,
                net_entity: id,
            }),
        )
    }

    pub fn remove_by_ecs(&mut self, ecs_entity: u64) -> Option<NetEntityId> {
        self.remove_by_ecs_with_event(ecs_entity)
            .map(|event| match event {
                NetEntityMapEvent::Removed { net_entity, .. } => net_entity,
                NetEntityMapEvent::Assigned { net_entity, .. } => net_entity,
            })
    }

    pub fn remove_by_ecs_with_event(&mut self, ecs_entity: u64) -> Option<NetEntityMapEvent> {
        let id = self.ecs_to_net.remove(&ecs_entity)?;
        self.net_to_ecs.remove(&id);
        Some(NetEntityMapEvent::Removed {
            ecs_entity,
            net_entity: id,
        })
    }

    pub fn resolve_ecs(&self, net_entity: NetEntityId) -> Option<u64> {
        self.net_to_ecs.get(&net_entity).copied()
    }

    pub fn resolve_net(&self, ecs_entity: u64) -> Option<NetEntityId> {
        self.ecs_to_net.get(&ecs_entity).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuthorityModel, NetComponentMetadata, NetEntityMap, NetEntityMapEvent,
        ReplicatedComponentDescriptor, ReplicationRegistry,
    };
    use crate::replication::{InterestPolicy, ReplicationDirection, ReplicationProfilePreset};

    #[test]
    fn net_entity_map_round_trips_ids() {
        let mut map = NetEntityMap::default();
        let id = map.get_or_assign(42);
        assert_eq!(map.resolve_net(42), Some(id));
        assert_eq!(map.resolve_ecs(id), Some(42));
        assert_eq!(map.remove_by_ecs(42), Some(id));
        assert_eq!(map.resolve_ecs(id), None);
    }

    #[test]
    fn net_entity_map_emits_events_for_assignment_and_removal() {
        let mut map = NetEntityMap::default();
        let (assigned, assignment_event) = map.get_or_assign_with_event(9);
        assert!(matches!(
            assignment_event,
            Some(NetEntityMapEvent::Assigned {
                ecs_entity: 9,
                net_entity
            }) if net_entity == assigned
        ));
        let removal_event = map.remove_by_ecs_with_event(9);
        assert!(matches!(
            removal_event,
            Some(NetEntityMapEvent::Removed {
                ecs_entity: 9,
                net_entity
            }) if net_entity == assigned
        ));
    }

    #[test]
    fn registry_replaces_descriptor_by_component_name() {
        let mut registry = ReplicationRegistry::default();
        registry.register(ReplicatedComponentDescriptor {
            component_name: "PlayerState".to_string(),
            authority: AuthorityModel::Server,
            direction: ReplicationDirection::ServerToClient,
            profile: ReplicationProfilePreset::PredictedMovement,
            interest: InterestPolicy::Global,
            owner_prediction: true,
        });
        registry.register(ReplicatedComponentDescriptor {
            component_name: "PlayerState".to_string(),
            authority: AuthorityModel::Server,
            direction: ReplicationDirection::ServerToClient,
            profile: ReplicationProfilePreset::ReliableState,
            interest: InterestPolicy::OwnerOnly,
            owner_prediction: false,
        });
        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry
                .descriptor("PlayerState")
                .expect("descriptor exists")
                .profile,
            ReplicationProfilePreset::ReliableState
        );
    }

    #[test]
    fn registry_registers_component_through_metadata_trait() {
        struct Health;

        impl NetComponentMetadata for Health {
            fn replication_descriptor() -> ReplicatedComponentDescriptor {
                ReplicatedComponentDescriptor {
                    component_name: "Health".to_string(),
                    authority: AuthorityModel::Server,
                    direction: ReplicationDirection::ServerToClient,
                    profile: ReplicationProfilePreset::ReliableState,
                    interest: InterestPolicy::Global,
                    owner_prediction: false,
                }
            }
        }

        let mut registry = ReplicationRegistry::default();
        registry.register_component::<Health>();
        assert!(registry.descriptor("Health").is_some());
    }

    #[test]
    fn registry_registers_macro_annotated_component() {
        #[engine_net_macros::net_component(
            authority = Server,
            profile = PredictedMovement,
            interest = OwnerOnly,
            owner_prediction = true
        )]
        #[derive(Clone, serde::Serialize, serde::Deserialize)]
        struct PlayerState;

        let mut registry = ReplicationRegistry::default();
        registry.register_component::<PlayerState>();
        let descriptor = registry
            .descriptor("PlayerState")
            .expect("descriptor should be generated by macro");
        assert_eq!(descriptor.authority, AuthorityModel::Server);
        assert_eq!(
            descriptor.profile,
            ReplicationProfilePreset::PredictedMovement
        );
        assert_eq!(descriptor.interest, InterestPolicy::OwnerOnly);
        assert!(descriptor.owner_prediction);
    }

    #[test]
    fn net_entity_macro_implements_entity_trait() {
        #[engine_net_macros::net_entity]
        struct PlayerEntity;

        fn assert_net_entity<T: super::NetEntity>() {}
        assert_net_entity::<PlayerEntity>();
    }
}
