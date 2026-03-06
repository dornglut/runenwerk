// Owner: Grotto Server - Operator Control
use anyhow::Result;
use cavern_hunt::domain::ServerNetworkConfigAssetV1;
use engine::prelude::App;
use std::sync::{Arc, atomic::AtomicBool};

#[path = "operator_control/commands.rs"]
mod commands;
#[path = "operator_control/common.rs"]
mod common;
#[path = "operator_control/emit.rs"]
mod emit;
#[path = "operator_control/install.rs"]
mod install;
#[path = "operator_control/types.rs"]
mod types;

pub fn try_install_operator_control(
    app: &mut App,
    running: Arc<AtomicBool>,
    config: &ServerNetworkConfigAssetV1,
) -> Result<()> {
    install::try_install_operator_control(app, running, config)
}
