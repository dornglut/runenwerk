use super::diagnostics::{
    material_diagnostic, material_graph_diagnostic, material_graph_subject_from_diagnostic,
};
use super::preview_build::rebuild_material_preview_for_asset;
use super::source_resolution::resolve_material_source_for_asset;
use super::*;

impl RunenwerkEditorApp {
    pub fn select_material_asset(&mut self, asset_id: AssetId) {
        self.material_lab_runtime_mut()
            .select_material_asset(Some(asset_id));
        self.append_console_line(format!("[material] selected asset {}", asset_id.raw()));
    }

    pub fn clear_material_diagnostics(&mut self) {
        self.material_lab_runtime_mut().clear_diagnostics();
        self.append_console_line("[material] diagnostics cleared");
    }

    pub fn apply_material_surface_action(
        &mut self,
        action: MaterialSurfaceAction,
    ) -> Result<(), editor_core::EditorMutationError> {
        match action {
            MaterialSurfaceAction::SelectMaterialAsset { asset_id } => {
                self.select_material_asset(asset_id);
                Ok(())
            }
            MaterialSurfaceAction::BuildMaterialPreview { asset_id } => {
                self.rebuild_material_preview(asset_id).map_err(|_| {
                    editor_core::EditorMutationError::runtime_rejected(
                        "material preview build failed",
                    )
                })
            }
            MaterialSurfaceAction::BuildSelectedMaterialPreview => {
                self.rebuild_selected_material_preview().map_err(|_| {
                    editor_core::EditorMutationError::runtime_rejected(
                        "selected material preview build failed",
                    )
                })
            }
            MaterialSurfaceAction::ClearMaterialDiagnostics => {
                self.clear_material_diagnostics();
                Ok(())
            }
            MaterialSurfaceAction::AssignModelMeshMaterialSlot {
                model_asset_id,
                material_region_key,
                slot_id,
            } => self.assign_model_mesh_material_region_slot(
                model_asset_id,
                &material_region_key,
                slot_id,
            ),
            MaterialSurfaceAction::AssignSdfPrimitiveMaterialSlot { entity_id, slot_id } => self
                .runtime_mut()
                .assign_sdf_primitive_material_slot(entity_id, slot_id)
                .map_err(|_| {
                    editor_core::EditorMutationError::runtime_rejected(
                        "SDF primitive material slot assignment failed",
                    )
                }),
            action => self.apply_source_backed_material_edit(action),
        }
    }

    fn apply_source_backed_material_edit(
        &mut self,
        mut action: MaterialSurfaceAction,
    ) -> Result<(), editor_core::EditorMutationError> {
        if let MaterialSurfaceAction::SelectGraphNode { node_id } = action {
            self.material_lab_runtime_mut().select_graph_node(node_id);
            return Ok(());
        }
        if let MaterialSurfaceAction::SelectGraphEdge { edge_id } = action {
            self.material_lab_runtime_mut().select_graph_edge(edge_id);
            return Ok(());
        }
        if let MaterialSurfaceAction::ClearGraphSelection = action {
            self.material_lab_runtime_mut().clear_graph_selection();
            return Ok(());
        }
        if let MaterialSurfaceAction::SetMaterialNodePaletteSearch { query } = action {
            self.material_lab_runtime_mut()
                .set_node_palette_search_query(query);
            return Ok(());
        }
        if let MaterialSurfaceAction::OpenNodePicker = action {
            self.material_lab_runtime_mut().open_node_picker();
            return Ok(());
        }
        if let MaterialSurfaceAction::CloseNodePicker = action {
            self.material_lab_runtime_mut().close_node_picker();
            return Ok(());
        }
        if let MaterialSurfaceAction::SetNodePickerSearch { query } = action {
            self.material_lab_runtime_mut()
                .set_node_picker_search_query(query);
            return Ok(());
        }
        if let MaterialSurfaceAction::HighlightNodePickerNode { descriptor_key } = action {
            self.material_lab_runtime_mut()
                .highlight_node_picker_node(descriptor_key);
            return Ok(());
        }
        if let MaterialSurfaceAction::SetTextureResourceSearch { query } = action {
            self.material_lab_runtime_mut()
                .set_texture_resource_search_query(query);
            return Ok(());
        }
        if let MaterialSurfaceAction::NavigateDiagnostic { diagnostic_index } = action {
            return self.navigate_material_graph_diagnostic(diagnostic_index);
        }

        if let MaterialSurfaceAction::UndoMaterialEdit = action {
            return self.undo_material_edit();
        }
        if let MaterialSurfaceAction::RedoMaterialEdit = action {
            return self.redo_material_edit();
        }
        let close_node_picker_after_change =
            if matches!(action, MaterialSurfaceAction::ConfirmNodePickerSelection) {
                let Some(descriptor_key) = self
                    .material_lab_runtime()
                    .node_picker_highlighted_descriptor_key()
                    .map(str::to_string)
                else {
                    return Ok(());
                };
                action = MaterialSurfaceAction::AddGraphNode { descriptor_key };
                true
            } else {
                false
            };

        let Some(asset_id) = self.material_lab_runtime().selected_material_asset_id() else {
            return Err(editor_core::EditorMutationError::runtime_rejected(
                "no selected material asset for source-backed edit",
            ));
        };
        let before = self
            .load_material_graph_document_for_asset(asset_id)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to load material graph source for edit",
                )
            })?;
        let mut document = before.clone();
        let selected_nodes = self
            .material_lab_runtime()
            .selected_graph_nodes()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        let selected_edges = self
            .material_lab_runtime()
            .selected_graph_edges()
            .iter()
            .copied()
            .collect::<Vec<_>>();
        if let Err(message) = validate_texture_resource_picker_selection(
            self.asset_catalog_runtime().catalog(),
            &document,
            &action,
        ) {
            self.record_material_workflow_diagnostics([material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                message,
            )]);
            return Err(editor_core::EditorMutationError::runtime_rejected(
                "catalog texture picker selection is invalid",
            ));
        }
        let changed = apply_material_document_action(
            &mut document,
            &selected_nodes,
            &selected_edges,
            &action,
        )
        .map_err(|message| editor_core::EditorMutationError::runtime_rejected(message))?;
        if changed {
            self.material_lab_runtime_mut()
                .push_undo_snapshot(asset_id, before);
            self.write_material_graph_document_for_asset(asset_id, &document)
                .map_err(|_| {
                    editor_core::EditorMutationError::runtime_rejected(
                        "failed to persist material graph source edit",
                    )
                })?;
            if close_node_picker_after_change {
                self.material_lab_runtime_mut().close_node_picker();
            }
            self.refresh_material_source_projection(asset_id, document);
        }
        Ok(())
    }

    fn navigate_material_graph_diagnostic(
        &mut self,
        diagnostic_index: usize,
    ) -> Result<(), editor_core::EditorMutationError> {
        let diagnostic = self
            .material_lab_runtime()
            .diagnostics()
            .get(diagnostic_index)
            .cloned()
            .ok_or_else(|| {
                editor_core::EditorMutationError::runtime_rejected(
                    "material diagnostic index is not available",
                )
            })?;
        self.material_lab_runtime_mut()
            .set_active_diagnostic_index(Some(diagnostic_index));

        let (subject_node_id, subject_port_id) =
            material_graph_subject_from_diagnostic(diagnostic.subject.as_deref());
        if subject_node_id.is_none() && subject_port_id.is_none() {
            return Ok(());
        }

        let Some(asset_id) = self.material_lab_runtime().selected_material_asset_id() else {
            return Err(editor_core::EditorMutationError::runtime_rejected(
                "no selected material asset for diagnostic navigation",
            ));
        };
        let mut document = self
            .load_material_graph_document_for_asset(asset_id)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to load material graph source for diagnostic navigation",
                )
            })?;
        let target_node_id = subject_node_id.or_else(|| {
            subject_port_id.and_then(|port_id| {
                document
                    .graph
                    .nodes
                    .iter()
                    .find(|node| node.ports.iter().any(|port| port.id == port_id))
                    .map(|node| node.id)
            })
        });
        let Some(target_node_id) = target_node_id else {
            return Ok(());
        };

        center_material_graph_viewport_on_node(&mut document, target_node_id);
        self.write_material_graph_document_for_asset(asset_id, &document)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to persist material graph diagnostic navigation",
                )
            })?;
        self.material_lab_runtime_mut()
            .select_graph_node(target_node_id);
        self.material_lab_runtime_mut()
            .set_active_source_document(asset_id, document);
        Ok(())
    }

    fn undo_material_edit(&mut self) -> Result<(), editor_core::EditorMutationError> {
        let Some((asset_id, previous)) = self.material_lab_runtime_mut().pop_undo_snapshot() else {
            return Ok(());
        };
        let current = self
            .load_material_graph_document_for_asset(asset_id)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to load material graph source before undo",
                )
            })?;
        self.write_material_graph_document_for_asset(asset_id, &previous)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to persist material graph undo",
                )
            })?;
        self.material_lab_runtime_mut()
            .push_redo_snapshot(asset_id, current);
        self.refresh_material_source_projection(asset_id, previous);
        Ok(())
    }

    fn redo_material_edit(&mut self) -> Result<(), editor_core::EditorMutationError> {
        let Some((asset_id, next)) = self.material_lab_runtime_mut().pop_redo_snapshot() else {
            return Ok(());
        };
        let current = self
            .load_material_graph_document_for_asset(asset_id)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to load material graph source before redo",
                )
            })?;
        self.write_material_graph_document_for_asset(asset_id, &next)
            .map_err(|_| {
                editor_core::EditorMutationError::runtime_rejected(
                    "failed to persist material graph redo",
                )
            })?;
        self.material_lab_runtime_mut()
            .push_undo_snapshot(asset_id, current);
        self.refresh_material_source_projection(asset_id, next);
        Ok(())
    }

    pub fn rebuild_selected_material_preview(&mut self) -> Result<()> {
        let Some(asset_id) = self.material_lab_runtime().selected_material_asset_id() else {
            let diagnostic = material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                "no selected material asset to build",
            );
            self.record_material_workflow_diagnostics([diagnostic]);
            return Ok(());
        };
        self.rebuild_material_preview(asset_id)
    }

    pub fn rebuild_material_preview(&mut self, asset_id: AssetId) -> Result<()> {
        if self.asset_project_session.is_none() {
            self.record_missing_asset_project_session("build material preview");
            return Ok(());
        }

        self.material_lab_runtime_mut()
            .select_material_asset(Some(asset_id));
        if let Ok(document) = self.load_material_graph_document_for_asset(asset_id) {
            self.material_lab_runtime_mut()
                .set_active_source_document(asset_id, document);
        }
        let catalog_snapshot = self.asset_catalog_runtime().catalog().clone();
        let outcome = {
            let session = self
                .asset_project_session
                .as_mut()
                .expect("asset project session checked above");
            rebuild_material_preview_for_asset(&catalog_snapshot, session, asset_id)
        };

        self.record_material_workflow_diagnostics(outcome.diagnostics.clone());
        if let Some(publication) = outcome.publication {
            self.queue_material_preview_publication(publication);
            self.material_lab_runtime_mut()
                .set_workflow_status("preview publication queued");
        } else {
            self.material_lab_runtime_mut()
                .set_workflow_status("preview build blocked");
        }
        Ok(())
    }

    fn refresh_material_source_projection(
        &mut self,
        asset_id: AssetId,
        document: MaterialGraphDocument,
    ) {
        let lowering = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());
        let diagnostics = lowering
            .report
            .issues()
            .iter()
            .map(|issue| {
                material_graph_diagnostic(
                    issue.code(),
                    issue.subject(),
                    issue.message().to_string(),
                )
            })
            .collect::<Vec<_>>();
        self.material_lab_runtime_mut().clear_diagnostics();
        self.material_lab_runtime_mut()
            .set_active_source_document(asset_id, document);
        self.record_material_workflow_diagnostics(diagnostics);
        if lowering.report.has_blocking_issues() {
            self.material_lab_runtime_mut()
                .set_workflow_status("material source edited; ratification blocked");
        } else {
            self.material_lab_runtime_mut()
                .set_workflow_status("material source edited; ratification accepted");
        }
    }

    pub fn load_material_graph_document_for_asset(
        &self,
        asset_id: AssetId,
    ) -> Result<MaterialGraphDocument> {
        let session = self.asset_project_session().ok_or_else(|| {
            anyhow::anyhow!("cannot load material graph document: no asset project session")
        })?;
        let source =
            resolve_material_source_for_asset(self.asset_catalog_runtime().catalog(), asset_id)
                .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
        read_material_graph_document(&session.project_root().join(&source.relative_path))
    }

    pub fn write_material_graph_document_for_asset(
        &self,
        asset_id: AssetId,
        document: &MaterialGraphDocument,
    ) -> Result<()> {
        let session = self.asset_project_session().ok_or_else(|| {
            anyhow::anyhow!("cannot write material graph document: no asset project session")
        })?;
        let source =
            resolve_material_source_for_asset(self.asset_catalog_runtime().catalog(), asset_id)
                .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
        write_material_graph_document(
            &session.project_root().join(&source.relative_path),
            document,
        )
    }

    pub(super) fn record_material_workflow_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = AssetDiagnosticRecord>,
    ) {
        let diagnostics = diagnostics.into_iter().collect::<Vec<_>>();
        for diagnostic in &diagnostics {
            self.asset_catalog_runtime_mut()
                .record_diagnostic(diagnostic.clone());
        }
        self.material_lab_runtime_mut()
            .record_diagnostics(diagnostics);
    }
}

pub(super) fn apply_material_document_action(
    document: &mut MaterialGraphDocument,
    selected_nodes: &[graph::NodeId],
    selected_edges: &[graph::EdgeId],
    action: &MaterialSurfaceAction,
) -> Result<bool, &'static str> {
    match action {
        MaterialSurfaceAction::PanGraph { delta_x, delta_y } => {
            document.editor_state.viewport.pan_x += *delta_x;
            document.editor_state.viewport.pan_y += *delta_y;
            Ok(true)
        }
        MaterialSurfaceAction::SetGraphZoom { zoom_milli } => {
            document.editor_state.viewport.zoom_milli = (*zoom_milli).clamp(100, 4000);
            Ok(true)
        }
        MaterialSurfaceAction::SelectPreviewFixture { fixture } => {
            document.editor_state.selected_fixture = *fixture;
            Ok(true)
        }
        MaterialSurfaceAction::SelectPreviewProduct { selection } => {
            document.editor_state.selected_preview = *selection;
            Ok(true)
        }
        MaterialSurfaceAction::PersistMaterialLayout => Ok(true),
        MaterialSurfaceAction::AddGraphNode { descriptor_key } => {
            material_graph::add_catalog_node(
                document,
                descriptor_key,
                &MaterialNodeCatalog::first_slice(),
            )
            .map_err(|error| error.as_static_str())?;
            Ok(true)
        }
        MaterialSurfaceAction::MoveGraphNode {
            node_id,
            delta_x,
            delta_y,
        } => material_graph::move_node_layout(document, *node_id, *delta_x, *delta_y)
            .map_err(|error| error.as_static_str()),
        MaterialSurfaceAction::DeleteSelectedGraphNodes => Ok(material_graph::delete_selection(
            document,
            selected_nodes,
            &[],
        )),
        MaterialSurfaceAction::DeleteSelectedGraphSelection => Ok(
            material_graph::delete_selection(document, selected_nodes, selected_edges),
        ),
        MaterialSurfaceAction::ConnectPorts {
            from_port_id,
            to_port_id,
        } => {
            material_graph::connect_ports(document, *from_port_id, *to_port_id)
                .map_err(|error| error.as_static_str())?;
            Ok(true)
        }
        MaterialSurfaceAction::DisconnectEdge { edge_id } => {
            material_graph::disconnect_edge(document, *edge_id)
                .map_err(|error| error.as_static_str())
        }
        MaterialSurfaceAction::SetNodeValue {
            node_id,
            key,
            value,
        } => material_graph::set_node_text_value(document, *node_id, key, value.clone())
            .map_err(|error| error.as_static_str()),
        MaterialSurfaceAction::PickTextureResource {
            node_id,
            key,
            stable_id,
        } => material_graph::set_node_texture_resource(
            document,
            *node_id,
            key,
            stable_id,
            &MaterialNodeCatalog::first_slice(),
        )
        .map_err(|error| error.as_static_str()),
        _ => Err("material surface action is not a source document edit"),
    }
}

fn center_material_graph_viewport_on_node(
    document: &mut MaterialGraphDocument,
    node_id: graph::NodeId,
) {
    let (position_x, position_y) = document
        .editor_state
        .node_layouts
        .iter()
        .find(|layout| layout.node_id == node_id)
        .map(|layout| (layout.position_x, layout.position_y))
        .or_else(|| {
            document
                .graph
                .nodes
                .iter()
                .position(|node| node.id == node_id)
                .map(|index| ((index as i32 % 4) * 220, (index as i32 / 4) * 120))
        })
        .unwrap_or((0, 0));
    document.editor_state.viewport.pan_x = 320_i32.saturating_sub(position_x);
    document.editor_state.viewport.pan_y = 220_i32.saturating_sub(position_y);
}

fn validate_texture_resource_picker_selection(
    catalog: &AssetCatalog,
    document: &MaterialGraphDocument,
    action: &MaterialSurfaceAction,
) -> Result<(), String> {
    let MaterialSurfaceAction::PickTextureResource {
        node_id,
        key,
        stable_id,
    } = action
    else {
        return Ok(());
    };
    let node = document
        .graph
        .nodes
        .iter()
        .find(|node| node.id == *node_id)
        .ok_or_else(|| format!("texture picker node {} is missing", node_id.raw()))?;
    let material_node_catalog = MaterialNodeCatalog::first_slice();
    let descriptor = material_node_catalog
        .descriptor(&node.name)
        .ok_or_else(|| {
            format!(
                "texture picker node '{}' is not in the material node catalog",
                node.name
            )
        })?;
    let resource = descriptor
        .resources
        .iter()
        .find(|resource| resource.key == *key)
        .ok_or_else(|| format!("texture picker resource binding '{key}' is missing"))?;
    let expected_dimension = match resource.kind {
        material_graph::MaterialResourceKind::Texture2D => texture::TextureDimension::Texture2D,
        material_graph::MaterialResourceKind::Texture3D => {
            texture::TextureDimension::Texture3DVolume
        }
    };
    let asset_record = catalog
        .assets()
        .find(|record| record.stable_name == *stable_id)
        .ok_or_else(|| {
            format!("texture picker selection '{stable_id}' is not a catalog texture asset")
        })?;
    let Some(artifact) = asset_record
        .artifact_ids
        .iter()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .find(|artifact| {
            matches!(
                &artifact.payload_kind,
                ArtifactPayloadKind::TextureProduct { descriptor, .. }
                    | ArtifactPayloadKind::GeneratedTextureProduct { descriptor, .. }
                    if descriptor.dimension == expected_dimension
            )
        })
    else {
        return Err(format!(
            "texture picker selection '{}' has no catalog-backed {:?} product",
            stable_id, resource.kind
        ));
    };
    if artifact.validity != asset::ArtifactValidity::Valid {
        return Err(format!(
            "texture picker selection '{}' artifact {} is {:?}",
            stable_id,
            artifact.artifact_id.raw(),
            artifact.validity
        ));
    }
    let artifact_uri = match &artifact.payload_kind {
        ArtifactPayloadKind::TextureProduct { artifact_uri, .. }
        | ArtifactPayloadKind::GeneratedTextureProduct { artifact_uri, .. } => artifact_uri,
        _ => unreachable!("artifact payload was matched above"),
    };
    if artifact_uri.is_none() {
        return Err(format!(
            "texture picker selection '{}' artifact {} has no artifact_uri",
            stable_id,
            artifact.artifact_id.raw()
        ));
    }
    Ok(())
}
