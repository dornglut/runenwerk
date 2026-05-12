//! Drawing tool input routing.

use drawing::{CanvasCoordinate, StrokeId, StrokeSample, StrokeToolKind, StylusTilt};
use ui_input::{
    PointerBarrelButtons, PointerContactState, PointerDeviceId, PointerEvent, PointerEventKind,
    PointerLatencyClass, PointerSourceKind, PointerToolKind,
};
use ui_math::UiPoint;

use crate::app::DrawingCanvasView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingToolRouteKind {
    Hover,
    BeginPreviewStroke,
    UpdatePreviewStroke,
    EndPreviewStroke,
    Scroll,
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingToolInputEvent {
    pub route_kind: DrawingToolRouteKind,
    pub pointer_kind: PointerEventKind,
    pub screen_position: UiPoint,
    pub canvas_position: Option<CanvasCoordinate>,
    pub source_kind: PointerSourceKind,
    pub tool_kind: PointerToolKind,
    pub device_id: Option<PointerDeviceId>,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<StylusTilt>,
    pub twist_degrees: Option<f32>,
    pub eraser: bool,
    pub barrel_buttons: PointerBarrelButtons,
    pub low_latency_preview: bool,
    pub coalesced_sample_count: usize,
    pub predicted_sample_count: usize,
}

impl DrawingToolInputEvent {
    pub fn from_pointer(pointer: &PointerEvent, canvas_view: DrawingCanvasView) -> Self {
        let canvas_position = canvas_view.screen_to_canvas(pointer.position);
        let route_kind = route_kind(pointer, canvas_position);
        Self {
            route_kind,
            pointer_kind: pointer.kind,
            screen_position: pointer.position,
            canvas_position,
            source_kind: pointer.packet.source_kind,
            tool_kind: pointer.packet.tool_kind,
            device_id: pointer.packet.device_id,
            timestamp_micros: pointer.packet.timestamp_micros,
            pressure: pointer.packet.pressure,
            tilt: pointer
                .packet
                .tilt
                .map(|tilt| StylusTilt::new(tilt.x_degrees, tilt.y_degrees)),
            twist_degrees: pointer.packet.twist_degrees,
            eraser: pointer.packet.eraser,
            barrel_buttons: pointer.packet.barrel_buttons,
            low_latency_preview: pointer.packet.latency_class
                == PointerLatencyClass::LowLatencyPreview,
            coalesced_sample_count: pointer.packet.coalesced_samples.len(),
            predicted_sample_count: pointer.packet.predicted_samples.len(),
        }
    }

    pub fn to_stroke_sample(self, sequence: u64) -> Option<StrokeSample> {
        let position = self.canvas_position?;
        let mut sample = StrokeSample::new(position, sequence);
        if let Some(timestamp_micros) = self.timestamp_micros {
            sample = sample.with_timestamp_micros(timestamp_micros);
        }
        if let Some(pressure) = self.pressure {
            sample = sample.with_pressure(pressure);
        }
        if let Some(tilt) = self.tilt {
            sample = sample.with_tilt(tilt);
        }
        if let Some(twist_degrees) = self.twist_degrees {
            sample = sample.with_twist_degrees(twist_degrees);
        }
        sample = sample.with_tool_kind(stroke_tool_kind(self.tool_kind, self.eraser));
        Some(sample)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingPreviewStroke {
    pub stroke_id: StrokeId,
    pub active: bool,
    pub samples: Vec<StrokeSample>,
}

impl DrawingPreviewStroke {
    pub fn new(stroke_id: StrokeId) -> Self {
        Self {
            stroke_id,
            active: true,
            samples: Vec::new(),
        }
    }

    pub fn append_sample(&mut self, sample: StrokeSample) {
        self.samples.push(sample);
    }

    pub fn finish(&mut self) {
        self.active = false;
    }
}

fn route_kind(
    pointer: &PointerEvent,
    canvas_position: Option<CanvasCoordinate>,
) -> DrawingToolRouteKind {
    if pointer.packet.contact == PointerContactState::Hover {
        return DrawingToolRouteKind::Hover;
    }
    if canvas_position.is_none() {
        return DrawingToolRouteKind::Ignored;
    }
    match pointer.kind {
        PointerEventKind::Down => DrawingToolRouteKind::BeginPreviewStroke,
        PointerEventKind::Move => DrawingToolRouteKind::UpdatePreviewStroke,
        PointerEventKind::Up => DrawingToolRouteKind::EndPreviewStroke,
        PointerEventKind::Scroll => DrawingToolRouteKind::Scroll,
        PointerEventKind::Enter | PointerEventKind::Leave => DrawingToolRouteKind::Ignored,
    }
}

fn stroke_tool_kind(tool_kind: PointerToolKind, eraser: bool) -> StrokeToolKind {
    if eraser || tool_kind == PointerToolKind::Eraser {
        return StrokeToolKind::Eraser;
    }
    match tool_kind {
        PointerToolKind::Pen => StrokeToolKind::Pen,
        PointerToolKind::Brush | PointerToolKind::Airbrush => StrokeToolKind::Brush,
        PointerToolKind::Marker => StrokeToolKind::Marker,
        PointerToolKind::Mouse
        | PointerToolKind::Finger
        | PointerToolKind::Eraser
        | PointerToolKind::Unknown => StrokeToolKind::Unknown,
    }
}
