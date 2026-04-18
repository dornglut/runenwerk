//! File: domain/editor/editor_core/src/capability.rs
//! Purpose: Capability tokens for enforcing runtime boundary access.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanRatify {
    _sealed: SealedCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanMutateAuthored {
    _sealed: SealedCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanMutateSimulated {
    _sealed: SealedCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanObserveFrame {
    _sealed: SealedCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CanShareSession {
    _sealed: SealedCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SealedCapability;

impl CanRatify {
    pub(crate) const fn issue() -> Self {
        Self {
            _sealed: SealedCapability,
        }
    }
}

impl CanMutateAuthored {
    pub(crate) const fn issue() -> Self {
        Self {
            _sealed: SealedCapability,
        }
    }
}

impl CanMutateSimulated {
    pub(crate) const fn issue() -> Self {
        Self {
            _sealed: SealedCapability,
        }
    }
}

impl CanObserveFrame {
    pub(crate) const fn issue() -> Self {
        Self {
            _sealed: SealedCapability,
        }
    }
}

impl CanShareSession {
    pub(crate) const fn issue() -> Self {
        Self {
            _sealed: SealedCapability,
        }
    }
}
