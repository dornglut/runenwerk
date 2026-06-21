use super::*;

pub(super) struct MaterialGraphCanvasProvider;

impl EditorSurfaceProvider for MaterialGraphCanvasProvider {
    fn descriptor(&self) -> SurfaceProviderDescriptor {
        SurfaceProviderDescriptor::new(
            MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
            "Material Graph Canvas",
            SurfaceProviderPriority::DEFAULT,
        )
    }

    fn supports(&self, request: &SurfaceProviderRequest) -> bool {
        self.support_mode(request).is_supported()
    }

    fn support_mode(&self, request: &SurfaceProviderRequest) -> SurfaceProviderSupportMode {
        stable_key_support(request, MATERIAL_GRAPH_CANVAS_SURFACE_KEY)
    }

    fn build_frame(
        &self,
        context: &SurfaceProviderBuildContext<'_>,
        request: &SurfaceProviderRequest,
        _session: &SurfaceSessionState,
    ) -> Result<ProviderSurfaceFrame, SurfaceProviderDiagnostic> {
        let mut view_model = context
            .app
            .material_lab_runtime()
            .graph_canvas_view_model_with_scene_material_assignments(
                context.app.asset_catalog_runtime().catalog(),
                context.app.asset_catalog_status_lines(),
                Some(context.app.runtime().scene_material_assignments()),
            );
        view_model.sdf_primitives = material_sdf_primitive_bindings(context.app.runtime());
        let mut lines = vec![
            "material graph canvas: source-backed MaterialGraphDocument workflow".to_string(),
            surface_document_context_line(&request.document_context),
            "canvas state is projection only; domain/material_graph remains material truth"
                .to_string(),
            format!("material graph assets: {}", view_model.rows.len()),
        ];
        lines.extend(view_model.catalog_status_lines.clone());
        for row in &view_model.rows {
            let marker = if row.is_selected { "*" } else { " " };
            let preserved = if row.has_prior_valid_preservation {
                " prior-valid-preserved"
            } else {
                ""
            };
            lines.push(format!(
                "{marker} material asset {} {} [{}] source={:?} artifacts={}{}",
                row.asset_id.raw(),
                row.display_name,
                row.stable_name,
                row.source_id.map(|source_id| source_id.raw()),
                row.artifact_count,
                preserved
            ));
        }
        if view_model.sdf_primitives.is_empty() {
            lines.push("SDF primitive material bindings: none".to_string());
        } else {
            lines.push(format!(
                "SDF primitive material bindings: {}",
                view_model.sdf_primitives.len()
            ));
            for primitive in &view_model.sdf_primitives {
                let assigned = primitive
                    .assigned_slot_label
                    .as_deref()
                    .unwrap_or("unassigned");
                let diagnostic = primitive.diagnostic.as_deref().unwrap_or("ok");
                lines.push(format!(
                    "SDF primitive entity={} {} [{}] assigned={} resolved_slot={} table={} fallback={} status={}",
                    primitive.entity_id.0,
                    primitive.display_name,
                    primitive.primitive_kind_label,
                    assigned,
                    primitive.resolved_slot_id.raw(),
                    primitive.material_table_index,
                    primitive.used_default_fallback,
                    diagnostic
                ));
            }
        }
        if let Some(selected) = &view_model.selected {
            lines.push(format!(
                "selected material asset {} source={:?} path={}",
                selected.asset_id.raw(),
                selected.source_id.map(|source_id| source_id.raw()),
                selected.source_path.as_deref().unwrap_or("none")
            ));
            if let Some(document_id) = selected.document_id {
                lines.push(format!("document id: {}", document_id.raw()));
            }
            if let Some(output_target) = selected.output_target {
                lines.push(format!("output target: {output_target:?}"));
            }
            lines.push(format!("source-map nodes: {}", selected.node_count));
        }
        lines.extend(material_diagnostic_row_lines(&view_model.diagnostic_rows));
        lines.extend(material_resource_binding_diagnostic_lines(
            &view_model.resource_binding_diagnostics,
        ));
        lines.extend(view_model.diagnostic_lines.clone());
        lines.extend(crate::material_lab::material_artifact_lines(
            context.app.asset_catalog_runtime().catalog(),
        ));
        lines.extend(context.app.asset_catalog_runtime().reload_status_lines());

        let mut actions = Vec::new();
        actions.extend(view_model.rows.iter().flat_map(|row| {
            [
                (
                    format!("Select {}", row.display_name),
                    SurfaceLocalAction::Material(MaterialSurfaceAction::SelectMaterialAsset {
                        asset_id: row.asset_id,
                    }),
                ),
                (
                    format!("Build {}", row.display_name),
                    SurfaceLocalAction::Material(MaterialSurfaceAction::BuildMaterialPreview {
                        asset_id: row.asset_id,
                    }),
                ),
            ]
        }));
        if view_model.selected.is_some() {
            actions.push((
                "Build selected preview".to_string(),
                SurfaceLocalAction::Material(MaterialSurfaceAction::BuildSelectedMaterialPreview),
            ));
        }
        actions.push((
            "Clear material diagnostics".to_string(),
            SurfaceLocalAction::Material(MaterialSurfaceAction::ClearMaterialDiagnostics),
        ));

        let (root, routes) = build_material_graph_surface(
            context.theme,
            request.tool_surface_instance_id,
            &view_model,
            lines,
            actions,
        );

        Ok(ProviderSurfaceFrame {
            title: "Material Graph Canvas".to_string(),
            artifact: SurfacePresentationArtifact::provider(root),
            routes,
        })
    }

    fn map_action(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        action: SurfaceLocalAction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let SurfaceLocalAction::Material(action) = action else {
            return Ok(None);
        };
        Ok(
            material_surface_action_command(action, context.projection_epoch)
                .map(SurfaceCommandProposal::Shell),
        )
    }

    fn map_interaction(
        &self,
        context: &SurfaceProviderDispatchContext<'_>,
        _request: &SurfaceProviderRequest,
        interaction: SurfaceInteraction,
    ) -> Result<Option<SurfaceCommandProposal>, SurfaceProviderDiagnostic> {
        let SurfaceInteraction::GraphCanvasAction(action) = interaction;
        Ok(material_action_for_graph_canvas_action(action)
            .and_then(|action| material_surface_action_command(action, context.projection_epoch))
            .map(SurfaceCommandProposal::Shell))
    }
}

fn material_sdf_primitive_bindings(
    runtime: &crate::editor_runtime::RunenwerkEditorRuntime,
) -> Vec<MaterialSdfPrimitiveBindingViewModel> {
    let assignments = runtime.scene_material_assignments();
    runtime
        .list_scene_entities()
        .into_iter()
        .filter_map(|entity| {
            let ecs_entity = runtime.ids().resolve_entity(entity.id)?;
            let primitive = runtime
                .world()
                .get::<crate::editor_runtime::EditorPrimitive>(ecs_entity)
                .copied()?;
            let source_id = editor_scene::SdfPrimitiveSourceId::new(entity.id);
            let explicit_assignment = assignments
                .assignments()
                .find(|assignment| assignment.primitive == source_id);
            let assigned_slot_id = explicit_assignment.map(|assignment| assignment.slot_id);
            let assigned_slot_label = assigned_slot_id.and_then(|slot_id| {
                assignments
                    .palette()
                    .slots
                    .iter()
                    .find(|slot| slot.slot_id == slot_id)
                    .map(|slot| slot.display_name.clone())
            });
            let (resolution, diagnostics) =
                assignments.resolve_material_binding_for_sdf_scene_packet(source_id);
            Some(MaterialSdfPrimitiveBindingViewModel {
                entity_id: entity.id,
                display_name: entity.display_name,
                primitive_kind_label: sdf_primitive_kind_label(primitive.kind()).to_string(),
                assigned_slot_id,
                assigned_slot_label,
                requested_slot_id: resolution.requested_slot_id,
                resolved_slot_id: resolution.resolved_slot_id,
                material_table_index: resolution.material_table_index,
                used_default_fallback: resolution.used_default_fallback,
                diagnostic: diagnostics
                    .first()
                    .map(|diagnostic| diagnostic.message.clone()),
            })
        })
        .collect()
}

fn sdf_primitive_kind_label(kind: crate::editor_runtime::EditorPrimitiveKind) -> &'static str {
    match kind {
        crate::editor_runtime::EditorPrimitiveKind::Box => "box",
        crate::editor_runtime::EditorPrimitiveKind::Sphere => "sphere",
        crate::editor_runtime::EditorPrimitiveKind::Capsule => "capsule",
        crate::editor_runtime::EditorPrimitiveKind::Cylinder => "cylinder",
        crate::editor_runtime::EditorPrimitiveKind::Torus => "torus",
        crate::editor_runtime::EditorPrimitiveKind::Plane => "plane",
    }
}

pub(super) fn material_diagnostic_row_lines(
    rows: &[MaterialDiagnosticRowViewModel],
) -> Vec<String> {
    if rows.is_empty() {
        return vec!["material diagnostics: none (structured)".to_string()];
    }
    rows.iter()
        .map(|row| {
            let subject = row.subject_label.as_deref().unwrap_or("none");
            let category = row.category_label.as_deref().unwrap_or("uncategorized");
            format!(
                "material diagnostic [{:?}] {} subject={} category={}: {}",
                row.severity, row.code, subject, category, row.message
            )
        })
        .collect()
}

pub(super) fn material_resource_binding_diagnostic_lines(
    rows: &[MaterialResourceBindingDiagnosticViewModel],
) -> Vec<String> {
    if rows.is_empty() {
        return vec!["texture/resource bindings: none (structured)".to_string()];
    }
    let mut lines = vec!["Texture / Resource Bindings".to_string()];
    lines.extend(rows.iter().map(|row| {
        let expected = row.expected_kind_label.as_deref().unwrap_or("unknown");
        let resolved = row.resolved_artifact_label.as_deref().unwrap_or("none");
        format!(
            "resource binding [{:?}] {} status={:?} binding={} slot={} expected={} resolved={}: {}",
            row.severity,
            row.code,
            row.status,
            row.binding_label,
            row.resource_key_or_slot_label,
            expected,
            resolved,
            row.message
        )
    }));
    lines
}

pub(super) fn material_action_for_graph_canvas_action(
    action: ui_graph_editor::GraphCanvasAction,
) -> Option<MaterialSurfaceAction> {
    match action {
        ui_graph_editor::GraphCanvasAction::SelectNode { node, .. } => {
            Some(MaterialSurfaceAction::SelectGraphNode {
                node_id: graph::NodeId::new(node.0),
            })
        }
        ui_graph_editor::GraphCanvasAction::SelectEdge { edge, .. } => {
            Some(MaterialSurfaceAction::SelectGraphEdge {
                edge_id: graph::EdgeId::new(edge.0),
            })
        }
        ui_graph_editor::GraphCanvasAction::ClearSelection => {
            Some(MaterialSurfaceAction::ClearGraphSelection)
        }
        ui_graph_editor::GraphCanvasAction::Pan {
            phase: ui_graph_editor::GraphGesturePhase::End,
            delta,
        } => Some(MaterialSurfaceAction::PanGraph {
            delta_x: delta.x,
            delta_y: delta.y,
        }),
        ui_graph_editor::GraphCanvasAction::Zoom { zoom_milli, .. } => {
            Some(MaterialSurfaceAction::SetGraphZoom { zoom_milli })
        }
        ui_graph_editor::GraphCanvasAction::EndNodeDrag { node, delta } => {
            Some(MaterialSurfaceAction::MoveGraphNode {
                node_id: graph::NodeId::new(node.0),
                delta_x: delta.x,
                delta_y: delta.y,
            })
        }
        ui_graph_editor::GraphCanvasAction::EndConnection { from, to: Some(to) } => {
            Some(MaterialSurfaceAction::ConnectPorts {
                from_port_id: graph::PortId::new(from.0),
                to_port_id: graph::PortId::new(to.0),
            })
        }
        ui_graph_editor::GraphCanvasAction::KeyboardDeleteSelection => {
            Some(MaterialSurfaceAction::DeleteSelectedGraphSelection)
        }
        ui_graph_editor::GraphCanvasAction::KeyboardShortcut(shortcut) => {
            Some(material_action_for_graph_shortcut(shortcut))
        }
        // Gesture lifecycle and provisional interactions are canvas-local state.
        // Only committed graph edits become Material Lab source/workflow commands.
        ui_graph_editor::GraphCanvasAction::Pan { .. }
        | ui_graph_editor::GraphCanvasAction::BeginNodeDrag { .. }
        | ui_graph_editor::GraphCanvasAction::UpdateNodeDrag { .. }
        | ui_graph_editor::GraphCanvasAction::BeginConnection { .. }
        | ui_graph_editor::GraphCanvasAction::UpdateConnection { .. }
        | ui_graph_editor::GraphCanvasAction::EndConnection { to: None, .. }
        | ui_graph_editor::GraphCanvasAction::BeginMarquee { .. }
        | ui_graph_editor::GraphCanvasAction::UpdateMarquee { .. }
        | ui_graph_editor::GraphCanvasAction::EndMarquee { .. }
        | ui_graph_editor::GraphCanvasAction::CancelGesture => None,
    }
}

fn material_action_for_graph_shortcut(
    shortcut: ui_graph_editor::GraphShortcutAction,
) -> MaterialSurfaceAction {
    match shortcut {
        ui_graph_editor::GraphShortcutAction::AddNode => MaterialSurfaceAction::OpenNodePicker,
        ui_graph_editor::GraphShortcutAction::DeleteSelection => {
            MaterialSurfaceAction::DeleteSelectedGraphSelection
        }
        ui_graph_editor::GraphShortcutAction::Undo => MaterialSurfaceAction::UndoMaterialEdit,
        ui_graph_editor::GraphShortcutAction::Redo => MaterialSurfaceAction::RedoMaterialEdit,
        ui_graph_editor::GraphShortcutAction::BuildPreview => {
            MaterialSurfaceAction::BuildSelectedMaterialPreview
        }
        ui_graph_editor::GraphShortcutAction::FocusPreview => {
            MaterialSurfaceAction::SelectPreviewProduct {
                selection: material_graph::MaterialGraphPreviewSelection::MaterialPreviewProduct,
            }
        }
    }
}

pub(super) fn material_surface_action_command(
    action: MaterialSurfaceAction,
    projection_epoch: u64,
) -> Option<ShellCommand> {
    match action {
        MaterialSurfaceAction::SelectMaterialAsset { asset_id } => {
            Some(ShellCommand::SelectMaterialAsset {
                asset_id,
                projection_epoch,
            })
        }
        MaterialSurfaceAction::BuildMaterialPreview { asset_id } => {
            Some(ShellCommand::BuildMaterialPreview {
                asset_id,
                projection_epoch,
            })
        }
        MaterialSurfaceAction::BuildSelectedMaterialPreview => {
            Some(ShellCommand::BuildSelectedMaterialPreview { projection_epoch })
        }
        MaterialSurfaceAction::ClearMaterialDiagnostics => {
            Some(ShellCommand::ClearMaterialDiagnostics { projection_epoch })
        }
        action => Some(ShellCommand::ApplyMaterialSurfaceAction {
            action,
            projection_epoch,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::asset_id;

    #[test]
    fn material_graph_canvas_actions_are_epoch_guarded() {
        let epoch = 42;
        let asset_id = asset_id(7);
        let actions = [
            MaterialSurfaceAction::SelectMaterialAsset { asset_id },
            MaterialSurfaceAction::BuildMaterialPreview { asset_id },
            MaterialSurfaceAction::BuildSelectedMaterialPreview,
            MaterialSurfaceAction::ClearMaterialDiagnostics,
            MaterialSurfaceAction::PanGraph {
                delta_x: 8,
                delta_y: -4,
            },
            MaterialSurfaceAction::MoveGraphNode {
                node_id: graph::NodeId::new(3),
                delta_x: 12,
                delta_y: -6,
            },
            MaterialSurfaceAction::ConnectPorts {
                from_port_id: graph::PortId::new(4),
                to_port_id: graph::PortId::new(5),
            },
            MaterialSurfaceAction::DeleteSelectedGraphSelection,
            MaterialSurfaceAction::SetMaterialNodePaletteSearch {
                query: "noise".to_string(),
            },
            MaterialSurfaceAction::SetTextureResourceSearch {
                query: "albedo".to_string(),
            },
            MaterialSurfaceAction::SelectPreviewFixture {
                fixture: material_graph::MaterialGraphPreviewFixture::Sphere,
            },
            MaterialSurfaceAction::AssignSdfPrimitiveMaterialSlot {
                entity_id: editor_core::EntityId(9),
                slot_id: editor_scene::SceneMaterialSlotId::new(2),
            },
        ];

        for action in actions {
            let command = material_surface_action_command(action, epoch)
                .expect("material surface action should produce a shell command");
            assert_eq!(command.projection_epoch(), Some(epoch));
        }
    }

    #[test]
    fn material_graph_provider_maps_graph_canvas_action_to_material_command() {
        let provider = MaterialGraphCanvasProvider;
        let request = material_graph_request();
        let proposal = provider
            .map_interaction(
                &dispatch_context(17),
                &request,
                SurfaceInteraction::GraphCanvasAction(
                    ui_graph_editor::GraphCanvasAction::EndNodeDrag {
                        node: ui_graph_editor::GraphNodeKey(3),
                        delta: ui_graph_editor::GraphVector::new(12, -6),
                    },
                ),
            )
            .expect("graph interaction mapping should not error")
            .expect("graph interaction should map to a command proposal");

        assert!(matches!(
            proposal,
            SurfaceCommandProposal::Shell(ShellCommand::ApplyMaterialSurfaceAction {
                action: MaterialSurfaceAction::MoveGraphNode {
                    node_id,
                    delta_x: 12,
                    delta_y: -6,
                },
                projection_epoch: 17,
            }) if node_id == graph::NodeId::new(3)
        ));
    }

    #[test]
    fn material_graph_provider_maps_supported_graph_actions_through_provider_owned_route() {
        let cases = [
            (
                ui_graph_editor::GraphCanvasAction::SelectNode {
                    node: ui_graph_editor::GraphNodeKey(3),
                    additive: false,
                },
                MaterialSurfaceAction::SelectGraphNode {
                    node_id: graph::NodeId::new(3),
                },
            ),
            (
                ui_graph_editor::GraphCanvasAction::SelectEdge {
                    edge: ui_graph_editor::GraphEdgeKey(4),
                    additive: true,
                },
                MaterialSurfaceAction::SelectGraphEdge {
                    edge_id: graph::EdgeId::new(4),
                },
            ),
            (
                ui_graph_editor::GraphCanvasAction::ClearSelection,
                MaterialSurfaceAction::ClearGraphSelection,
            ),
            (
                ui_graph_editor::GraphCanvasAction::Pan {
                    phase: ui_graph_editor::GraphGesturePhase::End,
                    delta: ui_graph_editor::GraphVector::new(8, -4),
                },
                MaterialSurfaceAction::PanGraph {
                    delta_x: 8,
                    delta_y: -4,
                },
            ),
            (
                ui_graph_editor::GraphCanvasAction::Zoom {
                    anchor: ui_graph_editor::GraphPoint::new(20, 30),
                    previous_zoom_milli: 1000,
                    zoom_milli: 1250,
                },
                MaterialSurfaceAction::SetGraphZoom { zoom_milli: 1250 },
            ),
            (
                ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                    ui_graph_editor::GraphShortcutAction::AddNode,
                ),
                MaterialSurfaceAction::OpenNodePicker,
            ),
            (
                ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                    ui_graph_editor::GraphShortcutAction::Undo,
                ),
                MaterialSurfaceAction::UndoMaterialEdit,
            ),
            (
                ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                    ui_graph_editor::GraphShortcutAction::Redo,
                ),
                MaterialSurfaceAction::RedoMaterialEdit,
            ),
            (
                ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                    ui_graph_editor::GraphShortcutAction::FocusPreview,
                ),
                MaterialSurfaceAction::SelectPreviewProduct {
                    selection:
                        material_graph::MaterialGraphPreviewSelection::MaterialPreviewProduct,
                },
            ),
        ];

        for (graph_action, expected_material_action) in cases {
            assert_eq!(
                material_action_for_graph_canvas_action(graph_action.clone()),
                Some(expected_material_action.clone())
            );
            assert_eq!(
                map_graph_action_to_shell_command(graph_action, 73),
                Some(material_surface_action_command(expected_material_action, 73).unwrap())
            );
        }
    }

    #[test]
    fn material_graph_provider_maps_node_move_to_material_action() {
        assert_eq!(
            material_action_for_graph_canvas_action(
                ui_graph_editor::GraphCanvasAction::EndNodeDrag {
                    node: ui_graph_editor::GraphNodeKey(9),
                    delta: ui_graph_editor::GraphVector::new(14, -3),
                },
            ),
            Some(MaterialSurfaceAction::MoveGraphNode {
                node_id: graph::NodeId::new(9),
                delta_x: 14,
                delta_y: -3,
            })
        );
    }

    #[test]
    fn material_graph_provider_maps_edge_connect_to_material_action() {
        assert_eq!(
            material_action_for_graph_canvas_action(
                ui_graph_editor::GraphCanvasAction::EndConnection {
                    from: ui_graph_editor::GraphPortKey(11),
                    to: Some(ui_graph_editor::GraphPortKey(12)),
                },
            ),
            Some(MaterialSurfaceAction::ConnectPorts {
                from_port_id: graph::PortId::new(11),
                to_port_id: graph::PortId::new(12),
            })
        );
    }

    #[test]
    fn material_graph_provider_maps_delete_to_material_action_if_supported() {
        assert_eq!(
            material_action_for_graph_canvas_action(
                ui_graph_editor::GraphCanvasAction::KeyboardDeleteSelection
            ),
            Some(MaterialSurfaceAction::DeleteSelectedGraphSelection)
        );
        assert_eq!(
            material_action_for_graph_canvas_action(
                ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                    ui_graph_editor::GraphShortcutAction::DeleteSelection,
                ),
            ),
            Some(MaterialSurfaceAction::DeleteSelectedGraphSelection)
        );
    }

    #[test]
    fn material_graph_provider_intentionally_ignores_incomplete_graph_interactions() {
        let ignored = [
            ui_graph_editor::GraphCanvasAction::Pan {
                phase: ui_graph_editor::GraphGesturePhase::Begin,
                delta: ui_graph_editor::GraphVector::ZERO,
            },
            ui_graph_editor::GraphCanvasAction::Pan {
                phase: ui_graph_editor::GraphGesturePhase::Update,
                delta: ui_graph_editor::GraphVector::new(4, 5),
            },
            ui_graph_editor::GraphCanvasAction::BeginNodeDrag {
                node: ui_graph_editor::GraphNodeKey(1),
                start: ui_graph_editor::GraphPoint::new(0, 0),
            },
            ui_graph_editor::GraphCanvasAction::UpdateNodeDrag {
                node: ui_graph_editor::GraphNodeKey(1),
                delta: ui_graph_editor::GraphVector::new(4, 5),
            },
            ui_graph_editor::GraphCanvasAction::BeginConnection {
                from: ui_graph_editor::GraphPortKey(2),
                start: ui_graph_editor::GraphPoint::new(5, 6),
            },
            ui_graph_editor::GraphCanvasAction::UpdateConnection {
                from: ui_graph_editor::GraphPortKey(2),
                current: ui_graph_editor::GraphPoint::new(7, 8),
                hover: Some(ui_graph_editor::GraphPortKey(3)),
            },
            ui_graph_editor::GraphCanvasAction::EndConnection {
                from: ui_graph_editor::GraphPortKey(2),
                to: None,
            },
            ui_graph_editor::GraphCanvasAction::BeginMarquee {
                start: ui_graph_editor::GraphPoint::new(1, 1),
            },
            ui_graph_editor::GraphCanvasAction::UpdateMarquee {
                rect: ui_graph_editor::GraphRect::new(1, 1, 20, 20),
            },
            ui_graph_editor::GraphCanvasAction::EndMarquee {
                rect: ui_graph_editor::GraphRect::new(1, 1, 20, 20),
            },
            ui_graph_editor::GraphCanvasAction::CancelGesture,
        ];

        for graph_action in ignored {
            assert_eq!(
                material_action_for_graph_canvas_action(graph_action.clone()),
                None
            );
            assert_eq!(map_graph_action_to_shell_command(graph_action, 73), None);
        }
    }

    #[test]
    fn material_graph_provider_rejects_unsupported_graph_interaction_without_shell_mapping() {
        assert_eq!(
            map_graph_action_to_shell_command(
                ui_graph_editor::GraphCanvasAction::EndConnection {
                    from: ui_graph_editor::GraphPortKey(2),
                    to: None,
                },
                73,
            ),
            None,
            "incomplete connections fail closed at the provider and never need editor_shell semantics"
        );
    }

    #[test]
    fn material_graph_provider_preserves_epoch_when_mapping_graph_interaction() {
        let provider = MaterialGraphCanvasProvider;
        let request = material_graph_request();
        let proposal = provider
            .map_interaction(
                &dispatch_context(29),
                &request,
                SurfaceInteraction::GraphCanvasAction(
                    ui_graph_editor::GraphCanvasAction::KeyboardShortcut(
                        ui_graph_editor::GraphShortcutAction::BuildPreview,
                    ),
                ),
            )
            .expect("graph interaction mapping should not error")
            .expect("graph shortcut should map to a command proposal");

        let SurfaceCommandProposal::Shell(command) = proposal else {
            panic!("graph interaction should map to shell command proposal");
        };
        assert_eq!(command.projection_epoch(), Some(29));
        assert!(matches!(
            command,
            ShellCommand::BuildSelectedMaterialPreview {
                projection_epoch: 29
            }
        ));
    }

    #[test]
    fn material_graph_graph_canvas_interaction_dispatches_through_provider_registry() {
        let registry = EditorSurfaceProviderRegistry::runenwerk_default();
        let request = material_graph_request();
        let proposal = registry
            .map_interaction(
                &dispatch_context(41),
                &request,
                MATERIAL_GRAPH_CANVAS_PROVIDER_ID,
                SurfaceInteraction::GraphCanvasAction(ui_graph_editor::GraphCanvasAction::Pan {
                    phase: ui_graph_editor::GraphGesturePhase::End,
                    delta: ui_graph_editor::GraphVector::new(8, -4),
                }),
            )
            .expect("provider registry should dispatch graph interaction")
            .expect("graph pan should map to a command proposal");

        assert!(matches!(
            proposal,
            SurfaceCommandProposal::Shell(ShellCommand::ApplyMaterialSurfaceAction {
                action: MaterialSurfaceAction::PanGraph {
                    delta_x: 8,
                    delta_y: -4,
                },
                projection_epoch: 41,
            })
        ));
    }

    fn map_graph_action_to_shell_command(
        action: ui_graph_editor::GraphCanvasAction,
        epoch: u64,
    ) -> Option<ShellCommand> {
        let provider = MaterialGraphCanvasProvider;
        let request = material_graph_request();
        let proposal = provider
            .map_interaction(
                &dispatch_context(epoch),
                &request,
                SurfaceInteraction::GraphCanvasAction(action),
            )
            .expect("provider graph interaction mapping should not error")?;
        let SurfaceCommandProposal::Shell(command) = proposal else {
            panic!("graph interaction should map only to shell command proposals");
        };
        Some(command)
    }

    fn material_graph_request() -> SurfaceProviderRequest {
        SurfaceProviderRequest {
            mounted_unit_id: ui_composition::MountedUnitId::new(1),
            unavailable_content_policy: ui_composition::UnavailableContentPolicy::ShowFallback,
            workspace_profile_id: editor_shell::MATERIAL_WORKSPACE_PROFILE_ID,
            document_context: SurfaceDocumentContext::Resolved {
                document_id: editor_core::DocumentId(6),
                document_kind: DocumentKind::MaterialGraph,
            },
            panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(20).unwrap(),
            tab_stack_id: editor_shell::TabStackId::try_from_raw(20).unwrap(),
            tool_surface_instance_id: editor_shell::ToolSurfaceInstanceId::try_from_raw(20)
                .unwrap(),
            stable_surface_key: editor_shell::ToolSurfaceStableKey::new(
                super::MATERIAL_GRAPH_CANVAS_SURFACE_KEY,
            )
            .unwrap(),
            provider_family_id: None,
            surface_route: None,
            surface_definition_id: editor_shell::MATERIAL_GRAPH_CANVAS_SURFACE_DEFINITION_ID,
            capabilities: editor_shell::tool_surface_capability_set(
                ToolSurfaceKind::MaterialGraphCanvas,
            ),
        }
    }

    fn dispatch_context(epoch: u64) -> SurfaceProviderDispatchContext<'static> {
        SurfaceProviderDispatchContext {
            projection_epoch: epoch,
            _marker: std::marker::PhantomData,
        }
    }
}
