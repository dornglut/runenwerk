//! Windows Pointer/Ink backend.

use std::collections::{HashMap, VecDeque};

use winit::event_loop::EventLoopBuilder;

use crate::backend::NativeTabletBackendAdapter;
use crate::model::{
    NativeTabletBackendHealth, NativeTabletBackendKind, NativeTabletBarrelButtons,
    NativeTabletButton, NativeTabletCalibration, NativeTabletCapabilities,
    NativeTabletContactState, NativeTabletDelta, NativeTabletDeviceControlResource,
    NativeTabletEventKind, NativeTabletLatencyClass, NativeTabletPacket, NativeTabletPosition,
    NativeTabletRuntimeResource, NativeTabletSample, NativeTabletSourceKind, NativeTabletTilt,
    NativeTabletToolKind,
};

type PointerPosition = NativeTabletPosition;
type PointerDelta = NativeTabletDelta;
type PointerTilt = NativeTabletTilt;
type PointerBarrelButtons = NativeTabletBarrelButtons;
type PointerButton = NativeTabletButton;
type PointerCalibration = NativeTabletCalibration;
type PointerContactState = NativeTabletContactState;
type PointerEventKind = NativeTabletEventKind;
type PointerLatencyClass = NativeTabletLatencyClass;
type PointerSourceKind = NativeTabletSourceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsPointerInputKind {
    Mouse,
    Pen,
    Touch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowsPointerHistorySample {
    pub pointer_id: u32,
    pub source_device: Option<u64>,
    pub input_kind: WindowsPointerInputKind,
    pub position: PointerPosition,
    pub predicted_position: Option<PointerPosition>,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<PointerTilt>,
    pub twist_degrees: Option<f32>,
    pub eraser: bool,
    pub barrel_buttons: PointerBarrelButtons,
    pub event_button: Option<PointerButton>,
    pub primary_button_down: bool,
    pub in_contact: bool,
    pub in_range: bool,
    pub cancelled: bool,
}

impl WindowsPointerHistorySample {
    pub fn mouse(pointer_id: u32, position: PointerPosition) -> Self {
        Self {
            pointer_id,
            source_device: None,
            input_kind: WindowsPointerInputKind::Mouse,
            position,
            predicted_position: None,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            eraser: false,
            barrel_buttons: PointerBarrelButtons::none(),
            event_button: Some(PointerButton::Left),
            primary_button_down: true,
            in_contact: true,
            in_range: true,
            cancelled: false,
        }
    }

    pub fn pen(pointer_id: u32, position: PointerPosition) -> Self {
        Self {
            pointer_id,
            source_device: None,
            input_kind: WindowsPointerInputKind::Pen,
            position,
            predicted_position: None,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            eraser: false,
            barrel_buttons: PointerBarrelButtons::none(),
            event_button: Some(PointerButton::Left),
            primary_button_down: true,
            in_contact: true,
            in_range: true,
            cancelled: false,
        }
    }

    pub fn touch(pointer_id: u32, position: PointerPosition) -> Self {
        Self {
            pointer_id,
            source_device: None,
            input_kind: WindowsPointerInputKind::Touch,
            position,
            predicted_position: None,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            eraser: false,
            barrel_buttons: PointerBarrelButtons::none(),
            event_button: Some(PointerButton::Left),
            primary_button_down: true,
            in_contact: true,
            in_range: true,
            cancelled: false,
        }
    }

    pub fn with_timestamp_micros(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = Some(timestamp_micros);
        self
    }

    pub fn with_source_device(mut self, source_device: u64) -> Self {
        self.source_device = Some(source_device);
        self
    }

    pub fn with_predicted_position(mut self, position: PointerPosition) -> Self {
        self.predicted_position = Some(position);
        self
    }

    pub fn with_cancelled(mut self, cancelled: bool) -> Self {
        self.cancelled = cancelled;
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct WindowsPointerHistoryPacket {
    pub kind: PointerEventKind,
    pub pointer_id: u32,
    /// Win32 returns pointer history newest-first. This vector preserves that raw order.
    pub samples_newest_first: Vec<WindowsPointerHistorySample>,
}

impl WindowsPointerHistoryPacket {
    pub fn new(
        kind: PointerEventKind,
        pointer_id: u32,
        samples_newest_first: impl IntoIterator<Item = WindowsPointerHistorySample>,
    ) -> Self {
        Self {
            kind,
            pointer_id,
            samples_newest_first: samples_newest_first.into_iter().collect(),
        }
    }
}

#[derive(Debug, Default)]
pub struct WindowsPointerBackend {
    queue: SharedWindowsPointerQueue,
    last_positions: HashMap<u32, PointerPosition>,
    messages_seen: u64,
    packets_queued: u64,
}

impl WindowsPointerBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_history_for_test(&mut self, history: WindowsPointerHistoryPacket) {
        self.queue.push(history);
    }

    fn drain_queue(
        &mut self,
        runtime: &mut NativeTabletRuntimeResource,
        control: &NativeTabletDeviceControlResource,
    ) {
        if !control
            .backend_preference
            .accepts(NativeTabletBackendKind::WindowsPointer)
        {
            self.queue.clear();
            runtime.set_backend_health(NativeTabletBackendHealth::available(
                NativeTabletBackendKind::WindowsPointer,
                "Windows Pointer/Ink observed but disabled by backend preference",
            ));
            return;
        }

        let mut drained = 0_u64;
        while let Some(history) = self.queue.pop_front() {
            drained = drained.saturating_add(1);
            let pointer_id = history.pointer_id;
            let previous = self.last_positions.get(&pointer_id).copied();
            match map_windows_pointer_history(history, previous, control.calibration) {
                Ok(packet) => {
                    self.last_positions
                        .insert(packet.contact_id as u32, packet.position);
                    let terminal = packet.is_terminal();
                    runtime.push_packet(packet);
                    self.packets_queued = self.packets_queued.saturating_add(1);
                    if terminal {
                        self.last_positions.remove(&pointer_id);
                    }
                }
                Err(diagnostic) => runtime.diagnostics.push(diagnostic),
            }
        }
        if drained > 0 {
            self.messages_seen = self.messages_seen.saturating_add(drained);
            runtime.set_backend_health(NativeTabletBackendHealth::active(
                NativeTabletBackendKind::WindowsPointer,
                format!(
                    "active; observed {messages} WM_POINTER messages, queued {packets} packets for neutral admission",
                    messages = self.messages_seen,
                    packets = self.packets_queued
                ),
            ));
        }
    }
}

impl NativeTabletBackendAdapter for WindowsPointerBackend {
    fn kind(&self) -> NativeTabletBackendKind {
        NativeTabletBackendKind::WindowsPointer
    }

    fn configure_event_loop(&mut self, builder: &mut EventLoopBuilder<()>) {
        platform::install_windows_pointer_message_hook(builder, self.queue.clone());
    }

    fn attach(
        &mut self,
        window: &winit::window::Window,
        runtime: &mut NativeTabletRuntimeResource,
        control: &NativeTabletDeviceControlResource,
    ) {
        if !control
            .backend_preference
            .accepts(NativeTabletBackendKind::WindowsPointer)
        {
            runtime.set_backend_health(NativeTabletBackendHealth::available(
                NativeTabletBackendKind::WindowsPointer,
                "Windows Pointer/Ink disabled by backend preference",
            ));
            return;
        }
        let health = platform::attach_windows_pointer_target(window);
        runtime.set_backend_health(health);
    }

    fn frame(
        &mut self,
        runtime: &mut NativeTabletRuntimeResource,
        control: &NativeTabletDeviceControlResource,
    ) {
        self.drain_queue(runtime, control);
    }
}

pub fn map_windows_pointer_history(
    history: WindowsPointerHistoryPacket,
    previous_position: Option<PointerPosition>,
    calibration: PointerCalibration,
) -> Result<NativeTabletPacket, crate::model::NativeTabletDiagnostic> {
    let mut chronological = history
        .samples_newest_first
        .into_iter()
        .filter(|sample| sample.pointer_id == history.pointer_id)
        .collect::<Vec<_>>();
    chronological.reverse();

    let (current, coalesced) = chronological.split_last().ok_or(
        crate::model::NativeTabletDiagnostic::InvalidObservation("empty pointer history".into()),
    )?;
    let established_devices = chronological
        .iter()
        .chain(std::iter::once(current))
        .filter_map(|sample| sample.source_device)
        .collect::<std::collections::BTreeSet<_>>();
    if established_devices.len() > 1 {
        return Err(crate::model::NativeTabletDiagnostic::ConflictingDeviceIdentity);
    }
    let mut last_position = previous_position;
    let coalesced_samples = coalesced
        .iter()
        .map(|sample| {
            let delta = delta_from(last_position, sample.position);
            last_position = Some(sample.position);
            native_sample_from_windows(*sample, delta)
        })
        .collect::<Vec<_>>();

    let current_delta = delta_from(last_position, current.position);
    let capabilities =
        capabilities_from_windows_history(&chronological, !coalesced_samples.is_empty());
    if current.input_kind == WindowsPointerInputKind::Mouse
        && matches!(history.kind, PointerEventKind::Down | PointerEventKind::Up)
        && current.event_button != Some(PointerButton::Left)
    {
        return Err(crate::model::NativeTabletDiagnostic::InvalidObservation(
            "non-primary mouse transition".into(),
        ));
    }
    let packet_kind = if current.cancelled {
        PointerEventKind::Leave
    } else {
        history.kind
    };
    let (mut packet, tool_kind) = match current.input_kind {
        WindowsPointerInputKind::Mouse => (
            NativeTabletPacket::windows_pointer_mouse(
                u64::from(history.pointer_id),
                packet_kind,
                current.position,
                current_delta,
            ),
            NativeTabletToolKind::Mouse,
        ),
        WindowsPointerInputKind::Pen => (
            NativeTabletPacket::windows_pointer(
                u64::from(history.pointer_id),
                packet_kind,
                current.position,
                current_delta,
            ),
            if current.eraser {
                NativeTabletToolKind::Eraser
            } else {
                NativeTabletToolKind::Pen
            },
        ),
        WindowsPointerInputKind::Touch => (
            NativeTabletPacket::windows_pointer(
                u64::from(history.pointer_id),
                packet_kind,
                current.position,
                current_delta,
            )
            .with_source_kind(PointerSourceKind::Touch),
            NativeTabletToolKind::Finger,
        ),
    };

    packet = packet
        .with_tool_kind(tool_kind)
        .with_capabilities(capabilities)
        .with_calibration(calibration)
        .with_latency_class(PointerLatencyClass::LowLatencyPreview)
        .with_coalesced_samples(coalesced_samples)
        .with_contact(contact_from_windows_sample(*current))
        .with_event_button(event_button_for_sample(packet_kind, *current));

    if let Some(pressure) = current.pressure {
        packet = packet.with_pressure(pressure);
    }
    if let Some(tilt) = current.tilt {
        packet = packet.with_tilt(tilt);
    }
    if let Some(twist) = current.twist_degrees {
        packet = packet.with_twist_degrees(twist);
    }
    if current.eraser {
        packet = packet.with_eraser(true);
    }
    if current.barrel_buttons.primary || current.barrel_buttons.secondary {
        packet = packet.with_barrel_buttons(current.barrel_buttons);
    }
    if let Some(predicted_position) = current.predicted_position {
        let mut predicted = NativeTabletSample::new(predicted_position, PointerDelta::ZERO)
            .with_contact(contact_from_windows_sample(*current));
        if let Some(timestamp) = current.timestamp_micros {
            predicted = predicted.with_timestamp_micros(timestamp);
        }
        packet = packet.with_predicted_samples([predicted]);
    }
    if let Some(timestamp) = current.timestamp_micros {
        packet = packet.with_timestamp_micros(timestamp);
    }

    packet.device_id = established_devices.into_iter().next();
    packet.contact_id = u64::from(history.pointer_id);
    Ok(packet)
}

fn native_sample_from_windows(
    sample: WindowsPointerHistorySample,
    delta: PointerDelta,
) -> NativeTabletSample {
    let mut native = NativeTabletSample::new(sample.position, delta)
        .with_contact(contact_from_windows_sample(sample));
    if let Some(timestamp) = sample.timestamp_micros {
        native = native.with_timestamp_micros(timestamp);
    }
    if let Some(pressure) = sample.pressure {
        native = native.with_pressure(pressure);
    }
    if let Some(tilt) = sample.tilt {
        native = native.with_tilt(tilt);
    }
    if let Some(twist) = sample.twist_degrees {
        native = native.with_twist_degrees(twist);
    }
    native
}

fn capabilities_from_windows_history(
    samples: &[WindowsPointerHistorySample],
    has_coalesced_samples: bool,
) -> NativeTabletCapabilities {
    let input_kind = samples
        .last()
        .map(|sample| sample.input_kind)
        .unwrap_or(WindowsPointerInputKind::Pen);
    let mut capabilities = match input_kind {
        WindowsPointerInputKind::Mouse => NativeTabletCapabilities {
            coalesced_samples: has_coalesced_samples,
            ..NativeTabletCapabilities::windows_pointer_mouse()
        },
        WindowsPointerInputKind::Pen => NativeTabletCapabilities::windows_pointer(),
        WindowsPointerInputKind::Touch => NativeTabletCapabilities {
            pressure: false,
            tilt: false,
            twist: false,
            tangential_pressure: false,
            hover: false,
            eraser: false,
            barrel_buttons: false,
            coalesced_samples: has_coalesced_samples,
            predicted_samples: false,
            calibration: true,
        },
    };
    for sample in samples {
        capabilities.pressure |= sample.pressure.is_some();
        capabilities.tilt |= sample.tilt.is_some();
        capabilities.twist |= sample.twist_degrees.is_some();
        capabilities.eraser |= sample.eraser;
        capabilities.barrel_buttons |=
            sample.barrel_buttons.primary || sample.barrel_buttons.secondary;
        capabilities.predicted_samples |= sample.predicted_position.is_some();
    }
    capabilities.coalesced_samples = has_coalesced_samples;
    capabilities
}

fn contact_from_windows_sample(sample: WindowsPointerHistorySample) -> PointerContactState {
    if sample.in_contact {
        PointerContactState::Contact
    } else if sample.in_range {
        PointerContactState::Hover
    } else {
        PointerContactState::OutOfRange
    }
}

fn event_button_for_sample(
    kind: PointerEventKind,
    sample: WindowsPointerHistorySample,
) -> Option<PointerButton> {
    match sample.input_kind {
        WindowsPointerInputKind::Mouse => match kind {
            PointerEventKind::Down | PointerEventKind::Up => sample.event_button,
            PointerEventKind::Move if sample.primary_button_down => Some(PointerButton::Left),
            PointerEventKind::Move | PointerEventKind::Enter | PointerEventKind::Leave => None,
        },
        WindowsPointerInputKind::Pen | WindowsPointerInputKind::Touch => match kind {
            PointerEventKind::Down | PointerEventKind::Up | PointerEventKind::Move => {
                Some(PointerButton::Left)
            }
            PointerEventKind::Enter | PointerEventKind::Leave => None,
        },
    }
}

fn delta_from(previous: Option<PointerPosition>, position: PointerPosition) -> PointerDelta {
    let Some(previous) = previous else {
        return PointerDelta::ZERO;
    };
    PointerDelta::new(position.x - previous.x, position.y - previous.y)
}

#[derive(Debug, Default, Clone)]
struct SharedWindowsPointerQueue {
    inner: std::rc::Rc<std::cell::RefCell<VecDeque<WindowsPointerHistoryPacket>>>,
}

impl SharedWindowsPointerQueue {
    fn push(&self, packet: WindowsPointerHistoryPacket) {
        self.inner.borrow_mut().push_back(packet);
    }

    fn pop_front(&self) -> Option<WindowsPointerHistoryPacket> {
        self.inner.borrow_mut().pop_front()
    }

    fn clear(&self) {
        self.inner.borrow_mut().clear();
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::ffi::c_void;
    use std::ptr;

    use windows_sys::Win32::Foundation::{HWND, POINT};
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::Accessibility::RegisterPointerInputTargetEx;
    use windows_sys::Win32::UI::Input::Pointer::{
        GetPointerFrameInfoHistory, GetPointerFramePenInfoHistory, GetPointerFrameTouchInfoHistory,
        GetPointerInfo, GetPointerPenInfo, GetPointerTouchInfo, GetPointerType,
        POINTER_CHANGE_FIFTHBUTTON_DOWN, POINTER_CHANGE_FIFTHBUTTON_UP,
        POINTER_CHANGE_FIRSTBUTTON_DOWN, POINTER_CHANGE_FIRSTBUTTON_UP,
        POINTER_CHANGE_FOURTHBUTTON_DOWN, POINTER_CHANGE_FOURTHBUTTON_UP, POINTER_CHANGE_NONE,
        POINTER_CHANGE_SECONDBUTTON_DOWN, POINTER_CHANGE_SECONDBUTTON_UP,
        POINTER_CHANGE_THIRDBUTTON_DOWN, POINTER_CHANGE_THIRDBUTTON_UP, POINTER_FLAG_CANCELED,
        POINTER_FLAG_FIRSTBUTTON, POINTER_FLAG_INCONTACT, POINTER_FLAG_INRANGE, POINTER_INFO,
        POINTER_PEN_INFO, POINTER_TOUCH_INFO,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MSG, PEN_FLAG_BARREL, PEN_FLAG_ERASER, PEN_FLAG_INVERTED, PEN_MASK_PRESSURE,
        PEN_MASK_ROTATION, PEN_MASK_TILT_X, PEN_MASK_TILT_Y, PT_MOUSE, PT_PEN, PT_POINTER,
        PT_TOUCH, TOUCH_MASK_PRESSURE, WM_POINTERDOWN, WM_POINTERENTER, WM_POINTERLEAVE,
        WM_POINTERUP, WM_POINTERUPDATE,
    };
    use winit::platform::windows::EventLoopBuilderExtWindows;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    pub(super) fn install_windows_pointer_message_hook(
        builder: &mut EventLoopBuilder<()>,
        queue: SharedWindowsPointerQueue,
    ) {
        builder.with_msg_hook(move |message| {
            if let Some(history) = read_windows_pointer_history(message) {
                queue.push(history);
            }
            false
        });
    }

    pub(super) fn attach_windows_pointer_target(
        window: &winit::window::Window,
    ) -> NativeTabletBackendHealth {
        let Some(hwnd) = window_hwnd(window) else {
            return NativeTabletBackendHealth::unavailable(
                NativeTabletBackendKind::WindowsPointer,
                "window handle is not a Win32 HWND",
            );
        };

        // SAFETY: HWND belongs to the live winit window on this UI thread. The calls register
        // observation for pointer messages; they do not take ownership of the HWND.
        let pen_ok = unsafe { RegisterPointerInputTargetEx(hwnd, PT_PEN, 1) != 0 };
        // SAFETY: Same HWND lifetime as above. Registering touch keeps the backend able to
        // distinguish pen from non-pen pointer streams and report diagnostics consistently.
        let touch_ok = unsafe { RegisterPointerInputTargetEx(hwnd, PT_TOUCH, 1) != 0 };
        // SAFETY: Same HWND lifetime as above. Mouse registration provides pointer history before
        // the generic winit mouse fallback runs.
        let mouse_ok = unsafe { RegisterPointerInputTargetEx(hwnd, PT_MOUSE, 1) != 0 };
        if pen_ok || touch_ok || mouse_ok {
            NativeTabletBackendHealth::available(
                NativeTabletBackendKind::WindowsPointer,
                "attached to Win32 pointer message stream",
            )
        } else {
            NativeTabletBackendHealth::unavailable(
                NativeTabletBackendKind::WindowsPointer,
                "RegisterPointerInputTargetEx failed for pen, touch, and mouse",
            )
        }
    }

    fn read_windows_pointer_history(message: *const c_void) -> Option<WindowsPointerHistoryPacket> {
        let msg = message.cast::<MSG>();
        if msg.is_null() {
            return None;
        }
        // SAFETY: winit passes a valid pointer to the MSG currently being dispatched for the
        // duration of the callback.
        let msg = unsafe { &*msg };
        let kind = pointer_event_kind(msg.message)?;
        let pointer_id = pointer_id_from_wparam(msg.wParam);
        match pointer_type(pointer_id) {
            Some(PT_MOUSE) => read_mouse_history(pointer_id, msg.hwnd, kind),
            Some(PT_PEN) => read_pen_history(pointer_id, msg.hwnd, kind),
            Some(PT_TOUCH) => read_touch_history(pointer_id, msg.hwnd, kind),
            _ => read_pen_history(pointer_id, msg.hwnd, kind)
                .or_else(|| read_touch_history(pointer_id, msg.hwnd, kind))
                .or_else(|| read_mouse_history(pointer_id, msg.hwnd, kind)),
        }
    }

    fn pointer_type(pointer_id: u32) -> Option<i32> {
        let mut pointer_type = PT_POINTER;
        // SAFETY: pointer_type is a valid out pointer for the current pointer message id.
        let ok = unsafe { GetPointerType(pointer_id, &mut pointer_type) != 0 };
        ok.then_some(pointer_type)
    }

    fn read_mouse_history(
        pointer_id: u32,
        hwnd: HWND,
        kind: PointerEventKind,
    ) -> Option<WindowsPointerHistoryPacket> {
        let mut entries = 0_u32;
        let mut pointer_count = 0_u32;
        // SAFETY: The first call intentionally passes a null output buffer to query dimensions.
        unsafe {
            GetPointerFrameInfoHistory(
                pointer_id,
                &mut entries,
                &mut pointer_count,
                ptr::null_mut(),
            );
        }
        if entries == 0 || pointer_count == 0 {
            return read_single_mouse(pointer_id, hwnd, kind);
        }

        let mut infos =
            vec![POINTER_INFO::default(); entries.saturating_mul(pointer_count) as usize];
        let mut entries_in_out = entries;
        let mut pointer_count_in_out = pointer_count;
        // SAFETY: Buffer has entries * pointer_count elements as required by the Win32 API.
        let ok = unsafe {
            GetPointerFrameInfoHistory(
                pointer_id,
                &mut entries_in_out,
                &mut pointer_count_in_out,
                infos.as_mut_ptr(),
            ) != 0
        };
        if !ok {
            return read_single_mouse(pointer_id, hwnd, kind);
        }

        let samples = infos
            .into_iter()
            .filter(|info| info.pointerId == pointer_id && info.pointerType == PT_MOUSE)
            .map(|info| pointer_info_to_mouse_sample(info, hwnd))
            .collect::<Vec<_>>();
        if samples.is_empty() {
            None
        } else {
            Some(WindowsPointerHistoryPacket::new(kind, pointer_id, samples))
        }
    }

    fn read_single_mouse(
        pointer_id: u32,
        hwnd: HWND,
        kind: PointerEventKind,
    ) -> Option<WindowsPointerHistoryPacket> {
        let mut info = POINTER_INFO::default();
        // SAFETY: info is a valid out pointer for the current pointer message on this thread.
        let ok = unsafe { GetPointerInfo(pointer_id, &mut info) != 0 };
        ok.then(|| {
            WindowsPointerHistoryPacket::new(
                kind,
                pointer_id,
                [pointer_info_to_mouse_sample(info, hwnd)],
            )
        })
    }

    fn read_pen_history(
        pointer_id: u32,
        hwnd: HWND,
        kind: PointerEventKind,
    ) -> Option<WindowsPointerHistoryPacket> {
        let mut entries = 0_u32;
        let mut pointer_count = 0_u32;
        // SAFETY: The first call intentionally passes a null output buffer to query dimensions.
        unsafe {
            GetPointerFramePenInfoHistory(
                pointer_id,
                &mut entries,
                &mut pointer_count,
                ptr::null_mut(),
            );
        }
        if entries == 0 || pointer_count == 0 {
            return read_single_pen(pointer_id, hwnd, kind);
        }

        let mut infos =
            vec![POINTER_PEN_INFO::default(); entries.saturating_mul(pointer_count) as usize];
        let mut entries_in_out = entries;
        let mut pointer_count_in_out = pointer_count;
        // SAFETY: Buffer has entries * pointer_count elements as required by the Win32 API.
        let ok = unsafe {
            GetPointerFramePenInfoHistory(
                pointer_id,
                &mut entries_in_out,
                &mut pointer_count_in_out,
                infos.as_mut_ptr(),
            ) != 0
        };
        if !ok {
            return read_single_pen(pointer_id, hwnd, kind);
        }

        let samples = infos
            .into_iter()
            .filter(|info| info.pointerInfo.pointerId == pointer_id)
            .map(|info| pen_info_to_sample(info, hwnd))
            .collect::<Vec<_>>();
        if samples.is_empty() {
            None
        } else {
            Some(WindowsPointerHistoryPacket::new(kind, pointer_id, samples))
        }
    }

    fn read_single_pen(
        pointer_id: u32,
        hwnd: HWND,
        kind: PointerEventKind,
    ) -> Option<WindowsPointerHistoryPacket> {
        let mut info = POINTER_PEN_INFO::default();
        // SAFETY: info is a valid out pointer for the current pointer message on this thread.
        let ok = unsafe { GetPointerPenInfo(pointer_id, &mut info) != 0 };
        ok.then(|| {
            WindowsPointerHistoryPacket::new(kind, pointer_id, [pen_info_to_sample(info, hwnd)])
        })
    }

    fn read_touch_history(
        pointer_id: u32,
        hwnd: HWND,
        kind: PointerEventKind,
    ) -> Option<WindowsPointerHistoryPacket> {
        let mut entries = 0_u32;
        let mut pointer_count = 0_u32;
        // SAFETY: The first call intentionally passes a null output buffer to query dimensions.
        unsafe {
            GetPointerFrameTouchInfoHistory(
                pointer_id,
                &mut entries,
                &mut pointer_count,
                ptr::null_mut(),
            );
        }
        if entries == 0 || pointer_count == 0 {
            return read_single_touch(pointer_id, hwnd, kind);
        }

        let mut infos =
            vec![POINTER_TOUCH_INFO::default(); entries.saturating_mul(pointer_count) as usize];
        let mut entries_in_out = entries;
        let mut pointer_count_in_out = pointer_count;
        // SAFETY: Buffer has entries * pointer_count elements as required by the Win32 API.
        let ok = unsafe {
            GetPointerFrameTouchInfoHistory(
                pointer_id,
                &mut entries_in_out,
                &mut pointer_count_in_out,
                infos.as_mut_ptr(),
            ) != 0
        };
        if !ok {
            return read_single_touch(pointer_id, hwnd, kind);
        }

        let samples = infos
            .into_iter()
            .filter(|info| info.pointerInfo.pointerId == pointer_id)
            .map(|info| touch_info_to_sample(info, hwnd))
            .collect::<Vec<_>>();
        if samples.is_empty() {
            None
        } else {
            Some(WindowsPointerHistoryPacket::new(kind, pointer_id, samples))
        }
    }

    fn read_single_touch(
        pointer_id: u32,
        hwnd: HWND,
        kind: PointerEventKind,
    ) -> Option<WindowsPointerHistoryPacket> {
        let mut info = POINTER_TOUCH_INFO::default();
        // SAFETY: info is a valid out pointer for the current pointer message on this thread.
        let ok = unsafe { GetPointerTouchInfo(pointer_id, &mut info) != 0 };
        ok.then(|| {
            WindowsPointerHistoryPacket::new(kind, pointer_id, [touch_info_to_sample(info, hwnd)])
        })
    }

    fn pen_info_to_sample(info: POINTER_PEN_INFO, hwnd: HWND) -> WindowsPointerHistorySample {
        let position = client_position(
            info.pointerInfo.ptPixelLocation,
            info.pointerInfo.hwndTarget,
            hwnd,
        );
        let mut sample = WindowsPointerHistorySample::pen(info.pointerInfo.pointerId, position);
        sample.timestamp_micros = Some(u64::from(info.pointerInfo.dwTime) * 1_000);
        if info.penMask & PEN_MASK_PRESSURE != 0 {
            sample.pressure = Some(info.pressure as f32 / 1024.0);
        }
        let source_device = info.pointerInfo.sourceDevice as usize;
        sample.source_device = (source_device != 0).then_some(source_device as u64);
        let has_tilt_x = info.penMask & PEN_MASK_TILT_X != 0;
        let has_tilt_y = info.penMask & PEN_MASK_TILT_Y != 0;
        if has_tilt_x && has_tilt_y {
            sample.tilt = Some(PointerTilt::new(info.tiltX as f32, info.tiltY as f32));
        }
        if info.penMask & PEN_MASK_ROTATION != 0 {
            sample.twist_degrees = Some(info.rotation as f32);
        }
        sample.eraser = info.penFlags & (PEN_FLAG_ERASER | PEN_FLAG_INVERTED) != 0;
        sample.barrel_buttons = PointerBarrelButtons {
            primary: info.penFlags & PEN_FLAG_BARREL != 0,
            secondary: false,
        };
        sample.in_contact = info.pointerInfo.pointerFlags & POINTER_FLAG_INCONTACT != 0;
        sample.in_range = info.pointerInfo.pointerFlags & POINTER_FLAG_INRANGE != 0;
        sample.cancelled = info.pointerInfo.pointerFlags & POINTER_FLAG_CANCELED != 0;
        sample
    }

    fn touch_info_to_sample(info: POINTER_TOUCH_INFO, hwnd: HWND) -> WindowsPointerHistorySample {
        let position = client_position(
            info.pointerInfo.ptPixelLocationRaw,
            info.pointerInfo.hwndTarget,
            hwnd,
        );
        let mut sample = WindowsPointerHistorySample::touch(info.pointerInfo.pointerId, position);
        sample.predicted_position = Some(client_position(
            info.pointerInfo.ptPixelLocation,
            info.pointerInfo.hwndTarget,
            hwnd,
        ));
        sample.timestamp_micros = Some(u64::from(info.pointerInfo.dwTime) * 1_000);
        if info.touchMask & TOUCH_MASK_PRESSURE != 0 {
            sample.pressure = Some(info.pressure as f32 / 1024.0);
        }
        let source_device = info.pointerInfo.sourceDevice as usize;
        sample.source_device = (source_device != 0).then_some(source_device as u64);
        sample.in_contact = info.pointerInfo.pointerFlags & POINTER_FLAG_INCONTACT != 0;
        sample.in_range = info.pointerInfo.pointerFlags & POINTER_FLAG_INRANGE != 0;
        sample.cancelled = info.pointerInfo.pointerFlags & POINTER_FLAG_CANCELED != 0;
        sample
    }

    fn pointer_info_to_mouse_sample(info: POINTER_INFO, hwnd: HWND) -> WindowsPointerHistorySample {
        let position = client_position(info.ptPixelLocation, info.hwndTarget, hwnd);
        let mut sample = WindowsPointerHistorySample::mouse(info.pointerId, position);
        let source_device = info.sourceDevice as usize;
        sample.source_device = (source_device != 0).then_some(source_device as u64);
        sample.timestamp_micros = Some(u64::from(info.dwTime) * 1_000);
        sample.event_button = pointer_button_from_change(info.ButtonChangeType);
        sample.primary_button_down = info.pointerFlags & POINTER_FLAG_FIRSTBUTTON != 0;
        sample.in_contact =
            info.pointerFlags & (POINTER_FLAG_INCONTACT | POINTER_FLAG_FIRSTBUTTON) != 0;
        sample.in_range = true;
        sample.cancelled = info.pointerFlags & POINTER_FLAG_CANCELED != 0;
        sample
    }

    fn pointer_button_from_change(change: i32) -> Option<PointerButton> {
        match change {
            POINTER_CHANGE_FIRSTBUTTON_DOWN | POINTER_CHANGE_FIRSTBUTTON_UP => {
                Some(PointerButton::Left)
            }
            POINTER_CHANGE_SECONDBUTTON_DOWN | POINTER_CHANGE_SECONDBUTTON_UP => {
                Some(PointerButton::Right)
            }
            POINTER_CHANGE_THIRDBUTTON_DOWN | POINTER_CHANGE_THIRDBUTTON_UP => {
                Some(PointerButton::Middle)
            }
            POINTER_CHANGE_FOURTHBUTTON_DOWN | POINTER_CHANGE_FOURTHBUTTON_UP => {
                Some(PointerButton::Other(4))
            }
            POINTER_CHANGE_FIFTHBUTTON_DOWN | POINTER_CHANGE_FIFTHBUTTON_UP => {
                Some(PointerButton::Other(5))
            }
            POINTER_CHANGE_NONE => None,
            _ => None,
        }
    }

    fn client_position(
        mut point: POINT,
        target_hwnd: HWND,
        fallback_hwnd: HWND,
    ) -> PointerPosition {
        let hwnd = if !target_hwnd.is_null() {
            target_hwnd
        } else {
            fallback_hwnd
        };
        if !hwnd.is_null() {
            // SAFETY: point is a local mutable POINT and hwnd is the target HWND provided by the
            // pointer message. On failure, the point remains a finite screen-space fallback.
            unsafe {
                ScreenToClient(hwnd, &mut point);
            }
        }
        PointerPosition::new(point.x as f32, point.y as f32)
    }

    fn pointer_event_kind(message: u32) -> Option<PointerEventKind> {
        match message {
            WM_POINTERDOWN => Some(PointerEventKind::Down),
            WM_POINTERUP => Some(PointerEventKind::Up),
            WM_POINTERUPDATE => Some(PointerEventKind::Move),
            WM_POINTERENTER => Some(PointerEventKind::Enter),
            WM_POINTERLEAVE => Some(PointerEventKind::Leave),
            _ => None,
        }
    }

    fn pointer_id_from_wparam(wparam: usize) -> u32 {
        (wparam & 0xffff) as u32
    }

    fn window_hwnd(window: &winit::window::Window) -> Option<HWND> {
        let handle = window.window_handle().ok()?.as_raw();
        match handle {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as HWND),
            _ => None,
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;

    pub(super) fn install_windows_pointer_message_hook(
        _builder: &mut EventLoopBuilder<()>,
        _queue: SharedWindowsPointerQueue,
    ) {
    }

    pub(super) fn attach_windows_pointer_target(
        _window: &winit::window::Window,
    ) -> NativeTabletBackendHealth {
        NativeTabletBackendHealth::unavailable(
            NativeTabletBackendKind::WindowsPointer,
            "Windows Pointer/Ink is only available on Windows",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_pointer_history_maps_newest_first_to_ordered_coalesced_samples() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            7,
            [
                WindowsPointerHistorySample::pen(7, PointerPosition::new(30.0, 0.0))
                    .with_timestamp_micros(30)
                    .with_pressure(0.8),
                WindowsPointerHistorySample::pen(7, PointerPosition::new(20.0, 0.0))
                    .with_timestamp_micros(20)
                    .with_pressure(0.6),
                WindowsPointerHistorySample::pen(7, PointerPosition::new(10.0, 0.0))
                    .with_timestamp_micros(10)
                    .with_pressure(0.4),
            ],
        );

        let packet = map_windows_pointer_history(
            history,
            Some(PointerPosition::new(0.0, 0.0)),
            PointerCalibration::identity(),
        )
        .expect("history should map");

        assert_eq!(packet.position, PointerPosition::new(30.0, 0.0));
        assert_eq!(packet.delta, PointerDelta::new(10.0, 0.0));
        assert_eq!(packet.pressure, Some(0.8));
        assert_eq!(packet.coalesced_samples.len(), 2);
        assert_eq!(
            packet.coalesced_samples[0].position,
            PointerPosition::new(10.0, 0.0)
        );
        assert_eq!(
            packet.coalesced_samples[0].delta,
            PointerDelta::new(10.0, 0.0)
        );
        assert_eq!(
            packet.coalesced_samples[1].position,
            PointerPosition::new(20.0, 0.0)
        );
        assert_eq!(
            packet.coalesced_samples[1].delta,
            PointerDelta::new(10.0, 0.0)
        );
        assert!(
            packet
                .coalesced_samples
                .iter()
                .all(|sample| sample.position.x.is_finite() && sample.position.y.is_finite())
        );
    }

    #[test]
    fn windows_pointer_touch_history_maps_as_touch_source() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Down,
            9,
            [WindowsPointerHistorySample::touch(
                9,
                PointerPosition::new(14.0, 15.0),
            )],
        );

        let packet = map_windows_pointer_history(history, None, PointerCalibration::identity())
            .expect("history should map");

        assert_eq!(packet.source_kind, PointerSourceKind::Touch);
        assert_eq!(packet.tool_kind, NativeTabletToolKind::Finger);
        assert_eq!(packet.event_button, Some(PointerButton::Left));
    }

    #[test]
    fn windows_pointer_mouse_history_maps_as_mouse_source_with_ordered_samples() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            11,
            [
                WindowsPointerHistorySample::mouse(11, PointerPosition::new(36.0, 18.0))
                    .with_timestamp_micros(300),
                WindowsPointerHistorySample::mouse(11, PointerPosition::new(24.0, 12.0))
                    .with_timestamp_micros(200),
                WindowsPointerHistorySample::mouse(11, PointerPosition::new(12.0, 6.0))
                    .with_timestamp_micros(100),
            ],
        );

        let packet = map_windows_pointer_history(
            history,
            Some(PointerPosition::new(0.0, 0.0)),
            PointerCalibration::identity(),
        )
        .expect("mouse history should map");

        assert_eq!(packet.source_kind, PointerSourceKind::Mouse);
        assert_eq!(packet.tool_kind, NativeTabletToolKind::Mouse);
        assert_eq!(packet.event_button, Some(PointerButton::Left));
        assert_eq!(packet.position, PointerPosition::new(36.0, 18.0));
        assert_eq!(packet.delta, PointerDelta::new(12.0, 6.0));
        assert_eq!(packet.coalesced_samples.len(), 2);
        assert_eq!(
            packet.coalesced_samples[0].position,
            PointerPosition::new(12.0, 6.0)
        );
        assert_eq!(
            packet.coalesced_samples[1].position,
            PointerPosition::new(24.0, 12.0)
        );
        assert!(packet.capabilities.coalesced_samples);
        assert!(packet.capabilities.hover);
        assert!(!packet.capabilities.pressure);
    }

    #[test]
    fn windows_pointer_mouse_secondary_contact_does_not_start_native_primary_stream() {
        let mut sample = WindowsPointerHistorySample::mouse(12, PointerPosition::new(8.0, 9.0));
        sample.event_button = Some(PointerButton::Right);

        let history = WindowsPointerHistoryPacket::new(PointerEventKind::Down, 12, [sample]);

        assert!(
            map_windows_pointer_history(history, None, PointerCalibration::identity()).is_err(),
            "secondary mouse down must not become a native primary drawing stream"
        );
    }

    #[test]
    fn windows_pointer_keeps_source_device_separate_from_pointer_contact() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            17,
            [
                WindowsPointerHistorySample::pen(17, PointerPosition::new(1.0, 2.0))
                    .with_source_device(9001),
            ],
        );

        let packet = map_windows_pointer_history(history, None, PointerCalibration::identity())
            .expect("pointer sample should map");

        assert_eq!(packet.device_id, Some(9001));
        assert_eq!(packet.contact_id, 17);
    }

    #[test]
    fn windows_pointer_does_not_synthesize_device_from_pointer_contact() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            18,
            [WindowsPointerHistorySample::pen(
                18,
                PointerPosition::new(1.0, 2.0),
            )],
        );

        let packet = map_windows_pointer_history(history, None, PointerCalibration::identity())
            .expect("pointer sample should map");

        assert_eq!(packet.device_id, None);
        assert_eq!(packet.contact_id, 18);
    }

    #[test]
    fn windows_pointer_rejects_conflicting_source_devices_atomically() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            19,
            [
                WindowsPointerHistorySample::pen(19, PointerPosition::new(1.0, 2.0))
                    .with_source_device(9001),
                WindowsPointerHistorySample::pen(19, PointerPosition::new(3.0, 4.0))
                    .with_source_device(9002),
            ],
        );

        assert!(matches!(
            map_windows_pointer_history(history, None, PointerCalibration::identity()),
            Err(crate::model::NativeTabletDiagnostic::ConflictingDeviceIdentity)
        ));
    }

    #[test]
    fn windows_pointer_history_preserves_historical_only_capabilities() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            20,
            [
                WindowsPointerHistorySample::pen(20, PointerPosition::new(30.0, 0.0)),
                WindowsPointerHistorySample::pen(20, PointerPosition::new(20.0, 0.0))
                    .with_pressure(0.8)
                    .with_tilt(PointerTilt::new(10.0, -5.0)),
            ],
        );

        let packet = map_windows_pointer_history(history, None, PointerCalibration::identity())
            .expect("history should map");

        assert_eq!(packet.pressure, None);
        assert_eq!(packet.tilt, None);
        assert!(packet.capabilities.pressure);
        assert!(packet.capabilities.tilt);
    }

    #[test]
    fn windows_pointer_touch_preserves_raw_and_predicted_positions_separately() {
        let history = WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            21,
            [
                WindowsPointerHistorySample::touch(21, PointerPosition::new(10.0, 20.0))
                    .with_predicted_position(PointerPosition::new(11.0, 21.0)),
            ],
        );

        let packet = map_windows_pointer_history(history, None, PointerCalibration::identity())
            .expect("touch history should map");

        assert_eq!(packet.position, PointerPosition::new(10.0, 20.0));
        assert_eq!(packet.predicted_samples.len(), 1);
        assert_eq!(
            packet.predicted_samples[0].position,
            PointerPosition::new(11.0, 21.0)
        );
        assert!(packet.capabilities.predicted_samples);
    }

    #[test]
    fn windows_pointer_cancellation_is_terminal_and_reused_ids_start_without_stale_delta() {
        let mut backend = WindowsPointerBackend::new();
        let control = NativeTabletDeviceControlResource::default();
        let mut runtime = NativeTabletRuntimeResource::default();
        let mut frame = crate::model::NativeTabletFrameResource::default();

        backend.push_history_for_test(WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            22,
            [
                WindowsPointerHistorySample::pen(22, PointerPosition::new(100.0, 100.0))
                    .with_cancelled(true),
            ],
        ));
        backend.frame(&mut runtime, &control);
        runtime.publish_frame(
            &mut frame,
            &mut NativeTabletDeviceControlResource::default(),
        );
        assert_eq!(frame.packets[0].kind, PointerEventKind::Leave);
        frame.publish_to_neutral(&mut engine::plugins::InputState::new());

        backend.push_history_for_test(WindowsPointerHistoryPacket::new(
            PointerEventKind::Move,
            22,
            [WindowsPointerHistorySample::pen(
                22,
                PointerPosition::new(150.0, 100.0),
            )],
        ));
        backend.frame(&mut runtime, &control);
        runtime.publish_frame(
            &mut frame,
            &mut NativeTabletDeviceControlResource::default(),
        );
        assert_eq!(frame.packets[0].delta, PointerDelta::ZERO);
    }
}
