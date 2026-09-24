//! File: apps/runenwerk_editor/src/shell/tool_suites/gameplay_tool_suite.rs
//! Purpose: Gameplay, particle, and physics tool-suite metadata.

use editor_shell::{EditorToolSuite, ToolSurfaceKind, ToolSurfaceRole, ToolSurfaceRoute};

use super::{ToolSuiteSurface, tool_suite};

pub fn gameplay_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.gameplay",
        "Gameplay",
        &[
            primary(ToolSurfaceKind::GameplayGraphCanvas),
            inspector(ToolSurfaceKind::GameplayCompilerDiagnostics),
        ],
    )
}

pub fn particle_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.particle",
        "Particle",
        &[
            primary(ToolSurfaceKind::ParticleGraphCanvas),
            preview(ToolSurfaceKind::ParticlePreview),
        ],
    )
}

pub fn physics_tool_suite() -> EditorToolSuite {
    tool_suite(
        "runenwerk.physics",
        "Physics",
        &[
            primary(ToolSurfaceKind::PhysicsAuthoring),
            inspector(ToolSurfaceKind::PhysicsDebug),
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
