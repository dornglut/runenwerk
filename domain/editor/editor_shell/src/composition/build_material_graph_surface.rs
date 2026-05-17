//! File: domain/editor/editor_shell/src/composition/build_material_graph_surface.rs
//! Purpose: Compose the source-backed Material Lab graph surface from typed view models.

use crate::{
    MaterialGraphCanvasViewModel, SurfaceLocalAction, SurfaceLocalRoute, SurfaceRouteTable,
    SurfaceWidgetScope, ToolSurfaceInstanceId, UiNode, WidgetId, button, label, panel, vscroll,
    vstack,
};
use ui_text::FontId;
use ui_theme::ThemeTokens;

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
    let mut children = Vec::new();

    children.push(label(
        scope.widget_id(WidgetId(43_000)),
        format!(
            "graph surface: nodes={} edges={} groups={} overlays={} palette_categories={}",
            view_model.graph.nodes.len(),
            view_model.graph.edges.len(),
            view_model.graph.groups.len(),
            view_model.validation_overlays.len(),
            view_model.palette.categories.len()
        ),
        text_style.clone(),
    ));
    for (index, node) in view_model.graph.nodes.iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(43_100 + index as u64)),
            format!(
                "node {} '{}' at {},{} inputs={} outputs={} properties={} resources={}{}",
                node.node_id.raw(),
                node.label,
                node.position_x,
                node.position_y,
                node.input_ports.len(),
                node.output_ports.len(),
                node.editable_values.len(),
                node.resource_bindings.len(),
                if node.selected { " selected" } else { "" }
            ),
            text_style.clone(),
        ));
    }
    for (index, edge) in view_model.graph.edges.iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(43_500 + index as u64)),
            format!(
                "edge {}: {} -> {}",
                edge.edge_id.raw(),
                edge.from_port_id.raw(),
                edge.to_port_id.raw()
            ),
            text_style.clone(),
        ));
    }
    for (index, category) in view_model.palette.categories.iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(43_900 + index as u64)),
            format!("palette {}: {} nodes", category.label, category.nodes.len()),
            text_style.clone(),
        ));
    }
    for (index, overlay) in view_model.validation_overlays.iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(44_200 + index as u64)),
            format!(
                "overlay {:?} node={:?} port={:?}: {}",
                overlay.severity,
                overlay.subject_node_id.map(|id| id.raw()),
                overlay.subject_port_id.map(|id| id.raw()),
                overlay.message
            ),
            text_style.clone(),
        ));
    }
    for (index, shortcut) in view_model.shortcuts.iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(44_600 + index as u64)),
            format!("shortcut {} -> {:?}", shortcut.chord, shortcut.action),
            text_style.clone(),
        ));
    }
    for (index, line) in lines.into_iter().enumerate() {
        children.push(label(
            scope.widget_id(WidgetId(45_000 + index as u64)),
            line,
            text_style.clone(),
        ));
    }
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

    let body = vstack(
        scope.widget_id(WidgetId(42_000)),
        theme.spacing.xs,
        children,
    );
    let scroll = vscroll(scope.widget_id(WidgetId(42_001)), theme.clone(), vec![body]);
    (
        panel(
            scope.widget_id(WidgetId(42_002)),
            theme.clone(),
            vec![scroll],
        ),
        routes,
    )
}
