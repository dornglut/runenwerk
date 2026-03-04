use engine_net::{Transport, TransportKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuicTransportConfig {
    pub alpn_protocols: Vec<Vec<u8>>,
    pub max_concurrent_sessions: u32,
}

impl Default for QuicTransportConfig {
    fn default() -> Self {
        Self {
            alpn_protocols: vec![b"grottoq/1".to_vec()],
            max_concurrent_sessions: 256,
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuicTransport {
    config: QuicTransportConfig,
}

impl QuicTransport {
    pub fn new(config: QuicTransportConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &QuicTransportConfig {
        &self.config
    }

    pub fn build_transport_config(&self) -> quinn::TransportConfig {
        quinn::TransportConfig::default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quic_transport_defaults_to_quic_and_grotto_alpn() {
        let transport = QuicTransport::default();
        assert_eq!(transport.kind(), TransportKind::Quic);
        assert_eq!(
            transport.config().alpn_protocols,
            vec![b"grottoq/1".to_vec()]
        );
        let _config = transport.build_transport_config();
    }
}
