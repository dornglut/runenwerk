//! macOS AppKit/NSEvent tablet backend DTO mapping.

use ui_input::{
    PointerBarrelButtons, PointerButton, PointerCalibration, PointerContactState, PointerDelta,
    PointerEventKind, PointerLatencyClass, PointerPosition, PointerTilt,
};

use crate::backend::NativeTabletBackendAdapter;
use crate::model::{
    NativeTabletBackendHealth, NativeTabletBackendKind, NativeTabletCapabilities,
    NativeTabletDeviceControlResource, NativeTabletPacket, NativeTabletRuntimeResource,
    NativeTabletToolKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacosNseventTabletSubtype {
    Pen,
    Eraser,
    Cursor,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacosNseventPacketDto {
    pub device_id: u64,
    pub subtype: MacosNseventTabletSubtype,
    pub kind: PointerEventKind,
    pub position: PointerPosition,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<PointerTilt>,
    pub rotation_degrees: Option<f32>,
    pub tangential_pressure: Option<f32>,
    pub barrel_buttons: PointerBarrelButtons,
    pub in_proximity: bool,
}

impl MacosNseventPacketDto {
    pub fn new(
        device_id: u64,
        subtype: MacosNseventTabletSubtype,
        kind: PointerEventKind,
        position: PointerPosition,
    ) -> Self {
        Self {
            device_id,
            subtype,
            kind,
            position,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            rotation_degrees: None,
            tangential_pressure: None,
            barrel_buttons: PointerBarrelButtons::none(),
            in_proximity: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct MacosNseventBackend {
    probed: bool,
}

impl MacosNseventBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NativeTabletBackendAdapter for MacosNseventBackend {
    fn kind(&self) -> NativeTabletBackendKind {
        NativeTabletBackendKind::MacosNsevent
    }

    fn attach(
        &mut self,
        _window: &winit::window::Window,
        runtime: &mut NativeTabletRuntimeResource,
        control: &NativeTabletDeviceControlResource,
    ) {
        if !control
            .backend_preference
            .accepts(NativeTabletBackendKind::MacosNsevent)
        {
            runtime.set_backend_health(NativeTabletBackendHealth::available(
                NativeTabletBackendKind::MacosNsevent,
                "macOS NSEvent backend disabled by backend preference",
            ));
            return;
        }
        if self.probed {
            return;
        }
        self.probed = true;
        runtime.set_backend_health(platform::probe_macos_nsevent());
    }
}

pub fn map_macos_nsevent_packet(
    dto: MacosNseventPacketDto,
    previous_position: Option<PointerPosition>,
    calibration: PointerCalibration,
) -> NativeTabletPacket {
    let delta = previous_position
        .map(|previous| PointerDelta::new(dto.position.x - previous.x, dto.position.y - previous.y))
        .unwrap_or(PointerDelta::ZERO);
    let eraser = dto.subtype == MacosNseventTabletSubtype::Eraser;
    let capabilities = NativeTabletCapabilities {
        pressure: dto.pressure.is_some(),
        tilt: dto.tilt.is_some(),
        twist: dto.rotation_degrees.is_some(),
        tangential_pressure: dto.tangential_pressure.is_some(),
        hover: true,
        eraser,
        barrel_buttons: dto.barrel_buttons.primary || dto.barrel_buttons.secondary,
        coalesced_samples: false,
        predicted_samples: false,
        calibration: true,
    };

    let mut packet =
        NativeTabletPacket::macos_nsevent(dto.device_id, dto.kind, dto.position, delta)
            .with_tool_kind(match dto.subtype {
                MacosNseventTabletSubtype::Pen => NativeTabletToolKind::Pen,
                MacosNseventTabletSubtype::Eraser => NativeTabletToolKind::Eraser,
                MacosNseventTabletSubtype::Cursor | MacosNseventTabletSubtype::Unknown => {
                    NativeTabletToolKind::Unknown
                }
            })
            .with_capabilities(capabilities)
            .with_calibration(calibration)
            .with_latency_class(PointerLatencyClass::LowLatencyPreview)
            .with_contact(if dto.in_proximity {
                PointerContactState::Contact
            } else {
                PointerContactState::OutOfRange
            })
            .with_event_button(event_button_for_kind(dto.kind));

    if let Some(timestamp) = dto.timestamp_micros {
        packet = packet.with_timestamp_micros(timestamp);
    }
    if let Some(pressure) = dto.pressure {
        packet = packet.with_pressure(pressure);
    }
    if let Some(tilt) = dto.tilt {
        packet = packet.with_tilt(tilt);
    }
    if let Some(rotation) = dto.rotation_degrees {
        packet = packet.with_twist_degrees(rotation);
    }
    if let Some(tangential_pressure) = dto.tangential_pressure {
        packet = packet.with_tangential_pressure(tangential_pressure);
    }
    if eraser {
        packet = packet.with_eraser(true);
    }
    if dto.barrel_buttons.primary || dto.barrel_buttons.secondary {
        packet = packet.with_barrel_buttons(dto.barrel_buttons);
    }

    packet
}

fn event_button_for_kind(kind: PointerEventKind) -> Option<PointerButton> {
    match kind {
        PointerEventKind::Down | PointerEventKind::Up | PointerEventKind::Move => {
            Some(PointerButton::Primary)
        }
        PointerEventKind::Enter | PointerEventKind::Leave | PointerEventKind::Scroll => None,
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::*;

    pub(super) fn probe_macos_nsevent() -> NativeTabletBackendHealth {
        NativeTabletBackendHealth::available(
            NativeTabletBackendKind::MacosNsevent,
            "macOS AppKit tablet DTO mapping is available; event monitor installation must run on macOS",
        )
    }
}

#[cfg(not(target_os = "macos"))]
mod platform {
    use super::*;

    pub(super) fn probe_macos_nsevent() -> NativeTabletBackendHealth {
        NativeTabletBackendHealth::unavailable(
            NativeTabletBackendKind::MacosNsevent,
            "macOS NSEvent tablet data is only available on macOS",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_nsevent_packet_preserves_public_tablet_fields() {
        let dto = MacosNseventPacketDto {
            device_id: 22,
            subtype: MacosNseventTabletSubtype::Pen,
            kind: PointerEventKind::Move,
            position: PointerPosition::new(60.0, 70.0),
            timestamp_micros: Some(200),
            pressure: Some(0.5),
            tilt: Some(PointerTilt::new(-10.0, 25.0)),
            rotation_degrees: Some(90.0),
            tangential_pressure: Some(0.3),
            barrel_buttons: PointerBarrelButtons {
                primary: true,
                secondary: true,
            },
            in_proximity: true,
        };

        let packet = map_macos_nsevent_packet(
            dto,
            Some(PointerPosition::new(50.0, 65.0)),
            PointerCalibration::identity(),
        );

        assert_eq!(packet.backend, NativeTabletBackendKind::MacosNsevent);
        assert_eq!(packet.delta, PointerDelta::new(10.0, 5.0));
        assert_eq!(packet.pressure, Some(0.5));
        assert_eq!(packet.tilt, Some(PointerTilt::new(-10.0, 25.0)));
        assert_eq!(packet.twist_degrees, Some(90.0));
        assert_eq!(packet.tangential_pressure, Some(0.3));
        assert!(packet.barrel_buttons.primary);
        assert!(packet.barrel_buttons.secondary);
    }

    #[test]
    fn macos_nsevent_eraser_sets_eraser_tool() {
        let dto = MacosNseventPacketDto::new(
            23,
            MacosNseventTabletSubtype::Eraser,
            PointerEventKind::Down,
            PointerPosition::new(1.0, 2.0),
        );

        let packet = map_macos_nsevent_packet(dto, None, PointerCalibration::identity());

        assert!(packet.eraser);
        assert_eq!(packet.tool_kind, NativeTabletToolKind::Eraser);
    }
}
