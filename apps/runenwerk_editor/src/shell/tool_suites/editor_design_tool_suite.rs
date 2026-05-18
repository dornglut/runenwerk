//! File: apps/runenwerk_editor/src/shell/tool_suites/editor_design_tool_suite.rs
//! Purpose: Editor-design and self-authoring tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{tool_suite, ToolSuiteSurface};

pub fn editor_design_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.editor_design",
        "Editor Design",
        &[
            primary(ToolSurfaceKind::EditorDesignOutliner),
            primary(ToolSurfaceKind::UiHierarchy),
            primary(ToolSurfaceKind::UiCanvas),
            inspector(ToolSurfaceKind::StyleInspector),
            inspector(ToolSurfaceKind::Bindings),
            preview(ToolSurfaceKind::DockLayoutPreview),
            primary(ToolSurfaceKind::ThemeEditor),
            primary(ToolSurfaceKind::ShortcutEditor),
            primary(ToolSurfaceKind::MenuEditor),
            inspector(ToolSurfaceKind::DefinitionValidation),
            inspector(ToolSurfaceKind::CommandDiff),
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

const fn preview(kind: ToolSurfaceKind) -> ToolSuiteSurface {
    ToolSuiteSurface {
        kind,
        role: ToolSurfaceRole::Preview,
        route: ToolSurfaceRoute::ProviderOwnedLocal,
    }
}
