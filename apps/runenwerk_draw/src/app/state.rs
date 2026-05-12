//! Drawing app shell state.

use drawing::{
    BrushId, DrawingDocument, DrawingRatificationReport, LayerStackEntryId, StrokeId,
    ratify_drawing_document,
};
use ui_input::{PointerEventKind, UiInputEvent};
use ui_math::UiSize;
use ui_render_data::UiFrame;

use crate::app::{
    DrawingPreviewStroke, DrawingToolInputEvent, DrawingToolRouteKind, DrawingWorkspaceProjection,
    build_workspace_frame, minimal_drawing_document,
};

pub const DEFAULT_DRAWING_BRUSH_ID: BrushId = BrushId::new(1);
pub const DEFAULT_DRAWING_LAYER_ENTRY_ID: LayerStackEntryId = LayerStackEntryId::new(1);

#[derive(Debug, Clone)]
pub struct RunenwerkDrawApp {
    document: Option<DrawingDocument>,
    workspace: DrawingWorkspaceProjection,
    active_brush_id: BrushId,
    active_layer_entry_id: LayerStackEntryId,
    routed_inputs: Vec<DrawingToolInputEvent>,
    preview_stroke: Option<DrawingPreviewStroke>,
    next_preview_stroke_id: u64,
    next_preview_sequence: u64,
    last_frame: UiFrame,
}

impl Default for RunenwerkDrawApp {
    fn default() -> Self {
        Self::new()
    }
}

impl RunenwerkDrawApp {
    pub fn new() -> Self {
        let mut app = Self::empty();
        app.open_document(minimal_drawing_document())
            .expect("minimal drawing document should ratify");
        app
    }

    pub fn empty() -> Self {
        let workspace = DrawingWorkspaceProjection::default();
        let last_frame = build_workspace_frame(&workspace);
        Self {
            document: None,
            workspace,
            active_brush_id: DEFAULT_DRAWING_BRUSH_ID,
            active_layer_entry_id: DEFAULT_DRAWING_LAYER_ENTRY_ID,
            routed_inputs: Vec::new(),
            preview_stroke: None,
            next_preview_stroke_id: 1,
            next_preview_sequence: 1,
            last_frame,
        }
    }

    pub fn open_document(
        &mut self,
        document: DrawingDocument,
    ) -> Result<(), DrawingRatificationReport> {
        let report = ratify_drawing_document(&document);
        if !report.is_accepted() {
            return Err(report);
        }

        self.workspace = DrawingWorkspaceProjection::canvas_first(
            self.workspace.window_size,
            document.canvas_bounds,
        );
        self.document = Some(document);
        self.routed_inputs.clear();
        self.preview_stroke = None;
        self.next_preview_stroke_id = 1;
        self.next_preview_sequence = 1;
        self.last_frame = build_workspace_frame(&self.workspace);
        Ok(())
    }

    pub fn document(&self) -> Option<&DrawingDocument> {
        self.document.as_ref()
    }

    pub fn workspace(&self) -> &DrawingWorkspaceProjection {
        &self.workspace
    }

    pub fn active_brush_id(&self) -> BrushId {
        self.active_brush_id
    }

    pub fn active_layer_entry_id(&self) -> LayerStackEntryId {
        self.active_layer_entry_id
    }

    pub fn routed_inputs(&self) -> &[DrawingToolInputEvent] {
        &self.routed_inputs
    }

    pub fn preview_stroke(&self) -> Option<&DrawingPreviewStroke> {
        self.preview_stroke.as_ref()
    }

    pub fn last_frame(&self) -> &UiFrame {
        &self.last_frame
    }

    pub fn set_window_size(&mut self, size: UiSize) {
        let canvas_bounds = self
            .document
            .as_ref()
            .map(|document| document.canvas_bounds)
            .unwrap_or(self.workspace.canvas_view.canvas_bounds);
        self.workspace = DrawingWorkspaceProjection::canvas_first(size, canvas_bounds);
        self.last_frame = build_workspace_frame(&self.workspace);
    }

    pub fn rebuild_frame(&mut self, size: UiSize) -> &UiFrame {
        self.set_window_size(size);
        &self.last_frame
    }

    pub fn dispatch_input(&mut self, event: &UiInputEvent) -> bool {
        let UiInputEvent::Pointer(pointer) = event else {
            return false;
        };
        let routed = DrawingToolInputEvent::from_pointer(pointer, self.workspace.canvas_view);
        let handled = routed.route_kind != DrawingToolRouteKind::Ignored;
        self.apply_routed_input(routed.clone());
        self.routed_inputs.push(routed);
        handled
    }

    fn apply_routed_input(&mut self, routed: DrawingToolInputEvent) {
        match routed.route_kind {
            DrawingToolRouteKind::BeginPreviewStroke => {
                let mut preview =
                    DrawingPreviewStroke::new(StrokeId::new(self.next_preview_stroke_id));
                self.next_preview_stroke_id = self.next_preview_stroke_id.saturating_add(1);
                self.append_preview_sample(&mut preview, routed);
                self.preview_stroke = Some(preview);
            }
            DrawingToolRouteKind::UpdatePreviewStroke => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    if preview.active {
                        self.append_preview_sample(&mut preview, routed);
                    }
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolRouteKind::EndPreviewStroke => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    if preview.active && routed.pointer_kind == PointerEventKind::Up {
                        self.append_preview_sample(&mut preview, routed);
                    }
                    preview.finish();
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolRouteKind::Hover
            | DrawingToolRouteKind::Scroll
            | DrawingToolRouteKind::Ignored => {}
        }
    }

    fn append_preview_sample(
        &mut self,
        preview: &mut DrawingPreviewStroke,
        routed: DrawingToolInputEvent,
    ) {
        if let Some(sample) = routed.to_stroke_sample(self.next_preview_sequence) {
            self.next_preview_sequence = self.next_preview_sequence.saturating_add(1);
            preview.append_sample(sample);
        }
    }
}
