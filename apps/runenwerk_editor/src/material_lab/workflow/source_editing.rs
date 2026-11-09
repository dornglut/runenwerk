use super::diagnostics::{
    material_diagnostic, material_graph_diagnostic, material_graph_subject_from_diagnostic,
};
use super::preview_build::rebuild_material_preview_for_asset;
use super::source_resolution::resolve_material_source_for_asset;
use super::*;

impl RunenwerkEditorApp {
    /// One project-session incarnation and one catalog source locator.
    /// Opaque owner provenance; not a filesystem lock or a durable revision.
    fn material_source_context_digest(&self, relative_path: &str) -> Result<[u8; 32]> {
        let session = self
            .asset_project_session()
            .ok_or_else(|| anyhow::anyhow!("no asset project session"))?;
        let root_bytes = session.project_root().as_os_str().as_encoded_bytes();
        let source_bytes = std::path::Path::new(relative_path)
            .as_os_str()
            .as_encoded_bytes();
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"runenwerk/material-source-invocation/v1");
        hasher.update(&self.material_source_context_generation.to_le_bytes());
        hasher.update(&(root_bytes.len() as u64).to_le_bytes());
        hasher.update(root_bytes);
        hasher.update(&(source_bytes.len() as u64).to_le_bytes());
        hasher.update(source_bytes);
        Ok(*hasher.finalize().as_bytes())
    }

    pub fn select_material_asset(&mut self, asset_id: AssetId) {
        self.material_lab_runtime_mut()
            .select_material_asset(Some(asset_id));
        // Source loading happens at selection, not repeatedly during frame
        // formation. Missing/invalid sources remain selected but unqualified.
        match self.load_material_graph_document_for_asset_with_digest(asset_id) {
            Ok((document, source_id, digest, context_digest)) => {
                self.material_lab_runtime_mut()
                    .set_active_source_document_with_observation(
                        asset_id, source_id, document, digest, context_digest,
                    );
            }
            Err(error) => {
                let catalog = self.asset_catalog_runtime().catalog();
                let source_declared = catalog
                    .asset(asset_id)
                    .is_some_and(|record| record.primary_source_id.is_some())
                    || catalog
                        .sources
                        .values()
                        .any(|source| source.asset_id == asset_id);
                // Previously observed project source is not silently
                // reclassified as a source-less in-memory fixture if its
                // catalog binding disappears between selections.
                let had_observation = self
                    .material_lab_runtime()
                    .active_source_observation()
                    .is_some();
                if source_declared || had_observation {
                    self.material_lab_runtime_mut()
                        .discard_failed_selected_source_reload(asset_id);
                    let diagnostic = material_source_load_diagnostic("selection", &error);
                    self.record_material_workflow_diagnostics([diagnostic]);
                }
            }
        }
        self.append_console_line(format!("[material] selected asset {}", asset_id.raw()));
    }

    pub fn clear_material_diagnostics(&mut self) {
        self.material_lab_runtime_mut().clear_diagnostics();
        self.append_console_line("[material] diagnostics cleared");
    }

    /// Captures source identity and exact bytes for a specific Material invocation.
    /// The optional surface instance scopes UI-originated actions; direct callers
    /// can retain an origin without pretending to own a mounted editor surface.
    pub fn material_source_origin_for_selected(
        &self,
        surface_instance_id: Option<editor_shell::ToolSurfaceInstanceId>,
    ) -> Result<editor_shell::MaterialSourceOrigin> {
        let asset_id = self
            .material_lab_runtime()
            .selected_material_asset_id()
            .ok_or_else(|| anyhow::anyhow!("no selected material asset"))?;
        let session = self
            .asset_project_session()
            .ok_or_else(|| anyhow::anyhow!("no asset project session"))?;
        let source =
            resolve_material_source_for_asset(self.asset_catalog_runtime().catalog(), asset_id)
                .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
        let path = session.project_root().join(&source.relative_path);
        let (document, content_digest) =
            crate::material_lab::read_material_graph_document_with_digest(&path)?;
        let source_context_digest = self.material_source_context_digest(&source.relative_path)?;
        let document_id = material_document_id_for_source(asset_id, source.source_id);
        anyhow::ensure!(
            document.document_id == document_id,
            "current Material source does not match catalog identity"
        );
        if let Some((active_asset, active_document)) =
            self.material_lab_runtime().active_source_document()
        {
            anyhow::ensure!(
                active_asset != asset_id || *active_document == document,
                "current Material source differs from the displayed edit-session document"
            );
        }
        Ok(editor_shell::MaterialSourceOrigin {
            asset_id,
            source_id: source.source_id,
            document_id,
            content_digest,
            source_context_digest,
            surface_instance_id,
        })
    }

    /// Returns only owner-observed provenance already published by a source
    /// load/edit. Frame formation performs no source filesystem access.
    /// A later mutation separately re-reads live bytes before admission.
    pub fn material_published_source_origin_for_selected(
        &self,
        surface_instance_id: Option<editor_shell::ToolSurfaceInstanceId>,
    ) -> Result<editor_shell::MaterialSourceOrigin> {
        let observation = self
            .material_lab_runtime()
            .active_source_observation()
            .ok_or_else(|| anyhow::anyhow!("selected source has no admitted observation"))?;
        let source = resolve_material_source_for_asset(
            self.asset_catalog_runtime().catalog(),
            observation.asset_id,
        )
        .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
        anyhow::ensure!(
            source.source_id == observation.source_id
                && material_document_id_for_source(observation.asset_id, source.source_id)
                    == observation.document_id
                && self.material_source_context_digest(&source.relative_path)?
                    == observation.source_context_digest,
            "catalog source/context identity no longer matches the observed Material document"
        );
        let (active_asset, active_document) = self
            .material_lab_runtime()
            .active_source_document()
            .ok_or_else(|| anyhow::anyhow!("no active Material source document"))?;
        anyhow::ensure!(
            active_asset == observation.asset_id
                && active_document.document_id == observation.document_id,
            "active Material document no longer matches source observation"
        );
        Ok(editor_shell::MaterialSourceOrigin {
            asset_id: observation.asset_id,
            source_id: observation.source_id,
            document_id: observation.document_id,
            content_digest: observation.content_digest,
            source_context_digest: observation.source_context_digest,
            surface_instance_id,
        })
    }

    fn admit_material_source_origin(
        &mut self,
        origin: editor_shell::MaterialSourceOrigin,
    ) -> Result<(), editor_core::EditorMutationError> {
        let observed_context_matches = self
            .material_lab_runtime()
            .active_source_observation()
            .is_none_or(|observation| {
                observation.source_context_digest == origin.source_context_digest
            });
        let current = self.material_source_origin_for_selected(origin.surface_instance_id);
        if observed_context_matches && current.as_ref().is_ok_and(|current| *current == origin) {
            return Ok(());
        }
        self.record_material_workflow_diagnostics([material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            "Material invocation source identity/revision no longer matches the selected source",
        )]);
        Err(editor_core::EditorMutationError::runtime_rejected(
            "Material source invocation is stale or missing",
        ))
    }

    pub fn apply_material_surface_action(
        &mut self,
        action: MaterialSurfaceAction,
    ) -> Result<(), editor_core::EditorMutationError> {
        match action {
            MaterialSurfaceAction::SourceBound { origin, action } => {
                if !action.requires_source_origin() {
                    return Err(editor_core::EditorMutationError::runtime_rejected(
                        "a source-bound Material action must be a source-scoped operation",
                    ));
                }
                self.admit_material_source_origin(origin)?;
                let history_origin = match action.as_ref() {
                    MaterialSurfaceAction::UndoMaterialEdit => {
                        self.material_lab_runtime().next_undo_source_identity()
                    }
                    MaterialSurfaceAction::RedoMaterialEdit => {
                        self.material_lab_runtime().next_redo_source_identity()
                    }
                    _ => None,
                };
                if history_origin
                    .is_some_and(|identity| identity != (origin.asset_id, origin.document_id))
                {
                    self.record_material_workflow_diagnostics([material_diagnostic(
                        AssetDiagnosticCode::RatificationRejected,
                        "Material history operation belongs to a different source document",
                    )]);
                    return Err(editor_core::EditorMutationError::runtime_rejected(
                        "Material history source differs from the invocation source",
                    ));
                }
                self.apply_qualified_material_surface_action(*action)
            }
            action if action.requires_source_origin() => {
                self.record_material_workflow_diagnostics([material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    "Material source action requires its originating source",
                )]);
                Err(editor_core::EditorMutationError::runtime_rejected(
                    "Material source action requires its originating source",
                ))
            }
            action => self.apply_qualified_material_surface_action(action),
        }
    }

    fn apply_qualified_material_surface_action(
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
        let before = self.load_material_graph_document_for_action(
            asset_id,
            "edit",
            "failed to load material graph source for edit",
        )?;
        if self
            .material_lab_runtime()
            .active_source_document()
            .is_some_and(|(active_asset, active_document)| {
                active_asset == asset_id && active_document.document_id != before.document_id
            })
        {
            self.record_material_workflow_diagnostics([material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                "selected material graph source changed since the active edit session",
            )]);
            return Err(editor_core::EditorMutationError::runtime_rejected(
                "material graph source changed; reload before editing",
            ));
        }
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
        .map_err(editor_core::EditorMutationError::runtime_rejected)?;
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
        let mut document = self.load_material_graph_document_for_action(
            asset_id,
            "diagnostic navigation",
            "failed to load material graph source for diagnostic navigation",
        )?;
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
        // Diagnostic navigation writes source viewport state too. Re-observe
        // exactly the bytes just published before issuing a new UI origin.
        // Otherwise the canvas loses its admitted source after navigation.
        match self.load_material_graph_document_for_asset_with_digest(asset_id) {
            Ok((observed, source_id, digest, context_digest)) if observed == document => {
                self.material_lab_runtime_mut()
                    .set_active_source_document_with_observation(
                        asset_id, source_id, document, digest, context_digest,
                    );
                self.material_lab_runtime_mut()
                    .select_graph_node(target_node_id);
                self.material_lab_runtime_mut()
                    .set_active_diagnostic_index(Some(diagnostic_index));
                Ok(())
            }
            _ => {
                self.material_lab_runtime_mut()
                    .discard_failed_selected_source_reload(asset_id);
                self.record_material_workflow_diagnostics([material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    "material source changed before diagnostic navigation could be observed",
                )]);
                Err(editor_core::EditorMutationError::runtime_rejected(
                    "material graph source could not be observed after diagnostic navigation",
                ))
            }
        }
    }

    fn undo_material_edit(&mut self) -> Result<(), editor_core::EditorMutationError> {
        let Some((asset_id, previous)) = self.material_lab_runtime_mut().pop_undo_snapshot() else {
            return Ok(());
        };
        let current = self.load_material_graph_document_for_action(
            asset_id,
            "undo",
            "failed to load material graph source before undo",
        )?;
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
        let current = self.load_material_graph_document_for_action(
            asset_id,
            "redo",
            "failed to load material graph source before redo",
        )?;
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
        if let Ok((document, source_id, digest, context_digest)) =
            self.load_material_graph_document_for_asset_with_digest(asset_id)
        {
            self.material_lab_runtime_mut()
                .set_active_source_document_with_observation(
                    asset_id,
                    source_id,
                    document,
                    digest,
                    context_digest,
                );
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
        let observed = self
            .load_material_graph_document_for_asset_with_digest(asset_id)
            .ok()
            .filter(|(reloaded, _, _, _)| *reloaded == document);
        self.material_lab_runtime_mut().clear_diagnostics();
        if let Some((_, source_id, digest, context_digest)) = observed {
            self.material_lab_runtime_mut()
                .set_active_source_document_with_observation(
                    asset_id,
                    source_id,
                    document,
                    digest,
                    context_digest,
                );
        } else {
            // Source may have changed immediately after publication; do not
            // manufacture a current revision token for an unobserved file.
            self.material_lab_runtime_mut()
                .set_active_source_document(asset_id, document);
        }
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
        self.load_material_graph_document_for_asset_with_digest(asset_id)
            .map(|(document, _, _, _)| document)
    }

    fn load_material_graph_document_for_asset_with_digest(
        &self,
        asset_id: AssetId,
    ) -> Result<(
        MaterialGraphDocument,
        asset::AssetSourceId,
        [u8; 32],
        [u8; 32],
    )> {
        let session = self.asset_project_session().ok_or_else(|| {
            anyhow::anyhow!("cannot load material graph document: no asset project session")
        })?;
        let source =
            resolve_material_source_for_asset(self.asset_catalog_runtime().catalog(), asset_id)
                .map_err(|diagnostic| anyhow::anyhow!(diagnostic.message))?;
        let (document, digest) = crate::material_lab::read_material_graph_document_with_digest(
            &session.project_root().join(&source.relative_path),
        )?;
        let expected_document_id = material_document_id_for_source(asset_id, source.source_id);
        anyhow::ensure!(
            document.document_id == expected_document_id,
            "material graph source document id {} does not match asset/source identity {}",
            document.document_id.raw(),
            expected_document_id.raw()
        );
        let context_digest = self.material_source_context_digest(&source.relative_path)?;
        Ok((document, source.source_id, digest, context_digest))
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
        let expected_document_id = material_document_id_for_source(asset_id, source.source_id);
        anyhow::ensure!(
            document.document_id == expected_document_id,
            "material graph source document id {} does not match asset/source identity {}",
            document.document_id.raw(),
            expected_document_id.raw()
        );
        let path = session.project_root().join(&source.relative_path);
        if path.exists() {
            let current = read_material_graph_document(&path)?;
            anyhow::ensure!(
                current.document_id == expected_document_id,
                "existing material graph source document id {} does not match asset/source identity {}",
                current.document_id.raw(),
                expected_document_id.raw()
            );
        }
        write_material_graph_document(&path, document)
    }

    fn load_material_graph_document_for_action(
        &mut self,
        asset_id: AssetId,
        action: &'static str,
        rejection: &'static str,
    ) -> Result<MaterialGraphDocument, editor_core::EditorMutationError> {
        match self.load_material_graph_document_for_asset(asset_id) {
            Ok(document) => Ok(document),
            Err(error) => {
                self.record_material_workflow_diagnostics([material_source_load_diagnostic(
                    action, &error,
                )]);
                Err(editor_core::EditorMutationError::runtime_rejected(
                    rejection,
                ))
            }
        }
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

fn material_source_load_diagnostic(action: &str, error: &anyhow::Error) -> AssetDiagnosticRecord {
    let code = if error_chain_contains(error, "failed to read material graph source") {
        AssetDiagnosticCode::SourceMissing
    } else {
        AssetDiagnosticCode::RatificationRejected
    };
    material_diagnostic(
        code,
        format!(
            "failed to load material graph source for {action}: {}",
            error_chain_summary(error)
        ),
    )
}

fn error_chain_contains(error: &anyhow::Error, needle: &str) -> bool {
    error
        .chain()
        .any(|cause| cause.to_string().contains(needle))
}

fn error_chain_summary(error: &anyhow::Error) -> String {
    error
        .chain()
        .map(|cause| cause.to_string())
        .collect::<Vec<_>>()
        .join(": ")
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
