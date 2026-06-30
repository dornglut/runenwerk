use crate::shell::tool_suites::UI_LAB_INTERACTION_STORY_SURFACE_KEY;

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
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        if request.matches_stable_key(UI_LAB_INTERACTION_STORY_SURFACE_KEY) {
            SurfaceProviderSupportMode::StableKey
        } else {
            stable_keys_support(request, DIAGNOSTICS_SURFACE_KEYS)
        }
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        if request.matches_stable_key(UI_LAB_INTERACTION_STORY_SURFACE_KEY) {
            return Ok(ui_lab_interaction_story_frame(context, request, session));
        }

        let surface_kind = tool_surface_kind_for_stable_key(request.stable_key());
        let mut lines = vec![
            "M6 route is fail-closed until the owning domain contract ratifies the document"
                .to_string(),
            format!(
                "surface: {}",
                surface_kind
                    .map(m6_surface_key)
                    .unwrap_or_else(|| request.stable_key().as_str())
            ),
            format!(
                "document: {}",
                request
                    .document_context
                    .resolved_document_kind()
                    .map(|kind| kind.stable_name())
                    .unwrap_or("none")
            ),
            surface_kind
                .map(m6_surface_gate_line)
                .unwrap_or("gate: stable surface key is not mapped to an M6 contract")
                .to_string(),
        ];

        if surface_kind.is_some_and(|kind| {
            matches!(
                kind,
                ToolSurfaceKind::Diagnostics
                    | ToolSurfaceKind::RuntimeDebug
                    | ToolSurfaceKind::GameplayCompilerDiagnostics
                    | ToolSurfaceKind::PhysicsDebug
                    | ToolSurfaceKind::SimulationDiagnostics
            )
        }) {
            lines.extend(context.app.asset_catalog_runtime().reload_status_lines());
        }

        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            Vec::new(),
        );

        Ok(ProviderSurfaceFrame {
            title: surface_kind
                .map(m6_surface_title)
                .unwrap_or("M6 Workspace")
                .to_string(),
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

fn ui_lab_interaction_story_frame(
    _context: &SurfaceProviderBuildContext<'_>,
    request: &SurfaceProviderRequest,
    session: &SurfaceSessionState,
) -> ProviderSurfaceFrame {
    let proof_host = session.phase12_interaction_proof_host();
    let proof = proof_host.current_proof();
    let report = proof_host.run_report();
    let static_mount = proof_host.static_mount_report();
    let parity = proof_host.replay_live_parity_report();
    let boundary = proof_host.boundary_assertions();

    let mut lines = vec![
        "UI Lab visible executable interaction story".to_string(),
        "source: Phase12aInteractionProofHost -> InteractionStorySession".to_string(),
        format!("story: {}", proof_host.story_id()),
        format!("mode: {}", report.mode.as_str()),
        format!("input samples: {}", report.input_log.len()),
        format!(
            "static mount: {}",
            if static_mount.passed() { "passed" } else { "failed" }
        ),
        format!(
            "replay/live parity: {}",
            if parity.passed() { "passed" } else { "failed" }
        ),
        format!("no-bypass: {}", boundary.no_bypass_evidence()),
        format!(
            "boundary counters: host_commands={} product_mutations={} overlays={} text_edits={}",
            boundary.host_commands_executed,
            boundary.product_mutations,
            boundary.overlay_events,
            boundary.text_edit_transactions
        ),
        format!(
            "report rows: targets={} focus={} transitions={} facts={} events={} outcomes={} suppressed={} no_target={}",
            report.formation_report.target_resolution.len(),
            report.formation_report.focus_resolution.len(),
            report.formation_report.state_transitions.len(),
            report.formation_report.runtime_facts.len(),
            report.formation_report.runtime_events.len(),
            report.formation_report.semantic_outcomes.len(),
            report.formation_report.suppressed_events.len(),
            report.formation_report.no_target_events.len()
        ),
        "mounted controls:".to_string(),
    ];

    for control in &proof.main_view.controls {
        lines.push(format!(
            "- {:?} {} [{}] markers=[{}] current=[{}]",
            control.widget_id,
            control.label,
            control.control_kind_id,
            marker_labels(control),
            state_labels(&control.current_states)
        ));
    }

    if !proof.report_view.rows.is_empty() {
        lines.push("report evidence:".to_string());
        for row in &proof.report_view.rows {
            lines.push(format!("- {}: {}", row.kind, row.message));
        }
    }

    let (root, routes) = build_self_authoring_control_panel(
        _context.theme,
        request.tool_surface_instance_id,
        lines,
        Vec::new(),
    );

    ProviderSurfaceFrame {
        title: "Interaction Story Lab".to_string(),
        artifact: SurfacePresentationArtifact::provider(root),
        routes,
    }
}

fn marker_labels(control: &ui_runtime::InteractionVisualControl) -> String {
    let labels = control
        .observed_markers
        .iter()
        .map(|marker| marker.label.as_str())
        .collect::<Vec<_>>();
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(", ")
    }
}

fn state_labels(states: &[ui_runtime::InteractionVisibleState]) -> String {
    let labels = states.iter().map(|state| state.as_str()).collect::<Vec<_>>();
    if labels.is_empty() {
        "none".to_string()
    } else {
        labels.join(", ")
    }
}

#[cfg(test)]
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
