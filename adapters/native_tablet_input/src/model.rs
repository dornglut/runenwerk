//! Native tablet DTOs, backend status, and translation staging.

use std::collections::VecDeque;

use engine::plugins::InputState;
use runen_input::{
    AnalogMeasurement, CoordinateSpace, InputContext, InputDeviceId, InputSourceId, InputToolKind,
    MeasurementDomain, Point2, SourceTime, SourceTimeUnit, Vector2,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletPosition {
    pub x: f32,
    pub y: f32,
}
impl NativeTabletPosition {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletDelta {
    pub x: f32,
    pub y: f32,
}
impl NativeTabletDelta {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletTilt {
    pub x_degrees: f32,
    pub y_degrees: f32,
}
impl NativeTabletTilt {
    pub const fn new(x_degrees: f32, y_degrees: f32) -> Self {
        Self {
            x_degrees,
            y_degrees,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTabletEventKind {
    Down,
    Move,
    Up,
    Enter,
    Leave,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletSourceKind {
    Mouse,
    Stylus,
    Touch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTabletButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NativeTabletBarrelButtons {
    pub primary: bool,
    pub secondary: bool,
}
impl NativeTabletBarrelButtons {
    pub const fn none() -> Self {
        Self {
            primary: false,
            secondary: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletCalibration {
    pub cursor_offset: NativeTabletDelta,
    pub pressure_scale: f32,
    pub pressure_bias: f32,
}
impl NativeTabletCalibration {
    pub const fn identity() -> Self {
        Self {
            cursor_offset: NativeTabletDelta::ZERO,
            pressure_scale: 1.0,
            pressure_bias: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTabletLatencyClass {
    Normal,
    LowLatencyPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletPlatform {
    Windows,
    Macos,
    Unknown,
}
impl NativeTabletPlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else {
            Self::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletVendor {
    Wacom,
    Generic,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeTabletBackendKind {
    WindowsPointer,
    WindowsWintab,
    MacosNsevent,
    MacosWacomDriver,
    WinitFallback,
}
impl NativeTabletBackendKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::WindowsPointer => "Windows Pointer/Ink",
            Self::WindowsWintab => "Wacom Wintab",
            Self::MacosNsevent => "macOS NSEvent",
            Self::MacosWacomDriver => "Wacom macOS Driver",
            Self::WinitFallback => "winit fallback",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum NativeTabletBackendPreference {
    #[default]
    AutoOsFirst,
    WindowsPointer,
    WindowsWintab,
    MacosNsevent,
    MacosWacomDriver,
    WinitFallback,
}
impl NativeTabletBackendPreference {
    pub fn accepts(self, backend: NativeTabletBackendKind) -> bool {
        match self {
            Self::AutoOsFirst => true,
            Self::WindowsPointer => backend == NativeTabletBackendKind::WindowsPointer,
            Self::WindowsWintab => backend == NativeTabletBackendKind::WindowsWintab,
            Self::MacosNsevent => backend == NativeTabletBackendKind::MacosNsevent,
            Self::MacosWacomDriver => backend == NativeTabletBackendKind::MacosWacomDriver,
            Self::WinitFallback => backend == NativeTabletBackendKind::WinitFallback,
        }
    }
}

pub type NativeTabletToolKind = InputToolKind;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeTabletContactState {
    Hover,
    Contact,
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeTabletDiagnostic {
    MissingCapability(NativeTabletCapabilityKind),
    InvalidObservation(String),
    ConflictingDeviceIdentity,
    BackendUnavailable {
        backend: NativeTabletBackendKind,
        reason: String,
    },
    BackendWarning {
        backend: NativeTabletBackendKind,
        message: String,
    },
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
    pub const fn windows_pointer() -> Self {
        Self {
            pressure: true,
            tilt: false,
            twist: false,
            tangential_pressure: false,
            hover: true,
            eraser: false,
            barrel_buttons: false,
            coalesced_samples: true,
            predicted_samples: false,
            calibration: true,
        }
    }
    pub const fn windows_pointer_mouse() -> Self {
        Self {
            pressure: false,
            tilt: false,
            twist: false,
            tangential_pressure: false,
            hover: true,
            eraser: false,
            barrel_buttons: false,
            coalesced_samples: false,
            predicted_samples: false,
            calibration: true,
        }
    }
    pub const fn macos_nsevent() -> Self {
        Self {
            pressure: true,
            tilt: true,
            twist: true,
            tangential_pressure: true,
            hover: true,
            eraser: true,
            barrel_buttons: true,
            coalesced_samples: false,
            predicted_samples: false,
            calibration: true,
        }
    }
    pub const fn macos_wacom() -> Self {
        Self::full_stylus()
    }
    pub const fn windows_wintab() -> Self {
        Self::full_stylus()
    }
    const fn full_stylus() -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletSample {
    pub position: NativeTabletPosition,
    pub delta: NativeTabletDelta,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<NativeTabletTilt>,
    pub twist_degrees: Option<f32>,
    pub tangential_pressure: Option<f32>,
    pub contact: NativeTabletContactState,
}
impl NativeTabletSample {
    pub const fn new(position: NativeTabletPosition, delta: NativeTabletDelta) -> Self {
        Self {
            position,
            delta,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            tangential_pressure: None,
            contact: NativeTabletContactState::Contact,
        }
    }
    pub fn with_timestamp_micros(mut self, value: u64) -> Self {
        self.timestamp_micros = Some(value);
        self
    }
    pub fn with_pressure(mut self, value: f32) -> Self {
        self.pressure = Some(value);
        self
    }
    pub fn with_tilt(mut self, value: NativeTabletTilt) -> Self {
        self.tilt = Some(value);
        self
    }
    pub fn with_twist_degrees(mut self, value: f32) -> Self {
        self.twist_degrees = Some(value);
        self
    }
    pub fn with_tangential_pressure(mut self, value: f32) -> Self {
        self.tangential_pressure = Some(value);
        self
    }
    pub fn with_contact(mut self, value: NativeTabletContactState) -> Self {
        self.contact = value;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeTabletPacket {
    pub platform: NativeTabletPlatform,
    pub vendor: NativeTabletVendor,
    pub backend: NativeTabletBackendKind,
    pub source_kind: NativeTabletSourceKind,
    pub device_id: Option<u64>,
    pub contact_id: u64,
    pub tool_id: Option<u64>,
    pub kind: NativeTabletEventKind,
    pub position: NativeTabletPosition,
    pub delta: NativeTabletDelta,
    pub event_button: Option<NativeTabletButton>,
    pub tool_kind: NativeTabletToolKind,
    pub timestamp_micros: Option<u64>,
    pub contact: NativeTabletContactState,
    pub pressure: Option<f32>,
    pub tilt: Option<NativeTabletTilt>,
    pub twist_degrees: Option<f32>,
    pub tangential_pressure: Option<f32>,
    pub eraser: bool,
    pub barrel_buttons: NativeTabletBarrelButtons,
    pub capabilities: NativeTabletCapabilities,
    pub calibration: Option<NativeTabletCalibration>,
    pub latency_class: NativeTabletLatencyClass,
    pub coalesced_samples: Vec<NativeTabletSample>,
    pub predicted_samples: Vec<NativeTabletSample>,
}
impl NativeTabletPacket {
    #[allow(clippy::too_many_arguments)]
    fn new(
        platform: NativeTabletPlatform,
        vendor: NativeTabletVendor,
        backend: NativeTabletBackendKind,
        source_kind: NativeTabletSourceKind,
        capabilities: NativeTabletCapabilities,
        contact_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
        delta: NativeTabletDelta,
    ) -> Self {
        Self {
            platform,
            vendor,
            backend,
            source_kind,
            device_id: Some(contact_id),
            contact_id,
            tool_id: None,
            kind,
            position,
            delta,
            event_button: None,
            tool_kind: InputToolKind::Pen,
            timestamp_micros: None,
            contact: NativeTabletContactState::Contact,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            tangential_pressure: None,
            eraser: false,
            barrel_buttons: NativeTabletBarrelButtons::none(),
            capabilities,
            calibration: None,
            latency_class: NativeTabletLatencyClass::Normal,
            coalesced_samples: Vec::new(),
            predicted_samples: Vec::new(),
        }
    }
    pub fn macos_wacom(
        device_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
        delta: NativeTabletDelta,
    ) -> Self {
        Self::new(
            NativeTabletPlatform::Macos,
            NativeTabletVendor::Wacom,
            NativeTabletBackendKind::MacosWacomDriver,
            NativeTabletSourceKind::Stylus,
            NativeTabletCapabilities::macos_wacom(),
            device_id,
            kind,
            position,
            delta,
        )
    }
    pub fn windows_pointer(
        device_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
        delta: NativeTabletDelta,
    ) -> Self {
        Self::new(
            NativeTabletPlatform::Windows,
            NativeTabletVendor::Generic,
            NativeTabletBackendKind::WindowsPointer,
            NativeTabletSourceKind::Stylus,
            NativeTabletCapabilities::windows_pointer(),
            device_id,
            kind,
            position,
            delta,
        )
    }
    pub fn windows_pointer_mouse(
        device_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
        delta: NativeTabletDelta,
    ) -> Self {
        let mut packet = Self::new(
            NativeTabletPlatform::Windows,
            NativeTabletVendor::Generic,
            NativeTabletBackendKind::WindowsPointer,
            NativeTabletSourceKind::Mouse,
            NativeTabletCapabilities::windows_pointer_mouse(),
            device_id,
            kind,
            position,
            delta,
        );
        packet.tool_kind = InputToolKind::Mouse;
        packet
    }
    pub fn windows_wintab(
        device_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
        delta: NativeTabletDelta,
    ) -> Self {
        Self::new(
            NativeTabletPlatform::Windows,
            NativeTabletVendor::Wacom,
            NativeTabletBackendKind::WindowsWintab,
            NativeTabletSourceKind::Stylus,
            NativeTabletCapabilities::windows_wintab(),
            device_id,
            kind,
            position,
            delta,
        )
    }
    pub fn macos_nsevent(
        device_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
        delta: NativeTabletDelta,
    ) -> Self {
        Self::new(
            NativeTabletPlatform::Macos,
            NativeTabletVendor::Generic,
            NativeTabletBackendKind::MacosNsevent,
            NativeTabletSourceKind::Stylus,
            NativeTabletCapabilities::macos_nsevent(),
            device_id,
            kind,
            position,
            delta,
        )
    }
    pub fn with_event_button(mut self, value: Option<NativeTabletButton>) -> Self {
        self.event_button = value;
        self
    }
    pub fn with_timestamp_micros(mut self, value: u64) -> Self {
        self.timestamp_micros = Some(value);
        self
    }
    pub fn with_tool_kind(mut self, value: NativeTabletToolKind) -> Self {
        self.tool_kind = value;
        self
    }
    pub fn with_source_kind(mut self, value: NativeTabletSourceKind) -> Self {
        self.source_kind = value;
        self
    }
    pub fn with_pressure(mut self, value: f32) -> Self {
        self.pressure = Some(value);
        self
    }
    pub fn with_tilt(mut self, value: NativeTabletTilt) -> Self {
        self.tilt = Some(value);
        self
    }
    pub fn with_twist_degrees(mut self, value: f32) -> Self {
        self.twist_degrees = Some(value);
        self
    }
    pub fn with_tangential_pressure(mut self, value: f32) -> Self {
        self.tangential_pressure = Some(value);
        self
    }
    pub fn with_contact(mut self, value: NativeTabletContactState) -> Self {
        self.contact = value;
        self
    }
    pub fn with_eraser(mut self, value: bool) -> Self {
        self.eraser = value;
        if value {
            self.tool_kind = InputToolKind::Eraser;
        }
        self
    }
    pub fn with_barrel_buttons(mut self, value: NativeTabletBarrelButtons) -> Self {
        self.barrel_buttons = value;
        self
    }
    pub fn with_capabilities(mut self, value: NativeTabletCapabilities) -> Self {
        self.capabilities = value;
        self
    }
    pub fn with_calibration(mut self, value: NativeTabletCalibration) -> Self {
        self.calibration = Some(value);
        self
    }
    pub fn with_latency_class(mut self, value: NativeTabletLatencyClass) -> Self {
        self.latency_class = value;
        self
    }
    pub fn with_coalesced_samples(
        mut self,
        values: impl IntoIterator<Item = NativeTabletSample>,
    ) -> Self {
        self.coalesced_samples = values.into_iter().collect();
        self
    }
    pub fn with_predicted_samples(
        mut self,
        values: impl IntoIterator<Item = NativeTabletSample>,
    ) -> Self {
        self.predicted_samples = values.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeTabletBackendStatus {
    Active,
    Available,
    Unavailable,
    Error,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeTabletBackendHealth {
    pub backend: NativeTabletBackendKind,
    pub status: NativeTabletBackendStatus,
    pub message: String,
}
impl NativeTabletBackendHealth {
    fn new(
        backend: NativeTabletBackendKind,
        status: NativeTabletBackendStatus,
        message: impl Into<String>,
    ) -> Self {
        Self {
            backend,
            status,
            message: message.into(),
        }
    }
    pub fn active(backend: NativeTabletBackendKind, message: impl Into<String>) -> Self {
        Self::new(backend, NativeTabletBackendStatus::Active, message)
    }
    pub fn available(backend: NativeTabletBackendKind, message: impl Into<String>) -> Self {
        Self::new(backend, NativeTabletBackendStatus::Available, message)
    }
    pub fn unavailable(backend: NativeTabletBackendKind, message: impl Into<String>) -> Self {
        Self::new(backend, NativeTabletBackendStatus::Unavailable, message)
    }
    pub fn error(backend: NativeTabletBackendKind, message: impl Into<String>) -> Self {
        Self::new(backend, NativeTabletBackendStatus::Error, message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeTabletDeviceDescriptor {
    pub device_id: Option<InputDeviceId>,
    pub platform: NativeTabletPlatform,
    pub vendor: NativeTabletVendor,
    pub backend: NativeTabletBackendKind,
    pub name: String,
    pub capabilities: NativeTabletCapabilities,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeTabletSampleTelemetry {
    pub packets_this_frame: u32,
    pub samples_this_frame: u32,
    pub confirmed_samples_this_frame: u32,
    pub coalesced_samples_this_frame: u32,
    pub predicted_samples_this_frame: u32,
    pub dropped_samples_this_frame: u32,
    pub duplicate_samples_this_frame: u32,
    pub max_segment_gap_px: f32,
    pub sample_rate_hz: f32,
    pub pressure_available: bool,
    pub tilt_available: bool,
}
impl Default for NativeTabletSampleTelemetry {
    fn default() -> Self {
        Self {
            packets_this_frame: 0,
            samples_this_frame: 0,
            confirmed_samples_this_frame: 0,
            coalesced_samples_this_frame: 0,
            predicted_samples_this_frame: 0,
            dropped_samples_this_frame: 0,
            duplicate_samples_this_frame: 0,
            max_segment_gap_px: 0.0,
            sample_rate_hz: 0.0,
            pressure_available: false,
            tilt_available: false,
        }
    }
}
impl NativeTabletSampleTelemetry {
    pub fn observe_packet(
        &mut self,
        packet: &NativeTabletPacket,
        previous: Option<NativeTabletPosition>,
    ) {
        self.packets_this_frame = self.packets_this_frame.saturating_add(1);
        let confirmed_count = 1usize + packet.coalesced_samples.len();
        let delivered_count = confirmed_count + packet.predicted_samples.len();
        self.samples_this_frame = self
            .samples_this_frame
            .saturating_add(delivered_count as u32);
        self.confirmed_samples_this_frame = self
            .confirmed_samples_this_frame
            .saturating_add(confirmed_count as u32);
        self.coalesced_samples_this_frame = self
            .coalesced_samples_this_frame
            .saturating_add(packet.coalesced_samples.len() as u32);
        self.predicted_samples_this_frame = self
            .predicted_samples_this_frame
            .saturating_add(packet.predicted_samples.len() as u32);
        self.pressure_available |= packet.capabilities.pressure;
        self.tilt_available |= packet.capabilities.tilt;
        let mut last = previous;
        for position in packet
            .coalesced_samples
            .iter()
            .map(|s| s.position)
            .chain(std::iter::once(packet.position))
        {
            if let Some(previous) = last {
                let dx = position.x - previous.x;
                let dy = position.y - previous.y;
                let gap = (dx * dx + dy * dy).sqrt();
                if gap <= f32::EPSILON {
                    self.duplicate_samples_this_frame =
                        self.duplicate_samples_this_frame.saturating_add(1);
                }
                self.max_segment_gap_px = self.max_segment_gap_px.max(gap);
            }
            last = Some(position);
        }
        if let (Some(first), Some(last)) = (
            packet
                .coalesced_samples
                .first()
                .and_then(|s| s.timestamp_micros)
                .or(packet.timestamp_micros),
            packet.timestamp_micros,
        ) {
            let elapsed = last.saturating_sub(first);
            if elapsed > 0 {
                self.sample_rate_hz =
                    (confirmed_count.saturating_sub(1) as f32) * 1_000_000.0 / elapsed as f32;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, runen_ecs::Component, runen_ecs::Resource)]
pub struct NativeTabletDeviceControlResource {
    pub backend_preference: NativeTabletBackendPreference,
    pub calibration: NativeTabletCalibration,
    pub reset_calibration_requested: bool,
}
impl Default for NativeTabletDeviceControlResource {
    fn default() -> Self {
        Self {
            backend_preference: NativeTabletBackendPreference::AutoOsFirst,
            calibration: NativeTabletCalibration::identity(),
            reset_calibration_requested: false,
        }
    }
}
impl NativeTabletDeviceControlResource {
    pub fn request_reset_calibration(&mut self) {
        self.reset_calibration_requested = true;
    }
    pub fn apply_pending_reset(&mut self) {
        if self.reset_calibration_requested {
            self.calibration = NativeTabletCalibration::identity();
            self.reset_calibration_requested = false;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, runen_ecs::Component, runen_ecs::Resource)]
pub struct NativeTabletFrameResource {
    pub packets: Vec<NativeTabletPacket>,
    pub devices: Vec<NativeTabletDeviceDescriptor>,
    pub backend_health: Vec<NativeTabletBackendHealth>,
    pub telemetry: NativeTabletSampleTelemetry,
    pub diagnostics: Vec<NativeTabletDiagnostic>,
    stream_positions: std::collections::HashMap<NativeTabletStreamKey, NativeTabletPosition>,
}
impl NativeTabletFrameResource {
    pub fn push_packet(&mut self, packet: NativeTabletPacket) {
        self.packets.push(packet);
    }
    pub fn publish_to_neutral(&mut self, input: &mut InputState) {
        let packets = std::mem::take(&mut self.packets);
        self.telemetry = NativeTabletSampleTelemetry::default();
        for packet in packets {
            let stream_key = NativeTabletStreamKey::from_packet(&packet);
            let previous = self.stream_positions.get(&stream_key).copied();
            match crate::mapping::map_native_tablet_packet(&packet) {
                Ok(mapping) => {
                    for diagnostic in mapping.diagnostics {
                        if !self.diagnostics.contains(&diagnostic) {
                            self.diagnostics.push(diagnostic);
                        }
                    }
                    match input.admit_device_observation_group(mapping.group) {
                        Ok(()) => {
                            self.telemetry.observe_packet(&packet, previous);
                            if packet.is_terminal() {
                                self.stream_positions.remove(&stream_key);
                            } else {
                                self.stream_positions.insert(stream_key, packet.position);
                            }
                        }
                        Err(error) => {
                            self.diagnostics
                                .push(NativeTabletDiagnostic::InvalidObservation(format!(
                                    "neutral admission rejected tablet packet: {error:?}"
                                )))
                        }
                    }
                }
                Err(diagnostic) => self.diagnostics.push(diagnostic),
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct NativeTabletStreamKey {
    backend: NativeTabletBackendKind,
    source_kind: NativeTabletSourceKind,
    device_id: Option<u64>,
    tool_id: Option<u64>,
    tool_kind: NativeTabletToolKind,
    contact_id: u64,
}

impl NativeTabletStreamKey {
    fn from_packet(packet: &NativeTabletPacket) -> Self {
        Self {
            backend: packet.backend,
            source_kind: packet.source_kind,
            device_id: packet.device_id,
            tool_id: packet.tool_id,
            tool_kind: packet.tool_kind,
            contact_id: packet.contact_id,
        }
    }
}

impl NativeTabletPacket {
    pub(crate) fn is_terminal(&self) -> bool {
        matches!(
            self.kind,
            NativeTabletEventKind::Up | NativeTabletEventKind::Leave
        ) || self.contact == NativeTabletContactState::OutOfRange
    }
}

#[derive(Debug, Clone, Default, PartialEq, runen_ecs::Component, runen_ecs::Resource)]
pub struct NativeTabletRuntimeResource {
    pending_packets: VecDeque<NativeTabletPacket>,
    pub devices: Vec<NativeTabletDeviceDescriptor>,
    pub backend_health: Vec<NativeTabletBackendHealth>,
    pub diagnostics: Vec<NativeTabletDiagnostic>,
}
impl NativeTabletRuntimeResource {
    pub fn push_packet(&mut self, packet: NativeTabletPacket) {
        self.upsert_device(&packet, true);
        self.pending_packets.push_back(packet);
    }
    pub fn set_backend_health(&mut self, health: NativeTabletBackendHealth) {
        if let Some(existing) = self
            .backend_health
            .iter_mut()
            .find(|entry| entry.backend == health.backend)
        {
            *existing = health;
        } else {
            self.backend_health.push(health);
        }
    }
    pub fn push_diagnostic(&mut self, diagnostic: NativeTabletDiagnostic) {
        if !self.diagnostics.contains(&diagnostic) {
            self.diagnostics.push(diagnostic);
        }
    }
    pub fn publish_frame(
        &mut self,
        frame: &mut NativeTabletFrameResource,
        control: &mut NativeTabletDeviceControlResource,
    ) {
        control.apply_pending_reset();
        frame.packets.extend(self.pending_packets.drain(..));
        frame.devices = self.devices.clone();
        frame.backend_health = self.backend_health.clone();
        frame.diagnostics = self.diagnostics.clone();
    }
    fn upsert_device(&mut self, packet: &NativeTabletPacket, active: bool) {
        let descriptor = NativeTabletDeviceDescriptor {
            device_id: packet.device_id.map(InputDeviceId::new),
            platform: packet.platform,
            vendor: packet.vendor,
            backend: packet.backend,
            name: format!(
                "{} device {}",
                packet.backend.label(),
                packet
                    .device_id
                    .map_or_else(|| "unknown".to_string(), |id| id.to_string())
            ),
            capabilities: packet.capabilities,
            active,
        };
        if let Some(existing) = self
            .devices
            .iter_mut()
            .find(|device| device.device_id == descriptor.device_id)
        {
            *existing = descriptor;
        } else {
            self.devices.push(descriptor);
        }
    }
}

pub(crate) fn calibrated_position(
    position: NativeTabletPosition,
    calibration: Option<NativeTabletCalibration>,
) -> NativeTabletPosition {
    let Some(calibration) = calibration else {
        return position;
    };
    NativeTabletPosition::new(
        position.x + calibration.cursor_offset.x,
        position.y + calibration.cursor_offset.y,
    )
}
pub(crate) fn calibrated_pressure(
    pressure: Option<f32>,
    capabilities: NativeTabletCapabilities,
    calibration: Option<NativeTabletCalibration>,
) -> Option<f32> {
    let pressure = pressure.filter(|_| capabilities.pressure)?;
    calibration.map_or(Some(pressure), |calibration| {
        Some(pressure * calibration.pressure_scale + calibration.pressure_bias)
    })
}
pub(crate) fn input_context(
    backend: NativeTabletBackendKind,
    device_id: Option<u64>,
) -> InputContext {
    let source = match backend {
        NativeTabletBackendKind::WindowsPointer => InputSourceId::new(3),
        NativeTabletBackendKind::WindowsWintab => InputSourceId::new(4),
        NativeTabletBackendKind::MacosNsevent => InputSourceId::new(5),
        NativeTabletBackendKind::MacosWacomDriver => InputSourceId::new(6),
        NativeTabletBackendKind::WinitFallback => InputSourceId::new(7),
    };
    InputContext::new(source, device_id.map(InputDeviceId::new))
}
pub(crate) fn point(position: NativeTabletPosition) -> Point2 {
    Point2::new(
        position.x,
        position.y,
        CoordinateSpace::WindowPhysicalPixels,
    )
}
pub(crate) fn vector(delta: NativeTabletDelta) -> Vector2 {
    Vector2::new(delta.x, delta.y)
}
pub(crate) fn source_time(
    context: InputContext,
    timestamp_micros: Option<u64>,
) -> Option<SourceTime> {
    timestamp_micros.map(|value| SourceTime::new(context, value, SourceTimeUnit::Microseconds))
}
pub(crate) fn measurement(
    value: Option<f32>,
    capabilities: bool,
    domain: MeasurementDomain,
) -> Option<AnalogMeasurement> {
    value
        .filter(|_| capabilities)
        .map(|value| AnalogMeasurement::new(value, domain))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(
        device_id: u64,
        contact_id: u64,
        kind: NativeTabletEventKind,
        position: NativeTabletPosition,
    ) -> NativeTabletPacket {
        let mut packet =
            NativeTabletPacket::windows_pointer(device_id, kind, position, NativeTabletDelta::ZERO);
        packet.contact_id = contact_id;
        packet
    }

    #[test]
    fn telemetry_previous_positions_are_scoped_to_complete_stream_identity() {
        let mut frame = NativeTabletFrameResource::default();
        let mut input = InputState::new();
        frame.push_packet(packet(
            1,
            1,
            NativeTabletEventKind::Down,
            NativeTabletPosition::new(0.0, 0.0),
        ));
        frame.push_packet(packet(
            2,
            2,
            NativeTabletEventKind::Down,
            NativeTabletPosition::new(1_000.0, 0.0),
        ));
        frame.publish_to_neutral(&mut input);
        assert_eq!(frame.telemetry.max_segment_gap_px, 0.0);

        frame.push_packet(packet(
            1,
            1,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(10.0, 0.0),
        ));
        frame.push_packet(packet(
            2,
            2,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(1_010.0, 0.0),
        ));
        frame.publish_to_neutral(&mut input);

        assert_eq!(frame.telemetry.packets_this_frame, 2);
        assert_eq!(frame.telemetry.max_segment_gap_px, 10.0);
    }

    #[test]
    fn terminal_and_rejected_packets_do_not_seed_telemetry_state() {
        let mut frame = NativeTabletFrameResource::default();
        let mut input = InputState::new();
        frame.push_packet(packet(
            3,
            3,
            NativeTabletEventKind::Down,
            NativeTabletPosition::new(0.0, 0.0),
        ));
        frame.publish_to_neutral(&mut input);

        frame.push_packet(packet(
            3,
            3,
            NativeTabletEventKind::Up,
            NativeTabletPosition::new(100.0, 0.0),
        ));
        frame.publish_to_neutral(&mut input);

        frame.push_packet(packet(
            3,
            3,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(105.0, 0.0),
        ));
        frame.publish_to_neutral(&mut input);
        assert_eq!(frame.telemetry.max_segment_gap_px, 0.0);

        let rejected = packet(
            4,
            4,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(f32::NAN, 0.0),
        );
        frame.push_packet(rejected);
        frame.publish_to_neutral(&mut input);
        assert_eq!(frame.telemetry.packets_this_frame, 0);
        assert_eq!(frame.telemetry.max_segment_gap_px, 0.0);

        frame.push_packet(packet(
            4,
            4,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(20.0, 0.0),
        ));
        frame.publish_to_neutral(&mut input);
        assert_eq!(frame.telemetry.max_segment_gap_px, 0.0);
    }

    #[test]
    fn predicted_samples_are_delivered_without_inflating_confirmed_sample_rate() {
        let packet = packet(
            5,
            5,
            NativeTabletEventKind::Move,
            NativeTabletPosition::new(30.0, 0.0),
        )
        .with_timestamp_micros(300)
        .with_coalesced_samples([
            NativeTabletSample::new(
                NativeTabletPosition::new(10.0, 0.0),
                NativeTabletDelta::ZERO,
            )
            .with_timestamp_micros(100),
            NativeTabletSample::new(
                NativeTabletPosition::new(20.0, 0.0),
                NativeTabletDelta::ZERO,
            )
            .with_timestamp_micros(200),
        ])
        .with_predicted_samples([NativeTabletSample::new(
            NativeTabletPosition::new(40.0, 0.0),
            NativeTabletDelta::ZERO,
        )]);
        let mut telemetry = NativeTabletSampleTelemetry::default();

        telemetry.observe_packet(&packet, None);

        assert_eq!(telemetry.confirmed_samples_this_frame, 3);
        assert_eq!(telemetry.samples_this_frame, 4);
        assert_eq!(telemetry.predicted_samples_this_frame, 1);
        assert_eq!(telemetry.sample_rate_hz, 10_000.0);
    }
}
