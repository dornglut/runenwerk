use std::collections::HashMap;

use crate::runtime::window::NativeWindowId;
use runen_input::{
    AnalogMeasurement, ContactId, ContactInput, ContactPhase, CoordinateSpace, DigitalState,
    InputContext, InputDeviceId, InputSourceId, KeyLocation, KeyboardInput, LogicalKey,
    MeasurementDomain, NativeLogicalKey, NativePhysicalKeyCode, ObservationOrigin,
    PhysicalKeyIdentity, Point2, PointerButton, PointerButtonInput, ScrollDelta, ScrollDomain,
    ScrollInput, ScrollPhase,
};
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceId, ElementState, Force, MouseButton, MouseScrollDelta, Touch, TouchPhase,
};
use winit::keyboard::{Key, NativeKey, NativeKeyCode, PhysicalKey};

const FIRST_WINIT_SOURCE_ID: u64 = 1024;

#[derive(Debug)]
pub(crate) struct WinitInputAdapter {
    next_source_id: u64,
    next_device_id: u64,
    window_devices: HashMap<DeviceId, InputDeviceId>,
    raw_devices: HashMap<DeviceId, InputDeviceId>,
    window_sources: HashMap<NativeWindowId, InputSourceId>,
    raw_sources: HashMap<InputDeviceId, InputSourceId>,
}

impl Default for WinitInputAdapter {
    fn default() -> Self {
        Self {
            next_source_id: FIRST_WINIT_SOURCE_ID,
            next_device_id: 0,
            window_devices: HashMap::new(),
            raw_devices: HashMap::new(),
            window_sources: HashMap::new(),
            raw_sources: HashMap::new(),
        }
    }
}

impl WinitInputAdapter {
    pub(crate) fn window_context(
        &mut self,
        native_window_id: NativeWindowId,
        backend_device_id: DeviceId,
    ) -> InputContext {
        let device = self.intern_window_device(backend_device_id);
        let source = self.source_for_window(native_window_id);
        InputContext::new(source, Some(device))
    }

    pub(crate) fn raw_device_context(&mut self, backend_device_id: DeviceId) -> InputContext {
        let device = self.intern_raw_device(backend_device_id);
        let source = self.source_for_raw_device(device);
        InputContext::new(source, Some(device))
    }

    fn intern_window_device(&mut self, backend_device_id: DeviceId) -> InputDeviceId {
        if let Some(device) = self.window_devices.get(&backend_device_id).copied() {
            return device;
        }
        let device = self.allocate_device();
        self.window_devices.insert(backend_device_id, device);
        device
    }

    fn intern_raw_device(&mut self, backend_device_id: DeviceId) -> InputDeviceId {
        if let Some(device) = self.raw_devices.get(&backend_device_id).copied() {
            return device;
        }
        let device = self.allocate_device();
        self.raw_devices.insert(backend_device_id, device);
        device
    }

    fn allocate_device(&mut self) -> InputDeviceId {
        self.next_device_id = self
            .next_device_id
            .checked_add(1)
            .expect("winit input device identity exhausted");
        InputDeviceId::new(self.next_device_id)
    }

    fn source_for_window(&mut self, native_window_id: NativeWindowId) -> InputSourceId {
        if let Some(source) = self.window_sources.get(&native_window_id).copied() {
            return source;
        }
        let source = self.allocate_source();
        self.window_sources.insert(native_window_id, source);
        source
    }

    fn source_for_raw_device(&mut self, device: InputDeviceId) -> InputSourceId {
        if let Some(source) = self.raw_sources.get(&device).copied() {
            return source;
        }
        let source = self.allocate_source();
        self.raw_sources.insert(device, source);
        source
    }

    fn allocate_source(&mut self) -> InputSourceId {
        self.next_source_id = self
            .next_source_id
            .checked_add(1)
            .expect("winit input source identity exhausted");
        InputSourceId::new(self.next_source_id)
    }
}

pub(crate) fn keyboard_input(event: &winit::event::KeyEvent, is_synthetic: bool) -> KeyboardInput {
    KeyboardInput {
        physical_key: physical_key(event.physical_key),
        logical_key: logical_key(&event.logical_key),
        location: key_location(event.location),
        state: digital_state(event.state),
        repeat: event.repeat,
        origin: observation_origin(is_synthetic),
    }
}

pub(crate) fn text_input(event: &winit::event::KeyEvent, is_synthetic: bool) -> Option<String> {
    if is_synthetic || event.state != ElementState::Pressed {
        return None;
    }
    event
        .text
        .as_deref()
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

pub(crate) fn scroll_input(delta: MouseScrollDelta, phase: TouchPhase) -> ScrollInput {
    match delta {
        MouseScrollDelta::LineDelta(horizontal, vertical) => ScrollInput {
            delta: ScrollDelta::two_dimensional(horizontal, vertical),
            domain: ScrollDomain::Lines,
            phase: Some(scroll_phase(phase)),
        },
        MouseScrollDelta::PixelDelta(position) => ScrollInput {
            delta: ScrollDelta::two_dimensional(position.x as f32, position.y as f32),
            domain: ScrollDomain::WindowPhysicalPixels,
            phase: Some(scroll_phase(phase)),
        },
    }
}

pub(crate) fn cursor_position(position: PhysicalPosition<f64>) -> Point2 {
    Point2::new(
        position.x as f32,
        position.y as f32,
        CoordinateSpace::WindowPhysicalPixels,
    )
}

pub(crate) fn pointer_button_input(state: ElementState, button: MouseButton) -> PointerButtonInput {
    PointerButtonInput {
        button: pointer_button(button),
        state: digital_state(state),
    }
}

pub(crate) fn contact_input(touch: Touch) -> ContactInput {
    let (pressure, altitude_angle_radians) = touch
        .force
        .map(force_evidence)
        .map(|evidence| (Some(evidence.pressure), evidence.altitude_angle_radians))
        .unwrap_or((None, None));

    ContactInput {
        contact: ContactId::new(touch.id),
        phase: contact_phase(touch.phase),
        position: cursor_position(touch.location),
        pressure,
        altitude_angle_radians,
    }
}

fn physical_key(key: PhysicalKey) -> PhysicalKeyIdentity {
    match key {
        PhysicalKey::Code(code) => PhysicalKeyIdentity::code(format!("{code:?}")),
        PhysicalKey::Unidentified(code) => {
            PhysicalKeyIdentity::Native(native_physical_key_code(code))
        }
    }
}

fn native_physical_key_code(code: NativeKeyCode) -> NativePhysicalKeyCode {
    match code {
        NativeKeyCode::Unidentified => NativePhysicalKeyCode::Unidentified,
        NativeKeyCode::Android(code) => NativePhysicalKeyCode::Android(code),
        NativeKeyCode::MacOS(code) => NativePhysicalKeyCode::MacOs(code as u32),
        NativeKeyCode::Windows(code) => NativePhysicalKeyCode::Windows(code as u32),
        NativeKeyCode::Xkb(code) => NativePhysicalKeyCode::Xkb(code),
    }
}

fn logical_key(key: &Key) -> LogicalKey {
    match key {
        Key::Named(key) => LogicalKey::Named(format!("{key:?}")),
        Key::Character(value) => LogicalKey::Character(value.to_string()),
        Key::Unidentified(key) => LogicalKey::Native(native_logical_key(key)),
        Key::Dead(value) => LogicalKey::Dead(*value),
    }
}

fn native_logical_key(key: &NativeKey) -> NativeLogicalKey {
    match key {
        NativeKey::Unidentified => NativeLogicalKey::Unidentified,
        NativeKey::Android(code) => NativeLogicalKey::Android(*code),
        NativeKey::MacOS(code) => NativeLogicalKey::MacOs(*code as u32),
        NativeKey::Windows(code) => NativeLogicalKey::Windows(*code as u32),
        NativeKey::Xkb(code) => NativeLogicalKey::Xkb(*code),
        NativeKey::Web(value) => NativeLogicalKey::Web(value.to_string()),
    }
}

fn key_location(location: winit::keyboard::KeyLocation) -> KeyLocation {
    match location {
        winit::keyboard::KeyLocation::Standard => KeyLocation::Standard,
        winit::keyboard::KeyLocation::Left => KeyLocation::Left,
        winit::keyboard::KeyLocation::Right => KeyLocation::Right,
        winit::keyboard::KeyLocation::Numpad => KeyLocation::Numpad,
    }
}

fn digital_state(state: ElementState) -> DigitalState {
    match state {
        ElementState::Pressed => DigitalState::Pressed,
        ElementState::Released => DigitalState::Released,
    }
}

fn observation_origin(is_synthetic: bool) -> ObservationOrigin {
    if is_synthetic {
        ObservationOrigin::BackendSyntheticReconciliation
    } else {
        ObservationOrigin::SourceReport
    }
}

fn pointer_button(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Left,
        MouseButton::Right => PointerButton::Right,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(value) => PointerButton::Other(value),
    }
}

fn scroll_phase(phase: TouchPhase) -> ScrollPhase {
    match phase {
        TouchPhase::Started => ScrollPhase::Begin,
        TouchPhase::Moved => ScrollPhase::Update,
        TouchPhase::Ended => ScrollPhase::End,
        TouchPhase::Cancelled => ScrollPhase::Cancel,
    }
}

fn contact_phase(phase: TouchPhase) -> ContactPhase {
    match phase {
        TouchPhase::Started => ContactPhase::Begin,
        TouchPhase::Moved => ContactPhase::Update,
        TouchPhase::Ended => ContactPhase::End,
        TouchPhase::Cancelled => ContactPhase::Cancel,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ForceEvidence {
    pressure: AnalogMeasurement,
    altitude_angle_radians: Option<f32>,
}

fn force_evidence(force: Force) -> ForceEvidence {
    match force {
        Force::Calibrated {
            force,
            max_possible_force,
            altitude_angle,
        } => ForceEvidence {
            pressure: AnalogMeasurement::new(
                force as f32,
                MeasurementDomain::calibrated_force(max_possible_force),
            ),
            altitude_angle_radians: altitude_angle.map(|angle| angle as f32),
        },
        Force::Normalized(value) => ForceEvidence {
            pressure: AnalogMeasurement::new(
                value as f32,
                MeasurementDomain::NormalizedUnitInterval,
            ),
            altitude_angle_radians: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{KeyCode, NativeKeyCode};

    #[test]
    fn window_sources_are_target_scoped_while_device_identity_stays_separate() {
        let mut adapter = WinitInputAdapter::default();
        let primary = NativeWindowId::primary();
        let secondary = NativeWindowId::try_from_raw(2).expect("secondary window id");
        let backend_device = DeviceId::dummy();

        let primary_context = adapter.window_context(primary, backend_device);
        let secondary_context = adapter.window_context(secondary, backend_device);

        assert_ne!(primary_context.source, secondary_context.source);
        assert_eq!(primary_context.device, secondary_context.device);
        assert_eq!(
            primary_context,
            adapter.window_context(primary, backend_device)
        );
    }

    #[test]
    fn distinct_raw_devices_receive_distinct_stable_sources() {
        let mut adapter = WinitInputAdapter::default();
        let device_a = InputDeviceId::new(1);
        let device_b = InputDeviceId::new(2);

        let source_a = adapter.source_for_raw_device(device_a);
        let source_b = adapter.source_for_raw_device(device_b);

        assert_ne!(source_a, source_b);
        assert_eq!(source_a, adapter.source_for_raw_device(device_a));
    }

    #[test]
    fn raw_and_window_device_namespaces_remain_distinct() {
        let mut adapter = WinitInputAdapter::default();
        let backend_device = DeviceId::dummy();
        let window = adapter.window_context(NativeWindowId::primary(), backend_device);
        let raw = adapter.raw_device_context(backend_device);

        assert_ne!(window.source, raw.source);
        assert_ne!(window.device, raw.device);
    }

    #[test]
    fn physical_key_preserves_known_and_unidentified_identity() {
        assert_eq!(
            physical_key(PhysicalKey::Code(KeyCode::KeyA)),
            PhysicalKeyIdentity::code("KeyA")
        );
        assert_eq!(
            physical_key(PhysicalKey::Unidentified(NativeKeyCode::Xkb(41))),
            PhysicalKeyIdentity::Native(NativePhysicalKeyCode::Xkb(41))
        );
        assert_ne!(
            physical_key(PhysicalKey::Unidentified(NativeKeyCode::Xkb(41))),
            physical_key(PhysicalKey::Unidentified(NativeKeyCode::Xkb(42)))
        );
    }

    #[test]
    fn logical_key_and_location_preserve_backend_interpretation() {
        assert_eq!(
            logical_key(&Key::Character("z".into())),
            LogicalKey::Character("z".to_owned())
        );
        assert_eq!(
            key_location(winit::keyboard::KeyLocation::Numpad),
            KeyLocation::Numpad
        );
    }

    #[test]
    fn line_and_pixel_scroll_preserve_both_axes_domain_and_phase() {
        let line = scroll_input(MouseScrollDelta::LineDelta(2.0, -3.0), TouchPhase::Moved);
        let pixel = scroll_input(
            MouseScrollDelta::PixelDelta(PhysicalPosition::new(12.0, -18.0)),
            TouchPhase::Moved,
        );

        assert_eq!(line.delta, ScrollDelta::two_dimensional(2.0, -3.0));
        assert_eq!(line.domain, ScrollDomain::Lines);
        assert_eq!(line.phase, Some(ScrollPhase::Update));
        assert_eq!(pixel.delta, ScrollDelta::two_dimensional(12.0, -18.0));
        assert_eq!(pixel.domain, ScrollDomain::WindowPhysicalPixels);
        assert_eq!(pixel.phase, Some(ScrollPhase::Update));
        assert_ne!(line.domain, pixel.domain);
    }

    #[test]
    fn synthetic_origin_remains_distinct_from_source_report() {
        assert_eq!(
            observation_origin(true),
            ObservationOrigin::BackendSyntheticReconciliation
        );
        assert_eq!(observation_origin(false), ObservationOrigin::SourceReport);
    }

    #[test]
    fn contact_cancel_remains_distinct_from_ordinary_end() {
        assert_eq!(contact_phase(TouchPhase::Cancelled), ContactPhase::Cancel);
        assert_eq!(contact_phase(TouchPhase::Ended), ContactPhase::End);
        assert_ne!(
            contact_phase(TouchPhase::Cancelled),
            contact_phase(TouchPhase::Ended)
        );
    }

    #[test]
    fn calibrated_force_keeps_pressure_domain_and_altitude_separate() {
        let evidence = force_evidence(Force::Calibrated {
            force: 2.0,
            max_possible_force: 4.0,
            altitude_angle: Some(std::f64::consts::FRAC_PI_2),
        });

        assert_eq!(evidence.pressure.value, 2.0);
        assert_eq!(evidence.pressure.domain.max_possible_force(), Some(4.0));
        assert_eq!(
            evidence.altitude_angle_radians,
            Some(std::f32::consts::FRAC_PI_2)
        );
    }
}
