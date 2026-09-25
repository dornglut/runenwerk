mod state;

use engine::runtime::NativeWindowId;
use engine::runtime::platform::PlatformEvent;
use runen_input::{
    ContactPhase, DigitalState, LogicalKey, MeasurementDomain, ObservationOrigin,
    PointerButton as EnginePointerButton, ScrollDomain,
};
use ui_input::{
    Key, KeyState, KeyboardEvent, PointerButton, PointerContactId, PointerContactPhase,
    PointerContactState, PointerDeviceId, PointerEvent, PointerEventKind, PointerPacket,
    PointerSourceKind, PointerToolKind, TextInputEvent, UiInputEvent,
};
use ui_math::{UiPoint, UiVector};

pub use state::EditorTargetInputRuntimeResource;

const UI_WHEEL_STEP_PX: f32 = 28.0;

pub(crate) fn translate_platform_event(
    runtime: &mut EditorTargetInputRuntimeResource,
    native_window_id: NativeWindowId,
    event: PlatformEvent,
) -> Vec<UiInputEvent> {
    if !platform_event_is_finite(&event) {
        return Vec::new();
    }
    if matches!(event, PlatformEvent::Focused { focused: false }) {
        runtime.clear_window(native_window_id);
        return Vec::new();
    }

    let device_id = match &event {
        PlatformEvent::MouseWheel { context, .. }
        | PlatformEvent::CursorMoved { context, .. }
        | PlatformEvent::MouseInput { context, .. }
        | PlatformEvent::Touch { context, .. } => runtime.device_id(*context),
        PlatformEvent::KeyboardInput { .. }
        | PlatformEvent::Resumed
        | PlatformEvent::CloseRequested
        | PlatformEvent::Focused { .. }
        | PlatformEvent::Resized { .. }
        | PlatformEvent::ScaleFactorChanged { .. }
        | PlatformEvent::TextInput { .. }
        | PlatformEvent::RedrawRequested => None,
    };
    let state = runtime.target_mut(native_window_id);

    match event {
        PlatformEvent::CursorMoved { context, position } => {
            let next = UiPoint::new(position.x, position.y);
            let delta = state.observe_mouse_position(context.source, next);
            vec![UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Move,
                next,
                delta,
                None,
                state.modifiers(),
                0,
                pointer_packet(PointerSourceKind::Mouse, PointerToolKind::Mouse, device_id),
            ))]
        }
        PlatformEvent::MouseWheel { context, input } => {
            vec![UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Scroll,
                state.mouse_position(context.source),
                scroll_delta(input.delta.horizontal, input.delta.vertical, input.domain),
                None,
                state.modifiers(),
                0,
                pointer_packet(PointerSourceKind::Mouse, PointerToolKind::Mouse, device_id),
            ))]
        }
        PlatformEvent::MouseInput { context, input } => pointer_button(input.button)
            .map(|button| {
                vec![UiInputEvent::Pointer(pointer_event(
                    if input.state == DigitalState::Pressed {
                        PointerEventKind::Down
                    } else {
                        PointerEventKind::Up
                    },
                    state.mouse_position(context.source),
                    UiVector::ZERO,
                    Some(button),
                    state.modifiers(),
                    u8::from(input.state == DigitalState::Pressed),
                    pointer_packet(PointerSourceKind::Mouse, PointerToolKind::Mouse, device_id),
                ))]
            })
            .unwrap_or_default(),
        PlatformEvent::KeyboardInput { context, input } => {
            state.update_modifiers(context, &input.physical_key, input.state);
            if input.origin == ObservationOrigin::BackendSyntheticReconciliation {
                return Vec::new();
            }
            logical_key(&input.logical_key)
                .map(|key| {
                    vec![UiInputEvent::Keyboard(KeyboardEvent {
                        key,
                        state: key_state(input.state, input.repeat),
                        modifiers: state.modifiers(),
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
        PlatformEvent::Touch { context, input } => {
            let next = UiPoint::new(input.position.x, input.position.y);
            let delta = state.observe_touch(context, input.contact.raw(), input.phase, next);
            let (kind, phase) = match input.phase {
                ContactPhase::Begin => (PointerEventKind::Down, PointerContactPhase::Begin),
                ContactPhase::Update => (PointerEventKind::Move, PointerContactPhase::Update),
                ContactPhase::End => (PointerEventKind::Up, PointerContactPhase::End),
                ContactPhase::Cancel => (PointerEventKind::Leave, PointerContactPhase::Cancel),
            };
            let mut packet =
                pointer_packet(PointerSourceKind::Touch, PointerToolKind::Finger, device_id)
                    .with_contact_lifecycle(PointerContactId(input.contact.raw()), phase)
                    .with_contact(
                        if matches!(input.phase, ContactPhase::Begin | ContactPhase::Update) {
                            PointerContactState::Contact
                        } else {
                            PointerContactState::OutOfRange
                        },
                    );
            if let Some(pressure) = input.pressure
                && pressure.domain == MeasurementDomain::NormalizedUnitInterval
                && (0.0..=1.0).contains(&pressure.value)
            {
                packet = packet.with_pressure(pressure.value);
            }
            vec![UiInputEvent::Pointer(pointer_event(
                kind,
                next,
                delta,
                Some(PointerButton::Primary),
                state.modifiers(),
                u8::from(input.phase == ContactPhase::Begin),
                packet,
            ))]
        }
        PlatformEvent::Focused { .. }
        | PlatformEvent::Resumed
        | PlatformEvent::CloseRequested
        | PlatformEvent::Resized { .. }
        | PlatformEvent::ScaleFactorChanged { .. }
        | PlatformEvent::RedrawRequested => Vec::new(),
    }
}

fn platform_event_is_finite(event: &PlatformEvent) -> bool {
    match event {
        PlatformEvent::MouseWheel { input, .. } => {
            input.delta.horizontal.is_none_or(f32::is_finite)
                && input.delta.vertical.is_none_or(f32::is_finite)
        }
        PlatformEvent::CursorMoved { position, .. } => {
            position.x.is_finite() && position.y.is_finite()
        }
        PlatformEvent::Touch { input, .. } => {
            input.position.x.is_finite()
                && input.position.y.is_finite()
                && input.pressure.is_none_or(|pressure| {
                    pressure.value.is_finite()
                        && pressure
                            .domain
                            .max_possible_force()
                            .is_none_or(f32::is_finite)
                })
                && input.altitude_angle_radians.is_none_or(f32::is_finite)
        }
        PlatformEvent::Resumed
        | PlatformEvent::CloseRequested
        | PlatformEvent::Focused { .. }
        | PlatformEvent::Resized { .. }
        | PlatformEvent::ScaleFactorChanged { .. }
        | PlatformEvent::KeyboardInput { .. }
        | PlatformEvent::TextInput { .. }
        | PlatformEvent::MouseInput { .. }
        | PlatformEvent::RedrawRequested => true,
    }
}

fn pointer_packet(
    source_kind: PointerSourceKind,
    tool_kind: PointerToolKind,
    device_id: Option<PointerDeviceId>,
) -> PointerPacket {
    PointerPacket {
        source_kind,
        tool_kind,
        device_id,
        ..PointerPacket::default()
    }
}

fn pointer_event(
    kind: PointerEventKind,
    position: UiPoint,
    delta: UiVector,
    button: Option<PointerButton>,
    modifiers: ui_input::Modifiers,
    click_count: u8,
    packet: PointerPacket,
) -> PointerEvent {
    PointerEvent {
        kind,
        position,
        delta,
        button,
        modifiers,
        click_count,
        packet,
    }
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

fn key_state(state: DigitalState, repeat: bool) -> KeyState {
    match (state, repeat) {
        (DigitalState::Pressed, true) => KeyState::Repeated,
        (DigitalState::Pressed, false) => KeyState::Pressed,
        (DigitalState::Released, _) => KeyState::Released,
    }
}

fn logical_key(key: &LogicalKey) -> Option<Key> {
    match key {
        LogicalKey::Character(value) => Some(Key::Character(value.clone())),
        LogicalKey::Named(value) => named_key(value),
        LogicalKey::Native(_) | LogicalKey::Dead(_) => None,
    }
}

fn named_key(value: &str) -> Option<Key> {
    Some(match value {
        "Enter" => Key::Enter,
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
        _ => return None,
    })
}

fn scroll_delta(horizontal: Option<f32>, vertical: Option<f32>, domain: ScrollDomain) -> UiVector {
    let scale = match domain {
        ScrollDomain::Unspecified | ScrollDomain::Lines => 1.0,
        ScrollDomain::WindowPhysicalPixels => 1.0 / UI_WHEEL_STEP_PX,
    };
    UiVector::new(
        horizontal.unwrap_or(0.0) * scale,
        vertical.unwrap_or(0.0) * scale,
    )
}

#[cfg(test)]
mod tests;
