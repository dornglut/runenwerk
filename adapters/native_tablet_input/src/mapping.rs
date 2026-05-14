//! Mapping from native tablet DTOs into platform-neutral UI input events.

use ui_input::{
    PointerBarrelButtons, PointerContactState, PointerDeviceId, PointerEvent, PointerPacket,
    PointerSampleRole, PointerToolKind, UiInputEvent,
};

use crate::model::{
    NativeTabletCapabilities, NativeTabletCapabilityKind, NativeTabletDiagnostic,
    NativeTabletPacket, calibrated_position, calibrated_pressure,
};

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTabletMapping {
    pub event: UiInputEvent,
    pub diagnostics: Vec<NativeTabletDiagnostic>,
}

pub fn map_native_tablet_packet(packet: &NativeTabletPacket) -> NativeTabletMapping {
    let capabilities = packet.capabilities;
    let pointer_capabilities = ui_input::PointerDeviceCapabilities::from(capabilities);
    let diagnostics = missing_capability_diagnostics(capabilities);
    let pointer_packet = PointerPacket {
        source_kind: packet.source_kind,
        tool_kind: pointer_tool_kind(packet),
        device_id: Some(PointerDeviceId(packet.device_id)),
        timestamp_micros: packet.timestamp_micros,
        contact: if capabilities.hover {
            packet.contact
        } else {
            PointerContactState::Contact
        },
        pressure: calibrated_pressure(packet.pressure, capabilities, packet.calibration),
        tilt: packet.tilt.filter(|_| capabilities.tilt),
        twist_degrees: packet.twist_degrees.filter(|_| capabilities.twist),
        tangential_pressure: packet
            .tangential_pressure
            .filter(|_| capabilities.tangential_pressure),
        eraser: packet.eraser && capabilities.eraser,
        barrel_buttons: if capabilities.barrel_buttons {
            packet.barrel_buttons
        } else {
            PointerBarrelButtons::none()
        },
        capabilities: pointer_capabilities,
        calibration: packet.calibration.filter(|_| capabilities.calibration),
        latency_class: packet.latency_class,
        coalesced_samples: packet
            .coalesced_samples
            .iter()
            .copied()
            .map(|sample| {
                sample.into_pointer_sample(
                    PointerSampleRole::Coalesced,
                    capabilities,
                    packet.calibration,
                )
            })
            .collect(),
        predicted_samples: packet
            .predicted_samples
            .iter()
            .copied()
            .map(|sample| {
                sample.into_pointer_sample(
                    PointerSampleRole::Predicted,
                    capabilities,
                    packet.calibration,
                )
            })
            .collect(),
    };
    let position = calibrated_position(packet.position, packet.calibration);
    let event = PointerEvent {
        kind: packet.kind,
        position,
        delta: packet.delta,
        button: packet.event_button,
        modifiers: packet.modifiers,
        click_count: packet.click_count,
        packet: pointer_packet,
    };

    NativeTabletMapping {
        event: UiInputEvent::Pointer(event),
        diagnostics,
    }
}

fn pointer_tool_kind(packet: &NativeTabletPacket) -> PointerToolKind {
    if packet.eraser && packet.capabilities.eraser {
        PointerToolKind::Eraser
    } else {
        PointerToolKind::from(packet.tool_kind)
    }
}

fn missing_capability_diagnostics(
    capabilities: NativeTabletCapabilities,
) -> Vec<NativeTabletDiagnostic> {
    let mut diagnostics = Vec::new();
    push_missing(
        &mut diagnostics,
        capabilities.pressure,
        NativeTabletCapabilityKind::Pressure,
    );
    push_missing(
        &mut diagnostics,
        capabilities.tilt,
        NativeTabletCapabilityKind::Tilt,
    );
    push_missing(
        &mut diagnostics,
        capabilities.twist,
        NativeTabletCapabilityKind::Twist,
    );
    push_missing(
        &mut diagnostics,
        capabilities.tangential_pressure,
        NativeTabletCapabilityKind::TangentialPressure,
    );
    push_missing(
        &mut diagnostics,
        capabilities.hover,
        NativeTabletCapabilityKind::Hover,
    );
    push_missing(
        &mut diagnostics,
        capabilities.eraser,
        NativeTabletCapabilityKind::Eraser,
    );
    push_missing(
        &mut diagnostics,
        capabilities.barrel_buttons,
        NativeTabletCapabilityKind::BarrelButtons,
    );
    push_missing(
        &mut diagnostics,
        capabilities.coalesced_samples,
        NativeTabletCapabilityKind::CoalescedSamples,
    );
    push_missing(
        &mut diagnostics,
        capabilities.predicted_samples,
        NativeTabletCapabilityKind::PredictedSamples,
    );
    push_missing(
        &mut diagnostics,
        capabilities.calibration,
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
    use crate::{
        NativeTabletCapabilities, NativeTabletSample,
        model::{NativeTabletPacket, NativeTabletToolKind},
    };
    use ui_input::{
        PointerBarrelButtons, PointerButton, PointerCalibration, PointerDelta, PointerEventKind,
        PointerLatencyClass, PointerPosition, PointerSampleRole, PointerSourceKind, PointerTilt,
    };

    #[test]
    fn macos_wacom_packet_maps_pressure_tilt_and_samples() {
        let packet = NativeTabletPacket::macos_wacom(
            314,
            PointerEventKind::Move,
            PointerPosition::new(42.0, 24.0),
            PointerDelta::new(1.0, 2.0),
        )
        .with_timestamp_micros(25_000)
        .with_pressure(0.7)
        .with_tilt(PointerTilt::new(-20.0, 15.0))
        .with_twist_degrees(45.0)
        .with_barrel_buttons(PointerBarrelButtons {
            primary: true,
            secondary: false,
        })
        .with_latency_class(PointerLatencyClass::LowLatencyPreview)
        .with_coalesced_samples([NativeTabletSample::new(
            PointerPosition::new(40.0, 21.0),
            PointerDelta::new(0.5, 1.0),
        )
        .with_timestamp_micros(24_900)
        .with_pressure(0.6)
        .with_tilt(PointerTilt::new(-18.0, 12.0))])
        .with_predicted_samples([NativeTabletSample::new(
            PointerPosition::new(44.0, 27.0),
            PointerDelta::new(2.0, 3.0),
        )
        .with_timestamp_micros(25_100)
        .with_pressure(0.75)]);

        let mapping = map_native_tablet_packet(&packet);
        assert!(mapping.diagnostics.is_empty());
        let UiInputEvent::Pointer(event) = mapping.event else {
            panic!("native tablet mapping must produce a pointer event");
        };

        assert_eq!(event.kind, PointerEventKind::Move);
        assert_eq!(event.position, PointerPosition::new(42.0, 24.0));
        assert_eq!(event.packet.source_kind, PointerSourceKind::Stylus);
        assert_eq!(event.packet.device_id, Some(PointerDeviceId(314)));
        assert_eq!(event.packet.timestamp_micros, Some(25_000));
        assert_eq!(event.packet.pressure, Some(0.7));
        assert_eq!(event.packet.tilt, Some(PointerTilt::new(-20.0, 15.0)));
        assert_eq!(event.packet.twist_degrees, Some(45.0));
        assert!(event.packet.barrel_buttons.primary);
        assert_eq!(
            event.packet.latency_class,
            PointerLatencyClass::LowLatencyPreview
        );
        assert_eq!(event.packet.coalesced_samples.len(), 1);
        assert_eq!(
            event.packet.coalesced_samples[0].role,
            PointerSampleRole::Coalesced
        );
        assert_eq!(event.packet.predicted_samples.len(), 1);
        assert_eq!(
            event.packet.predicted_samples[0].role,
            PointerSampleRole::Predicted
        );
        assert!(event.packet.is_valid());
    }

    #[test]
    fn eraser_packet_routes_to_eraser_tool() {
        let packet = NativeTabletPacket::macos_wacom(
            12,
            PointerEventKind::Down,
            PointerPosition::new(3.0, 4.0),
            PointerDelta::ZERO,
        )
        .with_eraser(true)
        .with_event_button(Some(PointerButton::Primary));

        let mapping = map_native_tablet_packet(&packet);
        let UiInputEvent::Pointer(event) = mapping.event else {
            panic!("native tablet mapping must produce a pointer event");
        };

        assert_eq!(event.button, Some(PointerButton::Primary));
        assert!(event.packet.eraser);
        assert_eq!(event.packet.tool_kind, PointerToolKind::Eraser);
        assert!(event.packet.capabilities.eraser);
    }

    #[test]
    fn missing_capabilities_are_reported_without_fake_values() {
        let packet = NativeTabletPacket::macos_wacom(
            8,
            PointerEventKind::Move,
            PointerPosition::new(10.0, 10.0),
            PointerDelta::ZERO,
        )
        .with_pressure(0.9)
        .with_tilt(PointerTilt::new(10.0, 20.0))
        .with_contact(PointerContactState::Hover)
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

        let mapping = map_native_tablet_packet(&packet);
        assert!(
            mapping
                .diagnostics
                .contains(&NativeTabletDiagnostic::MissingCapability(
                    NativeTabletCapabilityKind::Pressure
                ))
        );
        assert!(
            mapping
                .diagnostics
                .contains(&NativeTabletDiagnostic::MissingCapability(
                    NativeTabletCapabilityKind::Tilt
                ))
        );
        assert!(
            mapping
                .diagnostics
                .contains(&NativeTabletDiagnostic::MissingCapability(
                    NativeTabletCapabilityKind::Hover
                ))
        );
        let UiInputEvent::Pointer(event) = mapping.event else {
            panic!("native tablet mapping must produce a pointer event");
        };

        assert_eq!(event.packet.pressure, None);
        assert_eq!(event.packet.tilt, None);
        assert_eq!(event.packet.contact, PointerContactState::Contact);
        assert!(!event.packet.capabilities.pressure);
        assert!(!event.packet.capabilities.tilt);
        assert!(!event.packet.capabilities.hover);
        assert!(event.packet.is_valid());
    }

    #[test]
    fn calibration_offsets_position_and_pressure() {
        let packet = NativeTabletPacket::windows_pointer(
            91,
            PointerEventKind::Move,
            PointerPosition::new(100.0, 200.0),
            PointerDelta::ZERO,
        )
        .with_pressure(0.5)
        .with_calibration(PointerCalibration {
            cursor_offset: PointerDelta::new(3.0, -2.0),
            pressure_scale: 1.5,
            pressure_bias: -0.1,
        });

        let mapping = map_native_tablet_packet(&packet);
        let UiInputEvent::Pointer(event) = mapping.event else {
            panic!("native tablet mapping must produce a pointer event");
        };

        assert_eq!(event.position, PointerPosition::new(103.0, 198.0));
        assert_eq!(event.packet.pressure, Some(0.65));
    }

    #[test]
    fn windows_wintab_constructor_uses_wacom_backend_and_full_capabilities() {
        let packet = NativeTabletPacket::windows_wintab(
            44,
            PointerEventKind::Move,
            PointerPosition::new(1.0, 2.0),
            PointerDelta::ZERO,
        )
        .with_tool_kind(NativeTabletToolKind::Airbrush);

        assert_eq!(
            packet.backend,
            crate::NativeTabletBackendKind::WindowsWintab
        );
        assert_eq!(packet.vendor, crate::NativeTabletVendor::Wacom);
        assert!(packet.capabilities.tangential_pressure);
    }
}
