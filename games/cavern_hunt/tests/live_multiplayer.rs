use anyhow::{Result, bail};
use cavern_hunt::domain::{
    CavernMetaPersistenceConfig, CavernMetaProfile, CavernRunConfig, CavernRunPhase,
    CavernRunState, EnemyKind, ExtractionZone, Health, InventoryRunState, LocalPlayerRef,
    PlayerActive, PlayerId, Transform2,
};
use cavern_hunt::{
    CavernHuntClientPlugin, CavernHuntPlugin, CavernHuntServerPlugin, CavernReplicationDriver,
};
use engine::net::prelude::*;
use engine::plugins::net::NetworkRuntimeHandle;
use engine::plugins::{RenderPlugin, ScenePlugin, default_plugins};
use engine::{App, AuthorityRole, SimulationProfile};
use engine_net_quic::{QuicTransport, QuicTrustPolicy, default_client_bind_addr};
use rustls::pki_types::CertificateDer;
use std::time::Duration;

include!("live_multiplayer/helpers.rs");

include!("live_multiplayer/two_clients.rs");

include!("live_multiplayer/four_clients.rs");
