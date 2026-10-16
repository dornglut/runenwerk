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
    // The exact physical source which originated this Down transition.
    pub(super) context: InputContext,
    pub(super) physical_key: PhysicalKeyIdentity,
    /// Non-repeating identity of this exact physical Down generation.
    pub(super) generation: u64,
    pub(super) modifiers: ModifiersSnapshot,
    /// Exact source-local modifier identities visible at this Down's prefix.
    /// The held-state index may change before delayed M13 disposition arrives.
    pub(super) modifier_lifetimes: Vec<KeyboardModifierLifetime>,
    /// Source/device continuity loss invalidates even an already-released
    /// Down's pending frame press, without fabricating an Up.
    pub(super) retired_by_continuity: bool,
}

/// One press-time physical modifier, carrying its exact genuine Down
/// generation when known. Synthetic restoration never mints a generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KeyboardModifierLifetime {
    pub(super) physical_key: PhysicalKeyIdentity,
    pub(super) generation: Option<u64>,
    pub(super) generationless_game_eligible: bool,
}

/// An authentic physical Up report, recorded for ActionState's same-source
/// denial retirement. Reconciliation/continuity-loss are never release proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KeyboardRealReleaseSample {
    pub(super) context: InputContext,
    pub(super) physical_key: PhysicalKeyIdentity,
}

/// A derived index of held keyboard contributors, never a replacement for
/// RunenInput's authoritative source/device-scoped physical state.
#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyboardHeldContributor {
    context: InputContext,
    physical_key: PhysicalKeyIdentity,
    generation: Option<u64>,
    /// An identity-less synthetic held state is eligible only if admitted
    /// before genuine physical Down generation exhaustion.
    generationless_game_eligible: bool,
}

#[derive(Debug, runen_ecs::Component, runen_ecs::Resource)]
pub struct InputState {
    neutral: NeutralInputState,
    keyboard_press_samples: Vec<KeyboardPressSample>,
    keyboard_real_release_samples: Vec<KeyboardRealReleaseSample>,
    keyboard_held_contributors: Vec<KeyboardHeldContributor>,
    /// Exhaustion disables new game press identities instead of wrapping.
    next_keyboard_generation: Option<u64>,
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
            keyboard_real_release_samples: Vec::new(),
            keyboard_held_contributors: Vec::new(),
            next_keyboard_generation: Some(1),
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
        // The neutral reducer remains the sole physical-state authority. Its
        // atomic validation/admission must succeed before any product projection.
        self.neutral.admit(group)?;
        for observation in &group.observations {
            match observation {
                InputObservation::Keyboard(input) => {
                    self.observe_keyboard_contributor(group.context, input);
                }
                InputObservation::ContinuityLoss(loss) => {
                    self.keyboard_held_contributors.retain(|contributor| {
                        !continuity_loss_contains_context(group.context, *loss, contributor.context)
                    });
                    for press in &mut self.keyboard_press_samples {
                        if continuity_loss_contains_context(group.context, *loss, press.context) {
                            press.retired_by_continuity = true;
                        }
                    }
                }
                _ => {}
            }
        }
        if self.admitted_input_capture_active {
            self.captured_admitted_groups.push(group.clone());
        }
        Ok(())
    }

    fn observe_keyboard_contributor(&mut self, context: InputContext, input: &KeyboardInput) {
        let held_index = self
            .keyboard_held_contributors
            .iter()
            .position(|contributor| {
                contributor.context == context && contributor.physical_key == input.physical_key
            });
        match (input.state, held_index) {
            (DigitalState::Pressed, None) => {
                // Synthetic/repeated restoration may update neutral physical
                // truth but MUST NOT mint a fresh gameplay press generation.
                let genuine = input.origin == ObservationOrigin::SourceReport && !input.repeat;
                let generation = if genuine {
                    let issued = self.next_keyboard_generation;
                    self.next_keyboard_generation =
                        issued.and_then(|current| current.checked_add(1));
                    issued
                } else {
                    None
                };
                self.keyboard_held_contributors
                    .push(KeyboardHeldContributor {
                        context,
                        physical_key: input.physical_key.clone(),
                        generation,
                        generationless_game_eligible: !genuine
                            && self.next_keyboard_generation.is_some(),
                    });
                if let Some(generation) = generation {
                    self.keyboard_press_samples.push(KeyboardPressSample {
                        context,
                        physical_key: input.physical_key.clone(),
                        generation,
                        modifiers: self.source_modifiers_from_held_index(context),
                        modifier_lifetimes: self.source_modifier_lifetimes(context),
                        retired_by_continuity: false,
                    });
                }
            }
            (DigitalState::Released, Some(index)) => {
                self.keyboard_held_contributors.remove(index);
                if input.origin == ObservationOrigin::SourceReport {
                    self.keyboard_real_release_samples
                        .push(KeyboardRealReleaseSample {
                            context,
                            physical_key: input.physical_key.clone(),
                        });
                }
            }
            (DigitalState::Released, None) if input.origin == ObservationOrigin::SourceReport => {
                // A genuine Up may follow an OS synthetic cancellation. That
                // is the first authentic release proof for a revoked lifetime.
                self.keyboard_real_release_samples
                    .push(KeyboardRealReleaseSample {
                        context,
                        physical_key: input.physical_key.clone(),
                    });
            }
            _ => {}
        }
    }

    fn source_modifiers_from_held_index(&self, context: InputContext) -> ModifiersSnapshot {
        // The admission group may have several chronological keyboard edges.
        // The neutral reducer has already committed the whole group, so use
        // the derived prefix index for exact press-time modifier snapshots.
        let down = |key| {
            let key = physical_identity_for_key_code(key);
            self.keyboard_held_contributors.iter().any(|contributor| {
                contributor.context == context && contributor.physical_key == key
            })
        };
        ModifiersSnapshot {
            shift: down(KeyCode::ShiftLeft) || down(KeyCode::ShiftRight),
            ctrl: down(KeyCode::ControlLeft) || down(KeyCode::ControlRight),
            alt: down(KeyCode::AltLeft) || down(KeyCode::AltRight),
            super_key: down(KeyCode::SuperLeft) || down(KeyCode::SuperRight),
        }
    }

    fn source_modifier_lifetimes(&self, context: InputContext) -> Vec<KeyboardModifierLifetime> {
        let keys = [
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::ControlLeft,
            KeyCode::ControlRight,
            KeyCode::AltLeft,
            KeyCode::AltRight,
            KeyCode::SuperLeft,
            KeyCode::SuperRight,
        ];
        self.keyboard_held_contributors
            .iter()
            .filter(|held| {
                held.context == context
                    && keys
                        .iter()
                        .any(|code| held.physical_key == physical_identity_for_key_code(*code))
            })
            .map(|held| KeyboardModifierLifetime {
                physical_key: held.physical_key.clone(),
                generation: held.generation,
                generationless_game_eligible: held.generationless_game_eligible,
            })
            .collect()
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

    pub(crate) fn absolute_pointer_position_for_source(
        &self,
        source: InputSourceId,
    ) -> Option<Point2> {
        self.neutral.absolute_pointer_position(source)
    }

    pub(crate) fn frame_projection_is_quiescent(&self) -> bool {
        self.keyboard_press_samples.is_empty()
            && self.keyboard_real_release_samples.is_empty()
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
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::Keyboard(input.clone()),
        ))
        .expect("digital keyboard observation should always be valid");
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

    fn admit_cursor_position(
        &mut self,
        context: InputContext,
        position: Point2,
    ) -> Result<(), InputError> {
        let previous = self
            .neutral
            .absolute_pointer_position(context.source)
            .map(|point| (point.x, point.y))
            .unwrap_or((0.0, 0.0));
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::AbsolutePointerPosition { position },
        ))?;

        self.mouse_position = (position.x, position.y);
        self.mouse_motion_samples.push(MouseMotionSample {
            position: (position.x, position.y),
            delta: (position.x - previous.0, position.y - previous.1),
        });
        Ok(())
    }

    pub(crate) fn handle_cursor_position(&mut self, context: InputContext, position: Point2) {
        let _ = self.admit_cursor_position(context, position);
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
        self.admit_relative_motion_with_legacy_projection(context, delta, unit, true)
    }

    fn admit_relative_motion_with_legacy_projection(
        &mut self,
        context: InputContext,
        delta: Vector2,
        unit: RelativeMotionUnit,
        project_legacy: bool,
    ) -> Result<(), InputError> {
        // Raw device observations are physical truth even when the native host
        // cannot attribute them to a permitted gameplay window. Do not turn
        // an unattributed observation into a source-erased camera command.
        self.admit_group(&InputObservationGroup::single(
            context,
            InputObservation::RelativeMotion { delta, unit },
        ))?;
        if project_legacy {
            self.mouse_delta.0 += delta.x;
            self.mouse_delta.1 += delta.y;
        }
        Ok(())
    }

    pub(crate) fn handle_unattributed_relative_motion(
        &mut self,
        context: InputContext,
        dx: f32,
        dy: f32,
        project_legacy: bool,
    ) {
        let _ = self.admit_relative_motion_with_legacy_projection(
            context,
            Vector2::new(dx, dy),
            RelativeMotionUnit::BackendDeviceUnits,
            project_legacy,
        );
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
            InputObservation::AbsolutePointerPosition { position } => {
                self.admit_cursor_position(context, position)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    pub fn clear_frame(&mut self) {
        self.keyboard_press_samples.clear();
        self.keyboard_real_release_samples.clear();
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

    fn key_down(&self, key: KeyCode) -> bool {
        self.neutral
            .key_down_anywhere(&physical_identity_for_key_code(key))
    }

    #[cfg(test)]
    pub(super) fn physical_key_down(&self, key: &PhysicalKeyIdentity) -> bool {
        self.neutral.key_down_anywhere(key)
    }

    pub(super) fn physical_key_down_in(
        &self,
        context: InputContext,
        key: &PhysicalKeyIdentity,
    ) -> bool {
        self.neutral.key_down_in(context, key)
    }

    pub(super) fn keyboard_held_contexts(&self) -> impl Iterator<Item = InputContext> + '_ {
        self.keyboard_held_contributors
            .iter()
            .map(|contributor| contributor.context)
    }

    pub(super) fn keyboard_press_samples(&self) -> &[KeyboardPressSample] {
        &self.keyboard_press_samples
    }

    pub(super) fn keyboard_real_release_samples(&self) -> &[KeyboardRealReleaseSample] {
        &self.keyboard_real_release_samples
    }

    /// Neutral physical state does not confer eligibility on a genuine Down
    /// for which the Engine's non-wrapping generation space is exhausted.
    pub(super) fn keyboard_held_contributor_game_eligible_in(
        &self,
        context: InputContext,
        key: &PhysicalKeyIdentity,
    ) -> bool {
        self.keyboard_held_contributors
            .iter()
            .find(|held| held.context == context && &held.physical_key == key)
            .is_some_and(|held| held.generation.is_some() || held.generationless_game_eligible)
    }

    /// Return the exact physical Down generation, never an inferred key press
    /// from RunenUI focus, another device or a synthetic reconciliation event.
    pub fn keyboard_down_generation_in(
        &self,
        context: InputContext,
        key: &PhysicalKeyIdentity,
    ) -> Option<u64> {
        self.keyboard_held_contributors
            .iter()
            .find(|held| held.context == context && &held.physical_key == key)
            .and_then(|held| held.generation)
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

#[cfg(test)]
mod generation_exhaustion_tests {
    use super::*;
    use crate::plugins::{ActionState, action};
    use winit::event::ElementState;
    use winit::keyboard::KeyCode;

    #[test]
    fn exhausted_genuine_down_and_synthetic_restoration_cannot_create_game_held_state() {
        let mut input = InputState::new();
        let mut actions = ActionState::new();
        input.next_keyboard_generation = Some(u64::MAX);

        // The final issued identity remains a legitimate physical Down.
        input.handle_keyboard_input(KeyCode::KeyW, ElementState::Pressed, None);
        actions.project(&input);
        assert!(actions.action_pressed(action::WORLD_MOVE_UP));
        assert!(actions.action_down(action::WORLD_MOVE_UP));
        assert!(input.next_keyboard_generation.is_none());

        // The neutral reducer keeps physical truth, but exhausted identity
        // cannot be mistaken for a held or pressed game permission.
        input.handle_keyboard_input(KeyCode::KeyA, ElementState::Pressed, None);
        actions.project(&input);
        assert!(input.physical_key_down(&physical_identity_for_key_code(KeyCode::KeyA)));
        assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));
        assert!(!actions.action_down(action::WORLD_MOVE_LEFT));
        assert!(actions.action_down(action::WORLD_MOVE_UP));

        input.handle_keyboard_reconciliation(KeyCode::KeyA, ElementState::Released);
        input.handle_keyboard_reconciliation(KeyCode::KeyA, ElementState::Pressed);
        actions.project(&input);
        assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));
        assert!(!actions.action_down(action::WORLD_MOVE_LEFT));

        input.handle_keyboard_input(KeyCode::KeyA, ElementState::Released, None);
        input.handle_keyboard_input(KeyCode::KeyA, ElementState::Pressed, None);
        actions.project(&input);
        assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));
        assert!(!actions.action_down(action::WORLD_MOVE_LEFT));
    }

    #[test]
    fn pre_exhaustion_synthetic_legacy_hold_remains_held_without_new_press() {
        let mut input = InputState::new();
        let mut actions = ActionState::new();
        input.next_keyboard_generation = Some(u64::MAX);
        input.handle_keyboard_reconciliation(KeyCode::KeyA, ElementState::Pressed);
        actions.project(&input);
        assert!(actions.action_down(action::WORLD_MOVE_LEFT));
        assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));

        input.handle_keyboard_input(KeyCode::KeyW, ElementState::Pressed, None);
        actions.project(&input);
        assert!(input.next_keyboard_generation.is_none());
        assert!(actions.action_down(action::WORLD_MOVE_LEFT));
        assert!(!actions.action_pressed(action::WORLD_MOVE_LEFT));
    }
}
