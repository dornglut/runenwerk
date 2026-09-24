use runen_net::input::{AuthorityInputAggregateLimits, AuthorityInputLimits};
use runen_net::replication::{
    ClientAggregateLimits, ReplicationLineageKey, ReplicationRetentionLimits,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NetRole {
    Client,
    Server,
    Host,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct AuthorityInputPolicy {
    participant_limits: AuthorityInputLimits,
    aggregate_limits: AuthorityInputAggregateLimits,
}

impl AuthorityInputPolicy {
    pub const fn new(
        participant_limits: AuthorityInputLimits,
        aggregate_limits: AuthorityInputAggregateLimits,
    ) -> Self {
        Self {
            participant_limits,
            aggregate_limits,
        }
    }

    pub const fn participant_limits(self) -> AuthorityInputLimits {
        self.participant_limits
    }

    pub const fn aggregate_limits(self) -> AuthorityInputAggregateLimits {
        self.aggregate_limits
    }

    pub const fn max_future_tick_distance(self) -> u64 {
        self.participant_limits.max_future_tick_distance()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ClientReplicationPolicy {
    lineage: ReplicationLineageKey,
    aggregate_limits: ClientAggregateLimits,
    retention_limits: ReplicationRetentionLimits,
}

impl ClientReplicationPolicy {
    pub const fn new(
        lineage: ReplicationLineageKey,
        aggregate_limits: ClientAggregateLimits,
        retention_limits: ReplicationRetentionLimits,
    ) -> Self {
        Self {
            lineage,
            aggregate_limits,
            retention_limits,
        }
    }

    pub const fn lineage(self) -> ReplicationLineageKey {
        self.lineage
    }

    pub const fn aggregate_limits(self) -> ClientAggregateLimits {
        self.aggregate_limits
    }

    pub const fn retention_limits(self) -> ReplicationRetentionLimits {
        self.retention_limits
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NetPluginConfig {
    pub enable_diagnostics: bool,
    pub client_replication: Option<ClientReplicationPolicy>,
}

impl NetPluginConfig {
    pub const fn with_client_replication_policy(mut self, policy: ClientReplicationPolicy) -> Self {
        self.client_replication = Some(policy);
        self
    }
}
