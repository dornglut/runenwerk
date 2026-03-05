use anyhow::{Context, Result};
use cavern_hunt::domain::{NetConfigHotReloadState, load_client_network_config_from_path};
use cavern_hunt::{CavernHuntClientPlugin, CavernHuntPlugin};
use engine::plugins::render::domain::ShaderRegistryResource;
use engine::plugins::{
    NetworkClientPlugin, NetworkRuntimeHandle, RenderPlugin, ScenePlugin, UiInputPlugin,
    UiRenderPlugin, default_plugins,
};
use engine::{App, AppRunner, AuthorityRole, SimulationProfile};
use engine_net::{ProtocolVersion, Transport};
use engine_net_quic::{
    QuicClientTargetProvider, QuicTransport, QuicTrustPolicy, certificate_fingerprint_sha256,
    default_client_bind_addr,
};
use grotto_online::{AxiomAuthState, AxiomHttpClient, AxiomJoinGrantProvider, JoinGrant};
use rustls::pki_types::CertificateDer;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

const DEFAULT_CLIENT_CONFIG_PATH: &str = "game/assets/networking/client/local_dev.ron";

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
    let launch = parse_client_launch_options()?;
    let config = load_client_network_config_from_path(&launch.config_path).with_context(|| {
        format!(
            "failed loading client config {}",
            launch.config_path.display()
        )
    })?;

    let _tracing_guard = engine::utils::setup_tracing();
    let mut app = App::new();
    app.set_title("Cavern Hunt Client");
    app.set_simulation_profile(SimulationProfile::DedicatedAuthority);
    app.set_authority_role(AuthorityRole::Client);
    app.add_plugins(default_plugins());
    app.add_plugins((
        ScenePlugin,
        UiInputPlugin,
        UiRenderPlugin,
        RenderPlugin,
        NetworkClientPlugin,
    ));
    app.add_plugins((CavernHuntPlugin, CavernHuntClientPlugin));
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

    let protocol = ProtocolVersion::from(&config.protocol);
    let transport = QuicTransport::default();
    let server_id = config.server_id.clone();
    let server_name = config.server_name.clone();
    let server_endpoint = config.server_endpoint.clone();
    let ticket = config.join_ticket.clone();
    let cert_path = PathBuf::from(&config.cert_path);
    let cert_bytes = std::fs::read(&cert_path).with_context(|| {
        format!(
            "failed to read server certificate from {}",
            cert_path.display()
        )
    })?;
    let trusted_certificate = CertificateDer::from(cert_bytes);
    let fingerprint = config
        .cert_fingerprint_sha256
        .clone()
        .unwrap_or_else(|| certificate_fingerprint_sha256(&trusted_certificate));

    let maybe_axiom_provider = if config.use_axiom_handoff {
        if let Some(lobby_id) = &config.axiom_lobby_id {
            let api = AxiomHttpClient::new(config.axiom_api_base_url.clone())?;
            Some(Arc::new(AxiomJoinGrantProvider::new(
                api,
                AxiomAuthState::new(
                    config.axiom_device_id.clone(),
                    config.axiom_access_token.clone(),
                    config.axiom_refresh_token.clone(),
                ),
                lobby_id.clone(),
                Some(fingerprint.clone()),
            )) as Arc<dyn QuicClientTargetProvider>)
        } else {
            eprintln!(
                "client config requested Axiom handoff but axiom_lobby_id is missing; using local join"
            );
            None
        }
    } else {
        None
    };
    let using_axiom_handoff = maybe_axiom_provider.is_some();

    let fallback_target = JoinGrant {
        server_id: server_id.clone(),
        server_endpoint: server_endpoint.clone(),
        transport_kind: transport.kind(),
        protocol_version: protocol,
        server_cert_fingerprint_sha256: fingerprint.clone(),
        ticket,
    }
    .into_client_session_target(&server_id, protocol)?;
    let target = if let Some(provider) = &maybe_axiom_provider {
        provider.refresh_target(&fallback_target).await?
    } else {
        fallback_target
    };
    let target_endpoint = target.server_endpoint.clone();
    let trust_policy = QuicTrustPolicy::PinnedServer {
        expected_fingerprint_sha256: target.server_cert_fingerprint_sha256.clone(),
        trusted_certificates: vec![trusted_certificate],
    };
    let runtime = if let Some(provider) = maybe_axiom_provider {
        transport.spawn_client_runtime_with_provider(
            default_client_bind_addr(),
            &server_name,
            target,
            trust_policy,
            Some(provider),
        )?
    } else {
        transport.spawn_client_runtime(
            default_client_bind_addr(),
            &server_name,
            target,
            trust_policy,
        )?
    };
    let (command_tx, event_rx) = runtime.into_channels();
    app.world_mut()
        .insert_resource(NetworkRuntimeHandle::new(command_tx, event_rx));

    let running = Arc::new(AtomicBool::new(true));
    let shutdown = running.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.store(false, Ordering::SeqCst);
    });
    app.set_runner(SignalRunner { running });

    println!(
        "grotto_client live profile={:?} authority={:?} protocol={protocol:?} transport={:?} join_mode={} target={} server_name={} cert_path={} cert_fingerprint_sha256={} config={}",
        SimulationProfile::DedicatedAuthority,
        AuthorityRole::Client,
        transport.kind(),
        if using_axiom_handoff {
            "axiom"
        } else {
            "local"
        },
        target_endpoint,
        server_name,
        cert_path.display(),
        fingerprint,
        launch.config_path.display(),
    );

    app.run()?;
    Ok(())
}

struct ClientLaunchOptions {
    config_path: PathBuf,
}

fn parse_client_launch_options() -> Result<ClientLaunchOptions> {
    let mut config_path = PathBuf::from(DEFAULT_CLIENT_CONFIG_PATH);
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--config" => {
                let Some(value) = args.next() else {
                    anyhow::bail!("missing value for --config");
                };
                config_path = PathBuf::from(value);
            }
            "--help" | "-h" => {
                println!("Usage: grotto_client [--config <path>]");
                std::process::exit(0);
            }
            unknown => anyhow::bail!("unknown argument '{unknown}'"),
        }
    }
    Ok(ClientLaunchOptions { config_path })
}
