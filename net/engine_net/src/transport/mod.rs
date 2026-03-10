pub mod lanes;
pub mod semantics;
use serde::{Deserialize, Serialize};

// src/transport.rs

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ConnectionId(pub u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportKind {
    Quic,
}

pub trait Transport: Send + Sync {
    fn kind(&self) -> TransportKind;
}

pub use lanes::*;
pub use semantics::*;
