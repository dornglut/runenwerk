use super::*;

pub(super) struct SdfGraphCanvasProvider;

impl EditorSurfaceProvider for SdfGraphCanvasProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            SDF_GRAPH_CANVAS_PROVIDER_ID,
            "SDF Graph Canvas",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        request.tool_surface_kind == ToolSurfaceKind::SdfGraphCanvas
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let workspace = context.app.sdf_operation_workspace();
        let operation_projection = workspace.projection();
        let graph_projection = workspace.graph_projection();
        let lines = vec![
            surface_document_context_line(&request.document_context),
            "command-backed SDF graph projection".to_string(),
            "canvas/session state is not SDF graph truth".to_string(),
            format!("graph: {}", graph_projection.display_name),
            format!("graph revision: {}", graph_projection.source_revision),
            format!("nodes: {}", graph_projection.node_count),
            format!("edges: {}", graph_projection.edge_count),
            format!("primitive nodes: {}", graph_projection.primitive_count),
            format!("output nodes: {}", graph_projection.output_count),
            format!("graph can lower: {}", graph_projection.can_lower),
            format!("operation document: {}", operation_projection.display_name),
            format!("layers: {}", operation_projection.layers.len()),
            format!(
                "authored operations: {}",
                operation_projection.operations.len()
            ),
            format!(
                "lowered world_ops records: {}",
                operation_projection.lowered_operation_count
            ),
            format!("commit eligible: {}", operation_projection.can_commit),
        ];
        let actions = sdf_graph_actions();
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );
        Ok(ProviderSurfaceFrame {
            title: "SDF Graph Canvas".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let SurfaceLocalAction::SdfOperation(action) = action else {
            return Ok(None);
        };
        match action {
            editor_shell::SdfOperationSurfaceAction::ApplyGraphCommand { intent } => {
                Ok(Some(editor_domain_proposal(
                    request,
                    context.projection_epoch,
                    EditorDomainMutation::SdfOperation(
                        editor_shell::SdfOperationDomainMutation::ApplyGraphCommand { intent },
                    ),
                )))
            }
            editor_shell::SdfOperationSurfaceAction::LowerGraphToOperationDocument => {
                Ok(Some(editor_domain_proposal(
                    request,
                    context.projection_epoch,
                    EditorDomainMutation::SdfOperation(
                        editor_shell::SdfOperationDomainMutation::LowerGraphToOperationDocument,
                    ),
                )))
            }
            editor_shell::SdfOperationSurfaceAction::SelectLayer { .. }
            | editor_shell::SdfOperationSurfaceAction::SelectOperation { .. }
            | editor_shell::SdfOperationSurfaceAction::ApplyCommand { .. }
            | editor_shell::SdfOperationSurfaceAction::CommitOperationWindow => Ok(None),
        }
    }
}

fn sdf_graph_actions() -> Vec<(String, SurfaceLocalAction)> {
    vec![
        (
            "Add Graph Sphere".to_string(),
            SurfaceLocalAction::SdfOperation(
                editor_shell::SdfOperationSurfaceAction::ApplyGraphCommand {
                    intent: editor_scene::SdfGraphCommandIntent::AddPrimitiveNode {
                        display_name: "Graph Sphere".to_string(),
                        primitive: editor_scene::SdfPrimitiveSpec::new(
                            editor_scene::SdfPrimitiveKind::Sphere,
                            editor_scene::SdfBooleanIntent::Add,
                        ),
                        material_channel: 0,
                    },
                },
            ),
        ),
        (
            "Add Graph Output".to_string(),
            SurfaceLocalAction::SdfOperation(
                editor_shell::SdfOperationSurfaceAction::ApplyGraphCommand {
                    intent: editor_scene::SdfGraphCommandIntent::AddOutputNode {
                        display_name: "Output".to_string(),
                    },
                },
            ),
        ),
        (
            "Lower Graph".to_string(),
            SurfaceLocalAction::SdfOperation(
                editor_shell::SdfOperationSurfaceAction::LowerGraphToOperationDocument,
            ),
        ),
    ]
}
