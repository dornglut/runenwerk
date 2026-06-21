//! Drawing app shell state.

use drawing::{
    BrushId, CanvasCoordinate, CanvasRect, ColorRgba, DrawingCommand, DrawingDocument,
    DrawingDocumentRevision, DrawingInkPreviewStroke, DrawingRatificationReport,
    DrawingTileFormationPolicy, DrawingTransaction, LayerStackEntryId, PaintTarget, StrokeId,
    StrokeSample, drawing_ink_tile_invalidation_for_preview_stroke, ratify_drawing_document,
};
use ui_composition::{ContentLiveness, MountedUnitId};
use ui_input::{PointerEventKind, UiInputEvent};
use ui_math::UiSize;
use ui_render_data::UiFrame;

use crate::app::{
    DrawingCompositionContentState, DrawingCompositionProjection, DrawingCompositionRejection,
    DrawingCompositionRuntime, DrawingImmediateStrokeProjection, DrawingInkRuntimeState,
    DrawingInkSurfaceKind, DrawingInkSurfaceProjection, DrawingPreviewStroke,
    DrawingTabletPanelProjection, DrawingToolControlInputEvent, DrawingToolInputEvent,
    DrawingToolIntent, DrawingToolSession, build_composition_frame,
    build_composition_frame_with_ink_surface_refs_and_strokes, minimal_drawing_document,
};

pub const DEFAULT_DRAWING_BRUSH_ID: BrushId = BrushId::new(1);
pub const DEFAULT_DRAWING_LAYER_ENTRY_ID: LayerStackEntryId = LayerStackEntryId::new(1);
const MAX_PENDING_COMMITTED_STROKE_OVERLAYS: usize = 64;

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
    pub preview_sample_count: usize,
    pub dirty_start_sample_index: usize,
    pub formation_key: String,
}

#[derive(Debug, Clone)]
pub struct RunenwerkDrawApp {
    document: Option<DrawingDocument>,
    composition_runtime: DrawingCompositionRuntime,
    composition_content: DrawingCompositionContentState,
    composition_projection: DrawingCompositionProjection,
    tablet_panel: DrawingTabletPanelProjection,
    active_brush_id: BrushId,
    active_layer_entry_id: LayerStackEntryId,
    tool_session: DrawingToolSession,
    routed_inputs: Vec<DrawingToolInputEvent>,
    preview_stroke: Option<DrawingPreviewStroke>,
    committed_stroke_overlays: Vec<DrawingPreviewStroke>,
    preview_stroke_visible: bool,
    preview_generation: u64,
    preview_dirty_start_sample_index: Option<usize>,
    preview_product_sample_count: usize,
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
        let composition_runtime =
            DrawingCompositionRuntime::builtin().expect("built-in Draw composition should form");
        let composition_content = DrawingCompositionContentState::resolved(&composition_runtime);
        let tablet_panel = DrawingTabletPanelProjection::default();
        let composition_projection = DrawingCompositionProjection::project(
            &composition_runtime,
            &composition_content,
            UiSize::new(1280.0, 720.0),
            default_canvas_bounds(),
            tablet_panel.clone(),
        )
        .expect("built-in Draw composition should project");
        let last_frame = build_composition_frame(&composition_projection);
        Self {
            document: None,
            composition_runtime,
            composition_content,
            composition_projection,
            tablet_panel,
            active_brush_id: DEFAULT_DRAWING_BRUSH_ID,
            active_layer_entry_id: DEFAULT_DRAWING_LAYER_ENTRY_ID,
            tool_session: DrawingToolSession::default(),
            routed_inputs: Vec::new(),
            preview_stroke: None,
            committed_stroke_overlays: Vec::new(),
            preview_stroke_visible: false,
            preview_generation: 0,
            preview_dirty_start_sample_index: None,
            preview_product_sample_count: 0,
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

        self.composition_projection = DrawingCompositionProjection::project(
            &self.composition_runtime,
            &self.composition_content,
            self.composition_projection.window_size,
            document.canvas_bounds,
            self.tablet_panel.clone(),
        )
        .expect("ratified drawing document should project into the active composition");
        self.document = Some(document);
        self.tool_session = DrawingToolSession::default();
        self.routed_inputs.clear();
        self.preview_stroke = None;
        self.committed_stroke_overlays.clear();
        self.preview_stroke_visible = false;
        self.preview_generation = 0;
        self.preview_dirty_start_sample_index = None;
        self.preview_product_sample_count = 0;
        self.pending_preview_job = None;
        self.ink_runtime = DrawingInkRuntimeState::default();
        self.next_preview_stroke_id = 1;
        self.next_preview_sequence = 1;
        self.last_frame = build_composition_frame(&self.composition_projection);
        Ok(())
    }

    pub fn document(&self) -> Option<&DrawingDocument> {
        self.document.as_ref()
    }

    pub fn composition_runtime(&self) -> &DrawingCompositionRuntime {
        &self.composition_runtime
    }

    pub fn composition_content(&self) -> &DrawingCompositionContentState {
        &self.composition_content
    }

    pub fn composition_projection(&self) -> &DrawingCompositionProjection {
        &self.composition_projection
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

    pub fn preview_product_sample_count(&self) -> usize {
        self.preview_product_sample_count
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
        if !preview.active {
            return None;
        }
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

    pub(crate) fn preview_tile_job_can_apply(
        &self,
        stroke_id: StrokeId,
        document_revision: DrawingDocumentRevision,
        preview_sample_count: usize,
    ) -> bool {
        self.preview_stroke_visible
            && self.preview_stroke.as_ref().is_some_and(|preview| {
                preview.active
                    && preview.stroke_id == stroke_id
                    && preview_sample_count > 0
                    && preview_sample_count <= preview.samples.len()
                    && preview_sample_count >= self.preview_product_sample_count
            })
            && self
                .document
                .as_ref()
                .is_some_and(|document| document.revision == document_revision)
    }

    pub(crate) fn record_applied_preview_tile_job(&mut self, preview_sample_count: usize) {
        self.preview_product_sample_count =
            self.preview_product_sample_count.max(preview_sample_count);
    }

    pub(crate) fn clear_preview_after_committed_acceptance(&mut self) {
        self.preview_dirty_start_sample_index = None;
        self.preview_product_sample_count = 0;
        self.pending_preview_job = None;
        self.ink_runtime.clear_preview_products();
        if self.ink_runtime.dirty_tiles().is_empty() {
            self.preview_stroke_visible = false;
            self.committed_stroke_overlays.clear();
        }
        self.rebuild_last_frame();
    }

    pub(crate) fn clear_committed_stroke_overlays_if_clean(&mut self) {
        if self.ink_runtime.dirty_tiles().is_empty() {
            self.committed_stroke_overlays.clear();
        }
        self.rebuild_last_frame();
    }

    pub fn set_window_size(&mut self, size: UiSize) -> Result<(), DrawingCompositionRejection> {
        let canvas_bounds = self
            .document
            .as_ref()
            .map(|document| document.canvas_bounds)
            .unwrap_or(self.composition_projection.canvas_view.canvas_bounds);
        let projection = DrawingCompositionProjection::project(
            &self.composition_runtime,
            &self.composition_content,
            size,
            canvas_bounds,
            self.tablet_panel.clone(),
        )?;
        self.composition_projection = projection;
        self.rebuild_last_frame();
        Ok(())
    }

    pub fn update_tablet_panel(&mut self, tablet_panel: DrawingTabletPanelProjection) {
        if self.tablet_panel != tablet_panel {
            self.tablet_panel = tablet_panel;
            self.rebuild_composition_projection()
                .expect("tablet state should not invalidate composition projection");
            self.rebuild_last_frame();
        }
    }

    pub fn set_composition_content_liveness(
        &mut self,
        mounted_unit: MountedUnitId,
        liveness: ContentLiveness,
    ) -> Result<(), DrawingCompositionRejection> {
        self.composition_content
            .set_liveness(&self.composition_runtime, mounted_unit, liveness)?;
        self.rebuild_composition_projection()?;
        self.rebuild_last_frame();
        Ok(())
    }

    pub fn rebuild_frame(&mut self, size: UiSize) -> &UiFrame {
        let size_changed = (self.composition_projection.window_size.width - size.width).abs()
            > f32::EPSILON
            || (self.composition_projection.window_size.height - size.height).abs() > f32::EPSILON;
        if size_changed {
            self.set_window_size(size)
                .expect("native window size should be a finite composition target");
        }
        &self.last_frame
    }

    pub fn dispatch_input(&mut self, event: &UiInputEvent) -> bool {
        match event {
            UiInputEvent::Pointer(pointer) => {
                let capture_active = self
                    .preview_stroke
                    .as_ref()
                    .is_some_and(|preview| preview.active);
                let routed = DrawingToolInputEvent::from_pointer_with_capture(
                    pointer,
                    self.composition_projection.canvas_view,
                    capture_active,
                );
                let outcome = self.tool_session.handle_input(routed.clone());
                let handled = outcome.handled;
                self.apply_tool_intent(outcome.intent);
                self.rebuild_last_frame();
                self.routed_inputs.push(routed);
                handled
            }
            UiInputEvent::Keyboard(keyboard) => {
                let input = DrawingToolControlInputEvent::from_keyboard(keyboard);
                let outcome = self.tool_session.handle_control_input(input);
                let handled = outcome.handled;
                self.apply_tool_intent(outcome.intent);
                handled
            }
            UiInputEvent::Text(_) => false,
        }
    }

    fn apply_tool_intent(&mut self, intent: DrawingToolIntent) {
        match intent {
            DrawingToolIntent::BeginPreviewStroke { input } => {
                self.preserve_released_preview_overlay();
                let mut preview =
                    DrawingPreviewStroke::new(StrokeId::new(self.next_preview_stroke_id));
                self.next_preview_stroke_id = self.next_preview_stroke_id.saturating_add(1);
                self.preview_stroke_visible = true;
                self.preview_product_sample_count = 0;
                self.ink_runtime.clear_preview_products();
                let dirty_start_sample_index = preview.samples.len();
                let appended = self.append_preview_samples(&mut preview, &input);
                if appended > 0 {
                    self.mark_preview_dirty(dirty_start_sample_index);
                }
                self.preview_stroke = Some(preview);
            }
            DrawingToolIntent::UpdatePreviewStroke { input } => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    if preview.active {
                        let dirty_start_sample_index = preview.samples.len().saturating_sub(1);
                        let appended = self.append_preview_samples(&mut preview, &input);
                        if appended > 0 {
                            self.mark_preview_dirty(dirty_start_sample_index);
                        }
                    }
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolIntent::FinishPreviewStroke { input } => {
                if let Some(mut preview) = self.preview_stroke.take() {
                    let pointer_up = input.pointer_kind == PointerEventKind::Up;
                    if preview.active && pointer_up {
                        let dirty_start_sample_index = preview.samples.len().saturating_sub(1);
                        let appended = self.append_preview_samples(&mut preview, &input);
                        if appended > 0 {
                            self.mark_preview_dirty(dirty_start_sample_index);
                        }
                    }
                    preview.finish();
                    if pointer_up && self.commit_preview_stroke(&preview) {
                        self.freeze_preview_after_commit();
                    }
                    self.preview_stroke = Some(preview);
                }
            }
            DrawingToolIntent::Hover { .. }
            | DrawingToolIntent::Scroll { .. }
            | DrawingToolIntent::Ignore { .. }
            | DrawingToolIntent::ControlInputObserved { .. }
            | DrawingToolIntent::RequestCancel { .. }
            | DrawingToolIntent::RequestRadialMenu { .. } => {}
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

    fn freeze_preview_after_commit(&mut self) {
        self.preview_dirty_start_sample_index = None;
        self.pending_preview_job = None;
    }

    fn preserve_released_preview_overlay(&mut self) {
        let Some(preview) = self.preview_stroke.as_ref() else {
            return;
        };
        if preview.active || !self.preview_stroke_visible || preview.samples.is_empty() {
            return;
        }
        if self
            .committed_stroke_overlays
            .iter()
            .any(|overlay| overlay.stroke_id == preview.stroke_id)
        {
            return;
        }
        self.committed_stroke_overlays.push(preview.clone());
        let overflow = self
            .committed_stroke_overlays
            .len()
            .saturating_sub(MAX_PENDING_COMMITTED_STROKE_OVERLAYS);
        if overflow > 0 {
            self.committed_stroke_overlays.drain(..overflow);
        }
    }

    fn commit_preview_stroke(&mut self, preview: &DrawingPreviewStroke) -> bool {
        if preview.samples.is_empty() {
            return false;
        }
        let Some(document) = self.document.as_ref() else {
            return false;
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
            return false;
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
            true
        } else {
            false
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
        let samples = self.immediate_stroke_samples(stroke)?;
        Some(DrawingImmediateStrokeProjection {
            samples,
            width_px: self.preview_stroke_width_px(),
        })
    }

    fn immediate_stroke_samples<'a>(
        &self,
        stroke: &'a DrawingPreviewStroke,
    ) -> Option<&'a [StrokeSample]> {
        if stroke.samples.is_empty() {
            return None;
        }
        let formed_sample_count = self.preview_product_sample_count.min(stroke.samples.len());
        if formed_sample_count >= stroke.samples.len() {
            return None;
        }
        let continuity_start = formed_sample_count.saturating_sub(1);
        Some(&stroke.samples[continuity_start..])
    }

    fn immediate_stroke_projections(&self) -> Vec<DrawingImmediateStrokeProjection<'_>> {
        let width_px = self.preview_stroke_width_px();
        let mut strokes = self
            .committed_stroke_overlays
            .iter()
            .filter(|stroke| !stroke.samples.is_empty())
            .map(|stroke| DrawingImmediateStrokeProjection {
                samples: stroke.samples.as_slice(),
                width_px,
            })
            .collect::<Vec<_>>();
        if let Some(stroke) = self.immediate_stroke_projection() {
            strokes.push(stroke);
        }
        strokes
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
        (f64::from(brush_width_canvas) * self.composition_projection.canvas_view.zoom) as f32
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
            let immediate_strokes = self.immediate_stroke_projections();
            build_composition_frame_with_ink_surface_refs_and_strokes(
                &self.composition_projection,
                &visible_products,
                &preview_products,
                &immediate_strokes,
            )
        };
        self.last_frame = frame;
    }

    fn rebuild_composition_projection(&mut self) -> Result<(), DrawingCompositionRejection> {
        let canvas_bounds = self
            .document
            .as_ref()
            .map(|document| document.canvas_bounds)
            .unwrap_or(self.composition_projection.canvas_view.canvas_bounds);
        self.composition_projection = DrawingCompositionProjection::project(
            &self.composition_runtime,
            &self.composition_content,
            self.composition_projection.window_size,
            canvas_bounds,
            self.tablet_panel.clone(),
        )?;
        Ok(())
    }
}

fn default_canvas_bounds() -> CanvasRect {
    CanvasRect::new(
        CanvasCoordinate::new(0.0, 0.0),
        CanvasCoordinate::new(4096.0, 4096.0),
    )
}
