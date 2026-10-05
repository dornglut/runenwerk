use editor_shell::ShellCommand;
use editor_viewport::ViewportId;
use engine::plugins::input::domain::action;
use engine::plugins::render::{EditorGizmoAxis, EditorPickingTarget};
use engine::runtime::platform::{PlatformEvent, PlatformWindowEventQueueResource};
use engine::runtime::{NativeWindowId, Res, ResMut, WindowStateRegistryResource};
use engine::{PrimaryPresentationMetricsResource, WindowCursorIcon};
use scene::LocalTransform;
use ui_input::{
    EventPropagation, PointerButton, PointerEventKind, PointerSourceKind, UiInputEvent,
};
use ui_math::{UiPoint, UiRect};

use super::picking::{clear_editor_picking_for_target, update_editor_picking_for_target};
use crate::editor_features::viewport::ViewportInteractionCommand;
use crate::runtime::app::{
    ACTION_EDITOR_REDO, ACTION_EDITOR_TOOL_ROTATE, ACTION_EDITOR_TOOL_SCALE,
    ACTION_EDITOR_TOOL_SELECT, ACTION_EDITOR_TOOL_TRANSLATE, ACTION_EDITOR_UNDO,
    ACTION_EDITOR_VIEWPORT_FOCUS, ACTION_EDITOR_VIEWPORT_TOOL_RADIAL,
};
use crate::runtime::composition::{EditorTargetInputRuntimeResource, translate_platform_event};
use crate::runtime::resources::{
    EditorCameraPointerButton, EditorHostResource, EditorInputBridgeState, EditorPointerOwner,
    scaled_shell_theme,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportRenderStateCommand,
    ViewportRenderStateCommandQueueResource, ViewportRenderStateResource,
    fallback_viewport_binding, resolve_structural_viewport_products, structural_context_for_widget,
    viewport_binding_by_id_for_target, viewport_scene_binding_for_widget,
};
use crate::runtime::{build_viewport_picking_product_frame, viewport_hit_from_picking_product};
use crate::shell::dispatch_shell_command;
use crate::shell::{
    KnownEditorCommand, RunenwerkEditorShellController, RunenwerkEditorShellState,
    ShellCursorIntent, resolve_active_editor_shortcuts,
};

mod shortcuts;
mod viewport_pointer;

#[cfg(test)]
use shortcuts::{dispatch_active_editor_shortcuts, viewport_binding_for_focus};
use shortcuts::{
    dispatch_global_shortcuts, dispatch_viewport_shortcuts, handle_viewport_tool_radial_shortcut,
    sync_active_editor_shortcut_bindings,
};
use viewport_pointer::presentation_bounds;
#[cfg(test)]
use viewport_pointer::{
    active_camera_viewport_binding, viewport_capture_active_for_surface, viewport_pointer_route,
};
pub(crate) use viewport_pointer::{
    clear_editor_viewport_interaction_for_target, dispatch_editor_viewport_input_for_target,
};

#[allow(clippy::too_many_arguments)]
pub fn dispatch_editor_input_system(
    input: Res<engine::plugins::InputState>,
    mut actions: ResMut<engine::plugins::ActionState>,
    mut target_input: ResMut<EditorTargetInputRuntimeResource>,
    mut platform_events: ResMut<PlatformWindowEventQueueResource>,
    presentation: Res<PrimaryPresentationMetricsResource>,
    mut windows: ResMut<WindowStateRegistryResource>,
    mut host: ResMut<EditorHostResource>,
    mut bridge: ResMut<EditorInputBridgeState>,
    mut picking_results: ResMut<ViewportPickingResultsResource>,
    viewport_render_states: Res<ViewportRenderStateResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    mut viewport_render_commands: ResMut<ViewportRenderStateCommandQueueResource>,
) {
    sync_active_editor_shortcut_bindings(&input, &mut actions, &host, &mut bridge);

    let primary_window_id = NativeWindowId::primary();
    let primary_target_id = host.shell_state.primary_composition_target_id();
    let mut primary_ui_events = Vec::new();
    let mut retained_events = Vec::new();
    for window_event in platform_events.drain() {
        if window_event.native_window_id != primary_window_id {
            retained_events.push(window_event);
            continue;
        }
        let focus_lost = matches!(
            &window_event.event,
            PlatformEvent::Focused { focused: false }
        );
        primary_ui_events.extend(translate_platform_event(
            &mut target_input,
            primary_window_id,
            window_event.event,
        ));
        if focus_lost {
            host.shell_state.runtime_mut().set_focused_widget(None);
            host.shell_state.clear_tab_drag();
            clear_editor_viewport_interaction_for_target(
                primary_target_id,
                &mut bridge,
                &mut picking_results,
                &tool_surface_bindings,
            );
        }
    }
    for event in retained_events {
        platform_events.publish(event);
    }

    if !primary_window_is_focused(&windows, primary_window_id) {
        target_input.clear_window(primary_window_id);
        host.shell_state.runtime_mut().set_focused_widget(None);
        host.shell_state.clear_tab_drag();
        clear_editor_viewport_interaction_for_target(
            primary_target_id,
            &mut bridge,
            &mut picking_results,
            &tool_surface_bindings,
        );
        return;
    }

    let bounds = presentation_bounds(&presentation);
    let shell_theme = scaled_shell_theme(&host.theme, presentation.scale_factor());
    let viewport_products = resolve_structural_viewport_products(
        &host.shell_state,
        &viewport_observations,
        &tool_surface_bindings,
    );
    let position = UiPoint::new(input.mouse_position.0, input.mouse_position.1);
    if let Some(binding) =
        tool_surface_bindings.binding_containing_cursor_for_target(primary_target_id, position)
    {
        bridge
            .interaction_for_target_mut(primary_target_id)
            .last_target_viewport = Some(binding.viewport_id);
    }
    let preferred_viewport_id = bridge
        .interaction_for_target(primary_target_id)
        .last_target_viewport;
    let authored_viewport_shortcuts_blocked =
        shell_focus_captures_viewport_shortcuts(&host.shell_state);

    dispatch_global_shortcuts(
        &actions,
        &mut host,
        &bridge,
        &mut viewport_presentations,
        &viewport_observations,
        &tool_surface_bindings,
        position,
        preferred_viewport_id,
        authored_viewport_shortcuts_blocked,
    );

    if picking_results.global_revision() != bridge.last_logged_picking_revision {
        bridge.last_logged_picking_revision = picking_results.global_revision();
    }

    for event in primary_ui_events {
        let outcome = dispatch_ui_event(
            &mut host,
            &shell_theme,
            bounds,
            &event,
            viewport_products,
            Some(&mut *viewport_presentations),
            Some(&viewport_observations),
            Some(&tool_surface_bindings),
            Some(&viewport_instances),
            Some(&mut *viewport_render_commands),
        );
        let ui_consumed = pointer_event_consumed_by_ui(&outcome);
        dispatch_editor_viewport_input_for_target(
            primary_target_id,
            &event,
            outcome.as_ref().map(|value| &value.dispatch),
            ui_consumed,
            &mut host,
            &mut bridge,
            &mut picking_results,
            &viewport_render_states,
            &tool_surface_bindings,
            &mut viewport_render_commands,
        );
    }

    let viewport_shortcuts_blocked = shell_focus_captures_viewport_shortcuts(&host.shell_state);
    handle_viewport_tool_radial_shortcut(
        &actions,
        &mut host,
        Some(&viewport_observations),
        Some(&tool_surface_bindings),
        position,
        viewport_shortcuts_blocked,
    );

    if !viewport_shortcuts_blocked {
        dispatch_viewport_shortcuts(
            &actions,
            &mut host,
            &bridge,
            &viewport_observations,
            &tool_surface_bindings,
            &mut viewport_render_commands,
            position,
            preferred_viewport_id,
        );
    }

    if !actions.action_down(ACTION_EDITOR_VIEWPORT_TOOL_RADIAL) {
        host.app
            .surface_sessions_mut()
            .close_tab_hold_viewport_radial_menus();
    }
    if actions.action_pressed(action::SYSTEM_TOGGLE_PAUSE_MENU) {
        host.app
            .surface_sessions_mut()
            .close_all_viewport_tool_radial_menus();
    }

    let cursor_intent =
        RunenwerkEditorShellController::cursor_intent_for_pointer(&host.shell_state, position);
    set_primary_cursor_intent(&mut windows, primary_window_id, cursor_intent);
}

fn primary_window_is_focused(
    windows: &WindowStateRegistryResource,
    primary_window_id: NativeWindowId,
) -> bool {
    windows
        .record(primary_window_id)
        .is_some_and(|record| record.focused)
}

fn set_primary_cursor_intent(
    windows: &mut WindowStateRegistryResource,
    primary_window_id: NativeWindowId,
    cursor_intent: ShellCursorIntent,
) {
    if let Some(primary_window) = windows.record_mut(primary_window_id) {
        primary_window.set_cursor_icon(window_cursor_icon(cursor_intent));
    }
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
fn dispatch_ui_event(
    host: &mut EditorHostResource,
    shell_theme: &ui_theme::ThemeTokens,
    bounds: UiRect,
    event: &UiInputEvent,
    viewport_products: Option<&editor_viewport::ArtifactObservationFrame>,
    viewport_presentations: Option<&mut ViewportPresentationStateResource>,
    viewport_observations: Option<&ViewportArtifactObservationResource>,
    tool_surface_bindings: Option<&ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Option<&ViewportInstanceRegistryResource>,
    viewport_render_commands: Option<&mut ViewportRenderStateCommandQueueResource>,
) -> Option<editor_shell::UiInputOutcome> {
    match host.app.dispatch_shell_input(
        &mut host.shell_state,
        bounds,
        shell_theme,
        event,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_app::RunenwerkEditorApp;
    use crate::runtime::viewport::{ViewportLayoutEntry, ViewportLayoutMapResource};
    use crate::shell::{RunenwerkEditorShellController, validate_editor_shortcuts};
    use editor_definition::{EditorShortcutDefinition, EditorShortcutSetDefinition};
    use editor_viewport::ViewportId;
    use engine::plugins::render::UiFontAtlasResource;
    use engine::plugins::{ActionState, InputState};
    use ui_input::{InputResponse, Modifiers, PointerEvent, PointerPacket, PointerToolKind};
    use ui_math::UiVector;
    use ui_theme::ThemeTokens;
    use winit::event::ElementState;
    use winit::keyboard::KeyCode;

    #[test]
    fn primary_focus_state_is_read_from_native_window_registry() {
        let mut windows = WindowStateRegistryResource::default();
        let primary = windows.register_primary_window("Runenwerk", (1280, 720), 1.0, true);
        assert!(primary_window_is_focused(&windows, primary));

        windows.record_mut(primary).unwrap().focused = false;
        assert!(!primary_window_is_focused(&windows, primary));
    }

    #[test]
    fn editor_cursor_intent_updates_primary_native_record() {
        let mut windows = WindowStateRegistryResource::default();
        let primary = windows.register_primary_window("Runenwerk", (1280, 720), 1.0, true);

        set_primary_cursor_intent(&mut windows, primary, ShellCursorIntent::ResizeColumn);

        assert_eq!(
            windows.record(primary).unwrap().cursor_icon,
            WindowCursorIcon::ColResize
        );
        assert!(windows.record(primary).unwrap().redraw_requested);
    }

    #[test]
    fn active_shortcuts_dispatch_known_tool_and_definition_commands() {
        let mut host = EditorHostResource {
            shell_state: seeded_shell_state_with_projection(),
            ..Default::default()
        };
        host.shell_state
            .active_editor_definitions_mut()
            .install_shortcuts(
                EditorShortcutSetDefinition {
                    id: "runenwerk.editor.test.shortcuts.active".to_string(),
                    label: "Active Shortcuts".to_string(),
                    shortcuts: vec![
                        EditorShortcutDefinition {
                            id: "select".to_string(),
                            command: "editor.tool.select".to_string(),
                            chord: "Cmd+1".to_string(),
                            context: None,
                        },
                        EditorShortcutDefinition {
                            id: "apply_definition".to_string(),
                            command: "editor.definition.apply_selected".to_string(),
                            chord: "Cmd+Shift+A".to_string(),
                            context: None,
                        },
                    ],
                },
                validate_editor_shortcuts,
            )
            .expect("active shortcut set should validate");
        let mut input = InputState::default();
        let mut actions = ActionState::default();
        let mut bridge = EditorInputBridgeState::default();
        let mut viewport_presentations = ViewportPresentationStateResource::default();
        let viewport_observations = ViewportArtifactObservationResource::default();
        let tool_surface_bindings = seeded_bindings(
            &host.shell_state,
            ViewportId(5),
            UiRect::new(100.0, 80.0, 900.0, 560.0),
        );
        let mounted_unit_id = host
            .shell_state
            .mounted_unit_id_for_tool_surface(
                tool_surface_bindings
                    .bindings()
                    .next()
                    .expect("seeded viewport binding should exist")
                    .tool_surface_id,
            )
            .expect("seeded viewport should have a mounted unit");

        sync_active_editor_shortcut_bindings(&input, &mut actions, &host, &mut bridge);
        input.handle_keyboard_input(KeyCode::SuperLeft, ElementState::Pressed, None);
        input.handle_keyboard_input(KeyCode::Digit1, ElementState::Pressed, None);
        actions.project(&input);
        dispatch_active_editor_shortcuts(
            &actions,
            &mut host,
            &bridge,
            &mut viewport_presentations,
            &viewport_observations,
            &tool_surface_bindings,
            UiPoint::new(220.0, 300.0),
            Some(ViewportId(5)),
            false,
        );
        assert_eq!(
            host.app.surface_sessions().viewport_tool(mounted_unit_id),
            editor_shell::ViewportToolKind::Select
        );

        let mut input = InputState::default();
        let mut actions = ActionState::default();
        let mut bridge = EditorInputBridgeState::default();
        sync_active_editor_shortcut_bindings(&input, &mut actions, &host, &mut bridge);
        input.handle_keyboard_input(KeyCode::SuperLeft, ElementState::Pressed, None);
        input.handle_keyboard_input(KeyCode::ShiftLeft, ElementState::Pressed, None);
        input.handle_keyboard_input(KeyCode::KeyA, ElementState::Pressed, None);
        actions.project(&input);
        dispatch_active_editor_shortcuts(
            &actions,
            &mut host,
            &bridge,
            &mut viewport_presentations,
            &viewport_observations,
            &tool_surface_bindings,
            UiPoint::new(220.0, 300.0),
            Some(ViewportId(5)),
            false,
        );
        assert_eq!(
            host.app.pending_editor_definition_activation_count(),
            1,
            "apply-definition shortcut should dispatch through KnownEditorCommand",
        );
    }

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
            presentation_target_id: ui_composition::PresentationTargetId::try_from_raw(1).unwrap(),
            viewport_id,
            host_widget_id: viewport_embed_widget_id,
            structural_context,
            bounds,
            effective_shell_scale: 1.0,
        });
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.rebuild_from_layout_map(&layout_map);
        bindings
    }

    fn viewport_surface_id(
        shell_state: &RunenwerkEditorShellState,
    ) -> editor_shell::ToolSurfaceInstanceId {
        let extension = shell_state
            .composition_runtime()
            .extension()
            .mounted_units()
            .iter()
            .find(|unit| {
                unit.stable_content_key == crate::shell::tool_suites::SCENE_VIEWPORT_SURFACE_KEY
            })
            .expect("seeded shell should contain a mounted viewport");
        editor_shell::ToolSurfaceInstanceId::try_from_raw(extension.compatibility_surface_raw)
            .expect("mounted viewport compatibility surface identity should be valid")
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
        manual_binding_for_target(
            raw_id,
            ui_composition::PresentationTargetId::try_from_raw(1).unwrap(),
            viewport_id,
            bounds,
        )
    }

    fn manual_binding_for_target(
        raw_id: u64,
        target_id: ui_composition::PresentationTargetId,
        viewport_id: ViewportId,
        bounds: UiRect,
    ) -> crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord {
        crate::runtime::viewport::ToolSurfaceRuntimeBindingRecord {
            presentation_target_id: target_id,
            tool_surface_id: editor_shell::ToolSurfaceInstanceId::try_from_raw(raw_id).unwrap(),
            panel_instance_id: editor_shell::PanelInstanceId::try_from_raw(raw_id).unwrap(),
            tab_stack_id: editor_shell::TabStackId::try_from_raw(raw_id).unwrap(),
            viewport_id,
            host_widget_id: editor_shell::WidgetId(10_000 + raw_id),
            bounds,
            effective_shell_scale: 1.0,
            generation: 1,
        }
    }

    fn mouse_pointer_event(
        kind: PointerEventKind,
        position: UiPoint,
        delta: UiVector,
        button: Option<PointerButton>,
    ) -> UiInputEvent {
        UiInputEvent::Pointer(PointerEvent {
            kind,
            position,
            delta,
            button,
            modifiers: Modifiers::default(),
            click_count: 0,
            packet: PointerPacket {
                source_kind: PointerSourceKind::Mouse,
                tool_kind: PointerToolKind::Mouse,
                ..PointerPacket::default()
            },
        })
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
            shell_state.primary_composition_target_id(),
            Some(&dispatch),
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

        let route = viewport_pointer_route(
            &shell_state,
            &bindings,
            shell_state.primary_composition_target_id(),
            Some(&dispatch),
            UiPoint::new(20.0, 20.0),
        );

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
            shell_state.primary_composition_target_id(),
            Some(&dispatch),
            UiPoint::new(220.0, 300.0),
        );

        assert!(
            route.is_none(),
            "viewport chrome must not fall back into scene interaction routing",
        );
    }

    #[test]
    fn viewport_pointer_route_rejects_viewport_status_dispatch_target() {
        let shell_state = seeded_shell_state_with_projection();
        let viewport_bounds = UiRect::new(100.0, 80.0, 900.0, 560.0);
        let bindings = seeded_bindings(&shell_state, ViewportId(5), viewport_bounds);
        let dispatch = editor_shell::UiInputDispatchResult {
            target: Some(viewport_chrome_widget_id(
                &shell_state,
                editor_shell::VIEWPORT_STATUS_WIDGET_ID,
            )),
            response: InputResponse::handled(),
        };

        let route = viewport_pointer_route(
            &shell_state,
            &bindings,
            shell_state.primary_composition_target_id(),
            Some(&dispatch),
            UiPoint::new(220.0, 300.0),
        );

        assert!(
            route.is_none(),
            "viewport status must not fall back into scene interaction routing",
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
            shell_state.primary_composition_target_id(),
            Some(&editor_shell::UiInputDispatchResult {
                target: None,
                response: InputResponse::ignored(),
            }),
            UiPoint::new(220.0, 300.0),
        )
        .expect("viewport route should resolve");

        assert!(viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            shell_state.primary_composition_target_id(),
            route.tool_surface_id
        ));
        assert!(!viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            shell_state.primary_composition_target_id(),
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
            shell_state.primary_composition_target_id(),
            route.tool_surface_id
        ));

        let viewport_embed_widget = viewport_embed_widget_id(&shell_state);
        shell_state.runtime_mut().state_mut().captured_widget = Some(viewport_embed_widget);
        assert!(viewport_capture_active_for_surface(
            &shell_state,
            &bindings,
            shell_state.primary_composition_target_id(),
            route.tool_surface_id
        ));
    }

    #[test]
    fn camera_drag_binding_keeps_captured_viewport_after_cursor_leaves_bounds() {
        let bindings = dual_viewport_bindings();

        let binding = active_camera_viewport_binding(
            &bindings,
            ui_composition::PresentationTargetId::try_from_raw(1).unwrap(),
            Some(ViewportId(8)),
        )
        .expect("captured camera viewport should resolve outside its bounds");

        assert_eq!(binding.viewport_id, ViewportId(8));
    }

    #[test]
    fn viewport_pointer_route_scopes_overlapping_coordinates_by_target() {
        let shell_state = seeded_shell_state_with_projection();
        let primary_target = ui_composition::PresentationTargetId::try_from_raw(1).unwrap();
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let shared_bounds = UiRect::new(0.0, 0.0, 200.0, 120.0);
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            1,
            primary_target,
            ViewportId(5),
            shared_bounds,
        ));
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            ViewportId(8),
            shared_bounds,
        ));
        let dispatch = editor_shell::UiInputDispatchResult {
            target: None,
            response: InputResponse::ignored(),
        };

        let route = viewport_pointer_route(
            &shell_state,
            &bindings,
            secondary_target,
            Some(&dispatch),
            UiPoint::new(40.0, 50.0),
        )
        .expect("secondary target route should resolve");

        assert_eq!(route.viewport_id, ViewportId(8));
    }

    #[test]
    fn target_local_scroll_routes_only_to_secondary_viewport() {
        let primary_target = ui_composition::PresentationTargetId::try_from_raw(1).unwrap();
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let shared_bounds = UiRect::new(0.0, 0.0, 200.0, 120.0);
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            1,
            primary_target,
            ViewportId(5),
            shared_bounds,
        ));
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            ViewportId(8),
            shared_bounds,
        ));
        let mut host = EditorHostResource::default();
        let mut bridge = EditorInputBridgeState::default();
        let mut picking = ViewportPickingResultsResource::default();
        let render_states = ViewportRenderStateResource::default();
        let mut commands = ViewportRenderStateCommandQueueResource::default();
        let event = mouse_pointer_event(
            PointerEventKind::Scroll,
            UiPoint::new(40.0, 50.0),
            UiVector::new(0.0, 2.0),
            None,
        );

        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &event,
            None,
            false,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![ViewportRenderStateCommand::ZoomCamera {
                viewport_id: ViewportId(8),
                scroll_delta: 2.0,
            }]
        );
    }

    #[test]
    fn ui_consumption_suppresses_target_local_scene_scroll() {
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            ViewportId(8),
            UiRect::new(0.0, 0.0, 200.0, 120.0),
        ));
        let mut host = EditorHostResource::default();
        let mut bridge = EditorInputBridgeState::default();
        let mut picking = ViewportPickingResultsResource::default();
        let render_states = ViewportRenderStateResource::default();
        let mut commands = ViewportRenderStateCommandQueueResource::default();

        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &mouse_pointer_event(
                PointerEventKind::Scroll,
                UiPoint::new(40.0, 50.0),
                UiVector::new(0.0, 2.0),
                None,
            ),
            None,
            true,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );

        assert_eq!(commands.len(), 0);
    }

    #[test]
    fn secondary_camera_capture_survives_pointer_leaving_viewport() {
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            ViewportId(8),
            UiRect::new(0.0, 0.0, 200.0, 120.0),
        ));
        let mut host = EditorHostResource::default();
        let mut bridge = EditorInputBridgeState::default();
        let mut picking = ViewportPickingResultsResource::default();
        let render_states = ViewportRenderStateResource::default();
        let mut commands = ViewportRenderStateCommandQueueResource::default();

        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &mouse_pointer_event(
                PointerEventKind::Down,
                UiPoint::new(40.0, 50.0),
                UiVector::ZERO,
                Some(PointerButton::Middle),
            ),
            None,
            false,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );
        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &mouse_pointer_event(
                PointerEventKind::Move,
                UiPoint::new(900.0, 900.0),
                UiVector::new(6.0, -3.0),
                None,
            ),
            None,
            false,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![ViewportRenderStateCommand::PanCamera {
                viewport_id: ViewportId(8),
                delta: UiVector::new(6.0, -3.0),
            }]
        );
    }

    #[test]
    fn secondary_right_drag_orbits_only_captured_secondary_viewport() {
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            ViewportId(8),
            UiRect::new(0.0, 0.0, 200.0, 120.0),
        ));
        let mut host = EditorHostResource::default();
        let mut bridge = EditorInputBridgeState::default();
        let mut picking = ViewportPickingResultsResource::default();
        let render_states = ViewportRenderStateResource::default();
        let mut commands = ViewportRenderStateCommandQueueResource::default();

        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &mouse_pointer_event(
                PointerEventKind::Down,
                UiPoint::new(40.0, 50.0),
                UiVector::ZERO,
                Some(PointerButton::Secondary),
            ),
            None,
            false,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );
        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &mouse_pointer_event(
                PointerEventKind::Move,
                UiPoint::new(900.0, 900.0),
                UiVector::new(4.0, 5.0),
                None,
            ),
            None,
            false,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );

        assert_eq!(
            commands.drain().collect::<Vec<_>>(),
            vec![ViewportRenderStateCommand::OrbitCamera {
                viewport_id: ViewportId(8),
                delta: UiVector::new(4.0, 5.0),
            }]
        );
    }

    #[test]
    fn secondary_primary_down_refreshes_picking_before_scene_interaction() {
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let mut host = EditorHostResource::default();
        let tool_surface_id = viewport_surface_id(&host.shell_state);
        let viewport_id = ViewportId(8);
        let bounds = UiRect::new(0.0, 0.0, 200.0, 120.0);
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            tool_surface_id.raw(),
            secondary_target,
            viewport_id,
            bounds,
        ));
        let mut bridge = EditorInputBridgeState::default();
        let mut picking = ViewportPickingResultsResource::default();
        picking.set_viewport_result(
            viewport_id,
            (40.0, 50.0),
            (bounds.x, bounds.y, bounds.width, bounds.height),
            engine::plugins::render::EditorPickingHit {
                target: EditorPickingTarget::Entity(999),
                distance: 1.0,
            },
        );
        let stale_revision = picking.global_revision();
        let render_states = ViewportRenderStateResource::default();
        let mut commands = ViewportRenderStateCommandQueueResource::default();

        dispatch_editor_viewport_input_for_target(
            secondary_target,
            &mouse_pointer_event(
                PointerEventKind::Down,
                UiPoint::new(40.0, 50.0),
                UiVector::ZERO,
                Some(PointerButton::Primary),
            ),
            None,
            false,
            &mut host,
            &mut bridge,
            &mut picking,
            &render_states,
            &bindings,
            &mut commands,
        );

        assert!(
            picking.global_revision() > stale_revision,
            "secondary pointer down must recompute target-local picking before selection",
        );
        assert_eq!(
            picking.result_for(viewport_id).unwrap().hit.target,
            EditorPickingTarget::None,
            "stale secondary hit must be replaced by the freshly computed result",
        );
    }

    #[test]
    fn target_local_cleanup_preserves_other_target_capture_and_picking() {
        let primary_target = ui_composition::PresentationTargetId::try_from_raw(1).unwrap();
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let primary_viewport = ViewportId(5);
        let secondary_viewport = ViewportId(8);
        let bounds = UiRect::new(0.0, 0.0, 200.0, 120.0);
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            1,
            primary_target,
            primary_viewport,
            bounds,
        ));
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            secondary_viewport,
            bounds,
        ));
        let mut bridge = EditorInputBridgeState::default();
        bridge
            .interaction_for_target_mut(primary_target)
            .pointer_owner = EditorPointerOwner::ViewportCamera {
            viewport_id: primary_viewport,
            button: EditorCameraPointerButton::Middle,
        };
        bridge
            .interaction_for_target_mut(secondary_target)
            .pointer_owner = EditorPointerOwner::ViewportCamera {
            viewport_id: secondary_viewport,
            button: EditorCameraPointerButton::Secondary,
        };
        let mut picking = ViewportPickingResultsResource::default();
        for (viewport_id, entity) in [(primary_viewport, 5), (secondary_viewport, 8)] {
            picking.set_viewport_result(
                viewport_id,
                (40.0, 50.0),
                (bounds.x, bounds.y, bounds.width, bounds.height),
                engine::plugins::render::EditorPickingHit {
                    target: EditorPickingTarget::Entity(entity),
                    distance: 1.0,
                },
            );
        }

        clear_editor_viewport_interaction_for_target(
            secondary_target,
            &mut bridge,
            &mut picking,
            &bindings,
        );

        assert!(matches!(
            bridge.interaction_for_target(primary_target).pointer_owner,
            EditorPointerOwner::ViewportCamera {
                viewport_id: ViewportId(5),
                ..
            }
        ));
        assert_eq!(
            bridge
                .interaction_for_target(secondary_target)
                .pointer_owner,
            EditorPointerOwner::None,
        );
        assert_eq!(
            picking.result_for(primary_viewport).unwrap().hit.target,
            EditorPickingTarget::Entity(5),
        );
        assert_eq!(
            picking.result_for(secondary_viewport).unwrap().hit.target,
            EditorPickingTarget::None,
        );
    }

    #[test]
    fn primary_and_secondary_camera_captures_remain_independent() {
        let primary_target = ui_composition::PresentationTargetId::try_from_raw(1).unwrap();
        let secondary_target = ui_composition::PresentationTargetId::try_from_raw(2).unwrap();
        let shared_bounds = UiRect::new(0.0, 0.0, 200.0, 120.0);
        let mut bindings = ToolSurfaceRuntimeBindingRegistryResource::default();
        bindings.upsert_binding(manual_binding_for_target(
            1,
            primary_target,
            ViewportId(5),
            shared_bounds,
        ));
        bindings.upsert_binding(manual_binding_for_target(
            2,
            secondary_target,
            ViewportId(8),
            shared_bounds,
        ));
        let mut host = EditorHostResource::default();
        let mut bridge = EditorInputBridgeState::default();
        let mut picking = ViewportPickingResultsResource::default();
        let render_states = ViewportRenderStateResource::default();
        let mut commands = ViewportRenderStateCommandQueueResource::default();

        for (target_id, button) in [
            (primary_target, PointerButton::Middle),
            (secondary_target, PointerButton::Secondary),
        ] {
            dispatch_editor_viewport_input_for_target(
                target_id,
                &mouse_pointer_event(
                    PointerEventKind::Down,
                    UiPoint::new(40.0, 50.0),
                    UiVector::ZERO,
                    Some(button),
                ),
                None,
                false,
                &mut host,
                &mut bridge,
                &mut picking,
                &render_states,
                &bindings,
                &mut commands,
            );
        }

        assert!(matches!(
            bridge.interaction_for_target(primary_target).pointer_owner,
            EditorPointerOwner::ViewportCamera {
                viewport_id: ViewportId(5),
                button: EditorCameraPointerButton::Middle,
            }
        ));
        assert!(matches!(
            bridge
                .interaction_for_target(secondary_target)
                .pointer_owner,
            EditorPointerOwner::ViewportCamera {
                viewport_id: ViewportId(8),
                button: EditorCameraPointerButton::Secondary,
            }
        ));

        bridge.clear_interaction_for_target(secondary_target);
        assert!(matches!(
            bridge.interaction_for_target(primary_target).pointer_owner,
            EditorPointerOwner::ViewportCamera {
                viewport_id: ViewportId(5),
                ..
            }
        ));
        assert_eq!(
            bridge
                .interaction_for_target(secondary_target)
                .pointer_owner,
            EditorPointerOwner::None,
        );
    }

    #[test]
    fn focus_binding_uses_last_target_viewport_when_cursor_is_not_hovering() {
        let bindings = dual_viewport_bindings();

        let binding = viewport_binding_for_focus(
            &bindings,
            ui_composition::PresentationTargetId::try_from_raw(1).unwrap(),
            UiPoint::new(900.0, 900.0),
            Some(ViewportId(8)),
        )
        .expect("last target viewport should resolve");

        assert_eq!(binding.viewport_id, ViewportId(8));
    }
}
