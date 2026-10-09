//! RunenUI's typed parallel intake into the existing ordered surface compositor.
//!
//! Selection is surface-scoped before both payload families share one ordering key.
//! The registry stores exact renderer paint publications, never predecessor UiFrame
//! payloads. Its source-specific identity never replaces producer/surface identity.

use std::collections::BTreeMap;

use runenui_runtime::PaintPublication;

use crate::plugins::render::api::ids::RenderFrameProducerId;
use crate::plugins::render::host::RenderSurfaceId;

use super::{
    SurfaceFrameRoute, SurfaceFrameSubmission, SurfaceFrameSubmissionOrder,
    SurfaceFrameSubmissionRegistryResource,
};

/// Renderer-correlation identity for one exact Engine-published RunenUI snapshot.
/// This is not a RunenUI paint revision, hit-generation identity or GPU receipt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RunenUiPublicationId(std::num::NonZeroU64);

impl RunenUiPublicationId {
    /// Creates one explicit, non-zero integration publication correlation ID.
    /// Engine's presentation association owner issues successive IDs monotonically.
    #[must_use]
    pub const fn try_from_raw(raw: u64) -> Option<Self> {
        match std::num::NonZeroU64::new(raw) {
            Some(id) => Some(Self(id)),
            None => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

/// One immutable RunenUI renderer publication submitted by an Engine producer.
#[derive(Clone, Debug)]
pub struct RunenUiPaintSubmission {
    pub producer_id: RenderFrameProducerId,
    pub publication_id: RunenUiPublicationId,
    pub render_surface_id: Option<RenderSurfaceId>,
    pub route: SurfaceFrameRoute,
    pub order: SurfaceFrameSubmissionOrder,
    pub publication: PaintPublication,
}

impl RunenUiPaintSubmission {
    #[must_use]
    pub fn new(
        producer_id: impl Into<RenderFrameProducerId>,
        publication_id: RunenUiPublicationId,
        publication: PaintPublication,
    ) -> Self {
        Self {
            producer_id: producer_id.into(),
            publication_id,
            render_surface_id: None,
            route: SurfaceFrameRoute::Screen,
            order: SurfaceFrameSubmissionOrder::default(),
            publication,
        }
    }

    #[must_use]
    pub const fn with_render_surface(mut self, render_surface_id: RenderSurfaceId) -> Self {
        self.render_surface_id = Some(render_surface_id);
        self
    }

    #[must_use]
    pub const fn with_route(mut self, route: SurfaceFrameRoute) -> Self {
        self.route = route;
        self
    }

    #[must_use]
    pub const fn with_order(mut self, order: SurfaceFrameSubmissionOrder) -> Self {
        self.order = order;
        self
    }
}

/// Separate immutable-paint intake; it does not supersede the legacy registry.
#[derive(Clone, Debug, Default, runen_ecs::Component, runen_ecs::Resource)]
pub struct RunenUiPaintSubmissionRegistryResource {
    submissions: BTreeMap<(RenderFrameProducerId, Option<RenderSurfaceId>), RunenUiPaintSubmission>,
}

impl RunenUiPaintSubmissionRegistryResource {
    #[must_use]
    pub fn submission_count(&self) -> usize {
        self.submissions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.submissions.is_empty()
    }

    pub fn clear(&mut self) {
        self.submissions.clear();
    }

    pub fn replace(
        &mut self,
        submission: RunenUiPaintSubmission,
    ) -> Option<RunenUiPaintSubmission> {
        self.submissions.insert(
            (submission.producer_id, submission.render_surface_id),
            submission,
        )
    }

    /// Returns only the exactly scoped submission; a global fallback is not
    /// authority to overwrite an unrelated source publication.
    #[must_use]
    pub fn get_for_surface(
        &self,
        producer_id: &RenderFrameProducerId,
        render_surface_id: RenderSurfaceId,
    ) -> Option<&RunenUiPaintSubmission> {
        self.submissions
            .get(&(*producer_id, Some(render_surface_id)))
    }

    pub fn remove(
        &mut self,
        producer_id: &RenderFrameProducerId,
    ) -> Option<RunenUiPaintSubmission> {
        self.submissions.remove(&(*producer_id, None))
    }

    pub fn remove_for_surface(
        &mut self,
        producer_id: &RenderFrameProducerId,
        render_surface_id: RenderSurfaceId,
    ) -> Option<RunenUiPaintSubmission> {
        self.submissions
            .remove(&(*producer_id, Some(render_surface_id)))
    }

    #[must_use]
    pub fn ordered_submissions(&self) -> Vec<&RunenUiPaintSubmission> {
        let mut selected = self.submissions.values().collect::<Vec<_>>();
        sort_paint_submissions(&mut selected);
        selected
    }

    /// Exact-surface publication replaces the producer's global publication.
    #[must_use]
    pub fn ordered_submissions_for_surface(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Vec<&RunenUiPaintSubmission> {
        let mut by_producer = BTreeMap::<RenderFrameProducerId, &RunenUiPaintSubmission>::new();
        for submission in self
            .submissions
            .values()
            .filter(|s| s.render_surface_id.is_none())
        {
            by_producer.insert(submission.producer_id, submission);
        }
        for submission in self
            .submissions
            .values()
            .filter(|s| s.render_surface_id == Some(render_surface_id))
        {
            by_producer.insert(submission.producer_id, submission);
        }
        let mut selected = by_producer.into_values().collect::<Vec<_>>();
        sort_paint_submissions(&mut selected);
        selected
    }
}

fn sort_paint_submissions(submissions: &mut [&RunenUiPaintSubmission]) {
    submissions.sort_by_key(|submission| {
        (
            submission.route,
            submission.order.layer,
            submission.order.priority,
            submission.producer_id,
        )
    });
}

/// The transient single ordering domain for legacy and RunenUI submissions.
/// This is ordering-only plumbing, not a shared rendering IR.
#[derive(Clone, Copy, Debug)]
pub enum OrderedSurfaceUiSubmission<'a> {
    Legacy(&'a SurfaceFrameSubmission),
    RunenUi(&'a RunenUiPaintSubmission),
}

impl OrderedSurfaceUiSubmission<'_> {
    #[must_use]
    pub const fn producer_id(self) -> RenderFrameProducerId {
        match self {
            Self::Legacy(s) => s.producer_id,
            Self::RunenUi(s) => s.producer_id,
        }
    }

    #[must_use]
    pub const fn route(self) -> SurfaceFrameRoute {
        match self {
            Self::Legacy(s) => s.route,
            Self::RunenUi(s) => s.route,
        }
    }

    #[must_use]
    pub const fn order(self) -> SurfaceFrameSubmissionOrder {
        match self {
            Self::Legacy(s) => s.order,
            Self::RunenUi(s) => s.order,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum MixedSurfaceUiSubmissionError {
    #[error(
        "UI producer {producer_id:?} submitted both legacy and RunenUI paint to the same surface"
    )]
    CrossFamilyProducerCollision { producer_id: RenderFrameProducerId },
}

/// Selects the surface overrides, rejects collisions, and sorts both paint
/// families in exactly the legacy route/layer/priority/producer ordering.
///
/// # Errors
///
/// Rejects a producer published in both payload families for this surface,
/// including global publication overridden only within one payload family.
pub fn ordered_mixed_ui_submissions_for_surface<'a>(
    legacy: &'a SurfaceFrameSubmissionRegistryResource,
    runenui: &'a RunenUiPaintSubmissionRegistryResource,
    surface: RenderSurfaceId,
) -> Result<Vec<OrderedSurfaceUiSubmission<'a>>, MixedSurfaceUiSubmissionError> {
    let mut by_producer = BTreeMap::<RenderFrameProducerId, OrderedSurfaceUiSubmission<'a>>::new();
    for submission in legacy.ordered_submissions_for_surface(surface) {
        by_producer.insert(
            submission.producer_id,
            OrderedSurfaceUiSubmission::Legacy(submission),
        );
    }
    for submission in runenui.ordered_submissions_for_surface(surface) {
        if by_producer
            .insert(
                submission.producer_id,
                OrderedSurfaceUiSubmission::RunenUi(submission),
            )
            .is_some()
        {
            return Err(
                MixedSurfaceUiSubmissionError::CrossFamilyProducerCollision {
                    producer_id: submission.producer_id,
                },
            );
        }
    }
    let mut ordered = by_producer.into_values().collect::<Vec<_>>();
    ordered.sort_by_key(|submission| {
        (
            submission.route(),
            submission.order().layer,
            submission.order().priority,
            submission.producer_id(),
        )
    });
    Ok(ordered)
}

#[cfg(test)]
mod tests {
    use super::*;

    use runenui_core::{
        FontFamilyName, GenericFontFamily, IntoUpdateOutput, LogicalLength, NoHostProtocol,
        StyleEnvironment, UiApp, View, text,
    };
    use runenui_runtime::{AppRuntime, LogicalSize, SurfaceBuildContext};

    struct SimplePaintApp;

    impl UiApp for SimplePaintApp {
        type State = ();
        type Action = ();
        type HostProtocol = NoHostProtocol;

        fn root(_: &Self::State) -> impl View<Self::Action> {
            text("Counter")
        }

        fn update(
            _: &mut Self::State,
            _: Self::Action,
        ) -> impl IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            ()
        }
    }

    fn paint() -> PaintPublication {
        let mut runtime = AppRuntime::<SimplePaintApp>::mount(());
        let registered = runtime
            .register_text_font_bytes(
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../assets/fonts/JetBrainsMono-Regular.ttf"
                ))
                .to_vec(),
            )
            .expect("bundled UI font must be valid");
        assert!(registered > 0);
        let family = FontFamilyName::new("JetBrains Mono").expect("valid font family");
        assert!(
            runtime
                .set_text_generic_family_mapping(GenericFontFamily::SansSerif, &[family])
                .expect("font mapping")
        );
        let style = StyleEnvironment::default();
        runtime
            .publish_surface(&SurfaceBuildContext::tight(
                &style,
                LogicalSize::new(LogicalLength::from(320_u16), LogicalLength::from(160_u16)),
            ))
            .expect("simple paint publication")
            .paint_publication()
            .clone()
    }

    fn producer(raw: u64) -> RenderFrameProducerId {
        RenderFrameProducerId::try_from_raw(raw).expect("test id")
    }

    fn publication_id(raw: u64) -> RunenUiPublicationId {
        RunenUiPublicationId::try_from_raw(raw).expect("test publication id")
    }

    #[test]
    fn new_and_legacy_paint_share_one_compositor_sort_key() {
        let surface = RenderSurfaceId::primary();
        let mut legacy = SurfaceFrameSubmissionRegistryResource::default();
        let mut runenui = RunenUiPaintSubmissionRegistryResource::default();
        legacy.replace(
            SurfaceFrameSubmission::new(producer(1))
                .with_order(SurfaceFrameSubmissionOrder::new(0, 0)),
        );
        runenui.replace(
            RunenUiPaintSubmission::new(producer(2), publication_id(1), paint())
                .with_order(SurfaceFrameSubmissionOrder::new(1, 0)),
        );
        legacy.replace(
            SurfaceFrameSubmission::new(producer(3))
                .with_order(SurfaceFrameSubmissionOrder::new(2, 0)),
        );
        let result = ordered_mixed_ui_submissions_for_surface(&legacy, &runenui, surface)
            .expect("separate producers");
        assert_eq!(
            result
                .iter()
                .map(|value| value.producer_id())
                .collect::<Vec<_>>(),
            vec![producer(1), producer(2), producer(3)],
        );
        assert!(matches!(result[0], OrderedSurfaceUiSubmission::Legacy(_)));
        assert!(matches!(result[1], OrderedSurfaceUiSubmission::RunenUi(_)));
        assert!(matches!(result[2], OrderedSurfaceUiSubmission::Legacy(_)));
        let (status, prepared) = super::super::resource::prepare_submissions(result)
            .expect("mixed UI preparation must preserve both payload families");
        assert_eq!(
            status,
            crate::plugins::render::features::FeatureContributionStatus::Ready
        );
        assert_eq!(prepared.submissions.len(), 2);
        assert_eq!(prepared.runenui_submissions.len(), 1);
        assert_eq!(
            prepared.ordered,
            vec![
                super::super::PreparedUiSubmissionKind::Legacy(0),
                super::super::PreparedUiSubmissionKind::RunenUi(0),
                super::super::PreparedUiSubmissionKind::Legacy(1),
            ],
        );
        assert_eq!(prepared.submissions[0].submission_order, 0);
        assert_eq!(prepared.runenui_submissions[0].submission_order, 1);
        assert_eq!(prepared.submissions[1].submission_order, 2);
    }

    #[test]
    fn exact_surface_overrides_global_runenui_paint_before_mixed_sorting() {
        let surface = RenderSurfaceId::primary();
        let mut runenui = RunenUiPaintSubmissionRegistryResource::default();
        let legacy = SurfaceFrameSubmissionRegistryResource::default();
        let original = paint();
        runenui.replace(
            RunenUiPaintSubmission::new(producer(2), publication_id(2), original.clone())
                .with_order(SurfaceFrameSubmissionOrder::new(0, 0)),
        );
        runenui.replace(
            RunenUiPaintSubmission::new(producer(2), publication_id(3), original)
                .with_render_surface(surface)
                .with_order(SurfaceFrameSubmissionOrder::new(9, 1)),
        );
        let ordered = ordered_mixed_ui_submissions_for_surface(&legacy, &runenui, surface)
            .expect("surface overrides global");
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].order(), SurfaceFrameSubmissionOrder::new(9, 1));
    }

    #[test]
    fn collision_rejected_even_if_surface_specific_in_only_one_family() {
        let surface = RenderSurfaceId::primary();
        let mut legacy = SurfaceFrameSubmissionRegistryResource::default();
        let mut runenui = RunenUiPaintSubmissionRegistryResource::default();
        legacy.replace(SurfaceFrameSubmission::new(producer(2)));
        runenui.replace(
            RunenUiPaintSubmission::new(producer(2), publication_id(1), paint())
                .with_render_surface(surface),
        );
        assert!(matches!(
            ordered_mixed_ui_submissions_for_surface(&legacy, &runenui, surface),
            Err(MixedSurfaceUiSubmissionError::CrossFamilyProducerCollision { producer_id })
                if producer_id == producer(2)
        ));
    }
}
