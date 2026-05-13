//! Drawing app shell state.

use drawing::{
    BrushId, ColorRgba, DrawingCommand, DrawingDocument, DrawingInkPreviewStroke,
    DrawingRatificationReport, DrawingTileFormationPolicy, DrawingTransaction, LayerStackEntryId,
    PaintTarget, StrokeId, drawing_ink_tile_invalidation_for_preview_stroke,
    form_drawing_ink_preview_tiles_for_ids, ratify_drawing_document,
};
use ui_input::{PointerEventKind, UiInputEvent};
use ui_math::UiSize;
use ui_render_data::UiFrame;

use crate::app::{
    DrawingInkRuntimeState, DrawingPreviewStroke, DrawingToolInputEvent, DrawingToolRouteKind,
    DrawingWorkspaceProjection, build_workspace_frame, build_workspace_frame_with_ink,
    minimal_drawing_document,
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
    ink_runtime: DrawingInkRuntimeState,
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
            ink_runtime: DrawingInkRuntimeState::default(),
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
        self.ink_runtime = DrawingInkRuntimeState::default();
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

    pub fn ink_runtime(&self) -> &DrawingInkRuntimeState {
        &self.ink_runtime
    }

    pub fn ink_runtime_mut(&mut self) -> &mut DrawingInkRuntimeState {
        &mut self.ink_runtime
    }

    pub fn last_frame(&self) -> &UiFrame {
        &self.last_frame
    }

    pub(crate) fn rebuild_visible_frame(&mut self) {
        self.rebuild_last_frame();
    }

    pub fn set_window_size(&mut self, size: UiSize) {
        let canvas_bounds = self
            .document
            .as_ref()
            .map(|document| document.canvas_bounds)
            .unwrap_or(self.workspace.canvas_view.canvas_bounds);
        self.workspace = DrawingWorkspaceProjection::canvas_first(size, canvas_bounds);
        self.rebuild_last_frame();
    }

    pub fn rebuild_frame(&mut self, size: UiSize) -> &UiFrame {
        let size_changed = (self.workspace.window_size.width - size.width).abs() > f32::EPSILON
            || (self.workspace.window_size.height - size.height).abs() > f32::EPSILON;
        if size_changed {
            self.set_window_size(size);
        }
        &self.last_frame
    }

    pub fn dispatch_input(&mut self, event: &UiInputEvent) -> bool {
        let UiInputEvent::Pointer(pointer) = event else {
            return false;
        };
        let routed = DrawingToolInputEvent::from_pointer(pointer, self.workspace.canvas_view);
        let handled = routed.route_kind != DrawingToolRouteKind::Ignored;
        self.apply_routed_input(routed.clone());
        self.rebuild_last_frame();
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
                self.rebuild_preview_products(&preview);
                self.preview_stroke = Some(preview);
            }
            DrawingToolRouteKind::UpdatePreviewStroke => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    if preview.active {
                        self.append_preview_sample(&mut preview, routed);
                        self.rebuild_preview_products(&preview);
                    }
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolRouteKind::EndPreviewStroke => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    let pointer_up = routed.pointer_kind == PointerEventKind::Up;
                    if preview.active && pointer_up {
                        self.append_preview_sample(&mut preview, routed);
                        self.rebuild_preview_products(&preview);
                    }
                    preview.finish();
                    if pointer_up {
                        self.commit_preview_stroke(&preview);
                    }
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolRouteKind::Hover
            | DrawingToolRouteKind::Scroll
            | DrawingToolRouteKind::Ignored => {}
        }
    }

    fn rebuild_preview_products(&mut self, preview: &DrawingPreviewStroke) {
        let Some(document) = self.document.as_ref() else {
            self.ink_runtime.clear_preview_products();
            return;
        };
        let policy = DrawingTileFormationPolicy::default();
        let preview_stroke = self.preview_ink_stroke(preview, document.revision);
        let invalidation =
            drawing_ink_tile_invalidation_for_preview_stroke(document, &preview_stroke, policy);
        if !invalidation.is_accepted() {
            self.ink_runtime
                .record_preview_failure(invalidation.diagnostics);
            return;
        }
        if invalidation.tile_ids.is_empty() {
            self.ink_runtime.clear_preview_products();
            return;
        }

        let mut products = Vec::new();
        let mut diagnostics = invalidation.diagnostics;
        let batch_size = policy.max_affected_tiles.max(1);
        for tile_batch in invalidation.tile_ids.chunks(batch_size) {
            let formation = form_drawing_ink_preview_tiles_for_ids(
                document,
                &preview_stroke,
                tile_batch.iter().copied(),
                policy,
            );
            diagnostics.extend(formation.diagnostics.clone());
            if !formation.is_accepted() {
                self.ink_runtime.record_preview_failure(diagnostics);
                return;
            }
            products.extend(formation.products);
        }
        self.ink_runtime
            .record_preview_products(products, diagnostics);
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

    fn commit_preview_stroke(&mut self, preview: &DrawingPreviewStroke) {
        if preview.samples.is_empty() {
            return;
        }
        let Some(document) = self.document.as_ref() else {
            return;
        };
        let policy = DrawingTileFormationPolicy::default();
        let preview_stroke = self.preview_ink_stroke(preview, document.revision);
        let invalidation =
            drawing_ink_tile_invalidation_for_preview_stroke(document, &preview_stroke, policy);
        let dirty_tiles = if invalidation.is_accepted() {
            invalidation.tile_ids
        } else {
            Vec::new()
        };
        let Some(document) = self.document.as_mut() else {
            return;
        };
        let mut commands = Vec::with_capacity(preview.samples.len() + 2);
        commands.push(DrawingCommand::BeginStroke {
            stroke_id: preview.stroke_id,
            target: PaintTarget::StackEntry(self.active_layer_entry_id),
            brush_id: self.active_brush_id,
            color: ColorRgba::new(0.04, 0.035, 0.03, 1.0),
        });
        commands.extend(preview.samples.iter().copied().map(|sample| {
            DrawingCommand::AppendStrokeSample {
                stroke_id: preview.stroke_id,
                sample,
            }
        }));
        commands.push(DrawingCommand::CommitStroke {
            stroke_id: preview.stroke_id,
        });
        let transaction = DrawingTransaction::new("commit ink stroke", commands);
        if transaction.apply_to(document).is_ok() {
            self.ink_runtime.invalidate_after_document_change();
            self.ink_runtime.mark_dirty_tiles(dirty_tiles);
        }
    }

    fn preview_ink_stroke(
        &self,
        preview: &DrawingPreviewStroke,
        source_revision: drawing::DrawingDocumentRevision,
    ) -> DrawingInkPreviewStroke {
        DrawingInkPreviewStroke::new(
            preview.stroke_id,
            PaintTarget::StackEntry(self.active_layer_entry_id),
            self.active_brush_id,
            ColorRgba::new(0.04, 0.035, 0.03, 1.0),
            source_revision,
        )
        .with_samples(preview.samples.iter().copied())
    }

    fn rebuild_last_frame(&mut self) {
        let visible_products = self
            .ink_runtime
            .visible_products()
            .cloned()
            .collect::<Vec<_>>();
        let preview_products = self.ink_runtime.preview_products().to_vec();
        self.last_frame =
            build_workspace_frame_with_ink(&self.workspace, &visible_products, &preview_products);
    }
}
