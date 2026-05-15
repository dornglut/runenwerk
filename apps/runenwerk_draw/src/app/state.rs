//! Drawing app shell state.

use drawing::{
    BrushId, ColorRgba, DrawingCommand, DrawingDocument, DrawingDocumentRevision,
    DrawingInkPreviewStroke, DrawingRatificationReport, DrawingTileFormationPolicy,
    DrawingTransaction, LayerStackEntryId, PaintTarget, StrokeId,
    drawing_ink_tile_invalidation_for_preview_stroke, ratify_drawing_document,
};
use ui_input::{PointerEventKind, UiInputEvent};
use ui_math::UiSize;
use ui_render_data::UiFrame;

use crate::app::{
    DrawingImmediateStrokeProjection, DrawingInkRuntimeState, DrawingInkSurfaceKind,
    DrawingInkSurfaceProjection, DrawingPreviewStroke, DrawingTabletPanelProjection,
    DrawingToolInputEvent, DrawingToolRouteKind, DrawingWorkspaceProjection, build_workspace_frame,
    build_workspace_frame_with_ink_surface_refs_and_stroke, minimal_drawing_document,
};

pub const DEFAULT_DRAWING_BRUSH_ID: BrushId = BrushId::new(1);
pub const DEFAULT_DRAWING_LAYER_ENTRY_ID: LayerStackEntryId = LayerStackEntryId::new(1);

#[derive(Debug, Clone)]
pub(crate) struct DrawingPreviewTileJobSnapshot {
    pub document: DrawingDocument,
    pub preview_stroke: DrawingInkPreviewStroke,
    pub dirty_preview_stroke: DrawingInkPreviewStroke,
    pub dirty_start_sample_index: usize,
    pub preview_generation: u64,
    pub policy: DrawingTileFormationPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrawingPreviewTileJobTracker {
    pub stroke_id: StrokeId,
    pub document_revision: DrawingDocumentRevision,
    pub preview_generation: u64,
    pub dirty_start_sample_index: usize,
    pub formation_key: String,
}

#[derive(Debug, Clone)]
pub struct RunenwerkDrawApp {
    document: Option<DrawingDocument>,
    workspace: DrawingWorkspaceProjection,
    active_brush_id: BrushId,
    active_layer_entry_id: LayerStackEntryId,
    routed_inputs: Vec<DrawingToolInputEvent>,
    preview_stroke: Option<DrawingPreviewStroke>,
    preview_stroke_visible: bool,
    preview_generation: u64,
    preview_dirty_start_sample_index: Option<usize>,
    pending_preview_job: Option<DrawingPreviewTileJobTracker>,
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
            preview_stroke_visible: false,
            preview_generation: 0,
            preview_dirty_start_sample_index: None,
            pending_preview_job: None,
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

        let tablet_panel = self.workspace.tablet_panel.clone();
        self.workspace = DrawingWorkspaceProjection::canvas_first(
            self.workspace.window_size,
            document.canvas_bounds,
        )
        .with_tablet_panel(tablet_panel);
        self.document = Some(document);
        self.routed_inputs.clear();
        self.preview_stroke = None;
        self.preview_stroke_visible = false;
        self.preview_generation = 0;
        self.preview_dirty_start_sample_index = None;
        self.pending_preview_job = None;
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

    pub fn preview_generation(&self) -> u64 {
        self.preview_generation
    }

    pub fn preview_dirty_start_sample_index(&self) -> Option<usize> {
        self.preview_dirty_start_sample_index
    }

    pub fn pending_preview_tile_job(&self) -> Option<&DrawingPreviewTileJobTracker> {
        self.pending_preview_job.as_ref()
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

    pub(crate) fn next_preview_tile_job_snapshot(&self) -> Option<DrawingPreviewTileJobSnapshot> {
        if self.pending_preview_job.is_some() || !self.preview_stroke_visible {
            return None;
        }
        let dirty_start_sample_index = self.preview_dirty_start_sample_index?;
        let preview = self.preview_stroke.as_ref()?;
        if preview.samples.is_empty() {
            return None;
        }
        let document = self.document.as_ref()?.clone();
        let document_revision = document.revision;
        let dirty_start_sample_index = dirty_start_sample_index.min(preview.samples.len() - 1);
        let policy = DrawingTileFormationPolicy::default();
        Some(DrawingPreviewTileJobSnapshot {
            document,
            preview_stroke: self.preview_ink_stroke(preview, document_revision),
            dirty_preview_stroke: self.preview_ink_stroke_for_samples(
                preview.stroke_id,
                preview.samples[dirty_start_sample_index..].iter().copied(),
                document_revision,
            ),
            dirty_start_sample_index,
            preview_generation: self.preview_generation,
            policy,
        })
    }

    pub(crate) fn record_pending_preview_tile_job(
        &mut self,
        tracker: DrawingPreviewTileJobTracker,
    ) {
        self.pending_preview_job = Some(tracker);
        self.preview_dirty_start_sample_index = None;
    }

    pub(crate) fn clear_pending_preview_tile_job(&mut self, formation_key: &str) {
        if self
            .pending_preview_job
            .as_ref()
            .is_some_and(|pending| pending.formation_key == formation_key)
        {
            self.pending_preview_job = None;
        }
    }

    pub(crate) fn clear_pending_preview_tile_job_generation(&mut self, preview_generation: u64) {
        if self
            .pending_preview_job
            .as_ref()
            .is_some_and(|pending| pending.preview_generation == preview_generation)
        {
            self.pending_preview_job = None;
        }
    }

    pub(crate) fn preview_tile_job_is_current(
        &self,
        stroke_id: StrokeId,
        document_revision: DrawingDocumentRevision,
        preview_generation: u64,
    ) -> bool {
        self.preview_stroke_visible
            && self
                .preview_stroke
                .as_ref()
                .is_some_and(|preview| preview.stroke_id == stroke_id)
            && self
                .document
                .as_ref()
                .is_some_and(|document| document.revision == document_revision)
            && self.preview_generation == preview_generation
    }

    pub(crate) fn clear_preview_after_committed_acceptance(&mut self) {
        self.preview_stroke_visible = false;
        self.preview_dirty_start_sample_index = None;
        self.pending_preview_job = None;
        self.ink_runtime.clear_preview_products();
        self.rebuild_last_frame();
    }

    pub fn set_window_size(&mut self, size: UiSize) {
        let canvas_bounds = self
            .document
            .as_ref()
            .map(|document| document.canvas_bounds)
            .unwrap_or(self.workspace.canvas_view.canvas_bounds);
        let tablet_panel = self.workspace.tablet_panel.clone();
        self.workspace = DrawingWorkspaceProjection::canvas_first(size, canvas_bounds)
            .with_tablet_panel(tablet_panel);
        self.rebuild_last_frame();
    }

    pub fn update_tablet_panel(&mut self, tablet_panel: DrawingTabletPanelProjection) {
        if self.workspace.tablet_panel != tablet_panel {
            self.workspace.tablet_panel = tablet_panel;
            self.rebuild_last_frame();
        }
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
        let capture_active = self
            .preview_stroke
            .as_ref()
            .is_some_and(|preview| preview.active);
        let routed = DrawingToolInputEvent::from_pointer_with_capture(
            pointer,
            self.workspace.canvas_view,
            capture_active,
        );
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
                self.preview_stroke_visible = true;
                self.ink_runtime.clear_preview_products();
                let dirty_start_sample_index = preview.samples.len();
                let appended = self.append_preview_samples(&mut preview, &routed);
                if appended > 0 {
                    self.mark_preview_dirty(dirty_start_sample_index);
                }
                self.preview_stroke = Some(preview);
            }
            DrawingToolRouteKind::UpdatePreviewStroke => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    if preview.active {
                        let dirty_start_sample_index = preview.samples.len().saturating_sub(1);
                        let appended = self.append_preview_samples(&mut preview, &routed);
                        if appended > 0 {
                            self.mark_preview_dirty(dirty_start_sample_index);
                        }
                    }
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolRouteKind::EndPreviewStroke => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    let pointer_up = routed.pointer_kind == PointerEventKind::Up;
                    if preview.active && pointer_up {
                        let dirty_start_sample_index = preview.samples.len().saturating_sub(1);
                        let appended = self.append_preview_samples(&mut preview, &routed);
                        if appended > 0 {
                            self.mark_preview_dirty(dirty_start_sample_index);
                        }
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

    fn append_preview_samples(
        &mut self,
        preview: &mut DrawingPreviewStroke,
        routed: &DrawingToolInputEvent,
    ) -> usize {
        let samples = routed.to_stroke_samples(&mut self.next_preview_sequence);
        let appended = samples.len();
        for sample in samples {
            preview.append_sample(sample);
        }
        appended
    }

    fn mark_preview_dirty(&mut self, dirty_start_sample_index: usize) {
        self.preview_generation = self.preview_generation.saturating_add(1).max(1);
        self.preview_dirty_start_sample_index = Some(
            self.preview_dirty_start_sample_index
                .map(|existing| existing.min(dirty_start_sample_index))
                .unwrap_or(dirty_start_sample_index),
        );
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
        self.preview_ink_stroke_for_samples(
            preview.stroke_id,
            preview.samples.iter().copied(),
            source_revision,
        )
    }

    fn preview_ink_stroke_for_samples(
        &self,
        stroke_id: StrokeId,
        samples: impl IntoIterator<Item = drawing::StrokeSample>,
        source_revision: drawing::DrawingDocumentRevision,
    ) -> DrawingInkPreviewStroke {
        DrawingInkPreviewStroke::new(
            stroke_id,
            PaintTarget::StackEntry(self.active_layer_entry_id),
            self.active_brush_id,
            ColorRgba::new(0.04, 0.035, 0.03, 1.0),
            source_revision,
        )
        .with_samples(samples)
    }

    fn immediate_stroke_projection(&self) -> Option<DrawingImmediateStrokeProjection<'_>> {
        if !self.preview_stroke_visible {
            return None;
        }
        let stroke = self.preview_stroke.as_ref()?;
        if stroke.samples.is_empty() {
            return None;
        }
        Some(DrawingImmediateStrokeProjection {
            stroke,
            width_px: self.preview_stroke_width_px(),
        })
    }

    fn preview_stroke_width_px(&self) -> f32 {
        let brush_width_canvas = self
            .document
            .as_ref()
            .and_then(|document| {
                document
                    .brushes
                    .iter()
                    .find(|brush| brush.brush_id == self.active_brush_id)
                    .map(|brush| brush.ink.size.max)
            })
            .unwrap_or(4.0);
        (f64::from(brush_width_canvas) * self.workspace.canvas_view.zoom) as f32
    }

    fn rebuild_last_frame(&mut self) {
        let frame = {
            let visible_products = self
                .ink_runtime
                .visible_products()
                .map(|product| DrawingInkSurfaceProjection {
                    product,
                    surface_kind: self
                        .ink_runtime
                        .visible_surface_kind_for(DrawingInkSurfaceKind::Committed, product),
                })
                .collect::<Vec<_>>();
            let preview_products = self
                .ink_runtime
                .preview_products()
                .iter()
                .map(|product| DrawingInkSurfaceProjection {
                    product,
                    surface_kind: self
                        .ink_runtime
                        .visible_surface_kind_for(DrawingInkSurfaceKind::Preview, product),
                })
                .collect::<Vec<_>>();
            let immediate_stroke = self.immediate_stroke_projection();
            build_workspace_frame_with_ink_surface_refs_and_stroke(
                &self.workspace,
                &visible_products,
                &preview_products,
                immediate_stroke,
            )
        };
        self.last_frame = frame;
    }
}
