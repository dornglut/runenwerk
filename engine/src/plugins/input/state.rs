use runen_input::{
    AnalogMeasurement, ContactId, ContactInput, ContactPhase as NeutralContactPhase,
    ContinuityLoss, CoordinateSpace, DigitalState, InputContext, InputError, InputObservation,
    InputObservationGroup, InputSourceId, InputState as NeutralInputState, KeyLocation,
    KeyboardInput, LogicalKey, MeasurementDomain, NativeLogicalKey, ObservationOrigin,
    PhysicalKeyIdentity, Point2, PointerButton, PointerButtonInput, RelativeMotionUnit,
    ScrollDelta, ScrollDomain, ScrollInput, Vector2,
};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

const LEGACY_WINDOW_SOURCE: InputSourceId = InputSourceId::new(1);
const LEGACY_DEVICE_SOURCE: InputSourceId = InputSourceId::new(2);
const LEGACY_WINDOW_CONTEXT: InputContext = InputContext::new(LEGACY_WINDOW_SOURCE, None);
const LEGACY_DEVICE_CONTEXT: InputContext = InputContext::new(LEGACY_DEVICE_SOURCE, None);

fn physical_identity_for_key_code(key: KeyCode) -> PhysicalKeyIdentity {
    PhysicalKeyIdentity::code(format!("{key:?}"))
}

fn pointer_button_from_legacy(button: MouseButton) -> PointerButton {
    match button {
        MouseButton::Left => PointerButton::Left,
        MouseButton::Right => PointerButton::Right,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Back => PointerButton::Back,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Other(value) => PointerButton::Other(value),
    }
}

fn legacy_mouse_button(button: PointerButton) -> MouseButton {
    match button {
        PointerButton::Left => MouseButton::Left,
        PointerButton::Right => MouseButton::Right,
        PointerButton::Middle => MouseButton::Middle,
        PointerButton::Back => MouseButton::Back,
        PointerButton::Forward => MouseButton::Forward,
        PointerButton::Other(value) => MouseButton::Other(value),
    }
}

fn digital_state_from_legacy(state: ElementState) -> DigitalState {
    match state {
        ElementState::Pressed => DigitalState::Pressed,
        ElementState::Released => DigitalState::Released,
    }
}

fn legacy_element_state(state: DigitalState) -> ElementState {
    match state {
        DigitalState::Pressed => ElementState::Pressed,
        DigitalState::Released => ElementState::Released,
    }
}

// Owner: Engine Input Plugin - Input State and Event Processing
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseMotionSample {
    pub position: (f32, f32),
    pub delta: (f32, f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseButtonTransitionSample {
    pub button: MouseButton,
    pub state: ElementState,
    pub position: (f32, f32),
    pub motion_sample_index: usize,
}

impl MouseButtonTransitionSample {
    pub fn is_left_pressed(self) -> bool {
        self.button == MouseButton::Left && self.state == ElementState::Pressed
    }

    pub fn is_left_released(self) -> bool {
        self.button == MouseButton::Left && self.state == ElementState::Released
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchInputPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

impl From<NeutralContactPhase> for TouchInputPhase {
    fn from(value: NeutralContactPhase) -> Self {
        match value {
            NeutralContactPhase::Begin => Self::Started,
            NeutralContactPhase::Update => Self::Moved,
            NeutralContactPhase::End => Self::Ended,
            NeutralContactPhase::Cancel => Self::Cancelled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TouchInputSample {
    pub id: u64,
    pub phase: TouchInputPhase,
    pub position: (f32, f32),
    pub delta: (f32, f32),
    pub pressure: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KeyboardPressSample {
    pub(super) physical_key: PhysicalKeyIdentity,
    pub(super) modifiers: ModifiersSnapshot,
}

#[derive(Debug, runen_ecs::Component, runen_ecs::Resource)]
pub struct InputState {
    neutral: NeutralInputState,
    keyboard_press_samples: Vec<KeyboardPressSample>,
    pub typed_text: String,
    pub overlay_consumed: bool,
    pub mouse_delta: (f32, f32),
    pub mouse_position: (f32, f32),
    mouse_motion_samples: Vec<MouseMotionSample>,
    mouse_button_transitions: Vec<MouseButtonTransitionSample>,
    touch_samples: Vec<TouchInputSample>,
    device_observation_groups: Vec<InputObservationGroup>,
    admitted_input_capture_active: bool,
    captured_admitted_groups: Vec<InputObservationGroup>,
    primary_touch: Option<(InputContext, ContactId)>,
    pub scroll_delta: f32,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
    right_mouse_pressed: bool,
    right_mouse_released: bool,
    middle_mouse_pressed: bool,
    middle_mouse_released: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            neutral: NeutralInputState::default(),
            keyboard_press_samples: Vec::new(),
            typed_text: String::new(),
            overlay_consumed: false,
            mouse_delta: (0.0, 0.0),
            mouse_position: (0.0, 0.0),
            mouse_motion_samples: Vec::new(),
            mouse_button_transitions: Vec::new(),
            touch_samples: Vec::new(),
            device_observation_groups: Vec::new(),
            admitted_input_capture_active: false,
            captured_admitted_groups: Vec::new(),
            primary_touch: None,
            scroll_delta: 0.0,
            left_mouse_pressed: false,
            left_mouse_released: false,
            right_mouse_pressed: false,
            right_mouse_released: false,
            middle_mouse_pressed: false,
            middle_mouse_released: false,
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    fn admit_group(&mut self, group: &InputObservationGroup) -> Result<(), InputError> {
        self.neutral.admit(group)?;
        if self.admitted_input_capture_active {
            self.captured_admitted_groups.push(group.clone());
        }
        Ok(())
    }

    pub fn start_admitted_input_capture(&mut self) {
        self.captured_admitted_groups.clear();
        self.admitted_input_capture_active = true;
    }

    pub fn admitted_input_capture_active(&self) -> bool {
        self.admitted_input_capture_active
    }

    pub(crate) fn pointer_button_down_anywhere(&self, button: PointerButton) -> bool {
        self.neutral.pointer_button_down_anywhere(button)
    }

    pub(crate) fn frame_projection_is_quiescent(&self) -> bool {
        self.keyboard_press_samples.is_empty()
            && self.typed_text.is_empty()
            && !self.overlay_consumed
            && self.mouse_delta == (0.0, 0.0)
            && self.mouse_motion_samples.is_empty()
            && self.mouse_button_transitions.is_empty()
            && self.touch_samples.is_empty()
            && self.device_observation_groups.is_empty()
            && self.scroll_delta == 0.0
            && !self.left_mouse_pressed
            && !self.left_mouse_released
            && !self.right_mouse_pressed
            && !self.right_mouse_released
            && !self.middle_mouse_pressed
            && !self.middle_mouse_released
    }

    pub fn drain_admitted_input_capture(&mut self) -> Vec<InputObservationGroup> {
        std::mem::take(&mut self.captured_admitted_groups)
    }

    pub fn reset_admitted_input_capture(&mut self) {
        self.captured_admitted_groups.clear();
    }

    pub fn stop_admitted_input_capture(&mut self) -> Vec<InputObservationGroup> {
        self.admitted_input_capture_active = false;
        self.drain_admitted_input_capture()
    }

    pub fn admit_device_observation_group(
        &mut self,
        group: InputObservationGroup,
    ) -> Result<(), InputError> {
        self.admit_group(&group)?;
        self.device_observation_groups.push(group);
        Ok(())
    }

    pub fn drain_device_observation_groups(&mut self) -> Vec<InputObservationGroup> {
        std::mem::take(&mut self.device_observation_groups)
    }

    pub(crate) fn handle_continuity_loss(&mut self, context: InputContext, loss: ContinuityLoss) {
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::ContinuityLoss(loss),
        ))
        .expect("input continuity loss should always be valid");

        if self.primary_touch.is_some_and(|(touch_context, _)| {
            continuity_loss_contains_context(context, loss, touch_context)
        }) {
            self.primary_touch = None;
        }
    }

    pub(crate) fn handle_normalized_keyboard(
        &mut self,
        context: InputContext,
        input: &KeyboardInput,
    ) {
        let was_down_anywhere = self.neutral.key_down_anywhere(&input.physical_key);
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::Keyboard(input.clone()),
        ))
        .expect("digital keyboard observation should always be valid");
        let is_down_anywhere = self.neutral.key_down_anywhere(&input.physical_key);

        if input.origin == ObservationOrigin::SourceReport
            && input.state == DigitalState::Pressed
            && !input.repeat
            && !was_down_anywhere
            && is_down_anywhere
        {
            self.keyboard_press_samples.push(KeyboardPressSample {
                physical_key: input.physical_key.clone(),
                modifiers: self.modifiers_snapshot(),
            });
        }
    }

    pub(crate) fn handle_text_input(&mut self, text: &str) {
        for ch in text.chars() {
            if !ch.is_control() {
                self.typed_text.push(ch);
            }
        }
    }

    pub fn handle_keyboard_input(
        &mut self,
        code: KeyCode,
        state: ElementState,
        text: Option<&str>,
    ) {
        let input = KeyboardInput {
            physical_key: physical_identity_for_key_code(code),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: digital_state_from_legacy(state),
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        };
        self.handle_normalized_keyboard(LEGACY_WINDOW_CONTEXT, &input);
        if let Some(text) = text {
            self.handle_text_input(text);
        }
    }

    #[cfg(test)]
    pub(crate) fn handle_keyboard_reconciliation(&mut self, code: KeyCode, state: ElementState) {
        let input = KeyboardInput {
            physical_key: physical_identity_for_key_code(code),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: digital_state_from_legacy(state),
            repeat: false,
            origin: ObservationOrigin::BackendSyntheticReconciliation,
        };
        self.handle_normalized_keyboard(LEGACY_WINDOW_CONTEXT, &input);
    }

    fn admit_scroll_input(
        &mut self,
        context: InputContext,
        input: ScrollInput,
    ) -> Result<(), InputError> {
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::Scroll(input),
        ))?;
        if let Some(vertical) = input.delta.vertical {
            self.scroll_delta += vertical;
        }
        Ok(())
    }

    pub(crate) fn handle_scroll_input(&mut self, context: InputContext, input: ScrollInput) {
        let _ = self.admit_scroll_input(context, input);
    }

    pub fn handle_mouse_wheel_delta(&mut self, delta: f32) {
        self.handle_scroll_input(
            LEGACY_WINDOW_CONTEXT,
            ScrollInput {
                delta: ScrollDelta::vertical_only(delta),
                domain: ScrollDomain::Unspecified,
                phase: None,
            },
        );
    }

    pub(crate) fn handle_cursor_position(&mut self, context: InputContext, position: Point2) {
        let previous = self
            .neutral
            .absolute_pointer_position(context.source)
            .map(|point| (point.x, point.y))
            .unwrap_or((0.0, 0.0));
        if self
            .admit_group(&InputObservationGroup::single(
                context,
                InputObservation::AbsolutePointerPosition { position },
            ))
            .is_err()
        {
            return;
        }

        self.mouse_position = (position.x, position.y);
        self.mouse_motion_samples.push(MouseMotionSample {
            position: (position.x, position.y),
            delta: (position.x - previous.0, position.y - previous.1),
        });
    }

    pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
        self.handle_cursor_position(
            LEGACY_WINDOW_CONTEXT,
            Point2::new(x, y, CoordinateSpace::UnspecifiedTargetUnits),
        );
    }

    fn admit_pointer_button(
        &mut self,
        context: InputContext,
        input: PointerButtonInput,
    ) -> Result<(), InputError> {
        let was_down_anywhere = self.neutral.pointer_button_down_anywhere(input.button);
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::PointerButton(input),
        ))?;
        let is_down_anywhere = self.neutral.pointer_button_down_anywhere(input.button);

        let changed = match input.state {
            DigitalState::Pressed => !was_down_anywhere && is_down_anywhere,
            DigitalState::Released => was_down_anywhere && !is_down_anywhere,
        };
        if !changed {
            return Ok(());
        }

        let position = self
            .neutral
            .absolute_pointer_position(context.source)
            .map(|point| (point.x, point.y))
            .unwrap_or((0.0, 0.0));
        self.mouse_button_transitions
            .push(MouseButtonTransitionSample {
                button: legacy_mouse_button(input.button),
                state: legacy_element_state(input.state),
                position,
                motion_sample_index: self.mouse_motion_samples.len(),
            });

        match input.state {
            DigitalState::Pressed => {
                if input.button == PointerButton::Left {
                    self.left_mouse_pressed = true;
                } else if input.button == PointerButton::Right {
                    self.right_mouse_pressed = true;
                } else if input.button == PointerButton::Middle {
                    self.middle_mouse_pressed = true;
                }
            }
            DigitalState::Released => {
                if input.button == PointerButton::Left {
                    self.left_mouse_released = true;
                } else if input.button == PointerButton::Right {
                    self.right_mouse_released = true;
                } else if input.button == PointerButton::Middle {
                    self.middle_mouse_released = true;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn handle_pointer_button(
        &mut self,
        context: InputContext,
        input: PointerButtonInput,
    ) {
        self.admit_pointer_button(context, input)
            .expect("digital pointer-button observation should always be valid");
    }

    pub fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        self.handle_pointer_button(
            LEGACY_WINDOW_CONTEXT,
            PointerButtonInput {
                button: pointer_button_from_legacy(button),
                state: digital_state_from_legacy(state),
            },
        );
    }

    fn admit_relative_motion(
        &mut self,
        context: InputContext,
        delta: Vector2,
        unit: RelativeMotionUnit,
    ) -> Result<(), InputError> {
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::RelativeMotion { delta, unit },
        ))?;
        self.mouse_delta.0 += delta.x;
        self.mouse_delta.1 += delta.y;
        Ok(())
    }

    pub(crate) fn handle_relative_motion(&mut self, context: InputContext, dx: f32, dy: f32) {
        let _ = self.admit_relative_motion(
            context,
            Vector2::new(dx, dy),
            RelativeMotionUnit::BackendDeviceUnits,
        );
    }

    pub fn handle_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.handle_relative_motion(LEGACY_DEVICE_CONTEXT, dx, dy);
    }

    pub(crate) fn handle_contact_input(&mut self, context: InputContext, input: &ContactInput) {
        let previous = self
            .neutral
            .contact_position_in(context, input.contact)
            .map(|position| (position.x, position.y))
            .unwrap_or((input.position.x, input.position.y));
        if self
            .admit_group(&InputObservationGroup::single(
                context,
                InputObservation::Contact(input.clone()),
            ))
            .is_err()
        {
            return;
        }

        let key = (context, input.contact);
        let accepted = match input.phase {
            NeutralContactPhase::Begin if self.primary_touch.is_none() => {
                self.primary_touch = Some(key);
                true
            }
            _ => self.primary_touch == Some(key),
        };

        if accepted {
            self.touch_samples.push(TouchInputSample {
                id: input.contact.raw(),
                phase: input.phase.into(),
                position: (input.position.x, input.position.y),
                delta: (input.position.x - previous.0, input.position.y - previous.1),
                pressure: input.pressure.and_then(legacy_pressure_projection),
            });
            if matches!(
                input.phase,
                NeutralContactPhase::End | NeutralContactPhase::Cancel
            ) {
                self.primary_touch = None;
            }
        }
    }

    pub fn handle_touch_input(
        &mut self,
        phase: TouchInputPhase,
        id: u64,
        x: f32,
        y: f32,
        pressure: Option<f32>,
    ) {
        let phase = match phase {
            TouchInputPhase::Started => NeutralContactPhase::Begin,
            TouchInputPhase::Moved => NeutralContactPhase::Update,
            TouchInputPhase::Ended => NeutralContactPhase::End,
            TouchInputPhase::Cancelled => NeutralContactPhase::Cancel,
        };
        self.handle_contact_input(
            LEGACY_WINDOW_CONTEXT,
            &ContactInput {
                contact: ContactId::new(id),
                phase,
                position: Point2::new(x, y, CoordinateSpace::UnspecifiedTargetUnits),
                pressure: pressure.map(|value| {
                    AnalogMeasurement::new(value, MeasurementDomain::UnspecifiedScalar)
                }),
                altitude_angle_radians: None,
            },
        );
    }

    pub(crate) fn admit_automation_observation(
        &mut self,
        context: InputContext,
        observation: InputObservation,
    ) -> Result<bool, InputError> {
        match observation {
            InputObservation::PointerButton(input) => {
                self.admit_pointer_button(context, input)?;
                Ok(true)
            }
            InputObservation::RelativeMotion { delta, unit } => {
                self.admit_relative_motion(context, delta, unit)?;
                Ok(true)
            }
            InputObservation::Scroll(input) => {
                self.admit_scroll_input(context, input)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn clear_frame(&mut self) {
        self.keyboard_press_samples.clear();
        self.typed_text.clear();
        self.overlay_consumed = false;
        self.mouse_delta = (0.0, 0.0);
        self.mouse_motion_samples.clear();
        self.mouse_button_transitions.clear();
        self.touch_samples.clear();
        self.device_observation_groups.clear();
        self.scroll_delta = 0.0;
        self.left_mouse_pressed = false;
        self.left_mouse_released = false;
        self.right_mouse_pressed = false;
        self.right_mouse_released = false;
        self.middle_mouse_pressed = false;
        self.middle_mouse_released = false;
    }

    pub fn left_mouse_down(&self) -> bool {
        self.button_down(PointerButton::Left)
    }

    pub fn mouse_motion_samples(&self) -> &[MouseMotionSample] {
        &self.mouse_motion_samples
    }

    pub fn mouse_button_transitions(&self) -> &[MouseButtonTransitionSample] {
        &self.mouse_button_transitions
    }

    pub fn left_mouse_pressed_transition(&self) -> Option<MouseButtonTransitionSample> {
        self.mouse_button_transitions
            .iter()
            .find(|transition| transition.is_left_pressed())
            .copied()
    }

    pub fn left_mouse_released_transition(&self) -> Option<MouseButtonTransitionSample> {
        self.mouse_button_transitions
            .iter()
            .rev()
            .find(|transition| transition.is_left_released())
            .copied()
    }

    pub fn touch_samples(&self) -> &[TouchInputSample] {
        &self.touch_samples
    }

    pub fn right_mouse_down(&self) -> bool {
        self.button_down(PointerButton::Right)
    }

    pub fn middle_mouse_down(&self) -> bool {
        self.button_down(PointerButton::Middle)
    }

    pub fn left_mouse_pressed(&self) -> bool {
        self.left_mouse_pressed
    }

    pub fn left_mouse_released(&self) -> bool {
        self.left_mouse_released
    }

    pub fn right_mouse_pressed(&self) -> bool {
        self.right_mouse_pressed
    }

    pub fn right_mouse_released(&self) -> bool {
        self.right_mouse_released
    }

    pub fn middle_mouse_pressed(&self) -> bool {
        self.middle_mouse_pressed
    }

    pub fn middle_mouse_released(&self) -> bool {
        self.middle_mouse_released
    }

    pub fn shift_down(&self) -> bool {
        self.key_down(KeyCode::ShiftLeft) || self.key_down(KeyCode::ShiftRight)
    }

    fn ctrl_down(&self) -> bool {
        self.key_down(KeyCode::ControlLeft) || self.key_down(KeyCode::ControlRight)
    }

    fn alt_down(&self) -> bool {
        self.key_down(KeyCode::AltLeft) || self.key_down(KeyCode::AltRight)
    }

    fn super_down(&self) -> bool {
        self.key_down(KeyCode::SuperLeft) || self.key_down(KeyCode::SuperRight)
    }

    pub(super) fn modifiers_snapshot(&self) -> ModifiersSnapshot {
        ModifiersSnapshot {
            shift: self.shift_down(),
            ctrl: self.ctrl_down(),
            alt: self.alt_down(),
            super_key: self.super_down(),
        }
    }

    fn key_down(&self, key: KeyCode) -> bool {
        self.neutral
            .key_down_anywhere(&physical_identity_for_key_code(key))
    }

    pub(super) fn physical_key_down(&self, key: &PhysicalKeyIdentity) -> bool {
        self.neutral.key_down_anywhere(key)
    }

    pub(super) fn keyboard_press_samples(&self) -> &[KeyboardPressSample] {
        &self.keyboard_press_samples
    }

    fn button_down(&self, button: PointerButton) -> bool {
        self.neutral.pointer_button_down_anywhere(button)
    }

    #[cfg(test)]
    pub(crate) fn neutral_touch_active(&self, id: u64) -> bool {
        self.neutral
            .contact_position_in(LEGACY_WINDOW_CONTEXT, ContactId::new(id))
            .is_some()
    }
}

fn continuity_loss_contains_context(
    context: InputContext,
    loss: ContinuityLoss,
    candidate: InputContext,
) -> bool {
    match loss {
        ContinuityLoss::Source => candidate.source == context.source,
        ContinuityLoss::Device => {
            candidate.source == context.source && candidate.device == context.device
        }
    }
}

fn legacy_pressure_projection(measurement: AnalogMeasurement) -> Option<f32> {
    let normalized = match measurement.domain {
        MeasurementDomain::CalibratedForce { .. } => {
            let max_possible_force = measurement.domain.max_possible_force()?;
            if max_possible_force <= 0.0 {
                return None;
            }
            measurement.value / max_possible_force
        }
        MeasurementDomain::UnspecifiedScalar | MeasurementDomain::NormalizedUnitInterval => {
            measurement.value
        }
        MeasurementDomain::SignedNormalizedUnitInterval => measurement.value,
        MeasurementDomain::Bounded { min, max } | MeasurementDomain::Degrees { min, max } => {
            let span = max - min;
            if span <= 0.0 {
                return None;
            }
            (measurement.value - min) / span
        }
    };
    normalized.is_finite().then(|| normalized.clamp(0.0, 1.0))
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct ModifiersSnapshot {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) super_key: bool,
}
