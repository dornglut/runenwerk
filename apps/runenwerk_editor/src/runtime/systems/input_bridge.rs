use editor_shell::ShellCommand;
use editor_viewport::ViewportId;
use engine::plugins::input::domain::action;
use engine::plugins::render::{EditorGizmoAxis, EditorPickingTarget};
use engine::runtime::{Res, ResMut};
use engine::{WindowCursorIcon, WindowState};
use scene::LocalTransform;
use ui_input::{
    EventPropagation, Modifiers, PointerButton, PointerEvent, PointerEventKind, UiInputEvent,
};
use ui_math::{UiPoint, UiRect, UiVector};

use crate::editor_features::viewport::ViewportInteractionCommand;
use crate::runtime::app::{
    ACTION_EDITOR_REDO, ACTION_EDITOR_TOOL_ROTATE, ACTION_EDITOR_TOOL_SCALE,
    ACTION_EDITOR_TOOL_SELECT, ACTION_EDITOR_TOOL_TRANSLATE, ACTION_EDITOR_UNDO,
    ACTION_EDITOR_VIEWPORT_FOCUS, ACTION_EDITOR_VIEWPORT_TOOL_RADIAL,
};
use crate::runtime::resources::{
    EditorCameraPointerButton, EditorHostResource, EditorInputBridgeState, EditorPointerOwner,
    scaled_shell_theme,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportRenderStateCommand,
    ViewportRenderStateCommandQueueResource, resolve_structural_viewport_products,
};
use crate::runtime::{build_viewport_picking_product_frame, viewport_hit_from_picking_product};
use crate::shell::dispatch_shell_command;
use crate::shell::{RunenwerkEditorShellController, RunenwerkEditorShellState, ShellCursorIntent};

#[derive(Debug, Clone, Copy)]
struct ViewportPointerRoute {
    tool_surface_id: editor_shell::ToolSurfaceInstanceId,
    viewport_id: ViewportId,
    host_widget_id: editor_shell::WidgetId,
    structural_context: editor_shell::StructuralWidgetRoutingContext,
    local_position: UiPoint,
}

#[allow(clippy::too_many_arguments)]
pub fn dispatch_editor_input_system(
    input: Res<engine::plugins::InputState>,
    mut window: ResMut<WindowState>,
    mut host: ResMut<EditorHostResource>,
    mut bridge: ResMut<EditorInputBridgeState>,
    picking_results: Res<ViewportPickingResultsResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    mut viewport_render_commands: ResMut<ViewportRenderStateCommandQueueResource>,
) {
    let bounds = window_bounds(&window);
    let shell_theme = scaled_shell_theme(&host.theme, window.scale_factor);
    let viewport_products = resolve_structural_viewport_products(
        &host.shell_state,
        &viewport_observations,
        &tool_surface_bindings,
    );
    let position = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    let previous = UiPoint::new(bridge.last_mouse_position.0, bridge.last_mouse_position.1);
    if let Some(binding) = tool_surface_bindings.binding_containing_cursor(position) {
        bridge.last_target_viewport = Some(binding.viewport_id);
    }
    let preferred_viewport_id = bridge.last_target_viewport;
    let modifiers = Modifiers {
        shift: input.shift_down(),
        ctrl: false,
        alt: false,
        meta: false,
    };

    dispatch_global_shortcuts(
        &input,
        &mut host,
        &mut viewport_presentations,
        &viewport_observations,
        &tool_surface_bindings,
    );

    if picking_results.global_revision() != bridge.last_logged_picking_revision {
        bridge.last_logged_picking_revision = picking_results.global_revision();
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
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
    }

    if input.scroll_delta.abs() > f32::EPSILON {
        let outcome = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Scroll,
            position,
            UiVector::new(0.0, input.scroll_delta),
            None,
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
        if let Some(binding) = fallback_viewport_binding(&tool_surface_bindings, position)
            && !pointer_event_consumed_by_ui(&outcome)
        {
            bridge.last_target_viewport = Some(binding.viewport_id);
            viewport_render_commands.push(ViewportRenderStateCommand::ZoomCamera {
                viewport_id: binding.viewport_id,
                scroll_delta: input.scroll_delta,
            });
        }
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
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );

        let pointer_route = outcome.as_ref().and_then(|value| {
            viewport_pointer_route(
                &host.shell_state,
                &tool_surface_bindings,
                &value.dispatch,
                position,
            )
        });
        if let Some(route) = pointer_route {
            bridge.pointer_owner = EditorPointerOwner::ViewportTool {
                tool_surface_id: route.tool_surface_id,
            };
            dispatch_viewport_pointer_down(&mut host, &picking_results, position, route);
        } else {
            bridge.pointer_owner = EditorPointerOwner::None;
            if host.app.debug_logs_enabled() {
                host.app.append_console_input(format!(
                    "[input] pointer-down routed to shell only: cursor=({:.1},{:.1})",
                    position.x, position.y
                ));
            }
        }
    }

    if input.middle_mouse_pressed() {
        let outcome = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Down,
            position,
            UiVector::ZERO,
            Some(PointerButton::Middle),
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
        if let Some(route) = outcome.as_ref().and_then(|value| {
            viewport_pointer_route(
                &host.shell_state,
                &tool_surface_bindings,
                &value.dispatch,
                position,
            )
        }) {
            bridge.active_camera_viewport = Some(route.viewport_id);
            bridge.last_target_viewport = Some(route.viewport_id);
            bridge.pointer_owner = EditorPointerOwner::ViewportCamera {
                viewport_id: route.viewport_id,
                button: EditorCameraPointerButton::Middle,
            };
        } else if outcome
            .as_ref()
            .and_then(|value| value.dispatch.target)
            .is_some()
        {
            bridge.active_camera_viewport = None;
            bridge.pointer_owner = EditorPointerOwner::UiMiddleScroll;
        } else {
            bridge.active_camera_viewport = None;
            bridge.pointer_owner = EditorPointerOwner::None;
        }
    }

    if input.right_mouse_pressed() {
        let outcome = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Down,
            position,
            UiVector::ZERO,
            Some(PointerButton::Secondary),
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
        if let Some(route) = outcome.as_ref().and_then(|value| {
            viewport_pointer_route(
                &host.shell_state,
                &tool_surface_bindings,
                &value.dispatch,
                position,
            )
        }) {
            bridge.active_camera_viewport = Some(route.viewport_id);
            bridge.last_target_viewport = Some(route.viewport_id);
            bridge.pointer_owner = EditorPointerOwner::ViewportCamera {
                viewport_id: route.viewport_id,
                button: EditorCameraPointerButton::Secondary,
            };
        } else {
            bridge.active_camera_viewport = None;
            bridge.pointer_owner = EditorPointerOwner::None;
        }
    }

    if input.left_mouse_down()
        && let Some(tool_surface_id) = host.app.surface_sessions().active_viewport_drag_surface()
        && matches!(
            bridge.pointer_owner,
            EditorPointerOwner::ViewportTool {
                tool_surface_id: owner_surface
            } if owner_surface == tool_surface_id
        )
        && position != previous
        && viewport_capture_active_for_surface(
            &host.shell_state,
            &tool_surface_bindings,
            tool_surface_id,
        )
    {
        let amount = position.x - previous.x;
        if amount != 0.0
            && let Err(error) = host.app.dispatch_viewport_interaction_for_surface(
                tool_surface_id,
                ViewportInteractionCommand::PointerDragAxis { amount },
            )
        {
            eprintln!("viewport axis drag failed: {error}");
        }
    }

    if input.middle_mouse_down()
        && position != previous
        && matches!(
            bridge.pointer_owner,
            EditorPointerOwner::ViewportCamera {
                button: EditorCameraPointerButton::Middle,
                ..
            }
        )
        && let Some(binding) = active_camera_viewport_binding(
            &tool_surface_bindings,
            bridge.active_camera_viewport,
            position,
        )
    {
        viewport_render_commands.push(ViewportRenderStateCommand::PanCamera {
            viewport_id: binding.viewport_id,
            delta: position - previous,
        });
    }

    if input.right_mouse_down()
        && position != previous
        && matches!(
            bridge.pointer_owner,
            EditorPointerOwner::ViewportCamera {
                button: EditorCameraPointerButton::Secondary,
                ..
            }
        )
        && let Some(binding) = active_camera_viewport_binding(
            &tool_surface_bindings,
            bridge.active_camera_viewport,
            position,
        )
    {
        viewport_render_commands.push(ViewportRenderStateCommand::OrbitCamera {
            viewport_id: binding.viewport_id,
            delta: position - previous,
        });
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
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
        let captured_surface = host.app.surface_sessions().active_viewport_drag_surface();
        let routed_release_surface = outcome
            .as_ref()
            .and_then(|value| {
                viewport_pointer_route(
                    &host.shell_state,
                    &tool_surface_bindings,
                    &value.dispatch,
                    position,
                )
            })
            .map(|route| route.tool_surface_id);

        if let Some(tool_surface_id) = captured_surface
            && routed_release_surface
                .map(|release_surface| release_surface == tool_surface_id)
                .unwrap_or_else(|| {
                    tool_surface_bindings
                        .binding_for_tool_surface(tool_surface_id)
                        .is_some()
                })
            && let Err(error) = host.app.dispatch_viewport_interaction_for_surface(
                tool_surface_id,
                ViewportInteractionCommand::PointerUp,
            )
        {
            eprintln!("viewport pointer-up failed: {error}");
        }
        bridge.pointer_owner = EditorPointerOwner::None;
    }

    if input.middle_mouse_released() {
        bridge.active_camera_viewport = None;
        bridge.pointer_owner = EditorPointerOwner::None;
        let _ = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Up,
            position,
            UiVector::ZERO,
            Some(PointerButton::Middle),
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
    }

    if input.right_mouse_released() {
        bridge.active_camera_viewport = None;
        bridge.pointer_owner = EditorPointerOwner::None;
        let _ = dispatch_pointer_event(
            &mut host,
            &shell_theme,
            bounds,
            PointerEventKind::Up,
            position,
            UiVector::ZERO,
            Some(PointerButton::Secondary),
            modifiers,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
    }

    dispatch_shell_keyboard_and_text(
        &input,
        &mut host,
        &shell_theme,
        bounds,
        modifiers,
        viewport_products,
        Some(&mut *viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        Some(&viewport_instances),
        Some(&mut *viewport_render_commands),
    );

    let viewport_shortcuts_blocked = shell_focus_captures_viewport_shortcuts(&host.shell_state);
    handle_viewport_tool_radial_shortcut(
        &input,
        &mut host,
        &shell_theme,
        bounds,
        modifiers,
        viewport_products,
        Some(&mut *viewport_presentations),
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        Some(&viewport_instances),
        position,
        viewport_shortcuts_blocked,
    );

    if !viewport_shortcuts_blocked {
        dispatch_viewport_shortcuts(
            &input,
            &mut host,
            &mut viewport_presentations,
            &viewport_observations,
            &tool_surface_bindings,
            &mut viewport_render_commands,
            position,
            preferred_viewport_id,
        );
    }

    if !input.action_down(ACTION_EDITOR_VIEWPORT_TOOL_RADIAL) {
        host.app
            .surface_sessions_mut()
            .close_tab_hold_viewport_radial_menus();
    }
    if input.toggle_pause_menu {
        host.app
            .surface_sessions_mut()
            .close_all_viewport_tool_radial_menus();
    }

    let cursor_intent =
        RunenwerkEditorShellController::cursor_intent_for_pointer(&host.shell_state, position);
    window.set_cursor_icon(window_cursor_icon(cursor_intent));
    bridge.last_mouse_position = (position.x, position.y);
}

fn window_cursor_icon(cursor_intent: ShellCursorIntent) -> WindowCursorIcon {
    match cursor_intent {
        ShellCursorIntent::Default => WindowCursorIcon::Default,
        ShellCursorIntent::ResizeColumn => WindowCursorIcon::ColResize,
        ShellCursorIntent::ResizeRow => WindowCursorIcon::RowResize,
        ShellCursorIntent::ResizeNwse => WindowCursorIcon::NwseResize,
        ShellCursorIntent::ResizeNesw => WindowCursorIcon::NeswResize,
        ShellCursorIntent::Grab => WindowCursorIcon::Grab,
        ShellCursorIntent::Grabbing => WindowCursorIcon::Grabbing,
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_global_shortcuts(
    input: &engine::plugins::InputState,
    host: &mut EditorHostResource,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
) {
    if input.action_pressed(ACTION_EDITOR_UNDO)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::Undo,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("undo shortcut failed: {error}");
    }

    if input.action_pressed(ACTION_EDITOR_REDO)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::Redo,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("redo shortcut failed: {error}");
    }

    if input.action_pressed(action::UI_SAVE_TEMPLATE)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::SaveScene,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("save shortcut failed: {error}");
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_viewport_shortcuts(
    input: &engine::plugins::InputState,
    host: &mut EditorHostResource,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_observations: &ViewportArtifactObservationResource,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    viewport_render_commands: &mut ViewportRenderStateCommandQueueResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
) {
    if (input.action_pressed(ACTION_EDITOR_TOOL_SELECT)
        || input.action_pressed(action::UI_EDITOR_RESTORE_ALL))
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::ActivateSelectTool,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("select-tool shortcut failed: {error}");
    }

    if (input.action_pressed(ACTION_EDITOR_TOOL_TRANSLATE)
        || input.action_pressed(action::UI_EDITOR_HIDE_SELECTED))
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::ActivateTranslateTool,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("translate-tool shortcut failed: {error}");
    }

    if input.action_pressed(ACTION_EDITOR_TOOL_ROTATE)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::ActivateRotateTool,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("rotate-tool shortcut failed: {error}");
    }

    if input.action_pressed(ACTION_EDITOR_TOOL_SCALE)
        && let Err(error) = dispatch_shell_command(
            &mut host.app,
            Some(&mut host.shell_state),
            ShellCommand::ActivateScaleTool,
            Some(&mut *viewport_presentations),
            Some(viewport_observations),
            Some(tool_surface_bindings),
            None,
        )
    {
        eprintln!("scale-tool shortcut failed: {error}");
    }

    if input.action_pressed(ACTION_EDITOR_VIEWPORT_FOCUS)
        && let Some(orbit_target) = selected_entity_origin(&host.app)
        && let Some(binding) =
            viewport_binding_for_focus(tool_surface_bindings, cursor, preferred_viewport_id)
    {
        viewport_render_commands.push(ViewportRenderStateCommand::FocusCameraOn {
            viewport_id: binding.viewport_id,
            orbit_target,
        });
    }
}

fn selected_entity_origin(app: &crate::editor_app::RunenwerkEditorApp) -> Option<[f32; 3]> {
    let selected = app.runtime().selected_entity()?;
    let ecs_entity = app.runtime().ids().resolve_entity(selected)?;
    let transform = app
        .runtime()
        .world()
        .get::<LocalTransform>(ecs_entity)
        .copied()?;
    Some([
        transform.translation.x,
        transform.translation.y,
        transform.translation.z,
    ])
}

#[allow(clippy::too_many_arguments)]
fn dispatch_pointer_event(
    host: &mut EditorHostResource,
    shell_theme: &ui_theme::ThemeTokens,
    bounds: UiRect,
    kind: PointerEventKind,
    position: UiPoint,
    delta: UiVector,
    button: Option<PointerButton>,
    modifiers: Modifiers,
    viewport_products: Option<&editor_viewport::ArtifactObservationFrame>,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
    viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
) -> Option<editor_shell::UiInputOutcome> {
    let event = UiInputEvent::Pointer(PointerEvent {
        kind,
        position,
        delta,
        button,
        modifiers,
        click_count: 1,
    });

    match host.app.dispatch_shell_input(
        &mut host.shell_state,
        bounds,
        shell_theme,
        &event,
        viewport_products,
        viewport_presentations,
        viewport_observations,
        tool_surface_bindings,
        viewport_instances,
        viewport_render_commands,
    ) {
        Ok(outcome) => Some(outcome),
        Err(error) => {
            eprintln!("editor shell input dispatch failed: {error}");
            None
        }
    }
}

fn pointer_event_consumed_by_ui(outcome: &Option<editor_shell::UiInputOutcome>) -> bool {
    outcome
        .as_ref()
        .is_some_and(|value| value.dispatch.response.propagation == EventPropagation::Stop)
}

fn shell_focus_captures_viewport_shortcuts(shell_state: &RunenwerkEditorShellState) -> bool {
    let Some(tree) = shell_state.last_tree() else {
        return false;
    };
    shell_state
        .runtime()
        .focused_widget_captures_viewport_shortcuts(tree)
}

#[allow(clippy::too_many_arguments)]
fn handle_viewport_tool_radial_shortcut(
    input: &engine::plugins::InputState,
    host: &mut EditorHostResource,
    shell_theme: &ui_theme::ThemeTokens,
    bounds: UiRect,
    modifiers: Modifiers,
    viewport_products: Option<&editor_viewport::ArtifactObservationFrame>,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
    cursor: UiPoint,
    viewport_shortcuts_blocked: bool,
) {
    if !input.action_pressed(ACTION_EDITOR_VIEWPORT_TOOL_RADIAL) {
        return;
    }

    let Some(tool_surface_bindings) = tool_surface_bindings else {
        dispatch_shell_key_event(
            host,
            shell_theme,
            bounds,
            modifiers,
            ui_input::Key::Tab,
            viewport_products,
            viewport_presentations,
            viewport_observations,
            None,
            viewport_instances,
        );
        return;
    };

    if viewport_shortcuts_blocked {
        dispatch_shell_key_event(
            host,
            shell_theme,
            bounds,
            modifiers,
            ui_input::Key::Tab,
            viewport_products,
            viewport_presentations,
            viewport_observations,
            Some(tool_surface_bindings),
            viewport_instances,
        );
        return;
    }

    let Some(binding) = fallback_viewport_binding(tool_surface_bindings, cursor) else {
        dispatch_shell_key_event(
            host,
            shell_theme,
            bounds,
            modifiers,
            ui_input::Key::Tab,
            viewport_products,
            viewport_presentations,
            viewport_observations,
            Some(tool_surface_bindings),
            viewport_instances,
        );
        return;
    };

    let target = editor_shell::StructuralCommandTarget {
        panel_instance_id: binding.panel_instance_id,
        active_tool_surface: Some(binding.tool_surface_id),
        tab_stack_id: binding.tab_stack_id,
    };
    let projection_epoch = host.shell_state.current_projection_epoch();
    if let Err(error) = dispatch_shell_command(
        &mut host.app,
        Some(&mut host.shell_state),
        ShellCommand::ApplySurfaceSessionMutation {
            target,
            mutation: editor_shell::SurfaceSessionMutation::Viewport(
                editor_shell::ViewportSessionMutation::OpenToolRadialMenu {
                    viewport_id: binding.viewport_id,
                    anchor_position: cursor,
                    opened_by_tab_hold: true,
                },
            ),
            projection_epoch,
        },
        None,
        viewport_observations,
        Some(tool_surface_bindings),
        Some(projection_epoch),
    ) {
        eprintln!("viewport radial shortcut failed: {error}");
    }
}

#[allow(clippy::too_many_arguments)]
fn dispatch_shell_key_event(
    host: &mut EditorHostResource,
    shell_theme: &ui_theme::ThemeTokens,
    bounds: UiRect,
    modifiers: Modifiers,
    key: ui_input::Key,
    viewport_products: Option<&editor_viewport::ArtifactObservationFrame>,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
) {
    let event = UiInputEvent::Keyboard(ui_input::KeyboardEvent {
        key,
        state: ui_input::KeyState::Pressed,
        modifiers,
    });
    let _ = host.app.dispatch_shell_input(
        &mut host.shell_state,
        bounds,
        shell_theme,
        &event,
        viewport_products,
        viewport_presentations,
        viewport_observations,
        tool_surface_bindings,
        viewport_instances,
        None,
    );
}

#[allow(clippy::too_many_arguments)]
fn dispatch_shell_keyboard_and_text(
    input: &engine::plugins::InputState,
    host: &mut EditorHostResource,
    shell_theme: &ui_theme::ThemeTokens,
    bounds: UiRect,
    modifiers: Modifiers,
    viewport_products: Option<&editor_viewport::ArtifactObservationFrame>,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
    viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
) {
    let mut viewport_presentations = viewport_presentations;
    let mut viewport_render_commands = viewport_render_commands;
    let mut send_key =
        |key: ui_input::Key,
         host: &mut EditorHostResource,
         viewport_presentations: Option<&mut ViewportPresentationStateResource>| {
            let event = UiInputEvent::Keyboard(ui_input::KeyboardEvent {
                key,
                state: ui_input::KeyState::Pressed,
                modifiers,
            });
            let _ = host.app.dispatch_shell_input(
                &mut host.shell_state,
                bounds,
                shell_theme,
                &event,
                viewport_products,
                viewport_presentations,
                viewport_observations,
                tool_surface_bindings,
                viewport_instances,
                viewport_render_commands.as_deref_mut(),
            );
        };

    if input.backspace {
        send_key(
            ui_input::Key::Backspace,
            host,
            viewport_presentations.as_deref_mut(),
        );
    }
    if input.delete {
        send_key(
            ui_input::Key::Delete,
            host,
            viewport_presentations.as_deref_mut(),
        );
    }
    if input.submitted {
        send_key(
            ui_input::Key::Enter,
            host,
            viewport_presentations.as_deref_mut(),
        );
    }
    if input.toggle_pause_menu {
        send_key(
            ui_input::Key::Escape,
            host,
            viewport_presentations.as_deref_mut(),
        );
    }

    if !input.typed_text.is_empty() {
        let event = UiInputEvent::Text(ui_input::TextInputEvent {
            text: input.typed_text.clone(),
        });
        let _ = host.app.dispatch_shell_input(
            &mut host.shell_state,
            bounds,
            shell_theme,
            &event,
            viewport_products,
            viewport_presentations,
            viewport_observations,
            tool_surface_bindings,
            viewport_instances,
            viewport_render_commands,
        );
    }
}

fn viewport_pointer_route(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    dispatch: &editor_shell::UiInputDispatchResult,
    position: UiPoint,
) -> Option<ViewportPointerRoute> {
    if let Some(host_widget_id) = dispatch.target {
        let binding =
            viewport_scene_binding_for_widget(shell_state, tool_surface_bindings, host_widget_id)?;
        if !binding.bounds.contains(position) {
            return None;
        }

        let structural_context = structural_context_for_widget(shell_state, host_widget_id)?;
        return Some(ViewportPointerRoute {
            tool_surface_id: binding.tool_surface_id,
            viewport_id: binding.viewport_id,
            host_widget_id,
            structural_context,
            local_position: UiPoint::new(
                position.x - binding.bounds.x,
                position.y - binding.bounds.y,
            ),
        });
    }

    let binding = fallback_viewport_binding(tool_surface_bindings, position)?;
    let host_widget_id = binding.host_widget_id;
    let structural_context = structural_context_for_widget(shell_state, host_widget_id).unwrap_or(
        editor_shell::StructuralWidgetRoutingContext {
            panel_instance_id: binding.panel_instance_id,
            active_tool_surface: Some(binding.tool_surface_id),
            tab_stack_id: binding.tab_stack_id,
        },
    );
    Some(ViewportPointerRoute {
        tool_surface_id: binding.tool_surface_id,
        viewport_id: binding.viewport_id,
        host_widget_id,
        structural_context,
        local_position: UiPoint::new(position.x - binding.bounds.x, position.y - binding.bounds.y),
    })
}

fn viewport_capture_active_for_surface(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    tool_surface_id: editor_shell::ToolSurfaceInstanceId,
) -> bool {
    if let Some(captured_widget) = shell_state.runtime().state().captured_widget {
        return viewport_scene_binding_for_widget(
            shell_state,
            tool_surface_bindings,
            captured_widget,
        )
        .map(|binding| binding.tool_surface_id == tool_surface_id)
        .unwrap_or(false);
    }

    tool_surface_bindings
        .binding_for_tool_surface(tool_surface_id)
        .is_some()
}

fn fallback_viewport_binding(
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    tool_surface_bindings.binding_containing_cursor(cursor)
}

fn active_camera_viewport_binding(
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    active_viewport_id: Option<ViewportId>,
    _cursor: UiPoint,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    active_viewport_id
        .and_then(|viewport_id| viewport_binding_by_id(tool_surface_bindings, viewport_id))
}

fn viewport_binding_for_focus(
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    cursor: UiPoint,
    preferred_viewport_id: Option<ViewportId>,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    fallback_viewport_binding(tool_surface_bindings, cursor)
        .or_else(|| {
            preferred_viewport_id
                .and_then(|viewport_id| viewport_binding_by_id(tool_surface_bindings, viewport_id))
        })
        .or_else(|| tool_surface_bindings.bindings().next())
}

fn viewport_binding_by_id(
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    viewport_id: ViewportId,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    tool_surface_bindings
        .bindings()
        .find(|binding| binding.viewport_id == viewport_id)
}

fn viewport_scene_binding_for_widget(
    shell_state: &RunenwerkEditorShellState,
    tool_surface_bindings: &ToolSurfaceRuntimeBindingRegistryResource,
    widget_id: editor_shell::WidgetId,
) -> Option<crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord> {
    let context = structural_context_for_widget(shell_state, widget_id)?;
    let binding = tool_surface_bindings.resolve_structural_context(context)?;
    (binding.host_widget_id == widget_id).then_some(binding)
}

fn structural_context_for_widget(
    shell_state: &RunenwerkEditorShellState,
    widget_id: editor_shell::WidgetId,
) -> Option<editor_shell::StructuralWidgetRoutingContext> {
    shell_state
        .last_projection_artifacts()
        .and_then(|artifacts| artifacts.widget_structural_context_by_id.get(&widget_id))
        .copied()
}

fn window_bounds(window: &WindowState) -> UiRect {
    let width = window.size_px.0.max(1) as f32;
    let height = window.size_px.1.max(1) as f32;
    UiRect::new(0.0, 0.0, width, height)
}

fn dispatch_viewport_pointer_down(
    host: &mut EditorHostResource,
    picking_results: &ViewportPickingResultsResource,
    position: UiPoint,
    route: ViewportPointerRoute,
) {
    let expression = build_viewport_picking_product_frame(
        route.viewport_id,
        picking_results,
        host.app.runtime().current_scene_reality_version(),
    );
    let hit = viewport_hit_from_picking_product(&expression);
    let picking = picking_results.result_for(route.viewport_id);
    let selection_before = host.app.runtime().selected_entity();

    if host.app.debug_logs_enabled() {
        host.app.append_console_input(format!(
            "[input] viewport pointer-down viewport={} tool_surface={} widget={} panel={} tab_stack={} structural_tool_surface={:?} cursor=({:.1},{:.1}) local=({:.1},{:.1}) hit={} dist={:.3} expr_frame={} sel_before={:?}",
            route.viewport_id.0,
            route.tool_surface_id.raw(),
            route.host_widget_id.0,
            route.structural_context.panel_instance_id.raw(),
            route.structural_context.tab_stack_id.raw(),
            route.structural_context.active_tool_surface.map(|value| value.raw()),
            position.x,
            position.y,
            route.local_position.x,
            route.local_position.y,
            picking
                .map(|value| picking_target_label(value.hit.target))
                .unwrap_or_else(|| "none".to_string()),
            picking.map(|value| value.hit.distance).unwrap_or(f32::INFINITY),
            expression.expression.metadata.frame_id.0,
            selection_before
        ));
    }

    let result = host.app.dispatch_viewport_interaction_for_surface(
        route.tool_surface_id,
        ViewportInteractionCommand::PointerDown { hit },
    );
    if let Err(error) = result {
        eprintln!("viewport pointer-down failed: {error}");
        return;
    }

    if host.app.debug_logs_enabled() {
        host.app.append_console_input(format!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_app::RunenwerkEditorApp;
    use crate::runtime::viewport::{ViewportLayoutEntry, ViewportLayoutMapResource};
    use crate::shell::RunenwerkEditorShellController;
    use editor_viewport::ViewportId;
    use engine::plugins::render::UiFontAtlasResource;
    use ui_input::InputResponse;
    use ui_theme::ThemeTokens;

    fn seeded_shell_state_with_projection() -> RunenwerkEditorShellState {
        let app = RunenwerkEditorApp::new();
        let mut shell_state = RunenwerkEditorShellState::new();
        let atlas = UiFontAtlasResource::default();
        let _ = RunenwerkEditorShellController::build_frame(
            &app,
            &mut shell_state,
            UiRect::new(0.0, 0.0, 1280.0, 720.0),
            &ThemeTokens::default(),
            &atlas,
        );
        shell_state
    }

    fn seeded_bindings(
        shell_state: &RunenwerkEditorShellState,
        viewport_id: ViewportId,
        bounds: UiRect,
    ) -> ToolSurfaceRuntimeBindingRegistryResource {
        let viewport_embed_widget_id = viewport_embed_widget_id(shell_state);
        let structural_context = shell_state
            .last_projection_artifacts()
            .and_then(|artifacts| {
                artifacts
                    .widget_structural_context_by_id
                    .get(&viewport_embed_widget_id)
                    .copied()
            })
            .expect("viewport embed structural context should exist");
        let mut layout_map = ViewportLayoutMapResource::default();
        layout_map.upsert_entry(ViewportLayoutEntry {
            viewport_id,
            host_widget_id: viewport_embed_widget_id,
            structural_context,
            bounds,
        });
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.rebuild_from_layout_map(&layout_map);
        bindings
    }

    fn viewport_surface_id(
        shell_state: &RunenwerkEditorShellState,
    ) -> editor_shell::ToolSurfaceInstanceId {
        shell_state
            .workspace_state()
            .panels()
            .filter_map(|panel| panel.active_tool_surface)
            .find(|surface_id| {
                shell_state
                    .workspace_state()
                    .tool_surface(*surface_id)
                    .map(|surface| {
                        surface.tool_surface_kind == editor_shell::ToolSurfaceKind::Viewport
                    })
                    .unwrap_or(false)
            })
            .expect("seeded shell state should contain an active viewport surface")
    }

    fn viewport_embed_widget_id(shell_state: &RunenwerkEditorShellState) -> editor_shell::WidgetId {
        editor_shell::surface_widget_id(
            viewport_surface_id(shell_state),
            editor_shell::VIEWPORT_SURFACE_EMBED_WIDGET_ID,
        )
    }

    fn viewport_chrome_widget_id(
        shell_state: &RunenwerkEditorShellState,
        local_id: editor_shell::WidgetId,
    ) -> editor_shell::WidgetId {
        editor_shell::surface_widget_id(viewport_surface_id(shell_state), local_id)
    }

    fn manual_binding(
        raw_id: u64,
        viewport_id: ViewportId,
        bounds: UiRect,
    ) -> crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord {
        crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord {
            tool_surface_id: editor_shell::ToolSurfaceInstanceId::try_from_raw(raw_id).unwrap(),
            panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(raw_id).unwrap(),
            tab_stack_id: editor_shell::TabStackId::try_from_raw(raw_id).unwrap(),
            viewport_id,
            host_widget_id: editor_shell::WidgetId(10_000 + raw_id),
            bounds,
            generation: 1,
        }
    }

    fn dual_viewport_bindings() -> ToolSurfaceRuntimeBindingRegistryResource {
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding(
            1,
            ViewportId(5),
            UiRect::new(0.0, 0.0, 200.0, 120.0),
        ));
        bindings.upsert_binding(manual_binding(
            2,
            ViewportId(8),
            UiRect::new(220.0, 0.0, 200.0, 120.0),
        ));
        bindings
    }

    #[test]
    fn viewport_pointer_route_uses_canonical_fallback_when_dispatch_target_is_missing() {
        let shell_state = seeded_shell_state_with_projection();
        let viewport_bounds = UiRect::new(100.0, 80.0, 900.0, 560.0);
        let bindings = seeded_bindings(&shell_state, ViewportId(5), viewport_bounds);
        let dispatch = editor_shell::UiInputDispatchResult {
            target: None,
            response: InputResponse::ignored(),
        };

        let route = viewport_pointer_route(
            &shell_state,
            &bindings,
            &dispatch,
            UiPoint::new(220.0, 300.0),
        )
        .expect("fallback routing should resolve viewport route");

        assert_eq!(route.viewport_id, ViewportId(5));
        assert_eq!(route.host_widget_id, viewport_embed_widget_id(&shell_state));
        assert_eq!(
            Some(route.tool_surface_id),
            route.structural_context.active_tool_surface
        );
        assert!((route.local_position.x - 120.0).abs() <= 0.001);
        assert!((route.local_position.y - 220.0).abs() <= 0.001);
    }

    #[test]
    fn viewport_pointer_route_does_not_fallback_outside_viewport_bounds() {
        let shell_state = seeded_shell_state_with_projection();
        let viewport_bounds = UiRect::new(100.0, 80.0, 900.0, 560.0);
        let bindings = seeded_bindings(&shell_state, ViewportId(5), viewport_bounds);
        let dispatch = editor_shell::UiInputDispatchResult {
            target: None,
            response: InputResponse::ignored(),
        };

        let route =
            viewport_pointer_route(&shell_state, &bindings, &dispatch, UiPoint::new(20.0, 20.0));

        assert!(
            route.is_none(),
            "outside viewport clicks must not route to viewport fallback",
        );
    }

    #[test]
    fn viewport_pointer_route_rejects_viewport_chrome_dispatch_target() {
        let shell_state = seeded_shell_state_with_projection();
        let viewport_bounds = UiRect::new(100.0, 80.0, 900.0, 560.0);
        let bindings = seeded_bindings(&shell_state, ViewportId(5), viewport_bounds);
        let dispatch = editor_shell::UiInputDispatchResult {
            target: Some(viewport_chrome_widget_id(
                &shell_state,
                editor_shell::VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
            )),
            response: InputResponse::handled(),
        };

        let route = viewport_pointer_route(
            &shell_state,
            &bindings,
            &dispatch,
            UiPoint::new(220.0, 300.0),
        );

        assert!(
            route.is_none(),
            "viewport chrome must not fall back into scene interaction routing",
        );
    }

    #[test]
    fn viewport_capture_validation_is_tool_surface_scoped() {
        let mut shell_state = seeded_shell_state_with_projection();
        let viewport_bounds = UiRect::new(100.0, 80.0, 900.0, 560.0);
        let bindings = seeded_bindings(&shell_state, ViewportId(5), viewport_bounds);
        let route = viewport_pointer_route(
            &shell_state,
            &bindings,
            &editor_shell::UiInputDispatchResult {
                target: None,
                response: InputResponse::ignored(),
            },
            UiPoint::new(220.0, 300.0),
        )
        .expect("viewport route should resolve");

        assert!(viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            route.tool_surface_id
        ));
        assert!(!viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            editor_shell::ToolSurfaceInstanceId::try_from_raw(999).unwrap()
        ));

        let viewport_chrome_widget = viewport_chrome_widget_id(
            &shell_state,
            editor_shell::VIEWPORT_DETAILS_TOGGLE_WIDGET_ID,
        );
        shell_state.runtime_mut().state_mut().captured_widget = Some(viewport_chrome_widget);
        assert!(!viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            route.tool_surface_id
        ));

        let viewport_embed_widget = viewport_embed_widget_id(&shell_state);
        shell_state.runtime_mut().state_mut().captured_widget = Some(viewport_embed_widget);
        assert!(viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            route.tool_surface_id
        ));
    }

    #[test]
    fn camera_drag_binding_keeps_captured_viewport_after_cursor_leaves_bounds() {
        let bindings = dual_viewport_bindings();

        let binding = active_camera_viewport_binding(
            &bindings,
            Some(ViewportId(8)),
            UiPoint::new(900.0, 900.0),
        )
        .expect("captured camera viewport should resolve outside its bounds");

        assert_eq!(binding.viewport_id, ViewportId(8));
    }

    #[test]
    fn focus_binding_uses_last_target_viewport_when_cursor_is_not_hovering() {
        let bindings = dual_viewport_bindings();

        let binding =
            viewport_binding_for_focus(&bindings, UiPoint::new(900.0, 900.0), Some(ViewportId(8)))
                .expect("last target viewport should resolve");

        assert_eq!(binding.viewport_id, ViewportId(8));
    }
}
