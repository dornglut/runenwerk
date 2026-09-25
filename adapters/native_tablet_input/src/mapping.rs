//! Translation from native tablet DTOs into RunenInput observations.

use runen_input::{
    ContactId, ContactPhase, ContactPresence, DeliveryRole, EvidenceStatus, InputContext,
    InputObservation, InputObservationGroup, MeasurementDomain, ObservationOrigin,
    PhysicalTabletControls, SourceTimeUnit, StylusTilt, TabletCapabilities, TabletObservation,
    ToolId,
};

use crate::model::{
    NativeTabletCapabilityKind, NativeTabletContactState, NativeTabletDiagnostic,
    NativeTabletEventKind, NativeTabletPacket, NativeTabletSample, calibrated_position,
    calibrated_pressure, input_context, measurement, point, source_time, vector,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTabletMapping {
    pub group: InputObservationGroup,
    pub diagnostics: Vec<NativeTabletDiagnostic>,
}

pub fn map_native_tablet_packet(
    packet: &NativeTabletPacket,
) -> Result<NativeTabletMapping, NativeTabletDiagnostic> {
    let context = input_context(packet.backend, packet.device_id);
    let diagnostics = missing_capability_diagnostics(packet);
    let mut observations =
        Vec::with_capacity(packet.coalesced_samples.len() + packet.predicted_samples.len() + 1);

    for sample in &packet.coalesced_samples {
        observations.push(InputObservation::Tablet(tablet_observation(
            packet,
            context,
            *sample,
            DeliveryRole::HistoricalCoalesced,
            EvidenceStatus::ObservedConfirmed,
        )));
    }
    observations.push(InputObservation::Tablet(tablet_observation(
        packet,
        context,
        NativeTabletSample {
            position: packet.position,
            delta: packet.delta,
            timestamp_micros: packet.timestamp_micros,
            pressure: packet.pressure,
            tilt: packet.tilt,
            twist_degrees: packet.twist_degrees,
            tangential_pressure: packet.tangential_pressure,
            contact: packet.contact,
        },
        DeliveryRole::OrdinaryCurrent,
        EvidenceStatus::ObservedConfirmed,
    )));
    for sample in &packet.predicted_samples {
        observations.push(InputObservation::Tablet(tablet_observation(
            packet,
            context,
            *sample,
            DeliveryRole::OrdinaryCurrent,
            EvidenceStatus::PredictedProvisional,
        )));
    }

    Ok(NativeTabletMapping {
        group: InputObservationGroup::new(context, observations),
        diagnostics,
    })
}

fn tablet_observation(
    packet: &NativeTabletPacket,
    context: InputContext,
    sample: NativeTabletSample,
    delivery: DeliveryRole,
    evidence: EvidenceStatus,
) -> TabletObservation {
    let position = calibrated_position(sample.position, packet.calibration);
    TabletObservation {
        contact: ContactId::new(packet.contact_id),
        tool: packet.tool_id.map(ToolId::new),
        tool_kind: packet.tool_kind,
        phase: phase_for_event(packet.kind),
        presence: presence(sample.contact),
        position: point(position),
        delta: vector(sample.delta),
        pressure: measurement(
            calibrated_pressure(sample.pressure, packet.capabilities, packet.calibration),
            packet.capabilities.pressure,
            MeasurementDomain::NormalizedUnitInterval,
        ),
        tangential_pressure: measurement(
            sample.tangential_pressure,
            packet.capabilities.tangential_pressure,
            MeasurementDomain::SignedNormalizedUnitInterval,
        ),
        tilt: sample
            .tilt
            .filter(|_| packet.capabilities.tilt)
            .map(|tilt| StylusTilt::new(tilt.x_degrees, tilt.y_degrees)),
        twist: measurement(
            sample.twist_degrees,
            packet.capabilities.twist,
            MeasurementDomain::Degrees {
                min: 0.0,
                max: 360.0,
            },
        ),
        controls: PhysicalTabletControls {
            eraser: packet.eraser && packet.capabilities.eraser,
            barrel_primary: packet.capabilities.barrel_buttons && packet.barrel_buttons.primary,
            barrel_secondary: packet.capabilities.barrel_buttons && packet.barrel_buttons.secondary,
        },
        capabilities: TabletCapabilities {
            pressure: packet.capabilities.pressure,
            tilt: packet.capabilities.tilt,
            twist: packet.capabilities.twist,
            tangential_pressure: packet.capabilities.tangential_pressure,
            hover: packet.capabilities.hover,
            eraser: packet.capabilities.eraser,
            barrel_controls: packet.capabilities.barrel_buttons,
            historical_samples: packet.capabilities.coalesced_samples,
            predicted_samples: packet.capabilities.predicted_samples,
        },
        source_time: source_time(context, sample.timestamp_micros),
        evidence,
        delivery,
        origin: ObservationOrigin::SourceReport,
    }
}

fn phase_for_event(kind: NativeTabletEventKind) -> ContactPhase {
    match kind {
        NativeTabletEventKind::Down => ContactPhase::Begin,
        NativeTabletEventKind::Move | NativeTabletEventKind::Enter => ContactPhase::Update,
        NativeTabletEventKind::Up => ContactPhase::End,
        NativeTabletEventKind::Leave => ContactPhase::Cancel,
    }
}

fn presence(state: NativeTabletContactState) -> ContactPresence {
    match state {
        NativeTabletContactState::Hover => ContactPresence::Hover,
        NativeTabletContactState::Contact => ContactPresence::Contact,
        NativeTabletContactState::OutOfRange => ContactPresence::OutOfRange,
    }
}

fn missing_capability_diagnostics(packet: &NativeTabletPacket) -> Vec<NativeTabletDiagnostic> {
    let mut diagnostics = Vec::new();
    push_missing(
        &mut diagnostics,
        packet.capabilities.pressure,
        NativeTabletCapabilityKind::Pressure,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.tilt,
        NativeTabletCapabilityKind::Tilt,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.twist,
        NativeTabletCapabilityKind::Twist,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.tangential_pressure,
        NativeTabletCapabilityKind::TangentialPressure,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.hover,
        NativeTabletCapabilityKind::Hover,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.eraser,
        NativeTabletCapabilityKind::Eraser,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.barrel_buttons,
        NativeTabletCapabilityKind::BarrelButtons,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.coalesced_samples,
        NativeTabletCapabilityKind::CoalescedSamples,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.predicted_samples,
        NativeTabletCapabilityKind::PredictedSamples,
    );
    push_missing(
        &mut diagnostics,
        packet.capabilities.calibration,
        NativeTabletCapabilityKind::Calibration,
    );
    diagnostics
}

fn push_missing(
    diagnostics: &mut Vec<NativeTabletDiagnostic>,
    available: bool,
    capability: NativeTabletCapabilityKind,
) {
    if !available {
        diagnostics.push(NativeTabletDiagnostic::MissingCapability(capability));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        NativeTabletBarrelButtons, NativeTabletCapabilities, NativeTabletDelta,
        NativeTabletLatencyClass, NativeTabletPosition, NativeTabletTilt,
    };
    use engine::plugins::InputState;

    #[test]
    fn mapping_preserves_neutral_identity_and_orthogonal_sample_roles() {
        let packet = NativeTabletPacket::macos_nsevent(
            314,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(42.0, 24.0),
            NativeTabletDelta::new(1.0, 2.0),
        )
        .with_timestamp_micros(25_000)
        .with_pressure(0.7)
        .with_tilt(NativeTabletTilt::new(-20.0, 15.0))
        .with_twist_degrees(45.0)
        .with_barrel_buttons(NativeTabletBarrelButtons {
            primary: true,
            secondary: false,
        })
        .with_latency_class(NativeTabletLatencyClass::LowLatencyPreview)
        .with_coalesced_samples([NativeTabletSample::new(
            NativeTabletPosition::new(40.0, 21.0),
            NativeTabletDelta::new(0.5, 1.0),
        )
        .with_timestamp_micros(24_900)
        .with_pressure(0.6)])
        .with_predicted_samples([NativeTabletSample::new(
            NativeTabletPosition::new(44.0, 27.0),
            NativeTabletDelta::new(2.0, 3.0),
        )
        .with_timestamp_micros(25_100)
        .with_pressure(0.75)]);

        let mapping = map_native_tablet_packet(&packet).expect("packet should map");
        assert_eq!(mapping.group.context.device.unwrap().raw(), 314);
        assert_eq!(mapping.group.observations.len(), 3);
        let InputObservation::Tablet(history) = &mapping.group.observations[0] else {
            panic!("history should be tablet observation")
        };
        assert_eq!(history.delivery, DeliveryRole::HistoricalCoalesced);
        assert_eq!(history.evidence, EvidenceStatus::ObservedConfirmed);
        let InputObservation::Tablet(predicted) = &mapping.group.observations[2] else {
            panic!("prediction should be tablet observation")
        };
        assert_eq!(predicted.evidence, EvidenceStatus::PredictedProvisional);
        assert_eq!(predicted.delivery, DeliveryRole::OrdinaryCurrent);
        assert_eq!(
            predicted.source_time.unwrap().unit,
            SourceTimeUnit::Microseconds
        );
    }

    #[test]
    fn native_backend_source_clocks_remain_distinct_for_equal_device_ids() {
        let windows = map_native_tablet_packet(
            &NativeTabletPacket::windows_pointer(
                314,
                NativeTabletEventKind::Move,
                NativeTabletPosition::new(1.0, 2.0),
                NativeTabletDelta::ZERO,
            )
            .with_timestamp_micros(10),
        )
        .expect("Windows Pointer packet should map");
        let wintab = map_native_tablet_packet(
            &NativeTabletPacket::windows_wintab(
                314,
                NativeTabletEventKind::Move,
                NativeTabletPosition::new(1.0, 2.0),
                NativeTabletDelta::ZERO,
            )
            .with_timestamp_micros(10),
        )
        .expect("Wintab packet should map");

        assert_ne!(windows.group.context.source, wintab.group.context.source);
        let InputObservation::Tablet(windows_observation) = &windows.group.observations[0] else {
            panic!("Windows observation should be a tablet observation")
        };
        let InputObservation::Tablet(wintab_observation) = &wintab.group.observations[0] else {
            panic!("Wintab observation should be a tablet observation")
        };
        assert_eq!(
            windows_observation.source_time.unwrap().context,
            windows.group.context
        );
        assert_eq!(
            wintab_observation.source_time.unwrap().context,
            wintab.group.context
        );
    }

    #[test]
    fn missing_capabilities_do_not_fabricate_measurements() {
        let packet = NativeTabletPacket::windows_pointer(
            8,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(10.0, 10.0),
            NativeTabletDelta::ZERO,
        )
        .with_pressure(0.9)
        .with_tilt(NativeTabletTilt::new(10.0, 20.0))
        .with_contact(NativeTabletContactState::Hover)
        .with_capabilities(NativeTabletCapabilities {
            pressure: false,
            tilt: false,
            twist: false,
            tangential_pressure: false,
            hover: false,
            eraser: true,
            barrel_buttons: true,
            coalesced_samples: true,
            predicted_samples: true,
            calibration: true,
        });

        let mapping = map_native_tablet_packet(&packet).expect("packet should map");
        assert!(
            mapping
                .diagnostics
                .contains(&NativeTabletDiagnostic::MissingCapability(
                    NativeTabletCapabilityKind::Pressure
                ))
        );
        let InputObservation::Tablet(current) = &mapping.group.observations[0] else {
            panic!("current observation should be tablet observation")
        };
        assert_eq!(current.pressure, None);
        assert_eq!(current.tilt, None);
        assert_eq!(current.presence, ContactPresence::Hover);
    }

    #[test]
    fn invalid_measurement_is_rejected_by_neutral_admission() {
        let packet = NativeTabletPacket::windows_pointer(
            91,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(100.0, 200.0),
            NativeTabletDelta::ZERO,
        )
        .with_pressure(1.5);
        let mapping = map_native_tablet_packet(&packet).expect("mapping does not clamp input");
        let mut input = InputState::new();
        assert!(input.admit_device_observation_group(mapping.group).is_err());
        assert!(input.drain_device_observation_groups().is_empty());
    }
}
