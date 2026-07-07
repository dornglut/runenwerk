use crate::plugins::render::backend::RenderSurfaceId;
use crate::plugins::render::{
    RenderFrameProducerId, SurfaceFrameRoute, SurfaceFrameSubmission, SurfaceFrameSubmissionOrder,
    SurfaceFrameSubmissionRegistryResource,
};
use crate::runtime::{Res, ResMut};

use super::{
    UiRuntimeDiagnostic, UiRuntimeDiagnosticsResource, UiRuntimeEvaluationResource,
    UiRuntimeFramePublicationFailureReason, UiRuntimeFramePublicationReport,
    UiRuntimeFramePublicationResource, UiRuntimePreparedFrameResource, UiRuntimeTraceEvent,
    UiRuntimeTraceResource,
};

pub const UI_RUNTIME_FRAME_PRODUCER_ID: RenderFrameProducerId =
    ui_runtime_frame_producer_id(10_000);

const fn ui_runtime_frame_producer_id(raw: u64) -> RenderFrameProducerId {
    match RenderFrameProducerId::try_from_raw(raw) {
        Ok(id) => id,
        Err(_) => panic!("UI runtime frame producer id constants must be non-zero"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ecs::Resource)]
pub struct UiRuntimeFramePublicationTarget {
    producer_id: RenderFrameProducerId,
    render_surface_id: RenderSurfaceId,
    route: SurfaceFrameRoute,
    order: SurfaceFrameSubmissionOrder,
}

impl Default for UiRuntimeFramePublicationTarget {
    fn default() -> Self {
        Self {
            producer_id: UI_RUNTIME_FRAME_PRODUCER_ID,
            render_surface_id: RenderSurfaceId::primary(),
            route: SurfaceFrameRoute::Screen,
            order: SurfaceFrameSubmissionOrder::new(20, 0),
        }
    }
}

impl UiRuntimeFramePublicationTarget {
    pub fn new(producer_id: RenderFrameProducerId, render_surface_id: RenderSurfaceId) -> Self {
        Self {
            producer_id,
            render_surface_id,
            ..Self::default()
        }
    }

    pub fn producer_id(&self) -> RenderFrameProducerId {
        self.producer_id
    }

    pub fn render_surface_id(&self) -> RenderSurfaceId {
        self.render_surface_id
    }

    pub fn route(&self) -> SurfaceFrameRoute {
        self.route
    }

    pub fn order(&self) -> SurfaceFrameSubmissionOrder {
        self.order
    }

    pub fn with_route(mut self, route: SurfaceFrameRoute) -> Self {
        self.route = route;
        self
    }

    pub fn with_order(mut self, order: SurfaceFrameSubmissionOrder) -> Self {
        self.order = order;
        self
    }
}

pub fn publish_ui_runtime_frame_system(
    runtime: Res<UiRuntimeEvaluationResource>,
    target: Res<UiRuntimeFramePublicationTarget>,
    prepared_frames: Res<UiRuntimePreparedFrameResource>,
    mut submissions: ResMut<SurfaceFrameSubmissionRegistryResource>,
    mut publications: ResMut<UiRuntimeFramePublicationResource>,
    mut trace: ResMut<UiRuntimeTraceResource>,
    mut diagnostics: ResMut<UiRuntimeDiagnosticsResource>,
) {
    publish_latest_ui_runtime_frame(
        &runtime,
        &target,
        &prepared_frames,
        &mut submissions,
        &mut publications,
        &mut trace,
        &mut diagnostics,
    );
}

pub fn publish_latest_ui_runtime_frame(
    runtime: &UiRuntimeEvaluationResource,
    target: &UiRuntimeFramePublicationTarget,
    prepared_frames: &UiRuntimePreparedFrameResource,
    submissions: &mut SurfaceFrameSubmissionRegistryResource,
    publications: &mut UiRuntimeFramePublicationResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) -> UiRuntimeFramePublicationReport {
    let Some(evaluation) = runtime.latest_report() else {
        let producer_id = target.producer_id();
        let render_surface_id = target.render_surface_id();
        submissions.remove_for_surface(&producer_id, render_surface_id);
        let report = UiRuntimeFramePublicationReport::missing_runtime_evaluation(
            producer_id,
            render_surface_id,
        );
        diagnostics.push(UiRuntimeDiagnostic::frame_publication_rejected(
            producer_id,
            render_surface_id,
            UiRuntimeFramePublicationFailureReason::MissingRuntimeEvaluation,
        ));
        trace.record(UiRuntimeTraceEvent::frame_published(&report));
        publications.record(report.clone());
        return report;
    };

    let Some(prepared_frame) = prepared_frames.latest_for_evaluation(evaluation) else {
        let producer_id = target.producer_id();
        let render_surface_id = target.render_surface_id();
        submissions.remove_for_surface(&producer_id, render_surface_id);
        let report = UiRuntimeFramePublicationReport::missing_prepared_frame(
            evaluation,
            producer_id,
            render_surface_id,
        );
        diagnostics.push(UiRuntimeDiagnostic::frame_publication_rejected(
            producer_id,
            render_surface_id,
            UiRuntimeFramePublicationFailureReason::MissingPreparedFrame,
        ));
        trace.record(UiRuntimeTraceEvent::frame_published(&report));
        publications.record(report.clone());
        return report;
    };

    let frame = prepared_frame.frame().clone();
    submissions.replace_for_surface(
        target.producer_id(),
        target.render_surface_id(),
        |producer_id| {
            SurfaceFrameSubmission::new(producer_id)
                .with_route(target.route())
                .with_order(target.order())
                .with_frame(frame)
        },
    );

    let report = UiRuntimeFramePublicationReport::published(
        evaluation,
        target.producer_id(),
        target.render_surface_id(),
        prepared_frame.primitive_count(),
    );
    trace.record(UiRuntimeTraceEvent::frame_published(&report));
    trace.record(UiRuntimeTraceEvent::frame_presented(&report));
    publications.record(report.clone());
    report
}
