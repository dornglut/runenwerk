//! File: domain/drawing/src/tile/coordinate.rs
//! Purpose: Stable logical canvas coordinate and tile identifiers.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasCoordinate {
    pub x: f64,
    pub y: f64,
}

impl CanvasCoordinate {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasRect {
    pub min: CanvasCoordinate,
    pub max: CanvasCoordinate,
}

impl CanvasRect {
    pub const fn new(min: CanvasCoordinate, max: CanvasCoordinate) -> Self {
        Self { min, max }
    }

    pub fn from_points(points: impl IntoIterator<Item = CanvasCoordinate>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;
        if !first.is_finite() {
            return None;
        }
        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x;
        let mut max_y = first.y;

        for point in points {
            if !point.is_finite() {
                return None;
            }
            min_x = min_x.min(point.x);
            min_y = min_y.min(point.y);
            max_x = max_x.max(point.x);
            max_y = max_y.max(point.y);
        }

        Some(Self {
            min: CanvasCoordinate::new(min_x, min_y),
            max: CanvasCoordinate::new(max_x, max_y),
        })
    }

    pub fn is_valid(self) -> bool {
        self.min.is_finite()
            && self.max.is_finite()
            && self.min.x <= self.max.x
            && self.min.y <= self.max.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TilePyramidLevel(pub u32);

impl TilePyramidLevel {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanvasTileId {
    pub level: TilePyramidLevel,
    pub x: i64,
    pub y: i64,
}

impl CanvasTileId {
    pub const fn new(level: TilePyramidLevel, x: i64, y: i64) -> Self {
        Self { level, x, y }
    }
}
