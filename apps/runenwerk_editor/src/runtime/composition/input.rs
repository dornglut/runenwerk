use std::collections::BTreeMap;

use engine::plugins::{
    ContactPhase, DigitalState, PhysicalKeyIdentity, PointerButton as EnginePointerButton,
};
use engine::runtime::platform::{PlatformEvent, PlatformWindowEventQueueResource};
use engine::runtime::{NativeWindowId, Res, ResMut, WindowStateRegistryResource};
use ui_input::{
    Key, KeyState, KeyboardEvent, Modifiers, PointerButton, PointerEvent, PointerEventKind,
    PointerSourceKind, PointerToolKind, SemanticActionEvent, SemanticInputSource, TextInputEvent,
    UiInputEvent, UiSemanticAction,
};
use ui_math::{UiPoint, UiRect, UiVector};

use crate::runtime::resources::{EditorHostResource, scaled_shell_theme};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPresentationStateResource,
    ViewportRenderStateCommandQueueResource,
};

#[derive(Clone, Copy, Debug, Default)]
struct TargetInputState {
    cursor: UiPoint,
    modifiers: Modifiers,
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct EditorTargetInputRuntimeResource {
    by_window: BTreeMap<NativeWindowId, TargetInputState>,
}

#[allow(clippy::too_many_arguments)]
pub fn dispatch_editor_target_input_system(
    mut host: ResMut<EditorHostResource>,
    mut runtime: ResMut<EditorTargetInputRuntimeResource>,
    mut events: ResMut<PlatformWindowEventQueueResource>,
    windows: Res<WindowStateRegistryResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    mut viewport_render_commands: ResMut<ViewportRenderStateCommandQueueResource>,
) {
    for window_event in events.drain() {
        let native_window_id = window_event.native_window_id;
        if native_window_id == NativeWindowId::primary() {
            continue;
        }
        let Some(target_id) = host
            .shell_state
            .composition_target_bindings()
            .find(|entry| entry.binding.native_window_id == native_window_id)
            .map(|entry| entry.target_id)
        else {
            continue;
        };
        let Some(window) = windows.record(native_window_id) else {
            continue;
        };
        if matches!(
            window_event.event,
            PlatformEvent::Focused { focused: false }
        ) {
            host.shell_state
                .runtime_for_target_mut(target_id)
                .set_focused_widget(None);
            host.shell_state.clear_tab_drag_for_target(target_id);
            continue;
        }
        let state = runtime.by_window.entry(native_window_id).or_default();
        let ui_events = translate_event(state, window_event.event);
        if ui_events.is_empty() {
            continue;
        }
        let bounds = UiRect::new(
            0.0,
            0.0,
            window.size_px.0.max(1) as f32,
            window.size_px.1.max(1) as f32,
        );
        let theme = scaled_shell_theme(&host.theme, window.scale_factor);
        for event in ui_events {
            let EditorHostResource {
                app, shell_state, ..
            } = &mut *host;
            if let Err(error) = app.dispatch_shell_input_for_target(
                shell_state,
                target_id,
                bounds,
                &theme,
                &event,
                Some(&mut *viewport_presentations),
                Some(&viewport_observations),
                Some(&tool_surface_bindings),
                Some(&viewport_instances),
                Some(&mut *viewport_render_commands),
            ) {
                app.append_console_line(format!(
                    "[editor_composition.input.target_dispatch_failed] {error}"
                ));
            }
        }
    }
}

fn translate_event(state: &mut TargetInputState, event: PlatformEvent) -> Vec<UiInputEvent> {
    match event {
        PlatformEvent::CursorMoved { position, .. } => {
            let next = UiPoint::new(position.x, position.y);
            let delta = next - state.cursor;
            state.cursor = next;
            vec![pointer_event(state, PointerEventKind::Move, delta, None)]
        }
        PlatformEvent::MouseWheel { input, .. } => vec![pointer_event(
            state,
            PointerEventKind::Scroll,
            UiVector::new(0.0, input.delta.vertical.unwrap_or(0.0)),
            None,
        )],
        PlatformEvent::MouseInput { input, .. } => pointer_button(input.button)
            .map(|button| {
                vec![pointer_event(
                    state,
                    if input.state == DigitalState::Pressed {
                        PointerEventKind::Down
                    } else {
                        PointerEventKind::Up
                    },
                    UiVector::ZERO,
                    Some(button),
                )]
            })
            .unwrap_or_default(),
        PlatformEvent::KeyboardInput { input, .. } => {
            update_modifiers(&mut state.modifiers, &input.physical_key, input.state);
            key_from_physical(&input.physical_key)
                .map(|key| {
                    vec![UiInputEvent::Keyboard(KeyboardEvent {
                        key,
                        state: if input.state == DigitalState::Pressed {
                            KeyState::Pressed
                        } else {
                            KeyState::Released
                        },
                        modifiers: state.modifiers,
                    })]
                })
                .unwrap_or_default()
        }
        PlatformEvent::TextInput { text } => {
            if text.is_empty() {
                Vec::new()
            } else {
                vec![UiInputEvent::Text(TextInputEvent { text })]
            }
        }
        PlatformEvent::Touch { input, .. } => {
            let next = UiPoint::new(input.position.x, input.position.y);
            let delta = next - state.cursor;
            state.cursor = next;
            let kind = match input.phase {
                ContactPhase::Begin => PointerEventKind::Down,
                ContactPhase::Update => PointerEventKind::Move,
                ContactPhase::End => PointerEventKind::Up,
                ContactPhase::Cancel => {
                    return vec![UiInputEvent::Semantic(SemanticActionEvent::new(
                        SemanticInputSource::Touch,
                        UiSemanticAction::Cancel,
                    ))];
                }
            };
            let mut event = pointer_event(state, kind, delta, Some(PointerButton::Primary));
            let UiInputEvent::Pointer(pointer) = &mut event else {
                unreachable!()
            };
            pointer.packet.source_kind = PointerSourceKind::Touch;
            pointer.packet.tool_kind = PointerToolKind::Finger;
            vec![event]
        }
        PlatformEvent::Focused { .. }
        | PlatformEvent::Resumed
        | PlatformEvent::CloseRequested
        | PlatformEvent::Resized { .. }
        | PlatformEvent::ScaleFactorChanged { .. }
        | PlatformEvent::RedrawRequested => Vec::new(),
    }
}

fn pointer_event(
    state: &TargetInputState,
    kind: PointerEventKind,
    delta: UiVector,
    button: Option<PointerButton>,
) -> UiInputEvent {
    UiInputEvent::Pointer(PointerEvent {
        kind,
        position: state.cursor,
        delta,
        button,
        modifiers: state.modifiers,
        click_count: 1,
        ..PointerEvent::default()
    })
}

fn pointer_button(button: EnginePointerButton) -> Option<PointerButton> {
    match button {
        EnginePointerButton::Left => Some(PointerButton::Primary),
        EnginePointerButton::Right => Some(PointerButton::Secondary),
        EnginePointerButton::Middle => Some(PointerButton::Middle),
        EnginePointerButton::Back => Some(PointerButton::Other(4)),
        EnginePointerButton::Forward => Some(PointerButton::Other(5)),
        EnginePointerButton::Other(value) => Some(PointerButton::Other(value)),
    }
}

fn update_modifiers(
    modifiers: &mut Modifiers,
    key: &PhysicalKeyIdentity,
    state: DigitalState,
) {
    let pressed = state == DigitalState::Pressed;
    let PhysicalKeyIdentity::Code(code) = key else {
        return;
    };
    match code.as_str() {
        "ShiftLeft" | "ShiftRight" => modifiers.shift = pressed,
        "ControlLeft" | "ControlRight" => modifiers.ctrl = pressed,
        "AltLeft" | "AltRight" => modifiers.alt = pressed,
        "SuperLeft" | "SuperRight" => modifiers.meta = pressed,
        _ => {}
    }
}

fn key_from_physical(key: &PhysicalKeyIdentity) -> Option<Key> {
    let PhysicalKeyIdentity::Code(code) = key else {
        return None;
    };
    Some(match code.as_str() {
        "Enter" | "NumpadEnter" => Key::Enter,
        "Escape" => Key::Escape,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Tab" => Key::Tab,
        "Space" => Key::Space,
        "ArrowLeft" => Key::Left,
        "ArrowRight" => Key::Right,
        "ArrowUp" => Key::Up,
        "ArrowDown" => Key::Down,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "Insert" => Key::Insert,
        "F1" => Key::F(1),
        "F2" => Key::F(2),
        "F3" => Key::F(3),
        "F4" => Key::F(4),
        "F5" => Key::F(5),
        "F6" => Key::F(6),
        "F7" => Key::F(7),
        "F8" => Key::F(8),
        "F9" => Key::F(9),
        "F10" => Key::F(10),
        "F11" => Key::F(11),
        "F12" => Key::F(12),
        "KeyA" => Key::Character("a".to_owned()),
        "KeyC" => Key::Character("c".to_owned()),
        "KeyD" => Key::Character("d".to_owned()),
        "KeyV" => Key::Character("v".to_owned()),
        "KeyX" => Key::Character("x".to_owned()),
        "KeyY" => Key::Character("y".to_owned()),
        "KeyZ" => Key::Character("z".to_owned()),
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::plugins::{
        ContactInput, CoordinateSpace, InputContext, InputSourceId, Point2,
    };

    fn test_context() -> InputContext {
        InputContext::new(InputSourceId::new(1), None)
    }

    fn touch_event(phase: ContactPhase) -> PlatformEvent {
        PlatformEvent::Touch {
            context: test_context(),
            input: ContactInput {
                id: 1,
                phase,
                position: Point2::new(12.0, 18.0, CoordinateSpace::WindowPhysicalPixels),
                pressure: None,
                altitude_angle_radians: None,
            },
        }
    }

    #[test]
    fn touch_events_preserve_existing_editor_translation_during_i1b() {
        let mut state = TargetInputState::default();
        let started = translate_event(&mut state, touch_event(ContactPhase::Begin));
        assert!(matches!(
            started.as_slice(),
            [UiInputEvent::Pointer(PointerEvent {
                packet,
                ..
            })] if packet.source_kind == PointerSourceKind::Touch
                && packet.tool_kind == PointerToolKind::Finger
        ));

        let cancelled = translate_event(&mut state, touch_event(ContactPhase::Cancel));
        assert_eq!(
            cancelled,
            vec![UiInputEvent::Semantic(SemanticActionEvent::new(
                SemanticInputSource::Touch,
                UiSemanticAction::Cancel,
            ))]
        );
    }

    #[test]
    fn physical_character_mapping_remains_deliberately_legacy_until_i1d() {
        assert_eq!(
            key_from_physical(&PhysicalKeyIdentity::code("KeyZ")),
            Some(Key::Character("z".to_owned()))
        );
    }
}
