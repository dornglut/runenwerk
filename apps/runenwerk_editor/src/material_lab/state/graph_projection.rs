use super::picker_projection::{
    material_model_mesh_region_binding_view_model, material_node_palette_view_model,
    material_node_picker_view_model, material_scene_material_slot_option_view_model,
    material_texture_resource_picker_view_model,
};
use super::*;

impl MaterialLabRuntime {
    pub fn graph_canvas_view_model(
        &self,
        catalog: &AssetCatalog,
        catalog_status_lines: Vec<String>,
    ) -> MaterialGraphCanvasViewModel {
        self.graph_canvas_view_model_with_scene_material_assignments(
            catalog,
            catalog_status_lines,
            None,
        )
    }

    pub fn graph_canvas_view_model_with_scene_material_assignments(
        &self,
        catalog: &AssetCatalog,
        catalog_status_lines: Vec<String>,
        scene_material_assignments: Option<&SceneMaterialAssignmentState>,
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
        let mut validation_overlays =
            material_graph_validation_overlays(&self.diagnostics, self.active_diagnostic_index);
        if let Some((_, document)) = self.active_source_document() {
            validation_overlays.extend(material_graph_projection_overlays(document));
        }
        let palette = material_node_palette_view_model(&self.node_palette_search_query);
        let node_picker = material_node_picker_view_model(
            self.node_picker_open,
            &self.node_picker_search_query,
            self.node_picker_highlighted_descriptor_key.as_deref(),
        );
        MaterialGraphCanvasViewModel {
            rows,
            selected,
            graph: material_graph_editor_view_model(self, &validation_overlays),
            palette,
            texture_picker: material_texture_resource_picker_view_model(
                catalog,
                &self.texture_resource_search_query,
            ),
            sdf_primitives: Vec::new(),
            model_mesh_regions: material_model_mesh_region_binding_view_model(
                catalog,
                scene_material_assignments,
            ),
            scene_material_slots: material_scene_material_slot_option_view_model(
                scene_material_assignments,
            ),
            toolbar: material_graph_toolbar_view_model(
                self.active_source_document().map(|(_, document)| document),
                self.active_preview.as_ref(),
            ),
            validation_overlays,
            active_diagnostic_index: self.active_diagnostic_index,
            node_picker,
            shortcuts: material_graph_shortcut_view_model(),
            undo_redo: MaterialUndoRedoViewModel {
                can_undo: self.can_undo(),
                can_redo: self.can_redo(),
                active_group_id: None,
            },
            catalog_status_lines,
            diagnostic_rows: self.material_diagnostic_rows(),
            resource_binding_diagnostics: self.material_resource_binding_diagnostic_rows(catalog),
            diagnostic_lines: self.diagnostic_lines(),
        }
    }

    pub fn inspector_view_model(&self, catalog: &AssetCatalog) -> MaterialInspectorViewModel {
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
            diagnostic_rows: self.material_diagnostic_rows(),
            resource_binding_diagnostics: self.material_resource_binding_diagnostic_rows(catalog),
            diagnostic_lines: self.diagnostic_lines(),
        }
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

fn material_graph_editor_view_model(
    runtime: &MaterialLabRuntime,
    overlays: &[MaterialGraphValidationOverlayViewModel],
) -> MaterialGraphEditorViewModel {
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
        .collect::<Vec<_>>();
    let edges = document
        .graph
        .edges
        .iter()
        .map(|edge| MaterialGraphEdgeViewModel {
            edge_id: edge.id,
            from_port_id: edge.from_port,
            to_port_id: edge.to_port,
        })
        .collect::<Vec<_>>();
    let selected_edges = runtime
        .selected_graph_edges()
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let graph_canvas = material_graph_canvas_projection(
        document,
        &nodes,
        &edges,
        runtime.selected_graph_edges(),
        overlays,
    );

    MaterialGraphEditorViewModel {
        document_id: Some(document.document_id),
        output_target: Some(document.output_target),
        graph_editor: ui_graph_editor::GraphEditorViewModel {
            can_undo: runtime.can_undo(),
            can_redo: runtime.can_redo(),
            viewport: graph_canvas.viewport,
            selection: graph_canvas.selection.clone(),
            hit_test_scene: graph_canvas.hit_test_scene.clone(),
            canvas: graph_canvas,
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
        selected_edge_ids: selected_edges,
    }
}

fn material_graph_canvas_projection(
    document: &material_graph::MaterialGraphDocument,
    nodes: &[MaterialGraphNodeViewModel],
    edges: &[MaterialGraphEdgeViewModel],
    selected_edges: &BTreeSet<graph::EdgeId>,
    overlays: &[MaterialGraphValidationOverlayViewModel],
) -> ui_graph_editor::GraphCanvasViewModel {
    const NODE_WIDTH: i32 = 220;
    const NODE_HEADER_HEIGHT: i32 = 34;
    const PORT_SIZE: i32 = 12;
    const PORT_STEP: i32 = 24;
    const PORT_TOP: i32 = 46;

    let viewport = ui_graph_editor::GraphViewport {
        pan_x: document.editor_state.viewport.pan_x,
        pan_y: document.editor_state.viewport.pan_y,
        zoom_milli: document.editor_state.viewport.zoom_milli,
    };
    let mut port_centers = BTreeMap::new();
    let mut graph_nodes = Vec::new();
    let mut graph_ports = Vec::new();
    let mut node_bounds = Vec::new();
    let mut port_bounds = Vec::new();
    let mut selection_bounds = Vec::new();

    for node in nodes {
        let rows = node
            .input_ports
            .len()
            .max(node.output_ports.len())
            .max(
                node.editable_values
                    .len()
                    .saturating_add(node.resource_bindings.len()),
            )
            .max(1);
        let node_height = NODE_HEADER_HEIGHT + PORT_TOP + rows as i32 * PORT_STEP;
        let node_rect = ui_graph_editor::GraphRect::new(
            node.position_x,
            node.position_y,
            NODE_WIDTH,
            node_height,
        );
        let node_key = ui_graph_editor::GraphNodeKey(node.node_id.raw());
        graph_nodes.push(
            ui_graph_editor::GraphNodeView::new(node_key, node.label.clone(), node_rect)
                .selected(node.selected),
        );
        node_bounds.push(ui_graph_editor::GraphNodeBounds {
            node: node_key,
            rect: node_rect,
        });
        if node.selected {
            selection_bounds.push(ui_graph_editor::GraphSelectionBounds {
                selection: ui_graph_editor::GraphSelectionKey(node.node_id.raw()),
                rect: node_rect,
            });
        }

        for (index, port) in node.input_ports.iter().enumerate() {
            let rect = ui_graph_editor::GraphRect::new(
                node.position_x,
                node.position_y + PORT_TOP + index as i32 * PORT_STEP,
                PORT_SIZE,
                PORT_SIZE,
            );
            push_graph_port(
                &mut graph_ports,
                &mut port_bounds,
                &mut port_centers,
                port,
                node_key,
                ui_graph_editor::GraphPortDirection::Input,
                rect,
            );
        }
        for (index, port) in node.output_ports.iter().enumerate() {
            let rect = ui_graph_editor::GraphRect::new(
                node.position_x + NODE_WIDTH - PORT_SIZE,
                node.position_y + PORT_TOP + index as i32 * PORT_STEP,
                PORT_SIZE,
                PORT_SIZE,
            );
            push_graph_port(
                &mut graph_ports,
                &mut port_bounds,
                &mut port_centers,
                port,
                node_key,
                ui_graph_editor::GraphPortDirection::Output,
                rect,
            );
        }
    }

    let mut graph_edges = Vec::new();
    let mut edge_bounds = Vec::new();
    for edge in edges {
        let Some(from) = port_centers.get(&edge.from_port_id).copied() else {
            continue;
        };
        let Some(to) = port_centers.get(&edge.to_port_id).copied() else {
            continue;
        };
        let hit_rect = edge_hit_rect(from, to);
        let edge_key = ui_graph_editor::GraphEdgeKey(edge.edge_id.raw());
        let mut edge_view = ui_graph_editor::GraphEdgeView::new(
            edge_key,
            ui_graph_editor::GraphPortKey(edge.from_port_id.raw()),
            ui_graph_editor::GraphPortKey(edge.to_port_id.raw()),
            from,
            to,
            hit_rect,
        );
        edge_view.selected = selected_edges.contains(&edge.edge_id);
        graph_edges.push(edge_view);
        edge_bounds.push(ui_graph_editor::GraphEdgeBounds {
            edge: edge_key,
            rect: hit_rect,
        });
        if selected_edges.contains(&edge.edge_id) {
            selection_bounds.push(ui_graph_editor::GraphSelectionBounds {
                selection: ui_graph_editor::GraphSelectionKey(edge.edge_id.raw()),
                rect: hit_rect,
            });
        }
    }

    let overlays = overlays
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, overlay)| {
            let anchor = overlay
                .subject_port_id
                .map(|port| {
                    ui_graph_editor::GraphOverlayAnchor::Port(ui_graph_editor::GraphPortKey(
                        port.raw(),
                    ))
                })
                .or_else(|| {
                    overlay.subject_node_id.map(|node| {
                        ui_graph_editor::GraphOverlayAnchor::Node(ui_graph_editor::GraphNodeKey(
                            node.raw(),
                        ))
                    })
                })
                .unwrap_or_else(|| {
                    ui_graph_editor::GraphOverlayAnchor::Point(ui_graph_editor::GraphPoint::new(
                        16,
                        16 + index as i32 * 24,
                    ))
                });
            ui_graph_editor::GraphOverlayView::new(
                anchor,
                overlay.message,
                ui_graph_editor::GraphRect::new(16, 16 + index as i32 * 24, 220, 20),
                match overlay.severity {
                    MaterialGraphValidationSeverity::Info => {
                        ui_graph_editor::GraphOverlaySeverity::Info
                    }
                    MaterialGraphValidationSeverity::Warning => {
                        ui_graph_editor::GraphOverlaySeverity::Warning
                    }
                    MaterialGraphValidationSeverity::Blocking => {
                        ui_graph_editor::GraphOverlaySeverity::Error
                    }
                },
            )
            .active(overlay.active)
        })
        .collect::<Vec<_>>();

    let selection = ui_graph_editor::GraphSelection {
        nodes: nodes
            .iter()
            .filter(|node| node.selected)
            .map(|node| ui_graph_editor::GraphNodeKey(node.node_id.raw()))
            .collect(),
        edges: selected_edges
            .iter()
            .map(|edge| ui_graph_editor::GraphEdgeKey(edge.raw()))
            .collect(),
    };
    let canvas_rect = canvas_bounds(&node_bounds, &edge_bounds);
    let hit_test_scene = ui_graph_editor::GraphHitTestScene {
        canvas_rect,
        nodes: node_bounds,
        ports: port_bounds,
        edges: edge_bounds,
        selections: selection_bounds,
    };

    ui_graph_editor::GraphCanvasViewModel {
        canvas_id: ui_graph_editor::GraphCanvasId(document.document_id.raw()),
        viewport,
        nodes: graph_nodes,
        ports: graph_ports,
        edges: graph_edges,
        overlays,
        selection,
        hit_test_scene,
    }
}

fn push_graph_port(
    graph_ports: &mut Vec<ui_graph_editor::GraphPortView>,
    port_bounds: &mut Vec<ui_graph_editor::GraphPortBounds>,
    port_centers: &mut BTreeMap<graph::PortId, ui_graph_editor::GraphPoint>,
    port: &MaterialGraphPortViewModel,
    node: ui_graph_editor::GraphNodeKey,
    direction: ui_graph_editor::GraphPortDirection,
    rect: ui_graph_editor::GraphRect,
) {
    let port_key = ui_graph_editor::GraphPortKey(port.port_id.raw());
    graph_ports.push(ui_graph_editor::GraphPortView::new(
        port_key,
        node,
        port.name.clone(),
        direction,
        rect,
    ));
    port_bounds.push(ui_graph_editor::GraphPortBounds {
        port: port_key,
        node,
        rect,
    });
    port_centers.insert(
        port.port_id,
        ui_graph_editor::GraphPoint::new(rect.x + rect.width / 2, rect.y + rect.height / 2),
    );
}

fn edge_hit_rect(
    from: ui_graph_editor::GraphPoint,
    to: ui_graph_editor::GraphPoint,
) -> ui_graph_editor::GraphRect {
    let min_x = from.x.min(to.x) - 8;
    let min_y = from.y.min(to.y) - 8;
    let max_x = from.x.max(to.x) + 8;
    let max_y = from.y.max(to.y) + 8;
    ui_graph_editor::GraphRect::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn canvas_bounds(
    nodes: &[ui_graph_editor::GraphNodeBounds],
    edges: &[ui_graph_editor::GraphEdgeBounds],
) -> ui_graph_editor::GraphRect {
    let mut min_x = 0;
    let mut min_y = 0;
    let mut max_x = 1600;
    let mut max_y = 1200;
    for rect in nodes
        .iter()
        .map(|node| node.rect)
        .chain(edges.iter().map(|edge| edge.rect))
    {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.width);
        max_y = max_y.max(rect.y + rect.height);
    }
    ui_graph_editor::GraphRect::new(
        min_x - 256,
        min_y - 256,
        max_x - min_x + 512,
        max_y - min_y + 512,
    )
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
    active_diagnostic_index: Option<usize>,
) -> Vec<MaterialGraphValidationOverlayViewModel> {
    diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| {
            let (subject_node_id, subject_port_id) =
                material_graph_subject_from_diagnostic(diagnostic.subject.as_deref());
            MaterialGraphValidationOverlayViewModel {
                diagnostic_index: Some(index),
                subject_node_id,
                subject_port_id,
                severity: match diagnostic.severity {
                    asset::AssetDiagnosticSeverity::Info => MaterialGraphValidationSeverity::Info,
                    asset::AssetDiagnosticSeverity::Warning => {
                        MaterialGraphValidationSeverity::Warning
                    }
                    asset::AssetDiagnosticSeverity::Error
                    | asset::AssetDiagnosticSeverity::Fatal => {
                        MaterialGraphValidationSeverity::Blocking
                    }
                },
                message: diagnostic.message.clone(),
                active: active_diagnostic_index == Some(index),
            }
        })
        .collect()
}

fn material_graph_subject_from_diagnostic(
    subject: Option<&str>,
) -> (Option<graph::NodeId>, Option<graph::PortId>) {
    let Some(subject) = subject else {
        return (None, None);
    };
    if let Some(raw) = subject.strip_prefix("material_graph.node:")
        && let Ok(node_id) = raw.parse::<u64>()
    {
        return (Some(graph::NodeId::new(node_id)), None);
    }
    if let Some(raw) = subject.strip_prefix("material_graph.port:")
        && let Ok(port_id) = raw.parse::<u64>()
    {
        return (None, Some(graph::PortId::new(port_id)));
    }
    (None, None)
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
                        diagnostic_index: None,
                        subject_node_id: Some(node.id),
                        subject_port_id: Some(port.id),
                        severity: MaterialGraphValidationSeverity::Blocking,
                        message: format!(
                            "material graph projection does not recognize port type {} on node '{}' port '{}'",
                            port.port_type.raw(),
                            node.name,
                            port.name
                        ),
                        active: false,
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
