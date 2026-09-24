//! File: domain/editor/editor_shell/src/workbench/compiled.rs
//! Purpose: Validated Workbench composition output.

use crate::{
    HostCapabilityPolicy, ProfileRef, ProviderBundle, ProviderFamilyProviderMap, ToolSuiteRegistry,
    ToolSurfaceRegistry, WorkspaceProfileRegistry,
};

use super::manifest::WorkspaceProfileManifest;

#[derive(Debug, Clone)]
pub struct CompiledWorkbenchComposition {
    composition_ref: ProfileRef,
    label: String,
    tool_suite_registry: ToolSuiteRegistry,
    profiles: Vec<WorkspaceProfileManifest>,
    workspace_profile_registry: WorkspaceProfileRegistry,
    provider_bundle: ProviderBundle,
    provider_family_provider_map: ProviderFamilyProviderMap,
    host_policy: HostCapabilityPolicy,
}

#[derive(Debug, Clone)]
pub struct CompiledWorkbenchCompositionParts {
    pub composition_ref: ProfileRef,
    pub label: String,
    pub tool_suite_registry: ToolSuiteRegistry,
    pub profiles: Vec<WorkspaceProfileManifest>,
    pub workspace_profile_registry: WorkspaceProfileRegistry,
    pub provider_bundle: ProviderBundle,
    pub provider_family_provider_map: ProviderFamilyProviderMap,
    pub host_policy: HostCapabilityPolicy,
}

impl CompiledWorkbenchComposition {
    #[expect(
        clippy::too_many_arguments,
        reason = "compiled Workbench output carries independently validated registries"
    )]
    pub(crate) fn new(
        composition_ref: ProfileRef,
        label: String,
        tool_suite_registry: ToolSuiteRegistry,
        profiles: Vec<WorkspaceProfileManifest>,
        workspace_profile_registry: WorkspaceProfileRegistry,
        provider_bundle: ProviderBundle,
        provider_family_provider_map: ProviderFamilyProviderMap,
        host_policy: HostCapabilityPolicy,
    ) -> Self {
        Self {
            composition_ref,
            label,
            tool_suite_registry,
            profiles,
            workspace_profile_registry,
            provider_bundle,
            provider_family_provider_map,
            host_policy,
        }
    }

    pub fn composition_ref(&self) -> &ProfileRef {
        &self.composition_ref
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn tool_suite_registry(&self) -> &ToolSuiteRegistry {
        &self.tool_suite_registry
    }

    pub fn tool_surface_registry(&self) -> &ToolSurfaceRegistry {
        self.tool_suite_registry.surfaces()
    }

    pub fn profiles(&self) -> &[WorkspaceProfileManifest] {
        &self.profiles
    }

    pub fn workspace_profile_registry(&self) -> &WorkspaceProfileRegistry {
        &self.workspace_profile_registry
    }

    pub fn provider_bundle(&self) -> &ProviderBundle {
        &self.provider_bundle
    }

    pub fn provider_family_provider_map(&self) -> &ProviderFamilyProviderMap {
        &self.provider_family_provider_map
    }

    pub fn host_policy(&self) -> &HostCapabilityPolicy {
        &self.host_policy
    }

    pub fn into_parts(self) -> CompiledWorkbenchCompositionParts {
        CompiledWorkbenchCompositionParts {
            composition_ref: self.composition_ref,
            label: self.label,
            tool_suite_registry: self.tool_suite_registry,
            profiles: self.profiles,
            workspace_profile_registry: self.workspace_profile_registry,
            provider_bundle: self.provider_bundle,
            provider_family_provider_map: self.provider_family_provider_map,
            host_policy: self.host_policy,
        }
    }
}
