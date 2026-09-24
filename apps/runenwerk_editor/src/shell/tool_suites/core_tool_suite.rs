//! File: apps/runenwerk_editor/src/shell/tool_suites/core_tool_suite.rs
//! Purpose: Scene and editor-core tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{ToolSuiteSurface, tool_suite};

pub fn scene_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.scene",
        "Scene",
        &[
            primary(ToolSurfaceKind::Outliner),
            primary(ToolSurfaceKind::EntityTable),
            primary(ToolSurfaceKind::Viewport),
            inspector(ToolSurfaceKind::Inspector),
        ],
    )
}

pub fn editor_core_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.editor",
        "Editor Core",
        &[primary(ToolSurfaceKind::Console)],
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
