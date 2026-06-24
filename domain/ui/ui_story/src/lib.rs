//! Domain-owned UI story proof contracts.

pub mod cli;
pub mod diagnostic;
pub mod evidence;
pub mod gallery;
pub mod identity;
pub mod manifest;
pub mod manifest_v2;
pub mod mount;
pub mod proof;
pub mod registry;
pub mod registry_v2;
pub mod report;
pub mod run_v2;
pub mod runner;
pub mod workflow;

pub use cli::*;
pub use diagnostic::*;
pub use evidence::*;
pub use gallery::*;
pub use identity::*;
pub use manifest::{
    UiStoryCategory, UiStoryCompatibilityPolicy, UiStoryExpectedOutcome, UiStoryExpectedVerdict,
    UiStoryHostInput, UiStoryHostInputValue, UiStoryHostKind, UiStoryHostProfile,
    UiStoryManifest, UiStoryManifestDiagnostic, UiStoryManifestParseError,
    UiStoryMigrationPolicy, UiStoryMountPolicy, UiStoryRoutePolicy, UiStorySource,
    UiStorySourceKind, UiStoryThemeProfile, UiStoryViewportProfile,
};
pub use manifest_v2::*;
pub use mount::*;
pub use proof::*;
pub use registry::*;
pub use registry_v2::*;
pub use report::{
    UiStoryRunReport, UiStoryStageKind, UiStoryStageReport, UiStoryStageStatus, UiStoryVerdict,
    UiStoryVerdictStatus,
};
pub use run_v2::*;
pub use runner::*;
pub use workflow::*;
