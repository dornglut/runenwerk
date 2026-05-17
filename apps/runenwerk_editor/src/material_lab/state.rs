use std::collections::BTreeSet;

use asset::{
    ArtifactCacheKey, ArtifactPayloadKind, ArtifactValidity, AssetArtifactId, AssetCatalog,
    AssetDiagnosticRecord, AssetId, AssetKind, AssetSourceId,
};
use editor_core::EntityId;
use editor_scene::{SceneMaterialPalette, SceneMaterialSlotId};
use editor_shell::{
    MaterialGraphCanvasViewModel, MaterialGraphEdgeViewModel, MaterialGraphEditorViewModel,
    MaterialGraphGroupViewModel, MaterialGraphNodeViewModel, MaterialGraphPortViewModel,
    MaterialGraphPropertyViewModel, MaterialGraphResourceBindingViewModel,
    MaterialGraphShortcutViewModel, MaterialGraphSourceDetailViewModel,
    MaterialGraphSourceRowViewModel, MaterialGraphToolbarViewModel,
    MaterialGraphValidationOverlayViewModel, MaterialGraphValidationSeverity,
    MaterialInspectorViewModel, MaterialNodePaletteCategoryViewModel,
    MaterialNodePaletteItemViewModel, MaterialNodePaletteViewModel, MaterialPreviewViewModel,
    MaterialShortcutAction, MaterialUndoRedoViewModel,
};
use editor_viewport::ExpressionProductId;
use material_graph::{FormedMaterialProduct, MaterialProductId};
use product::ProductPublicationStatus;
use std::collections::BTreeMap;

use crate::material_lab::{
    MaterialRendererParameterProfile, ResolvedMaterialResource, material_document_id_for_source,
    material_parameter_payload, material_preview_expression_product_id,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewProduct {
    pub asset_id: AssetId,
    pub source_id: AssetSourceId,
    pub artifact_id: AssetArtifactId,
    pub artifact_cache_key: ArtifactCacheKey,
    pub shader_artifact_id: AssetArtifactId,
    pub shader_cache_key: ArtifactCacheKey,
    pub shader_path: String,
    pub shader_identity: String,
    pub scene_shader_artifact_id: AssetArtifactId,
    pub scene_shader_cache_key: ArtifactCacheKey,
    pub scene_shader_path: String,
    pub scene_shader_identity: String,
    pub product: FormedMaterialProduct,
    pub renderer_parameter_profile: MaterialRendererParameterProfile,
    pub viewport_product_id: ExpressionProductId,
    pub resolved_resources: Vec<ResolvedMaterialResource>,
}

impl EditorMaterialPreviewProduct {
    pub fn new(
        asset_id: AssetId,
        source_id: AssetSourceId,
        artifact_id: AssetArtifactId,
        artifact_cache_key: ArtifactCacheKey,
        product: FormedMaterialProduct,
        renderer_parameter_profile: MaterialRendererParameterProfile,
        shader_artifact_id: AssetArtifactId,
        shader_cache_key: ArtifactCacheKey,
        shader_path: impl Into<String>,
        shader_identity: impl Into<String>,
        scene_shader_artifact_id: AssetArtifactId,
        scene_shader_cache_key: ArtifactCacheKey,
        scene_shader_path: impl Into<String>,
        scene_shader_identity: impl Into<String>,
        resolved_resources: impl IntoIterator<Item = ResolvedMaterialResource>,
    ) -> Self {
        let viewport_product_id = material_preview_expression_product_id(product.product_id);
        Self {
            asset_id,
            source_id,
            artifact_id,
            artifact_cache_key,
            shader_artifact_id,
            shader_cache_key,
            shader_path: shader_path.into(),
            shader_identity: shader_identity.into(),
            scene_shader_artifact_id,
            scene_shader_cache_key,
            scene_shader_path: scene_shader_path.into(),
            scene_shader_identity: scene_shader_identity.into(),
            product,
            renderer_parameter_profile,
            viewport_product_id,
            resolved_resources: resolved_resources.into_iter().collect(),
        }
    }

    pub fn product_id(&self) -> MaterialProductId {
        self.product.product_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorMaterialPreviewPublicationJournalEntry {
    pub artifact_id: AssetArtifactId,
    pub product_id: Option<MaterialProductId>,
    pub status: ProductPublicationStatus,
}

#[derive(Debug, Clone, Default)]
pub struct MaterialLabRuntime {
    selected_material_asset_id: Option<AssetId>,
    active_preview: Option<EditorMaterialPreviewProduct>,
    active_source_document: Option<(AssetId, material_graph::MaterialGraphDocument)>,
    selected_graph_nodes: BTreeSet<graph::NodeId>,
    undo_stack: Vec<(AssetId, material_graph::MaterialGraphDocument)>,
    redo_stack: Vec<(AssetId, material_graph::MaterialGraphDocument)>,
    diagnostics: Vec<AssetDiagnosticRecord>,
    publication_journal: Vec<EditorMaterialPreviewPublicationJournalEntry>,
    last_workflow_status: Option<String>,
    scene_material_palette: SceneMaterialPalette,
    primitive_material_slots: BTreeMap<EntityId, SceneMaterialSlotId>,
}

impl MaterialLabRuntime {
    pub fn select_material_asset(&mut self, asset_id: Option<AssetId>) {
        self.selected_material_asset_id = asset_id;
    }

    pub fn selected_material_asset_id(&self) -> Option<AssetId> {
        self.selected_material_asset_id
    }

    pub fn active_preview(&self) -> Option<&EditorMaterialPreviewProduct> {
        self.active_preview.as_ref()
    }

    pub fn select_graph_node(&mut self, node_id: graph::NodeId) {
        self.selected_graph_nodes.clear();
        self.selected_graph_nodes.insert(node_id);
    }

    pub fn selected_graph_nodes(&self) -> &BTreeSet<graph::NodeId> {
        &self.selected_graph_nodes
    }

    pub fn push_undo_snapshot(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        self.undo_stack.push((asset_id, document));
        self.redo_stack.clear();
    }

    pub fn pop_undo_snapshot(
        &mut self,
    ) -> Option<(AssetId, material_graph::MaterialGraphDocument)> {
        self.undo_stack.pop()
    }

    pub fn push_redo_snapshot(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        self.redo_stack.push((asset_id, document));
    }

    pub fn pop_redo_snapshot(
        &mut self,
    ) -> Option<(AssetId, material_graph::MaterialGraphDocument)> {
        self.redo_stack.pop()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn set_active_preview(&mut self, preview: EditorMaterialPreviewProduct) {
        self.selected_material_asset_id = Some(preview.asset_id);
        self.active_preview = Some(preview);
    }

    pub fn set_active_source_document(
        &mut self,
        asset_id: AssetId,
        document: material_graph::MaterialGraphDocument,
    ) {
        self.selected_material_asset_id = Some(asset_id);
        self.active_source_document = Some((asset_id, document));
    }

    pub fn active_source_document(
        &self,
    ) -> Option<(AssetId, &material_graph::MaterialGraphDocument)> {
        self.active_source_document
            .as_ref()
            .map(|(asset_id, document)| (*asset_id, document))
    }

    pub fn record_diagnostic(&mut self, diagnostic: AssetDiagnosticRecord) {
        self.diagnostics.push(diagnostic);
    }

    pub fn record_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = AssetDiagnosticRecord>,
    ) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn diagnostics(&self) -> &[AssetDiagnosticRecord] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.diagnostics.clear();
    }

    pub fn set_workflow_status(&mut self, status: impl Into<String>) {
        self.last_workflow_status = Some(status.into());
    }

    pub fn publication_journal(&self) -> &[EditorMaterialPreviewPublicationJournalEntry] {
        &self.publication_journal
    }

    pub fn record_publication(&mut self, entry: EditorMaterialPreviewPublicationJournalEntry) {
        self.publication_journal.push(entry);
    }

    pub fn scene_material_palette(&self) -> &SceneMaterialPalette {
        &self.scene_material_palette
    }

    pub fn assign_primitive_material_slot(
        &mut self,
        entity_id: EntityId,
        slot_id: SceneMaterialSlotId,
    ) -> Result<(), String> {
        if !self.scene_material_palette.contains_slot(slot_id) {
            return Err(format!(
                "scene material assignment references unknown slot {}",
                slot_id.raw()
            ));
        }
        self.primitive_material_slots.insert(entity_id, slot_id);
        Ok(())
    }

    pub fn material_slot_index_for_entity(&self, entity_id: EntityId) -> u32 {
        let slot_id = self
            .primitive_material_slots
            .get(&entity_id)
            .copied()
            .unwrap_or_else(|| self.scene_material_palette.default_slot().slot_id);
        self.scene_material_palette
            .slots
            .iter()
            .position(|slot| slot.slot_id == slot_id)
            .unwrap_or_else(|| {
                self.scene_material_palette
                    .slots
                    .iter()
                    .position(|slot| slot.is_default)
                    .unwrap_or(0)
            }) as u32
    }

    pub fn graph_canvas_view_model(
        &self,
        catalog: &AssetCatalog,
        catalog_status_lines: Vec<String>,
    ) -> MaterialGraphCanvasViewModel {
        let rows = catalog
            .assets()
            .filter(|record| record.kind == AssetKind::MaterialGraph)
            .map(|record| {
                let has_prior_valid_preservation = record
                    .artifact_ids
                    .iter()
                    .filter_map(|artifact_id| catalog.artifact(*artifact_id))
                    .any(|artifact| artifact.validity.preserves_prior_valid());
                MaterialGraphSourceRowViewModel {
                    asset_id: record.asset_id,
                    display_name: record.display_name.clone(),
                    stable_name: record.stable_name.clone(),
                    source_id: record.primary_source_id,
                    artifact_count: record.artifact_ids.len(),
                    is_selected: Some(record.asset_id) == self.selected_material_asset_id,
                    has_prior_valid_preservation,
                }
            })
            .collect::<Vec<_>>();
        let selected = self.selected_material_asset_id.and_then(|asset_id| {
            selected_material_detail(
                catalog,
                asset_id,
                self.active_preview.as_ref(),
                self.active_source_document().map(|(_, document)| document),
            )
        });
        let mut validation_overlays = material_graph_validation_overlays(&self.diagnostics);
        if let Some((_, document)) = self.active_source_document() {
            validation_overlays.extend(material_graph_projection_overlays(document));
        }
        MaterialGraphCanvasViewModel {
            rows,
            selected,
            graph: material_graph_editor_view_model(self),
            palette: material_node_palette_view_model(),
            toolbar: material_graph_toolbar_view_model(
                self.active_source_document().map(|(_, document)| document),
                self.active_preview.as_ref(),
            ),
            validation_overlays,
            shortcuts: material_graph_shortcut_view_model(),
            undo_redo: MaterialUndoRedoViewModel {
                can_undo: self.can_undo(),
                can_redo: self.can_redo(),
                active_group_id: None,
            },
            catalog_status_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn inspector_view_model(&self) -> MaterialInspectorViewModel {
        let parameter_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No formed material product".to_string()],
            |preview| {
                preview
                    .product
                    .parameters
                    .iter()
                    .map(|parameter| format!("{}: {:?}", parameter.key, parameter.kind))
                    .collect()
            },
        );
        let source_map_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No material source map".to_string()],
            |preview| {
                preview
                    .product
                    .source_map
                    .entries
                    .iter()
                    .map(|entry| format!("node {} role={}", entry.node_id.raw(), entry.role))
                    .collect()
            },
        );
        MaterialInspectorViewModel {
            selected_asset_id: self.selected_material_asset_id,
            active_product_id: self
                .active_preview
                .as_ref()
                .map(EditorMaterialPreviewProduct::product_id),
            artifact_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.artifact_id),
            output_target: self
                .active_preview
                .as_ref()
                .map(|preview| preview.product.output_target),
            parameter_lines,
            source_map_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn preview_view_model(&self) -> MaterialPreviewViewModel {
        let preview_status_lines = self.active_preview.as_ref().map_or_else(
            || vec!["No active material preview product".to_string()],
            |preview| {
                vec![
                    format!("material product: {}", preview.product.product_id.raw()),
                    format!("artifact: {}", preview.artifact_id.raw()),
                    format!("shader artifact: {}", preview.shader_artifact_id.raw()),
                    format!(
                        "scene shader artifact: {}",
                        preview.scene_shader_artifact_id.raw()
                    ),
                    format!("viewport product: {}", preview.viewport_product_id.0),
                    format!("cache: {}", preview.artifact_cache_key.as_str()),
                    format!("shader cache: {}", preview.shader_cache_key.as_str()),
                    format!(
                        "scene shader cache: {}",
                        preview.scene_shader_cache_key.as_str()
                    ),
                ]
            },
        );
        MaterialPreviewViewModel {
            selected_asset_id: self.selected_material_asset_id,
            active_product_id: self
                .active_preview
                .as_ref()
                .map(EditorMaterialPreviewProduct::product_id),
            artifact_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.artifact_id),
            viewport_product_id: self
                .active_preview
                .as_ref()
                .map(|preview| preview.viewport_product_id),
            specialization_fragment: self
                .active_preview
                .as_ref()
                .map(|preview| preview.product.specialization_fragment.0.clone()),
            prepared_parameter_payload_bytes: self
                .active_preview
                .as_ref()
                .map(|preview| material_parameter_payload(preview).encoded_len())
                .unwrap_or(0),
            preview_status_lines,
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = self
            .diagnostics
            .iter()
            .map(|diagnostic| {
                format!(
                    "{:?} {:?}: {}",
                    diagnostic.severity, diagnostic.code, diagnostic.message
                )
            })
            .collect::<Vec<_>>();
        if let Some(status) = &self.last_workflow_status {
            lines.push(format!("last material workflow: {status}"));
        }
        if lines.is_empty() {
            lines.push("No material diagnostics".to_string());
        }
        lines
    }
}

fn selected_material_detail(
    catalog: &AssetCatalog,
    asset_id: AssetId,
    active_preview: Option<&EditorMaterialPreviewProduct>,
    active_source_document: Option<&material_graph::MaterialGraphDocument>,
) -> Option<MaterialGraphSourceDetailViewModel> {
    let record = catalog.asset(asset_id)?;
    let source = record
        .primary_source_id
        .and_then(|source_id| catalog.source(source_id));
    let source_id = source.map(|source| source.source_id);
    Some(MaterialGraphSourceDetailViewModel {
        asset_id,
        source_id,
        source_path: source.map(|source| source.relative_path.clone()),
        document_id: source_id
            .map(|source_id| material_document_id_for_source(asset_id, source_id)),
        output_target: active_preview
            .filter(|preview| preview.asset_id == asset_id)
            .map(|preview| preview.product.output_target),
        node_count: active_source_document
            .map(|document| document.graph.nodes.len())
            .or_else(|| {
                active_preview
                    .filter(|preview| preview.asset_id == asset_id)
                    .map(|preview| preview.product.source_map.entries.len())
            })
            .unwrap_or(0),
        edge_count: active_source_document
            .map(|document| document.graph.edges.len())
            .unwrap_or(0),
    })
}

fn material_graph_editor_view_model(runtime: &MaterialLabRuntime) -> MaterialGraphEditorViewModel {
    let Some((asset_id, document)) = runtime.active_source_document() else {
        return MaterialGraphEditorViewModel::default();
    };
    if Some(asset_id) != runtime.selected_material_asset_id {
        return MaterialGraphEditorViewModel::default();
    }

    let catalog = material_graph::MaterialNodeCatalog::first_slice();
    let layout_by_node = document
        .editor_state
        .node_layouts
        .iter()
        .map(|layout| (layout.node_id, layout))
        .collect::<BTreeMap<_, _>>();
    let input_ports = document
        .graph
        .edges
        .iter()
        .map(|edge| edge.to_port)
        .collect::<BTreeSet<_>>();
    let output_ports = document
        .graph
        .edges
        .iter()
        .map(|edge| edge.from_port)
        .collect::<BTreeSet<_>>();

    let nodes = document
        .graph
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let descriptor = catalog.descriptor(&node.name);
            let input_ports = node
                .ports
                .iter()
                .filter(|port| port.direction == graph::PortDirection::Input)
                .filter_map(|port| {
                    material_graph::MaterialValueType::from_port_type_id(port.port_type).map(
                        |value_type| MaterialGraphPortViewModel {
                            port_id: port.id,
                            name: port.name.clone(),
                            value_type,
                            connected: input_ports.contains(&port.id),
                        },
                    )
                })
                .collect();
            let output_ports = node
                .ports
                .iter()
                .filter(|port| port.direction == graph::PortDirection::Output)
                .filter_map(|port| {
                    material_graph::MaterialValueType::from_port_type_id(port.port_type).map(
                        |value_type| MaterialGraphPortViewModel {
                            port_id: port.id,
                            name: port.name.clone(),
                            value_type,
                            connected: output_ports.contains(&port.id),
                        },
                    )
                })
                .collect();
            let editable_values = descriptor.map_or_else(Vec::new, |descriptor| {
                descriptor
                    .values
                    .iter()
                    .map(|value| MaterialGraphPropertyViewModel {
                        node_id: node.id,
                        key: value.key.clone(),
                        value_type: value.value_type,
                        display_value: node
                            .value(&value.key)
                            .map(graph::GraphValue::canonical_component)
                            .or_else(|| {
                                value
                                    .default_value
                                    .as_ref()
                                    .map(material_graph::MaterialLiteral::canonical_component)
                            })
                            .unwrap_or_default(),
                        required: value.default_value.is_none(),
                    })
                    .collect()
            });
            let resource_bindings = descriptor.map_or_else(Vec::new, |descriptor| {
                descriptor
                    .resources
                    .iter()
                    .map(|resource| MaterialGraphResourceBindingViewModel {
                        node_id: node.id,
                        key: resource.key.clone(),
                        resource_kind: resource.kind,
                        reference: node.value(&resource.key).and_then(|value| match value {
                            graph::GraphValue::Resource(reference) => {
                                Some(reference.canonical_component())
                            }
                            _ => None,
                        }),
                        resolved_artifact_id: runtime.active_preview.as_ref().and_then(|preview| {
                            preview
                                .resolved_resources
                                .iter()
                                .find(|resolved| {
                                    resolved.node_id == node.id
                                        && resolved.binding_key == resource.key
                                })
                                .map(|resolved| resolved.artifact_id)
                        }),
                    })
                    .collect()
            });
            let layout = layout_by_node.get(&node.id);
            MaterialGraphNodeViewModel {
                node_id: node.id,
                descriptor_key: node.name.clone(),
                label: descriptor
                    .map(|descriptor| descriptor.label.clone())
                    .unwrap_or_else(|| node.name.clone()),
                position_x: layout
                    .map(|layout| layout.position_x)
                    .unwrap_or((index as i32 % 4) * 220),
                position_y: layout
                    .map(|layout| layout.position_y)
                    .unwrap_or((index as i32 / 4) * 120),
                input_ports,
                output_ports,
                editable_values,
                resource_bindings,
                selected: runtime.selected_graph_nodes().contains(&node.id),
            }
        })
        .collect();
    let edges = document
        .graph
        .edges
        .iter()
        .map(|edge| MaterialGraphEdgeViewModel {
            edge_id: edge.id,
            from_port_id: edge.from_port,
            to_port_id: edge.to_port,
        })
        .collect();

    MaterialGraphEditorViewModel {
        document_id: Some(document.document_id),
        output_target: Some(document.output_target),
        graph_editor: ui_graph_editor::GraphEditorViewModel {
            can_undo: runtime.can_undo(),
            can_redo: runtime.can_redo(),
            selection: ui_graph_editor::GraphSelection {
                nodes: runtime
                    .selected_graph_nodes()
                    .iter()
                    .map(|node| ui_graph_editor::GraphNodeKey(node.raw()))
                    .collect(),
                edges: Default::default(),
            },
            ..ui_graph_editor::GraphEditorViewModel::default()
        },
        viewport: document.editor_state.viewport,
        nodes,
        edges,
        groups: document
            .editor_state
            .groups
            .iter()
            .map(|group| MaterialGraphGroupViewModel {
                group_id: group.group_id.clone(),
                label: group.label.clone(),
                collapsed: group.collapsed,
            })
            .collect(),
        selected_node_ids: runtime.selected_graph_nodes().iter().copied().collect(),
        selected_edge_ids: Vec::new(),
    }
}

fn material_node_palette_view_model() -> MaterialNodePaletteViewModel {
    let mut categories =
        std::collections::BTreeMap::<String, Vec<MaterialNodePaletteItemViewModel>>::new();
    for descriptor in material_graph::MaterialNodeCatalog::first_slice().descriptors() {
        let category = descriptor
            .key
            .split_once('.')
            .map(|(prefix, _)| match prefix {
                "pbr" => "PBR",
                "sdf" => "SDF Context",
                "proc" => "Procedural",
                "math" => "Math",
                "texture" => "Textures",
                "coord" => "Coordinates",
                _ => "Material",
            })
            .unwrap_or("Material")
            .to_string();
        categories
            .entry(category)
            .or_default()
            .push(MaterialNodePaletteItemViewModel {
                descriptor_key: descriptor.key.clone(),
                label: descriptor.label.clone(),
                output_targets: descriptor.output_targets.clone(),
            });
    }
    MaterialNodePaletteViewModel {
        search_query: String::new(),
        categories: categories
            .into_iter()
            .map(|(label, nodes)| MaterialNodePaletteCategoryViewModel { label, nodes })
            .collect(),
    }
}

fn material_graph_toolbar_view_model(
    source_document: Option<&material_graph::MaterialGraphDocument>,
    active_preview: Option<&EditorMaterialPreviewProduct>,
) -> MaterialGraphToolbarViewModel {
    let mut toolbar = MaterialGraphToolbarViewModel::default();
    if let Some(document) = source_document {
        toolbar.selected_fixture = document.editor_state.selected_fixture;
        toolbar.selected_preview = document.editor_state.selected_preview;
    }
    if let Some(preview) = active_preview
        && preview.product.output_target == material_graph::MaterialOutputTarget::RenderMaterial
    {
        toolbar.selected_preview = material_graph::MaterialGraphPreviewSelection::SceneProduct;
    }
    toolbar
}

fn material_graph_validation_overlays(
    diagnostics: &[AssetDiagnosticRecord],
) -> Vec<MaterialGraphValidationOverlayViewModel> {
    diagnostics
        .iter()
        .map(|diagnostic| MaterialGraphValidationOverlayViewModel {
            subject_node_id: None,
            subject_port_id: None,
            severity: match diagnostic.severity {
                asset::AssetDiagnosticSeverity::Info => MaterialGraphValidationSeverity::Info,
                asset::AssetDiagnosticSeverity::Warning => MaterialGraphValidationSeverity::Warning,
                asset::AssetDiagnosticSeverity::Error | asset::AssetDiagnosticSeverity::Fatal => {
                    MaterialGraphValidationSeverity::Blocking
                }
            },
            message: diagnostic.message.clone(),
        })
        .collect()
}

fn material_graph_projection_overlays(
    document: &material_graph::MaterialGraphDocument,
) -> Vec<MaterialGraphValidationOverlayViewModel> {
    document
        .graph
        .nodes
        .iter()
        .flat_map(|node| {
            node.ports.iter().filter_map(move |port| {
                if material_graph::MaterialValueType::from_port_type_id(port.port_type).is_some() {
                    None
                } else {
                    Some(MaterialGraphValidationOverlayViewModel {
                        subject_node_id: Some(node.id),
                        subject_port_id: Some(port.id),
                        severity: MaterialGraphValidationSeverity::Blocking,
                        message: format!(
                            "material graph projection does not recognize port type {} on node '{}' port '{}'",
                            port.port_type.raw(),
                            node.name,
                            port.name
                        ),
                    })
                }
            })
        })
        .collect()
}

fn material_graph_shortcut_view_model() -> Vec<MaterialGraphShortcutViewModel> {
    vec![
        MaterialGraphShortcutViewModel {
            chord: "A".to_string(),
            action: MaterialShortcutAction::AddNode,
        },
        MaterialGraphShortcutViewModel {
            chord: "Delete".to_string(),
            action: MaterialShortcutAction::DeleteSelection,
        },
        MaterialGraphShortcutViewModel {
            chord: "Ctrl+Z".to_string(),
            action: MaterialShortcutAction::Undo,
        },
        MaterialGraphShortcutViewModel {
            chord: "Ctrl+Y".to_string(),
            action: MaterialShortcutAction::Redo,
        },
        MaterialGraphShortcutViewModel {
            chord: "Ctrl+B".to_string(),
            action: MaterialShortcutAction::BuildPreview,
        },
        MaterialGraphShortcutViewModel {
            chord: "F".to_string(),
            action: MaterialShortcutAction::FocusPreview,
        },
    ]
}

pub fn material_artifact_lines(catalog: &AssetCatalog) -> Vec<String> {
    let mut lines = catalog
        .artifacts
        .values()
        .filter_map(|artifact| match &artifact.payload_kind {
            ArtifactPayloadKind::FormedMaterialProduct { product_id } => Some(format!(
                "formed material artifact {} product={} validity={:?} cache={}",
                artifact.artifact_id.raw(),
                product_id,
                artifact.validity,
                artifact.cache_key.as_str()
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        lines.push("No formed material artifacts".to_string());
    }
    lines
}

pub fn previous_valid_material_artifact<'a>(
    catalog: &'a AssetCatalog,
    asset_id: AssetId,
) -> Option<&'a asset::AssetArtifactDescriptor> {
    let record = catalog.asset(asset_id)?;
    record
        .artifact_ids
        .iter()
        .rev()
        .filter_map(|artifact_id| catalog.artifact(*artifact_id))
        .find(|artifact| {
            artifact.kind == AssetKind::Material && artifact.validity == ArtifactValidity::Valid
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{AssetRecord, asset_id};
    use graph::{
        CyclePolicy, EdgeDefinition, GraphDefinition, GraphId, NodeDefinition, NodeId,
        PortDefinition, PortDirection, PortId,
    };
    use material_graph::{
        MaterialGraphDocument, MaterialGraphEditorState, MaterialGraphNodeLayout,
        MaterialGraphViewportState, MaterialOutputTarget, MaterialValueType,
    };

    #[test]
    fn graph_canvas_projects_source_document_without_formed_preview() {
        let asset_id = asset_id(7);
        let color_port = MaterialValueType::Color.port_type_id();
        let mut editor_state = MaterialGraphEditorState::default();
        editor_state.viewport = MaterialGraphViewportState {
            pan_x: 12,
            pan_y: -6,
            zoom_milli: 1500,
        };
        editor_state
            .node_layouts
            .push(MaterialGraphNodeLayout::new(NodeId::new(3), 420, 90));
        let document = MaterialGraphDocument::new(
            material_graph::MaterialGraphDocumentId::new(70),
            "source-backed",
            GraphDefinition::new(
                GraphId::new(1),
                "source",
                CyclePolicy::RejectDirectedCycles,
                [
                    NodeDefinition::new(
                        NodeId::new(3),
                        "pbr.base_color",
                        [PortDefinition::new(
                            PortId::new(30),
                            "color",
                            PortDirection::Output,
                            color_port,
                        )],
                    ),
                    NodeDefinition::new(
                        NodeId::new(4),
                        "pbr.output",
                        [PortDefinition::new(
                            PortId::new(40),
                            "base_color",
                            PortDirection::Input,
                            color_port,
                        )],
                    ),
                ],
                [EdgeDefinition::new(
                    graph::EdgeId::new(9),
                    PortId::new(30),
                    PortId::new(40),
                )],
            ),
            MaterialOutputTarget::RenderMaterial,
        )
        .with_editor_state(editor_state);
        let mut runtime = MaterialLabRuntime::default();
        runtime.set_active_source_document(asset_id, document);
        let mut catalog = AssetCatalog::new();
        catalog.insert_asset_record(AssetRecord::new(
            asset_id,
            "mat.source",
            "Source Material",
            AssetKind::MaterialGraph,
        ));

        let view = runtime.graph_canvas_view_model(&catalog, Vec::new());

        assert_eq!(
            view.graph.document_id,
            Some(material_graph::MaterialGraphDocumentId::new(70))
        );
        assert_eq!(view.graph.viewport.zoom_milli, 1500);
        assert_eq!(view.graph.nodes.len(), 2);
        let color_node = view
            .graph
            .nodes
            .iter()
            .find(|node| node.node_id == NodeId::new(3))
            .expect("source node should project");
        assert_eq!(color_node.position_x, 420);
        assert_eq!(color_node.output_ports[0].port_id, PortId::new(30));
        assert!(color_node.output_ports[0].connected);
        assert_eq!(view.graph.edges[0].from_port_id, PortId::new(30));
        assert_eq!(view.graph.edges[0].to_port_id, PortId::new(40));
    }
}
