use super::*;

fn context(source: u64, device: Option<u64>) -> InputContext {
    InputContext::new(
        InputSourceId::new(source),
        device.map(InputDeviceId::new),
    )
}

fn pointer_group(source: u64, state: DigitalState) -> InputObservationGroup {
    InputObservationGroup::single(
        context(source, None),
        InputObservation::PointerButton(PointerButtonInput {
            button: PointerButton::Left,
            state,
        }),
    )
}

fn replayable_trace() -> AutomationInputTrace {
    AutomationInputTrace {
        frames: vec![
            AutomationInputTraceFrame {
                frame_ordinal: 0,
                groups: vec![
                    pointer_group(100, DigitalState::Pressed),
                    InputObservationGroup::single(
                        context(100, None),
                        InputObservation::RelativeMotion {
                            delta: Vector2::new(4.0, -2.0),
                            unit: RelativeMotionUnit::BackendDeviceUnits,
                        },
                    ),
                ],
            },
            AutomationInputTraceFrame {
                frame_ordinal: 1,
                groups: Vec::new(),
            },
            AutomationInputTraceFrame {
                frame_ordinal: 2,
                groups: vec![
                    pointer_group(100, DigitalState::Released),
                    InputObservationGroup::single(
                        context(100, None),
                        InputObservation::Scroll(ScrollInput {
                            delta: ScrollDelta::two_dimensional(0.5, 1.0),
                            domain: ScrollDomain::Lines,
                            phase: Some(ScrollPhase::Update),
                        }),
                    ),
                ],
            },
        ],
        trailing_groups: Vec::new(),
    }
}

fn witness() -> AutomationInputTraceRecordingWitness {
    AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart
}

fn encode(trace: &AutomationInputTrace) -> String {
    export_automation_input_trace_v1(trace, witness(), None).expect("trace should export")
}

#[test]
fn v1_emits_exact_identity_and_round_trips_supported_ordinary_input() {
    let source = encode(&replayable_trace());
    assert!(source.contains(AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND));
    assert!(source.contains("schema_version: 1"));
    assert!(source.contains("RecordedSourcesPristineAtCaptureStart"));
    assert!(!source.contains("source: 100"));

    let imported = import_automation_input_trace_v1(source.as_bytes()).expect("V1 should import");
    assert_eq!(imported.recording_witness(), witness());
    assert_eq!(imported.trace().frames().len(), 3);
    assert!(imported.trace().frames()[1].groups().is_empty());
    assert!(imported.trace().trailing_groups().is_empty());

    let groups = imported.trace().frames()[0].groups();
    assert_eq!(groups[0].context.source, InputSourceId::new(1));
    assert_eq!(
        groups[1].observations,
        vec![InputObservation::RelativeMotion {
            delta: Vector2::new(4.0, -2.0),
            unit: RelativeMotionUnit::BackendDeviceUnits,
        }]
    );
    let scroll = &imported.trace().frames()[2].groups()[1].observations[0];
    assert_eq!(
        scroll,
        &InputObservation::Scroll(ScrollInput {
            delta: ScrollDelta::two_dimensional(0.5, 1.0),
            domain: ScrollDomain::Lines,
            phase: Some(ScrollPhase::Update),
        })
    );
}

#[test]
fn v1_source_and_device_slots_are_first_appearance_deterministic() {
    let trace = AutomationInputTrace {
        frames: vec![AutomationInputTraceFrame {
            frame_ordinal: 0,
            groups: vec![
                InputObservationGroup::single(
                    context(900, Some(44)),
                    InputObservation::RelativeMotion {
                        delta: Vector2::new(1.0, 0.0),
                        unit: RelativeMotionUnit::BackendDeviceUnits,
                    },
                ),
                InputObservationGroup::single(
                    context(800, Some(99)),
                    InputObservation::Scroll(ScrollInput {
                        delta: ScrollDelta::vertical_only(1.0),
                        domain: ScrollDomain::Unspecified,
                        phase: None,
                    }),
                ),
                InputObservationGroup::single(
                    context(900, Some(55)),
                    InputObservation::RelativeMotion {
                        delta: Vector2::new(2.0, 0.0),
                        unit: RelativeMotionUnit::BackendDeviceUnits,
                    },
                ),
            ],
        }],
        trailing_groups: Vec::new(),
    };
    let source = encode(&trace);
    let persisted: PersistedTraceV1 = ron_options().from_str(&source).unwrap();
    assert_eq!(persisted.frames[0].groups[0].source_slot, 0);
    assert_eq!(persisted.frames[0].groups[0].device_slot, Some(0));
    assert_eq!(persisted.frames[0].groups[1].source_slot, 1);
    assert_eq!(persisted.frames[0].groups[1].device_slot, Some(0));
    assert_eq!(persisted.frames[0].groups[2].source_slot, 0);
    assert_eq!(persisted.frames[0].groups[2].device_slot, Some(1));
}

#[test]
fn v1_preserves_atomic_tablet_payload_source_time_and_identity_relations() {
    let tablet_context = context(77, Some(8));
    let capabilities = TabletCapabilities {
        pressure: CapabilityKnowledge::Supported,
        tilt: CapabilityKnowledge::Supported,
        twist: CapabilityKnowledge::Supported,
        tangential_pressure: CapabilityKnowledge::Supported,
        hover: CapabilityKnowledge::Supported,
        eraser: CapabilityKnowledge::Supported,
        barrel_controls: CapabilityKnowledge::Supported,
        historical_samples: CapabilityKnowledge::Supported,
        predicted_samples: CapabilityKnowledge::Supported,
    };
    let sample = |contact: u64, delivery: DeliveryRole, evidence: EvidenceStatus| {
        InputObservation::Tablet(TabletObservation {
            contact: ContactId::new(contact),
            tool: Some(ToolId::new(55)),
            tool_kind: InputToolKind::Pen,
            phase: ContactPhase::Update,
            presence: ContactPresence::Contact,
            position: Point2::new(10.0 + contact as f32, 20.0, CoordinateSpace::WindowPhysicalPixels),
            delta: Vector2::new(1.0, -1.0),
            pressure: Some(AnalogMeasurement::new(
                0.5,
                MeasurementDomain::NormalizedUnitInterval,
            )),
            tangential_pressure: Some(AnalogMeasurement::new(
                -0.2,
                MeasurementDomain::SignedNormalizedUnitInterval,
            )),
            tilt: Some(StylusTilt::new(10.0, -20.0)),
            twist: Some(AnalogMeasurement::new(
                30.0,
                MeasurementDomain::Degrees {
                    min: 0.0,
                    max: 360.0,
                },
            )),
            controls: PhysicalTabletControls {
                eraser: false,
                barrel_primary: true,
                barrel_secondary: false,
            },
            capabilities,
            source_time: Some(SourceTime::new(
                tablet_context,
                123_456,
                SourceTimeUnit::NativeTicks {
                    ticks_per_second: 1_000_000,
                },
            )),
            evidence,
            delivery,
            origin: ObservationOrigin::SourceReport,
        })
    };
    let trace = AutomationInputTrace {
        frames: vec![AutomationInputTraceFrame {
            frame_ordinal: 0,
            groups: vec![InputObservationGroup::new(
                tablet_context,
                vec![
                    sample(
                        1,
                        DeliveryRole::HistoricalCoalesced,
                        EvidenceStatus::ObservedConfirmed,
                    ),
                    sample(
                        2,
                        DeliveryRole::OrdinaryCurrent,
                        EvidenceStatus::ObservedConfirmed,
                    ),
                ],
            )],
        }],
        trailing_groups: Vec::new(),
    };

    let source = encode(&trace);
    assert!(
        !source.contains("source: 77") && !source.contains("device: 8"),
        "raw runtime source/device IDs must not become durable identity"
    );
    let imported = import_automation_input_trace_v1(source.as_bytes()).unwrap();
    let group = &imported.trace().frames()[0].groups()[0];
    assert_eq!(group.observations.len(), 2);
    assert_eq!(group.context.source, InputSourceId::new(1));
    assert_eq!(group.context.device, Some(InputDeviceId::new(1)));
    for observation in &group.observations {
        let InputObservation::Tablet(tablet) = observation else {
            panic!("tablet group must remain atomic");
        };
        assert_eq!(tablet.tool, Some(ToolId::new(1)));
        assert_eq!(tablet.source_time.unwrap().context, group.context);
        assert_eq!(tablet.source_time.unwrap().value, 123_456);
        assert_eq!(
            tablet.source_time.unwrap().unit,
            SourceTimeUnit::NativeTicks {
                ticks_per_second: 1_000_000
            }
        );
        assert_eq!(tablet.capabilities, capabilities);
    }
    let InputObservation::Tablet(first) = &group.observations[0] else {
        unreachable!()
    };
    let InputObservation::Tablet(second) = &group.observations[1] else {
        unreachable!()
    };
    assert_ne!(first.contact, second.contact);
    assert_eq!(first.tool, second.tool);
}

#[test]
fn export_rejects_trailing_unsupported_and_release_first_input() {
    let mut trailing = replayable_trace();
    trailing.trailing_groups.push(pointer_group(100, DigitalState::Pressed));
    assert!(matches!(
        export_automation_input_trace_v1(&trailing, witness(), None),
        Err(AutomationInputTraceExportError::UnframedTrailingGroups)
    ));

    let unsupported = AutomationInputTrace {
        frames: vec![AutomationInputTraceFrame {
            frame_ordinal: 0,
            groups: vec![InputObservationGroup::single(
                context(1, None),
                InputObservation::AbsolutePointerPosition {
                    position: Point2::new(
                        1.0,
                        2.0,
                        CoordinateSpace::WindowPhysicalPixels,
                    ),
                },
            )],
        }],
        trailing_groups: Vec::new(),
    };
    assert!(matches!(
        export_automation_input_trace_v1(&unsupported, witness(), None),
        Err(AutomationInputTraceExportError::UnsupportedTraceShape(_))
    ));

    let release_first = AutomationInputTrace {
        frames: vec![AutomationInputTraceFrame {
            frame_ordinal: 0,
            groups: vec![pointer_group(1, DigitalState::Released)],
        }],
        trailing_groups: Vec::new(),
    };
    assert!(matches!(
        export_automation_input_trace_v1(&release_first, witness(), None),
        Err(AutomationInputTraceExportError::UnsupportedTraceShape(_))
    ));
}

#[test]
fn loader_probes_kind_and_version_before_strict_v1_payload() {
    let wrong_kind = "(artifact_kind:\"other\",schema_version:1,garbage:DefinitelyNotV1)";
    assert!(matches!(
        import_automation_input_trace_v1(wrong_kind.as_bytes()),
        Err(AutomationInputTraceImportError::WrongArtifactKind(found)) if found == "other"
    ));

    let future = format!(
        "(artifact_kind:\"{}\",schema_version:2,future_payload:DefinitelyNotV1)",
        AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND
    );
    assert!(matches!(
        import_automation_input_trace_v1(future.as_bytes()),
        Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(2))
    ));
}

#[test]
fn strict_v1_rejects_unknown_fields_variants_and_missing_fields() {
    let valid = encode(&replayable_trace());

    let unknown_field = valid.replacen(
        "schema_version: 1,",
        "schema_version: 1,\n    unknown_field: true,",
        1,
    );
    assert!(matches!(
        import_automation_input_trace_v1(unknown_field.as_bytes()),
        Err(AutomationInputTraceImportError::UnknownField(_))
    ));

    let unknown_variant = valid.replacen("Pressed", "Sideways", 1);
    assert!(matches!(
        import_automation_input_trace_v1(unknown_variant.as_bytes()),
        Err(AutomationInputTraceImportError::UnknownVariant(_))
    ));

    let missing = valid.replacen(
        "recording_witness: RecordedSourcesPristineAtCaptureStart,",
        "",
        1,
    );
    assert!(matches!(
        import_automation_input_trace_v1(missing.as_bytes()),
        Err(AutomationInputTraceImportError::MalformedArtifact(_))
    ));
}

#[test]
fn loader_enforces_byte_and_recursion_limits_before_materialization() {
    let too_large = vec![b' '; MAX_ARTIFACT_BYTES + 1];
    assert_eq!(
        import_automation_input_trace_v1(&too_large),
        Err(AutomationInputTraceImportError::ArtifactTooLarge)
    );

    let nested = "[".repeat(MAX_RON_RECURSION_DEPTH + 16)
        + &"]".repeat(MAX_RON_RECURSION_DEPTH + 16);
    let source = format!(
        "(artifact_kind:\"{}\",schema_version:1,deep:{nested})",
        AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND
    );
    assert!(matches!(
        import_automation_input_trace_v1(source.as_bytes()),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded("ron_recursion_depth"))
    ));
}

#[test]
fn import_rejects_non_contiguous_frames_empty_groups_and_bad_identity_slots() {
    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();
    persisted.frames[1].frame_ordinal = 9;
    let source = ron_options()
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .unwrap();
    assert!(matches!(
        import_automation_input_trace_v1(source.as_bytes()),
        Err(AutomationInputTraceImportError::UnsupportedTraceShape(_))
    ));

    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();
    persisted.frames[0].groups[0].observations.clear();
    let source = ron_options()
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .unwrap();
    assert!(matches!(
        import_automation_input_trace_v1(source.as_bytes()),
        Err(AutomationInputTraceImportError::UnsupportedTraceShape(_))
    ));

    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();
    persisted.frames[0].groups[0].source_slot = 1;
    let source = ron_options()
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .unwrap();
    assert!(matches!(
        import_automation_input_trace_v1(source.as_bytes()),
        Err(AutomationInputTraceImportError::InvalidIdentityReference(_))
    ));
}

#[test]
fn import_finishes_through_runen_input_validation() {
    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();
    let PersistedObservationV1::RelativeMotion(motion) =
        &mut persisted.frames[0].groups[1].observations[0]
    else {
        panic!("expected motion");
    };
    motion.delta.x = f32::NAN;
    let source = ron_options()
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .unwrap();
    assert!(matches!(
        import_automation_input_trace_v1(source.as_bytes()),
        Err(AutomationInputTraceImportError::InvalidNormalizedInput(_))
    ));
}

#[test]
fn metadata_is_bounded_and_round_trips_when_present() {
    let provenance = AutomationInputTraceProvenance {
        runenwerk_revision: Some("abc123".to_owned()),
        runen_input_revision: Some("def456".to_owned()),
        capture_host: Some(AutomationInputTraceCaptureHostClass::Headless),
        label: Some("camera trace".to_owned()),
        description: Some("recorded for persistence proof".to_owned()),
    };
    let source =
        export_automation_input_trace_v1(&replayable_trace(), witness(), Some(&provenance)).unwrap();
    let imported = import_automation_input_trace_v1(source.as_bytes()).unwrap();
    assert_eq!(imported.provenance(), Some(&provenance));

    let oversized = AutomationInputTraceProvenance {
        label: Some("x".repeat(MAX_METADATA_STRING_BYTES + 1)),
        ..AutomationInputTraceProvenance::default()
    };
    assert!(matches!(
        export_automation_input_trace_v1(&replayable_trace(), witness(), Some(&oversized)),
        Err(AutomationInputTraceExportError::ResourceLimitExceeded(
            "metadata_string_bytes"
        ))
    ));
}

#[test]
fn structural_resource_limits_are_enforced_before_runtime_materialization() {
    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();

    persisted.frames = (0..=MAX_FRAMES)
        .map(|index| PersistedFrameV1 {
            frame_ordinal: index as u64,
            groups: Vec::new(),
        })
        .collect();
    assert!(matches!(
        ImportBuilder::new().build(&persisted),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded("frames"))
    ));

    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();
    persisted.frames[0].groups = vec![persisted.frames[0].groups[0].clone(); MAX_GROUPS_PER_FRAME + 1];
    assert!(matches!(
        ImportBuilder::new().build(&persisted),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded(
            "groups_per_frame"
        ))
    ));

    let group = PersistedGroupV1 {
        source_slot: 0,
        device_slot: None,
        observations: vec![PersistedObservationV1::RelativeMotion(
            PersistedRelativeMotionV1 {
                delta: PersistedVector2V1 { x: 0.0, y: 0.0 },
                unit: PersistedRelativeMotionUnitV1::BackendDeviceUnits,
            },
        )],
    };
    let full_frames = MAX_TOTAL_GROUPS / MAX_GROUPS_PER_FRAME;
    let mut frames: Vec<_> = (0..full_frames)
        .map(|frame| PersistedFrameV1 {
            frame_ordinal: frame as u64,
            groups: vec![group.clone(); MAX_GROUPS_PER_FRAME],
        })
        .collect();
    frames.push(PersistedFrameV1 {
        frame_ordinal: full_frames as u64,
        groups: vec![group],
    });
    let persisted = PersistedTraceV1 {
        artifact_kind: AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND.to_owned(),
        schema_version: AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION,
        recording_witness: PersistedRecordingWitnessV1::RecordedSourcesPristineAtCaptureStart,
        provenance: None,
        frames,
    };
    assert!(matches!(
        ImportBuilder::new().build(&persisted),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded("total_groups"))
    ));

    let mut persisted: PersistedTraceV1 = ron_options().from_str(&encode(&replayable_trace())).unwrap();
    persisted.frames[0].groups[0].observations =
        vec![persisted.frames[0].groups[0].observations[0].clone(); MAX_OBSERVATIONS_PER_GROUP + 1];
    assert!(matches!(
        ImportBuilder::new().build(&persisted),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded(
            "observations_per_group"
        ))
    ));
}

#[test]
fn identity_cardinality_limits_are_enforced() {
    let mut builder = ImportBuilder::new();
    for slot in 0..MAX_SOURCES as u32 {
        builder.materialize_source(slot).unwrap();
    }
    assert!(matches!(
        builder.materialize_source(MAX_SOURCES as u32),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded("sources"))
    ));

    let mut builder = ImportBuilder::new();
    builder.materialize_source(0).unwrap();
    for slot in 0..MAX_DEVICES_PER_SOURCE as u32 {
        builder.materialize_device(0, slot).unwrap();
    }
    assert!(matches!(
        builder.materialize_device(0, MAX_DEVICES_PER_SOURCE as u32),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded(
            "devices_per_source"
        ))
    ));

    let mut builder = ImportBuilder::new();
    for slot in 0..MAX_CONTACTS_PER_CONTEXT as u32 {
        builder.materialize_contact(0, None, slot).unwrap();
    }
    assert!(matches!(
        builder.materialize_contact(0, None, MAX_CONTACTS_PER_CONTEXT as u32),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded(
            "contacts_per_context"
        ))
    ));

    let mut builder = ImportBuilder::new();
    for slot in 0..MAX_TOOLS_PER_CONTEXT as u32 {
        builder.materialize_tool(0, None, slot).unwrap();
    }
    assert!(matches!(
        builder.materialize_tool(0, None, MAX_TOOLS_PER_CONTEXT as u32),
        Err(AutomationInputTraceImportError::ResourceLimitExceeded(
            "tools_per_context"
        ))
    ));
}

#[test]
fn reexport_after_import_preserves_semantic_v1_ordering() {
    let first = encode(&replayable_trace());
    let imported = import_automation_input_trace_v1(first.as_bytes()).unwrap();
    let second = export_automation_input_trace_v1(
        imported.trace(),
        imported.recording_witness(),
        imported.provenance(),
    )
    .unwrap();
    let first_dto: PersistedTraceV1 = ron_options().from_str(&first).unwrap();
    let second_dto: PersistedTraceV1 = ron_options().from_str(&second).unwrap();
    assert_eq!(first_dto, second_dto);
}
