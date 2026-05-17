use std::path::{Path, PathBuf};

use anyhow::Result;
use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, AssetArtifactDescriptor, AssetCatalog,
    AssetDiagnosticCode, AssetDiagnosticRecord, AssetDiagnosticSeverity, AssetId, AssetKind,
    AssetSourceDescriptor, ImportPlan, deterministic_cache_key, ratify_asset_catalog,
    ratify_asset_import_plan_against_source, try_preserve_prior_valid_artifact,
};
use editor_shell::MaterialSurfaceAction;
use engine::plugins::render::{
    MaterialPreviewFixture, MaterialShaderCompileRequest, compile_material_shader,
};
use material_graph::{
    MaterialGraphDocument, MaterialGraphIssueCode, MaterialGraphIssueSubject,
    MaterialGraphNodeLayout, MaterialNodeCatalog, MaterialOutputTarget, MaterialResourceKind,
    lower_material_graph,
};
use product::ProductPublicationOutcome;

use crate::editor_app::RunenwerkEditorApp;
use crate::material_lab::{
    EditorMaterialPreviewProduct, EditorMaterialPreviewPublication, ResolvedMaterialLoweringRecipe,
    material_document_id_for_source, material_product_id_for_import_job,
    previous_valid_material_artifact, read_material_graph_document, resolve_material_resources,
    write_material_graph_document,
};

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
            action => self.apply_source_backed_material_edit(action),
        }
    }

    fn apply_source_backed_material_edit(
        &mut self,
        action: MaterialSurfaceAction,
    ) -> Result<(), editor_core::EditorMutationError> {
        if let MaterialSurfaceAction::SelectGraphNode { node_id } = action {
            self.material_lab_runtime_mut().select_graph_node(node_id);
            return Ok(());
        }

        if let MaterialSurfaceAction::UndoMaterialEdit = action {
            return self.undo_material_edit();
        }
        if let MaterialSurfaceAction::RedoMaterialEdit = action {
            return self.redo_material_edit();
        }

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
        let changed = apply_material_document_action(&mut document, &selected_nodes, &action)
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
            self.refresh_material_source_projection(asset_id, document);
        }
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

    fn record_material_workflow_diagnostics(
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewBuildOutcome {
    pub publication: Option<EditorMaterialPreviewPublication>,
    pub diagnostics: Vec<AssetDiagnosticRecord>,
}

fn apply_material_document_action(
    document: &mut MaterialGraphDocument,
    selected_nodes: &[graph::NodeId],
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
            add_material_graph_node(document, descriptor_key)?;
            Ok(true)
        }
        MaterialSurfaceAction::DeleteSelectedGraphNodes => {
            if selected_nodes.is_empty() {
                return Ok(false);
            }
            delete_material_graph_nodes(document, selected_nodes);
            Ok(true)
        }
        MaterialSurfaceAction::ConnectPorts {
            from_port_id,
            to_port_id,
        } => {
            let edge_id = next_edge_id(document);
            document.graph.edges.push(graph::EdgeDefinition::new(
                edge_id,
                *from_port_id,
                *to_port_id,
            ));
            Ok(true)
        }
        MaterialSurfaceAction::DisconnectEdge { edge_id } => {
            document.graph.edges.retain(|edge| edge.id != *edge_id);
            Ok(true)
        }
        MaterialSurfaceAction::SetNodeValue {
            node_id,
            key,
            value,
        } => {
            let Some(node) = document
                .graph
                .nodes
                .iter_mut()
                .find(|node| node.id == *node_id)
            else {
                return Err("material graph node is missing");
            };
            set_node_graph_value(node, key, graph::GraphValue::Text(value.clone()));
            Ok(true)
        }
        MaterialSurfaceAction::PickTextureResource {
            node_id,
            key,
            stable_id,
        } => {
            let Some(node) = document
                .graph
                .nodes
                .iter_mut()
                .find(|node| node.id == *node_id)
            else {
                return Err("material graph node is missing");
            };
            let catalog = MaterialNodeCatalog::first_slice();
            let Some(descriptor) = catalog.descriptor(&node.name) else {
                return Err("material graph node is not in the active catalog");
            };
            let resource_kind = descriptor
                .resources
                .iter()
                .find(|resource| resource.key == *key)
                .map(|resource| resource.kind)
                .ok_or("material graph node resource binding is missing")?;
            let kind = match resource_kind {
                MaterialResourceKind::Texture2D => "asset.catalog.texture2d",
                MaterialResourceKind::Texture3D => "asset.catalog.texture3d",
            };
            let reference = resource_ref::ResourceRef::new(kind, stable_id.as_str())
                .map_err(|_| "material graph texture resource reference is invalid")?;
            set_node_graph_value(node, key, graph::GraphValue::Resource(reference));
            Ok(true)
        }
        _ => Err("material surface action is not a source document edit"),
    }
}

fn add_material_graph_node(
    document: &mut MaterialGraphDocument,
    descriptor_key: &str,
) -> Result<(), &'static str> {
    let catalog = MaterialNodeCatalog::first_slice();
    let descriptor = catalog
        .descriptor(descriptor_key)
        .ok_or("material node descriptor is not in the active catalog")?;
    let node_id = next_node_id(document);
    let mut next_port = next_port_id(document).raw();
    let mut ports = Vec::new();
    for input in &descriptor.inputs {
        ports.push(graph::PortDefinition::new(
            graph::PortId::new(next_port),
            input.name.clone(),
            graph::PortDirection::Input,
            input.value_type.port_type_id(),
        ));
        next_port += 1;
    }
    for output in &descriptor.outputs {
        ports.push(graph::PortDefinition::new(
            graph::PortId::new(next_port),
            output.name.clone(),
            graph::PortDirection::Output,
            output.value_type.port_type_id(),
        ));
        next_port += 1;
    }
    let node = graph::NodeDefinition::new(node_id, descriptor.key.clone(), ports);
    document.graph.nodes.push(node);
    document
        .editor_state
        .node_layouts
        .push(MaterialGraphNodeLayout::new(
            node_id,
            (document.graph.nodes.len() as i32 % 4) * 220,
            (document.graph.nodes.len() as i32 / 4) * 120,
        ));
    Ok(())
}

fn delete_material_graph_nodes(document: &mut MaterialGraphDocument, node_ids: &[graph::NodeId]) {
    let node_ids = node_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let deleted_ports = document
        .graph
        .nodes
        .iter()
        .filter(|node| node_ids.contains(&node.id))
        .flat_map(|node| node.ports.iter().map(|port| port.id))
        .collect::<std::collections::BTreeSet<_>>();
    document
        .graph
        .nodes
        .retain(|node| !node_ids.contains(&node.id));
    document.graph.edges.retain(|edge| {
        !deleted_ports.contains(&edge.from_port) && !deleted_ports.contains(&edge.to_port)
    });
    document
        .editor_state
        .node_layouts
        .retain(|layout| !node_ids.contains(&layout.node_id));
}

fn set_node_graph_value(node: &mut graph::NodeDefinition, key: &str, value: graph::GraphValue) {
    if let Some(entry) = node.values.iter_mut().find(|entry| entry.key == key) {
        entry.value = value;
    } else {
        node.values
            .push(graph::GraphMetadataEntry::new(key.to_string(), value));
    }
}

fn next_node_id(document: &MaterialGraphDocument) -> graph::NodeId {
    graph::NodeId::new(
        document
            .graph
            .nodes
            .iter()
            .map(|node| node.id.raw())
            .max()
            .unwrap_or(0)
            + 1,
    )
}

fn next_port_id(document: &MaterialGraphDocument) -> graph::PortId {
    graph::PortId::new(
        document
            .graph
            .nodes
            .iter()
            .flat_map(|node| node.ports.iter().map(|port| port.id.raw()))
            .max()
            .unwrap_or(0)
            + 1,
    )
}

fn next_edge_id(document: &MaterialGraphDocument) -> graph::EdgeId {
    graph::EdgeId::new(
        document
            .graph
            .edges
            .iter()
            .map(|edge| edge.id.raw())
            .max()
            .unwrap_or(0)
            + 1,
    )
}

pub fn rebuild_material_preview_for_asset(
    catalog: &AssetCatalog,
    session: &mut crate::asset_pipeline::EditorAssetProjectSession,
    asset_id: AssetId,
) -> EditorMaterialPreviewBuildOutcome {
    match crate::asset_pipeline::EditorImportExecutionLedger::load_from_path(
        &session.import_ledger_path(),
    ) {
        Ok(ledger) => session.replace_import_ledger(ledger),
        Err(error) => {
            return blocked_preview(format!(
                "failed to load import execution ledger; material preview publication blocked: {error}"
            ));
        }
    }

    let source = match resolve_material_source_for_asset(catalog, asset_id).cloned() {
        Ok(source) => source,
        Err(diagnostic) => {
            return preserved_or_blocked(catalog, asset_id, diagnostic);
        }
    };

    let recipe = match session
        .import_profile_registry()
        .resolve_for_source(&source)
    {
        Ok(recipe) => recipe,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    let material_recipe = match ResolvedMaterialLoweringRecipe::resolve(&source, &recipe) {
        Ok(recipe) => recipe,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };

    let source_path = session.project_root().join(&source.relative_path);
    let document = match read_material_graph_document(&source_path) {
        Ok(document) => document,
        Err(error) => {
            return preserved_or_blocked(
                catalog,
                asset_id,
                material_diagnostic(
                    AssetDiagnosticCode::SourceMissing,
                    format!(
                        "material graph source could not be read: {} ({error})",
                        source.relative_path
                    ),
                ),
            );
        }
    };

    let expected_document_id = material_document_id_for_source(source.asset_id, source.source_id);
    if document.document_id != expected_document_id {
        return preserved_or_blocked(
            catalog,
            asset_id,
            material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                format!(
                    "material graph document id {} does not match asset/source identity {}",
                    document.document_id.raw(),
                    expected_document_id.raw()
                ),
            ),
        );
    }
    if let Err(diagnostic) = material_recipe.validate_document_output_target(document.output_target)
    {
        return preserved_or_blocked(catalog, asset_id, diagnostic);
    }

    let node_catalog = material_recipe.node_catalog();
    let lowering = lower_material_graph(&document, &node_catalog);
    if lowering.report.has_blocking_issues() {
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
        return preserved_or_blocked_many(catalog, asset_id, diagnostics);
    }
    let Some(product) = lowering.product else {
        return blocked_preview("material graph lowering produced no formed material product");
    };
    let Some(executable_ir) = product.executable_ir.as_ref() else {
        return blocked_preview("material graph lowering did not produce executable material IR");
    };
    let resolved_resources = match resolve_material_resources(catalog, executable_ir) {
        Ok(resources) => resources,
        Err(diagnostics) => return preserved_or_blocked_many(catalog, asset_id, diagnostics),
    };

    let import_cache_key = deterministic_cache_key(
        &source,
        &recipe.settings,
        material_recipe.expected_artifact_kind,
    );
    let artifact_cache_key = ArtifactCacheKey::new(format!(
        "{}:{}:formed_material={}",
        import_cache_key.as_str(),
        material_recipe.cache_key_component,
        product.cache_key.as_str()
    ));
    let entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(artifact_cache_key.clone(), catalog)
    {
        Ok(entry) => entry,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    if let Err(error) = session
        .import_ledger()
        .save_to_path(&session.import_ledger_path())
    {
        return blocked_preview(format!(
            "failed to persist import execution ledger: {error}"
        ));
    }

    let product = product.with_product_id(material_product_id_for_import_job(entry.job_id));
    let executable_ir = product
        .executable_ir
        .as_ref()
        .expect("executable IR checked before ledger allocation");
    let compiled_shader = match compile_material_shader(MaterialShaderCompileRequest {
        ir: executable_ir,
        fixture: MaterialPreviewFixture::Sphere,
    }) {
        Ok(shader) => shader,
        Err(error) => {
            return preserved_or_blocked(
                catalog,
                asset_id,
                material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    format!("material shader compilation failed: {error}"),
                ),
            );
        }
    };
    let shader_cache_key = ArtifactCacheKey::new(format!(
        "material-shader-v1:{}:{}",
        artifact_cache_key.as_str(),
        compiled_shader.identity
    ));
    let scene_shader_cache_key = ArtifactCacheKey::new(format!(
        "material-scene-shader-v1:{}:{}",
        artifact_cache_key.as_str(),
        compiled_shader.scene_identity
    ));
    let shader_entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(shader_cache_key.clone(), catalog)
    {
        Ok(entry) => entry,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    let scene_shader_entry = match session
        .import_ledger_mut()
        .resolve_or_allocate(scene_shader_cache_key.clone(), catalog)
    {
        Ok(entry) => entry,
        Err(diagnostics) => {
            return EditorMaterialPreviewBuildOutcome {
                publication: None,
                diagnostics,
            };
        }
    };
    if let Err(error) = session
        .import_ledger()
        .save_to_path(&session.import_ledger_path())
    {
        return blocked_preview(format!(
            "failed to persist import execution ledger after shader allocation: {error}"
        ));
    }
    let shader_relative_path = material_shader_artifact_path(&shader_cache_key);
    let shader_absolute_path = session.project_root().join(&shader_relative_path);
    if let Err(error) = write_material_shader_artifact(&shader_absolute_path, &compiled_shader.wgsl)
    {
        return blocked_preview(format!(
            "failed to write generated material shader artifact: {error}"
        ));
    }
    let scene_shader_relative_path = material_scene_shader_artifact_path(&scene_shader_cache_key);
    let scene_shader_absolute_path = session.project_root().join(&scene_shader_relative_path);
    if let Err(error) =
        write_material_shader_artifact(&scene_shader_absolute_path, &compiled_shader.scene_wgsl)
    {
        return blocked_preview(format!(
            "failed to write generated material scene shader artifact: {error}"
        ));
    }
    let plan = ImportPlan::deterministic(
        entry.job_id,
        &source,
        recipe.settings.clone(),
        material_recipe.expected_artifact_kind,
    );
    let report = ratify_asset_import_plan_against_source(&plan, &source);
    if report.has_blocking_issues() {
        return EditorMaterialPreviewBuildOutcome {
            publication: None,
            diagnostics: report
                .issues()
                .iter()
                .map(|issue| {
                    material_diagnostic(
                        AssetDiagnosticCode::RatificationRejected,
                        format!(
                            "material import plan rejected {:?}: {}",
                            issue.code(),
                            issue.message()
                        ),
                    )
                })
                .collect(),
        };
    }
    let Some(product_job) = plan.product_job.clone() else {
        return blocked_preview("material preview import plan did not create a product job");
    };

    let artifact = AssetArtifactDescriptor::new(
        entry.artifact_id,
        source.asset_id,
        material_recipe.expected_artifact_kind,
        ArtifactPayloadKind::FormedMaterialProduct {
            product_id: product.product_id.raw().to_string(),
        },
        artifact_cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(material_artifact_path(&entry.artifact_id));
    let shader_artifact = AssetArtifactDescriptor::new(
        shader_entry.artifact_id,
        source.asset_id,
        AssetKind::Shader,
        ArtifactPayloadKind::ShaderMetadata,
        shader_cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(shader_relative_path.clone());
    let scene_shader_artifact = AssetArtifactDescriptor::new(
        scene_shader_entry.artifact_id,
        source.asset_id,
        AssetKind::Shader,
        ArtifactPayloadKind::ShaderMetadata,
        scene_shader_cache_key.clone(),
    )
    .with_source(source.source_id, source.revision_id)
    .with_artifact_path(scene_shader_relative_path.clone());
    if let Err(diagnostics) = catalog_with_material_artifacts(
        catalog,
        [
            artifact.clone(),
            shader_artifact.clone(),
            scene_shader_artifact.clone(),
        ],
    ) {
        return EditorMaterialPreviewBuildOutcome {
            publication: None,
            diagnostics,
        };
    }

    let preview = EditorMaterialPreviewProduct::new(
        source.asset_id,
        source.source_id,
        entry.artifact_id,
        artifact_cache_key,
        product.clone(),
        material_recipe.renderer_parameter_profile,
        shader_entry.artifact_id,
        shader_cache_key,
        canonical_shader_registry_path(session.project_root(), &shader_relative_path),
        compiled_shader.identity,
        scene_shader_entry.artifact_id,
        scene_shader_cache_key,
        canonical_shader_registry_path(session.project_root(), &scene_shader_relative_path),
        compiled_shader.scene_identity,
        resolved_resources,
    );
    let publication = ProductPublicationOutcome::ready(
        product_job,
        [product.product_core.clone()],
        entry.job_id.raw(),
    );

    EditorMaterialPreviewBuildOutcome {
        publication: Some(EditorMaterialPreviewPublication::ready(
            publication,
            preview,
            artifact,
            vec![shader_artifact, scene_shader_artifact],
        )),
        diagnostics: Vec::new(),
    }
}

pub fn material_source_for_asset(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Option<&AssetSourceDescriptor> {
    resolve_material_source_for_asset(catalog, asset_id).ok()
}

pub fn resolve_material_source_for_asset(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Result<&AssetSourceDescriptor, AssetDiagnosticRecord> {
    let Some(record) = catalog.asset(asset_id) else {
        return Err(material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            format!("asset {} is not present in the catalog", asset_id.raw()),
        ));
    };
    if record.kind != AssetKind::MaterialGraph {
        return Err(material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            format!(
                "asset {} is {:?}, not a material graph asset",
                asset_id.raw(),
                record.kind
            ),
        ));
    }
    if let Some(primary_source_id) = record.primary_source_id {
        let Some(source) = catalog.source(primary_source_id) else {
            return Err(material_diagnostic(
                AssetDiagnosticCode::SourceMissing,
                format!(
                    "asset {} primary material graph source {} is missing",
                    asset_id.raw(),
                    primary_source_id.raw()
                ),
            ));
        };
        if source.asset_id != asset_id || source.kind != AssetKind::MaterialGraph {
            return Err(material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                format!(
                    "asset {} primary source {} is {:?} for asset {}",
                    asset_id.raw(),
                    source.source_id.raw(),
                    source.kind,
                    source.asset_id.raw()
                ),
            ));
        }
        return Ok(source);
    }

    let mut material_sources = catalog
        .sources
        .values()
        .filter(|source| source.asset_id == asset_id && source.kind == AssetKind::MaterialGraph);
    let Some(first) = material_sources.next() else {
        return Err(material_diagnostic(
            AssetDiagnosticCode::SourceMissing,
            format!(
                "asset {} has no material graph source descriptor",
                asset_id.raw()
            ),
        ));
    };
    if material_sources.next().is_some() {
        return Err(material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            format!(
                "asset {} has multiple material graph sources and no primary source",
                asset_id.raw()
            ),
        ));
    }
    Ok(first)
}

pub fn default_material_graph_document_for_source(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
    label: impl Into<String>,
) -> MaterialGraphDocument {
    default_material_graph_document_for_source_with_target(
        asset_id,
        source,
        label,
        MaterialOutputTarget::PbrPreview,
    )
}

pub fn default_material_graph_document_for_source_with_target(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
    label: impl Into<String>,
    output_target: MaterialOutputTarget,
) -> MaterialGraphDocument {
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId, PortDefinition,
        PortDirection, PortId, PortTypeId,
    };
    MaterialGraphDocument::new(
        material_document_id_for_source(asset_id, source.source_id),
        label,
        GraphDefinition::new(
            GraphId::new(1),
            "material.preview",
            CyclePolicy::RejectDirectedCycles,
            [NodeDefinition::new(
                NodeId::new(1),
                "pbr.output",
                [PortDefinition::new(
                    PortId::new(1),
                    "base_color",
                    PortDirection::Input,
                    PortTypeId::new(1),
                )],
            )],
            [],
        ),
        output_target,
    )
}

pub fn catalog_with_material_artifact(
    catalog: &AssetCatalog,
    artifact: AssetArtifactDescriptor,
) -> Result<AssetCatalog, Vec<AssetDiagnosticRecord>> {
    let mut candidate = catalog.clone();
    candidate.insert_artifact(artifact);
    let report = ratify_asset_catalog(&candidate);
    if report.has_blocking_issues() {
        return Err(report
            .issues()
            .iter()
            .map(|issue| {
                material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "material preview catalog publication rejected {:?}: {}",
                        issue.code(),
                        issue.message()
                    ),
                )
            })
            .collect());
    }
    Ok(candidate)
}

pub fn catalog_with_material_artifacts(
    catalog: &AssetCatalog,
    artifacts: impl IntoIterator<Item = AssetArtifactDescriptor>,
) -> Result<AssetCatalog, Vec<AssetDiagnosticRecord>> {
    let mut candidate = catalog.clone();
    for artifact in artifacts {
        candidate.insert_artifact(artifact);
    }
    let report = ratify_asset_catalog(&candidate);
    if report.has_blocking_issues() {
        return Err(report
            .issues()
            .iter()
            .map(|issue| {
                material_diagnostic(
                    AssetDiagnosticCode::RatificationRejected,
                    format!(
                        "material preview catalog publication rejected {:?}: {}",
                        issue.code(),
                        issue.message()
                    ),
                )
            })
            .collect());
    }
    Ok(candidate)
}

fn preserved_or_blocked(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    diagnostic: AssetDiagnosticRecord,
) -> EditorMaterialPreviewBuildOutcome {
    preserved_or_blocked_many(catalog, asset_id, vec![diagnostic])
}

fn preserved_or_blocked_many(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    diagnostics: Vec<AssetDiagnosticRecord>,
) -> EditorMaterialPreviewBuildOutcome {
    let Some(first) = diagnostics.first().cloned() else {
        return blocked_preview("material preview build failed without diagnostics");
    };
    let publication = previous_valid_material_artifact(catalog, asset_id)
        .and_then(|previous| try_preserve_prior_valid_artifact(previous, first).ok())
        .map(EditorMaterialPreviewPublication::failed_preserved);
    EditorMaterialPreviewBuildOutcome {
        publication,
        diagnostics,
    }
}

fn blocked_preview(message: impl Into<String>) -> EditorMaterialPreviewBuildOutcome {
    EditorMaterialPreviewBuildOutcome {
        publication: None,
        diagnostics: vec![material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            message,
        )],
    }
}

fn material_graph_diagnostic(
    code: &MaterialGraphIssueCode,
    subject: &MaterialGraphIssueSubject,
    message: String,
) -> AssetDiagnosticRecord {
    material_diagnostic(
        AssetDiagnosticCode::RatificationRejected,
        format!("material graph rejected {code:?} {subject:?}: {message}"),
    )
}

fn material_diagnostic(
    code: AssetDiagnosticCode,
    message: impl Into<String>,
) -> AssetDiagnosticRecord {
    AssetDiagnosticRecord::new(code, AssetDiagnosticSeverity::Error, message)
}

fn material_artifact_path(artifact_id: &asset::AssetArtifactId) -> String {
    format!(
        ".runenwerk/artifacts/material-preview-{}.artifact.ron",
        artifact_id.raw()
    )
}

fn material_shader_artifact_path(cache_key: &ArtifactCacheKey) -> String {
    content_addressed_artifact_path("material-shader", cache_key, "wgsl")
}

fn material_scene_shader_artifact_path(cache_key: &ArtifactCacheKey) -> String {
    content_addressed_artifact_path("material-scene-shader", cache_key, "wgsl")
}

fn content_addressed_artifact_path(
    prefix: &str,
    cache_key: &ArtifactCacheKey,
    ext: &str,
) -> String {
    let digest = blake3::hash(cache_key.as_str().as_bytes());
    format!(
        ".runenwerk/artifacts/generated/{prefix}/{}.{}",
        digest.to_hex(),
        ext
    )
}

fn write_material_shader_artifact(path: &Path, wgsl: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, wgsl)?;
    Ok(())
}

fn canonical_shader_registry_path(project_root: &Path, relative_path: &str) -> String {
    project_root
        .join(relative_path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[allow(dead_code)]
fn project_relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

#[allow(dead_code)]
fn absolute_source_path(project_root: &Path, source: &AssetSourceDescriptor) -> PathBuf {
    project_root.join(&source.relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{
        AssetProjectCatalogDescriptor, AssetRecord, AssetSourceDescriptor, AssetSourceRoot,
        AssetSourceRootKind, ImportSettings, SourceHash, asset_artifact_id, asset_id,
        asset_source_id, asset_source_root_id,
    };
    use editor_persistence::{
        ProjectFileV3, ProjectImportProfileDefaultV3, ProjectImportProfileDefinitionV3,
    };

    #[test]
    fn material_preview_build_lowers_source_document_and_queues_publication() {
        let root = unique_temp_dir("material_preview_build");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let mut project = ProjectFileV3::new("project.material", "Material");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::MaterialGraph,
                "render",
                ImportSettings::MaterialGraph {
                    lowering_target: "render_material".to_string(),
                },
                AssetKind::Material,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::MaterialGraph,
                "render",
            ));
        let mut session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(&root, &project)
                .expect("project session should form");
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let document = default_material_graph_document_for_source_with_target(
            asset_id,
            &source,
            "Rock",
            MaterialOutputTarget::RenderMaterial,
        );
        write_material_graph_document(&root.join(&source.relative_path), &document)
            .expect("source document should write");
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(outcome.diagnostics.is_empty());
        let publication = outcome.publication.expect("preview should queue");
        assert_eq!(publication.status, product::ProductPublicationStatus::Ready);
        assert_eq!(
            publication.preview.as_ref().map(|preview| preview.asset_id),
            Some(asset_id)
        );
        assert!(
            publication
                .preview
                .as_ref()
                .expect("preview")
                .shader_path
                .starts_with(
                    &root
                        .to_string_lossy()
                        .replace(std::path::MAIN_SEPARATOR, "/")
                ),
            "runtime shader registry path must be project-root aware"
        );
        assert_eq!(session.import_ledger().len(), 3);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn material_recipe_rejection_blocks_before_ledger_allocation() {
        let root = unique_temp_dir("material_recipe_rejected");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let mut project = ProjectFileV3::new("project.material", "Material");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::MaterialGraph,
                "bad",
                ImportSettings::MaterialGraph {
                    lowering_target: String::new(),
                },
                AssetKind::Material,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::MaterialGraph,
                "bad",
            ));
        let mut session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(&root, &project)
                .expect("project session should form");
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(outcome.publication.is_none());
        assert_eq!(session.import_ledger().len(), 0);
        assert!(
            outcome.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == AssetDiagnosticCode::ImportProfileRejected
            })
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn multiple_material_sources_without_primary_source_block_before_ledger_allocation() {
        let root = unique_temp_dir("material_source_ambiguous");
        let asset_id = asset_id(1);
        let first_source_id = asset_source_id(2);
        let second_source_id = asset_source_id(3);
        let mut session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
            &root,
            &material_project_with_recipe("render", "render_material"),
        )
        .expect("project session should form");
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            "rock",
            "Rock",
            AssetKind::MaterialGraph,
        ));
        catalog.insert_source(
            AssetSourceDescriptor::new(
                first_source_id,
                asset_id,
                AssetKind::MaterialGraph,
                "assets/materials/rock-a.material.ron",
            )
            .with_hash(SourceHash::new("sha256", "abc")),
        );
        catalog.insert_source(
            AssetSourceDescriptor::new(
                second_source_id,
                asset_id,
                AssetKind::MaterialGraph,
                "assets/materials/rock-b.material.ron",
            )
            .with_hash(SourceHash::new("sha256", "def")),
        );

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(outcome.publication.is_none());
        assert_eq!(session.import_ledger().len(), 0);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("multiple material graph sources and no primary source")
        }));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn missing_texture_resource_artifact_blocks_before_ledger_allocation() {
        let root = unique_temp_dir("material_texture_resource_missing");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut session = crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
            &root,
            &material_project_with_recipe("render", "render_material"),
        )
        .expect("project session should form");
        let document = texture_material_graph_document(asset_id, &source);
        write_material_graph_document(&root.join(&source.relative_path), &document)
            .expect("source document should write");
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(outcome.publication.is_none());
        assert_eq!(session.import_ledger().len(), 0);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("references missing texture asset 'rock.albedo'")
        }));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn changed_material_recipe_allocates_new_ledger_entry() {
        let root = unique_temp_dir("material_recipe_cache_split");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/rock.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source.clone());

        let mut preview_session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
                &root,
                &material_project_with_recipe("preview", "preview"),
            )
            .expect("preview project session should form");
        let preview_document = default_material_graph_document_for_source_with_target(
            asset_id,
            &source,
            "Rock",
            MaterialOutputTarget::PbrPreview,
        );
        write_material_graph_document(&root.join(&source.relative_path), &preview_document)
            .expect("preview source document should write");
        let preview = rebuild_material_preview_for_asset(&catalog, &mut preview_session, asset_id);
        assert!(preview.diagnostics.is_empty(), "{:?}", preview.diagnostics);
        assert_eq!(preview_session.import_ledger().len(), 3);

        let mut render_session =
            crate::asset_pipeline::EditorAssetProjectSession::from_project_file(
                &root,
                &material_project_with_recipe("render", "render_material"),
            )
            .expect("render project session should form");
        let render_document = default_material_graph_document_for_source_with_target(
            asset_id,
            &source,
            "Rock",
            MaterialOutputTarget::RenderMaterial,
        );
        write_material_graph_document(&root.join(&source.relative_path), &render_document)
            .expect("render source document should write");
        let render = rebuild_material_preview_for_asset(&catalog, &mut render_session, asset_id);

        assert!(render.diagnostics.is_empty(), "{:?}", render.diagnostics);
        assert_eq!(render_session.import_ledger().len(), 6);
        let cache_keys = render_session
            .import_ledger()
            .entries()
            .iter()
            .map(|entry| entry.cache_key.clone())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(cache_keys.len(), 6);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_material_graph_preserves_prior_valid_artifact() {
        let root = unique_temp_dir("material_preview_preserve");
        let asset_id = asset_id(1);
        let source_id = asset_source_id(2);
        let mut session = crate::asset_pipeline::EditorAssetProjectSession::new(
            &root,
            AssetProjectCatalogDescriptor::new(
                [AssetSourceRoot::new(
                    asset_source_root_id(1),
                    AssetSourceRootKind::ProjectAssets,
                    "Project assets",
                    "assets",
                )],
                ".runenwerk/artifacts",
                ".runenwerk/field-products",
                "assets/catalog.ron",
            ),
        );
        let source = AssetSourceDescriptor::new(
            source_id,
            asset_id,
            AssetKind::MaterialGraph,
            "assets/materials/missing.material.ron",
        )
        .with_hash(SourceHash::new("sha256", "abc"));
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(
            AssetRecord::new(asset_id, "rock", "Rock", AssetKind::MaterialGraph)
                .with_primary_source(source_id),
        );
        catalog.insert_source(source);
        catalog.insert_artifact(
            AssetArtifactDescriptor::new(
                asset_artifact_id(9),
                asset_id,
                AssetKind::Material,
                ArtifactPayloadKind::FormedMaterialProduct {
                    product_id: "3".to_string(),
                },
                ArtifactCacheKey::new("prior"),
            )
            .with_validity(asset::ArtifactValidity::Valid),
        );

        let outcome = rebuild_material_preview_for_asset(&catalog, &mut session, asset_id);

        assert!(!outcome.diagnostics.is_empty());
        let publication = outcome.publication.expect("prior valid should preserve");
        assert_eq!(
            publication.artifact.validity,
            asset::ArtifactValidity::FailedPreserved
        );
        assert_eq!(publication.artifact.artifact_id, asset_artifact_id(9));
        let _ = std::fs::remove_dir_all(root);
    }

    fn material_project_with_recipe(profile_name: &str, lowering_target: &str) -> ProjectFileV3 {
        let mut project = ProjectFileV3::new("project.material", "Material");
        project
            .import_profile_definitions
            .push(ProjectImportProfileDefinitionV3::new(
                AssetKind::MaterialGraph,
                profile_name,
                ImportSettings::MaterialGraph {
                    lowering_target: lowering_target.to_string(),
                },
                AssetKind::Material,
            ));
        project
            .import_profile_defaults
            .push(ProjectImportProfileDefaultV3::new(
                AssetKind::MaterialGraph,
                profile_name,
            ));
        project
    }

    fn texture_material_graph_document(
        asset_id: AssetId,
        source: &AssetSourceDescriptor,
    ) -> MaterialGraphDocument {
        use graph::{
            CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition,
            NodeId, PortDefinition, PortDirection, PortId, PortTypeId,
        };
        use resource_ref::ResourceRef;

        MaterialGraphDocument::new(
            material_document_id_for_source(asset_id, source.source_id),
            "Rock",
            GraphDefinition::new(
                GraphId::new(1),
                "material.texture",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(1),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(1),
                            "base_color",
                            PortDirection::Input,
                            PortTypeId::new(1),
                        )],
                    ),
                    NodeDefinition::new(NodeId::new(2), "texture.sample_2d", []).with_values([
                        GraphMetadataEntry::new(
                            material_graph::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
                            GraphValue::resource(
                                ResourceRef::new("asset.catalog.texture2d", "rock.albedo")
                                    .expect("resource ref"),
                            ),
                        ),
                    ]),
                ],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        )
    }

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        root.push(format!("{label}_{nanos}"));
        std::fs::create_dir_all(&root).expect("temp dir should be creatable");
        root
    }
}
