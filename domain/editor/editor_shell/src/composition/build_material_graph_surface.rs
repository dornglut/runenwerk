//! File: domain/editor/editor_shell/src/composition/build_material_graph_surface.rs
//! Purpose: Compose the source-backed Material Lab graph surface from typed view models.

use crate::{
    MaterialGraphCanvasViewModel, MaterialGraphNodeViewModel, MaterialSurfaceAction,
    SurfaceLocalAction, SurfaceLocalRoute, SurfaceRouteTable, SurfaceWidgetScope,
    ToolSurfaceInstanceId, UiNode, WidgetId, button, hstack_with_policies, label, panel,
    text_input, vscroll, vstack_with_policies,
};
use ui_layout::SizePolicy;
use ui_math::UiSize;
use ui_text::FontId;
use ui_theme::ThemeTokens;
use ui_tree::{GraphCanvasNode, PopupAlign, PopupFlipPolicy, PopupNode, PopupSide, UiNodeKind};

pub const MATERIAL_GRAPH_CANVAS_WIDGET_ID: WidgetId = WidgetId(42_100);

pub fn build_material_graph_surface(
    theme: &ThemeTokens,
    surface_id: ToolSurfaceInstanceId,
    view_model: &MaterialGraphCanvasViewModel,
    lines: Vec<String>,
    actions: Vec<(String, SurfaceLocalAction)>,
) -> (UiNode, SurfaceRouteTable) {
    let scope = SurfaceWidgetScope::new(surface_id);
    let text_style = theme.body_small_text_style(FontId(1));
    let mut routes = SurfaceRouteTable::empty();

    let toolbar = build_toolbar(
        scope,
        theme,
        text_style.clone(),
        view_model,
        actions,
        &mut routes,
    );
    let source_panel =
        build_source_panel(scope, theme, text_style.clone(), view_model, &mut routes);
    let graph_canvas =
        build_graph_canvas(scope, theme, text_style.clone(), view_model, &mut routes);
    let inspector_panel =
        build_inspector_panel(scope, theme, text_style.clone(), view_model, &mut routes);
    let palette_panel =
        build_palette_panel(scope, theme, text_style.clone(), view_model, &mut routes);
    let diagnostics_panel = build_diagnostics_panel(
        scope,
        theme,
        text_style.clone(),
        view_model,
        lines,
        &mut routes,
    );

    let side_rail = vstack_with_policies(
        scope.widget_id(WidgetId(42_220)),
        theme.spacing.sm,
        vec![
            SizePolicy::Auto,
            SizePolicy::flex(1.0),
            SizePolicy::flex(1.0),
        ],
        vec![inspector_panel, palette_panel, diagnostics_panel],
    );
    let body = hstack_with_policies(
        scope.widget_id(WidgetId(42_230)),
        theme.spacing.sm,
        vec![
            SizePolicy::Fixed(240.0),
            SizePolicy::flex(1.0),
            SizePolicy::Fixed(320.0),
        ],
        vec![source_panel, graph_canvas, side_rail],
    );
    let mut root_policies = vec![SizePolicy::Auto, SizePolicy::flex(1.0)];
    let mut root_children = vec![toolbar, body];
    if let Some(node_picker) =
        build_node_picker_popup(scope, theme, text_style.clone(), view_model, &mut routes)
    {
        root_policies.push(SizePolicy::Auto);
        root_children.push(node_picker);
    }
    let root = vstack_with_policies(
        scope.widget_id(WidgetId(42_240)),
        theme.spacing.sm,
        root_policies,
        root_children,
    );

    (
        panel(scope.widget_id(WidgetId(42_250)), theme.clone(), vec![root]),
        routes,
    )
}

fn build_toolbar(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    actions: Vec<(String, SurfaceLocalAction)>,
    routes: &mut SurfaceRouteTable,
) -> UiNode {
    let mut children = Vec::new();
    push_material_button(
        &mut children,
        routes,
        scope.widget_id(WidgetId(42_020)),
        "Add node",
        text_style.clone(),
        theme,
        true,
        MaterialSurfaceAction::OpenNodePicker,
    );
    push_material_button(
        &mut children,
        routes,
        scope.widget_id(WidgetId(42_021)),
        "Undo",
        text_style.clone(),
        theme,
        view_model.undo_redo.can_undo,
        MaterialSurfaceAction::UndoMaterialEdit,
    );
    push_material_button(
        &mut children,
        routes,
        scope.widget_id(WidgetId(42_022)),
        "Redo",
        text_style.clone(),
        theme,
        view_model.undo_redo.can_redo,
        MaterialSurfaceAction::RedoMaterialEdit,
    );
    push_material_button(
        &mut children,
        routes,
        scope.widget_id(WidgetId(42_023)),
        "Build preview",
        text_style.clone(),
        theme,
        view_model.selected.is_some(),
        MaterialSurfaceAction::BuildSelectedMaterialPreview,
    );
    push_material_button(
        &mut children,
        routes,
        scope.widget_id(WidgetId(42_024)),
        "Focus preview",
        text_style.clone(),
        theme,
        view_model.selected.is_some(),
        MaterialSurfaceAction::SelectPreviewProduct {
            selection: material_graph::MaterialGraphPreviewSelection::MaterialPreviewProduct,
        },
    );
    for (index, (label_text, action)) in actions.into_iter().enumerate() {
        let widget_id = scope.widget_id(WidgetId(47_000 + index as u64));
        children.push(button(
            widget_id,
            label_text,
            text_style.clone(),
            theme.clone(),
        ));
        routes.insert(widget_id, SurfaceLocalRoute::new(action));
    }
    hstack_with_policies(
        scope.widget_id(WidgetId(42_010)),
        theme.spacing.xs,
        vec![SizePolicy::Auto; children.len()],
        children,
    )
}

fn push_material_button(
    children: &mut Vec<UiNode>,
    routes: &mut SurfaceRouteTable,
    widget_id: WidgetId,
    label_text: impl Into<String>,
    text_style: ui_text::TextStyle,
    theme: &ThemeTokens,
    enabled: bool,
    action: MaterialSurfaceAction,
) {
    let mut node = button(widget_id, label_text, text_style, theme.clone());
    if let UiNodeKind::Button(button) = &mut node.kind {
        button.enabled = enabled;
    }
    children.push(node);
    if enabled {
        routes.insert(
            widget_id,
            SurfaceLocalRoute::new(SurfaceLocalAction::Material(action)),
        );
    }
}

fn build_source_panel(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    routes: &mut SurfaceRouteTable,
) -> UiNode {
    let mut children = vec![label(
        scope.widget_id(WidgetId(42_300)),
        "Sources",
        text_style.clone(),
    )];
    for (index, row) in view_model.rows.iter().enumerate() {
        let widget_id = scope.widget_id(WidgetId(42_310 + index as u64));
        children.push(button(
            widget_id,
            row.display_name.clone(),
            text_style.clone(),
            theme.clone(),
        ));
        routes.insert(
            widget_id,
            SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                MaterialSurfaceAction::SelectMaterialAsset {
                    asset_id: row.asset_id,
                },
            )),
        );
    }
    panel(
        scope.widget_id(WidgetId(42_320)),
        theme.clone(),
        vec![vscroll(
            scope.widget_id(WidgetId(42_321)),
            theme.clone(),
            vec![vstack_with_policies(
                scope.widget_id(WidgetId(42_322)),
                theme.spacing.xs,
                vec![SizePolicy::Auto; children.len()],
                children,
            )],
        )],
    )
}

fn build_graph_canvas(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    routes: &mut SurfaceRouteTable,
) -> UiNode {
    let widget_id = scope.widget_id(MATERIAL_GRAPH_CANVAS_WIDGET_ID);
    routes.insert(widget_id, SurfaceLocalRoute::provider_owned_graph_canvas());
    let mut canvas =
        GraphCanvasNode::new(view_model.graph.graph_editor.canvas.clone(), theme.clone())
            .with_min_size(UiSize::new(620.0, 420.0));
    canvas.text_style = text_style;
    UiNode::new(widget_id, UiNodeKind::GraphCanvas(canvas))
}

fn build_inspector_panel(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    routes: &mut SurfaceRouteTable,
) -> UiNode {
    let mut children = vec![label(
        scope.widget_id(WidgetId(43_000)),
        "Inspector",
        text_style.clone(),
    )];
    if let Some(node) = selected_node(view_model) {
        children.push(label(
            scope.widget_id(WidgetId(43_001)),
            format!("Node {} {}", node.node_id.raw(), node.label),
            text_style.clone(),
        ));
        for (index, value) in node.editable_values.iter().enumerate() {
            let widget_id = scope.widget_id(WidgetId(43_010 + index as u64 * 4));
            children.push(text_input(
                widget_id,
                value.display_value.clone(),
                value.key.clone(),
                text_style.clone(),
                theme.clone(),
            ));
            routes.insert(
                widget_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                    MaterialSurfaceAction::SetNodeValue {
                        node_id: value.node_id,
                        key: value.key.clone(),
                        value: value.display_value.clone(),
                    },
                )),
            );
            children.push(label(
                scope.widget_id(WidgetId(43_011 + index as u64 * 4)),
                format!("{} {:?}", value.key, value.value_type),
                text_style.clone(),
            ));
        }
        for (index, resource) in node.resource_bindings.iter().enumerate() {
            let widget_id = scope.widget_id(WidgetId(43_200 + index as u64 * 4));
            children.push(text_input(
                widget_id,
                resource.reference.clone().unwrap_or_default(),
                resource.key.clone(),
                text_style.clone(),
                theme.clone(),
            ));
            routes.insert(
                widget_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                    MaterialSurfaceAction::PickTextureResource {
                        node_id: resource.node_id,
                        key: resource.key.clone(),
                        stable_id: resource.reference.clone().unwrap_or_default(),
                    },
                )),
            );
            children.push(label(
                scope.widget_id(WidgetId(43_201 + index as u64 * 4)),
                format!("{} {:?}", resource.key, resource.resource_kind),
                text_style.clone(),
            ));
            let search_id = scope.widget_id(WidgetId(43_320 + index as u64 * 100));
            children.push(text_input(
                search_id,
                view_model.texture_picker.search_query.clone(),
                "Search catalog textures",
                text_style.clone(),
                theme.clone(),
            ));
            routes.insert(
                search_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                    MaterialSurfaceAction::SetTextureResourceSearch {
                        query: view_model.texture_picker.search_query.clone(),
                    },
                )),
            );
            let mut matching_options = view_model
                .texture_picker
                .options
                .iter()
                .filter(|option| option.resource_kind == resource.resource_kind)
                .take(12)
                .enumerate()
                .peekable();
            if matching_options.peek().is_none() {
                children.push(label(
                    scope.widget_id(WidgetId(43_330 + index as u64 * 100)),
                    "No catalog texture products match this resource kind",
                    text_style.clone(),
                ));
            } else {
                children.push(label(
                    scope.widget_id(WidgetId(43_331 + index as u64 * 100)),
                    "Texture picker",
                    text_style.clone(),
                ));
                for (option_index, option) in matching_options {
                    let option_widget_id = scope
                        .widget_id(WidgetId(43_340 + index as u64 * 100 + option_index as u64));
                    let validity = if option.valid { "valid" } else { "invalid" };
                    children.push(button(
                        option_widget_id,
                        format!(
                            "{} [{}] product={} artifact={} {}",
                            option.display_name,
                            option.stable_id,
                            option.product_id,
                            option.artifact_id.raw(),
                            validity
                        ),
                        text_style.clone(),
                        theme.clone(),
                    ));
                    routes.insert(
                        option_widget_id,
                        SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                            MaterialSurfaceAction::PickTextureResource {
                                node_id: resource.node_id,
                                key: resource.key.clone(),
                                stable_id: option.stable_id.clone(),
                            },
                        )),
                    );
                    children.push(label(
                        scope
                            .widget_id(WidgetId(43_360 + index as u64 * 100 + option_index as u64)),
                        format!(
                            "descriptor={} uri={}",
                            option.descriptor_hash, option.artifact_uri
                        ),
                        text_style.clone(),
                    ));
                }
            }
        }
    } else if let Some(selected) = &view_model.selected {
        children.push(label(
            scope.widget_id(WidgetId(43_002)),
            format!(
                "Material {} nodes={} edges={}",
                selected.asset_id.raw(),
                selected.node_count,
                selected.edge_count
            ),
            text_style.clone(),
        ));
    }
    panel(
        scope.widget_id(WidgetId(43_900)),
        theme.clone(),
        vec![vstack_with_policies(
            scope.widget_id(WidgetId(43_901)),
            theme.spacing.xs,
            vec![SizePolicy::Auto; children.len()],
            children,
        )],
    )
}

fn build_palette_panel(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    routes: &mut SurfaceRouteTable,
) -> UiNode {
    let mut children = vec![label(
        scope.widget_id(WidgetId(44_000)),
        "Palette",
        text_style.clone(),
    )];
    let search_id = scope.widget_id(WidgetId(44_001));
    children.push(text_input(
        search_id,
        view_model.palette.search_query.clone(),
        "Search nodes",
        text_style.clone(),
        theme.clone(),
    ));
    routes.insert(
        search_id,
        SurfaceLocalRoute::new(SurfaceLocalAction::Material(
            MaterialSurfaceAction::SetMaterialNodePaletteSearch {
                query: view_model.palette.search_query.clone(),
            },
        )),
    );
    let mut button_index = 0_u64;
    for category in &view_model.palette.categories {
        children.push(label(
            scope.widget_id(WidgetId(44_010 + button_index)),
            category.label.clone(),
            text_style.clone(),
        ));
        button_index += 1;
        for item in &category.nodes {
            let widget_id = scope.widget_id(WidgetId(44_100 + button_index));
            children.push(button(
                widget_id,
                item.label.clone(),
                text_style.clone(),
                theme.clone(),
            ));
            routes.insert(
                widget_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                    MaterialSurfaceAction::AddGraphNode {
                        descriptor_key: item.descriptor_key.clone(),
                    },
                )),
            );
            button_index += 1;
        }
    }
    panel(
        scope.widget_id(WidgetId(44_900)),
        theme.clone(),
        vec![vscroll(
            scope.widget_id(WidgetId(44_901)),
            theme.clone(),
            vec![vstack_with_policies(
                scope.widget_id(WidgetId(44_902)),
                theme.spacing.xs,
                vec![SizePolicy::Auto; children.len()],
                children,
            )],
        )],
    )
}

fn build_node_picker_popup(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    routes: &mut SurfaceRouteTable,
) -> Option<UiNode> {
    if !view_model.node_picker.open {
        return None;
    }
    let search_id = scope.widget_id(WidgetId(46_001));
    let close_id = scope.widget_id(WidgetId(46_002));
    let confirm_id = scope.widget_id(WidgetId(46_003));
    let mut children = vec![
        label(
            scope.widget_id(WidgetId(46_000)),
            "Add node",
            text_style.clone(),
        ),
        text_input(
            search_id,
            view_model.node_picker.search_query.clone(),
            "Search nodes",
            text_style.clone(),
            theme.clone(),
        ),
    ];
    routes.insert(
        search_id,
        SurfaceLocalRoute::new(SurfaceLocalAction::Material(
            MaterialSurfaceAction::SetNodePickerSearch {
                query: view_model.node_picker.search_query.clone(),
            },
        )),
    );
    push_material_button(
        &mut children,
        routes,
        confirm_id,
        "Add selected",
        text_style.clone(),
        theme,
        view_model
            .node_picker
            .highlighted_descriptor_key
            .as_ref()
            .is_some(),
        MaterialSurfaceAction::ConfirmNodePickerSelection,
    );
    push_material_button(
        &mut children,
        routes,
        close_id,
        "Close",
        text_style.clone(),
        theme,
        true,
        MaterialSurfaceAction::CloseNodePicker,
    );

    let mut button_index = 0_u64;
    for category in &view_model.node_picker.categories {
        children.push(label(
            scope.widget_id(WidgetId(46_100 + button_index)),
            category.label.clone(),
            text_style.clone(),
        ));
        button_index += 1;
        for item in &category.nodes {
            let widget_id = scope.widget_id(WidgetId(46_200 + button_index));
            let selected = view_model.node_picker.highlighted_descriptor_key.as_ref()
                == Some(&item.descriptor_key);
            children.push(button(
                widget_id,
                if selected {
                    format!("* {}", item.label)
                } else {
                    item.label.clone()
                },
                text_style.clone(),
                theme.clone(),
            ));
            routes.insert(
                widget_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                    MaterialSurfaceAction::HighlightNodePickerNode {
                        descriptor_key: item.descriptor_key.clone(),
                    },
                )),
            );
            button_index += 1;
        }
    }

    Some(UiNode::with_children(
        scope.widget_id(WidgetId(46_900)),
        UiNodeKind::Popup(PopupNode::anchored_outside(
            scope.widget_id(MATERIAL_GRAPH_CANVAS_WIDGET_ID),
            PopupSide::Right,
            PopupAlign::Start,
            PopupFlipPolicy::FlipToFit,
            theme.clone(),
        )),
        vec![vscroll(
            scope.widget_id(WidgetId(46_901)),
            theme.clone(),
            vec![vstack_with_policies(
                scope.widget_id(WidgetId(46_902)),
                theme.spacing.xs,
                vec![SizePolicy::Auto; children.len()],
                children,
            )],
        )],
    ))
}

fn build_diagnostics_panel(
    scope: SurfaceWidgetScope,
    theme: &ThemeTokens,
    text_style: ui_text::TextStyle,
    view_model: &MaterialGraphCanvasViewModel,
    lines: Vec<String>,
    routes: &mut SurfaceRouteTable,
) -> UiNode {
    let mut children = vec![label(
        scope.widget_id(WidgetId(45_000)),
        "Diagnostics",
        text_style.clone(),
    )];
    for (index, overlay) in view_model.validation_overlays.iter().enumerate() {
        let widget_id = scope.widget_id(WidgetId(45_010 + index as u64));
        children.push(button(
            widget_id,
            format!(
                "{}{:?}: {}",
                if overlay.active { "* " } else { "" },
                overlay.severity,
                overlay.message
            ),
            text_style.clone(),
            theme.clone(),
        ));
        if let Some(diagnostic_index) = overlay.diagnostic_index {
            routes.insert(
                widget_id,
                SurfaceLocalRoute::new(SurfaceLocalAction::Material(
                    MaterialSurfaceAction::NavigateDiagnostic { diagnostic_index },
                )),
            );
        }
    }
    for (index, line) in lines.into_iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(45_100 + index as u64)),
            line,
            text_style.clone(),
        ));
    }
    panel(
        scope.widget_id(WidgetId(45_900)),
        theme.clone(),
        vec![vscroll(
            scope.widget_id(WidgetId(45_901)),
            theme.clone(),
            vec![vstack_with_policies(
                scope.widget_id(WidgetId(45_902)),
                theme.spacing.xs,
                vec![SizePolicy::Auto; children.len()],
                children,
            )],
        )],
    )
}

fn selected_node(view_model: &MaterialGraphCanvasViewModel) -> Option<&MaterialGraphNodeViewModel> {
    view_model
        .graph
        .selected_node_ids
        .first()
        .and_then(|selected| {
            view_model
                .graph
                .nodes
                .iter()
                .find(|node| node.node_id == *selected)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MaterialGraphEditorViewModel, MaterialGraphNodeViewModel, MaterialGraphPortViewModel,
        MaterialGraphSourceDetailViewModel, MaterialGraphToolbarViewModel,
        MaterialGraphValidationOverlayViewModel, MaterialGraphValidationSeverity,
        MaterialNodePaletteCategoryViewModel, MaterialNodePaletteItemViewModel,
        MaterialNodePaletteViewModel, MaterialNodePickerViewModel, MaterialUndoRedoViewModel,
    };

    #[test]
    fn material_graph_surface_contains_graph_canvas() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(2).unwrap();
        let view_model = test_view_model();

        let (root, routes) = build_material_graph_surface(
            &ThemeTokens::default(),
            surface_id,
            &view_model,
            Vec::new(),
            Vec::new(),
        );

        assert!(contains_graph_canvas(&root));
        assert!(
            routes
                .get(&crate::surface_widget_id(
                    surface_id,
                    MATERIAL_GRAPH_CANVAS_WIDGET_ID
                ))
                .is_some_and(SurfaceLocalRoute::is_provider_owned_graph_canvas)
        );
    }

    #[test]
    fn material_graph_surface_rejects_label_list_completion() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(3).unwrap();
        let view_model = test_view_model();

        let (root, _) = build_material_graph_surface(
            &ThemeTokens::default(),
            surface_id,
            &view_model,
            vec!["diagnostic".to_string()],
            Vec::new(),
        );

        let canvas = graph_canvas(&root).expect("material graph surface must own graph canvas");
        assert_eq!(canvas.canvas.nodes.len(), 1);
        assert_eq!(canvas.canvas.ports.len(), 2);
        assert_eq!(canvas.canvas.edges.len(), 1);
    }

    #[test]
    fn material_graph_surface_routes_toolbar_and_node_picker_actions() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(4).unwrap();
        let mut view_model = test_view_model();
        view_model.selected = Some(MaterialGraphSourceDetailViewModel {
            asset_id: asset::asset_id(41),
            source_id: Some(asset::asset_source_id(42)),
            source_path: Some("assets/materials/edit.material.ron".to_string()),
            document_id: Some(material_graph::MaterialGraphDocumentId::new(43)),
            output_target: Some(material_graph::MaterialOutputTarget::RenderMaterial),
            node_count: 1,
            edge_count: 0,
        });
        view_model.undo_redo.can_undo = true;
        view_model.undo_redo.can_redo = true;
        view_model.node_picker = MaterialNodePickerViewModel {
            open: true,
            search_query: "base".to_string(),
            highlighted_descriptor_key: Some("pbr.base_color".to_string()),
            categories: vec![MaterialNodePaletteCategoryViewModel {
                label: "Surface".to_string(),
                nodes: vec![MaterialNodePaletteItemViewModel {
                    descriptor_key: "pbr.base_color".to_string(),
                    label: "Base Color".to_string(),
                    output_targets: vec![material_graph::MaterialOutputTarget::RenderMaterial],
                }],
            }],
        };

        let (root, routes) = build_material_graph_surface(
            &ThemeTokens::default(),
            surface_id,
            &view_model,
            Vec::new(),
            Vec::new(),
        );

        assert!(contains_popup(&root));
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(42_020),
            SurfaceLocalAction::Material(MaterialSurfaceAction::OpenNodePicker),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(42_021),
            SurfaceLocalAction::Material(MaterialSurfaceAction::UndoMaterialEdit),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(42_022),
            SurfaceLocalAction::Material(MaterialSurfaceAction::RedoMaterialEdit),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(42_023),
            SurfaceLocalAction::Material(MaterialSurfaceAction::BuildSelectedMaterialPreview),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(46_001),
            SurfaceLocalAction::Material(MaterialSurfaceAction::SetNodePickerSearch {
                query: "base".to_string(),
            }),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(46_003),
            SurfaceLocalAction::Material(MaterialSurfaceAction::ConfirmNodePickerSelection),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(46_002),
            SurfaceLocalAction::Material(MaterialSurfaceAction::CloseNodePicker),
        );
        assert_route_action(
            surface_id,
            &routes,
            WidgetId(46_201),
            SurfaceLocalAction::Material(MaterialSurfaceAction::HighlightNodePickerNode {
                descriptor_key: "pbr.base_color".to_string(),
            }),
        );
    }

    #[test]
    fn material_graph_surface_routes_diagnostic_buttons_to_navigation() {
        let surface_id = ToolSurfaceInstanceId::try_from_raw(5).unwrap();
        let mut view_model = test_view_model();
        view_model.validation_overlays = vec![MaterialGraphValidationOverlayViewModel {
            diagnostic_index: Some(7),
            subject_node_id: Some(graph::NodeId::new(1)),
            subject_port_id: None,
            severity: MaterialGraphValidationSeverity::Blocking,
            message: "output is missing base color".to_string(),
            active: true,
        }];

        let (_, routes) = build_material_graph_surface(
            &ThemeTokens::default(),
            surface_id,
            &view_model,
            Vec::new(),
            Vec::new(),
        );

        assert_route_action(
            surface_id,
            &routes,
            WidgetId(45_010),
            SurfaceLocalAction::Material(MaterialSurfaceAction::NavigateDiagnostic {
                diagnostic_index: 7,
            }),
        );
    }

    fn test_view_model() -> MaterialGraphCanvasViewModel {
        let canvas = ui_graph_editor::GraphCanvasViewModel {
            canvas_id: ui_graph_editor::GraphCanvasId(7),
            nodes: vec![ui_graph_editor::GraphNodeView::new(
                ui_graph_editor::GraphNodeKey(1),
                "Output",
                ui_graph_editor::GraphRect::new(0, 0, 220, 120),
            )],
            ports: vec![
                ui_graph_editor::GraphPortView::new(
                    ui_graph_editor::GraphPortKey(1),
                    ui_graph_editor::GraphNodeKey(1),
                    "in",
                    ui_graph_editor::GraphPortDirection::Input,
                    ui_graph_editor::GraphRect::new(0, 42, 12, 12),
                ),
                ui_graph_editor::GraphPortView::new(
                    ui_graph_editor::GraphPortKey(2),
                    ui_graph_editor::GraphNodeKey(1),
                    "out",
                    ui_graph_editor::GraphPortDirection::Output,
                    ui_graph_editor::GraphRect::new(208, 42, 12, 12),
                ),
            ],
            edges: vec![ui_graph_editor::GraphEdgeView::new(
                ui_graph_editor::GraphEdgeKey(1),
                ui_graph_editor::GraphPortKey(2),
                ui_graph_editor::GraphPortKey(1),
                ui_graph_editor::GraphPoint::new(214, 48),
                ui_graph_editor::GraphPoint::new(6, 48),
                ui_graph_editor::GraphRect::new(0, 40, 220, 16),
            )],
            hit_test_scene: ui_graph_editor::GraphHitTestScene::default()
                .with_canvas_rect(ui_graph_editor::GraphRect::new(0, 0, 640, 480)),
            ..ui_graph_editor::GraphCanvasViewModel::new(ui_graph_editor::GraphCanvasId(7))
        };
        MaterialGraphCanvasViewModel {
            rows: Vec::new(),
            selected: None,
            graph: MaterialGraphEditorViewModel {
                graph_editor: ui_graph_editor::GraphEditorViewModel {
                    canvas,
                    ..ui_graph_editor::GraphEditorViewModel::default()
                },
                nodes: vec![MaterialGraphNodeViewModel {
                    node_id: graph::NodeId::new(1),
                    descriptor_key: "pbr.output".to_string(),
                    label: "Output".to_string(),
                    position_x: 0,
                    position_y: 0,
                    input_ports: vec![MaterialGraphPortViewModel {
                        port_id: graph::PortId::new(1),
                        name: "in".to_string(),
                        value_type: material_graph::MaterialValueType::Float,
                        connected: true,
                    }],
                    output_ports: vec![MaterialGraphPortViewModel {
                        port_id: graph::PortId::new(2),
                        name: "out".to_string(),
                        value_type: material_graph::MaterialValueType::Float,
                        connected: true,
                    }],
                    editable_values: Vec::new(),
                    resource_bindings: Vec::new(),
                    selected: false,
                }],
                ..MaterialGraphEditorViewModel::default()
            },
            palette: MaterialNodePaletteViewModel {
                search_query: String::new(),
                categories: Vec::new(),
            },
            texture_picker: Default::default(),
            toolbar: MaterialGraphToolbarViewModel::default(),
            validation_overlays: Vec::new(),
            active_diagnostic_index: None,
            node_picker: MaterialNodePickerViewModel::default(),
            shortcuts: Vec::new(),
            undo_redo: MaterialUndoRedoViewModel::default(),
            catalog_status_lines: Vec::new(),
            diagnostic_rows: Vec::new(),
            resource_binding_diagnostics: Vec::new(),
            diagnostic_lines: Vec::new(),
        }
    }

    fn contains_graph_canvas(node: &UiNode) -> bool {
        graph_canvas(node).is_some()
    }

    fn contains_popup(node: &UiNode) -> bool {
        if matches!(node.kind, UiNodeKind::Popup(_)) {
            return true;
        }
        node.children.iter().any(contains_popup)
    }

    fn graph_canvas(node: &UiNode) -> Option<&GraphCanvasNode> {
        if let UiNodeKind::GraphCanvas(canvas) = &node.kind {
            return Some(canvas);
        }
        node.children.iter().find_map(graph_canvas)
    }

    fn assert_route_action(
        surface_id: ToolSurfaceInstanceId,
        routes: &SurfaceRouteTable,
        local_widget_id: WidgetId,
        expected: SurfaceLocalAction,
    ) {
        assert_eq!(
            routes
                .get(&crate::surface_widget_id(surface_id, local_widget_id))
                .and_then(SurfaceLocalRoute::action),
            Some(&expected),
        );
    }
}
