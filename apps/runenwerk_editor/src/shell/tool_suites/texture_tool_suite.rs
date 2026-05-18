//! File: apps/runenwerk_editor/src/shell/tool_suites/texture_tool_suite.rs
//! Purpose: Texture viewer tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{tool_suite, ToolSuiteSurface};

pub fn texture_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.texture",
        "Texture",
        &[
            preview(ToolSurfaceKind::TextureViewer),
            preview(ToolSurfaceKind::VolumeTextureViewer),
        ],
    )
}

const fn preview(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Preview,
        route: ToolSurfaceRoute::ProviderOwnedLocal,
    }
}
