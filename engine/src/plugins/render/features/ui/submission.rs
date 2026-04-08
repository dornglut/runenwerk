use std::collections::BTreeMap;
use ui_render_data::UiFrame;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiFrameProducerId(String);

impl UiFrameProducerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for UiFrameProducerId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for UiFrameProducerId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

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

#[derive(Debug, Clone, Default)]
pub struct UiFrameSubmission {
    pub producer_id: UiFrameProducerId,
    pub route: UiFrameRoute,
    pub order: UiFrameSubmissionOrder,
    pub frame: UiFrame,
    pub rect_shader_asset_id: Option<String>,
}

impl UiFrameSubmission {
    pub fn new(producer_id: impl Into<UiFrameProducerId>) -> Self {
        Self {
            producer_id: producer_id.into(),
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
    submissions_by_producer: BTreeMap<UiFrameProducerId, UiFrameSubmission>,
}

impl UiFrameSubmissionRegistryResource {
    pub fn submission_count(&self) -> usize {
        self.submissions_by_producer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.submissions_by_producer.is_empty()
    }

    pub fn clear(&mut self) {
        self.submissions_by_producer.clear();
    }

    pub fn replace(&mut self, submission: UiFrameSubmission) -> Option<UiFrameSubmission> {
        self.submissions_by_producer
            .insert(submission.producer_id.clone(), submission)
    }

    pub fn replace_for_producer(
        &mut self,
        producer_id: impl Into<UiFrameProducerId>,
        build: impl FnOnce(UiFrameProducerId) -> UiFrameSubmission,
    ) -> Option<UiFrameSubmission> {
        let producer_id = producer_id.into();
        let submission = build(producer_id.clone());
        debug_assert_eq!(
            submission.producer_id, producer_id,
            "submission producer_id must match replace_for_producer key",
        );
        self.submissions_by_producer.insert(producer_id, submission)
    }

    pub fn remove(&mut self, producer_id: &UiFrameProducerId) -> Option<UiFrameSubmission> {
        self.submissions_by_producer.remove(producer_id)
    }

    pub fn get(&self, producer_id: &UiFrameProducerId) -> Option<&UiFrameSubmission> {
        self.submissions_by_producer.get(producer_id)
    }

    pub fn ordered_submissions(&self) -> Vec<&UiFrameSubmission> {
        let mut values = self.submissions_by_producer.values().collect::<Vec<_>>();
        values.sort_by(|left, right| {
            left.route
                .cmp(&right.route)
                .then(left.order.layer.cmp(&right.order.layer))
                .then(left.order.priority.cmp(&right.order.priority))
                .then(left.producer_id.cmp(&right.producer_id))
        });
        values
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacing_submission_is_keyed_by_producer() {
        let mut registry = UiFrameSubmissionRegistryResource::default();

        registry.replace(
            UiFrameSubmission::new("editor.shell").with_order(UiFrameSubmissionOrder::new(10, 0)),
        );
        registry.replace(
            UiFrameSubmission::new("editor.shell").with_order(UiFrameSubmissionOrder::new(20, 0)),
        );

        assert_eq!(registry.submission_count(), 1);
        let submission = registry
            .get(&UiFrameProducerId::new("editor.shell"))
            .expect("editor shell submission should exist");
        assert_eq!(submission.order.layer, 20);
    }

    #[test]
    fn ordered_submissions_are_deterministic() {
        let mut registry = UiFrameSubmissionRegistryResource::default();

        registry.replace(
            UiFrameSubmission::new("debug.metrics")
                .with_route(UiFrameRoute::Screen)
                .with_order(UiFrameSubmissionOrder::new(100, 5)),
        );
        registry.replace(
            UiFrameSubmission::new("editor.shell")
                .with_route(UiFrameRoute::Screen)
                .with_order(UiFrameSubmissionOrder::new(10, 0)),
        );
        registry.replace(
            UiFrameSubmission::new("editor.viewport")
                .with_route(UiFrameRoute::ViewportOverlay)
                .with_order(UiFrameSubmissionOrder::new(0, 0)),
        );

        let ordered = registry
            .ordered_submissions()
            .into_iter()
            .map(|value| value.producer_id.as_str().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            ordered,
            vec![
                "editor.shell".to_string(),
                "debug.metrics".to_string(),
                "editor.viewport".to_string(),
            ]
        );
    }
}
