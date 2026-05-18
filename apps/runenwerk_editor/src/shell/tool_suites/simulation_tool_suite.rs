//! File: apps/runenwerk_editor/src/shell/tool_suites/simulation_tool_suite.rs
//! Purpose: Animation and simulation tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{tool_suite, ToolSuiteSurface};

pub fn animation_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.animation",
        "Animation",
        &[
            primary(ToolSurfaceKind::Timeline),
            primary(ToolSurfaceKind::CurveEditor),
            primary(ToolSurfaceKind::AnimationGraphCanvas),
        ],
    )
}

pub fn simulation_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.simulation",
        "Simulation",
        &[
            preview(ToolSurfaceKind::SimulationPreview),
            inspector(ToolSurfaceKind::SimulationDiagnostics),
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
