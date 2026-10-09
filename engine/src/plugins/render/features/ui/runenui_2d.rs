//! Fail-closed projection from one immutable RunenUI paint publication to RunenRender 2D.
//!
//! This is source-adaptation plumbing, not a second renderer or compositor authority.
//! RunenUI resource references are retained only in one adapter-owned identity map;
//! they never become RunenRender resource IDs. The complete SurfacePublication remains owned by
//! the UI integration until exact displayed-input promotion.

use std::collections::{HashMap, HashSet};

use runen_render::composition_2d::{
    Render2dAffineTransform, Render2dColorRgba8, Render2dComposition, Render2dCompositionError,
    Render2dEntry, Render2dFontBinding, Render2dGeometryError, Render2dGlyph, Render2dItem,
    Render2dOpacity, Render2dPoint, Render2dPrimitive, Render2dResourceBinding,
    Render2dResourceBindingError, Render2dResourceBindings, Render2dResourceError,
    Render2dResourceId, Render2dResourceValue, Render2dShapedTextPrimitive,
    Render2dShapedTextResource,
};
use runenui_core::{LogicalTransform, PaintPrimitive, ResourceRef, SceneLayer, SceneOpacity};
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
    #[error("RunenUI paint item has unsupported clips")]
    Clips,
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

/// Projects only the actual U5 Counter supported subset in RunenUI root order.
///
/// Resource identities are allocated once per immutable source reference and reused
/// across publications through the caller-retained identity map. Duplicate items
/// in one scene bind that semantic resource only once. No font lookup, reshaping,
/// rasterization, or sorting is performed here.
///
/// # Errors
///
/// Rejects every paint/group/opacity/clip/transform fact outside the bounded
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
    if !item.clips().is_empty() {
        return Err(RunenUi2dProjectionError::Clips);
    }
    if item.opacity() != SceneOpacity::OPAQUE {
        return Err(RunenUi2dProjectionError::Opacity);
    }
    if item.layer() != SceneLayer::ZERO {
        return Err(RunenUi2dProjectionError::Layer);
    }
    let transform = project_translation(item.local_to_surface())?;

    let PaintPrimitive::ShapedTextRun(run) = item.primitive() else {
        return Err(RunenUi2dProjectionError::Primitive);
    };

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

    Ok(Render2dItem::new(
        Render2dPrimitive::ShapedText(Render2dShapedTextPrimitive::new(resource_id, point, rgba)),
        transform,
        Vec::new(),
        Render2dOpacity::OPAQUE,
    ))
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
