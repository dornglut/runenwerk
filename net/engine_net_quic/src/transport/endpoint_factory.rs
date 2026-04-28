use anyhow::Result;
use engine_net::{Transport, TransportKind};
use quinn::{ClientConfig, Endpoint, ServerConfig};
use rustls::RootCertStore;
use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::{QuicTransportConfig, certificate_fingerprint_sha256};

#[derive(Debug, Clone)]
pub struct QuicTransport {
    config: QuicTransportConfig,
}

#[derive(Debug, Clone)]
pub struct QuicServerEndpoint {
    pub endpoint: Endpoint,
    pub certificate: CertificateDer<'static>,
    pub certificate_fingerprint_sha256: String,
    pub server_name: String,
}

impl QuicTransport {
    pub fn new(config: QuicTransportConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &QuicTransportConfig {
        &self.config
    }

    pub fn build_transport_config(&self) -> quinn::TransportConfig {
        let mut config = quinn::TransportConfig::default();
        config.max_concurrent_bidi_streams(self.config.max_concurrent_sessions.into());
        config.datagram_receive_buffer_size(Some(64 * 1024));
        config.datagram_send_buffer_size(64 * 1024);
        config
    }

    pub fn bind_server_endpoint(&self, bind_addr: SocketAddr) -> Result<QuicServerEndpoint> {
        self.bind_server_endpoint_named(bind_addr, "localhost")
    }

    pub fn bind_server_endpoint_named(
        &self,
        bind_addr: SocketAddr,
        server_name: &str,
    ) -> Result<QuicServerEndpoint> {
        let cert = rcgen::generate_simple_self_signed(vec![server_name.to_string()])?;
        let cert_der = cert.cert.der().clone();
        let priv_key = PrivatePkcs8KeyDer::from(cert.key_pair.serialize_der());
        let mut server_config =
            ServerConfig::with_single_cert(vec![cert_der.clone()], priv_key.into())?;
        server_config.transport = Arc::new(self.build_transport_config());
        let endpoint = Endpoint::server(server_config, bind_addr)?;
        Ok(QuicServerEndpoint {
            endpoint,
            certificate_fingerprint_sha256: certificate_fingerprint_sha256(&cert_der),
            certificate: cert_der,
            server_name: server_name.to_string(),
        })
    }

    pub fn bind_client_endpoint(
        &self,
        bind_addr: SocketAddr,
        trusted_certificates: &[CertificateDer<'static>],
    ) -> Result<Endpoint> {
        let mut roots = RootCertStore::empty();
        for cert in trusted_certificates {
            roots.add(cert.clone())?;
        }
        let mut client_config = ClientConfig::with_root_certificates(Arc::new(roots))?;
        client_config.transport_config(Arc::new(self.build_transport_config()));
        let mut endpoint = Endpoint::client(bind_addr)?;
        endpoint.set_default_client_config(client_config);
        Ok(endpoint)
    }
}

impl Default for QuicTransport {
    fn default() -> Self {
        Self::new(QuicTransportConfig::default())
    }
}

impl Transport for QuicTransport {
    fn kind(&self) -> TransportKind {
        TransportKind::Quic
    }
}
