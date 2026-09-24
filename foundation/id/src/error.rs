use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRawId {
    pub raw: u64,
}

impl InvalidRawId {
    pub const fn new(raw: u64) -> Self {
        Self { raw }
    }
}

impl fmt::Display for InvalidRawId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid raw id value: {}", self.raw)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidRawId {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    Exhausted,
    InvalidStart { start_at: u64 },
}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationError::Exhausted => f.write_str("allocator exhausted"),
            AllocationError::InvalidStart { start_at } => {
                write!(f, "invalid allocator start value: {}", start_at)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AllocationError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeError {
    UnknownSlot {
        slot: u32,
    },
    NotLive {
        slot: u32,
    },
    StaleGeneration {
        slot: u32,
        expected_generation: u32,
        provided_generation: u32,
    },
}

impl fmt::Display for FreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FreeError::UnknownSlot { slot } => write!(f, "unknown slot: {}", slot),
            FreeError::NotLive { slot } => write!(f, "slot is not live: {}", slot),
            FreeError::StaleGeneration {
                slot,
                expected_generation,
                provided_generation,
            } => write!(
                f,
                "stale generation for slot {} (expected {}, got {})",
                slot, expected_generation, provided_generation
            ),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for FreeError {}
