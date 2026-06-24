//! Domain-owned UI story proof contracts.

pub mod cli;
pub mod diagnostic;
pub mod gallery;
pub mod identity;
pub mod manifest;
pub mod mount;
pub mod proof;
pub mod registry;
pub mod report;
pub mod runner;

pub use cli::*;
pub use diagnostic::*;
pub use gallery::*;
pub use identity::*;
pub use manifest::{
    UiStoryCategory, UiStoryCompatibilityPolicy, UiStoryDiagnosticExpectation,
    UiStoryExpectedOutcome, UiStoryExpectedVerdict, UiStoryHostInput, UiStoryHostInputValue,
    UiStoryHostKind, UiStoryHostProfile, UiStoryManifest, UiStoryManifestDiagnostic,
    UiStoryManifestParseError, UiStoryMigrationPolicy, UiStoryMountPolicy, UiStoryRoutePolicy,
    UiStorySource, UiStorySourceKind, UiStoryThemeProfile, UiStoryViewportProfile,
};
pub use mount::*;
pub use proof::*;
pub use registry::*;
pub use report::{
    UiStoryRunReport, UiStoryStageKind, UiStoryStageReport, UiStoryStageStatus, UiStoryVerdict,
    UiStoryVerdictStatus,
};
pub use runner::*;
