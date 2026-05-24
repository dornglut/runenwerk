use super::*;

pub(super) struct FieldLayerStackProvider;

impl EditorSurfaceProvider for FieldLayerStackProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            FIELD_LAYER_STACK_PROVIDER_ID,
            "Field Layer Stack",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_support(request, FIELD_LAYER_STACK_SURFACE_KEY)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let workspace = context.app.sdf_operation_workspace();
        let projection = workspace.projection();
        let mut lines = vec![
            surface_document_context_line(&request.document_context),
            format!("document: {}", projection.display_name),
            format!("source revision: {}", projection.source_revision),
            format!("layers: {}", projection.layers.len()),
            format!("operations: {}", projection.operations.len()),
            format!(
                "lowered world_ops records: {}",
                projection.lowered_operation_count
            ),
            format!("touched chunks: {}", projection.touched_chunk_count),
            format!("commit eligible: {}", projection.can_commit),
            format!(
                "committed world_ops records: {}",
                workspace.committed_operation_log().operations.len()
            ),
            format!("dirty chunks: {}", workspace.dirty_chunks().by_chunk.len()),
            format!(
                "formed CPU preview products: {}",
                workspace.field_preview_products().len()
            ),
        ];
        lines.extend(projection.layers.iter().map(|layer| {
            let selected = if Some(layer.layer_id) == workspace.selected_layer_id() {
                "*"
            } else {
                " "
            };
            format!(
                "{selected} layer #{} {} enabled={} ops={}/{}",
                layer.layer_id.raw(),
                layer.display_name,
                layer.enabled,
                layer.enabled_operation_count,
                layer.operation_count
            )
        }));
        lines.extend(projection.operations.iter().map(|operation| {
            let selected = if Some(operation.operation_id) == workspace.selected_operation_id() {
                "*"
            } else {
                " "
            };
            format!(
                "{selected} op #{} layer={} {} {} {} enabled={}",
                operation.operation_id.raw(),
                operation.layer_id.raw(),
                operation.display_name,
                operation.primitive_kind,
                operation.boolean_intent,
                operation.enabled
            )
        }));
        lines.extend(
            projection
                .issues
                .iter()
                .map(|issue| format!("{:?} {:?}: {}", issue.severity, issue.code, issue.message)),
        );
        if projection.issues.is_empty() {
            lines.push("ratification: accepted".to_string());
        }

        let actions = field_layer_actions(context.app);
        let (root, routes) = build_self_authoring_control_panel(
            context.theme,
            request.tool_surface_instance_id,
            lines,
            actions,
        );
        Ok(ProviderSurfaceFrame {
            title: "Field Layer Stack".to_string(),
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
            editor_shell::SdfOperationSurfaceAction::SelectLayer { layer_id } => {
                Ok(Some(surface_session_proposal(
                    request,
                    context.projection_epoch,
                    SurfaceSessionMutation::SdfOperation(
                        editor_shell::SdfOperationSessionMutation::SelectLayer { layer_id },
                    ),
                )))
            }
            editor_shell::SdfOperationSurfaceAction::SelectOperation { operation_id } => {
                Ok(Some(surface_session_proposal(
                    request,
                    context.projection_epoch,
                    SurfaceSessionMutation::SdfOperation(
                        editor_shell::SdfOperationSessionMutation::SelectOperation { operation_id },
                    ),
                )))
            }
            editor_shell::SdfOperationSurfaceAction::ApplyCommand { intent } => {
                Ok(Some(editor_domain_proposal(
                    request,
                    context.projection_epoch,
                    EditorDomainMutation::SdfOperation(
                        editor_shell::SdfOperationDomainMutation::ApplyCommand { intent },
                    ),
                )))
            }
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
            editor_shell::SdfOperationSurfaceAction::CommitOperationWindow => {
                Ok(Some(editor_domain_proposal(
                    request,
                    context.projection_epoch,
                    EditorDomainMutation::SdfOperation(
                        editor_shell::SdfOperationDomainMutation::CommitOperationWindow,
                    ),
                )))
            }
        }
    }
}

fn field_layer_actions(app: &RunenwerkEditorApp) -> Vec<(String, SurfaceLocalAction)> {
    let workspace = app.sdf_operation_workspace();
    let Some(layer_id) = workspace
        .selected_layer_id()
        .or_else(|| workspace.document().layers().first().map(|layer| layer.id))
    else {
        return vec![(
            "Add Layer".to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddLayer {
                    stable_name: "base".to_string(),
                    display_name: "Base".to_string(),
                },
            }),
        )];
    };

    let mut actions = vec![
        (
            "Add Sphere".to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Sphere Add".to_string(),
                    primitive: editor_scene::SdfPrimitiveSpec::new(
                        editor_scene::SdfPrimitiveKind::Sphere,
                        editor_scene::SdfBooleanIntent::Add,
                    ),
                    material_channel: 0,
                },
            }),
        ),
        (
            "Add Subtract Box".to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Box Subtract".to_string(),
                    primitive: editor_scene::SdfPrimitiveSpec::new(
                        editor_scene::SdfPrimitiveKind::Box,
                        editor_scene::SdfBooleanIntent::Subtract,
                    ),
                    material_channel: 0,
                },
            }),
        ),
        (
            "Add Intersect Cylinder".to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Cylinder Intersect".to_string(),
                    primitive: editor_scene::SdfPrimitiveSpec::new(
                        editor_scene::SdfPrimitiveKind::Cylinder,
                        editor_scene::SdfBooleanIntent::Intersect,
                    ),
                    material_channel: 0,
                },
            }),
        ),
        (
            "Add Smooth Sphere".to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::AddPrimitiveOperation {
                    layer_id,
                    display_name: "Sphere Smooth Add".to_string(),
                    primitive: editor_scene::SdfPrimitiveSpec::new(
                        editor_scene::SdfPrimitiveKind::Sphere,
                        editor_scene::SdfBooleanIntent::SmoothAdd,
                    )
                    .with_default_smooth_radius(),
                    material_channel: 0,
                },
            }),
        ),
    ];

    if let Some(layer) = workspace.document().layer(layer_id) {
        actions.push((
            if layer.metadata.enabled {
                "Disable Layer"
            } else {
                "Enable Layer"
            }
            .to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::SetLayerEnabled {
                    layer_id,
                    enabled: !layer.metadata.enabled,
                },
            }),
        ));
        actions.push((
            "Move Layer Up".to_string(),
            sdf_action(editor_shell::SdfOperationSurfaceAction::ApplyCommand {
                intent: editor_scene::SdfOperationCommandIntent::MoveLayer {
                    layer_id,
                    direction: editor_scene::SdfLayerMoveDirection::Up,
                },
            }),
        ));
    }
    actions.push((
        "Commit Window".to_string(),
        sdf_action(editor_shell::SdfOperationSurfaceAction::CommitOperationWindow),
    ));
    actions
}

fn sdf_action(action: editor_shell::SdfOperationSurfaceAction) -> SurfaceLocalAction {
    SurfaceLocalAction::SdfOperation(action)
}
