//! Pure projection from ratified composition structure into Draw UI geometry.

use std::collections::{BTreeMap, BTreeSet};

use drawing::{CanvasCoordinate, CanvasRect};
use ui_composition::{
    CompositionSnapshot, ContentLiveness, ContentProjectionFallback, MountedUnitId, RegionId,
    RegionKind, SplitAxis,
};
use ui_math::{UiInsets, UiPoint, UiRect, UiSize};

use super::{
    DRAWING_PRESENTATION_TARGET_ID, DrawingCompositionContentState,
    DrawingCompositionDiagnosticCode as Code, DrawingCompositionDiagnosticRecord as Record,
    DrawingCompositionDiagnosticStage as Stage, DrawingCompositionDiagnosticSubject as Subject,
    DrawingCompositionRejection, DrawingCompositionRuntime, DrawingContentRole,
    select_drawing_content_fallback, unavailable_content_diagnostic,
};

const CANVAS_MARGIN: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawingCanvasView {
    pub canvas_bounds: CanvasRect,
    pub screen_bounds: UiRect,
    pub zoom: f64,
    pub pan: CanvasCoordinate,
}

impl DrawingCanvasView {
    pub fn screen_to_canvas(self, screen_position: UiPoint) -> Option<CanvasCoordinate> {
        if self.zoom <= 0.0 || !self.screen_bounds.contains(screen_position) {
            return None;
        }
        self.screen_to_canvas_unbounded(screen_position)
    }

    pub fn screen_to_canvas_unbounded(self, screen_position: UiPoint) -> Option<CanvasCoordinate> {
        if self.zoom <= 0.0 || !screen_position.x.is_finite() || !screen_position.y.is_finite() {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingTabletPanelProjection {
    pub active_backend: String,
    pub active_device: String,
    pub sample_rate_hz: f32,
    pub max_segment_gap_px: f32,
    pub pressure_available: bool,
    pub tilt_available: bool,
    pub dropped_samples: u32,
    pub duplicate_samples: u32,
    pub warning_count: usize,
    pub backend_mode: String,
    pub pressure_scale: f32,
    pub pressure_bias: f32,
    pub cursor_offset: ui_math::UiVector,
}

impl Default for DrawingTabletPanelProjection {
    fn default() -> Self {
        Self {
            active_backend: "winit fallback".to_string(),
            active_device: "mouse or trackpad".to_string(),
            sample_rate_hz: 0.0,
            max_segment_gap_px: 0.0,
            pressure_available: false,
            tilt_available: false,
            dropped_samples: 0,
            duplicate_samples: 0,
            warning_count: 0,
            backend_mode: "AutoOsFirst".to_string(),
            pressure_scale: 1.0,
            pressure_bias: 0.0,
            cursor_offset: ui_math::UiVector::ZERO,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingMountedContentProjection {
    pub mounted_unit_id: MountedUnitId,
    pub role: DrawingContentRole,
    pub bounds: UiRect,
    pub liveness: ContentLiveness,
    pub fallback: ContentProjectionFallback,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawingCompositionProjection {
    pub window_size: UiSize,
    pub top_bar_bounds: UiRect,
    pub tool_rail_bounds: UiRect,
    pub support_panel_bounds: UiRect,
    pub canvas_view: DrawingCanvasView,
    pub tablet_panel: DrawingTabletPanelProjection,
    mounted_content: Vec<DrawingMountedContentProjection>,
    diagnostics: Vec<Record>,
}

impl DrawingCompositionProjection {
    pub fn project(
        runtime: &DrawingCompositionRuntime,
        content: &DrawingCompositionContentState,
        window_size: UiSize,
        canvas_bounds: CanvasRect,
        tablet_panel: DrawingTabletPanelProjection,
    ) -> Result<Self, DrawingCompositionRejection> {
        if !window_size.width.is_finite()
            || !window_size.height.is_finite()
            || window_size.width < 0.0
            || window_size.height < 0.0
        {
            return Err(DrawingCompositionRejection::single(Record::error(
                Code::ProjectionTargetInvalid,
                Stage::Projection,
                Subject::Target(DRAWING_PRESENTATION_TARGET_ID),
                "Project Draw composition into finite, non-negative app-owned target bounds.",
            )));
        }

        let snapshot = runtime.composition().snapshot();
        let root = snapshot
            .roots()
            .iter()
            .find(|root| root.target == DRAWING_PRESENTATION_TARGET_ID && root.primary)
            .ok_or_else(|| {
                DrawingCompositionRejection::single(Record::error(
                    Code::ProjectionRootMissing,
                    Stage::Projection,
                    Subject::Target(DRAWING_PRESENTATION_TARGET_ID),
                    "Provide one primary Draw composition root for the app presentation target.",
                ))
            })?;
        let target_bounds = UiRect::new(0.0, 0.0, window_size.width, window_size.height);
        let mut region_bounds = BTreeMap::new();
        let mut mounted_bounds = BTreeMap::new();
        project_region(
            snapshot,
            root.region,
            target_bounds,
            &mut BTreeSet::new(),
            &mut region_bounds,
            &mut mounted_bounds,
        )?;

        let mut top_bar_bounds = None;
        let mut tool_rail_bounds = None;
        let mut canvas_region_bounds = None;
        let mut support_panel_bounds = None;
        let mut mounted_content = Vec::with_capacity(runtime.extension().mounted_units().len());
        let mut diagnostics = Vec::new();

        for extension in runtime.extension().mounted_units() {
            let unit = snapshot
                .mounted_unit(extension.mounted_unit_id)
                .ok_or_else(|| missing_unit_bounds(extension.mounted_unit_id))?;
            let bounds = mounted_bounds
                .get(&extension.mounted_unit_id)
                .copied()
                .ok_or_else(|| missing_unit_bounds(extension.mounted_unit_id))?;
            let liveness = content.liveness(extension.mounted_unit_id).ok_or_else(|| {
                DrawingCompositionRejection::single(Record::error(
                    Code::ContentUnitUnknown,
                    Stage::Content,
                    Subject::MountedUnit(extension.mounted_unit_id),
                    "Declare exactly one liveness observation for every Draw mounted unit.",
                ))
            })?;
            let fallback = select_drawing_content_fallback(unit, extension, liveness, true, false)?;
            if let Some(diagnostic) =
                unavailable_content_diagnostic(extension.mounted_unit_id, liveness, fallback)
            {
                diagnostics.push(diagnostic);
            }
            match extension.role {
                DrawingContentRole::TopBar => top_bar_bounds = Some(bounds),
                DrawingContentRole::ToolRail => tool_rail_bounds = Some(bounds),
                DrawingContentRole::Canvas => canvas_region_bounds = Some(bounds),
                DrawingContentRole::SupportPanel => support_panel_bounds = Some(bounds),
            }
            mounted_content.push(DrawingMountedContentProjection {
                mounted_unit_id: extension.mounted_unit_id,
                role: extension.role,
                bounds,
                liveness,
                fallback,
            });
        }
        diagnostics.sort();

        let canvas_region_bounds = canvas_region_bounds
            .ok_or_else(|| missing_unit_bounds_for_role(DrawingContentRole::Canvas))?;
        let screen_bounds = canvas_region_bounds.inset(UiInsets::all(CANVAS_MARGIN));
        let canvas_width = (canvas_bounds.max.x - canvas_bounds.min.x).max(1.0);
        let canvas_height = (canvas_bounds.max.y - canvas_bounds.min.y).max(1.0);
        let zoom_x = f64::from(screen_bounds.width.max(1.0)) / canvas_width;
        let zoom_y = f64::from(screen_bounds.height.max(1.0)) / canvas_height;

        Ok(Self {
            window_size,
            top_bar_bounds: top_bar_bounds
                .ok_or_else(|| missing_unit_bounds_for_role(DrawingContentRole::TopBar))?,
            tool_rail_bounds: tool_rail_bounds
                .ok_or_else(|| missing_unit_bounds_for_role(DrawingContentRole::ToolRail))?,
            support_panel_bounds: support_panel_bounds
                .ok_or_else(|| missing_unit_bounds_for_role(DrawingContentRole::SupportPanel))?,
            canvas_view: DrawingCanvasView {
                canvas_bounds,
                screen_bounds,
                zoom: zoom_x.min(zoom_y),
                pan: canvas_bounds.min,
            },
            tablet_panel,
            mounted_content,
            diagnostics,
        })
    }

    pub fn mounted_content(&self) -> &[DrawingMountedContentProjection] {
        &self.mounted_content
    }

    pub fn diagnostics(&self) -> &[Record] {
        &self.diagnostics
    }

    pub fn content_for_role(
        &self,
        role: DrawingContentRole,
    ) -> Option<&DrawingMountedContentProjection> {
        self.mounted_content
            .iter()
            .find(|content| content.role == role)
    }

    pub fn canvas_area_ratio(&self) -> f32 {
        let window_area = (self.window_size.width * self.window_size.height).max(1.0);
        (self.canvas_view.screen_bounds.width * self.canvas_view.screen_bounds.height) / window_area
    }
}

fn project_region(
    snapshot: CompositionSnapshot<'_>,
    region_id: RegionId,
    bounds: UiRect,
    visiting: &mut BTreeSet<RegionId>,
    region_bounds: &mut BTreeMap<RegionId, UiRect>,
    mounted_bounds: &mut BTreeMap<MountedUnitId, UiRect>,
) -> Result<(), DrawingCompositionRejection> {
    if !visiting.insert(region_id) || region_bounds.insert(region_id, bounds).is_some() {
        return Err(DrawingCompositionRejection::single(Record::error(
            Code::ProjectionRegionCycle,
            Stage::Projection,
            Subject::Region(region_id),
            "Project each Draw region exactly once from an acyclic composition graph.",
        )));
    }
    let region = snapshot.region(region_id).ok_or_else(|| {
        DrawingCompositionRejection::single(Record::error(
            Code::ProjectionRegionMissing,
            Stage::Projection,
            Subject::Region(region_id),
            "Reference only regions present in the active Draw composition.",
        ))
    })?;
    match &region.kind {
        RegionKind::Split {
            axis,
            fraction,
            first,
            second,
        } => {
            let ratio = f32::from(u16::from(*fraction)) / 10_000.0;
            let (first_bounds, second_bounds) = split_bounds(bounds, *axis, ratio);
            project_region(
                snapshot,
                *first,
                first_bounds,
                visiting,
                region_bounds,
                mounted_bounds,
            )?;
            project_region(
                snapshot,
                *second,
                second_bounds,
                visiting,
                region_bounds,
                mounted_bounds,
            )?;
        }
        RegionKind::Stack { ordered_units, .. } => {
            for mounted_unit in ordered_units {
                mounted_bounds.insert(*mounted_unit, bounds);
            }
        }
        RegionKind::Overlay {
            base,
            ordered_overlays,
        } => {
            project_region(
                snapshot,
                *base,
                bounds,
                visiting,
                region_bounds,
                mounted_bounds,
            )?;
            for overlay in ordered_overlays {
                project_region(
                    snapshot,
                    *overlay,
                    bounds,
                    visiting,
                    region_bounds,
                    mounted_bounds,
                )?;
            }
        }
        RegionKind::MountPoint { mounted_unit } => {
            mounted_bounds.insert(*mounted_unit, bounds);
        }
    }
    visiting.remove(&region_id);
    Ok(())
}

fn split_bounds(bounds: UiRect, axis: SplitAxis, ratio: f32) -> (UiRect, UiRect) {
    match axis {
        SplitAxis::Horizontal => {
            let first_width = bounds.width * ratio;
            (
                UiRect::new(bounds.x, bounds.y, first_width, bounds.height),
                UiRect::new(
                    bounds.x + first_width,
                    bounds.y,
                    (bounds.width - first_width).max(0.0),
                    bounds.height,
                ),
            )
        }
        SplitAxis::Vertical => {
            let first_height = bounds.height * ratio;
            (
                UiRect::new(bounds.x, bounds.y, bounds.width, first_height),
                UiRect::new(
                    bounds.x,
                    bounds.y + first_height,
                    bounds.width,
                    (bounds.height - first_height).max(0.0),
                ),
            )
        }
    }
}

fn missing_unit_bounds(mounted_unit: MountedUnitId) -> DrawingCompositionRejection {
    DrawingCompositionRejection::single(Record::error(
        Code::ProjectionMountedUnitBoundsMissing,
        Stage::Projection,
        Subject::MountedUnit(mounted_unit),
        "Place every Draw extension mounted unit in the active composition region graph.",
    ))
}

fn missing_unit_bounds_for_role(role: DrawingContentRole) -> DrawingCompositionRejection {
    DrawingCompositionRejection::single(Record::error(
        Code::ProjectionMountedUnitBoundsMissing,
        Stage::Projection,
        Subject::General(format!("drawing_content_role::{role:?}")),
        "Bind every required Draw content role to a projected mounted unit.",
    ))
}
