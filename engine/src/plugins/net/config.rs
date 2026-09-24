use runen_net::input::{AuthorityInputAggregateLimits, AuthorityInputLimits};

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct NetPluginConfig {
    pub enable_diagnostics: bool,
}
