//! File: domain/editor/editor_shell/src/tool_suite/definition.rs
//! Purpose: App-neutral tool-suite surface declaration contracts.

use ui_surface::{SessionRetentionClass, SurfaceCapabilitySet};

use crate::PanelKind;

use super::{
    ProductCapabilityKey, ProfileRef, ProviderFamilyId, SuiteRef, SurfaceRef, ToolServiceKey,
    ToolSuiteId, ToolSurfaceStableKey,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorToolSuite {
    pub suite_id: ToolSuiteId,
    pub label: String,
    pub provider_families: Vec<ProviderFamilyDefinition>,
    pub surfaces: Vec<ToolSurfaceDefinition>,
}

impl EditorToolSuite {
    pub fn new(
        suite_ref: SuiteRef,
        label: impl Into<String>,
        provider_families: Vec<ProviderFamilyDefinition>,
        surfaces: Vec<ToolSurfaceDefinition>,
    ) -> Self {
        Self {
            suite_id: suite_ref.id().clone(),
            label: label.into(),
            provider_families,
            surfaces,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSuiteCapabilityDeclaration {
    pub suite_ref: SuiteRef,
    pub product_needs: Vec<ProductCapabilityNeed>,
    pub service_needs: Vec<ToolServiceNeed>,
}

impl ToolSuiteCapabilityDeclaration {
    pub fn new(
        suite_ref: SuiteRef,
        product_needs: Vec<ProductCapabilityNeed>,
        service_needs: Vec<ToolServiceNeed>,
    ) -> Self {
        Self {
            suite_ref,
            product_needs,
            service_needs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductCapabilityNeed {
    pub key: ProductCapabilityKey,
    pub label: String,
}

impl ProductCapabilityNeed {
    pub fn new(key: ProductCapabilityKey, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolServiceNeed {
    pub key: ToolServiceKey,
    pub label: String,
}

impl ToolServiceNeed {
    pub fn new(key: ToolServiceKey, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFamilyDefinition {
    pub id: ProviderFamilyId,
    pub label: String,
}

impl ProviderFamilyDefinition {
    pub fn new(id: ProviderFamilyId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSurfaceDefinition {
    pub key: ToolSurfaceStableKey,
    pub label: String,
    pub role: ToolSurfaceRole,
    pub panel_kind: PanelKind,
    pub provider_family: ProviderFamilyId,
    pub route: ToolSurfaceRoute,
    pub persistence: ToolSurfacePersistence,
    pub capabilities: SurfaceCapabilitySet,
    pub session_retention: SessionRetentionClass,
    pub creation_policy: ToolSurfaceCreationPolicy,
    pub target_profile_compatibility: ToolSurfaceTargetProfileCompatibility,
    pub legacy_compatibility_key: Option<String>,
}

impl ToolSurfaceDefinition {
    #[expect(
        clippy::too_many_arguments,
        reason = "surface registry metadata must be explicit at declaration sites"
    )]
    pub fn new(
        surface_ref: SurfaceRef,
        label: impl Into<String>,
        role: ToolSurfaceRole,
        panel_kind: PanelKind,
        provider_family: ProviderFamilyId,
        route: ToolSurfaceRoute,
        capabilities: SurfaceCapabilitySet,
        session_retention: SessionRetentionClass,
        creation_policy: ToolSurfaceCreationPolicy,
    ) -> Self {
        Self {
            key: surface_ref.key().clone(),
            label: label.into(),
            role,
            panel_kind,
            provider_family,
            route,
            persistence: ToolSurfacePersistence::StableKey,
            capabilities,
            session_retention,
            creation_policy,
            target_profile_compatibility: ToolSurfaceTargetProfileCompatibility::AllProfiles,
            legacy_compatibility_key: None,
        }
    }

    pub fn with_target_profile_compatibility(
        mut self,
        compatibility: ToolSurfaceTargetProfileCompatibility,
    ) -> Self {
        self.target_profile_compatibility = compatibility;
        self
    }

    pub fn with_legacy_compatibility_key(mut self, key: impl Into<String>) -> Self {
        self.legacy_compatibility_key = Some(key.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSurfaceRole {
    Primary,
    Inspector,
    Preview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSurfaceRoute {
    StaticAction,
    ProviderOwnedGraphCanvas,
    ProviderOwnedLocal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSurfacePersistence {
    StableKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSurfaceCreationPolicy {
    SingletonPerWorkspace,
    MultipleInstances,
    HostProvidedOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSurfaceTargetProfileCompatibility {
    AllProfiles,
    Profiles(Vec<ProfileRef>),
}
