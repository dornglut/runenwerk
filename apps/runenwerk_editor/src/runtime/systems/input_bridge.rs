use editor_core::{ComponentTypeId, EntityId};
use editor_shell::ShellCommand;
use editor_viewport::ViewportHitResult;
use engine::WindowState;
use engine::plugins::input::domain::action;
use engine::plugins::render::{EditorGizmoAxis, EditorPickingResultResource, EditorPickingTarget};
use engine::runtime::{Res, ResMut};
use ui_input::{Modifiers, PointerButton, PointerEvent, PointerEventKind, UiInputEvent};
use ui_math::{UiPoint, UiRect, UiVector};

use crate::editor_features::viewport::ViewportInteractionCommand;
use crate::editor_runtime::{redo_last_scene_transaction, undo_last_scene_transaction};
use crate::runtime::app::{
    ACTION_EDITOR_REDO, ACTION_EDITOR_TOOL_SELECT, ACTION_EDITOR_TOOL_TRANSLATE, ACTION_EDITOR_UNDO,
};
use crate::runtime::resources::{EditorHostResource, EditorInputBridgeState};
use crate::shell::{TRANSLATE_TOOL_ID, dispatch_shell_command};

pub fn dispatch_editor_input_system(
    input: Res<engine::plugins::InputState>,
    window: Res<WindowState>,
    mut host: ResMut<EditorHostResource>,
    mut bridge: ResMut<EditorInputBridgeState>,
    mut picking: ResMut<EditorPickingResultResource>,
) {
    dispatch_shortcuts(&input, &mut host);

    let bounds = window_bounds(&window);
    let viewport_bounds = viewport_bounds(
        host.shell_state.last_tree(),
        host.shell_state.last_bounds(),
        host.shell_state.runtime(),
    )
    .unwrap_or(bounds);
    let position = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    let previous = UiPoint::new(bridge.last_mouse_position.0, bridge.last_mouse_position.1);
    picking.set_cursor(
        (position.x, position.y),
        (
            viewport_bounds.x,
            viewport_bounds.y,
            viewport_bounds.width,
            viewport_bounds.height,
        ),
    );

    if position != previous {
        dispatch_pointer_event(
            &mut host,
            bounds,
            PointerEventKind::Move,
            position,
            position - previous,
            None,
        );
    }

    if input.left_mouse_pressed() {
        dispatch_pointer_event(
            &mut host,
            bounds,
            PointerEventKind::Down,
            position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
        );

        if viewport_bounds.contains(position) {
            dispatch_viewport_pointer_down(&mut host, &picking);
        }
    }

    if input.left_mouse_down()
        && host.app.viewport_interaction_state().drag_in_progress()
        && position != previous
    {
        let amount = position.x - previous.x;
        if amount != 0.0 {
            if let Err(error) = host.app.dispatch_viewport_interaction_command(
                ViewportInteractionCommand::PointerDragAxis { amount },
            ) {
                eprintln!("viewport axis drag failed: {error}");
            }
        }
    }

    if input.left_mouse_released() {
        dispatch_pointer_event(
            &mut host,
            bounds,
            PointerEventKind::Up,
            position,
            UiVector::ZERO,
            Some(PointerButton::Primary),
        );

        if host.app.viewport_interaction_state().drag_in_progress() {
            if let Err(error) = host
                .app
                .dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerUp)
            {
                eprintln!("viewport pointer-up failed: {error}");
            }
        }
    }

    bridge.last_mouse_position = (position.x, position.y);
}

fn dispatch_shortcuts(input: &engine::plugins::InputState, host: &mut EditorHostResource) {
    if input.action_pressed(ACTION_EDITOR_UNDO) {
        if let Err(error) = undo_last_scene_transaction(host.app.runtime_mut()) {
            eprintln!("undo shortcut failed: {error}");
        }
    }

    if input.action_pressed(ACTION_EDITOR_REDO) {
        if let Err(error) = redo_last_scene_transaction(host.app.runtime_mut()) {
            eprintln!("redo shortcut failed: {error}");
        }
    }

    if input.action_pressed(ACTION_EDITOR_TOOL_SELECT)
        || input.action_pressed(action::UI_EDITOR_RESTORE_ALL)
    {
        if let Err(error) = dispatch_shell_command(&mut host.app, ShellCommand::ActivateSelectTool)
        {
            eprintln!("select-tool shortcut failed: {error}");
        }
    }

    if input.action_pressed(ACTION_EDITOR_TOOL_TRANSLATE)
        || input.action_pressed(action::UI_EDITOR_HIDE_SELECTED)
    {
        if let Err(error) =
            dispatch_shell_command(&mut host.app, ShellCommand::ActivateTranslateTool)
        {
            eprintln!("translate-tool shortcut failed: {error}");
        }
    }
}

fn dispatch_pointer_event(
    host: &mut EditorHostResource,
    bounds: UiRect,
    kind: PointerEventKind,
    position: UiPoint,
    delta: UiVector,
    button: Option<PointerButton>,
) {
    let event = UiInputEvent::Pointer(PointerEvent {
        kind,
        position,
        delta,
        button,
        modifiers: Modifiers::default(),
        click_count: 1,
    });

    if let Err(error) =
        host.app
            .dispatch_shell_input(&mut host.shell_state, bounds, &host.theme, &event)
    {
        eprintln!("editor shell input dispatch failed: {error}");
    }
}

fn window_bounds(window: &WindowState) -> UiRect {
    let width = window.size_px.0.max(1) as f32;
    let height = window.size_px.1.max(1) as f32;
    UiRect::new(0.0, 0.0, width, height)
}

fn viewport_bounds(
    tree: Option<&editor_shell::UiTree>,
    bounds: Option<UiRect>,
    runtime: &editor_shell::UiRuntime,
) -> Option<UiRect> {
    let tree = tree?;
    let bounds = bounds?;
    let layouts = runtime.compute_layout(tree, bounds);
    layouts
        .get(&editor_shell::VIEWPORT_PANEL_WIDGET_ID)
        .map(|layout| layout.bounds)
}

fn dispatch_viewport_pointer_down(
    host: &mut EditorHostResource,
    picking: &EditorPickingResultResource,
) {
    let hit = map_viewport_hit(
        picking,
        host.app.runtime().selected_entity(),
        host.app.runtime().session().active_tool(),
    );

    if let Err(error) = host
        .app
        .dispatch_viewport_interaction_command(ViewportInteractionCommand::PointerDown { hit })
    {
        eprintln!("viewport pointer-down failed: {error}");
    }
}

fn map_viewport_hit(
    picking: &EditorPickingResultResource,
    selected_entity: Option<EntityId>,
    active_tool: Option<editor_core::ToolId>,
) -> ViewportHitResult {
    let distance = if picking.hit.distance.is_finite() {
        picking.hit.distance
    } else {
        0.0
    };

    match picking.hit.target {
        EditorPickingTarget::None => fallback_viewport_hit(selected_entity, active_tool),
        EditorPickingTarget::Grid => ViewportHitResult::grid(distance),
        EditorPickingTarget::Entity(entity) => {
            ViewportHitResult::entity(EntityId(entity), distance)
        }
        EditorPickingTarget::ComponentHandle {
            entity,
            component_type,
        } => ViewportHitResult::component_handle(
            EntityId(entity),
            ComponentTypeId(component_type),
            distance,
        ),
        EditorPickingTarget::GizmoAxis(axis) => {
            ViewportHitResult::gizmo_axis(editor_axis_label(axis), distance)
        }
    }
}

fn fallback_viewport_hit(
    selected_entity: Option<EntityId>,
    active_tool: Option<editor_core::ToolId>,
) -> ViewportHitResult {
    if selected_entity.is_some() && active_tool == Some(TRANSLATE_TOOL_ID) {
        return ViewportHitResult::gizmo_axis("X", 0.0);
    }

    ViewportHitResult::none()
}

fn editor_axis_label(axis: EditorGizmoAxis) -> &'static str {
    match axis {
        EditorGizmoAxis::X => "X",
        EditorGizmoAxis::Y => "Y",
        EditorGizmoAxis::Z => "Z",
    }
}
