use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputSourceId(u64);

impl InputSourceId {
    /// Creates a runtime/session-scoped source identity. This value is not persistent identity.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputDeviceId(u64);

impl InputDeviceId {
    /// Creates a runtime/session-scoped device identity. This value is not persistent identity.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputContext {
    pub source: InputSourceId,
    pub device: Option<InputDeviceId>,
}

impl InputContext {
    pub const fn new(source: InputSourceId, device: Option<InputDeviceId>) -> Self {
        Self { source, device }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ControlId(u64);

impl ControlId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ContactId(u64);

impl ContactId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SourceSequence(u64);

impl SourceSequence {
    fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("input source sequence exhausted"),
        )
    }

    #[cfg(test)]
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct AdmissionSequence(u64);

impl AdmissionSequence {
    fn next(self) -> Self {
        Self(
            self.0
                .checked_add(1)
                .expect("input admission sequence exhausted"),
        )
    }

    #[cfg(test)]
    pub(crate) const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitalState {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationOrigin {
    SourceReport,
    BackendSyntheticReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DigitalTransition {
    Down,
    ReconcileDown,
    Up,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactPhase {
    Begin,
    Update,
    End,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyLocation {
    Standard,
    Left,
    Right,
    Numpad,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NativePhysicalKeyCode {
    Unidentified,
    Android(u32),
    MacOs(u32),
    Windows(u32),
    Xkb(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicalKeyIdentity {
    Code(String),
    Native(NativePhysicalKeyCode),
}

impl PhysicalKeyIdentity {
    pub fn code(value: impl Into<String>) -> Self {
        Self::Code(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeLogicalKey {
    Unidentified,
    Android(u32),
    MacOs(u32),
    Windows(u32),
    Xkb(u32),
    Web(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalKey {
    Named(String),
    Character(String),
    Native(NativeLogicalKey),
    Dead(Option<char>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardInput {
    pub physical_key: PhysicalKeyIdentity,
    pub logical_key: LogicalKey,
    pub location: KeyLocation,
    pub state: DigitalState,
    pub repeat: bool,
    pub origin: ObservationOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerButtonInput {
    pub button: PointerButton,
    pub state: DigitalState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSpace {
    LegacyWindowPhysicalPixels,
    WindowPhysicalPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelativeMotionUnit {
    BackendDeviceUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDomain {
    LegacyVerticalScalarUnknown,
    Lines,
    WindowPhysicalPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollPhase {
    Begin,
    Update,
    End,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MeasurementDomain {
    LegacyPressureScalar,
    NormalizedUnitInterval,
    CalibratedForce { max_possible_force: f32 },
}

impl MeasurementDomain {
    pub(crate) fn calibrated_force(max_possible_force: f64) -> Self {
        Self::CalibratedForce {
            max_possible_force: max_possible_force as f32,
        }
    }

    pub fn max_possible_force(self) -> Option<f32> {
        match self {
            Self::CalibratedForce { max_possible_force } => Some(max_possible_force),
            Self::LegacyPressureScalar | Self::NormalizedUnitInterval => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point2 {
    pub x: f32,
    pub y: f32,
    pub space: CoordinateSpace,
}

impl Point2 {
    pub const fn new(x: f32, y: f32, space: CoordinateSpace) -> Self {
        Self { x, y, space }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Vector2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Vector2 {
    pub(crate) const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollDelta {
    pub horizontal: Option<f32>,
    pub vertical: Option<f32>,
}

impl ScrollDelta {
    pub(crate) const fn legacy_vertical(vertical: f32) -> Self {
        Self {
            horizontal: None,
            vertical: Some(vertical),
        }
    }

    pub(crate) const fn two_dimensional(horizontal: f32, vertical: f32) -> Self {
        Self {
            horizontal: Some(horizontal),
            vertical: Some(vertical),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollInput {
    pub delta: ScrollDelta,
    pub domain: ScrollDomain,
    pub phase: Option<ScrollPhase>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnalogMeasurement {
    pub value: f32,
    pub domain: MeasurementDomain,
}

impl AnalogMeasurement {
    pub(crate) const fn new(value: f32, domain: MeasurementDomain) -> Self {
        Self { value, domain }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContactInput {
    pub id: u64,
    pub phase: ContactPhase,
    pub position: Point2,
    pub pressure: Option<AnalogMeasurement>,
    pub altitude_angle_radians: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum InputObservation {
    DigitalControl {
        control: ControlId,
        transition: DigitalTransition,
    },
    AbsolutePointerPosition {
        position: Point2,
    },
    RelativeMotion {
        delta: Vector2,
        unit: RelativeMotionUnit,
    },
    Scroll {
        delta: ScrollDelta,
        domain: ScrollDomain,
        phase: Option<ScrollPhase>,
    },
    Contact {
        contact: ContactId,
        phase: ContactPhase,
        position: Point2,
        pressure: Option<AnalogMeasurement>,
        altitude_angle_radians: Option<f32>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ObservationGroup {
    context: InputContext,
    observations: Vec<InputObservation>,
}

impl ObservationGroup {
    pub(crate) fn new_in(context: InputContext, observations: Vec<InputObservation>) -> Self {
        Self {
            context,
            observations,
        }
    }

    pub(crate) fn single_in(context: InputContext, observation: InputObservation) -> Self {
        Self::new_in(context, vec![observation])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ContactState {
    pub(crate) position: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NeutralInputError {
    NonFiniteObservation,
}

#[derive(Debug, Default)]
struct NeutralInputState {
    held_controls: HashSet<(InputSourceId, Option<InputDeviceId>, ControlId)>,
    contacts: HashMap<(InputSourceId, Option<InputDeviceId>, ContactId), ContactState>,
    absolute_pointer_positions: HashMap<InputSourceId, Point2>,
}

#[derive(Debug, Default)]
pub(crate) struct NeutralInputAuthority {
    state: NeutralInputState,
    source_sequences: HashMap<InputSourceId, SourceSequence>,
    admission_sequence: AdmissionSequence,
}

impl NeutralInputAuthority {
    pub(crate) fn admit(&mut self, group: ObservationGroup) -> Result<(), NeutralInputError> {
        if group
            .observations
            .iter()
            .any(|observation| !is_finite(observation))
        {
            return Err(NeutralInputError::NonFiniteObservation);
        }

        let source_sequence = self
            .source_sequences
            .entry(group.context.source)
            .or_default();
        *source_sequence = source_sequence.next();
        self.admission_sequence = self.admission_sequence.next();

        for observation in group.observations {
            self.apply(group.context, observation);
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn control_down_in(&self, context: InputContext, control: ControlId) -> bool {
        self.state
            .held_controls
            .contains(&(context.source, context.device, control))
    }

    pub(crate) fn control_down_anywhere(&self, control: ControlId) -> bool {
        self.state
            .held_controls
            .iter()
            .any(|(_, _, candidate)| *candidate == control)
    }

    pub(crate) fn absolute_pointer_position(&self, source: InputSourceId) -> Option<Point2> {
        self.state.absolute_pointer_positions.get(&source).copied()
    }

    pub(crate) fn contact_state_in(
        &self,
        context: InputContext,
        contact: ContactId,
    ) -> Option<ContactState> {
        self.state
            .contacts
            .get(&(context.source, context.device, contact))
            .copied()
    }

    #[cfg(test)]
    pub(crate) fn active_contact_count(&self, source: InputSourceId) -> usize {
        self.state
            .contacts
            .keys()
            .filter(|(candidate, _, _)| *candidate == source)
            .count()
    }

    #[cfg(test)]
    fn source_sequence(&self, source: InputSourceId) -> Option<SourceSequence> {
        self.source_sequences.get(&source).copied()
    }

    #[cfg(test)]
    fn admission_sequence(&self) -> AdmissionSequence {
        self.admission_sequence
    }

    fn apply(&mut self, context: InputContext, observation: InputObservation) {
        match observation {
            InputObservation::DigitalControl {
                control,
                transition,
            } => match transition {
                DigitalTransition::Down | DigitalTransition::ReconcileDown => {
                    self.state
                        .held_controls
                        .insert((context.source, context.device, control));
                }
                DigitalTransition::Up | DigitalTransition::Cancel => {
                    self.state
                        .held_controls
                        .remove(&(context.source, context.device, control));
                }
            },
            InputObservation::AbsolutePointerPosition { position } => {
                self.state
                    .absolute_pointer_positions
                    .insert(context.source, position);
            }
            InputObservation::RelativeMotion { .. } | InputObservation::Scroll { .. } => {}
            InputObservation::Contact {
                contact,
                phase,
                position,
                ..
            } => match phase {
                ContactPhase::Begin | ContactPhase::Update => {
                    self.state.contacts.insert(
                        (context.source, context.device, contact),
                        ContactState { position },
                    );
                }
                ContactPhase::End | ContactPhase::Cancel => {
                    self.state
                        .contacts
                        .remove(&(context.source, context.device, contact));
                }
            },
        }
    }
}

fn is_finite(observation: &InputObservation) -> bool {
    match observation {
        InputObservation::DigitalControl { .. } => true,
        InputObservation::AbsolutePointerPosition { position } => point_is_finite(*position),
        InputObservation::RelativeMotion { delta, .. } => {
            delta.x.is_finite() && delta.y.is_finite()
        }
        InputObservation::Scroll { delta, .. } => {
            delta.horizontal.is_none_or(|value| value.is_finite())
                && delta.vertical.is_none_or(|value| value.is_finite())
        }
        InputObservation::Contact {
            position,
            pressure,
            altitude_angle_radians,
            ..
        } => {
            point_is_finite(*position)
                && pressure.is_none_or(measurement_is_finite)
                && altitude_angle_radians.is_none_or(|value| value.is_finite())
        }
    }
}

fn measurement_is_finite(measurement: AnalogMeasurement) -> bool {
    measurement.value.is_finite()
        && measurement
            .domain
            .max_possible_force()
            .is_none_or(f32::is_finite)
}

fn point_is_finite(point: Point2) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_A: InputSourceId = InputSourceId::new(1);
    const SOURCE_B: InputSourceId = InputSourceId::new(2);
    const CONTEXT_A: InputContext = InputContext::new(SOURCE_A, None);
    const CONTEXT_B: InputContext = InputContext::new(SOURCE_B, None);
    const CONTROL: ControlId = ControlId::new(7);

    #[test]
    fn source_and_admission_sequences_are_distinct_and_deterministic() {
        let mut authority = NeutralInputAuthority::default();

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Down,
                },
            ))
            .expect("first source-A observation should admit");
        assert_eq!(authority.source_sequence(SOURCE_A).unwrap().get(), 1);
        assert_eq!(authority.admission_sequence().get(), 1);

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_B,
                InputObservation::RelativeMotion {
                    delta: Vector2::new(1.0, -1.0),
                    unit: RelativeMotionUnit::BackendDeviceUnits,
                },
            ))
            .expect("source-B observation should admit");
        assert_eq!(authority.source_sequence(SOURCE_B).unwrap().get(), 1);
        assert_eq!(authority.admission_sequence().get(), 2);

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Up,
                },
            ))
            .expect("second source-A observation should admit");
        assert_eq!(authority.source_sequence(SOURCE_A).unwrap().get(), 2);
        assert_eq!(authority.admission_sequence().get(), 3);
    }

    #[test]
    fn down_then_up_remain_two_admissions_even_when_final_state_matches_initial() {
        let mut authority = NeutralInputAuthority::default();

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Down,
                },
            ))
            .expect("down should admit");
        assert!(authority.control_down_in(CONTEXT_A, CONTROL));
        assert_eq!(authority.admission_sequence().get(), 1);

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Up,
                },
            ))
            .expect("up should admit");
        assert!(!authority.control_down_in(CONTEXT_A, CONTROL));
        assert_eq!(authority.admission_sequence().get(), 2);
    }

    #[test]
    fn same_control_on_distinct_devices_does_not_alias() {
        let mut authority = NeutralInputAuthority::default();
        let device_a = InputDeviceId::new(1);
        let device_b = InputDeviceId::new(2);
        let context_a = InputContext::new(SOURCE_A, Some(device_a));
        let context_b = InputContext::new(SOURCE_A, Some(device_b));

        authority
            .admit(ObservationGroup::single_in(
                context_a,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Down,
                },
            ))
            .expect("device-A control should admit");

        assert!(authority.control_down_in(context_a, CONTROL));
        assert!(!authority.control_down_in(context_b, CONTROL));
    }

    #[test]
    fn same_contact_id_on_distinct_contexts_does_not_alias() {
        let mut authority = NeutralInputAuthority::default();
        let contact = ContactId::new(9);
        let context_a = InputContext::new(SOURCE_A, Some(InputDeviceId::new(1)));
        let context_b = InputContext::new(SOURCE_B, Some(InputDeviceId::new(2)));
        let position = Point2::new(4.0, 5.0, CoordinateSpace::WindowPhysicalPixels);

        authority
            .admit(ObservationGroup::single_in(
                context_a,
                InputObservation::Contact {
                    contact,
                    phase: ContactPhase::Begin,
                    position,
                    pressure: None,
                    altitude_angle_radians: None,
                },
            ))
            .expect("context-A contact should admit");
        authority
            .admit(ObservationGroup::single_in(
                context_b,
                InputObservation::Contact {
                    contact,
                    phase: ContactPhase::Begin,
                    position,
                    pressure: None,
                    altitude_angle_radians: None,
                },
            ))
            .expect("context-B contact should admit");

        assert!(authority.contact_state_in(context_a, contact).is_some());
        assert!(authority.contact_state_in(context_b, contact).is_some());
    }

    #[test]
    fn reconciliation_changes_confirmed_state_without_an_ordinary_edge() {
        let mut authority = NeutralInputAuthority::default();

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::ReconcileDown,
                },
            ))
            .expect("reconciliation down should admit");
        assert!(authority.control_down_in(CONTEXT_A, CONTROL));

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Cancel,
                },
            ))
            .expect("cancel should admit");
        assert!(!authority.control_down_in(CONTEXT_A, CONTROL));
    }

    #[test]
    fn legacy_scalar_scroll_keeps_horizontal_absent_not_measured_zero() {
        let delta = ScrollDelta::legacy_vertical(0.0);

        assert_eq!(delta.horizontal, None);
        assert_eq!(delta.vertical, Some(0.0));
        assert_ne!(
            delta,
            ScrollDelta {
                horizontal: Some(0.0),
                vertical: Some(0.0),
            }
        );
    }

    #[test]
    fn omitted_pressure_remains_distinct_from_measured_zero() {
        let position = Point2::new(10.0, 12.0, CoordinateSpace::LegacyWindowPhysicalPixels);
        let omitted = InputObservation::Contact {
            contact: ContactId::new(9),
            phase: ContactPhase::Update,
            position,
            pressure: None,
            altitude_angle_radians: None,
        };
        let measured_zero = InputObservation::Contact {
            contact: ContactId::new(9),
            phase: ContactPhase::Update,
            position,
            pressure: Some(AnalogMeasurement::new(
                0.0,
                MeasurementDomain::LegacyPressureScalar,
            )),
            altitude_angle_radians: None,
        };

        assert_ne!(omitted, measured_zero);
    }

    #[test]
    fn invalid_numeric_group_is_rejected_atomically() {
        let mut authority = NeutralInputAuthority::default();

        let result = authority.admit(ObservationGroup::new_in(
            CONTEXT_A,
            vec![
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Down,
                },
                InputObservation::RelativeMotion {
                    delta: Vector2::new(f32::NAN, 0.0),
                    unit: RelativeMotionUnit::BackendDeviceUnits,
                },
            ],
        ));

        assert_eq!(result, Err(NeutralInputError::NonFiniteObservation));
        assert!(!authority.control_down_in(CONTEXT_A, CONTROL));
        assert_eq!(authority.admission_sequence().get(), 0);
    }
}
