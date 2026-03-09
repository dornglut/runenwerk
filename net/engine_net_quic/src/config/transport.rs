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
