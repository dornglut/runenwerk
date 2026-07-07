use ui_evaluator::UiEvaluationContext;
use ui_hosts::{GameHost, HostRouteMapVersion};
use ui_math::{UiPoint, UiRect, UiSize};
use ui_program::{RouteId, UiEventPacket, UiEventSourceControlId};
use ui_render_data::{
    BorderPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_runtime_view::{ButtonRuntimeView, ButtonRuntimeViewReport, RuntimeControlView, UiRuntimeView};
use ui_schema::UiSchemaValue;
use ui_text::{
    AtlasTextLayouter, FontAtlasSource, FontId, TextBlock, TextBlockId, TextBlockLayoutRequest,
    TextBlockLayoutResult, TextDirectionPolicy, TextHeightConstraint, TextHorizontalAlign,
    TextLayoutPolicy, TextLayouter, TextLineHeightPolicy, TextOverflowPolicy, TextRun, TextRunId,
    TextSemanticRole, TextStyle, TextVerticalAlign, TextWhitespacePolicy, TextWidthConstraint,
    TextWrapPolicy,
};
use ui_theme::{ThemeTokens, UiColor};

use super::{
    dispatch_ui_action, IntoUi, UiAction, UiActionDispatchReport, UiActionDispatchReportsResource,
    UiActionEvent, UiActionHandler, UiHostActionExecutor, UiMountRequestsResource,
    UiRuntimeDiagnosticsResource, UiRuntimeEvaluationReport, UiRuntimeEvaluationResource,
    UiRuntimePreparedFrameRecord, UiRuntimePreparedFrameResource, UiRuntimeTraceResource,
};

const BUTTON_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.button";
const LABEL_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.label";

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
    let prepared = render_runtime_view(&runtime_view_report, viewport, theme, atlas_source, font_id);

    if let Some(frame) = prepared.frame.clone() {
        hit_targets.replace_for_evaluation(&evaluation, prepared.hit_targets.iter().cloned());
        prepared_frames.record_frame(
            UiRuntimePreparedFrameRecord::new(&evaluation, frame).with_content_evidence(
                prepared.labels.iter().cloned(),
                prepared
                    .hit_targets
                    .iter()
                    .filter_map(|target| target.route().map(str::to_owned)),
            ),
        );
    } else {
        hit_targets.clear_for_evaluation(&evaluation);
    }

    UiPreparedRuntimeScreenReport {
        evaluation,
        labels: prepared.labels,
        hit_targets: prepared.hit_targets,
    }
}

#[derive(Debug, Clone, PartialEq)]
struct PreparedPrimitiveOutput {
    frame: Option<UiFrame>,
    labels: Vec<String>,
    hit_targets: Vec<UiRenderHitTarget>,
}

fn render_runtime_view(
    report: &ui_runtime_view::UiRuntimeViewReport,
    viewport: UiSize,
    theme: &ThemeTokens,
    atlas_source: &dyn FontAtlasSource,
    font_id: FontId,
) -> PreparedPrimitiveOutput {
    if !report.passed() {
        return PreparedPrimitiveOutput {
            frame: None,
            labels: Vec::new(),
            hit_targets: Vec::new(),
        };
    }
    let button_report = ButtonRuntimeViewReport::from_runtime_view_report(report);

    let mut layer = UiLayer::new(UiLayerId(0));
    let mut labels = Vec::new();
    let mut targets = Vec::new();
    let mut order = 0_u32;
    let mut stack = 0_usize;
    let buttons = &button_report.buttons;

    for control in report.view.controls() {
        match control.control.node.control_kind.as_str() {
            LABEL_CONTROL_KIND_ID => {
                let text = label_text(control);
                labels.push(text.clone());
                let bounds = UiRect::new(
                    24.0,
                    24.0 + stack as f32 * 40.0,
                    (viewport.width - 48.0).max(1.0),
                    30.0,
                );
                push_text(
                    &mut layer,
                    &mut order,
                    &text,
                    bounds,
                    theme,
                    atlas_source,
                    font_id,
                    TextHorizontalAlign::Start,
                );
                stack += 1;
            }
            BUTTON_CONTROL_KIND_ID => {
                if let Some(button) = buttons
                    .iter()
                    .find(|button| button.control_id == control.control_id().as_str())
                {
                    labels.push(button.label.clone());
                    let bounds = UiRect::new(24.0, 24.0 + stack as f32 * 48.0, 168.0, 38.0);
                    push_button(
                        &mut layer,
                        &mut order,
                        button,
                        bounds,
                        theme,
                        atlas_source,
                        font_id,
                    );
                    if button.route.is_some() {
                        targets.push(UiRenderHitTarget::new(
                            button.control_id.clone(),
                            button.label.clone(),
                            button.route.clone(),
                            button.capability.clone(),
                            bounds,
                            !button.disabled,
                        ));
                    }
                    stack += 1;
                }
            }
            _ => {}
        }
    }

    PreparedPrimitiveOutput {
        frame: Some(UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            viewport,
            vec![layer],
        )])),
        labels,
        hit_targets: targets,
    }
}

fn label_text(control: &RuntimeControlView) -> String {
    control
        .property()
        .and_then(|property| {
            property
                .snapshot
                .get("text")
                .or_else(|| property.snapshot.get("label"))
        })
        .and_then(UiSchemaValue::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn push_button(
    layer: &mut UiLayer,
    order: &mut u32,
    button: &ButtonRuntimeView,
    bounds: UiRect,
    theme: &ThemeTokens,
    atlas_source: &dyn FontAtlasSource,
    font_id: FontId,
) {
    let background = if button.selected {
        theme.status_input
    } else {
        theme.background_panel
    };
    layer.push(UiPrimitive::Rect(RectPrimitive::new(
        bounds,
        theme.radius.md,
        paint(background),
        UiDrawKey::new(1, None),
        sort(*order),
    )));
    *order += 1;
    layer.push(UiPrimitive::Border(BorderPrimitive::new(
        bounds,
        theme.radius.md,
        theme.border_width,
        paint(theme.border),
        UiDrawKey::new(2, None),
        sort(*order),
    )));
    *order += 1;
    let text_bounds = UiRect::new(
        bounds.x + 12.0,
        bounds.y,
        (bounds.width - 24.0).max(0.0),
        bounds.height,
    );
    push_text(
        layer,
        order,
        &button.label,
        text_bounds,
        theme,
        atlas_source,
        font_id,
        TextHorizontalAlign::Center,
    );
}

fn push_text(
    layer: &mut UiLayer,
    order: &mut u32,
    text: &str,
    bounds: UiRect,
    theme: &ThemeTokens,
    atlas_source: &dyn FontAtlasSource,
    font_id: FontId,
    align: TextHorizontalAlign,
) {
    let mut style = theme.body_text_style(font_id);
    style.font_size = theme.typography.body.max(1.0);
    style.line_height = TextLineHeightPolicy::Absolute((style.font_size * 1.35).max(1.0));
    style.color = [
        theme.foreground.r,
        theme.foreground.g,
        theme.foreground.b,
        theme.foreground.a,
    ];
    let layout = text_layout(text, bounds, &style, atlas_source, font_id, align);
    layer.push(UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
        layout,
        Some(bounds),
        paint_from_style(&style),
        UiDrawKey::new(0, Some(font_id.0)),
        sort(*order),
    )));
    *order += 1;
}

fn text_layout(
    text: &str,
    bounds: UiRect,
    style: &TextStyle,
    atlas_source: &dyn FontAtlasSource,
    _font_id: FontId,
    align: TextHorizontalAlign,
) -> TextBlockLayoutResult {
    let block = TextBlock::new(TextBlockId(1), style.clone())
        .with_run(TextRun::new(TextRunId(1), text).with_semantic_role(TextSemanticRole::Label))
        .with_layout(TextLayoutPolicy {
            width_constraint: TextWidthConstraint::Exact(bounds.width.max(0.0)),
            height_constraint: TextHeightConstraint::Unconstrained,
            wrap: TextWrapPolicy::NoWrap,
            whitespace: TextWhitespacePolicy::Preserve,
            horizontal_align: align,
            vertical_align: TextVerticalAlign::Start,
            overflow: TextOverflowPolicy::Clip,
            max_lines: Some(1),
            text_direction: TextDirectionPolicy::Ltr,
        });
    let mut layout = AtlasTextLayouter.layout(atlas_source, TextBlockLayoutRequest::new(block));
    let dy = ((bounds.height - layout.measured_size.height) * 0.5).max(0.0);
    translate_layout(&mut layout, bounds.x, bounds.y + dy);
    layout
}

fn translate_layout(layout: &mut TextBlockLayoutResult, dx: f32, dy: f32) {
    layout.content_bounds.x += dx;
    layout.content_bounds.y += dy;
    layout.ink_bounds.x += dx;
    layout.ink_bounds.y += dy;
    for line in &mut layout.line_metrics {
        line.origin.x += dx;
        line.origin.y += dy;
        line.baseline_y += dy;
        line.line_box.x += dx;
        line.line_box.y += dy;
        line.ink_bounds.x += dx;
        line.ink_bounds.y += dy;
    }
    for visual_run in &mut layout.visual_runs {
        visual_run.bounds.x += dx;
        visual_run.bounds.y += dy;
        for glyph in &mut visual_run.glyphs {
            glyph.origin.x += dx;
            glyph.origin.y += dy;
            glyph.bounds.x += dx;
            glyph.bounds.y += dy;
        }
    }
}

fn paint(color: UiColor) -> UiPaint {
    UiPaint::rgba(color.r, color.g, color.b, color.a)
}

fn paint_from_style(style: &TextStyle) -> UiPaint {
    UiPaint::rgba(
        style.color[0],
        style.color[1],
        style.color[2],
        style.color[3],
    )
}

fn sort(order: u32) -> UiSortKey {
    UiSortKey::new(0, 0, order)
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
