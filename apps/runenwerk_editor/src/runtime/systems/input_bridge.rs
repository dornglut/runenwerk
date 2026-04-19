use editor_shell::ShellCommand;
use editor_viewport::ViewportId;
use engine::WindowState;
use engine::plugins::input::domain::action;
use engine::plugins::render::{EditorGizmoAxis, EditorPickingResultResource, EditorPickingTarget};
use engine::runtime::{Res, ResMut};
use ui_input::{Modifiers, PointerButton, PointerEvent, PointerEventKind, UiInputEvent};
use ui_math::{UiPoint, UiRect, UiVector};

use crate::editor_features::viewport::ViewportInteractionCommand;
use crate::runtime::app::{
    ACTION_EDITOR_REDO, ACTION_EDITOR_TOOL_SELECT, ACTION_EDITOR_TOOL_TRANSLATE, ACTION_EDITOR_UNDO,
};
use crate::runtime::resources::{EditorHostResource, EditorInputBridgeState, scaled_shell_theme};
use crate::runtime::viewport::{
    MAIN_VIEWPORT_ID, ViewportArtifactObservationResource, ViewportLayoutMapResource,
    ViewportPresentationStateResource,
};
use crate::runtime::{
    build_viewport_picking_product_frame, viewport_hit_from_picking_product,
};
use crate::shell::dispatch_shell_command;

#[derive(Debug, Clone, Copy)]
struct ViewportPointerRoute {
    viewport_id: ViewportId,
    host_widget_id: editor_shell::WidgetId,
    local_position: UiPoint,
}

pub fn dispatch_editor_input_system(
    input: Res<engine::plugins::InputState>,
    window: Res<WindowState>,
    mut host: ResMut<EditorHostResource>,
    mut bridge: ResMut<EditorInputBridgeState>,
    picking: Res<EditorPickingResultResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    viewport_layout_map: Res<ViewportLayoutMapResource>,
) {
    dispatch_shortcuts(
        &input,
        &mut host,
        &mut viewport_presentations,
        &viewport_observations,
    );

    let bounds = window_bounds(&window);
    let shell_theme = scaled_shell_theme(&host.theme, window.scale_factor);
    let viewport_products = viewport_observations.frame_for(MAIN_VIEWPORT_ID);
    let position = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    let previous = UiPoint::new(bridge.last_mouse_position.0, bridge.last_mouse_position.1);

    if picking.revision != bridge.last_logged_picking_revision {
        bridge.last_logged_picking_revision = picking.revision;
    }

    if position != previous {
        let _ = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Move,
            position,
            position - previous,
            None,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
        );
    }

    if input.scroll_delta.abs() > f32::EPSILON {
        let _ = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Scroll,
            position,
            UiVector::new(0.0, input.scroll_delta),
            None,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
        );
    }

    if input.left_mouse_pressed() {
        let outcome = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Down,
            position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
        );

        let pointer_route = outcome
            .as_ref()
            .and_then(|value| viewport_pointer_route(&viewport_layout_map, &value.dispatch, position));
        if let Some(route) = pointer_route {
            dispatch_viewport_pointer_down(&mut host, &picking, position, route);
        } else if host.app.debug_logs_enabled() {
            host.app.append_console_line(format!(
                "[input] pointer-down routed to shell only: cursor=({:.1},{:.1})",
                position.x, position.y
            ));
        }
    }

    if input.left_mouse_down()
        && host.app.viewport_interaction_state().drag_in_progress()
        && position != previous
        && viewport_capture_active(&host, &viewport_layout_map)
    {
        let amount = position.x - previous.x;
        if amount != 0.0
            && let Err(error) = host.app.dispatch_viewport_interaction_command(
                ViewportInteractionCommand::PointerDragAxis { amount },
            )
        {
            eprintln!("viewport axis drag failed: {error}");
        }
    }

    if input.left_mouse_released() {
        let outcome = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Up,
            position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
        );
        let routed_release = outcome
            .as_ref()
            .and_then(|value| viewport_pointer_route(&viewport_layout_map, &value.dispatch, position))
            .is_some();

        if host.app.viewport_interaction_state().drag_in_progress()
            && routed_release
            && let Err(error) = host
                .app
                .dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerUp)
        {
            eprintln!("viewport pointer-up failed: {error}");
        }
    }

    bridge.last_mouse_position = (position.x, position.y);
}

fn dispatch_shortcuts(
    input: &engine::plugins::InputState,
    host: &mut EditorHostResource,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_observations: &ViewportArtifactObservationResource,
) {
    if input.action_pressed(ACTION_EDITOR_UNDO)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            ShellCommand::Undo,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
        )
    {
        eprintln!("undo shortcut failed: {error}");
    }

    if input.action_pressed(ACTION_EDITOR_REDO)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            ShellCommand::Redo,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
        )
    {
        eprintln!("redo shortcut failed: {error}");
    }

    if input.action_pressed(ACTION_EDITOR_TOOL_SELECT)
        || input.action_pressed(action::UI_EDITOR_RESTORE_ALL)
    {
        if let Err(error) = dispatch_shell_command(
            &mut host.app,
            ShellCommand::ActivateSelectTool,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
        )
        {
            eprintln!("select-tool shortcut failed: {error}");
        }
    }

    if input.action_pressed(ACTION_EDITOR_TOOL_TRANSLATE)
        || input.action_pressed(action::UI_EDITOR_HIDE_SELECTED)
    {
        if let Err(error) = dispatch_shell_command(
            &mut host.app,
            ShellCommand::ActivateTranslateTool,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
        )
        {
            eprintln!("translate-tool shortcut failed: {error}");
        }
    }
}

fn dispatch_pointer_event(
    host: &mut EditorHostResource,
    shell_theme: &ui_theme::ThemeTokens,
    bounds: UiRect,
    kind: PointerEventKind,
    position: UiPoint,
    delta: UiVector,
    button: Option<PointerButton>,
    viewport_products: Option<&editor_viewport::ArtifactObservationFrame>,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
) -> Option<editor_shell::UiInputOutcome> {
    let event = UiInputEvent::Pointer(PointerEvent {
        kind,
        position,
        delta,
        button,
        modifiers: Modifiers::default(),
        click_count: 1,
    });

    match host
        .app
        .dispatch_shell_input(
            &mut host.shell_state,
            bounds,
            shell_theme,
            &event,
            viewport_products,
            viewport_presentations,
            viewport_observations,
        )
    {
        Ok(outcome) => Some(outcome),
        Err(error) => {
            eprintln!("editor shell input dispatch failed: {error}");
            None
        }
    }
}

fn viewport_pointer_route(
    layout_map: &ViewportLayoutMapResource,
    dispatch: &editor_shell::UiInputDispatchResult,
    position: UiPoint,
) -> Option<ViewportPointerRoute> {
    let host_widget_id = dispatch.target?;
    let viewport_id = layout_map.viewport_for_widget(host_widget_id)?;
    let entry = layout_map.entry_for_viewport(viewport_id)?;
    Some(ViewportPointerRoute {
        viewport_id,
        host_widget_id,
        local_position: UiPoint::new(position.x - entry.bounds.x, position.y - entry.bounds.y),
    })
}

fn viewport_capture_active(
    host: &EditorHostResource,
    layout_map: &ViewportLayoutMapResource,
) -> bool {
    host.shell_state
        .runtime()
        .state()
        .captured_widget
        .and_then(|widget_id| layout_map.viewport_for_widget(widget_id))
        .is_some()
}

fn window_bounds(window: &WindowState) -> UiRect {
    let width = window.size_px.0.max(1) as f32;
    let height = window.size_px.1.max(1) as f32;
    UiRect::new(0.0, 0.0, width, height)
}

fn dispatch_viewport_pointer_down(
    host: &mut EditorHostResource,
    picking: &EditorPickingResultResource,
    position: UiPoint,
    route: ViewportPointerRoute,
) {
    let expression = build_viewport_picking_product_frame(
        route.viewport_id,
        picking,
        host.app.runtime().current_scene_reality_version(),
    );
    let hit = viewport_hit_from_picking_product(&expression);
    let selection_before = host.app.runtime().selected_entity();

    if host.app.debug_logs_enabled() {
        host.app.append_console_line(format!(
            "[input] viewport pointer-down viewport={} widget={} cursor=({:.1},{:.1}) local=({:.1},{:.1}) hit={} dist={:.3} expr_frame={} sel_before={:?}",
            route.viewport_id.0,
            route.host_widget_id.0,
            position.x,
            position.y,
            route.local_position.x,
            route.local_position.y,
            picking_target_label(picking.hit.target),
            picking.hit.distance,
            expression.expression.metadata.frame_id.0,
            selection_before
        ));
    }

    let result = host
        .app
        .dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDown { hit });
    if let Err(error) = result {
        eprintln!("viewport pointer-down failed: {error}");
        return;
    }

    if host.app.debug_logs_enabled() {
        host.app.append_console_line(format!(
            "[input] viewport command=PointerDown sel_after={:?}",
            host.app.runtime().selected_entity()
        ));
    }
}

fn editor_axis_label(axis: EditorGizmoAxis) -> &'static str {
    match axis {
        EditorGizmoAxis::X => "X",
        EditorGizmoAxis::Y => "Y",
        EditorGizmoAxis::Z => "Z",
    }
}

fn picking_target_label(target: EditorPickingTarget) -> String {
    match target {
        EditorPickingTarget::None => "none".to_string(),
        EditorPickingTarget::Grid => "grid".to_string(),
        EditorPickingTarget::Entity(entity) => format!("entity:{entity}"),
        EditorPickingTarget::ComponentHandle {
            entity,
            component_type,
        } => format!("component:{entity}:{component_type}"),
        EditorPickingTarget::GizmoAxis(axis) => format!("gizmo:{}", editor_axis_label(axis)),
    }
}
