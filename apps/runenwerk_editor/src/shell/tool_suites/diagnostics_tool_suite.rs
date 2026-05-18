//! File: apps/runenwerk_editor/src/shell/tool_suites/diagnostics_tool_suite.rs
//! Purpose: Diagnostics and runtime-debug tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{
    stable_tool_surface_definition, tool_suite, ToolSuiteSurface,
    TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
};

pub fn diagnostics_tool_suite() -> EditorToolSuite {
    let mut suite = tool_suite(
        "runenwerk.diagnostics",
        "Diagnostics",
        &[
            inspector(ToolSurfaceKind::Diagnostics),
            inspector(ToolSurfaceKind::RuntimeDebug),
            fallback(ToolSurfaceKind::Placeholder),
        ],
    );
    let provider_family = suite.provider_families[0].id.clone();
    suite.surfaces.push(stable_tool_surface_definition(
        TOOL_SUITE_REGISTRY_INSPECTOR_SURFACE_KEY,
        "Tool Suite Registry Inspector",
        ToolSurfaceRole::Inspector,
        ToolSurfaceRoute::ProviderOwnedLocal,
        provider_family,
    ));
    suite
}

const fn inspector(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Inspector,
        route: ToolSurfaceRoute::ProviderOwnedLocal,
    }
}

const fn fallback(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Preview,
        route: ToolSurfaceRoute::StaticAction,
    }
}
