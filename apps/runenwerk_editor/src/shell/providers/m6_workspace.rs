use super::*;

pub(super) struct M6WorkspaceProvider;

impl EditorSurfaceProvider for M6WorkspaceProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            M6_WORKSPACE_PROVIDER_ID,
            "M6 Workspace",
            SurfaceProviderPriority(500),
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        is_m6_workspace_surface(request.tool_surface_kind)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let mut lines = vec![
            "M6 route is fail-closed until the owning domain contract ratifies the document"
                .to_string(),
            format!("surface: {}", m6_surface_key(request.tool_surface_kind)),
            format!(
                "document: {}",
                request
                    .document_context
                    .resolved_document_kind()
                    .map(|kind| kind.stable_name())
                    .unwrap_or("none")
            ),
            m6_surface_gate_line(request.tool_surface_kind).to_string(),
        ];

        if matches!(
            request.tool_surface_kind,
            ToolSurfaceKind::Diagnostics
                | ToolSurfaceKind::RuntimeDebug
                | ToolSurfaceKind::GameplayCompilerDiagnostics
                | ToolSurfaceKind::PhysicsDebug
                | ToolSurfaceKind::SimulationDiagnostics
        ) {
            lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        }

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            Vec::new(),
        );

        Ok(ProviderSurfaceFrame {
            title: m6_surface_title(request.tool_surface_kind).to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        _context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        _action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        Ok(None)
    }
}

pub(super) fn is_m6_workspace_surface(kind: ToolSurfaceKind) -> bool {
    matches!(
        kind,
        ToolSurfaceKind::GraphCanvas
            | ToolSurfaceKind::Diagnostics
            | ToolSurfaceKind::RuntimeDebug
            | ToolSurfaceKind::FieldLayerStack
            | ToolSurfaceKind::SdfGraphCanvas
            | ToolSurfaceKind::MaterialGraphCanvas
            | ToolSurfaceKind::MaterialInspector
            | ToolSurfaceKind::MaterialPreview
            | ToolSurfaceKind::TextureViewer
            | ToolSurfaceKind::VolumeTextureViewer
            | ToolSurfaceKind::ProcgenGraphCanvas
            | ToolSurfaceKind::ProcgenPreview
            | ToolSurfaceKind::GameplayGraphCanvas
            | ToolSurfaceKind::GameplayCompilerDiagnostics
            | ToolSurfaceKind::ParticleGraphCanvas
            | ToolSurfaceKind::ParticlePreview
            | ToolSurfaceKind::PhysicsAuthoring
            | ToolSurfaceKind::PhysicsDebug
            | ToolSurfaceKind::Timeline
            | ToolSurfaceKind::CurveEditor
            | ToolSurfaceKind::AnimationGraphCanvas
            | ToolSurfaceKind::SimulationPreview
            | ToolSurfaceKind::SimulationDiagnostics
    )
}

pub(super) fn is_m6_global_diagnostic_surface(kind: ToolSurfaceKind) -> bool {
    matches!(
        kind,
        ToolSurfaceKind::Diagnostics
            | ToolSurfaceKind::RuntimeDebug
            | ToolSurfaceKind::GameplayCompilerDiagnostics
            | ToolSurfaceKind::PhysicsDebug
            | ToolSurfaceKind::SimulationDiagnostics
    )
}

fn m6_surface_title(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::GraphCanvas => "Graph Canvas",
        ToolSurfaceKind::Diagnostics => "Diagnostics",
        ToolSurfaceKind::RuntimeDebug => "Runtime Debug",
        ToolSurfaceKind::FieldLayerStack => "Field Layer Stack",
        ToolSurfaceKind::SdfGraphCanvas => "SDF Graph Canvas",
        ToolSurfaceKind::MaterialGraphCanvas => "Material Graph Canvas",
        ToolSurfaceKind::MaterialInspector => "Material Inspector",
        ToolSurfaceKind::MaterialPreview => "Material Preview",
        ToolSurfaceKind::TextureViewer => "Texture Viewer",
        ToolSurfaceKind::VolumeTextureViewer => "Volume Texture Viewer",
        ToolSurfaceKind::ProcgenGraphCanvas => "Procgen Graph Canvas",
        ToolSurfaceKind::ProcgenPreview => "Procgen Preview",
        ToolSurfaceKind::GameplayGraphCanvas => "Gameplay Graph Canvas",
        ToolSurfaceKind::GameplayCompilerDiagnostics => "Gameplay Compiler Diagnostics",
        ToolSurfaceKind::ParticleGraphCanvas => "Particle Graph Canvas",
        ToolSurfaceKind::ParticlePreview => "Particle Preview",
        ToolSurfaceKind::PhysicsAuthoring => "Physics Authoring",
        ToolSurfaceKind::PhysicsDebug => "Physics Debug",
        ToolSurfaceKind::Timeline => "Timeline",
        ToolSurfaceKind::CurveEditor => "Curve Editor",
        ToolSurfaceKind::AnimationGraphCanvas => "Animation Graph Canvas",
        ToolSurfaceKind::SimulationPreview => "Simulation Preview",
        ToolSurfaceKind::SimulationDiagnostics => "Simulation Diagnostics",
        _ => "M6 Workspace",
    }
}

fn m6_surface_key(kind: ToolSurfaceKind) -> &'static str {
    editor_shell::tool_surface_kind_definition_key(kind)
}

fn m6_surface_gate_line(kind: ToolSurfaceKind) -> &'static str {
    match kind {
        ToolSurfaceKind::MaterialGraphCanvas
        | ToolSurfaceKind::MaterialInspector
        | ToolSurfaceKind::MaterialPreview => {
            "gate: material_graph contracts exist; editor authoring, preview adapters, and formed-product surfaces are not implemented"
        }
        ToolSurfaceKind::TextureViewer | ToolSurfaceKind::VolumeTextureViewer => {
            "gate: texture descriptors exist; Texture3D preview/upload adapter and viewer provider are not implemented"
        }
        ToolSurfaceKind::ProcgenGraphCanvas | ToolSurfaceKind::ProcgenPreview => {
            "gate: procgen deterministic seed/scope and world_ops lowering are not implemented"
        }
        ToolSurfaceKind::GameplayGraphCanvas | ToolSurfaceKind::GameplayCompilerDiagnostics => {
            "gate: gameplay event/action/state/quest contracts are not accepted"
        }
        ToolSurfaceKind::ParticleGraphCanvas | ToolSurfaceKind::ParticlePreview => {
            "gate: particle simulation contract and formed product are not implemented"
        }
        ToolSurfaceKind::PhysicsAuthoring | ToolSurfaceKind::PhysicsDebug => {
            "gate: physics domain contract and solver-adapter boundary are not implemented"
        }
        ToolSurfaceKind::Timeline
        | ToolSurfaceKind::CurveEditor
        | ToolSurfaceKind::AnimationGraphCanvas => {
            "gate: animation clip/curve/timeline/state graph contract is not implemented"
        }
        ToolSurfaceKind::SimulationPreview | ToolSurfaceKind::SimulationDiagnostics => {
            "gate: simulation_process preview, bake, and rollback contracts are not implemented"
        }
        ToolSurfaceKind::FieldLayerStack | ToolSurfaceKind::SdfGraphCanvas => {
            "gate: concrete P1-A SDF operation providers own this surface; fallback is diagnostic only"
        }
        ToolSurfaceKind::GraphCanvas => {
            "gate: neutral graph structure is available; domain meaning must come from an owning crate"
        }
        ToolSurfaceKind::Diagnostics | ToolSurfaceKind::RuntimeDebug => {
            "gate: diagnostics are read-only until an owning domain command boundary exists"
        }
        _ => "gate: unsupported M6 surface",
    }
}
