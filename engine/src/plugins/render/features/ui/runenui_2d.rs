//! Fail-closed projection from one immutable RunenUI paint publication to RunenRender 2D.
//!
//! This is source-adaptation plumbing, not a second renderer or compositor authority.
//! RunenUI resource references are retained only in one adapter-owned identity map;
//! they never become RunenRender resource IDs. The complete SurfacePublication remains owned by
//! the UI integration until exact displayed-input promotion.

use std::collections::HashMap;

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
/// Hold this across frames for the same RunenRender realization lifetime. Retire stale
/// source references after their pending contributions complete. Numeric renderer IDs
/// are never recycled, including after source-resource retirement.
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

    /// Release a source reference after all pending rendering use has retired.
    /// Renderer IDs are still never assigned to a different source reference.
    pub fn retire(&mut self, source: &ResourceRef) -> bool {
        self.by_source.remove(source).is_some()
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
