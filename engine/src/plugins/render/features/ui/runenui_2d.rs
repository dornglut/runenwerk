//! Fail-closed projection from one immutable RunenUI paint publication to RunenRender 2D.
//!
//! This is source-adaptation plumbing, not a second renderer or compositor authority.
//! RunenUI resource references are retained only in one adapter-owned identity map;
//! they never become RunenRender resource IDs. The complete SurfacePublication remains owned by
//! the UI integration until exact displayed-input promotion.

use std::collections::{HashMap, HashSet};

use runen_render::composition_2d::{
    Render2dAffineTransform, Render2dBrush, Render2dClip, Render2dColorRgba8, Render2dComposition,
    Render2dCompositionError, Render2dCornerRadii, Render2dEntry, Render2dFillRule,
    Render2dFontBinding, Render2dGeometryError, Render2dGlyph, Render2dItem, Render2dOpacity,
    Render2dPath, Render2dPathCommand, Render2dPoint, Render2dPrimitive, Render2dRect,
    Render2dResourceBinding, Render2dResourceBindingError, Render2dResourceBindings,
    Render2dResourceError, Render2dResourceId, Render2dResourceValue, Render2dShape,
    Render2dShapedTextPrimitive, Render2dShapedTextResource, Render2dStrokeCap, Render2dStrokeJoin,
    Render2dStrokeStyle,
};
use runenui_core::{
    Brush, LogicalRect, LogicalTransform, PaintPrimitive, PathFillRule, PathVerb, ResourceRef,
    SceneLayer, SceneOpacity, SceneShape, StrokeCap, StrokeJoin, StrokeStyle,
};
use runenui_runtime::{PaintPublication, PaintSceneItem};

/// Retained, monotonic mapping of opaque source resources into RunenRender identities.
///
/// This map survives F2 executor CPU-cache generations. It retains currently used
/// source references and mints monotonically increasing semantic IDs; no ID is
/// recycled, even while older GPU submissions remain in flight.
#[derive(Clone, Debug, Default)]
pub struct RunenUi2dResourceIdentityMap {
    last_issued: u64,
    by_source: HashMap<ResourceRef, Render2dResourceId>,
}

impl RunenUi2dResourceIdentityMap {
    #[must_use]
    pub fn retained_count(&self) -> usize {
        self.by_source.len()
    }

    /// Release an obsolete source reference; its semantic ID is permanently spent.
    /// GPU work previously prepared from this source owns its work independently.
    pub fn retire(&mut self, source: &ResourceRef) -> bool {
        self.by_source.remove(source).is_some()
    }

    /// Prunes obsolete opaque source references. Prepared F2 GPU operations own
    /// their resource handles; no accepted submission depends on this CPU map.
    /// Monotonic semantic IDs are never reissued across executor generations.
    pub fn retain_exact_live(&mut self, live: &HashSet<ResourceRef>) -> usize {
        let before = self.by_source.len();
        self.by_source.retain(|source, _| live.contains(source));
        before - self.by_source.len()
    }

    /// Bound retained source-reference history relative to all currently
    /// published sources, even if GPU submissions never reach a globally idle
    /// point. At most 256 obsolete references remain between reclamations;
    /// keeping a minimum batch avoids resetting derived F2 fields every frame.
    pub fn reclaim_obsolete_over_budget(&mut self, live: &HashSet<ResourceRef>) -> usize {
        const RECLAIM_WATERMARK: usize = 256;
        const MINIMUM_STALE_TO_RECLAIM: usize = 128;
        if self.retained_count() <= RECLAIM_WATERMARK {
            return 0;
        }
        let stale = self
            .by_source
            .keys()
            .filter(|source| !live.contains(*source))
            .count();
        if stale < MINIMUM_STALE_TO_RECLAIM {
            return 0;
        }
        self.retain_exact_live(live)
    }

    fn resolve_or_allocate(
        &mut self,
        source: &ResourceRef,
    ) -> Result<Render2dResourceId, RunenUi2dProjectionError> {
        if let Some(id) = self.by_source.get(source) {
            return Ok(*id);
        }
        let raw = self
            .last_issued
            .checked_add(1)
            .ok_or(RunenUi2dProjectionError::ResourceIdsExhausted)?;
        let id =
            Render2dResourceId::new(raw).ok_or(RunenUi2dProjectionError::ResourceIdsExhausted)?;
        self.last_issued = raw;
        self.by_source.insert(source.clone(), id);
        Ok(id)
    }
}

/// Exact renderer-semantic content for one independently retained RunenUI publication.
#[derive(Clone, Debug)]
pub struct RunenUi2dSemanticProjection {
    composition: Render2dComposition,
    bindings: Render2dResourceBindings,
}

impl RunenUi2dSemanticProjection {
    #[must_use]
    pub const fn composition(&self) -> &Render2dComposition {
        &self.composition
    }

    #[must_use]
    pub const fn bindings(&self) -> &Render2dResourceBindings {
        &self.bindings
    }
}

/// Rejected paint facts are explicit, never silently omitted or rasterized by Runenwerk.
#[derive(Debug, thiserror::Error)]
pub enum RunenUi2dProjectionError {
    #[error("RunenUI paint groups are outside the bounded Counter cut")]
    Groups,
    #[error("RunenUI root contains a group or omits a paint item")]
    RootStructure,
    #[error("RunenUI root references missing paint item {0}")]
    MissingItem(usize),
    #[error("RunenUI root repeats paint item {0}")]
    RepeatedItem(usize),
    #[error("RunenUI paint item uses an unsupported primitive")]
    Primitive,
    #[error("RunenUI paint brush is outside the reviewed solid-brush subset")]
    Brush,
    /// Retained for downstream exhaustive matches. The accepted direct-root
    /// conjunctive-clip subset now projects clips rather than emitting this error.
    #[error("RunenUI paint item has unsupported clips")]
    Clips,
    /// Retained for downstream exhaustive matches. Validated per-item opacity
    /// is now projected; atomic group opacity remains outside the flat subset.
    #[error("RunenUI paint item has unsupported opacity")]
    Opacity,
    #[error("RunenUI paint item has unsupported scene layer")]
    Layer,
    #[error("RunenUI paint item has a non-translation transform")]
    Transform,
    #[error("RunenUI shaped text has no matching immutable resource")]
    ShapedTextMissing,
    #[error("RunenRender local semantic resource identity space exhausted")]
    ResourceIdsExhausted,
    #[error(transparent)]
    Geometry(#[from] Render2dGeometryError),
    #[error(transparent)]
    Resource(#[from] Render2dResourceError),
    #[error(transparent)]
    Composition(#[from] Render2dCompositionError),
    #[error(transparent)]
    Binding(#[from] Render2dResourceBindingError),
}

/// Projects direct-root, solid Fill/Stroke and accepted U5 shaped text
/// with per-item opacity and ordered conjunctive clips in RunenUI root order.
///
/// Resource identities are allocated once per immutable source reference and reused
/// across publications through the caller-retained identity map. Duplicate items
/// in one scene bind that semantic resource only once. No font lookup, reshaping,
/// rasterization, or sorting is performed here.
///
/// # Errors
///
/// Rejects every paint/group/transform fact outside the bounded
/// supported subset, missing shaped text, or invalid RunenRender semantic values.
pub fn project_runenui_paint_to_2d(
    publication: &PaintPublication,
    identities: &mut RunenUi2dResourceIdentityMap,
) -> Result<RunenUi2dSemanticProjection, RunenUi2dProjectionError> {
    let scene = publication.scene();
    if !scene.groups().is_empty() {
        return Err(RunenUi2dProjectionError::Groups);
    }

    // Stage identity changes transactionally: rejected publications never reserve
    // semantic identities or retain otherwise-dead source references.
    let mut staged_identities = identities.clone();
    let mut seen = vec![false; scene.items().len()];
    let mut bound_resources = HashMap::<ResourceRef, Render2dResourceId>::new();
    let mut bindings = Vec::new();
    let mut entries = Vec::with_capacity(scene.root_entries().len());

    for entry in scene.root_entries() {
        let index = entry
            .item_index()
            .ok_or(RunenUi2dProjectionError::RootStructure)?;
        let item = scene
            .items()
            .get(index)
            .ok_or(RunenUi2dProjectionError::MissingItem(index))?;
        if std::mem::replace(&mut seen[index], true) {
            return Err(RunenUi2dProjectionError::RepeatedItem(index));
        }

        if item.group().is_some() {
            return Err(RunenUi2dProjectionError::Groups);
        }
        entries.push(Render2dEntry::item(project_item(
            scene,
            item,
            &mut staged_identities,
            &mut bound_resources,
            &mut bindings,
        )?));
    }

    if seen.iter().any(|&present| !present) {
        return Err(RunenUi2dProjectionError::RootStructure);
    }

    let projection = RunenUi2dSemanticProjection {
        composition: Render2dComposition::new(entries)?,
        bindings: Render2dResourceBindings::new(bindings)?,
    };
    *identities = staged_identities;
    Ok(projection)
}

fn project_item(
    scene: &runenui_runtime::PaintScene,
    item: &PaintSceneItem,
    identities: &mut RunenUi2dResourceIdentityMap,
    bound_resources: &mut HashMap<ResourceRef, Render2dResourceId>,
    bindings: &mut Vec<Render2dResourceBinding>,
) -> Result<Render2dItem, RunenUi2dProjectionError> {
    if item.layer() != SceneLayer::ZERO {
        return Err(RunenUi2dProjectionError::Layer);
    }
    let transform = project_translation(item.local_to_surface())?;
    // RunenUI clips already live in the surface-logical (direct-root parent) space.
    // Never reapply this item's local-to-surface transform to clip geometry.
    let clips = item
        .clips()
        .iter()
        .map(|clip| project_clip(clip.shape(), clip.clip_to_surface()))
        .collect::<Result<Vec<_>, RunenUi2dProjectionError>>()?;

    let primitive =
        match item.primitive() {
            PaintPrimitive::Fill { shape, brush } => Render2dPrimitive::Fill {
                shape: project_shape(shape)?,
                brush: project_solid_brush(brush)?,
            },
            PaintPrimitive::Stroke {
                shape,
                brush,
                style,
            } => Render2dPrimitive::Stroke {
                shape: project_shape(shape)?,
                brush: project_solid_brush(brush)?,
                style: project_stroke_style(*style)?,
            },
            PaintPrimitive::ShapedTextRun(run) => Render2dPrimitive::ShapedText(
                project_shaped_text(scene, run, identities, bound_resources, bindings)?,
            ),
            _ => return Err(RunenUi2dProjectionError::Primitive),
        };

    Ok(Render2dItem::new(
        primitive,
        transform,
        clips,
        project_opacity(item.opacity()),
    ))
}

fn project_shaped_text(
    scene: &runenui_runtime::PaintScene,
    run: &runenui_core::ShapedTextRunPrimitive,
    identities: &mut RunenUi2dResourceIdentityMap,
    bound_resources: &mut HashMap<ResourceRef, Render2dResourceId>,
    bindings: &mut Vec<Render2dResourceBinding>,
) -> Result<Render2dShapedTextPrimitive, RunenUi2dProjectionError> {
    let resource_id = if let Some(existing) = bound_resources.get(run.resource_ref()) {
        *existing
    } else {
        let next = identities.resolve_or_allocate(run.resource_ref())?;
        let shaped = scene
            .shaped_text_resource(run.resource_ref())
            .ok_or(RunenUi2dProjectionError::ShapedTextMissing)?;
        let font = shaped.font();
        let binding = Render2dFontBinding::new(
            font.bytes().to_vec(),
            font.face_index(),
            font.normalized_coords().to_vec(),
            font.faux_bold(),
            font.faux_skew().map(f64::from),
        )?;
        let glyphs = shaped
            .glyphs()
            .iter()
            .map(|glyph| {
                Render2dGlyph::new(
                    glyph.id(),
                    f64::from(glyph.x()),
                    f64::from(glyph.y()),
                    f64::from(glyph.advance()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let value =
            Render2dShapedTextResource::new(binding, f64::from(shaped.font_size()), glyphs)?;
        bindings.push(Render2dResourceBinding::new(
            next,
            Render2dResourceValue::ShapedText(value),
        ));
        bound_resources.insert(run.resource_ref().clone(), next);
        next
    };

    let origin = run.origin();
    let point = Render2dPoint::new(f64::from(origin.x()), f64::from(origin.y()))?;
    let color = run.foreground();
    let rgba = Render2dColorRgba8::new(color.red(), color.green(), color.blue(), color.alpha());

    Ok(Render2dShapedTextPrimitive::new(resource_id, point, rgba))
}

/// Translate only immutable source-neutral geometry, without choosing a renderer
/// tessellation, a new hit region, or a second shape/brush authority.
fn project_shape(shape: &SceneShape) -> Result<Render2dShape, RunenUi2dProjectionError> {
    match shape {
        SceneShape::Rect(rect) => Ok(Render2dShape::rect(project_rect(*rect)?)),
        SceneShape::RoundedRect { rect, .. } => {
            let radii = shape
                .normalized_radius()
                .expect("rounded-rect variants always have normalized radii");
            Ok(Render2dShape::rounded_rect(
                project_rect(*rect)?,
                Render2dCornerRadii::new(
                    f64::from(radii.top_left().get()),
                    f64::from(radii.top_right().get()),
                    f64::from(radii.bottom_right().get()),
                    f64::from(radii.bottom_left().get()),
                )?,
            ))
        }
        SceneShape::Ellipse(rect) => Ok(Render2dShape::ellipse(project_rect(*rect)?)),
        SceneShape::Path(path) => {
            let fill_rule = match path.fill_rule() {
                PathFillRule::NonZero => Render2dFillRule::NonZero,
                PathFillRule::EvenOdd => Render2dFillRule::EvenOdd,
            };
            let commands = path
                .verbs()
                .iter()
                .map(|verb| {
                    Ok(match *verb {
                        PathVerb::MoveTo(point) => {
                            Render2dPathCommand::MoveTo(project_point(point)?)
                        }
                        PathVerb::LineTo(point) => {
                            Render2dPathCommand::LineTo(project_point(point)?)
                        }
                        PathVerb::QuadraticTo { control, to } => Render2dPathCommand::QuadraticTo {
                            control: project_point(control)?,
                            to: project_point(to)?,
                        },
                        PathVerb::CubicTo {
                            control1,
                            control2,
                            to,
                        } => Render2dPathCommand::CubicTo {
                            control1: project_point(control1)?,
                            control2: project_point(control2)?,
                            to: project_point(to)?,
                        },
                        PathVerb::Close => Render2dPathCommand::Close,
                    })
                })
                .collect::<Result<Vec<_>, RunenUi2dProjectionError>>()?;
            Ok(Render2dShape::path(Render2dPath::new(fill_rule, commands)?))
        }
    }
}

fn project_point(
    point: runenui_core::LogicalPoint,
) -> Result<Render2dPoint, RunenUi2dProjectionError> {
    Ok(Render2dPoint::new(
        f64::from(point.x()),
        f64::from(point.y()),
    )?)
}

fn project_rect(rect: LogicalRect) -> Result<Render2dRect, RunenUi2dProjectionError> {
    Ok(Render2dRect::new(
        f64::from(rect.x()),
        f64::from(rect.y()),
        f64::from(rect.width()),
        f64::from(rect.height()),
    )?)
}

fn project_solid_brush(brush: &Brush) -> Result<Render2dBrush, RunenUi2dProjectionError> {
    match brush {
        Brush::Solid(color) => Ok(Render2dBrush::solid(Render2dColorRgba8::new(
            color.red(),
            color.green(),
            color.blue(),
            color.alpha(),
        ))),
        Brush::Linear(_) | Brush::Radial(_) => Err(RunenUi2dProjectionError::Brush),
    }
}

fn project_stroke_style(
    style: StrokeStyle,
) -> Result<Render2dStrokeStyle, RunenUi2dProjectionError> {
    let cap = match style.cap() {
        StrokeCap::Butt => Render2dStrokeCap::Butt,
        StrokeCap::Round => Render2dStrokeCap::Round,
        StrokeCap::Square => Render2dStrokeCap::Square,
    };
    let join = match style.join() {
        StrokeJoin::Miter => Render2dStrokeJoin::Miter,
        StrokeJoin::Bevel => Render2dStrokeJoin::Bevel,
        StrokeJoin::Round => Render2dStrokeJoin::Round,
    };
    Ok(Render2dStrokeStyle::new(
        f64::from(style.width().get()),
        cap,
        join,
        f64::from(style.miter_limit()),
    )?)
}

/// Map validated source item opacity exactly once at the Render2D item owner.
/// Source SceneOpacity is already finite and in [0, 1], so f32-to-f64 widening
/// preserves its value and the Render2D validation cannot fail.
fn project_opacity(opacity: SceneOpacity) -> Render2dOpacity {
    Render2dOpacity::new(f64::from(opacity.get()))
        .expect("validated RunenUI SceneOpacity is a valid Render2D opacity")
}

/// Preserve a clip's complete independent clip-local-to-surface transform.
/// Singular matrices must stay singular: RunenRender defines them as no coverage.
fn project_clip(
    shape: &SceneShape,
    clip_to_surface: LogicalTransform,
) -> Result<Render2dClip, RunenUi2dProjectionError> {
    Ok(Render2dClip::new(
        project_shape(shape)?,
        project_affine(clip_to_surface)?,
    ))
}

fn project_affine(
    transform: LogicalTransform,
) -> Result<Render2dAffineTransform, RunenUi2dProjectionError> {
    let [m11, m12, m21, m22, tx, ty] = transform.components();
    Ok(Render2dAffineTransform::new(
        f64::from(m11),
        f64::from(m12),
        f64::from(m21),
        f64::from(m22),
        f64::from(tx),
        f64::from(ty),
    )?)
}

fn project_translation(
    transform: LogicalTransform,
) -> Result<Render2dAffineTransform, RunenUi2dProjectionError> {
    let [m11, m12, m21, m22, tx, ty] = transform.components();
    if (m11, m12, m21, m22) != (1.0, 0.0, 0.0, 1.0) {
        return Err(RunenUi2dProjectionError::Transform);
    }
    Ok(Render2dAffineTransform::translation(
        f64::from(tx),
        f64::from(ty),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use runenui_core::ResourceKind;

    #[test]
    fn flat_fills_preserve_rect_colors_and_rounded_ellipse_shapes() {
        let rect = runenui_core::LogicalRect::try_new(3.0, 5.0, 40.0, 24.0).unwrap();
        let shape = project_shape(&SceneShape::rect(rect)).unwrap();
        assert_eq!(
            shape.as_rect(),
            Some(Render2dRect::new(3.0, 5.0, 40.0, 24.0).unwrap())
        );
        let color = runenui_core::Color::rgba(18, 37, 91, 190);
        assert_eq!(
            project_solid_brush(&Brush::solid(color)).unwrap(),
            Render2dBrush::solid(Render2dColorRgba8::new(18, 37, 91, 190))
        );
        let rounded = SceneShape::rounded_rect(
            rect,
            runenui_core::Radius::all(runenui_core::LogicalLength::new(8.0).unwrap()),
        );
        let result = project_shape(&rounded).unwrap();
        assert_eq!(
            result.kind(),
            runen_render::composition_2d::Render2dShapeKind::RoundedRect
        );
        assert_eq!(
            result.as_rounded_rect().unwrap().0,
            project_rect(rect).unwrap()
        );
        assert_eq!(
            project_shape(&SceneShape::ellipse(rect))
                .unwrap()
                .as_ellipse(),
            Some(project_rect(rect).unwrap())
        );
    }

    #[test]
    fn structural_line_and_evenodd_path_keep_exact_commands() {
        use runenui_core::{LogicalPoint, PathFillRule, PathVerb, ScenePath};
        let point = |x, y| LogicalPoint::new(x, y).unwrap();
        let input = ScenePath::new(
            vec![
                PathVerb::MoveTo(point(1.0, 2.0)),
                PathVerb::LineTo(point(10.0, 20.0)),
                PathVerb::QuadraticTo {
                    control: point(3.0, 4.0),
                    to: point(5.0, 6.0),
                },
                PathVerb::Close,
            ],
            PathFillRule::EvenOdd,
        )
        .unwrap();
        let result = project_shape(&SceneShape::path(input)).unwrap();
        let path = result.as_path().unwrap();
        assert_eq!(path.fill_rule(), Render2dFillRule::EvenOdd);
        assert_eq!(
            path.commands(),
            &[
                Render2dPathCommand::MoveTo(Render2dPoint::new(1.0, 2.0).unwrap()),
                Render2dPathCommand::LineTo(Render2dPoint::new(10.0, 20.0).unwrap()),
                Render2dPathCommand::QuadraticTo {
                    control: Render2dPoint::new(3.0, 4.0).unwrap(),
                    to: Render2dPoint::new(5.0, 6.0).unwrap(),
                },
                Render2dPathCommand::Close,
            ]
        );
    }

    #[test]
    fn centered_strokes_preserve_nondefault_cap_join_width_and_miter() {
        let style = StrokeStyle::new(runenui_core::LogicalLength::new(2.5).unwrap())
            .with_cap(StrokeCap::Square)
            .with_join(StrokeJoin::Round)
            .with_miter_limit(7.0)
            .unwrap();
        let result = project_stroke_style(style).unwrap();
        assert_eq!(result.width(), 2.5);
        assert_eq!(result.cap(), Render2dStrokeCap::Square);
        assert_eq!(result.join(), Render2dStrokeJoin::Round);
        assert_eq!(result.miter_limit(), 7.0);
    }

    #[test]
    fn gradient_brushes_fail_closed_and_do_not_fake_solid_coverage() {
        use runenui_core::{
            Color, GradientStop, GradientStops, LinearGradient, LogicalPoint, UnitInterval,
        };
        let stops = GradientStops::new(vec![
            GradientStop::new(UnitInterval::ZERO, Color::BLACK),
            GradientStop::new(UnitInterval::ONE, Color::WHITE),
        ])
        .unwrap();
        let gradient = LinearGradient::new(
            LogicalPoint::new(0.0, 0.0).unwrap(),
            LogicalPoint::new(10.0, 0.0).unwrap(),
            stops,
        )
        .unwrap();
        assert!(matches!(
            project_solid_brush(&Brush::Linear(gradient)),
            Err(RunenUi2dProjectionError::Brush)
        ));
    }

    #[test]
    fn validated_item_opacity_preserves_zero_half_and_full_alpha_without_color_rewriting() {
        for value in [0.0_f32, 0.5, 1.0] {
            let source = SceneOpacity::new(value).unwrap();
            assert_eq!(
                project_opacity(source).get(),
                f64::from(value),
                "item alpha is transferred to the Render2D owner without pre-multiplication"
            );
        }
    }

    #[test]
    fn clipped_root_items_preserve_exact_affine_clip_coordinates_independent_of_item_motion() {
        use runenui_core::{LogicalRect, Radius};

        let bounds = LogicalRect::try_new(1.25, 2.5, 18.75, 9.5).unwrap();
        let shape = SceneShape::rounded_rect(
            bounds,
            Radius::all(runenui_core::LogicalLength::new(2.0).unwrap()),
        );
        let source =
            LogicalTransform::try_new(0.0, 1.0, -1.5, 0.25, 24.0, 4.0).expect("finite affine clip");
        let projected = project_clip(&shape, source).expect("structural clip projection");
        assert_eq!(
            projected.clip_to_parent().components(),
            [0.0, 1.0, -1.5, 0.25, 24.0, 4.0],
            "surface-space clip must not inherit the item's separate local transform"
        );
        assert_eq!(
            projected.shape(),
            &project_shape(&shape).expect("same structural rounded-rect geometry")
        );

        let item_offset = LogicalTransform::translation(18.0, 33.0).unwrap();
        assert_eq!(
            project_translation(item_offset).unwrap().components(),
            [1.0, 0.0, 0.0, 1.0, 18.0, 33.0]
        );
        assert_eq!(
            projected.clip_to_parent().components(),
            [0.0, 1.0, -1.5, 0.25, 24.0, 4.0]
        );
        assert!(matches!(
            project_translation(source),
            Err(RunenUi2dProjectionError::Transform)
        ));
    }

    #[test]
    fn conjunctive_clips_retain_authored_order_and_singular_coverage() {
        use runenui_core::LogicalRect;

        let rect = SceneShape::rect(LogicalRect::try_new(2.25, 4.5, 9.75, 11.5).unwrap());
        let ellipse = SceneShape::ellipse(LogicalRect::try_new(0.0, 0.0, 20.0, 20.0).unwrap());
        let singular = LogicalTransform::try_new(0.0, 0.0, 0.0, 1.0, 6.0, 8.0)
            .expect("finite singular transform is valid");
        let authored = [
            (&rect, LogicalTransform::translation(1.0, 2.0).unwrap()),
            (&ellipse, singular),
        ];
        let clips = authored
            .iter()
            .map(|(shape, transform)| project_clip(shape, *transform))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(clips.len(), 2);
        assert_eq!(clips[0].shape(), &project_shape(&rect).unwrap());
        assert_eq!(
            clips[0].clip_to_parent().components(),
            [1.0, 0.0, 0.0, 1.0, 1.0, 2.0]
        );
        assert_eq!(clips[1].shape(), &project_shape(&ellipse).unwrap());
        assert_eq!(
            clips[1].clip_to_parent().components(),
            [0.0, 0.0, 0.0, 1.0, 6.0, 8.0],
            "singular clip must not be dropped or replaced by an unbounded clip"
        );
    }

    #[test]
    fn idle_retirement_prunes_only_obsolete_exact_sources_without_recycling_ids() {
        let mut map = RunenUi2dResourceIdentityMap::default();
        let still_live = ResourceRef::new(ResourceKind::ShapedTextRun);
        let expired = ResourceRef::new(ResourceKind::ShapedTextRun);
        let live_id = map.resolve_or_allocate(&still_live).expect("live id");
        let expired_id = map.resolve_or_allocate(&expired).expect("expired id");
        let keep = HashSet::from([still_live.clone()]);
        assert_eq!(map.retain_exact_live(&keep), 1);
        assert_eq!(map.retained_count(), 1);
        assert_eq!(
            map.resolve_or_allocate(&still_live)
                .expect("stable live id"),
            live_id
        );
        assert_ne!(
            map.resolve_or_allocate(&expired).expect("fresh renewed id"),
            expired_id
        );
        assert_ne!(
            map.resolve_or_allocate(&expired).expect("same renewed id"),
            live_id
        );
    }

    #[test]
    fn continuous_resource_churn_reclaims_without_global_gpu_idle_and_never_reuses_ids() {
        let mut ids = RunenUi2dResourceIdentityMap::default();
        let shared = ResourceRef::new(ResourceKind::ShapedTextRun);
        let shared_id = ids.resolve_or_allocate(&shared).expect("shared source id");
        let mut greatest_id = shared_id.get();
        for _generation in 0..512 {
            // Two active surfaces can reference the same opaque immutable source.
            let mut current = HashSet::from([shared.clone()]);
            for index in 0..16 {
                let source = ResourceRef::new(ResourceKind::ShapedTextRun);
                let id = ids.resolve_or_allocate(&source).expect("monotonic id");
                assert!(id.get() > greatest_id, "never recycle retired identities");
                greatest_id = id.get();
                if index < 2 {
                    current.insert(source);
                }
            }
            ids.reclaim_obsolete_over_budget(&current);
            assert!(
                ids.retained_count() <= current.len() + 256,
                "CPU source retention must be bounded independently of outstanding GPU submissions"
            );
            assert_eq!(
                ids.resolve_or_allocate(&shared)
                    .expect("shared source survives"),
                shared_id
            );
        }
    }

    #[test]
    fn repeated_distinct_resource_generations_retire_without_recycling_semantic_ids() {
        let mut ids = RunenUi2dResourceIdentityMap::default();
        let mut issued = Vec::new();
        for _generation in 0..64 {
            let current = (0..24)
                .map(|_| ResourceRef::new(ResourceKind::ShapedTextRun))
                .collect::<HashSet<_>>();
            for source in &current {
                let id = ids
                    .resolve_or_allocate(source)
                    .expect("monotonic resource id");
                assert!(
                    !issued.contains(&id),
                    "retired resource ID must never reappear"
                );
                issued.push(id);
                assert_eq!(
                    ids.resolve_or_allocate(source).expect("same live resource"),
                    id,
                );
            }
            ids.retain_exact_live(&current);
            assert_eq!(
                ids.retained_count(),
                24,
                "only current source references survive"
            );
        }
        assert_eq!(issued.len(), 64 * 24);
        ids.retain_exact_live(&HashSet::new());
        assert_eq!(ids.retained_count(), 0);
        let new_id = ids
            .resolve_or_allocate(&ResourceRef::new(ResourceKind::ShapedTextRun))
            .expect("post-idle resource id");
        assert!(
            !issued.contains(&new_id),
            "idle retirement cannot recycle IDs"
        );
    }

    #[test]
    fn resource_ids_are_stable_for_live_sources_and_never_recycled() {
        let mut ids = RunenUi2dResourceIdentityMap::default();
        let source_a = ResourceRef::new(ResourceKind::ShapedTextRun);
        let source_b = ResourceRef::new(ResourceKind::ShapedTextRun);
        let first = ids
            .resolve_or_allocate(&source_a)
            .expect("first resource id");
        assert_eq!(
            first,
            ids.resolve_or_allocate(&source_a)
                .expect("stable resource id")
        );
        let second = ids
            .resolve_or_allocate(&source_b)
            .expect("different resource id");
        assert_ne!(first, second);
        assert_eq!(ids.retained_count(), 2);
        assert!(ids.retire(&source_a));
        assert_eq!(ids.retained_count(), 1);
        assert!(!ids.retire(&source_a));
        let successor = ids
            .resolve_or_allocate(&source_a)
            .expect("retired source renewed");
        assert_ne!(first, successor, "retired resource id must not be recycled");
        assert_ne!(second, successor, "resource identities must not alias");
    }
}
