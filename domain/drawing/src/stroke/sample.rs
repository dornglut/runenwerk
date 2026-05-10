//! File: domain/drawing/src/stroke/sample.rs
//! Purpose: Platform-neutral committed stroke sample facts.

use crate::CanvasCoordinate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrokeToolKind {
    Pen,
    Brush,
    Marker,
    Eraser,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StylusTilt {
    pub x_degrees: f32,
    pub y_degrees: f32,
}

impl StylusTilt {
    pub const fn new(x_degrees: f32, y_degrees: f32) -> Self {
        Self {
            x_degrees,
            y_degrees,
        }
    }

    pub fn is_valid(self) -> bool {
        self.x_degrees.is_finite()
            && self.y_degrees.is_finite()
            && (-90.0..=90.0).contains(&self.x_degrees)
            && (-90.0..=90.0).contains(&self.y_degrees)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrokeSample {
    pub position: CanvasCoordinate,
    pub sequence: u64,
    pub timestamp_micros: Option<u64>,
    pub pressure: Option<f32>,
    pub tilt: Option<StylusTilt>,
    pub twist_degrees: Option<f32>,
    pub tool_kind: Option<StrokeToolKind>,
}

impl StrokeSample {
    pub const fn new(position: CanvasCoordinate, sequence: u64) -> Self {
        Self {
            position,
            sequence,
            timestamp_micros: None,
            pressure: None,
            tilt: None,
            twist_degrees: None,
            tool_kind: None,
        }
    }

    pub fn with_pressure(mut self, pressure: f32) -> Self {
        self.pressure = Some(pressure);
        self
    }

    pub fn with_timestamp_micros(mut self, timestamp_micros: u64) -> Self {
        self.timestamp_micros = Some(timestamp_micros);
        self
    }

    pub fn with_tilt(mut self, tilt: StylusTilt) -> Self {
        self.tilt = Some(tilt);
        self
    }

    pub fn with_twist_degrees(mut self, twist_degrees: f32) -> Self {
        self.twist_degrees = Some(twist_degrees);
        self
    }

    pub fn with_tool_kind(mut self, tool_kind: StrokeToolKind) -> Self {
        self.tool_kind = Some(tool_kind);
        self
    }
}
