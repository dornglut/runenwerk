//! Optional Wacom Wintab backend.

use ui_input::{
    PointerBarrelButtons, PointerButton, PointerCalibration, PointerContactState, PointerDelta,
    PointerEventKind, PointerLatencyClass, PointerPosition, PointerTilt,
};

use crate::backend::NativeTabletBackendAdapter;
use crate::model::{
    NativeTabletBackendHealth, NativeTabletBackendKind, NativeTabletCapabilities,
    NativeTabletDeviceControlResource, NativeTabletPacket, NativeTabletRuntimeResource,
    NativeTabletSample, NativeTabletToolKind,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowsWintabPacketDto {
    pub device_id: u64,
    pub kind: PointerEventKind,
    pub position: PointerPosition,
    pub timestamp_micros: Option<u64>,
    pub normal_pressure: Option<f32>,
    pub tangential_pressure: Option<f32>,
    pub tilt: Option<PointerTilt>,
    pub rotation_degrees: Option<f32>,
    pub barrel_buttons: PointerBarrelButtons,
    pub eraser: bool,
    pub in_proximity: bool,
}

impl WindowsWintabPacketDto {
    pub fn new(device_id: u64, kind: PointerEventKind, position: PointerPosition) -> Self {
        Self {
            device_id,
            kind,
            position,
            timestamp_micros: None,
            normal_pressure: None,
            tangential_pressure: None,
            tilt: None,
            rotation_degrees: None,
            barrel_buttons: PointerBarrelButtons::none(),
            eraser: false,
            in_proximity: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct WindowsWintabBackend {
    probed: bool,
}

impl WindowsWintabBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl NativeTabletBackendAdapter for WindowsWintabBackend {
    fn kind(&self) -> NativeTabletBackendKind {
        NativeTabletBackendKind::WindowsWintab
    }

    fn attach(
        &mut self,
        _window: &winit::window::Window,
        runtime: &mut NativeTabletRuntimeResource,
        control: &NativeTabletDeviceControlResource,
    ) {
        if !control
            .backend_preference
            .accepts(NativeTabletBackendKind::WindowsWintab)
        {
            runtime.set_backend_health(NativeTabletBackendHealth::available(
                NativeTabletBackendKind::WindowsWintab,
                "Wintab backend disabled by backend preference",
            ));
            return;
        }

        if self.probed {
            return;
        }
        self.probed = true;
        runtime.set_backend_health(platform::probe_wintab());
    }
}

pub fn map_windows_wintab_packet(
    dto: WindowsWintabPacketDto,
    previous_position: Option<PointerPosition>,
    calibration: PointerCalibration,
) -> NativeTabletPacket {
    let delta = previous_position
        .map(|previous| PointerDelta::new(dto.position.x - previous.x, dto.position.y - previous.y))
        .unwrap_or(PointerDelta::ZERO);
    let capabilities = NativeTabletCapabilities {
        pressure: dto.normal_pressure.is_some(),
        tilt: dto.tilt.is_some(),
        twist: dto.rotation_degrees.is_some(),
        tangential_pressure: dto.tangential_pressure.is_some(),
        hover: true,
        eraser: dto.eraser,
        barrel_buttons: dto.barrel_buttons.primary || dto.barrel_buttons.secondary,
        coalesced_samples: false,
        predicted_samples: false,
        calibration: true,
    };

    let mut packet =
        NativeTabletPacket::windows_wintab(dto.device_id, dto.kind, dto.position, delta)
            .with_tool_kind(if dto.eraser {
                NativeTabletToolKind::Eraser
            } else {
                NativeTabletToolKind::Pen
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
    if let Some(pressure) = dto.normal_pressure {
        packet = packet.with_pressure(pressure);
    }
    if let Some(tangential_pressure) = dto.tangential_pressure {
        packet = packet.with_tangential_pressure(tangential_pressure);
    }
    if let Some(tilt) = dto.tilt {
        packet = packet.with_tilt(tilt);
    }
    if let Some(rotation) = dto.rotation_degrees {
        packet = packet.with_twist_degrees(rotation);
    }
    if dto.eraser {
        packet = packet.with_eraser(true);
    }
    if dto.barrel_buttons.primary || dto.barrel_buttons.secondary {
        packet = packet.with_barrel_buttons(dto.barrel_buttons);
    }

    packet
}

pub fn map_windows_wintab_history(
    samples: impl IntoIterator<Item = WindowsWintabPacketDto>,
    previous_position: Option<PointerPosition>,
    calibration: PointerCalibration,
) -> Option<NativeTabletPacket> {
    let mut chronological = samples.into_iter().collect::<Vec<_>>();
    let current = chronological.pop()?;
    let mut last_position = previous_position;
    let coalesced_samples = chronological
        .into_iter()
        .map(|sample| {
            let delta = last_position
                .map(|previous| {
                    PointerDelta::new(
                        sample.position.x - previous.x,
                        sample.position.y - previous.y,
                    )
                })
                .unwrap_or(PointerDelta::ZERO);
            last_position = Some(sample.position);
            native_sample_from_wintab(sample, delta)
        })
        .collect::<Vec<_>>();

    let mut packet = map_windows_wintab_packet(current, last_position, calibration);
    if !coalesced_samples.is_empty() {
        packet.capabilities.coalesced_samples = true;
        packet = packet.with_coalesced_samples(coalesced_samples);
    }
    Some(packet)
}

fn native_sample_from_wintab(
    sample: WindowsWintabPacketDto,
    delta: PointerDelta,
) -> NativeTabletSample {
    let mut native =
        NativeTabletSample::new(sample.position, delta).with_contact(if sample.in_proximity {
            PointerContactState::Contact
        } else {
            PointerContactState::OutOfRange
        });
    if let Some(timestamp) = sample.timestamp_micros {
        native = native.with_timestamp_micros(timestamp);
    }
    if let Some(pressure) = sample.normal_pressure {
        native = native.with_pressure(pressure);
    }
    if let Some(tangential_pressure) = sample.tangential_pressure {
        native = native.with_tangential_pressure(tangential_pressure);
    }
    if let Some(tilt) = sample.tilt {
        native = native.with_tilt(tilt);
    }
    if let Some(rotation) = sample.rotation_degrees {
        native = native.with_twist_degrees(rotation);
    }
    native
}

fn event_button_for_kind(kind: PointerEventKind) -> Option<PointerButton> {
    match kind {
        PointerEventKind::Down | PointerEventKind::Up | PointerEventKind::Move => {
            Some(PointerButton::Primary)
        }
        PointerEventKind::Enter | PointerEventKind::Leave | PointerEventKind::Scroll => None,
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use windows_sys::Win32::Foundation::FreeLibrary;
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};

    pub(super) fn probe_wintab() -> NativeTabletBackendHealth {
        // SAFETY: LoadLibraryA receives a static null-terminated DLL name. The module handle is
        // released before returning from this probe.
        let module = unsafe { LoadLibraryA(c"wintab32.dll".as_ptr().cast()) };
        if module.is_null() {
            return NativeTabletBackendHealth::unavailable(
                NativeTabletBackendKind::WindowsWintab,
                "wintab32.dll is not installed",
            );
        }

        // SAFETY: GetProcAddress only inspects the loaded module for exported symbols.
        let has_core_exports = unsafe {
            GetProcAddress(module, c"WTInfoA".as_ptr().cast()).is_some()
                && GetProcAddress(module, c"WTOpenA".as_ptr().cast()).is_some()
                && GetProcAddress(module, c"WTClose".as_ptr().cast()).is_some()
                && GetProcAddress(module, c"WTPacket".as_ptr().cast()).is_some()
        };
        // SAFETY: module was returned by LoadLibraryA above and is no longer used afterward.
        unsafe {
            FreeLibrary(module);
        }

        if has_core_exports {
            NativeTabletBackendHealth::available(
                NativeTabletBackendKind::WindowsWintab,
                "wintab32.dll exports detected; context opening is gated behind explicit backend selection",
            )
        } else {
            NativeTabletBackendHealth::unavailable(
                NativeTabletBackendKind::WindowsWintab,
                "wintab32.dll is missing required Wintab exports",
            )
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;

    pub(super) fn probe_wintab() -> NativeTabletBackendHealth {
        NativeTabletBackendHealth::unavailable(
            NativeTabletBackendKind::WindowsWintab,
            "Wintab is only available on Windows",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wintab_packet_preserves_wacom_specific_axes() {
        let dto = WindowsWintabPacketDto {
            device_id: 55,
            kind: PointerEventKind::Move,
            position: PointerPosition::new(10.0, 20.0),
            timestamp_micros: Some(40),
            normal_pressure: Some(0.6),
            tangential_pressure: Some(0.2),
            tilt: Some(PointerTilt::new(15.0, -5.0)),
            rotation_degrees: Some(270.0),
            barrel_buttons: PointerBarrelButtons {
                primary: true,
                secondary: false,
            },
            eraser: false,
            in_proximity: true,
        };

        let packet = map_windows_wintab_packet(
            dto,
            Some(PointerPosition::new(6.0, 17.0)),
            PointerCalibration::identity(),
        );

        assert_eq!(packet.backend, NativeTabletBackendKind::WindowsWintab);
        assert_eq!(packet.delta, PointerDelta::new(4.0, 3.0));
        assert_eq!(packet.pressure, Some(0.6));
        assert_eq!(packet.tangential_pressure, Some(0.2));
        assert_eq!(packet.tilt, Some(PointerTilt::new(15.0, -5.0)));
        assert_eq!(packet.twist_degrees, Some(270.0));
        assert!(packet.capabilities.tangential_pressure);
        assert!(packet.barrel_buttons.primary);
    }
}
