//! File: domain/editor/editor_shell/src/workbench/diagnostics.rs
//! Purpose: Workbench composition compiler diagnostics.

use std::fmt;

use crate::{
    ProfileRef, ProviderBundleError, ProviderFamilyId, SurfaceRef, ToolSuiteId,
    ToolSuiteRegistryError, ToolSurfaceStableKey, WorkspaceDefinitionFormationError,
    WorkspaceProfileId, WorkspaceProfileRegistryBackedBuildError,
};

#[derive(Debug)]
pub enum WorkbenchCompositionCompileError {
    DuplicateInstalledSuiteId {
        suite_id: ToolSuiteId,
    },
    UnknownInstalledSuiteId {
        suite_id: ToolSuiteId,
    },
    ToolSuiteRegistry(ToolSuiteRegistryError),
    DuplicateCompositionProfileRef {
        profile_ref: ProfileRef,
    },
    DuplicateProfileRef {
        profile_ref: ProfileRef,
    },
    DuplicateCompatibilityId {
        profile_id: WorkspaceProfileId,
    },
    UnknownCompositionProfileRef {
        profile_ref: ProfileRef,
    },
    DefaultProfileRefNotIncluded {
        profile_ref: ProfileRef,
    },
    UnknownProfileDefaultSurface {
        profile_ref: ProfileRef,
        surface_ref: SurfaceRef,
    },
    UnknownAuthoredLayoutSurface {
        profile_ref: ProfileRef,
        layout_ref: String,
        tab_id: String,
        stable_surface_key: ToolSurfaceStableKey,
    },
    UnknownAuthoredLegacySurface {
        profile_ref: ProfileRef,
        layout_ref: String,
        tab_id: String,
        authored_key: String,
    },
    UnmappedAuthoredLegacySurface {
        profile_ref: ProfileRef,
        layout_ref: String,
        tab_id: String,
    },
    ProviderBundle(ProviderBundleError),
    MissingProviderFamilyAssignment {
        provider_family_id: ProviderFamilyId,
    },
    WorkspaceProfileRegistry(WorkspaceProfileRegistryBackedBuildError),
    WorkspaceDefinitionFormation {
        profile_ref: ProfileRef,
        error: WorkspaceDefinitionFormationError,
    },
    TargetProfileCompatibility {
        profile_ref: ProfileRef,
        surface_key: ToolSurfaceStableKey,
    },
}

impl fmt::Display for WorkbenchCompositionCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateInstalledSuiteId { suite_id } => {
                write!(
                    f,
                    "Workbench composition installs suite `{suite_id}` more than once"
                )
            }
            Self::UnknownInstalledSuiteId { suite_id } => {
                write!(
                    f,
                    "Workbench composition references unknown suite `{suite_id}`"
                )
            }
            Self::ToolSuiteRegistry(error) => {
                write!(f, "Workbench suite registry validation failed: {error}")
            }
            Self::DuplicateCompositionProfileRef { profile_ref } => write!(
                f,
                "Workbench composition references profile `{profile_ref}` more than once"
            ),
            Self::DuplicateProfileRef { profile_ref } => {
                write!(f, "duplicate Workspace profile ref `{profile_ref}`")
            }
            Self::DuplicateCompatibilityId { profile_id } => write!(
                f,
                "duplicate Workspace profile compatibility id {}",
                profile_id.raw()
            ),
            Self::UnknownCompositionProfileRef { profile_ref } => write!(
                f,
                "Workbench composition references unknown profile `{profile_ref}`"
            ),
            Self::DefaultProfileRefNotIncluded { profile_ref } => write!(
                f,
                "Workbench default profile `{profile_ref}` is not included in the composition"
            ),
            Self::UnknownProfileDefaultSurface {
                profile_ref,
                surface_ref,
            } => write!(
                f,
                "Workspace profile `{profile_ref}` references unknown default surface `{surface_ref}`"
            ),
            Self::UnknownAuthoredLayoutSurface {
                profile_ref,
                layout_ref,
                tab_id,
                stable_surface_key,
            } => write!(
                f,
                "authored profile `{profile_ref}` layout `{layout_ref}` tab `{tab_id}` references unknown surface `{stable_surface_key}`"
            ),
            Self::UnknownAuthoredLegacySurface {
                profile_ref,
                layout_ref,
                tab_id,
                authored_key,
            } => write!(
                f,
                "authored profile `{profile_ref}` layout `{layout_ref}` tab `{tab_id}` references unknown legacy surface `{authored_key}`"
            ),
            Self::UnmappedAuthoredLegacySurface {
                profile_ref,
                layout_ref,
                tab_id,
            } => write!(
                f,
                "authored profile `{profile_ref}` layout `{layout_ref}` tab `{tab_id}` uses a legacy surface without a stable mapping"
            ),
            Self::ProviderBundle(error) => {
                write!(f, "Workbench provider bundle validation failed: {error}")
            }
            Self::MissingProviderFamilyAssignment { provider_family_id } => write!(
                f,
                "provider family `{provider_family_id}` has no concrete provider assignment"
            ),
            Self::WorkspaceProfileRegistry(error) => {
                write!(f, "Workbench profile registry validation failed: {error}")
            }
            Self::WorkspaceDefinitionFormation { profile_ref, error } => write!(
                f,
                "authored Workspace profile `{profile_ref}` failed layout formation: {error:?}"
            ),
            Self::TargetProfileCompatibility {
                profile_ref,
                surface_key,
            } => write!(
                f,
                "surface `{surface_key}` is not compatible with profile `{profile_ref}`"
            ),
        }
    }
}

impl std::error::Error for WorkbenchCompositionCompileError {}
