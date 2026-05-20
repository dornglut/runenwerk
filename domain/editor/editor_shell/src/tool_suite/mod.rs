//! File: domain/editor/editor_shell/src/tool_suite/mod.rs
//! Purpose: Stable tool-suite registry contracts for editor-hosted surfaces.

pub mod capability;
pub mod definition;
pub mod identity;
pub mod legacy;
pub mod registry;

pub use capability::{HostCapabilityPolicy, HostCapabilityRequirements};
pub use definition::{
    EditorToolSuite, ProductCapabilityNeed, ProviderFamilyDefinition, ToolServiceNeed,
    ToolSuiteCapabilityDeclaration, ToolSuiteProfileDefinition, ToolSurfaceDefinition,
    ToolSurfacePersistence, ToolSurfaceRole, ToolSurfaceRoute,
};
pub use identity::{
    CommandCapabilityKey, ProductCapabilityKey, ProfileRef, ProviderFamilyId,
    ResourceCapabilityKey, SuiteRef, SurfaceRef, ToolServiceKey, ToolSuiteId,
    ToolSuiteIdentityError, ToolSurfaceStableKey,
};
pub use legacy::{
    LegacyToolSurfaceResolution, LegacyToolSurfaceStableKeyCandidate,
    SAVEABLE_TOOL_SURFACE_STABLE_KEY_CANDIDATES, resolve_legacy_tool_surface_kind,
    saveable_tool_surface_stable_key_candidates, stable_key_candidate_for_key,
    stable_key_candidate_for_kind, stable_key_candidate_for_persisted_kind,
    stable_key_for_persisted_tool_surface_kind_v2, stable_key_for_tool_surface_kind,
    tool_surface_kind_for_stable_key,
};
pub use registry::{
    ProviderBundle, ProviderBundleError, ProviderFamilyProviderAssignment,
    ProviderFamilyProviderMap, ProviderFamilyProviderMapError, ToolSuiteRegistry,
    ToolSuiteRegistryError, ToolSurfaceRegistry, ToolSurfaceResolution, WorkbenchComposition,
    WorkbenchCompositionBuildError, WorkbenchCompositionBuilder,
};
