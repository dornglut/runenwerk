//! File: apps/runenwerk_editor/src/shell/tool_suites/procgen_tool_suite.rs
//! Purpose: Procedural-generation tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{tool_suite, ToolSuiteSurface};

pub fn procgen_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.procgen",
        "Procedural Generation",
        &[
            primary(ToolSurfaceKind::ProcgenGraphCanvas),
            preview(ToolSurfaceKind::ProcgenPreview),
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

const fn preview(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Preview,
        route: ToolSurfaceRoute::ProviderOwnedLocal,
    }
}
