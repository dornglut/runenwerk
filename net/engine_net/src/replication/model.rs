use engine_sim::NetEntityId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct Replicated;

pub trait Replicate:
    serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static
{
}

impl<T> Replicate for T where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static
{
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
    use super::{NetEntityMap, NetEntityMapEvent};

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
}
