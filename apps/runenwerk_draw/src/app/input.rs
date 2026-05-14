//! Drawing tool input routing.

use drawing::{CanvasCoordinate, StrokeId, StrokeSample, StrokeToolKind, StylusTilt};
use ui_input::{
    PointerBarrelButtons, PointerContactState, PointerDeviceId, PointerEvent, PointerEventKind,
    PointerLatencyClass, PointerSample, PointerSourceKind, PointerToolKind,
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
    pub coalesced_samples: Vec<DrawingToolInputSample>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingToolInputSample {
    pub screen_position: UiPoint,
    pub canvas_position: Option<CanvasCoordinate>,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<StylusTilt>,
    pub twist_degrees: Option<f32>,
}

impl DrawingToolInputEvent {
    pub fn from_pointer(pointer: &PointerEvent, canvas_view: DrawingCanvasView) -> Self {
        Self::from_pointer_with_capture(pointer, canvas_view, false)
    }

    pub fn from_pointer_with_capture(
        pointer: &PointerEvent,
        canvas_view: DrawingCanvasView,
        capture_active: bool,
    ) -> Self {
        let bounded_canvas_position = canvas_view.screen_to_canvas(pointer.position);
        let canvas_position = if capture_active {
            canvas_view
                .screen_to_canvas_unbounded(pointer.position)
                .or(bounded_canvas_position)
        } else {
            bounded_canvas_position
        };
        let route_kind = route_kind(
            pointer,
            bounded_canvas_position,
            canvas_position,
            capture_active,
        );
        let coalesced_samples = pointer
            .packet
            .coalesced_samples
            .iter()
            .filter(|sample| sample.contact == PointerContactState::Contact)
            .map(|sample| {
                DrawingToolInputSample::from_pointer_sample(sample, canvas_view, capture_active)
            })
            .collect::<Vec<_>>();
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
            coalesced_samples,
        }
    }

    pub fn to_stroke_sample(&self, sequence: u64) -> Option<StrokeSample> {
        let position = self.canvas_position?;
        Some(stroke_sample_from_input(
            StrokeSampleInput {
                position,
                timestamp_micros: self.timestamp_micros,
                pressure: self.pressure,
                tilt: self.tilt,
                twist_degrees: self.twist_degrees,
                tool_kind: self.tool_kind,
                eraser: self.eraser,
            },
            sequence,
        ))
    }

    pub fn to_stroke_samples(&self, next_sequence: &mut u64) -> Vec<StrokeSample> {
        let mut samples = Vec::with_capacity(self.coalesced_samples.len() + 1);
        for input_sample in &self.coalesced_samples {
            let Some(position) = input_sample.canvas_position else {
                continue;
            };
            samples.push(stroke_sample_from_input(
                StrokeSampleInput {
                    position,
                    timestamp_micros: input_sample.timestamp_micros,
                    pressure: input_sample.pressure,
                    tilt: input_sample.tilt,
                    twist_degrees: input_sample.twist_degrees,
                    tool_kind: self.tool_kind,
                    eraser: self.eraser,
                },
                *next_sequence,
            ));
            *next_sequence = (*next_sequence).saturating_add(1);
        }
        if let Some(sample) = self.to_stroke_sample(*next_sequence) {
            samples.push(sample);
            *next_sequence = (*next_sequence).saturating_add(1);
        }
        samples
    }
}

impl DrawingToolInputSample {
    fn from_pointer_sample(
        sample: &PointerSample,
        canvas_view: DrawingCanvasView,
        capture_active: bool,
    ) -> Self {
        let canvas_position = if capture_active {
            canvas_view.screen_to_canvas_unbounded(sample.position)
        } else {
            canvas_view.screen_to_canvas(sample.position)
        };
        Self {
            screen_position: sample.position,
            canvas_position,
            timestamp_micros: sample.timestamp_micros,
            pressure: sample.pressure,
            tilt: sample
                .tilt
                .map(|tilt| StylusTilt::new(tilt.x_degrees, tilt.y_degrees)),
            twist_degrees: sample.twist_degrees,
        }
    }
}

#[derive(Clone, Copy)]
struct StrokeSampleInput {
    position: CanvasCoordinate,
    timestamp_micros: Option<u64>,
    pressure: Option<f32>,
    tilt: Option<StylusTilt>,
    twist_degrees: Option<f32>,
    tool_kind: PointerToolKind,
    eraser: bool,
}

fn stroke_sample_from_input(input: StrokeSampleInput, sequence: u64) -> StrokeSample {
    let StrokeSampleInput {
        position,
        timestamp_micros,
        pressure,
        tilt,
        twist_degrees,
        tool_kind,
        eraser,
    } = input;
    let mut sample = StrokeSample::new(position, sequence);
    if let Some(timestamp_micros) = timestamp_micros {
        sample = sample.with_timestamp_micros(timestamp_micros);
    }
    if let Some(pressure) = pressure {
        sample = sample.with_pressure(pressure);
    }
    if let Some(tilt) = tilt {
        sample = sample.with_tilt(tilt);
    }
    if let Some(twist_degrees) = twist_degrees {
        sample = sample.with_twist_degrees(twist_degrees);
    }
    sample.with_tool_kind(stroke_tool_kind(tool_kind, eraser))
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
    bounded_canvas_position: Option<CanvasCoordinate>,
    canvas_position: Option<CanvasCoordinate>,
    capture_active: bool,
) -> DrawingToolRouteKind {
    if pointer.packet.contact == PointerContactState::Hover {
        return DrawingToolRouteKind::Hover;
    }
    match pointer.kind {
        PointerEventKind::Down if bounded_canvas_position.is_some() => {
            DrawingToolRouteKind::BeginPreviewStroke
        }
        PointerEventKind::Move if capture_active && canvas_position.is_some() => {
            DrawingToolRouteKind::UpdatePreviewStroke
        }
        PointerEventKind::Up if capture_active && canvas_position.is_some() => {
            DrawingToolRouteKind::EndPreviewStroke
        }
        PointerEventKind::Scroll => DrawingToolRouteKind::Scroll,
        PointerEventKind::Down
        | PointerEventKind::Move
        | PointerEventKind::Up
        | PointerEventKind::Enter
        | PointerEventKind::Leave => DrawingToolRouteKind::Ignored,
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
