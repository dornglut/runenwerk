//! Domain-owned UI story proof contracts.

pub mod cli_v2;
pub mod diagnostic;
pub mod evidence;
pub mod fixtures_v2;
pub mod identity;
pub mod manifest_v2;
pub mod mount_v2;
pub mod registry_v2;
pub mod report_v2;
pub mod run_v2;
pub mod workflow;

pub use cli_v2::*;
pub use diagnostic::*;
pub use evidence::*;
pub use fixtures_v2::*;
pub use identity::*;
pub use manifest_v2::*;
pub use mount_v2::*;
pub use registry_v2::*;
pub use report_v2::*;
pub use run_v2::*;
pub use workflow::*;
