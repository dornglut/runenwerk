//! File: apps/runenwerk_editor/src/shell/tool_suites/graph_tool_suite.rs
//! Purpose: Generic graph workspace tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{ToolSuiteSurface, tool_suite};

pub fn graph_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.graph",
        "Graph",
        &[ToolSuiteSurface {
            kind: ToolSurfaceKind::GraphCanvas,
            role: ToolSurfaceRole::Primary,
            route: ToolSurfaceRoute::ProviderOwnedLocal,
        }],
    )
}
