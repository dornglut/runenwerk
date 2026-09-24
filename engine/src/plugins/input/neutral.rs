use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputSourceId(u64);

impl InputSourceId {
    /// Creates a runtime/session-scoped source identity. This value is not persistent identity.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputDeviceId(u64);

impl InputDeviceId {
    /// Creates a runtime/session-scoped device identity. This value is not persistent identity.
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
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
pub struct ToolId(u64);

impl ToolId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ControlId(u64);

impl ControlId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContactId(u64);

impl ContactId {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
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
pub enum EvidenceStatus {
    ObservedConfirmed,
    EstimatedRevisable,
    PredictedProvisional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryRole {
    OrdinaryCurrent,
    HistoricalCoalesced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTimeUnit {
    Microseconds,
    Milliseconds,
    Nanoseconds,
    NativeTicks { ticks_per_second: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceTime {
    pub context: InputContext,
    pub value: u64,
    pub unit: SourceTimeUnit,
}

impl SourceTime {
    pub const fn new(context: InputContext, value: u64, unit: SourceTimeUnit) -> Self {
        Self {
            context,
            value,
            unit,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigitalTransition {
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

#[derive(Debug, Default)]
struct ControlInterner {
    next_control: u64,
    keys: HashMap<PhysicalKeyIdentity, ControlId>,
    buttons: HashMap<PointerButton, ControlId>,
}

impl ControlInterner {
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
            .expect("neutral input control identity exhausted");
        ControlId::new(self.next_control)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateSpace {
    UnspecifiedTargetUnits,
    WindowPhysicalPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeMotionUnit {
    BackendDeviceUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDomain {
    Unspecified,
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
    UnspecifiedScalar,
    NormalizedUnitInterval,
    SignedNormalizedUnitInterval,
    CalibratedForce { max_possible_force: f32 },
    Bounded { min: f32, max: f32 },
    Degrees { min: f32, max: f32 },
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
            Self::UnspecifiedScalar
            | Self::NormalizedUnitInterval
            | Self::SignedNormalizedUnitInterval
            | Self::Bounded { .. }
            | Self::Degrees { .. } => None,
        }
    }

    pub(crate) fn accepts(self, value: f32) -> bool {
        if !value.is_finite() {
            return false;
        }
        match self {
            Self::UnspecifiedScalar => true,
            Self::NormalizedUnitInterval => (0.0..=1.0).contains(&value),
            Self::SignedNormalizedUnitInterval => (-1.0..=1.0).contains(&value),
            Self::CalibratedForce { max_possible_force } => {
                max_possible_force.is_finite()
                    && max_possible_force >= 0.0
                    && (0.0..=max_possible_force).contains(&value)
            }
            Self::Bounded { min, max } | Self::Degrees { min, max } => {
                min.is_finite() && max.is_finite() && min <= max && (min..=max).contains(&value)
            }
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
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollDelta {
    pub horizontal: Option<f32>,
    pub vertical: Option<f32>,
}

impl ScrollDelta {
    pub(crate) const fn vertical_only(vertical: f32) -> Self {
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
    pub const fn new(value: f32, domain: MeasurementDomain) -> Self {
        Self { value, domain }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputToolKind {
    Mouse,
    Pen,
    Brush,
    Marker,
    Airbrush,
    Eraser,
    Finger,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TabletCapabilities {
    pub pressure: bool,
    pub tilt: bool,
    pub twist: bool,
    pub tangential_pressure: bool,
    pub hover: bool,
    pub eraser: bool,
    pub barrel_controls: bool,
    pub historical_samples: bool,
    pub predicted_samples: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactPresence {
    Hover,
    Contact,
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PhysicalTabletControls {
    pub eraser: bool,
    pub barrel_primary: bool,
    pub barrel_secondary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StylusTilt {
    pub x_degrees: f32,
    pub y_degrees: f32,
}

impl StylusTilt {
    pub const fn new(x_degrees: f32, y_degrees: f32) -> Self {
        Self {
            x_degrees,
            y_degrees,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabletObservation {
    pub contact: ContactId,
    pub tool: Option<ToolId>,
    pub tool_kind: InputToolKind,
    pub phase: ContactPhase,
    pub presence: ContactPresence,
    pub position: Point2,
    pub delta: Vector2,
    pub pressure: Option<AnalogMeasurement>,
    pub tangential_pressure: Option<AnalogMeasurement>,
    pub tilt: Option<StylusTilt>,
    pub twist: Option<AnalogMeasurement>,
    pub controls: PhysicalTabletControls,
    pub capabilities: TabletCapabilities,
    pub source_time: Option<SourceTime>,
    pub evidence: EvidenceStatus,
    pub delivery: DeliveryRole,
    pub origin: ObservationOrigin,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContactInput {
    pub id: u64,
    pub phase: ContactPhase,
    pub position: Point2,
    pub pressure: Option<AnalogMeasurement>,
    pub altitude_angle_radians: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputObservation {
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
    Tablet(TabletObservation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObservationGroup {
    pub context: InputContext,
    pub observations: Vec<InputObservation>,
}

impl InputObservationGroup {
    pub fn new(context: InputContext, observations: Vec<InputObservation>) -> Self {
        Self {
            context,
            observations,
        }
    }

    pub fn single(context: InputContext, observation: InputObservation) -> Self {
        Self::new(context, vec![observation])
    }

    #[cfg(test)]
    pub(crate) fn new_in(context: InputContext, observations: Vec<InputObservation>) -> Self {
        Self::new(context, observations)
    }

    pub(crate) fn single_in(context: InputContext, observation: InputObservation) -> Self {
        Self::single(context, observation)
    }
}

pub(crate) type ObservationGroup = InputObservationGroup;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ContactState {
    pub(crate) position: Point2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeutralInputError {
    NonFiniteObservation,
    InvalidMeasurement,
    SourceTimeContextMismatch,
}

#[derive(Debug, Default)]
struct NeutralInputState {
    held_controls: HashSet<(InputSourceId, Option<InputDeviceId>, ControlId)>,
    contacts: HashMap<(InputSourceId, Option<InputDeviceId>, ContactId), ContactState>,
    absolute_pointer_positions: HashMap<InputSourceId, Point2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DigitalAdmission {
    #[cfg(test)]
    pub(crate) transition: DigitalTransition,
    pub(crate) was_down_anywhere: bool,
    pub(crate) is_down_anywhere: bool,
}

#[derive(Debug, Default)]
pub(crate) struct NeutralInputAuthority {
    state: NeutralInputState,
    controls: ControlInterner,
    source_sequences: HashMap<InputSourceId, SourceSequence>,
    admission_sequence: AdmissionSequence,
}

impl NeutralInputAuthority {
    pub(crate) fn admit_keyboard(
        &mut self,
        context: InputContext,
        input: &KeyboardInput,
    ) -> Result<DigitalAdmission, NeutralInputError> {
        let control = self.controls.intern_key(&input.physical_key);
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
        self.admit_digital_control(context, control, transition)
    }

    #[cfg(test)]
    pub(crate) fn key_down_in(&self, context: InputContext, key: &PhysicalKeyIdentity) -> bool {
        self.controls
            .key(key)
            .is_some_and(|control| self.control_down_in(context, control))
    }

    pub(crate) fn key_down_anywhere(&self, key: &PhysicalKeyIdentity) -> bool {
        self.controls
            .key(key)
            .is_some_and(|control| self.control_down_anywhere(control))
    }

    pub(crate) fn admit_pointer_button(
        &mut self,
        context: InputContext,
        input: PointerButtonInput,
    ) -> Result<DigitalAdmission, NeutralInputError> {
        let control = self.controls.intern_button(input.button);
        let transition = match input.state {
            DigitalState::Pressed => DigitalTransition::Down,
            DigitalState::Released => DigitalTransition::Up,
        };
        self.admit_digital_control(context, control, transition)
    }

    #[cfg(test)]
    pub(crate) fn pointer_button_down_in(
        &self,
        context: InputContext,
        button: PointerButton,
    ) -> bool {
        self.controls
            .button(button)
            .is_some_and(|control| self.control_down_in(context, control))
    }

    pub(crate) fn pointer_button_down_anywhere(&self, button: PointerButton) -> bool {
        self.controls
            .button(button)
            .is_some_and(|control| self.control_down_anywhere(control))
    }

    fn admit_digital_control(
        &mut self,
        context: InputContext,
        control: ControlId,
        transition: DigitalTransition,
    ) -> Result<DigitalAdmission, NeutralInputError> {
        let was_down_anywhere = self.control_down_anywhere(control);
        self.admit(ObservationGroup::single_in(
            context,
            InputObservation::DigitalControl {
                control,
                transition,
            },
        ))?;
        Ok(DigitalAdmission {
            #[cfg(test)]
            transition,
            was_down_anywhere,
            is_down_anywhere: self.control_down_anywhere(control),
        })
    }

    pub(crate) fn admit(&mut self, group: ObservationGroup) -> Result<(), NeutralInputError> {
        if group
            .observations
            .iter()
            .any(|observation| !is_finite(observation))
        {
            return Err(NeutralInputError::NonFiniteObservation);
        }
        if group
            .observations
            .iter()
            .any(|observation| !has_valid_measurements(observation))
        {
            return Err(NeutralInputError::InvalidMeasurement);
        }
        for observation in &group.observations {
            if let InputObservation::Tablet(tablet) = observation
                && tablet.source_time.is_some_and(|time| {
                    time.context != group.context
                        || matches!(
                            time.unit,
                            SourceTimeUnit::NativeTicks {
                                ticks_per_second: 0
                            }
                        )
                })
            {
                return Err(NeutralInputError::SourceTimeContextMismatch);
            }
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
            InputObservation::Tablet(observation) => {
                if observation.evidence != EvidenceStatus::ObservedConfirmed {
                    return;
                }
                match observation.phase {
                    ContactPhase::Begin | ContactPhase::Update
                        if observation.presence == ContactPresence::Contact =>
                    {
                        self.state.contacts.insert(
                            (context.source, context.device, observation.contact),
                            ContactState {
                                position: observation.position,
                            },
                        );
                    }
                    ContactPhase::End
                    | ContactPhase::Cancel
                    | ContactPhase::Begin
                    | ContactPhase::Update => {
                        self.state.contacts.remove(&(
                            context.source,
                            context.device,
                            observation.contact,
                        ));
                    }
                }
            }
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
        InputObservation::Tablet(observation) => {
            point_is_finite(observation.position)
                && observation.delta.x.is_finite()
                && observation.delta.y.is_finite()
                && observation.pressure.is_none_or(measurement_is_finite)
                && observation
                    .tangential_pressure
                    .is_none_or(measurement_is_finite)
                && observation.twist.is_none_or(measurement_is_finite)
                && observation.tilt.is_none_or(|tilt| {
                    tilt.x_degrees.is_finite()
                        && tilt.y_degrees.is_finite()
                        && (-90.0..=90.0).contains(&tilt.x_degrees)
                        && (-90.0..=90.0).contains(&tilt.y_degrees)
                })
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

fn has_valid_measurements(observation: &InputObservation) -> bool {
    match observation {
        InputObservation::Contact { pressure, .. } => {
            pressure.is_none_or(|measurement| measurement.domain.accepts(measurement.value))
        }
        InputObservation::Tablet(observation) => {
            observation
                .pressure
                .is_none_or(|measurement| measurement.domain.accepts(measurement.value))
                && observation
                    .tangential_pressure
                    .is_none_or(|measurement| measurement.domain.accepts(measurement.value))
                && observation
                    .twist
                    .is_none_or(|measurement| measurement.domain.accepts(measurement.value))
                && observation.tilt.is_none_or(|tilt| {
                    (-90.0..=90.0).contains(&tilt.x_degrees)
                        && (-90.0..=90.0).contains(&tilt.y_degrees)
                })
        }
        _ => true,
    }
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

    fn keyboard_input(
        physical_key: PhysicalKeyIdentity,
        state: DigitalState,
        repeat: bool,
        origin: ObservationOrigin,
    ) -> KeyboardInput {
        KeyboardInput {
            physical_key,
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state,
            repeat,
            origin,
        }
    }

    #[test]
    fn keyboard_identity_and_aggregate_held_state_are_neutral_owned() {
        let mut authority = NeutralInputAuthority::default();
        let key_a = PhysicalKeyIdentity::code("KeyA");
        let key_b = PhysicalKeyIdentity::code("KeyB");
        let context_a = InputContext::new(SOURCE_A, Some(InputDeviceId::new(1)));
        let context_b = InputContext::new(SOURCE_A, Some(InputDeviceId::new(2)));

        let first = authority
            .admit_keyboard(
                context_a,
                &keyboard_input(
                    key_a.clone(),
                    DigitalState::Pressed,
                    false,
                    ObservationOrigin::SourceReport,
                ),
            )
            .expect("first key press should admit");
        assert!(!first.was_down_anywhere);
        assert!(first.is_down_anywhere);
        assert!(authority.key_down_in(context_a, &key_a));
        assert!(!authority.key_down_in(context_b, &key_a));
        assert!(!authority.key_down_anywhere(&key_b));

        let second = authority
            .admit_keyboard(
                context_b,
                &keyboard_input(
                    key_a.clone(),
                    DigitalState::Pressed,
                    false,
                    ObservationOrigin::SourceReport,
                ),
            )
            .expect("second-device key press should admit");
        assert!(second.was_down_anywhere);
        assert!(second.is_down_anywhere);

        let release_a = authority
            .admit_keyboard(
                context_a,
                &keyboard_input(
                    key_a.clone(),
                    DigitalState::Released,
                    false,
                    ObservationOrigin::SourceReport,
                ),
            )
            .expect("first-device key release should admit");
        assert!(release_a.was_down_anywhere);
        assert!(release_a.is_down_anywhere);
        assert!(!authority.key_down_in(context_a, &key_a));
        assert!(authority.key_down_in(context_b, &key_a));

        let release_b = authority
            .admit_keyboard(
                context_b,
                &keyboard_input(
                    key_a.clone(),
                    DigitalState::Released,
                    false,
                    ObservationOrigin::SourceReport,
                ),
            )
            .expect("second-device key release should admit");
        assert!(release_b.was_down_anywhere);
        assert!(!release_b.is_down_anywhere);
        assert!(!authority.key_down_anywhere(&key_a));
    }

    #[test]
    fn keyboard_repeat_and_reconciliation_preserve_neutral_transition_semantics() {
        let mut authority = NeutralInputAuthority::default();
        let key = PhysicalKeyIdentity::code("KeyR");

        let first = authority
            .admit_keyboard(
                CONTEXT_A,
                &keyboard_input(
                    key.clone(),
                    DigitalState::Pressed,
                    false,
                    ObservationOrigin::SourceReport,
                ),
            )
            .expect("ordinary press should admit");
        assert_eq!(first.transition, DigitalTransition::Down);
        assert!(!first.was_down_anywhere);

        let repeat = authority
            .admit_keyboard(
                CONTEXT_A,
                &keyboard_input(
                    key.clone(),
                    DigitalState::Pressed,
                    true,
                    ObservationOrigin::SourceReport,
                ),
            )
            .expect("repeat should admit as repeated evidence");
        assert_eq!(repeat.transition, DigitalTransition::Down);
        assert!(repeat.was_down_anywhere);
        assert!(repeat.is_down_anywhere);

        let cancel = authority
            .admit_keyboard(
                CONTEXT_A,
                &keyboard_input(
                    key.clone(),
                    DigitalState::Released,
                    false,
                    ObservationOrigin::BackendSyntheticReconciliation,
                ),
            )
            .expect("synthetic release should reconcile");
        assert_eq!(cancel.transition, DigitalTransition::Cancel);
        assert!(!cancel.is_down_anywhere);

        let reconcile = authority
            .admit_keyboard(
                CONTEXT_A,
                &keyboard_input(
                    key.clone(),
                    DigitalState::Pressed,
                    false,
                    ObservationOrigin::BackendSyntheticReconciliation,
                ),
            )
            .expect("synthetic press should reconcile");
        assert_eq!(reconcile.transition, DigitalTransition::ReconcileDown);
        assert!(reconcile.is_down_anywhere);
    }

    #[test]
    fn pointer_button_identity_and_aggregate_state_are_neutral_owned() {
        let mut authority = NeutralInputAuthority::default();
        let context_a = InputContext::new(SOURCE_A, Some(InputDeviceId::new(1)));
        let context_b = InputContext::new(SOURCE_B, Some(InputDeviceId::new(2)));
        let pressed = PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Pressed,
        };
        let released = PointerButtonInput {
            button: PointerButton::Left,
            state: DigitalState::Released,
        };

        let first = authority
            .admit_pointer_button(context_a, pressed)
            .expect("first button press should admit");
        assert!(!first.was_down_anywhere);
        assert!(authority.pointer_button_down_in(context_a, PointerButton::Left));
        assert!(!authority.pointer_button_down_anywhere(PointerButton::Right));

        let second = authority
            .admit_pointer_button(context_b, pressed)
            .expect("second-source button press should admit");
        assert!(second.was_down_anywhere);

        let release_a = authority
            .admit_pointer_button(context_a, released)
            .expect("first-source release should admit");
        assert!(release_a.is_down_anywhere);
        assert!(!authority.pointer_button_down_in(context_a, PointerButton::Left));
        assert!(authority.pointer_button_down_in(context_b, PointerButton::Left));

        let release_b = authority
            .admit_pointer_button(context_b, released)
            .expect("second-source release should admit");
        assert!(!release_b.is_down_anywhere);
        assert!(!authority.pointer_button_down_anywhere(PointerButton::Left));
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
    fn vertical_only_scroll_keeps_horizontal_absent_not_measured_zero() {
        let delta = ScrollDelta::vertical_only(0.0);

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
        let position = Point2::new(10.0, 12.0, CoordinateSpace::UnspecifiedTargetUnits);
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
                MeasurementDomain::UnspecifiedScalar,
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

    fn tablet_observation(
        phase: ContactPhase,
        evidence: EvidenceStatus,
        position: Point2,
        pressure: Option<AnalogMeasurement>,
    ) -> TabletObservation {
        TabletObservation {
            contact: ContactId::new(44),
            tool: Some(ToolId::new(8)),
            tool_kind: InputToolKind::Pen,
            phase,
            presence: ContactPresence::Contact,
            position,
            delta: Vector2::new(1.0, 2.0),
            pressure,
            tangential_pressure: None,
            tilt: None,
            twist: None,
            controls: PhysicalTabletControls::default(),
            capabilities: TabletCapabilities::default(),
            source_time: Some(SourceTime::new(
                CONTEXT_A,
                100,
                SourceTimeUnit::Microseconds,
            )),
            evidence,
            delivery: DeliveryRole::OrdinaryCurrent,
            origin: ObservationOrigin::SourceReport,
        }
    }

    #[test]
    fn predicted_tablet_observation_does_not_mutate_confirmed_contact_state() {
        let mut authority = NeutralInputAuthority::default();
        let position = Point2::new(10.0, 12.0, CoordinateSpace::WindowPhysicalPixels);
        let contact = ContactId::new(44);

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Begin,
                    EvidenceStatus::ObservedConfirmed,
                    position,
                    Some(AnalogMeasurement::new(
                        0.2,
                        MeasurementDomain::NormalizedUnitInterval,
                    )),
                )),
            ))
            .expect("confirmed tablet observation should admit");
        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Update,
                    EvidenceStatus::PredictedProvisional,
                    Point2::new(99.0, 101.0, CoordinateSpace::WindowPhysicalPixels),
                    Some(AnalogMeasurement::new(
                        0.9,
                        MeasurementDomain::NormalizedUnitInterval,
                    )),
                )),
            ))
            .expect("predicted tablet observation should admit");

        assert_eq!(
            authority
                .contact_state_in(CONTEXT_A, contact)
                .unwrap()
                .position,
            position
        );
    }

    #[test]
    fn estimated_tablet_observations_never_mutate_confirmed_contact_state() {
        let mut authority = NeutralInputAuthority::default();
        let position = Point2::new(10.0, 12.0, CoordinateSpace::WindowPhysicalPixels);
        let contact = ContactId::new(44);

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Begin,
                    EvidenceStatus::EstimatedRevisable,
                    position,
                    None,
                )),
            ))
            .expect("estimated begin should remain deliverable");
        assert!(authority.contact_state_in(CONTEXT_A, contact).is_none());

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Begin,
                    EvidenceStatus::ObservedConfirmed,
                    position,
                    None,
                )),
            ))
            .expect("confirmed begin should admit");
        let revised_position = Point2::new(99.0, 101.0, CoordinateSpace::WindowPhysicalPixels);
        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Update,
                    EvidenceStatus::EstimatedRevisable,
                    revised_position,
                    None,
                )),
            ))
            .expect("estimated update should remain deliverable");
        assert_eq!(
            authority
                .contact_state_in(CONTEXT_A, contact)
                .expect("confirmed contact should remain held")
                .position,
            position
        );

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::End,
                    EvidenceStatus::EstimatedRevisable,
                    revised_position,
                    None,
                )),
            ))
            .expect("estimated end should remain deliverable");
        assert!(authority.contact_state_in(CONTEXT_A, contact).is_some());
    }

    #[test]
    fn confirmed_hover_and_out_of_range_clear_stale_tablet_contact_state() {
        let mut authority = NeutralInputAuthority::default();
        let contact = ContactId::new(44);
        let position = Point2::new(10.0, 12.0, CoordinateSpace::WindowPhysicalPixels);

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Begin,
                    EvidenceStatus::ObservedConfirmed,
                    position,
                    None,
                )),
            ))
            .expect("confirmed begin should admit");
        let mut hover = tablet_observation(
            ContactPhase::Update,
            EvidenceStatus::ObservedConfirmed,
            position,
            None,
        );
        hover.presence = ContactPresence::Hover;
        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(hover),
            ))
            .expect("confirmed hover should admit");
        assert!(authority.contact_state_in(CONTEXT_A, contact).is_none());

        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Begin,
                    EvidenceStatus::ObservedConfirmed,
                    position,
                    None,
                )),
            ))
            .expect("second confirmed begin should admit");
        let mut out_of_range = tablet_observation(
            ContactPhase::Update,
            EvidenceStatus::ObservedConfirmed,
            position,
            None,
        );
        out_of_range.presence = ContactPresence::OutOfRange;
        authority
            .admit(ObservationGroup::single_in(
                CONTEXT_A,
                InputObservation::Tablet(out_of_range),
            ))
            .expect("confirmed out-of-range should admit");
        assert!(authority.contact_state_in(CONTEXT_A, contact).is_none());
    }

    #[test]
    fn invalid_tablet_measurement_rejects_group_without_partial_state() {
        let mut authority = NeutralInputAuthority::default();
        let result = authority.admit(ObservationGroup::new_in(
            CONTEXT_A,
            vec![
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Begin,
                    EvidenceStatus::ObservedConfirmed,
                    Point2::new(10.0, 12.0, CoordinateSpace::WindowPhysicalPixels),
                    Some(AnalogMeasurement::new(
                        0.3,
                        MeasurementDomain::NormalizedUnitInterval,
                    )),
                )),
                InputObservation::Tablet(tablet_observation(
                    ContactPhase::Update,
                    EvidenceStatus::ObservedConfirmed,
                    Point2::new(11.0, 13.0, CoordinateSpace::WindowPhysicalPixels),
                    Some(AnalogMeasurement::new(
                        1.5,
                        MeasurementDomain::NormalizedUnitInterval,
                    )),
                )),
            ],
        ));

        assert_eq!(result, Err(NeutralInputError::InvalidMeasurement));
        assert_eq!(authority.active_contact_count(SOURCE_A), 0);
        assert_eq!(authority.admission_sequence().get(), 0);
    }
}
