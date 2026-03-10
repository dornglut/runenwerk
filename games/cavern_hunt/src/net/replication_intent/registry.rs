use super::components::{HealthReplicated, PlayerInputReplicated, PlayerStateReplicated};
use super::policy::CavernReplicationPolicy;
use engine_net::replication::{ReplicatedComponentDescriptor, ReplicationRegistry};

#[derive(Debug, Clone)]
pub struct CavernReplicationIntent {
    pub policy: CavernReplicationPolicy,
    pub registry: ReplicationRegistry,
}

impl Default for CavernReplicationIntent {
    fn default() -> Self {
        Self {
            policy: CavernReplicationPolicy::default(),
            registry: build_cavern_replication_registry(),
        }
    }
}

impl CavernReplicationIntent {
    pub fn descriptor(&self, component_name: &str) -> Option<&ReplicatedComponentDescriptor> {
        self.registry.descriptor(component_name)
    }
}

pub fn build_cavern_replication_registry() -> ReplicationRegistry {
    let mut registry = ReplicationRegistry::default();
    registry.register_component::<PlayerStateReplicated>();
    registry.register_component::<PlayerInputReplicated>();
    registry.register_component::<HealthReplicated>();
    registry
}
