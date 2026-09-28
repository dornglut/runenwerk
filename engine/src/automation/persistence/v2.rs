//! Persisted normalized automation replay-trace V2.
//!
//! V2 preserves V1 semantics and adds only absolute pointer position observations.

use super::*;

pub const AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedAutomationInputTraceV2 {
    trace: AutomationInputTrace,
    recording_witness: AutomationInputTraceRecordingWitness,
    provenance: Option<AutomationInputTraceProvenance>,
}

impl ImportedAutomationInputTraceV2 {
    pub fn trace(&self) -> &AutomationInputTrace {
        &self.trace
    }

    pub fn into_trace(self) -> AutomationInputTrace {
        self.trace
    }

    pub fn recording_witness(&self) -> AutomationInputTraceRecordingWitness {
        self.recording_witness
    }

    pub fn provenance(&self) -> Option<&AutomationInputTraceProvenance> {
        self.provenance.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedAutomationInputTrace {
    trace: AutomationInputTrace,
    recording_witness: AutomationInputTraceRecordingWitness,
    provenance: Option<AutomationInputTraceProvenance>,
    schema_version: u32,
}

impl ImportedAutomationInputTrace {
    pub fn trace(&self) -> &AutomationInputTrace {
        &self.trace
    }

    pub fn into_trace(self) -> AutomationInputTrace {
        self.trace
    }

    pub fn recording_witness(&self) -> AutomationInputTraceRecordingWitness {
        self.recording_witness
    }

    pub fn provenance(&self) -> Option<&AutomationInputTraceProvenance> {
        self.provenance.as_ref()
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedTraceV2 {
    artifact_kind: String,
    schema_version: u32,
    recorded_sources_pristine_at_capture_start: bool,
    #[serde(default)]
    provenance: Option<AutomationInputTraceProvenance>,
    frames: Vec<PersistedFrameV2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedFrameV2 {
    frame_ordinal: u64,
    groups: Vec<PersistedGroupV2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedGroupV2 {
    source_slot: u32,
    device_slot: Option<u32>,
    observations: Vec<PersistedObservationV2>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum PersistedObservationV2 {
    PointerButton(PersistedPointerButtonInputV1),
    RelativeMotion(PersistedRelativeMotionV1),
    AbsolutePointerPosition(PersistedAbsolutePointerPositionV2),
    Scroll(PersistedScrollInputV1),
    Tablet(PersistedTabletObservationV1),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedAbsolutePointerPositionV2 {
    position: PersistedPoint2V1,
}

pub fn export_automation_input_trace_v2(
    trace: &AutomationInputTrace,
    recording_witness: AutomationInputTraceRecordingWitness,
    provenance: Option<&AutomationInputTraceProvenance>,
) -> Result<String, AutomationInputTraceExportError> {
    if !trace.trailing_groups.is_empty() {
        return Err(AutomationInputTraceExportError::UnframedTrailingGroups);
    }
    validate_export_normalized_input(trace)?;
    validate_provenance_export(provenance)?;
    let persisted = ExportBuilder::new().build_v2(trace, recording_witness, provenance.cloned())?;
    let encoded = ron_options()
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .map_err(|error| {
            AutomationInputTraceExportError::SerializationFailure(error.to_string())
        })?;
    if encoded.len() > MAX_ARTIFACT_BYTES {
        return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
            "artifact_bytes",
        ));
    }
    Ok(encoded)
}

pub fn import_automation_input_trace_v2(
    bytes: &[u8],
) -> Result<ImportedAutomationInputTraceV2, AutomationInputTraceImportError> {
    let (source, probe) = probe_persisted_trace(bytes)?;
    if probe.artifact_kind != AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND {
        return Err(AutomationInputTraceImportError::WrongArtifactKind(
            probe.artifact_kind,
        ));
    }
    if probe.schema_version != AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION {
        return Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(
            probe.schema_version,
        ));
    }
    import_automation_input_trace_v2_from_source(source)
}

pub fn import_automation_input_trace(
    bytes: &[u8],
) -> Result<ImportedAutomationInputTrace, AutomationInputTraceImportError> {
    let (source, probe) = probe_persisted_trace(bytes)?;
    if probe.artifact_kind != AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND {
        return Err(AutomationInputTraceImportError::WrongArtifactKind(
            probe.artifact_kind,
        ));
    }

    match probe.schema_version {
        AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION => {
            let imported = import_automation_input_trace_v1_from_source(source)?;
            Ok(ImportedAutomationInputTrace {
                trace: imported.trace,
                recording_witness: imported.recording_witness,
                provenance: imported.provenance,
                schema_version: AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION,
            })
        }
        AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION => {
            let imported = import_automation_input_trace_v2_from_source(source)?;
            Ok(ImportedAutomationInputTrace {
                trace: imported.trace,
                recording_witness: imported.recording_witness,
                provenance: imported.provenance,
                schema_version: AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION,
            })
        }
        version => Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(
            version,
        )),
    }
}

fn import_automation_input_trace_v2_from_source(
    source: &str,
) -> Result<ImportedAutomationInputTraceV2, AutomationInputTraceImportError> {
    let persisted: PersistedTraceV2 = ron_options().from_str(source).map_err(classify_ron_error)?;
    validate_persisted_header_v2(&persisted)?;
    validate_provenance_import(persisted.provenance.as_ref())?;
    if !persisted.recorded_sources_pristine_at_capture_start {
        return Err(AutomationInputTraceImportError::UnsupportedRecordingWitness);
    }
    let recording_witness =
        AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart;

    let trace = ImportBuilder::new().build_v2(&persisted)?;
    validate_imported_normalized_input(&trace)?;

    Ok(ImportedAutomationInputTraceV2 {
        trace,
        recording_witness,
        provenance: persisted.provenance,
    })
}

fn validate_persisted_header_v2(
    persisted: &PersistedTraceV2,
) -> Result<(), AutomationInputTraceImportError> {
    if persisted.artifact_kind != AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND {
        return Err(AutomationInputTraceImportError::WrongArtifactKind(
            persisted.artifact_kind.clone(),
        ));
    }
    if persisted.schema_version != AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION {
        return Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(
            persisted.schema_version,
        ));
    }
    Ok(())
}

impl ExportBuilder {
    fn build_v2(
        mut self,
        trace: &AutomationInputTrace,
        recording_witness: AutomationInputTraceRecordingWitness,
        provenance: Option<AutomationInputTraceProvenance>,
    ) -> Result<PersistedTraceV2, AutomationInputTraceExportError> {
        if trace.frames.len() > MAX_FRAMES {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "frames",
            ));
        }

        let mut frames = Vec::with_capacity(trace.frames.len());
        for (frame_index, frame) in trace.frames.iter().enumerate() {
            if frame.frame_ordinal != frame_index as u64 {
                return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                    "frame ordinals must be contiguous from zero".to_owned(),
                ));
            }
            if frame.groups.len() > MAX_GROUPS_PER_FRAME {
                return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                    "groups_per_frame",
                ));
            }
            self.total_groups = self.total_groups.checked_add(frame.groups.len()).ok_or(
                AutomationInputTraceExportError::ResourceLimitExceeded("total_groups"),
            )?;
            if self.total_groups > MAX_TOTAL_GROUPS {
                return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                    "total_groups",
                ));
            }

            let mut groups = Vec::with_capacity(frame.groups.len());
            for group in &frame.groups {
                groups.push(self.convert_group_v2(group)?);
            }
            frames.push(PersistedFrameV2 {
                frame_ordinal: frame.frame_ordinal,
                groups,
            });
        }

        Ok(PersistedTraceV2 {
            artifact_kind: AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND.to_owned(),
            schema_version: AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION,
            recorded_sources_pristine_at_capture_start: matches!(
                recording_witness,
                AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
            ),
            provenance,
            frames,
        })
    }

    fn convert_group_v2(
        &mut self,
        group: &InputObservationGroup,
    ) -> Result<PersistedGroupV2, AutomationInputTraceExportError> {
        if group.observations.is_empty() {
            return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                "empty observation groups are not persisted".to_owned(),
            ));
        }
        if group.observations.len() > MAX_OBSERVATIONS_PER_GROUP {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "observations_per_group",
            ));
        }

        let source_slot = self.source_slot(group.context.source)?;
        let device_slot = group
            .context
            .device
            .map(|device| self.device_slot(group.context.source, device))
            .transpose()?;

        let all_tablet = group
            .observations
            .iter()
            .all(|observation| matches!(observation, InputObservation::Tablet(_)));
        if !all_tablet && group.observations.len() != 1 {
            return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                "multi-observation non-tablet groups are not persisted".to_owned(),
            ));
        }

        let mut observations = Vec::with_capacity(group.observations.len());
        for observation in &group.observations {
            observations.push(self.convert_observation_v2(
                group.context,
                observation,
                all_tablet,
            )?);
        }

        Ok(PersistedGroupV2 {
            source_slot,
            device_slot,
            observations,
        })
    }

    fn convert_observation_v2(
        &mut self,
        context: InputContext,
        observation: &InputObservation,
        all_tablet: bool,
    ) -> Result<PersistedObservationV2, AutomationInputTraceExportError> {
        match observation {
            InputObservation::PointerButton(input) if !all_tablet => {
                if self.first_pointer_state.insert((context, input.button))
                    && input.state != DigitalState::Pressed
                {
                    return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                        "first pointer-button state must establish a press".to_owned(),
                    ));
                }
                Ok(PersistedObservationV2::PointerButton(
                    PersistedPointerButtonInputV1 {
                        button: pointer_button_to_persisted(input.button),
                        state: digital_state_to_persisted(input.state),
                    },
                ))
            }
            InputObservation::RelativeMotion { delta, unit } if !all_tablet => Ok(
                PersistedObservationV2::RelativeMotion(PersistedRelativeMotionV1 {
                    delta: vector_to_persisted(*delta),
                    unit: relative_unit_to_persisted(*unit),
                }),
            ),
            InputObservation::AbsolutePointerPosition { position } if !all_tablet => {
                Ok(PersistedObservationV2::AbsolutePointerPosition(
                    PersistedAbsolutePointerPositionV2 {
                        position: point_to_persisted(*position),
                    },
                ))
            }
            InputObservation::Scroll(input) if !all_tablet => {
                Ok(PersistedObservationV2::Scroll(scroll_to_persisted(*input)))
            }
            InputObservation::Tablet(tablet) if all_tablet => Ok(PersistedObservationV2::Tablet(
                self.tablet_to_persisted(context, tablet)?,
            )),
            _ => Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                "trace contains an observation outside persisted V2 replay scope".to_owned(),
            )),
        }
    }
}

impl ImportBuilder {
    fn build_v2(
        mut self,
        persisted: &PersistedTraceV2,
    ) -> Result<AutomationInputTrace, AutomationInputTraceImportError> {
        if persisted.frames.len() > MAX_FRAMES {
            return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                "frames",
            ));
        }

        let mut frames = Vec::with_capacity(persisted.frames.len());
        for (frame_index, frame) in persisted.frames.iter().enumerate() {
            if frame.frame_ordinal != frame_index as u64 {
                return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                    "frame ordinals must be contiguous from zero".to_owned(),
                ));
            }
            if frame.groups.len() > MAX_GROUPS_PER_FRAME {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "groups_per_frame",
                ));
            }
            self.total_groups = self.total_groups.checked_add(frame.groups.len()).ok_or(
                AutomationInputTraceImportError::ResourceLimitExceeded("total_groups"),
            )?;
            if self.total_groups > MAX_TOTAL_GROUPS {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "total_groups",
                ));
            }

            let mut groups = Vec::with_capacity(frame.groups.len());
            for group in &frame.groups {
                groups.push(self.convert_group_v2(group)?);
            }
            frames.push(AutomationInputTraceFrame {
                frame_ordinal: frame.frame_ordinal,
                groups,
            });
        }

        Ok(AutomationInputTrace {
            frames,
            trailing_groups: Vec::new(),
        })
    }

    fn convert_group_v2(
        &mut self,
        group: &PersistedGroupV2,
    ) -> Result<InputObservationGroup, AutomationInputTraceImportError> {
        if group.observations.is_empty() {
            return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                "empty observation groups are not replayable".to_owned(),
            ));
        }
        if group.observations.len() > MAX_OBSERVATIONS_PER_GROUP {
            return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                "observations_per_group",
            ));
        }

        let source = self.materialize_source(group.source_slot)?;
        let device = group
            .device_slot
            .map(|slot| self.materialize_device(group.source_slot, slot))
            .transpose()?;
        let context = InputContext::new(source, device);

        let all_tablet = group
            .observations
            .iter()
            .all(|observation| matches!(observation, PersistedObservationV2::Tablet(_)));
        if !all_tablet && group.observations.len() != 1 {
            return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                "multi-observation non-tablet groups are not replayable".to_owned(),
            ));
        }

        let mut observations = Vec::with_capacity(group.observations.len());
        for observation in &group.observations {
            observations.push(self.convert_observation_v2(
                group.source_slot,
                group.device_slot,
                context,
                observation,
                all_tablet,
            )?);
        }

        Ok(InputObservationGroup::new(context, observations))
    }

    fn convert_observation_v2(
        &mut self,
        source_slot: u32,
        device_slot: Option<u32>,
        context: InputContext,
        observation: &PersistedObservationV2,
        all_tablet: bool,
    ) -> Result<InputObservation, AutomationInputTraceImportError> {
        match observation {
            PersistedObservationV2::PointerButton(input) if !all_tablet => {
                let button = pointer_button_from_persisted(input.button);
                let state = digital_state_from_persisted(input.state);
                if self.first_pointer_state.insert((context, button))
                    && state != DigitalState::Pressed
                {
                    return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                        "first pointer-button state must establish a press".to_owned(),
                    ));
                }
                Ok(InputObservation::PointerButton(PointerButtonInput {
                    button,
                    state,
                }))
            }
            PersistedObservationV2::RelativeMotion(input) if !all_tablet => {
                Ok(InputObservation::RelativeMotion {
                    delta: vector_from_persisted(input.delta),
                    unit: relative_unit_from_persisted(input.unit),
                })
            }
            PersistedObservationV2::AbsolutePointerPosition(input) if !all_tablet => {
                Ok(InputObservation::AbsolutePointerPosition {
                    position: point_from_persisted(input.position),
                })
            }
            PersistedObservationV2::Scroll(input) if !all_tablet => {
                Ok(InputObservation::Scroll(scroll_from_persisted(*input)))
            }
            PersistedObservationV2::Tablet(tablet) if all_tablet => Ok(InputObservation::Tablet(
                self.tablet_from_persisted(source_slot, device_slot, context, tablet)?,
            )),
            _ => Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                "artifact contains an observation outside persisted V2 replay scope".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn witness() -> AutomationInputTraceRecordingWitness {
        AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
    }

    fn absolute_trace(space: CoordinateSpace) -> AutomationInputTrace {
        AutomationInputTrace {
            frames: vec![AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![InputObservationGroup::single(
                    InputContext::new(InputSourceId::new(1), None),
                    InputObservation::AbsolutePointerPosition {
                        position: Point2::new(12.5, 24.75, space),
                    },
                )],
            }],
            trailing_groups: Vec::new(),
        }
    }

    #[test]
    fn v2_round_trips_absolute_pointer_in_both_coordinate_spaces() {
        for space in [
            CoordinateSpace::UnspecifiedTargetUnits,
            CoordinateSpace::WindowPhysicalPixels,
        ] {
            let trace = absolute_trace(space);
            let encoded = export_automation_input_trace_v2(&trace, witness(), None).unwrap();
            let imported = import_automation_input_trace_v2(encoded.as_bytes()).unwrap();
            assert_eq!(imported.trace(), &trace);
            assert_eq!(
                imported.recording_witness(),
                AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
            );
        }
    }

    #[test]
    fn v1_stays_strict_while_version_aware_import_accepts_v1_and_v2() {
        let relative = AutomationInputTrace {
            frames: vec![AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![InputObservationGroup::single(
                    InputContext::new(InputSourceId::new(1), None),
                    InputObservation::RelativeMotion {
                        delta: Vector2::new(1.0, -2.0),
                        unit: RelativeMotionUnit::BackendDeviceUnits,
                    },
                )],
            }],
            trailing_groups: Vec::new(),
        };
        let v1 = export_automation_input_trace_v1(&relative, witness(), None).unwrap();
        let imported_v1 = import_automation_input_trace(v1.as_bytes()).unwrap();
        assert_eq!(
            imported_v1.schema_version(),
            AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION
        );
        assert_eq!(imported_v1.trace(), &relative);

        let absolute = absolute_trace(CoordinateSpace::WindowPhysicalPixels);
        assert!(matches!(
            export_automation_input_trace_v1(&absolute, witness(), None),
            Err(AutomationInputTraceExportError::UnsupportedTraceShape(_))
        ));

        let v2 = export_automation_input_trace_v2(&absolute, witness(), None).unwrap();
        assert!(matches!(
            import_automation_input_trace_v1(v2.as_bytes()),
            Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(2))
        ));
        let imported_v2 = import_automation_input_trace(v2.as_bytes()).unwrap();
        assert_eq!(
            imported_v2.schema_version(),
            AUTOMATION_INPUT_TRACE_V2_SCHEMA_VERSION
        );
        assert_eq!(imported_v2.trace(), &absolute);
    }

    #[test]
    fn version_aware_import_rejects_unknown_version_and_v2_rejects_keyboard() {
        let future =
            "(artifact_kind: \"runenwerk.automation.normalized-replay-trace\", schema_version: 3)";
        assert!(matches!(
            import_automation_input_trace(future.as_bytes()),
            Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(3))
        ));

        let keyboard = AutomationInputTrace {
            frames: vec![AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![InputObservationGroup::single(
                    InputContext::new(InputSourceId::new(9), None),
                    InputObservation::Keyboard(runen_input::KeyboardInput {
                        physical_key: runen_input::PhysicalKeyIdentity::code("KeyA"),
                        logical_key: runen_input::LogicalKey::Native(
                            runen_input::NativeLogicalKey::Unidentified,
                        ),
                        location: runen_input::KeyLocation::Standard,
                        state: DigitalState::Pressed,
                        repeat: false,
                        origin: runen_input::ObservationOrigin::SourceReport,
                    }),
                )],
            }],
            trailing_groups: Vec::new(),
        };
        assert!(matches!(
            export_automation_input_trace_v2(&keyboard, witness(), None),
            Err(AutomationInputTraceExportError::UnsupportedTraceShape(_))
        ));
    }

    #[test]
    fn v2_rejects_non_finite_absolute_pointer_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let trace = AutomationInputTrace {
                frames: vec![AutomationInputTraceFrame {
                    frame_ordinal: 0,
                    groups: vec![InputObservationGroup::single(
                        InputContext::new(InputSourceId::new(1), None),
                        InputObservation::AbsolutePointerPosition {
                            position: Point2::new(
                                value,
                                1.0,
                                CoordinateSpace::WindowPhysicalPixels,
                            ),
                        },
                    )],
                }],
                trailing_groups: Vec::new(),
            };
            assert!(matches!(
                export_automation_input_trace_v2(&trace, witness(), None),
                Err(AutomationInputTraceExportError::UnsupportedTraceShape(_))
            ));
        }
    }

    #[test]
    fn v2_import_remains_strict_for_unknown_fields_variants_and_witness() {
        let encoded = export_automation_input_trace_v2(
            &absolute_trace(CoordinateSpace::WindowPhysicalPixels),
            witness(),
            None,
        )
        .unwrap();

        let unknown_field = encoded.replacen("frames:", "unexpected_field: 1,\n    frames:", 1);
        assert!(matches!(
            import_automation_input_trace_v2(unknown_field.as_bytes()),
            Err(AutomationInputTraceImportError::UnknownField(_))
        ));

        let unknown_variant =
            encoded.replacen("AbsolutePointerPosition(", "FuturePointerPosition(", 1);
        assert!(matches!(
            import_automation_input_trace_v2(unknown_variant.as_bytes()),
            Err(AutomationInputTraceImportError::UnknownVariant(_))
        ));

        let false_witness = encoded.replacen(
            "recorded_sources_pristine_at_capture_start: true",
            "recorded_sources_pristine_at_capture_start: false",
            1,
        );
        assert_eq!(
            import_automation_input_trace_v2(false_witness.as_bytes()),
            Err(AutomationInputTraceImportError::UnsupportedRecordingWitness)
        );
    }

    #[test]
    fn v2_keeps_provenance_and_frame_resource_limits() {
        let trace = absolute_trace(CoordinateSpace::WindowPhysicalPixels);
        let provenance = AutomationInputTraceProvenance {
            label: Some("x".repeat(MAX_METADATA_STRING_BYTES + 1)),
            ..AutomationInputTraceProvenance::default()
        };
        assert_eq!(
            export_automation_input_trace_v2(&trace, witness(), Some(&provenance)),
            Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "metadata_string_bytes"
            ))
        );

        let too_many_frames = AutomationInputTrace {
            frames: (0..=MAX_FRAMES)
                .map(|index| AutomationInputTraceFrame {
                    frame_ordinal: index as u64,
                    groups: Vec::new(),
                })
                .collect(),
            trailing_groups: Vec::new(),
        };
        assert_eq!(
            export_automation_input_trace_v2(&too_many_frames, witness(), None),
            Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "frames"
            ))
        );
    }

    #[test]
    fn version_aware_import_rejects_wrong_artifact_kind_before_full_decode() {
        let wrong_kind = "(artifact_kind: \"other\", schema_version: 2)";
        assert_eq!(
            import_automation_input_trace(wrong_kind.as_bytes()),
            Err(AutomationInputTraceImportError::WrongArtifactKind(
                "other".to_owned()
            ))
        );
    }
}
