//! File: domain/editor/editor_shell/src/ux_lab/graph_canvas.rs
//! Purpose: Editor UX Lab graph canvas product evidence contracts.

use crate::{
    MaterialGraphCanvasViewModel, MaterialGraphEdgeViewModel, MaterialGraphEditorViewModel,
    MaterialGraphNodeViewModel, MaterialGraphPortViewModel, MaterialGraphPropertyViewModel,
    MaterialGraphShortcutViewModel, MaterialGraphSourceDetailViewModel,
    MaterialGraphSourceRowViewModel, MaterialGraphToolbarViewModel,
    MaterialGraphValidationOverlayViewModel, MaterialGraphValidationSeverity,
    MaterialNodePaletteCategoryViewModel, MaterialNodePaletteItemViewModel,
    MaterialNodePaletteViewModel, MaterialNodePickerViewModel, MaterialShortcutAction,
    MaterialUndoRedoViewModel, VisibleWidgetState,
};

pub const MATERIAL_GRAPH_CANVAS_SCENARIO_ID: &str = "editor.graph.material.canvas.product";
pub const MATERIAL_GRAPH_CANVAS_TARGET_PROFILE: &str = "editor.graph.material";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorUxGraphCanvasEvidence {
    pub target_profile: &'static str,
    pub graph_family: &'static str,
    pub interaction_kinds: Vec<&'static str>,
    pub route_kinds: Vec<&'static str>,
    pub readiness_decisions: Vec<&'static str>,
    pub native_evidence_checks: Vec<&'static str>,
}

pub fn material_graph_canvas_evidence() -> EditorUxGraphCanvasEvidence {
    EditorUxGraphCanvasEvidence {
        target_profile: MATERIAL_GRAPH_CANVAS_TARGET_PROFILE,
        graph_family: "material",
        interaction_kinds: vec![
            "select_node",
            "select_edge",
            "drag_node_commit",
            "connect_ports_commit",
            "pan_commit",
            "zoom",
            "keyboard_delete",
            "keyboard_shortcuts",
            "diagnostic_navigation",
            "dense_graph_overflow",
            "degraded_provider_diagnostics",
        ],
        route_kinds: vec![
            "provider_owned_graph_canvas",
            "open_node_picker",
            "highlight_node_picker_node",
            "confirm_node_picker_selection",
            "set_node_value",
            "pick_texture_resource",
            "navigate_diagnostic",
            "build_selected_preview",
            "focus_preview",
            "undo_material_edit",
            "redo_material_edit",
        ],
        readiness_decisions: vec![
            "material_graph_canvas=product",
            "sdf_graph_canvas=hidden_until_productized",
            "procgen_graph_canvas=hidden_until_productized",
            "gameplay_graph_canvas=hidden_until_productized",
            "particle_graph_canvas=hidden_until_productized",
            "animation_graph_canvas=hidden_until_productized",
        ],
        native_evidence_checks: vec![
            "retained_graph_canvas",
            "native_or_platform_impossible_capture",
            "focus_traversal",
            "accessibility",
            "diagnostics_snapshot",
            "timing_report",
        ],
    }
}

pub fn material_graph_canvas_required_states() -> [VisibleWidgetState; 5] {
    [
        VisibleWidgetState::Default,
        VisibleWidgetState::Focused,
        VisibleWidgetState::Selected,
        VisibleWidgetState::Error,
        VisibleWidgetState::Overflow,
    ]
}

pub fn material_graph_canvas_fixture_view_model() -> MaterialGraphCanvasViewModel {
    let graph_canvas = ui_graph_editor::GraphCanvasViewModel {
        canvas_id: ui_graph_editor::GraphCanvasId(9_001),
        viewport: ui_graph_editor::GraphViewport {
            pan_x: -64,
            pan_y: 24,
            zoom_milli: 1_250,
        },
        nodes: vec![
            ui_graph_editor::GraphNodeView::new(
                ui_graph_editor::GraphNodeKey(11),
                "Base Color Texture",
                ui_graph_editor::GraphRect::new(40, 48, 220, 132),
            )
            .selected(true),
            ui_graph_editor::GraphNodeView::new(
                ui_graph_editor::GraphNodeKey(12),
                "PBR Output",
                ui_graph_editor::GraphRect::new(380, 64, 220, 148),
            ),
        ],
        ports: vec![
            ui_graph_editor::GraphPortView::new(
                ui_graph_editor::GraphPortKey(101),
                ui_graph_editor::GraphNodeKey(11),
                "color",
                ui_graph_editor::GraphPortDirection::Output,
                ui_graph_editor::GraphRect::new(248, 96, 14, 14),
            ),
            ui_graph_editor::GraphPortView::new(
                ui_graph_editor::GraphPortKey(201),
                ui_graph_editor::GraphNodeKey(12),
                "base_color",
                ui_graph_editor::GraphPortDirection::Input,
                ui_graph_editor::GraphRect::new(378, 104, 14, 14),
            ),
        ],
        edges: {
            let mut edge = ui_graph_editor::GraphEdgeView::new(
                ui_graph_editor::GraphEdgeKey(301),
                ui_graph_editor::GraphPortKey(101),
                ui_graph_editor::GraphPortKey(201),
                ui_graph_editor::GraphPoint::new(262, 103),
                ui_graph_editor::GraphPoint::new(378, 111),
                ui_graph_editor::GraphRect::new(260, 96, 120, 22),
            );
            edge.selected = true;
            vec![edge]
        },
        overlays: vec![
            ui_graph_editor::GraphOverlayView::new(
                ui_graph_editor::GraphOverlayAnchor::Node(ui_graph_editor::GraphNodeKey(12)),
                "preview build required",
                ui_graph_editor::GraphRect::new(382, 216, 180, 24),
                ui_graph_editor::GraphOverlaySeverity::Warning,
            )
            .active(true),
        ],
        selection: ui_graph_editor::GraphSelection {
            nodes: [ui_graph_editor::GraphNodeKey(11)].into_iter().collect(),
            edges: [ui_graph_editor::GraphEdgeKey(301)].into_iter().collect(),
        },
        hit_test_scene: ui_graph_editor::GraphHitTestScene {
            canvas_rect: ui_graph_editor::GraphRect::new(0, 0, 960, 640),
            nodes: vec![
                ui_graph_editor::GraphNodeBounds {
                    node: ui_graph_editor::GraphNodeKey(11),
                    rect: ui_graph_editor::GraphRect::new(40, 48, 220, 132),
                },
                ui_graph_editor::GraphNodeBounds {
                    node: ui_graph_editor::GraphNodeKey(12),
                    rect: ui_graph_editor::GraphRect::new(380, 64, 220, 148),
                },
            ],
            ports: vec![
                ui_graph_editor::GraphPortBounds {
                    port: ui_graph_editor::GraphPortKey(101),
                    node: ui_graph_editor::GraphNodeKey(11),
                    rect: ui_graph_editor::GraphRect::new(248, 96, 14, 14),
                },
                ui_graph_editor::GraphPortBounds {
                    port: ui_graph_editor::GraphPortKey(201),
                    node: ui_graph_editor::GraphNodeKey(12),
                    rect: ui_graph_editor::GraphRect::new(378, 104, 14, 14),
                },
            ],
            edges: vec![ui_graph_editor::GraphEdgeBounds {
                edge: ui_graph_editor::GraphEdgeKey(301),
                rect: ui_graph_editor::GraphRect::new(260, 96, 120, 22),
            }],
            selections: vec![ui_graph_editor::GraphSelectionBounds {
                selection: ui_graph_editor::GraphSelectionKey(1),
                rect: ui_graph_editor::GraphRect::new(34, 42, 232, 144),
            }],
        },
    };
    let graph_editor = MaterialGraphEditorViewModel {
        document_id: Some(material_graph::MaterialGraphDocumentId::new(9_001)),
        output_target: Some(material_graph::MaterialOutputTarget::RenderMaterial),
        graph_editor: ui_graph_editor::GraphEditorViewModel {
            viewport: graph_canvas.viewport,
            selection: graph_canvas.selection.clone(),
            hit_test_scene: graph_canvas.hit_test_scene.clone(),
            canvas: graph_canvas,
            can_undo: true,
            can_redo: true,
            active_edit_group: Some(88),
        },
        viewport: material_graph::MaterialGraphViewportState {
            pan_x: -64,
            pan_y: 24,
            zoom_milli: 1_250,
        },
        nodes: vec![
            MaterialGraphNodeViewModel {
                node_id: graph::NodeId::new(11),
                descriptor_key: "texture.sample_color".to_string(),
                label: "Base Color Texture".to_string(),
                position_x: 40,
                position_y: 48,
                input_ports: Vec::new(),
                output_ports: vec![MaterialGraphPortViewModel {
                    port_id: graph::PortId::new(101),
                    name: "color".to_string(),
                    value_type: material_graph::MaterialValueType::Color,
                    connected: true,
                }],
                editable_values: vec![MaterialGraphPropertyViewModel {
                    node_id: graph::NodeId::new(11),
                    key: "uv_scale".to_string(),
                    value_type: material_graph::MaterialValueType::Float,
                    display_value: "1.0".to_string(),
                    required: true,
                }],
                resource_bindings: Vec::new(),
                selected: true,
            },
            MaterialGraphNodeViewModel {
                node_id: graph::NodeId::new(12),
                descriptor_key: "pbr.output".to_string(),
                label: "PBR Output".to_string(),
                position_x: 380,
                position_y: 64,
                input_ports: vec![MaterialGraphPortViewModel {
                    port_id: graph::PortId::new(201),
                    name: "base_color".to_string(),
                    value_type: material_graph::MaterialValueType::Color,
                    connected: true,
                }],
                output_ports: Vec::new(),
                editable_values: Vec::new(),
                resource_bindings: Vec::new(),
                selected: false,
            },
        ],
        edges: vec![MaterialGraphEdgeViewModel {
            edge_id: graph::EdgeId::new(301),
            from_port_id: graph::PortId::new(101),
            to_port_id: graph::PortId::new(201),
        }],
        groups: Vec::new(),
        selected_node_ids: vec![graph::NodeId::new(11)],
        selected_edge_ids: vec![graph::EdgeId::new(301)],
    };

    MaterialGraphCanvasViewModel {
        rows: vec![MaterialGraphSourceRowViewModel {
            asset_id: asset::asset_id(9_001),
            display_name: "Hero Material".to_string(),
            stable_name: "hero.material".to_string(),
            source_id: Some(asset::asset_source_id(9_002)),
            artifact_count: 3,
            is_selected: true,
            has_prior_valid_preservation: true,
        }],
        selected: Some(MaterialGraphSourceDetailViewModel {
            asset_id: asset::asset_id(9_001),
            source_id: Some(asset::asset_source_id(9_002)),
            source_path: Some("assets/materials/hero.material.ron".to_string()),
            document_id: Some(material_graph::MaterialGraphDocumentId::new(9_001)),
            output_target: Some(material_graph::MaterialOutputTarget::RenderMaterial),
            node_count: 2,
            edge_count: 1,
        }),
        graph: graph_editor,
        palette: MaterialNodePaletteViewModel {
            search_query: "pbr".to_string(),
            categories: vec![MaterialNodePaletteCategoryViewModel {
                label: "Surface".to_string(),
                nodes: vec![MaterialNodePaletteItemViewModel {
                    descriptor_key: "pbr.output".to_string(),
                    label: "PBR Output".to_string(),
                    output_targets: vec![material_graph::MaterialOutputTarget::RenderMaterial],
                }],
            }],
        },
        texture_picker: Default::default(),
        sdf_primitives: Vec::new(),
        model_mesh_regions: Vec::new(),
        scene_material_slots: Vec::new(),
        toolbar: MaterialGraphToolbarViewModel::default(),
        validation_overlays: vec![MaterialGraphValidationOverlayViewModel {
            diagnostic_index: Some(0),
            subject_node_id: Some(graph::NodeId::new(12)),
            subject_port_id: Some(graph::PortId::new(201)),
            severity: MaterialGraphValidationSeverity::Warning,
            message: "preview build required".to_string(),
            active: true,
        }],
        active_diagnostic_index: Some(0),
        node_picker: MaterialNodePickerViewModel {
            open: true,
            search_query: "pbr".to_string(),
            highlighted_descriptor_key: Some("pbr.output".to_string()),
            categories: vec![MaterialNodePaletteCategoryViewModel {
                label: "Surface".to_string(),
                nodes: vec![MaterialNodePaletteItemViewModel {
                    descriptor_key: "pbr.output".to_string(),
                    label: "PBR Output".to_string(),
                    output_targets: vec![material_graph::MaterialOutputTarget::RenderMaterial],
                }],
            }],
        },
        shortcuts: vec![
            MaterialGraphShortcutViewModel {
                chord: "Delete".to_string(),
                action: MaterialShortcutAction::DeleteSelection,
            },
            MaterialGraphShortcutViewModel {
                chord: "Cmd+B".to_string(),
                action: MaterialShortcutAction::BuildPreview,
            },
        ],
        undo_redo: MaterialUndoRedoViewModel {
            can_undo: true,
            can_redo: true,
            active_group_id: Some(88),
        },
        catalog_status_lines: vec!["catalog texture resources ready".to_string()],
        diagnostic_rows: Vec::new(),
        resource_binding_diagnostics: Vec::new(),
        diagnostic_lines: vec!["degraded provider: none".to_string()],
    }
}
