use crate::plugins::render::api::ids::UiFrameProducerId;
use crate::plugins::render::backend::RenderSurfaceId;
use std::collections::BTreeMap;
use ui_render_data::UiFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum UiFrameRoute {
    #[default]
    Screen,
    ViewportOverlay,
    WorldProjected,
}

impl UiFrameRoute {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::ViewportOverlay => "viewport_overlay",
            Self::WorldProjected => "world_projected",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct UiFrameSubmissionOrder {
    pub layer: i32,
    pub priority: i32,
}

impl UiFrameSubmissionOrder {
    pub const fn new(layer: i32, priority: i32) -> Self {
        Self { layer, priority }
    }
}

#[derive(Debug, Clone)]
pub struct UiFrameSubmission {
    pub producer_id: UiFrameProducerId,
    pub render_surface_id: Option<RenderSurfaceId>,
    pub route: UiFrameRoute,
    pub order: UiFrameSubmissionOrder,
    pub frame: UiFrame,
    pub rect_shader_asset_id: Option<String>,
}

impl UiFrameSubmission {
    pub fn new(producer_id: impl Into<UiFrameProducerId>) -> Self {
        Self {
            producer_id: producer_id.into(),
            render_surface_id: None,
            route: UiFrameRoute::Screen,
            order: UiFrameSubmissionOrder::default(),
            frame: UiFrame::default(),
            rect_shader_asset_id: None,
        }
    }

    pub fn with_route(mut self, route: UiFrameRoute) -> Self {
        self.route = route;
        self
    }

    pub fn with_render_surface(mut self, render_surface_id: RenderSurfaceId) -> Self {
        self.render_surface_id = Some(render_surface_id);
        self
    }

    pub fn with_order(mut self, order: UiFrameSubmissionOrder) -> Self {
        self.order = order;
        self
    }

    pub fn with_frame(mut self, frame: UiFrame) -> Self {
        self.frame = frame;
        self
    }

    pub fn with_rect_shader_asset_id(
        mut self,
        rect_shader_asset_id: impl Into<Option<String>>,
    ) -> Self {
        self.rect_shader_asset_id = rect_shader_asset_id.into();
        self
    }

    pub fn primitive_count_hint(&self) -> usize {
        self.frame
            .surfaces
            .iter()
            .map(|surface| {
                surface
                    .layers
                    .iter()
                    .map(|layer| layer.primitives.len())
                    .sum::<usize>()
            })
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.frame.is_empty()
    }
}

#[derive(Debug, Clone, Default, ecs::Component, ecs::Resource)]
pub struct UiFrameSubmissionRegistryResource {
    submissions: BTreeMap<(UiFrameProducerId, Option<RenderSurfaceId>), UiFrameSubmission>,
}

impl UiFrameSubmissionRegistryResource {
    pub fn submission_count(&self) -> usize {
        self.submissions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.submissions.is_empty()
    }

    pub fn clear(&mut self) {
        self.submissions.clear();
    }

    pub fn replace(&mut self, submission: UiFrameSubmission) -> Option<UiFrameSubmission> {
        self.submissions.insert(
            (submission.producer_id, submission.render_surface_id),
            submission,
        )
    }

    pub fn replace_for_producer(
        &mut self,
        producer_id: impl Into<UiFrameProducerId>,
        build: impl FnOnce(UiFrameProducerId) -> UiFrameSubmission,
    ) -> Option<UiFrameSubmission> {
        let producer_id = producer_id.into();
        let submission = build(producer_id);
        debug_assert_eq!(
            submission.producer_id, producer_id,
            "submission producer_id must match replace_for_producer key",
        );
        self.submissions.insert((producer_id, None), submission)
    }

    pub fn replace_for_surface(
        &mut self,
        producer_id: impl Into<UiFrameProducerId>,
        render_surface_id: RenderSurfaceId,
        build: impl FnOnce(UiFrameProducerId) -> UiFrameSubmission,
    ) -> Option<UiFrameSubmission> {
        let producer_id = producer_id.into();
        let submission = build(producer_id).with_render_surface(render_surface_id);
        debug_assert_eq!(submission.producer_id, producer_id);
        self.submissions
            .insert((producer_id, Some(render_surface_id)), submission)
    }

    pub fn remove(&mut self, producer_id: &UiFrameProducerId) -> Option<UiFrameSubmission> {
        self.submissions.remove(&(*producer_id, None))
    }

    pub fn get(&self, producer_id: &UiFrameProducerId) -> Option<&UiFrameSubmission> {
        self.submissions.get(&(*producer_id, None))
    }

    pub fn get_for_surface(
        &self,
        producer_id: &UiFrameProducerId,
        render_surface_id: RenderSurfaceId,
    ) -> Option<&UiFrameSubmission> {
        self.submissions
            .get(&(*producer_id, Some(render_surface_id)))
    }

    pub fn ordered_submissions(&self) -> Vec<&UiFrameSubmission> {
        let mut values = self.submissions.values().collect::<Vec<_>>();
        sort_submissions(&mut values);
        values
    }

    pub fn ordered_submissions_for_surface(
        &self,
        render_surface_id: RenderSurfaceId,
    ) -> Vec<&UiFrameSubmission> {
        let mut by_producer = BTreeMap::<UiFrameProducerId, &UiFrameSubmission>::new();
        for submission in self
            .submissions
            .values()
            .filter(|submission| submission.render_surface_id.is_none())
        {
            by_producer.insert(submission.producer_id, submission);
        }
        for submission in self
            .submissions
            .values()
            .filter(|submission| submission.render_surface_id == Some(render_surface_id))
        {
            by_producer.insert(submission.producer_id, submission);
        }
        let mut values = by_producer.into_values().collect::<Vec<_>>();
        sort_submissions(&mut values);
        values
    }
}

fn sort_submissions(values: &mut Vec<&UiFrameSubmission>) {
    values.sort_by(|left, right| {
        left.route
            .cmp(&right.route)
            .then(left.order.layer.cmp(&right.order.layer))
            .then(left.order.priority.cmp(&right.order.priority))
            .then(left.producer_id.cmp(&right.producer_id))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_submission_is_keyed_by_producer() {
        let mut registry = UiFrameSubmissionRegistryResource::default();

        registry.replace(
            UiFrameSubmission::new(UiFrameProducerId::try_from_raw(1).unwrap())
                .with_order(UiFrameSubmissionOrder::new(10, 0)),
        );
        registry.replace(
            UiFrameSubmission::new(UiFrameProducerId::try_from_raw(1).unwrap())
                .with_order(UiFrameSubmissionOrder::new(20, 0)),
        );

        assert_eq!(registry.submission_count(), 1);
        let submission = registry
            .get(&UiFrameProducerId::try_from_raw(1).unwrap())
            .expect("producer submission should exist");
        assert_eq!(submission.order.layer, 20);
    }

    #[test]
    fn ordered_submissions_are_deterministic() {
        let mut registry = UiFrameSubmissionRegistryResource::default();

        registry.replace(
            UiFrameSubmission::new(UiFrameProducerId::try_from_raw(2).unwrap())
                .with_route(UiFrameRoute::Screen)
                .with_order(UiFrameSubmissionOrder::new(100, 5)),
        );
        registry.replace(
            UiFrameSubmission::new(UiFrameProducerId::try_from_raw(1).unwrap())
                .with_route(UiFrameRoute::Screen)
                .with_order(UiFrameSubmissionOrder::new(10, 0)),
        );
        registry.replace(
            UiFrameSubmission::new(UiFrameProducerId::try_from_raw(3).unwrap())
                .with_route(UiFrameRoute::ViewportOverlay)
                .with_order(UiFrameSubmissionOrder::new(0, 0)),
        );

        let ordered = registry
            .ordered_submissions()
            .into_iter()
            .map(|value| value.producer_id)
            .collect::<Vec<_>>();

        assert_eq!(
            ordered,
            vec![
                UiFrameProducerId::try_from_raw(1).unwrap(),
                UiFrameProducerId::try_from_raw(2).unwrap(),
                UiFrameProducerId::try_from_raw(3).unwrap(),
            ]
        );
    }

    #[test]
    fn surface_submission_overrides_global_submission_for_same_producer() {
        let mut registry = UiFrameSubmissionRegistryResource::default();
        let producer = UiFrameProducerId::try_from_raw(1).unwrap();
        let primary = RenderSurfaceId::primary();
        registry.replace(
            UiFrameSubmission::new(producer).with_order(UiFrameSubmissionOrder::new(1, 0)),
        );
        registry.replace_for_surface(producer, primary, |producer_id| {
            UiFrameSubmission::new(producer_id).with_order(UiFrameSubmissionOrder::new(2, 0))
        });

        let submissions = registry.ordered_submissions_for_surface(primary);
        assert_eq!(submissions.len(), 1);
        assert_eq!(submissions[0].order.layer, 2);
        assert_eq!(submissions[0].render_surface_id, Some(primary));
    }
}
