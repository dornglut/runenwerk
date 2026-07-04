//! Inert app effect plans.

use crate::action::AppActionCapability;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEffectPlan {
    NoEffect,
    Proposed(Vec<AppEffect>),
}

impl AppEffectPlan {
    pub fn is_no_effect(&self) -> bool {
        matches!(self, Self::NoEffect)
    }

    pub fn effect_count(&self) -> usize {
        match self {
            Self::NoEffect => 0,
            Self::Proposed(effects) => effects.len(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::NoEffect => "no_effect",
            Self::Proposed(_) => "proposed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEffect {
    HostCommandProposal {
        effect_id: String,
        required_capability: Option<AppActionCapability>,
        summary: String,
    },
    DomainCommandProposal {
        effect_id: String,
        domain_id: String,
        required_capability: Option<AppActionCapability>,
        summary: String,
    },
    FutureInertProposal {
        effect_id: String,
        kind: String,
        summary: String,
    },
}
