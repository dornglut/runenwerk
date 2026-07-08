use ui_evaluator::UiEvaluationContext;
use ui_hosts::{GameHost, HostRouteMapVersion};
use ui_math::{UiPoint, UiRect, UiSize};
use ui_program::{RouteId, UiEventPacket, UiEventSourceControlId};
use ui_render_primitives::UiRenderPrimitiveReport;
use ui_runtime_view::UiRuntimeView;
use ui_schema::UiSchemaValue;
use ui_text::{FontAtlasSource, FontId};
use ui_theme::ThemeTokens;

use super::{
    IntoUi, UiAction, UiActionDispatchReport, UiActionDispatchReportsResource, UiActionEvent,
    UiActionHandler, UiHostActionExecutor, UiMountRequestsResource, UiRuntimeDiagnosticsResource,
    UiRuntimeEvaluationReport, UiRuntimeEvaluationResource, UiRuntimePreparedFrameRecord,
    UiRuntimePreparedFrameResource, UiRuntimeTraceResource, dispatch_ui_action,
};

#[derive(Debug, Clone, PartialEq)]
pub struct UiPreparedRuntimeScreenReport {
    evaluation: UiRuntimeEvaluationReport,
    labels: Vec<String>,
    hit_targets: Vec<UiRenderHitTarget>,
}

impl UiPreparedRuntimeScreenReport {
    pub fn evaluation(&self) -> &UiRuntimeEvaluationReport {
        &self.evaluation
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn hit_targets(&self) -> &[UiRenderHitTarget] {
        &self.hit_targets
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiRenderHitTarget {
    control_id: String,
    label: String,
    route: Option<String>,
    capability: Option<String>,
    bounds: UiRect,
    enabled: bool,
}

impl UiRenderHitTarget {
    pub fn new(
        control_id: impl Into<String>,
        label: impl Into<String>,
        route: Option<String>,
        capability: Option<String>,
        bounds: UiRect,
        enabled: bool,
    ) -> Self {
        Self {
            control_id: control_id.into(),
            label: label.into(),
            route,
            capability,
            bounds,
            enabled,
        }
    }

    pub fn control_id(&self) -> &str {
        &self.control_id
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn route(&self) -> Option<&str> {
        self.route.as_deref()
    }

    pub fn capability(&self) -> Option<&str> {
        self.capability.as_deref()
    }

    pub fn bounds(&self) -> UiRect {
        self.bounds
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl From<&ui_render_primitives::UiRenderHitTarget> for UiRenderHitTarget {
    fn from(target: &ui_render_primitives::UiRenderHitTarget) -> Self {
        Self::new(
            target.control_id().to_owned(),
            target.label().to_owned(),
            target.route().map(str::to_owned),
            target.capability().map(str::to_owned),
            target.bounds(),
            target.enabled(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Default, ecs::Resource)]
pub struct UiRuntimeHitTargetResource {
    frame_revision: Option<u64>,
    targets: Vec<UiRenderHitTarget>,
}

impl UiRuntimeHitTargetResource {
    pub fn frame_revision(&self) -> Option<u64> {
        self.frame_revision
    }

    pub fn targets(&self) -> &[UiRenderHitTarget] {
        &self.targets
    }

    pub fn replace_for_evaluation(
        &mut self,
        evaluation: &UiRuntimeEvaluationReport,
        targets: impl IntoIterator<Item = UiRenderHitTarget>,
    ) {
        self.frame_revision = Some(evaluation.frame_payload().frame_revision());
        self.targets = targets.into_iter().collect();
    }

    pub fn clear_for_evaluation(&mut self, evaluation: &UiRuntimeEvaluationReport) {
        self.frame_revision = Some(evaluation.frame_payload().frame_revision());
        self.targets.clear();
    }

    pub fn hit_test(&self, position: (f32, f32)) -> Option<&UiRenderHitTarget> {
        let point = UiPoint::new(position.0, position.1);
        self.targets
            .iter()
            .find(|target| target.enabled() && target.bounds().contains(point))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ecs::Resource)]
pub struct UiPointerActivationResource {
    pressed_control_id: Option<String>,
}

impl UiPointerActivationResource {
    pub fn press(&mut self, targets: &UiRuntimeHitTargetResource, position: (f32, f32)) {
        self.pressed_control_id = targets
            .hit_test(position)
            .map(|target| target.control_id().to_owned());
    }

    pub fn release(
        &mut self,
        targets: &UiRuntimeHitTargetResource,
        position: (f32, f32),
    ) -> Option<UiRenderHitTarget> {
        let pressed_control_id = self.pressed_control_id.take()?;
        let released = targets.hit_test(position)?;
        (released.control_id() == pressed_control_id).then(|| released.clone())
    }

    pub fn clear(&mut self) {
        self.pressed_control_id = None;
    }
}

pub fn evaluate_and_prepare_mounted_ui_screen<S>(
    screen: S,
    context: UiEvaluationContext,
    viewport: UiSize,
    theme: &ThemeTokens,
    atlas_source: &dyn FontAtlasSource,
    font_id: FontId,
    mounts: &UiMountRequestsResource,
    runtime: &mut UiRuntimeEvaluationResource,
    prepared_frames: &mut UiRuntimePreparedFrameResource,
    hit_targets: &mut UiRuntimeHitTargetResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) -> UiPreparedRuntimeScreenReport
where
    S: IntoUi,
{
    let registry = ui_controls::ControlPackageRegistry::new()
        .with_package(ui_controls::runenwerk_control_package())
        .expect("runenwerk controls package should register");
    let source = screen.into_ui_source();
    let lowering = source.lower_with_registry_snapshot(&registry.snapshot());
    assert!(
        lowering.passed(),
        "mounted UI screen lowering failed: {:?}",
        lowering.formation().diagnostics
    );

    let input = super::UiRuntimeEvaluationInput::from_lowering_report(&lowering);
    let runtime_view_report = UiRuntimeView::from_artifact_report(input.artifact());
    let mounted_session = mounts.mounted_sessions().first();
    let evaluation = runtime.evaluate(&input, mounted_session, context, trace, diagnostics);
    let primitive_report = UiRenderPrimitiveReport::from_runtime_view_report(
        &runtime_view_report,
        viewport,
        theme,
        atlas_source,
        font_id,
    );
    let runtime_hit_targets = primitive_report
        .hit_targets()
        .iter()
        .map(UiRenderHitTarget::from)
        .collect::<Vec<_>>();

    if let Some(frame) = primitive_report.frame().cloned() {
        hit_targets.replace_for_evaluation(&evaluation, runtime_hit_targets.iter().cloned());
        prepared_frames.record_frame(
            UiRuntimePreparedFrameRecord::new(&evaluation, frame).with_content_evidence(
                primitive_report.labels().iter().cloned(),
                runtime_hit_targets
                    .iter()
                    .filter_map(|target| target.route().map(str::to_owned)),
            ),
        );
    } else {
        hit_targets.clear_for_evaluation(&evaluation);
    }

    UiPreparedRuntimeScreenReport {
        evaluation,
        labels: primitive_report.labels().to_vec(),
        hit_targets: runtime_hit_targets,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiRuntimeActionRequest {
    source_control_id: UiEventSourceControlId,
    payload: UiSchemaValue,
    route_map_version: HostRouteMapVersion,
    surface_instance_id: Option<ui_surface::SurfaceInstanceId>,
}

impl UiRuntimeActionRequest {
    pub fn new(source_control_id: UiEventSourceControlId, payload: UiSchemaValue) -> Self {
        Self {
            source_control_id,
            payload,
            route_map_version: HostRouteMapVersion::new(1),
            surface_instance_id: None,
        }
    }

    pub fn from_hit_target(target: &UiRenderHitTarget, payload: UiSchemaValue) -> Self {
        Self::new(UiEventSourceControlId::new(target.control_id()), payload)
    }

    pub fn with_route_map_version(mut self, version: HostRouteMapVersion) -> Self {
        self.route_map_version = version;
        self
    }

    pub fn with_surface_instance_id(
        mut self,
        surface_instance_id: Option<ui_surface::SurfaceInstanceId>,
    ) -> Self {
        self.surface_instance_id = surface_instance_id;
        self
    }
}

pub fn dispatch_ui_runtime_action_request<A, Handler, Executor>(
    request: &UiRuntimeActionRequest,
    action: &A,
    handler: &Handler,
    executor: &mut Executor,
    reports: &mut UiActionDispatchReportsResource,
    trace: &mut UiRuntimeTraceResource,
    diagnostics: &mut UiRuntimeDiagnosticsResource,
) -> UiActionDispatchReport
where
    A: UiAction,
    Handler: UiActionHandler<A>,
    Executor: UiHostActionExecutor,
{
    let descriptor = action.action_descriptor();
    let intent = handler.host_intent(action);
    let host = GameHost::new(request.route_map_version)
        .with_mapping(intent.to_host_route_mapping(request.route_map_version));
    let packet = UiEventPacket::new(
        RouteId::new(descriptor.route().as_str()),
        descriptor.schema_version(),
        descriptor.payload_schema().clone(),
        request.payload.clone(),
    )
    .with_capability(descriptor.capability().clone())
    .with_source_control(request.source_control_id.clone());
    let mut event = UiActionEvent::new(packet);
    if let Some(surface_instance_id) = request.surface_instance_id {
        event = event.with_surface_instance_id(surface_instance_id);
    }
    dispatch_ui_action(
        action,
        handler,
        &event,
        &host,
        executor,
        reports,
        trace,
        diagnostics,
    )
}
