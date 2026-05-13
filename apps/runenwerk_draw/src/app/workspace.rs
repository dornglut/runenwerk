//! Canvas-first drawing workspace projection.

use drawing::{CanvasCoordinate, CanvasRect};
use ui_math::{UiPoint, UiRect, UiSize};

const MIN_CANVAS_MARGIN: f32 = 24.0;
const LEFT_TOOLBAR_WIDTH: f32 = 56.0;
const RIGHT_PANEL_WIDTH: f32 = 280.0;
const TOP_BAR_HEIGHT: f32 = 36.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawingCanvasView {
    pub canvas_bounds: CanvasRect,
    pub screen_bounds: UiRect,
    pub zoom: f64,
    pub pan: CanvasCoordinate,
}

impl DrawingCanvasView {
    pub fn screen_to_canvas(self, screen_position: ui_math::UiPoint) -> Option<CanvasCoordinate> {
        if self.zoom <= 0.0 || !self.screen_bounds.contains(screen_position) {
            return None;
        }

        let local_x = f64::from(screen_position.x - self.screen_bounds.x);
        let local_y = f64::from(screen_position.y - self.screen_bounds.y);
        Some(CanvasCoordinate::new(
            self.pan.x + local_x / self.zoom,
            self.pan.y + local_y / self.zoom,
        ))
    }

    pub fn canvas_to_screen(self, canvas_position: CanvasCoordinate) -> Option<UiPoint> {
        if self.zoom <= 0.0 || !canvas_position.is_finite() {
            return None;
        }
        Some(UiPoint::new(
            self.screen_bounds.x + ((canvas_position.x - self.pan.x) * self.zoom) as f32,
            self.screen_bounds.y + ((canvas_position.y - self.pan.y) * self.zoom) as f32,
        ))
    }

    pub fn canvas_rect_to_screen(self, rect: CanvasRect) -> Option<UiRect> {
        let min = self.canvas_to_screen(rect.min)?;
        let max = self.canvas_to_screen(rect.max)?;
        Some(UiRect::new(
            min.x.min(max.x),
            min.y.min(max.y),
            (max.x - min.x).abs(),
            (max.y - min.y).abs(),
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawingWorkspaceProjection {
    pub window_size: UiSize,
    pub toolbar_bounds: UiRect,
    pub layer_panel_bounds: UiRect,
    pub canvas_view: DrawingCanvasView,
}

impl DrawingWorkspaceProjection {
    pub fn canvas_first(window_size: UiSize, canvas_bounds: CanvasRect) -> Self {
        let right_panel_width = if window_size.width >= 900.0 {
            RIGHT_PANEL_WIDTH
        } else {
            0.0
        };
        let toolbar_bounds =
            UiRect::new(0.0, TOP_BAR_HEIGHT, LEFT_TOOLBAR_WIDTH, window_size.height);
        let layer_panel_bounds = UiRect::new(
            (window_size.width - right_panel_width).max(0.0),
            TOP_BAR_HEIGHT,
            right_panel_width,
            (window_size.height - TOP_BAR_HEIGHT).max(0.0),
        );
        let content_x = LEFT_TOOLBAR_WIDTH;
        let content_y = TOP_BAR_HEIGHT;
        let content_width = (window_size.width - LEFT_TOOLBAR_WIDTH - right_panel_width).max(0.0);
        let content_height = (window_size.height - TOP_BAR_HEIGHT).max(0.0);
        let canvas_area = UiRect::new(content_x, content_y, content_width, content_height);
        let screen_bounds = canvas_area.inset(ui_math::UiInsets::all(MIN_CANVAS_MARGIN));

        let canvas_width = (canvas_bounds.max.x - canvas_bounds.min.x).max(1.0);
        let canvas_height = (canvas_bounds.max.y - canvas_bounds.min.y).max(1.0);
        let zoom_x = f64::from(screen_bounds.width.max(1.0)) / canvas_width;
        let zoom_y = f64::from(screen_bounds.height.max(1.0)) / canvas_height;

        Self {
            window_size,
            toolbar_bounds,
            layer_panel_bounds,
            canvas_view: DrawingCanvasView {
                canvas_bounds,
                screen_bounds,
                zoom: zoom_x.min(zoom_y),
                pan: canvas_bounds.min,
            },
        }
    }

    pub fn canvas_area_ratio(self) -> f32 {
        let window_area = (self.window_size.width * self.window_size.height).max(1.0);
        (self.canvas_view.screen_bounds.width * self.canvas_view.screen_bounds.height) / window_area
    }
}

impl Default for DrawingWorkspaceProjection {
    fn default() -> Self {
        Self::canvas_first(
            UiSize::new(1280.0, 720.0),
            CanvasRect::new(
                CanvasCoordinate::new(0.0, 0.0),
                CanvasCoordinate::new(4096.0, 4096.0),
            ),
        )
    }
}
