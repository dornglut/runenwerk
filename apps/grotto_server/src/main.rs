mod operator_control;

use anyhow::Result;
use cavern_hunt::{
    CavernHuntPlugin, CavernHuntServerPlugin, CavernReplicationDriver, NetConfigHotReloadState,
    ServerNetworkConfigAssetV1, load_server_network_config_from_path,
};
use engine::plugins::render::domain::ShaderRegistryResource;
use engine::plugins::{
    NetPlugin, NetworkRuntimeHandle, RenderPlugin, ScenePlugin, default_plugins,
};
use engine::{App, AppRunner, AuthorityRole, SimulationProfile};
use engine_net::{ProtocolVersion, ServerSessionConfig, Transport};
use engine_net_quic::{QuicServerJoinVerifier, QuicTransport};
use grotto_online::{AxiomHttpClient, AxiomJoinGrantVerifier};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

const DEFAULT_SERVER_CONFIG_PATH: &str = "game/assets/networking/server/local_dev.ron";

struct SignalRunner {
    running: Arc<AtomicBool>,
}

impl AppRunner for SignalRunner {
    fn next_frame(&mut self, _completed_frames: usize, _world: &engine::prelude::World) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    fn before_frame(&mut self, _world: &mut engine::prelude::World) {
        std::thread::sleep(Duration::from_millis(16));
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let launch = parse_server_launch_options()?;
    let mut config =
        load_server_network_config_from_path(&launch.config_path).map_err(|error| {
            anyhow::anyhow!(
                "failed loading server config {}: {error:#}",
                launch.config_path.display()
            )
        })?;
    apply_operator_runtime_overrides(&mut config, &launch)?;

    let _tracing_guard = engine::utils::setup_tracing();
    let mut app = App::headless();
    app.set_title("Cavern Hunt Dedicated Server");
    app.set_simulation_profile(SimulationProfile::DedicatedAuthority);
    app.set_authority_role(AuthorityRole::Server);
    app.add_plugins(default_plugins());
    app.add_plugins((
        ScenePlugin,
        RenderPlugin,
        NetPlugin::<CavernReplicationDriver>::server(),
    ));
    app.add_plugins((CavernHuntPlugin, CavernHuntServerPlugin));
    app.world_mut().insert_resource(config.clone());
    app.world_mut()
        .insert_resource(NetConfigHotReloadState::new(
            launch.config_path.clone(),
            config.hot_reload.enabled,
            config.hot_reload.poll_interval_seconds,
        ));
    if let Ok(mut shaders) = app.world_mut().resource_mut::<ShaderRegistryResource>() {
        shaders.set_watch_enabled(config.shader_watch);
    }

    let bind_addr = config.bind_endpoint.parse::<SocketAddr>()?;
    let protocol = ProtocolVersion::from(&config.protocol);
    let session_config = ServerSessionConfig {
        server_id: config.server_id.clone(),
        protocol,
        tick_rate_hz: config.tick_rate_hz,
    };
    app.world_mut().insert_resource(session_config.clone());
    let transport = QuicTransport::default();
    let verifier = if config.use_axiom_verifier {
        let secret = config
            .dedicated_server_shared_secret
            .clone()
            .filter(|value| !value.trim().is_empty());
        if let Some(secret) = secret {
            Some(Arc::new(AxiomJoinGrantVerifier::new(
                AxiomHttpClient::new(config.axiom_api_base_url.clone())?,
                secret,
            )) as Arc<dyn QuicServerJoinVerifier>)
        } else {
            if config.profile_id.starts_with("local") {
                eprintln!(
                    "server config enables Axiom verifier but dedicated_server_shared_secret is missing in local profile {}; using local verifier",
                    config.profile_id
                );
                None
            } else {
                anyhow::bail!(
                    "server config enables Axiom verifier but dedicated_server_shared_secret is missing for non-local profile {}",
                    config.profile_id
                );
            }
        }
    } else {
        None
    };
    let server_runtime = transport.spawn_server_runtime_with_verifier(
        bind_addr,
        &config.server_name,
        session_config,
        verifier,
    )?;
    let cert_path = write_dev_certificate(
        &server_runtime.certificate,
        PathBuf::from(&config.cert_output_path),
    )?;
    let local_addr = server_runtime.local_addr;
    let fingerprint = server_runtime.certificate_fingerprint_sha256.clone();
    let server_name = server_runtime.server_name.clone();
    let (command_tx, event_rx) = server_runtime.into_channels();
    app.world_mut()
        .insert_resource(NetworkRuntimeHandle::new(command_tx, event_rx));

    let running = Arc::new(AtomicBool::new(true));
    operator_control::try_install_operator_control(&mut app, running.clone(), &config)?;
    let shutdown = running.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.store(false, Ordering::SeqCst);
    });
    app.set_runner(SignalRunner { running });

    println!(
        "grotto_server live profile={:?} authority={:?} protocol={protocol:?} transport={:?} endpoint={} server_name={} cert_path={} cert_fingerprint_sha256={} tick_rate_hz={} config={}",
        SimulationProfile::DedicatedAuthority,
        AuthorityRole::Server,
        transport.kind(),
        local_addr,
        server_name,
        cert_path.display(),
        fingerprint,
        config.tick_rate_hz,
        launch.config_path.display(),
    );

    app.run()?;
    Ok(())
}

struct ServerLaunchOptions {
    config_path: PathBuf,
    operator_enabled_override: Option<bool>,
    operator_ws_url_override: Option<String>,
    operator_runtime_token_override: Option<String>,
    operator_heartbeat_seconds_override: Option<u64>,
    operator_snapshot_interval_ticks_override: Option<u64>,
    operator_max_buffered_events_override: Option<usize>,
}

fn parse_server_launch_options() -> Result<ServerLaunchOptions> {
    let mut config_path = PathBuf::from(DEFAULT_SERVER_CONFIG_PATH);
    let mut operator_enabled_override = None;
    let mut operator_ws_url_override = None;
    let mut operator_runtime_token_override = None;
    let mut operator_heartbeat_seconds_override = None;
    let mut operator_snapshot_interval_ticks_override = None;
    let mut operator_max_buffered_events_override = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --config");
                };
                config_path = PathBuf::from(value);
            }
            "--axiom-operator-enabled" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --axiom-operator-enabled");
                };
                operator_enabled_override =
                    Some(parse_bool_flag("--axiom-operator-enabled", value.as_str())?);
            }
            "--axiom-operator-ws-url" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --axiom-operator-ws-url");
                };
                operator_ws_url_override = Some(value);
            }
            "--axiom-operator-runtime-token" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --axiom-operator-runtime-token");
                };
                operator_runtime_token_override = Some(value);
            }
            "--axiom-operator-heartbeat-seconds" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --axiom-operator-heartbeat-seconds");
                };
                operator_heartbeat_seconds_override = Some(parse_u64_flag(
                    "--axiom-operator-heartbeat-seconds",
                    value.as_str(),
                )?);
            }
            "--axiom-operator-snapshot-interval-ticks" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --axiom-operator-snapshot-interval-ticks");
                };
                operator_snapshot_interval_ticks_override = Some(parse_u64_flag(
                    "--axiom-operator-snapshot-interval-ticks",
                    value.as_str(),
                )?);
            }
            "--axiom-operator-max-buffered-events" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --axiom-operator-max-buffered-events");
                };
                operator_max_buffered_events_override = Some(parse_usize_flag(
                    "--axiom-operator-max-buffered-events",
                    value.as_str(),
                )?);
            }
            "--help" | "-h" => {
                println!(
                    "Usage: grotto_server [--config <path>] [--axiom-operator-enabled <bool>] [--axiom-operator-ws-url <url>] [--axiom-operator-runtime-token <token>]"
                );
                std::process::exit(0);
            }
            unknown => anyhow::bail!("unknown argument '{unknown}'"),
        }
    }
    Ok(ServerLaunchOptions {
        config_path,
        operator_enabled_override,
        operator_ws_url_override,
        operator_runtime_token_override,
        operator_heartbeat_seconds_override,
        operator_snapshot_interval_ticks_override,
        operator_max_buffered_events_override,
    })
}

fn write_dev_certificate(certificate: impl AsRef<[u8]>, path: PathBuf) -> Result<PathBuf> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, certificate.as_ref())?;
    Ok(path)
}

fn apply_operator_runtime_overrides(
    config: &mut ServerNetworkConfigAssetV1,
    launch: &ServerLaunchOptions,
) -> Result<()> {
    if let Some(enabled) = launch.operator_enabled_override {
        config.axiom_operator.enabled = enabled;
    }
    if let Some(ws_url) = &launch.operator_ws_url_override {
        config.axiom_operator.ws_url = ws_url.clone();
    }
    if let Some(runtime_token) = &launch.operator_runtime_token_override {
        config.axiom_operator.runtime_token = Some(runtime_token.clone());
    }
    if let Some(heartbeat_seconds) = launch.operator_heartbeat_seconds_override {
        config.axiom_operator.heartbeat_seconds = heartbeat_seconds.max(1);
    }
    if let Some(snapshot_interval_ticks) = launch.operator_snapshot_interval_ticks_override {
        config.axiom_operator.snapshot_interval_ticks = snapshot_interval_ticks.max(1);
    }
    if let Some(max_buffered_events) = launch.operator_max_buffered_events_override {
        config.axiom_operator.max_buffered_events = max_buffered_events.max(1);
    }

    if let Some(raw) = read_env("CAVERN_AXIOM_OPERATOR_ENABLED") {
        config.axiom_operator.enabled = parse_bool_flag("CAVERN_AXIOM_OPERATOR_ENABLED", &raw)?;
    }
    if let Some(raw) = read_env("CAVERN_AXIOM_OPERATOR_WS_URL") {
        config.axiom_operator.ws_url = raw;
    }
    if let Some(raw) = read_env("CAVERN_AXIOM_OPERATOR_RUNTIME_TOKEN") {
        config.axiom_operator.runtime_token = Some(raw);
    }
    if let Some(raw) = read_env("CAVERN_AXIOM_OPERATOR_HEARTBEAT_SECONDS") {
        config.axiom_operator.heartbeat_seconds =
            parse_u64_flag("CAVERN_AXIOM_OPERATOR_HEARTBEAT_SECONDS", &raw)?.max(1);
    }
    if let Some(raw) = read_env("CAVERN_AXIOM_OPERATOR_SNAPSHOT_INTERVAL_TICKS") {
        config.axiom_operator.snapshot_interval_ticks =
            parse_u64_flag("CAVERN_AXIOM_OPERATOR_SNAPSHOT_INTERVAL_TICKS", &raw)?.max(1);
    }
    if let Some(raw) = read_env("CAVERN_AXIOM_OPERATOR_MAX_BUFFERED_EVENTS") {
        config.axiom_operator.max_buffered_events =
            parse_usize_flag("CAVERN_AXIOM_OPERATOR_MAX_BUFFERED_EVENTS", &raw)?.max(1);
    }

    Ok(())
}

fn read_env(key: &str) -> Option<String> {
    let value = std::env::var(key).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn parse_bool_flag(name: &str, raw: &str) -> Result<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => anyhow::bail!("{name} expects one of: 1|0|true|false|yes|no|on|off"),
    }
}

fn parse_u64_flag(name: &str, raw: &str) -> Result<u64> {
    raw.trim()
        .parse::<u64>()
        .map_err(|error| anyhow::anyhow!("{name} expects an unsigned integer: {error}"))
}

fn parse_usize_flag(name: &str, raw: &str) -> Result<usize> {
    raw.trim()
        .parse::<usize>()
        .map_err(|error| anyhow::anyhow!("{name} expects an unsigned integer: {error}"))
}
