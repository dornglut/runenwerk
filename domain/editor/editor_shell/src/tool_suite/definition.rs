//! File: domain/editor/editor_shell/src/tool_suite/definition.rs
//! Purpose: App-neutral tool-suite surface declaration contracts.

use crate::PanelKind;

use super::{ProviderFamilyId, ToolSuiteId, ToolSurfaceStableKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorToolSuite {
    pub suite_id: ToolSuiteId,
    pub label: String,
    pub provider_families: Vec<ProviderFamilyDefinition>,
    pub surfaces: Vec<ToolSurfaceDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFamilyDefinition {
    pub id: ProviderFamilyId,
    pub label: String,
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
