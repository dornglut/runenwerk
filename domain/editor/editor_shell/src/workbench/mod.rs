//! File: domain/editor/editor_shell/src/workbench/mod.rs
//! Purpose: App-neutral Workbench composition manifests and compiler.

pub mod compiled;
pub mod compiler;
pub mod diagnostics;
pub mod manifest;

pub use compiled::{CompiledWorkbenchComposition, CompiledWorkbenchCompositionParts};
pub use compiler::{WorkbenchCompositionCompilerInput, compile_workbench_composition};
pub use diagnostics::WorkbenchCompositionCompileError;
pub use manifest::{
    AuthoredWorkbenchCompositionManifestError, AuthoredWorkspaceProfileManifestError,
    ToolSuiteManifest, WorkbenchCompositionManifest, WorkspaceProfileLayoutSource,
    WorkspaceProfileManifest, workbench_composition_manifest_from_definition,
    workspace_profile_manifest_from_definition,
    workspace_profile_manifests_from_authored_documents,
};
