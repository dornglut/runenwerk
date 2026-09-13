use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct InputSourceId(u64);

impl InputSourceId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
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
pub(crate) enum DigitalTransition {
    Down,
    ReconcileDown,
    Up,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContactPhase {
    Begin,
    Update,
    End,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoordinateSpace {
    LegacyWindowPhysicalPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RelativeMotionUnit {
    BackendDeviceUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollDomain {
    LegacyVerticalScalarUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MeasurementDomain {
    LegacyPressureScalar,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Point2 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) space: CoordinateSpace,
}

impl Point2 {
    pub(crate) const fn new(x: f32, y: f32, space: CoordinateSpace) -> Self {
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
pub(crate) struct AnalogMeasurement {
    pub(crate) value: f32,
    pub(crate) domain: MeasurementDomain,
}

impl AnalogMeasurement {
    pub(crate) const fn new(value: f32, domain: MeasurementDomain) -> Self {
        Self { value, domain }
    }
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
        delta: Vector2,
        domain: ScrollDomain,
    },
    Contact {
        contact: ContactId,
        phase: ContactPhase,
        position: Point2,
        pressure: Option<AnalogMeasurement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ObservationGroup {
    source: InputSourceId,
    observations: Vec<InputObservation>,
}

impl ObservationGroup {
    pub(crate) fn new(source: InputSourceId, observations: Vec<InputObservation>) -> Self {
        Self {
            source,
            observations,
        }
    }

    pub(crate) fn single(source: InputSourceId, observation: InputObservation) -> Self {
        Self::new(source, vec![observation])
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
    held_controls: HashSet<(InputSourceId, ControlId)>,
    contacts: HashMap<(InputSourceId, ContactId), ContactState>,
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

        let source_sequence = self.source_sequences.entry(group.source).or_default();
        *source_sequence = source_sequence.next();
        self.admission_sequence = self.admission_sequence.next();

        for observation in group.observations {
            self.apply(group.source, observation);
        }

        Ok(())
    }

    pub(crate) fn control_down(&self, source: InputSourceId, control: ControlId) -> bool {
        self.state.held_controls.contains(&(source, control))
    }

    pub(crate) fn absolute_pointer_position(&self, source: InputSourceId) -> Option<Point2> {
        self.state.absolute_pointer_positions.get(&source).copied()
    }

    pub(crate) fn contact_state(
        &self,
        source: InputSourceId,
        contact: ContactId,
    ) -> Option<ContactState> {
        self.state.contacts.get(&(source, contact)).copied()
    }

    #[cfg(test)]
    pub(crate) fn active_contact_count(&self, source: InputSourceId) -> usize {
        self.state
            .contacts
            .keys()
            .filter(|(candidate, _)| *candidate == source)
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

    fn apply(&mut self, source: InputSourceId, observation: InputObservation) {
        match observation {
            InputObservation::DigitalControl {
                control,
                transition,
            } => match transition {
                DigitalTransition::Down | DigitalTransition::ReconcileDown => {
                    self.state.held_controls.insert((source, control));
                }
                DigitalTransition::Up | DigitalTransition::Cancel => {
                    self.state.held_controls.remove(&(source, control));
                }
            },
            InputObservation::AbsolutePointerPosition { position } => {
                self.state
                    .absolute_pointer_positions
                    .insert(source, position);
            }
            InputObservation::RelativeMotion { .. } | InputObservation::Scroll { .. } => {}
            InputObservation::Contact {
                contact,
                phase,
                position,
                ..
            } => match phase {
                ContactPhase::Begin | ContactPhase::Update => {
                    self.state
                        .contacts
                        .insert((source, contact), ContactState { position });
                }
                ContactPhase::End | ContactPhase::Cancel => {
                    self.state.contacts.remove(&(source, contact));
                }
            },
        }
    }
}

fn is_finite(observation: &InputObservation) -> bool {
    match observation {
        InputObservation::DigitalControl { .. } => true,
        InputObservation::AbsolutePointerPosition { position } => {
            let _ = position.space;
            point_is_finite(*position)
        }
        InputObservation::RelativeMotion { delta, unit } => {
            let _ = unit;
            delta.x.is_finite() && delta.y.is_finite()
        }
        InputObservation::Scroll { delta, domain } => {
            let _ = domain;
            delta.x.is_finite() && delta.y.is_finite()
        }
        InputObservation::Contact {
            position, pressure, ..
        } => {
            let _ = position.space;
            point_is_finite(*position)
                && pressure.as_ref().is_none_or(|measurement| {
                    let _ = measurement.domain;
                    measurement.value.is_finite()
                })
        }
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
    const CONTROL: ControlId = ControlId::new(7);

    #[test]
    fn source_and_admission_sequences_are_distinct_and_deterministic() {
        let mut authority = NeutralInputAuthority::default();

        authority
            .admit(ObservationGroup::single(
                SOURCE_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Down,
                },
            ))
            .expect("first source-A observation should admit");
        assert_eq!(authority.source_sequence(SOURCE_A).unwrap().get(), 1);
        assert_eq!(authority.admission_sequence().get(), 1);

        authority
            .admit(ObservationGroup::single(
                SOURCE_B,
                InputObservation::RelativeMotion {
                    delta: Vector2::new(1.0, -1.0),
                    unit: RelativeMotionUnit::BackendDeviceUnits,
                },
            ))
            .expect("source-B observation should admit");
        assert_eq!(authority.source_sequence(SOURCE_B).unwrap().get(), 1);
        assert_eq!(authority.admission_sequence().get(), 2);

        authority
            .admit(ObservationGroup::single(
                SOURCE_A,
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
            .admit(ObservationGroup::single(
                SOURCE_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Down,
                },
            ))
            .expect("down should admit");
        assert!(authority.control_down(SOURCE_A, CONTROL));
        assert_eq!(authority.admission_sequence().get(), 1);

        authority
            .admit(ObservationGroup::single(
                SOURCE_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Up,
                },
            ))
            .expect("up should admit");
        assert!(!authority.control_down(SOURCE_A, CONTROL));
        assert_eq!(authority.admission_sequence().get(), 2);
    }

    #[test]
    fn reconciliation_changes_confirmed_state_without_an_ordinary_edge() {
        let mut authority = NeutralInputAuthority::default();

        authority
            .admit(ObservationGroup::single(
                SOURCE_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::ReconcileDown,
                },
            ))
            .expect("reconciliation down should admit");
        assert!(authority.control_down(SOURCE_A, CONTROL));

        authority
            .admit(ObservationGroup::single(
                SOURCE_A,
                InputObservation::DigitalControl {
                    control: CONTROL,
                    transition: DigitalTransition::Cancel,
                },
            ))
            .expect("cancel should admit");
        assert!(!authority.control_down(SOURCE_A, CONTROL));
    }

    #[test]
    fn omitted_pressure_remains_distinct_from_measured_zero() {
        let position = Point2::new(10.0, 12.0, CoordinateSpace::LegacyWindowPhysicalPixels);
        let omitted = InputObservation::Contact {
            contact: ContactId::new(9),
            phase: ContactPhase::Update,
            position,
            pressure: None,
        };
        let measured_zero = InputObservation::Contact {
            contact: ContactId::new(9),
            phase: ContactPhase::Update,
            position,
            pressure: Some(AnalogMeasurement::new(
                0.0,
                MeasurementDomain::LegacyPressureScalar,
            )),
        };

        assert_ne!(omitted, measured_zero);
    }

    #[test]
    fn invalid_numeric_group_is_rejected_atomically() {
        let mut authority = NeutralInputAuthority::default();

        let result = authority.admit(ObservationGroup::new(
            SOURCE_A,
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
        assert!(!authority.control_down(SOURCE_A, CONTROL));
        assert_eq!(authority.admission_sequence().get(), 0);
    }
}
