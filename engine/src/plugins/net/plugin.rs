use std::marker::PhantomData;

use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};
use engine_net::{AuthorityRole, SimulationProfile, SimulationProfileConfig};

use crate::app::App;
use crate::plugin::Plugin;
use crate::plugins::world::plugin::{WorldRuntimeConfig, world_runtime_mode_for_authority};

use super::config::{NetPluginConfig, NetRole};
use super::resources::{
    configure_client_role, configure_prediction, configure_replication, configure_runtime_bridge,
    configure_server_role,
};

pub struct NetPlugin<TDriver> {
    pub role: NetRole,
    pub config: NetPluginConfig,
    _marker: PhantomData<TDriver>,
}

impl<TDriver> NetPlugin<TDriver> {
    pub fn new(role: NetRole) -> Self {
        Self {
            role,
            config: NetPluginConfig::default(),
            _marker: PhantomData,
        }
    }

    pub fn client() -> Self {
        Self::new(NetRole::Client)
    }

    pub fn server() -> Self {
        Self::new(NetRole::Server)
    }

    pub fn host() -> Self {
        Self::new(NetRole::Host)
    }

    pub fn with_config(mut self, config: NetPluginConfig) -> Self {
        self.config = config;
        self
    }
}

impl<TDriver> Plugin for NetPlugin<TDriver>
where
    TDriver: ReplicationDriver + SnapshotApplyDriver + InputDriver + Send + Sync + 'static,
    TDriver::Snapshot: Clone + PartialEq,
    TDriver::Input: Clone + PartialEq,
{
    fn build(&self, app: &mut App) {
        match self.role {
            NetRole::Client => {
                configure_client_role(app);
                set_simulation_authority(app, AuthorityRole::Client);
            }
            NetRole::Server => {
                configure_server_role(app);
                set_simulation_authority(app, AuthorityRole::Server);
            }
            NetRole::Host => {
                // Host mode is strict role composition: client + server in one process.
                configure_server_role(app);
                configure_client_role(app);
                set_simulation_authority(app, AuthorityRole::Peer);
            }
        }

        configure_runtime_bridge::<TDriver>(app);
        configure_replication::<TDriver>(app);
        configure_prediction::<TDriver>(app);
    }
}

fn set_simulation_authority(app: &mut App, authority: AuthorityRole) {
    if let Ok(config) = app.world_mut().resource_mut::<SimulationProfileConfig>() {
        config.authority = authority;
        if matches!(config.profile, SimulationProfile::LocalSinglePlayer) {
            config.profile = SimulationProfile::DedicatedAuthority;
        }
    }
    if let Ok(runtime_config) = app.world_mut().resource_mut::<WorldRuntimeConfig>() {
        runtime_config.mode = world_runtime_mode_for_authority(authority);
    }
}
