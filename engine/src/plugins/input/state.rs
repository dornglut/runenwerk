use super::neutral::{
    AnalogMeasurement, ContactId, ContactInput, ContactPhase as NeutralContactPhase, ControlId,
    CoordinateSpace, DigitalState, DigitalTransition, InputContext, InputObservation,
    InputSourceId, KeyLocation, KeyboardInput, LogicalKey, MeasurementDomain, NativeLogicalKey,
    NeutralInputAuthority, ObservationGroup, ObservationOrigin, PhysicalKeyIdentity, Point2,
    PointerButton, PointerButtonInput, RelativeMotionUnit, ScrollDelta, ScrollDomain, ScrollInput,
    Vector2,
};
use crate::plugins::{
    InputBindingChange, InputBindingChangeResult, InputBindings, KeyChord, action,
};
use std::collections::{HashMap, HashSet};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::KeyCode;

const LEGACY_WINDOW_SOURCE: InputSourceId = InputSourceId::new(1);
const LEGACY_DEVICE_SOURCE: InputSourceId = InputSourceId::new(2);
const LEGACY_WINDOW_CONTEXT: InputContext = InputContext::new(LEGACY_WINDOW_SOURCE, None);
const LEGACY_DEVICE_CONTEXT: InputContext = InputContext::new(LEGACY_DEVICE_SOURCE, None);

#[derive(Debug, Default)]
struct LegacyControlInterner {
    next_control: u64,
    keys: HashMap<PhysicalKeyIdentity, ControlId>,
    buttons: HashMap<PointerButton, ControlId>,
}

impl LegacyControlInterner {
    fn intern_key(&mut self, key: &PhysicalKeyIdentity) -> ControlId {
        if let Some(control) = self.keys.get(key).copied() {
            return control;
        }
        let control = self.next_control();
        self.keys.insert(key.clone(), control);
        control
    }

    fn key(&self, key: &PhysicalKeyIdentity) -> Option<ControlId> {
        self.keys.get(key).copied()
    }

    fn key_code(&self, key: KeyCode) -> Option<ControlId> {
        self.key(&physical_identity_for_key_code(key))
    }

    fn intern_button(&mut self, button: PointerButton) -> ControlId {
        if let Some(control) = self.buttons.get(&button).copied() {
            return control;
        }
        let control = self.next_control();
        self.buttons.insert(button, control);
        control
    }

    fn button(&self, button: PointerButton) -> Option<ControlId> {
        self.buttons.get(&button).copied()
    }

    fn next_control(&mut self) -> ControlId {
        self.next_control = self
            .next_control
            .checked_add(1)
            .expect("legacy input control identity exhausted");
        ControlId::new(self.next_control)
    }
}

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

#[derive(Debug, runen_ecs::Component, runen_ecs::Resource)]
pub struct InputState {
    neutral: NeutralInputAuthority,
    controls: LegacyControlInterner,
    bindings: InputBindings,
    actions_down: HashSet<String>,
    actions_pressed: HashSet<String>,
    pub typed_text: String,
    pub submitted: bool,
    pub insert_newline: bool,
    pub backspace: bool,
    pub delete: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub move_home: bool,
    pub move_end: bool,
    pub page_up: bool,
    pub page_down: bool,
    pub world_move_left: bool,
    pub world_move_right: bool,
    pub world_move_up: bool,
    pub world_move_down: bool,
    pub toggle_pause_menu: bool,
    pub toggle_ui_editor_mode: bool,
    pub save_ui_template: bool,
    pub editor_hide_selected: bool,
    pub editor_restore_all: bool,
    pub scene_next: bool,
    pub scene_prev: bool,
    pub scene_console: bool,
    pub scene_hud: bool,
    pub scene_overlay_push: bool,
    pub scene_overlay_pop: bool,
    pub overlay_consumed: bool,
    pub mouse_delta: (f32, f32),
    pub mouse_position: (f32, f32),
    mouse_motion_samples: Vec<MouseMotionSample>,
    mouse_button_transitions: Vec<MouseButtonTransitionSample>,
    touch_samples: Vec<TouchInputSample>,
    primary_touch: Option<(InputContext, u64)>,
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
            neutral: NeutralInputAuthority::default(),
            controls: LegacyControlInterner::default(),
            bindings: InputBindings::with_default_bindings(),
            actions_down: HashSet::new(),
            actions_pressed: HashSet::new(),
            typed_text: String::new(),
            submitted: false,
            insert_newline: false,
            backspace: false,
            delete: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            move_home: false,
            move_end: false,
            page_up: false,
            page_down: false,
            world_move_left: false,
            world_move_right: false,
            world_move_up: false,
            world_move_down: false,
            toggle_pause_menu: false,
            toggle_ui_editor_mode: false,
            save_ui_template: false,
            editor_hide_selected: false,
            editor_restore_all: false,
            scene_next: false,
            scene_prev: false,
            scene_console: false,
            scene_hud: false,
            scene_overlay_push: false,
            scene_overlay_pop: false,
            overlay_consumed: false,
            mouse_delta: (0.0, 0.0),
            mouse_position: (0.0, 0.0),
            mouse_motion_samples: Vec::new(),
            mouse_button_transitions: Vec::new(),
            touch_samples: Vec::new(),
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

    pub fn bindings(&self) -> &InputBindings {
        &self.bindings
    }

    pub fn set_bindings(&mut self, bindings: InputBindings) {
        self.bindings = bindings;
        self.actions_pressed.clear();
        self.refresh_action_state_from_bindings();
    }

    pub fn reset_default_bindings(&mut self) {
        self.set_bindings(InputBindings::with_default_bindings());
    }

    pub fn map_key(&mut self, action: impl Into<String>, key: KeyCode) {
        self.map_chord(action, KeyChord::new(key));
    }

    pub fn map_chord(&mut self, action: impl Into<String>, chord: KeyChord) {
        if self.bindings.map_chord(action, chord) {
            self.refresh_action_state_from_bindings();
        }
    }

    pub fn unmap_key(&mut self, action: &str, key: KeyCode) -> usize {
        let removed = self.bindings.unmap_key(action, key);
        if removed > 0 {
            self.refresh_action_state_from_bindings();
        }
        removed
    }

    pub fn unmap_chord(&mut self, action: &str, chord: KeyChord) -> bool {
        let removed = self.bindings.unmap_chord(action, chord);
        if removed {
            self.refresh_action_state_from_bindings();
        }
        removed
    }

    pub fn clear_action_bindings(&mut self, action: &str) -> bool {
        let removed = self.bindings.clear_action(action);
        if removed {
            self.refresh_action_state_from_bindings();
        }
        removed
    }

    pub fn apply_binding_change(&mut self, change: InputBindingChange) -> InputBindingChangeResult {
        if self.apply_binding_change_inner(change) {
            self.refresh_action_state_from_bindings();
            InputBindingChangeResult::Applied
        } else {
            InputBindingChangeResult::Noop
        }
    }

    pub fn apply_binding_changes<I>(&mut self, changes: I) -> usize
    where
        I: IntoIterator<Item = InputBindingChange>,
    {
        let mut applied = 0usize;
        for change in changes {
            if self.apply_binding_change_inner(change) {
                applied = applied.saturating_add(1);
            }
        }
        if applied > 0 {
            self.refresh_action_state_from_bindings();
        }
        applied
    }

    pub fn action_down(&self, action: &str) -> bool {
        self.actions_down.contains(action)
    }

    pub fn action_pressed(&self, action: &str) -> bool {
        self.actions_pressed.contains(action)
    }

    pub(crate) fn handle_normalized_keyboard(
        &mut self,
        context: InputContext,
        input: &KeyboardInput,
    ) {
        let control = self.controls.intern_key(&input.physical_key);
        let was_down_for_product = self.neutral.control_down_anywhere(control);
        let transition = match (input.origin, input.state) {
            (ObservationOrigin::SourceReport, DigitalState::Pressed) => DigitalTransition::Down,
            (ObservationOrigin::SourceReport, DigitalState::Released) => DigitalTransition::Up,
            (ObservationOrigin::BackendSyntheticReconciliation, DigitalState::Pressed) => {
                DigitalTransition::ReconcileDown
            }
            (ObservationOrigin::BackendSyntheticReconciliation, DigitalState::Released) => {
                DigitalTransition::Cancel
            }
        };
        self.neutral
            .admit(ObservationGroup::single_in(
                context,
                InputObservation::DigitalControl {
                    control,
                    transition,
                },
            ))
            .expect("digital keyboard observation should always be valid");

        self.recompute_action_down_states();
        if input.origin == ObservationOrigin::SourceReport
            && input.state == DigitalState::Pressed
            && !input.repeat
            && !was_down_for_product
        {
            self.apply_action_press_for_physical(&input.physical_key);
        } else {
            self.sync_legacy_flags();
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

    pub(crate) fn handle_scroll_input(&mut self, context: InputContext, input: ScrollInput) {
        if self
            .neutral
            .admit(ObservationGroup::single_in(
                context,
                InputObservation::Scroll {
                    delta: input.delta,
                    domain: input.domain,
                    phase: input.phase,
                },
            ))
            .is_ok()
            && let Some(vertical) = input.delta.vertical
        {
            self.scroll_delta += vertical;
        }
    }

    pub fn handle_mouse_wheel_delta(&mut self, delta: f32) {
        self.handle_scroll_input(
            LEGACY_WINDOW_CONTEXT,
            ScrollInput {
                delta: ScrollDelta::legacy_vertical(delta),
                domain: ScrollDomain::LegacyVerticalScalarUnknown,
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
            .neutral
            .admit(ObservationGroup::single_in(
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
            Point2::new(x, y, CoordinateSpace::LegacyWindowPhysicalPixels),
        );
    }

    pub(crate) fn handle_pointer_button(
        &mut self,
        context: InputContext,
        input: PointerButtonInput,
    ) {
        let control = self.controls.intern_button(input.button);
        let was_down_for_product = self.neutral.control_down_anywhere(control);
        let transition = match input.state {
            DigitalState::Pressed => DigitalTransition::Down,
            DigitalState::Released => DigitalTransition::Up,
        };
        self.neutral
            .admit(ObservationGroup::single_in(
                context,
                InputObservation::DigitalControl {
                    control,
                    transition,
                },
            ))
            .expect("digital pointer-button observation should always be valid");
        let is_down_for_product = self.neutral.control_down_anywhere(control);

        let changed = match input.state {
            DigitalState::Pressed => !was_down_for_product && is_down_for_product,
            DigitalState::Released => was_down_for_product && !is_down_for_product,
        };
        if !changed {
            return;
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

    pub(crate) fn handle_relative_motion(&mut self, context: InputContext, dx: f32, dy: f32) {
        if self
            .neutral
            .admit(ObservationGroup::single_in(
                context,
                InputObservation::RelativeMotion {
                    delta: Vector2::new(dx, dy),
                    unit: RelativeMotionUnit::BackendDeviceUnits,
                },
            ))
            .is_ok()
        {
            self.mouse_delta.0 += dx;
            self.mouse_delta.1 += dy;
        }
    }

    pub fn handle_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.handle_relative_motion(LEGACY_DEVICE_CONTEXT, dx, dy);
    }

    pub(crate) fn handle_contact_input(&mut self, context: InputContext, input: &ContactInput) {
        let contact = ContactId::new(input.id);
        let previous = self
            .neutral
            .contact_state_in(context, contact)
            .map(|state| (state.position.x, state.position.y))
            .unwrap_or((input.position.x, input.position.y));
        if self
            .neutral
            .admit(ObservationGroup::single_in(
                context,
                InputObservation::Contact {
                    contact,
                    phase: input.phase,
                    position: input.position,
                    pressure: input.pressure,
                    altitude_angle_radians: input.altitude_angle_radians,
                },
            ))
            .is_err()
        {
            return;
        }

        let key = (context, input.id);
        let accepted = match input.phase {
            NeutralContactPhase::Begin if self.primary_touch.is_none() => {
                self.primary_touch = Some(key);
                true
            }
            _ => self.primary_touch == Some(key),
        };

        if accepted {
            self.touch_samples.push(TouchInputSample {
                id: input.id,
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
                id,
                phase,
                position: Point2::new(x, y, CoordinateSpace::LegacyWindowPhysicalPixels),
                pressure: pressure.map(|value| {
                    AnalogMeasurement::new(value, MeasurementDomain::LegacyPressureScalar)
                }),
                altitude_angle_radians: None,
            },
        );
    }

    pub fn clear_frame(&mut self) {
        self.typed_text.clear();
        self.actions_pressed.clear();
        self.overlay_consumed = false;
        self.mouse_delta = (0.0, 0.0);
        self.mouse_motion_samples.clear();
        self.mouse_button_transitions.clear();
        self.touch_samples.clear();
        self.scroll_delta = 0.0;
        self.left_mouse_pressed = false;
        self.left_mouse_released = false;
        self.right_mouse_pressed = false;
        self.right_mouse_released = false;
        self.middle_mouse_pressed = false;
        self.middle_mouse_released = false;
        self.refresh_action_state_from_bindings();
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

    fn modifiers_snapshot(&self) -> ModifiersSnapshot {
        ModifiersSnapshot {
            shift: self.shift_down(),
            ctrl: self.ctrl_down(),
            alt: self.alt_down(),
            super_key: self.super_down(),
        }
    }

    fn key_down(&self, key: KeyCode) -> bool {
        self.controls
            .key_code(key)
            .is_some_and(|control| self.neutral.control_down_anywhere(control))
    }

    fn button_down(&self, button: PointerButton) -> bool {
        self.controls
            .button(button)
            .is_some_and(|control| self.neutral.control_down_anywhere(control))
    }

    #[cfg(test)]
    pub(crate) fn neutral_touch_active(&self, id: u64) -> bool {
        self.neutral
            .contact_state_in(LEGACY_WINDOW_CONTEXT, ContactId::new(id))
            .is_some()
    }

    #[cfg(test)]
    pub(crate) fn neutral_active_touch_count(&self) -> usize {
        self.neutral.active_contact_count(LEGACY_WINDOW_SOURCE)
    }

    fn apply_action_press_for_physical(&mut self, key: &PhysicalKeyIdentity) {
        let modifiers = self.modifiers_snapshot();
        let actions = self
            .bindings
            .matching_actions_for_physical_identity(key, modifiers);
        for action in actions {
            self.actions_pressed.insert(action.clone());
            self.actions_down.insert(action);
        }
        self.sync_legacy_flags();
    }

    fn apply_binding_change_inner(&mut self, change: InputBindingChange) -> bool {
        match change {
            InputBindingChange::MapKey { action, key } => self.bindings.map_key(action, key),
            InputBindingChange::MapChord { action, chord } => {
                self.bindings.map_chord(action, chord)
            }
            InputBindingChange::UnmapKey { action, key } => {
                self.bindings.unmap_key(&action, key) > 0
            }
            InputBindingChange::UnmapChord { action, chord } => {
                self.bindings.unmap_chord(&action, chord)
            }
            InputBindingChange::ClearAction { action } => self.bindings.clear_action(&action),
            InputBindingChange::ResetDefaults => {
                self.bindings = InputBindings::with_default_bindings();
                true
            }
        }
    }

    fn refresh_action_state_from_bindings(&mut self) {
        self.recompute_action_down_states();
        self.sync_legacy_flags();
    }

    pub(crate) fn recompute_action_down_states(&mut self) {
        let modifiers = self.modifiers_snapshot();
        let neutral = &self.neutral;
        let controls = &self.controls;
        let mut actions_down = HashSet::new();
        for action in self.bindings.action_ids() {
            if self.bindings.action_down(
                action,
                |key| {
                    controls
                        .key_code(key)
                        .is_some_and(|control| neutral.control_down_anywhere(control))
                },
                modifiers,
            ) {
                actions_down.insert(action.clone());
            }
        }
        self.actions_down = actions_down;
    }

    pub(crate) fn sync_legacy_flags(&mut self) {
        self.submitted = self.action_pressed(action::UI_SUBMIT);
        self.insert_newline = self.action_pressed(action::UI_INSERT_NEWLINE);
        self.backspace = self.action_pressed(action::UI_BACKSPACE);
        self.delete = self.action_pressed(action::UI_DELETE);
        self.move_left = self.action_down(action::UI_MOVE_LEFT);
        self.move_right = self.action_down(action::UI_MOVE_RIGHT);
        self.move_up = self.action_down(action::UI_MOVE_UP);
        self.move_down = self.action_down(action::UI_MOVE_DOWN);
        self.move_home = self.action_down(action::UI_MOVE_HOME);
        self.move_end = self.action_down(action::UI_MOVE_END);
        self.page_up = self.action_down(action::UI_PAGE_UP);
        self.page_down = self.action_down(action::UI_PAGE_DOWN);
        self.world_move_left = self.action_down(action::WORLD_MOVE_LEFT);
        self.world_move_right = self.action_down(action::WORLD_MOVE_RIGHT);
        self.world_move_up = self.action_down(action::WORLD_MOVE_UP);
        self.world_move_down = self.action_down(action::WORLD_MOVE_DOWN);
        self.toggle_pause_menu = self.action_pressed(action::SYSTEM_TOGGLE_PAUSE_MENU);
        self.toggle_ui_editor_mode = self.action_pressed(action::UI_TOGGLE_EDITOR_MODE);
        self.save_ui_template = self.action_pressed(action::UI_SAVE_TEMPLATE);
        self.editor_hide_selected = self.action_pressed(action::UI_EDITOR_HIDE_SELECTED);
        self.editor_restore_all = self.action_pressed(action::UI_EDITOR_RESTORE_ALL);
        self.scene_next = self.action_pressed(action::SCENE_NEXT);
        self.scene_prev = self.action_pressed(action::SCENE_PREV);
        self.scene_console = self.action_pressed(action::SCENE_CONSOLE);
        self.scene_hud = self.action_pressed(action::SCENE_HUD);
        self.scene_overlay_push = self.action_pressed(action::SCENE_OVERLAY_PUSH);
        self.scene_overlay_pop = self.action_pressed(action::SCENE_OVERLAY_POP);
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
        MeasurementDomain::LegacyPressureScalar | MeasurementDomain::NormalizedUnitInterval => {
            measurement.value
        }
    };
    normalized.is_finite().then(|| normalized.clamp(0.0, 1.0))
}

#[derive(Debug, Copy, Clone, Default)]
pub struct ModifiersSnapshot {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) super_key: bool,
}

#[cfg(test)]
mod interner_tests {
    use super::LegacyControlInterner;
    use crate::plugins::{NativePhysicalKeyCode, PhysicalKeyIdentity, PointerButton};

    #[test]
    fn distinct_physical_controls_do_not_alias() {
        let mut interner = LegacyControlInterner::default();
        let native_a =
            interner.intern_key(&PhysicalKeyIdentity::Native(NativePhysicalKeyCode::Xkb(41)));
        let native_b =
            interner.intern_key(&PhysicalKeyIdentity::Native(NativePhysicalKeyCode::Xkb(42)));
        let known = interner.intern_key(&PhysicalKeyIdentity::code("F13"));
        let button = interner.intern_button(PointerButton::Left);

        assert_ne!(native_a, native_b);
        assert_ne!(native_a, known);
        assert_ne!(native_a, button);
        assert_ne!(known, button);
    }
}
