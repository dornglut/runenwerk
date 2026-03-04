use anyhow::Result;
use cavern_hunt::{CavernHuntPlugin, CavernHuntServerPlugin};
use engine::plugins::{
    NetworkRuntimeHandle, NetworkServerPlugin, PredictionPlugin, RenderPlugin, ReplicationPlugin,
    ScenePlugin, UiInputPlugin, UiRenderPlugin, default_plugins,
};
use engine::{App, AppRunner, AuthorityRole, SimulationProfile};
use engine_net::{ProtocolVersion, ServerSessionConfig, Transport};
use engine_net_quic::{QuicServerJoinVerifier, QuicTransport, default_client_bind_addr};
use grotto_online::{AxiomHttpClient, AxiomJoinGrantVerifier};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

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
    let mut app = App::headless();
    app.set_title("Cavern Hunt Dedicated Server");
    app.set_simulation_profile(SimulationProfile::DedicatedAuthority);
    app.set_authority_role(AuthorityRole::Server);
    app.add_plugins(default_plugins());
    app.add_plugins((
        ScenePlugin,
        UiInputPlugin,
        UiRenderPlugin,
        RenderPlugin,
        NetworkServerPlugin,
        ReplicationPlugin,
        PredictionPlugin,
    ));
    app.add_plugins((CavernHuntPlugin, CavernHuntServerPlugin));

    let protocol = ProtocolVersion::new(1, 1, 1);
    let session_config = ServerSessionConfig {
        server_id: "srv-local".to_string(),
        protocol,
        tick_rate_hz: 60,
    };
    app.world_mut().insert_resource(session_config.clone());
    let axiom_api_base_url =
        std::env::var("AXIOM_API_BASE_URL").unwrap_or_else(|_| "http://api.localhost".to_string());
    let server_secret = std::env::var("DEDICATED_SERVER_SHARED_SECRET").ok();

    let transport = QuicTransport::default();
    let verifier = if let Some(secret) = server_secret.filter(|value| !value.trim().is_empty()) {
        Some(Arc::new(AxiomJoinGrantVerifier::new(
            AxiomHttpClient::new(axiom_api_base_url)?,
            secret,
        )) as Arc<dyn QuicServerJoinVerifier>)
    } else {
        None
    };
    let server_runtime = transport.spawn_server_runtime_with_verifier(
        default_client_bind_addr(),
        "localhost",
        session_config,
        verifier,
    )?;
    let cert_path = write_dev_certificate(&server_runtime.certificate)?;
    let local_addr = server_runtime.local_addr;
    let fingerprint = server_runtime.certificate_fingerprint_sha256.clone();
    let server_name = server_runtime.server_name.clone();
    let (command_tx, event_rx) = server_runtime.into_channels();
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
        "grotto_server live profile={:?} authority={:?} protocol={protocol:?} transport={:?} endpoint={} server_name={} cert_path={} cert_fingerprint_sha256={} tick_rate_hz=60",
        SimulationProfile::DedicatedAuthority,
        AuthorityRole::Server,
        transport.kind(),
        local_addr,
        server_name,
        cert_path.display(),
        fingerprint,
    );

    app.run()?;
    Ok(())
}

fn write_dev_certificate(certificate: impl AsRef<[u8]>) -> Result<PathBuf> {
    let path = PathBuf::from("var/dev/server-cert.der");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, certificate.as_ref())?;
    Ok(path)
}
