use std::marker::PhantomData;

use engine_net::replication::{InputDriver, ReplicationDriver, SnapshotApplyDriver};

use crate::app::App;
use crate::plugin::Plugin;

use super::config::{NetPluginConfig, NetRole};
use super::resources::{
    NetworkClientPlugin, NetworkReplicationRuntimePlugin, NetworkServerPlugin, PredictionPlugin,
    ReplicationPlugin,
};

pub struct NetPlugin<TDriver> {
    pub role: NetRole,
    pub config: NetPluginConfig,
    _marker: PhantomData<TDriver>,
}

impl<TDriver> NetPlugin<TDriver> {
    pub fn client() -> Self {
        Self {
            role: NetRole::Client,
            config: NetPluginConfig::default(),
            _marker: PhantomData,
        }
    }

    pub fn server() -> Self {
        Self {
            role: NetRole::Server,
            config: NetPluginConfig::default(),
            _marker: PhantomData,
        }
    }

    pub fn host() -> Self {
        Self {
            role: NetRole::Host,
            config: NetPluginConfig::default(),
            _marker: PhantomData,
        }
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
                app.add_plugin(NetworkClientPlugin);
            }
            NetRole::Server => {
                app.add_plugin(NetworkServerPlugin);
            }
            NetRole::Host => {
                // Host mode is strict role composition: client + server in one process.
                app.add_plugin(NetworkServerPlugin);
                app.add_plugin(NetworkClientPlugin);
            }
        }

        app.add_plugin(NetworkReplicationRuntimePlugin::<TDriver>::default());
        app.add_plugin(ReplicationPlugin::<TDriver>::default());
        app.add_plugin(PredictionPlugin::<TDriver>::default());
    }
}

pub use super::resources::{
    NetworkClientPlugin as LegacyNetworkClientPlugin,
    NetworkReplicationRuntimePlugin as LegacyNetworkReplicationRuntimePlugin,
    NetworkServerPlugin as LegacyNetworkServerPlugin, PredictionPlugin as LegacyPredictionPlugin,
    ReplicationPlugin as LegacyReplicationPlugin,
};
