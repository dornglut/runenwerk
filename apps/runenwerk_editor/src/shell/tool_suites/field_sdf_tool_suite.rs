//! File: apps/runenwerk_editor/src/shell/tool_suites/field_sdf_tool_suite.rs
//! Purpose: Field-world and SDF tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{ToolSuiteSurface, tool_suite};

pub fn field_sdf_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.field_world",
        "Field World",
        &[
            preview(ToolSurfaceKind::FieldProductViewer),
            primary(ToolSurfaceKind::SdfBrushBrowser),
            primary(ToolSurfaceKind::FieldLayerStack),
            primary(ToolSurfaceKind::SdfGraphCanvas),
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
