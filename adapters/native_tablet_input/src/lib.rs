//! File: adapters/native_tablet_input/src/lib.rs
//! Purpose: Native tablet packet normalization into platform-neutral UI input events.

use ui_input::{
    Modifiers, PointerBarrelButtons, PointerButton, PointerCalibration, PointerContactState,
    PointerDelta, PointerDeviceCapabilities, PointerDeviceId, PointerEvent, PointerEventKind,
    PointerLatencyClass, PointerPacket, PointerPosition, PointerSample, PointerSampleRole,
    PointerSourceKind, PointerTilt, PointerToolKind, UiInputEvent,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletPlatform {
    Macos,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletVendor {
    Wacom,
    Generic,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletToolKind {
    Pen,
    Brush,
    Marker,
    Airbrush,
    Eraser,
    Unknown,
}

impl From<NativeTabletToolKind> for PointerToolKind {
    fn from(value: NativeTabletToolKind) -> Self {
        match value {
            NativeTabletToolKind::Pen => Self::Pen,
            NativeTabletToolKind::Brush => Self::Brush,
            NativeTabletToolKind::Marker => Self::Marker,
            NativeTabletToolKind::Airbrush => Self::Airbrush,
            NativeTabletToolKind::Eraser => Self::Eraser,
            NativeTabletToolKind::Unknown => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletCapabilityKind {
    Pressure,
    Tilt,
    Twist,
    TangentialPressure,
    Hover,
    Eraser,
    BarrelButtons,
    CoalescedSamples,
    PredictedSamples,
    Calibration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletDiagnostic {
    MissingCapability(NativeTabletCapabilityKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct NativeTabletCapabilities {
    pub pressure: bool,
    pub tilt: bool,
    pub twist: bool,
    pub tangential_pressure: bool,
    pub hover: bool,
    pub eraser: bool,
    pub barrel_buttons: bool,
    pub coalesced_samples: bool,
    pub predicted_samples: bool,
    pub calibration: bool,
}

impl NativeTabletCapabilities {
    pub const fn macos_wacom() -> Self {
        Self {
            pressure: true,
            tilt: true,
            twist: true,
            tangential_pressure: true,
            hover: true,
            eraser: true,
            barrel_buttons: true,
            coalesced_samples: true,
            predicted_samples: true,
            calibration: true,
        }
    }
}

impl From<NativeTabletCapabilities> for PointerDeviceCapabilities {
    fn from(value: NativeTabletCapabilities) -> Self {
        Self {
            pressure: value.pressure,
            tilt: value.tilt,
            twist: value.twist,
            tangential_pressure: value.tangential_pressure,
            hover: value.hover,
            eraser: value.eraser,
            barrel_buttons: value.barrel_buttons,
            coalesced_samples: value.coalesced_samples,
            predicted_samples: value.predicted_samples,
            calibration: value.calibration,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletSample {
    pub position: PointerPosition,
    pub delta: PointerDelta,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<PointerTilt>,
    pub twist_degrees: Option<f32>,
    pub tangential_pressure: Option<f32>,
    pub contact: PointerContactState,
}

impl NativeTabletSample {
    pub const fn new(position: PointerPosition, delta: PointerDelta) -> Self {
        Self {
            position,
            delta,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            tangential_pressure: None,
            contact: PointerContactState::Contact,
        }
    }

    pub fn with_timestamp_micros(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = Some(timestamp_micros);
        self
    }

    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = Some(pressure);
        self
    }

    pub fn with_tilt(mut self, tilt: PointerTilt) -> Self {
        self.tilt = Some(tilt);
        self
    }

    pub fn with_contact(mut self, contact: PointerContactState) -> Self {
        self.contact = contact;
        self
    }

    fn into_pointer_sample(
        self,
        role: PointerSampleRole,
        capabilities: NativeTabletCapabilities,
    ) -> PointerSample {
        PointerSample {
            role,
            position: self.position,
            delta: self.delta,
            timestamp_micros: self.timestamp_micros,
            pressure: self.pressure.filter(|_| capabilities.pressure),
            tilt: self.tilt.filter(|_| capabilities.tilt),
            twist_degrees: self.twist_degrees.filter(|_| capabilities.twist),
            tangential_pressure: self
                .tangential_pressure
                .filter(|_| capabilities.tangential_pressure),
            contact: if capabilities.hover {
                self.contact
            } else {
                PointerContactState::Contact
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTabletPacket {
    pub platform: NativeTabletPlatform,
    pub vendor: NativeTabletVendor,
    pub device_id: u64,
    pub kind: PointerEventKind,
    pub position: PointerPosition,
    pub delta: PointerDelta,
    pub event_button: Option<PointerButton>,
    pub modifiers: Modifiers,
    pub click_count: u8,
    pub tool_kind: NativeTabletToolKind,
    pub timestamp_micros: Option<u64>,
    pub contact: PointerContactState,
    pub pressure: Option<f32>,
    pub tilt: Option<PointerTilt>,
    pub twist_degrees: Option<f32>,
    pub tangential_pressure: Option<f32>,
    pub eraser: bool,
    pub barrel_buttons: PointerBarrelButtons,
    pub capabilities: NativeTabletCapabilities,
    pub calibration: Option<PointerCalibration>,
    pub latency_class: PointerLatencyClass,
    pub coalesced_samples: Vec<NativeTabletSample>,
    pub predicted_samples: Vec<NativeTabletSample>,
}

impl NativeTabletPacket {
    pub fn macos_wacom(
        device_id: u64,
        kind: PointerEventKind,
        position: PointerPosition,
        delta: PointerDelta,
    ) -> Self {
        Self {
            platform: NativeTabletPlatform::Macos,
            vendor: NativeTabletVendor::Wacom,
            device_id,
            kind,
            position,
            delta,
            event_button: None,
            modifiers: Modifiers::default(),
            click_count: 0,
            tool_kind: NativeTabletToolKind::Pen,
            timestamp_micros: None,
            contact: PointerContactState::Contact,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            tangential_pressure: None,
            eraser: false,
            barrel_buttons: PointerBarrelButtons::none(),
            capabilities: NativeTabletCapabilities::macos_wacom(),
            calibration: None,
            latency_class: PointerLatencyClass::Normal,
            coalesced_samples: Vec::new(),
            predicted_samples: Vec::new(),
        }
    }

    pub fn with_event_button(mut self, event_button: Option<PointerButton>) -> Self {
        self.event_button = event_button;
        self
    }

    pub fn with_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    pub fn with_timestamp_micros(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = Some(timestamp_micros);
        self
    }

    pub fn with_tool_kind(mut self, tool_kind: NativeTabletToolKind) -> Self {
        self.tool_kind = tool_kind;
        self
    }

    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = Some(pressure);
        self
    }

    pub fn with_tilt(mut self, tilt: PointerTilt) -> Self {
        self.tilt = Some(tilt);
        self
    }

    pub fn with_twist_degrees(mut self, twist_degrees: f32) -> Self {
        self.twist_degrees = Some(twist_degrees);
        self
    }

    pub fn with_contact(mut self, contact: PointerContactState) -> Self {
        self.contact = contact;
        self
    }

    pub fn with_eraser(mut self, eraser: bool) -> Self {
        self.eraser = eraser;
        if eraser {
            self.tool_kind = NativeTabletToolKind::Eraser;
        }
        self
    }

    pub fn with_barrel_buttons(mut self, barrel_buttons: PointerBarrelButtons) -> Self {
        self.barrel_buttons = barrel_buttons;
        self
    }

    pub fn with_capabilities(mut self, capabilities: NativeTabletCapabilities) -> Self {
        self.capabilities = capabilities;
        self
    }

    pub fn with_calibration(mut self, calibration: PointerCalibration) -> Self {
        self.calibration = Some(calibration);
        self
    }

    pub fn with_latency_class(mut self, latency_class: PointerLatencyClass) -> Self {
        self.latency_class = latency_class;
        self
    }

    pub fn with_coalesced_samples(
        mut self,
        samples: impl IntoIterator<Item = NativeTabletSample>,
    ) -> Self {
        self.coalesced_samples = samples.into_iter().collect();
        self
    }

    pub fn with_predicted_samples(
        mut self,
        samples: impl IntoIterator<Item = NativeTabletSample>,
    ) -> Self {
        self.predicted_samples = samples.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTabletMapping {
    pub event: UiInputEvent,
    pub diagnostics: Vec<NativeTabletDiagnostic>,
}

pub fn map_native_tablet_packet(packet: &NativeTabletPacket) -> NativeTabletMapping {
    let capabilities = packet.capabilities;
    let pointer_capabilities = PointerDeviceCapabilities::from(capabilities);
    let diagnostics = missing_capability_diagnostics(capabilities);
    let pointer_packet = PointerPacket {
        source_kind: PointerSourceKind::Stylus,
        tool_kind: pointer_tool_kind(packet),
        device_id: Some(PointerDeviceId(packet.device_id)),
        timestamp_micros: packet.timestamp_micros,
        contact: if capabilities.hover {
            packet.contact
        } else {
            PointerContactState::Contact
        },
        pressure: packet.pressure.filter(|_| capabilities.pressure),
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
            .map(|sample| sample.into_pointer_sample(PointerSampleRole::Coalesced, capabilities))
            .collect(),
        predicted_samples: packet
            .predicted_samples
            .iter()
            .copied()
            .map(|sample| sample.into_pointer_sample(PointerSampleRole::Predicted, capabilities))
            .collect(),
    };
    let event = PointerEvent {
        kind: packet.kind,
        position: packet.position,
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
}
