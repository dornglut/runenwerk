use anyhow::{Context, Result};
use cavern_hunt::{CavernHuntClientPlugin, CavernHuntPlugin};
use engine::plugins::render::domain::ShaderRegistryResource;
use engine::plugins::{
    NetworkClientPlugin, NetworkRuntimeHandle, PredictionPlugin, RenderPlugin, ReplicationPlugin,
    ScenePlugin, UiInputPlugin, UiRenderPlugin, default_plugins,
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
        ReplicationPlugin,
        PredictionPlugin,
    ));
    app.add_plugins((CavernHuntPlugin, CavernHuntClientPlugin));
    if let Ok(mut shaders) = app.world_mut().resource_mut::<ShaderRegistryResource>() {
        let watch_enabled = std::env::var("GROTTO_SHADER_WATCH")
            .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        shaders.set_watch_enabled(watch_enabled);
    }

    let protocol = ProtocolVersion::new(1, 1, 1);
    let transport = QuicTransport::default();
    let server_id = std::env::var("GROTTO_SERVER_ID").unwrap_or_else(|_| "srv-local".to_string());
    let server_name =
        std::env::var("GROTTO_SERVER_NAME").unwrap_or_else(|_| "localhost".to_string());
    let server_endpoint =
        std::env::var("GROTTO_SERVER_ENDPOINT").unwrap_or_else(|_| "127.0.0.1:7000".to_string());
    let ticket = std::env::var("GROTTO_JOIN_TICKET").unwrap_or_else(|_| "local-ticket".to_string());
    let axiom_api_base_url =
        std::env::var("AXIOM_API_BASE_URL").unwrap_or_else(|_| "http://api.localhost".to_string());
    let axiom_lobby_id = std::env::var("AXIOM_LOBBY_ID").ok();
    let axiom_access_token = std::env::var("AXIOM_ACCESS_TOKEN").ok();
    let axiom_refresh_token = std::env::var("AXIOM_REFRESH_TOKEN").ok();
    let axiom_device_id =
        std::env::var("AXIOM_DEVICE_ID").unwrap_or_else(|_| "grotto-client-local".to_string());
    let cert_path = std::env::var("GROTTO_SERVER_CERT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("var/dev/server-cert.der"));
    let cert_bytes = std::fs::read(&cert_path).with_context(|| {
        format!(
            "failed to read server certificate from {}",
            cert_path.display()
        )
    })?;
    let trusted_certificate = CertificateDer::from(cert_bytes);
    let fingerprint = std::env::var("GROTTO_SERVER_CERT_FINGERPRINT")
        .unwrap_or_else(|_| certificate_fingerprint_sha256(&trusted_certificate));

    let maybe_axiom_provider = axiom_lobby_id
        .as_ref()
        .map(|lobby_id| {
            let api = AxiomHttpClient::new(axiom_api_base_url.clone())?;
            Ok::<_, anyhow::Error>(Arc::new(AxiomJoinGrantProvider::new(
                api,
                AxiomAuthState::new(
                    axiom_device_id.clone(),
                    axiom_access_token.clone(),
                    axiom_refresh_token.clone(),
                ),
                lobby_id.clone(),
                Some(fingerprint.clone()),
            )) as Arc<dyn QuicClientTargetProvider>)
        })
        .transpose()?;

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
        "grotto_client live profile={:?} authority={:?} protocol={protocol:?} transport={:?} target={} server_name={} cert_path={} cert_fingerprint_sha256={}",
        SimulationProfile::DedicatedAuthority,
        AuthorityRole::Client,
        transport.kind(),
        server_endpoint,
        server_name,
        cert_path.display(),
        fingerprint,
    );

    app.run()?;
    Ok(())
}
