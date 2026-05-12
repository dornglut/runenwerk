use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductFreshness {
    Current,
    PotentiallyStale,
    Stale,
    Fallback,
    Missing,
    FailedPreserved,
    Retired,
    Rebuilding,
}

impl ProductFreshness {
    pub const fn is_strict_current(self) -> bool {
        matches!(self, Self::Current)
    }

    pub const fn is_failed_preserved(self) -> bool {
        matches!(self, Self::FailedPreserved)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductResidency {
    Resident,
    NonResident,
    PendingLoad,
    PendingUnload,
    Rebuilding,
    Stale,
    FallbackResident,
    GhostSummary,
    Missing,
    FailedPreserved,
    NotApplicable,
}

impl ProductResidency {
    pub const fn is_strict_available(self) -> bool {
        matches!(self, Self::Resident | Self::NotApplicable)
    }

    pub const fn is_fallback_or_ghost(self) -> bool {
        matches!(self, Self::FallbackResident | Self::GhostSummary)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductConsumerClass {
    Renderer,
    Physics,
    Ai,
    Simulation,
    Editor,
    Network,
    Tooling,
    Diagnostics,
    CollisionQuery,
    RuntimeRead,
    FamilySpecific,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductAuthorityClass {
    Authoritative,
    ServerValidated,
    DeterministicDerived,
    VisualOnly,
    DiagnosticOnly,
    LocalOnly,
}

impl ProductAuthorityClass {
    pub const fn can_satisfy_strict_query(self) -> bool {
        matches!(
            self,
            Self::Authoritative | Self::ServerValidated | Self::DeterministicDerived
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductRetentionPolicy {
    FrameLocal,
    SessionLocal,
    Cacheable,
    Persisted,
    RetainWhileReferenced,
    RebuildOnDemand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductRebuildPolicy {
    Immediate,
    Budgeted,
    Lazy,
    Idle,
    Manual,
    Offline,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductQueryPolicy {
    StrictCurrentOnly,
    CertifiedFallbackAllowed,
    VisualFallbackAllowed,
    DiagnosticOnly,
    LocalVisualOnly,
    OwnerCustom,
}

impl ProductQueryPolicy {
    pub const fn allows(
        self,
        freshness: ProductFreshness,
        residency: ProductResidency,
        authority: ProductAuthorityClass,
    ) -> bool {
        match self {
            Self::StrictCurrentOnly => {
                freshness.is_strict_current()
                    && residency.is_strict_available()
                    && authority.can_satisfy_strict_query()
            }
            Self::CertifiedFallbackAllowed => {
                !matches!(
                    freshness,
                    ProductFreshness::Missing | ProductFreshness::Retired
                ) && !matches!(
                    residency,
                    ProductResidency::Missing | ProductResidency::GhostSummary
                ) && authority.can_satisfy_strict_query()
            }
            Self::VisualFallbackAllowed => !matches!(
                freshness,
                ProductFreshness::Missing | ProductFreshness::Retired
            ),
            Self::DiagnosticOnly => {
                matches!(authority, ProductAuthorityClass::DiagnosticOnly)
                    || matches!(
                        freshness,
                        ProductFreshness::Missing | ProductFreshness::FailedPreserved
                    )
            }
            Self::LocalVisualOnly => matches!(
                authority,
                ProductAuthorityClass::LocalOnly | ProductAuthorityClass::VisualOnly
            ),
            Self::OwnerCustom => true,
        }
    }
}
