//! File: apps/runenwerk_editor/src/shell/tool_suites/asset_tool_suite.rs
//! Purpose: Asset workflow tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{ToolSuiteSurface, tool_suite};

pub fn asset_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.assets",
        "Assets",
        &[
            primary(ToolSurfaceKind::AssetBrowser),
            inspector(ToolSurfaceKind::ImportInspector),
        ],
    )
}

const fn primary(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Primary,
        route: ToolSurfaceRoute::ProviderOwnedLocal,
    }
}

const fn inspector(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Inspector,
        route: ToolSurfaceRoute::ProviderOwnedLocal,
    }
}
