//! File: domain/ui/ui_testing/src/lib.rs
//! Crate: ui_testing

mod architecture_fixture;
mod assertions;
mod errors;
mod headless_fixture;
mod program_fixture;
mod reproducibility;

pub use architecture_fixture::UiArchitectureFixture;
pub use assertions::{DiagnosticAssertion, DiagnosticExpectation, SourceMapAssertion};
pub use errors::UiTestingError;
pub use headless_fixture::{HeadlessFixture, HeadlessFixtureRun};
pub use reproducibility::ReproducibilityAssertion;

#[cfg(test)]
mod tests;
