use std::collections::{BTreeMap, HashMap};

use engine::plugins::{
    ContactPhase, DigitalState, InputContext, InputDeviceId, LogicalKey, MeasurementDomain,
    ObservationOrigin, PhysicalKeyIdentity, PointerButton as EnginePointerButton, ScrollDomain,
};
use engine::runtime::platform::PlatformEvent;
use engine::runtime::NativeWindowId;
use ui_input::{
    Key, KeyState, KeyboardEvent, Modifiers, PointerButton, PointerContactId, PointerContactPhase,
    PointerContactState, PointerDeviceId, PointerEvent, PointerEventKind, PointerPacket,
    PointerSourceKind, PointerToolKind, TextInputEvent, UiInputEvent,
};
use ui_math::{UiPoint, UiVector};

const UI_WHEEL_STEP_PX: f32 = 28.0;

#[derive(Clone, Copy, Debug, Default)]
struct ModifierState {
    shift_left: bool,
    shift_right: bool,
    control_left: bool,
    control_right: bool,
    alt_left: bool,
    alt_right: bool,
    meta_left: bool,
    meta_right: bool,
}

impl ModifierState {
    fn update(&mut self, key: &PhysicalKeyIdentity, state: DigitalState) {
        let pressed = state == DigitalState::Pressed;
        let PhysicalKeyIdentity::Code(code) = key else {
            return;
        };
        match code.as_str() {
            "ShiftLeft" => self.shift_left = pressed,
            "ShiftRight" => self.shift_right = pressed,
            "ControlLeft" => self.control_left = pressed,
            "ControlRight" => self.control_right = pressed,
            "AltLeft" => self.alt_left = pressed,
            "AltRight" => self.alt_right = pressed,
            "SuperLeft" => self.meta_left = pressed,
            "SuperRight" => self.meta_right = pressed,
            _ => {}
        }
    }

    const fn snapshot(self) -> Modifiers {
        Modifiers {
            shift: self.shift_left || self.shift_right,
            ctrl: self.control_left || self.control_right,
            alt: self.alt_left || self.alt_right,
            meta: self.meta_left || self.meta_right,
        }
    }
}

#[derive(Debug, Default)]
struct TargetInputState {
    mouse_cursor: UiPoint,
    touch_positions: BTreeMap<u64, UiPoint>,
    modifiers: ModifierState,
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct EditorTargetInputRuntimeResource {
    by_window: BTreeMap<NativeWindowId, TargetInputState>,
    device_ids: HashMap<InputDeviceId, PointerDeviceId>,
    next_device_id: u64,
}

impl EditorTargetInputRuntimeResource {
    pub(crate) fn clear_window(&mut self, native_window_id: NativeWindowId) {
        self.by_window.remove(&native_window_id);
    }

    fn device_id(&mut self, context: InputContext) -> Option<PointerDeviceId> {
        let device = context.device?;
        if let Some(id) = self.device_ids.get(&device) {
            return Some(*id);
        }
        self.next_device_id = self
            .next_device_id
            .checked_add(1)
            .expect("editor UI input device identity exhausted");
        let id = PointerDeviceId(self.next_device_id);
        self.device_ids.insert(device, id);
        Some(id)
    }
}

pub(crate) fn translate_platform_event(
    runtime: &mut EditorTargetInputRuntimeResource,
    native_window_id: NativeWindowId,
    event: PlatformEvent,
) -> Vec<UiInputEvent> {
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
    let state = runtime.by_window.entry(native_window_id).or_default();

    match event {
        PlatformEvent::CursorMoved { position, .. } => {
            let next = UiPoint::new(position.x, position.y);
            let delta = next - state.mouse_cursor;
            state.mouse_cursor = next;
            vec![UiInputEvent::Pointer(pointer_event(
                PointerEventKind::Move,
                next,
                delta,
                None,
                state.modifiers.snapshot(),
                0,
                pointer_packet(PointerSourceKind::Mouse, PointerToolKind::Mouse, device_id),
            ))]
        }
        PlatformEvent::MouseWheel { input, .. } => vec![UiInputEvent::Pointer(pointer_event(
            PointerEventKind::Scroll,
            state.mouse_cursor,
            scroll_delta(input.delta.horizontal, input.delta.vertical, input.domain),
            None,
            state.modifiers.snapshot(),
            0,
            pointer_packet(PointerSourceKind::Mouse, PointerToolKind::Mouse, device_id),
        ))],
        PlatformEvent::MouseInput { input, .. } => pointer_button(input.button)
            .map(|button| {
                vec![UiInputEvent::Pointer(pointer_event(
                    if input.state == DigitalState::Pressed {
                        PointerEventKind::Down
                    } else {
                        PointerEventKind::Up
                    },
                    state.mouse_cursor,
                    UiVector::ZERO,
                    Some(button),
                    state.modifiers.snapshot(),
                    u8::from(input.state == DigitalState::Pressed),
                    pointer_packet(PointerSourceKind::Mouse, PointerToolKind::Mouse, device_id),
                ))]
            })
            .unwrap_or_default(),
        PlatformEvent::KeyboardInput { input, .. } => {
            state.modifiers.update(&input.physical_key, input.state);
            if input.origin == ObservationOrigin::BackendSyntheticReconciliation {
                return Vec::new();
            }
            logical_key(&input.logical_key)
                .map(|key| {
                    vec![UiInputEvent::Keyboard(KeyboardEvent {
                        key,
                        state: key_state(input.state, input.repeat),
                        modifiers: state.modifiers.snapshot(),
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
            let previous = state.touch_positions.get(&input.id).copied().unwrap_or(next);
            let delta = next - previous;
            match input.phase {
                ContactPhase::Begin | ContactPhase::Update => {
                    state.touch_positions.insert(input.id, next);
                }
                ContactPhase::End | ContactPhase::Cancel => {
                    state.touch_positions.remove(&input.id);
                }
            }
            let (kind, phase) = match input.phase {
                ContactPhase::Begin => (PointerEventKind::Down, PointerContactPhase::Begin),
                ContactPhase::Update => (PointerEventKind::Move, PointerContactPhase::Update),
                ContactPhase::End => (PointerEventKind::Up, PointerContactPhase::End),
                ContactPhase::Cancel => (PointerEventKind::Leave, PointerContactPhase::Cancel),
            };
            let mut packet = pointer_packet(
                PointerSourceKind::Touch,
                PointerToolKind::Finger,
                device_id,
            )
            .with_contact_lifecycle(PointerContactId(input.id), phase)
            .with_contact(if matches!(input.phase, ContactPhase::Begin | ContactPhase::Update) {
                PointerContactState::Contact
            } else {
                PointerContactState::OutOfRange
            });
            if let Some(pressure) = input.pressure
                && pressure.domain == MeasurementDomain::NormalizedUnitInterval
            {
                packet = packet.with_pressure(pressure.value);
            }
            vec![UiInputEvent::Pointer(pointer_event(
                kind,
                next,
                delta,
                Some(PointerButton::Primary),
                state.modifiers.snapshot(),
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
    modifiers: Modifiers,
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
        ScrollDomain::LegacyVerticalScalarUnknown | ScrollDomain::Lines => 1.0,
        ScrollDomain::WindowPhysicalPixels => 1.0 / UI_WHEEL_STEP_PX,
    };
    UiVector::new(horizontal.unwrap_or(0.0) * scale, vertical.unwrap_or(0.0) * scale)
}

#[cfg(test)]
mod tests;
