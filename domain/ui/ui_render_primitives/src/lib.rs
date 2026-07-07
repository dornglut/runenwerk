//! Backend-neutral primitive generation from artifact-backed runtime views.

use serde::{Deserialize, Serialize};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_runtime_view::{
    ButtonRuntimeHostData, ButtonRuntimeView, ButtonRuntimeViewDiagnostic,
    ButtonRuntimeViewDiagnosticSeverity, ButtonRuntimeViewReport, RuntimeControlView,
    UiRuntimeViewReport,
};
use ui_schema::UiSchemaValue;
use ui_text::{
    AtlasTextLayouter, FontAtlasSource, FontId, TextBlock, TextBlockId, TextBlockLayoutRequest,
    TextBlockLayoutResult, TextDirectionPolicy, TextHeightConstraint, TextHorizontalAlign,
    TextLayoutPolicy, TextLayouter, TextLineHeightPolicy, TextOverflowPolicy, TextRun, TextRunId,
    TextSemanticRole, TextStyle, TextVerticalAlign, TextWhitespacePolicy, TextWidthConstraint,
    TextWrapPolicy,
};
use ui_theme::{ThemeTokens, UiColor};

pub const DIAGNOSTIC_RUNTIME_VIEW_FAILED: &str = "ui.render_primitives.runtime_view_failed";
pub const DIAGNOSTIC_BUTTON_VIEW_FAILED: &str = "ui.render_primitives.button_view_failed";
pub const DIAGNOSTIC_TEXT_LAYOUT_FAILED: &str = "ui.render_primitives.text.layout_failed";
pub const DIAGNOSTIC_EMPTY_GLYPH_RUN: &str = "ui.render_primitives.text.empty_glyph_run";
pub const DIAGNOSTIC_LABEL_MISSING_PROPERTY: &str = "ui.render_primitives.label.missing_property";
pub const DIAGNOSTIC_LABEL_MISSING_TEXT: &str = "ui.render_primitives.label.missing_text";

const BUTTON_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.button";
const LABEL_CONTROL_KIND_ID: &str = "runenwerk.ui.controls.label";

#[derive(Clone, Debug, PartialEq)]
pub struct UiRenderPrimitiveReport {
    frame: Option<UiFrame>,
    labels: Vec<String>,
    hit_targets: Vec<UiRenderHitTarget>,
    diagnostics: Vec<UiRenderPrimitiveDiagnostic>,
    provenance: Vec<UiRenderPrimitiveProvenance>,
    button_report: ButtonRuntimeViewReport,
}

impl UiRenderPrimitiveReport {
    pub fn from_runtime_view_report(
        report: &UiRuntimeViewReport,
        viewport: UiSize,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        font_id: FontId,
    ) -> Self {
        Self::from_runtime_view_report_with_host_data(
            report,
            viewport,
            theme,
            atlas_source,
            font_id,
            &ButtonRuntimeHostData::default(),
        )
    }

    pub fn from_runtime_view_report_with_host_data(
        report: &UiRuntimeViewReport,
        viewport: UiSize,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        font_id: FontId,
        host_data: &ButtonRuntimeHostData,
    ) -> Self {
        if !report.passed() {
            return Self::failed(
                DIAGNOSTIC_RUNTIME_VIEW_FAILED,
                "render primitive generation refused failed runtime view report",
            );
        }

        let labels = label_views_from_runtime_view_report(report);
        let button_report =
            ButtonRuntimeViewReport::from_runtime_view_report_with_host_data(report, host_data);
        Self::from_button_and_label_views(button_report, labels, viewport, theme, atlas_source, font_id)
    }

    pub fn from_button_report(
        button_report: ButtonRuntimeViewReport,
        viewport: UiSize,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        font_id: FontId,
    ) -> Self {
        Self::from_button_and_label_views(
            button_report,
            Vec::new(),
            viewport,
            theme,
            atlas_source,
            font_id,
        )
    }

    fn from_button_and_label_views(
        button_report: ButtonRuntimeViewReport,
        label_views: Vec<LabelRuntimePrimitiveView>,
        viewport: UiSize,
        theme: &ThemeTokens,
        atlas_source: &dyn FontAtlasSource,
        font_id: FontId,
    ) -> Self {
        let mut diagnostics = primitive_diagnostics_from_button_report(&button_report.diagnostics);
        let label_only_or_mixed_labels_exist = !label_views.is_empty();
        if !button_report.passed() && !label_only_or_mixed_labels_exist {
            diagnostics.push(UiRenderPrimitiveDiagnostic::error(
                DIAGNOSTIC_BUTTON_VIEW_FAILED,
                "render primitive generation refused failed button runtime view report",
                None,
            ));
            return Self {
                frame: None,
                labels: Vec::new(),
                hit_targets: Vec::new(),
                diagnostics,
                provenance: Vec::new(),
                button_report,
            };
        }

        let mut layer = UiLayer::new(UiLayerId(0));
        let mut provenance = Vec::new();
        let mut labels = Vec::new();
        let mut hit_targets = Vec::new();
        let mut order = 0_u32;
        let mut stack = 0_usize;
        let layouter = AtlasTextLayouter;

        for control in ControlPrimitiveView::from_views(&label_views, &button_report.buttons) {
            match control {
                ControlPrimitiveView::Label(label) => {
                    labels.push(label.text.clone());
                    let bounds = UiRect::new(24.0, 24.0 + stack as f32 * 40.0, (viewport.width - 48.0).max(1.0), 30.0);
                    let style = label_text_style(theme, font_id);
                    let layout = text_layout(&label.text, bounds, &style, atlas_source, &layouter, TextBlockId(stack as u64 + 1), TextHorizontalAlign::Start);
                    push_text_primitive(&mut layer, &mut provenance, &mut diagnostics, &mut order, label.source_map_index, &label.control_id, layout, bounds, &style);
                    stack += 1;
                }
                ControlPrimitiveView::Button(button) => {
                    labels.push(button.label.clone());
                    let bounds = UiRect::new(24.0, 24.0 + stack as f32 * 48.0, 168.0, 38.0);
                    push_button_primitives(&mut layer, &mut provenance, &mut diagnostics, &mut order, button, bounds, theme, atlas_source, &layouter, font_id);
                    if button.route.is_some() {
                        hit_targets.push(UiRenderHitTarget::new(
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
        }

        if diagnostics.iter().any(|d| d.severity == UiRenderPrimitiveDiagnosticSeverity::Error) {
            return Self {
                frame: None,
                labels,
                hit_targets,
                diagnostics,
                provenance,
                button_report,
            };
        }

        let frame = UiFrame::with_surfaces(vec![UiSurface::with_layers(
            UiSurfaceId(0),
            viewport,
            vec![layer],
        )]);
        Self {
            frame: Some(frame),
            labels,
            hit_targets,
            diagnostics,
            provenance,
            button_report,
        }
    }

    fn failed(code: &str, message: &str) -> Self {
        Self {
            frame: None,
            labels: Vec::new(),
            hit_targets: Vec::new(),
            diagnostics: vec![UiRenderPrimitiveDiagnostic::error(code, message, None)],
            provenance: Vec::new(),
            button_report: ButtonRuntimeViewReport::default(),
        }
    }

    pub fn passed(&self) -> bool {
        self.diagnostics.iter().all(|d| d.severity != UiRenderPrimitiveDiagnosticSeverity::Error)
    }

    pub fn frame(&self) -> Option<&UiFrame> {
        self.frame.as_ref()
    }

    pub fn into_frame(self) -> Option<UiFrame> {
        self.frame
    }

    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    pub fn hit_targets(&self) -> &[UiRenderHitTarget] {
        &self.hit_targets
    }

    pub fn diagnostics(&self) -> &[UiRenderPrimitiveDiagnostic] {
        &self.diagnostics
    }

    pub fn provenance(&self) -> &[UiRenderPrimitiveProvenance] {
        &self.provenance
    }

    pub fn button_report(&self) -> &ButtonRuntimeViewReport {
        &self.button_report
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

    pub fn control_id(&self) -> &str { &self.control_id }
    pub fn label(&self) -> &str { &self.label }
    pub fn route(&self) -> Option<&str> { self.route.as_deref() }
    pub fn capability(&self) -> Option<&str> { self.capability.as_deref() }
    pub fn bounds(&self) -> UiRect { self.bounds }
    pub fn enabled(&self) -> bool { self.enabled }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderPrimitiveDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: UiRenderPrimitiveDiagnosticSeverity,
    #[serde(default)]
    pub source_map_index: Option<u32>,
}

impl UiRenderPrimitiveDiagnostic {
    pub fn error(code: impl Into<String>, message: impl Into<String>, source_map_index: Option<u32>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: UiRenderPrimitiveDiagnosticSeverity::Error,
            source_map_index,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiRenderPrimitiveDiagnosticSeverity {
    Info,
    Warning,
    #[default]
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiRenderPrimitiveProvenance {
    pub surface_index: usize,
    pub layer_index: usize,
    pub primitive_index: usize,
    pub source_map_index: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct LabelRuntimePrimitiveView {
    control_id: String,
    text: String,
    source_map_index: Option<u32>,
    ordinal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ControlPrimitiveView<'a> {
    Label(&'a LabelRuntimePrimitiveView),
    Button(&'a ButtonRuntimeView),
}

impl<'a> ControlPrimitiveView<'a> {
    fn from_views(labels: &'a [LabelRuntimePrimitiveView], buttons: &'a [ButtonRuntimeView]) -> Vec<Self> {
        let mut controls = Vec::with_capacity(labels.len() + buttons.len());
        controls.extend(labels.iter().map(Self::Label));
        controls.extend(buttons.iter().map(Self::Button));
        controls.sort_by_key(|control| match control {
            Self::Label(label) => label.ordinal,
            Self::Button(button) => button.source_map_indexes.control.or(button.source_map_indexes.property).unwrap_or(u32::MAX) as usize,
        });
        controls
    }
}

fn label_views_from_runtime_view_report(report: &UiRuntimeViewReport) -> Vec<LabelRuntimePrimitiveView> {
    report.view.controls().enumerate().filter_map(|(ordinal, control)| label_view_from_control(control, ordinal)).collect()
}

fn label_view_from_control(control: &RuntimeControlView, ordinal: usize) -> Option<LabelRuntimePrimitiveView> {
    if control.control.node.control_kind.as_str() != LABEL_CONTROL_KIND_ID {
        return None;
    }
    let text = control.property()
        .and_then(|property| property.snapshot.get("text").or_else(|| property.snapshot.get("label")))
        .and_then(UiSchemaValue::as_str)
        .unwrap_or_default()
        .to_owned();
    let source_map_index = control.property().and_then(|property| property.source_map_index).or(control.control.source_map_index);
    Some(LabelRuntimePrimitiveView {
        control_id: control.control_id().as_str().to_owned(),
        text,
        source_map_index,
        ordinal,
    })
}

fn push_button_primitives(
    layer: &mut UiLayer,
    provenance: &mut Vec<UiRenderPrimitiveProvenance>,
    diagnostics: &mut Vec<UiRenderPrimitiveDiagnostic>,
    order: &mut u32,
    button: &ButtonRuntimeView,
    bounds: UiRect,
    theme: &ThemeTokens,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    font_id: FontId,
) {
    let source_map_index = button.source_map_indexes.property.or(button.source_map_indexes.control);
    let (background, border, text) = button_colors(theme, button);
    push_primitive(layer, provenance, 0, source_map_index, UiPrimitive::Rect(RectPrimitive::new(bounds, theme.radius.md, paint(background), UiDrawKey::new(1, None), sort_key(*order))));
    *order += 1;
    push_primitive(layer, provenance, 0, source_map_index, UiPrimitive::Border(BorderPrimitive::new(bounds, theme.radius.md, theme.border_width, paint(border), UiDrawKey::new(2, None), sort_key(*order))));
    *order += 1;
    let mut style = label_text_style(theme, font_id);
    style.color = [text.r, text.g, text.b, text.a];
    let text_bounds = UiRect::new(bounds.x + 12.0, bounds.y, (bounds.width - 24.0).max(0.0), bounds.height);
    let layout = text_layout(&button.label, text_bounds, &style, atlas_source, layouter, TextBlockId(*order as u64 + 1), TextHorizontalAlign::Center);
    push_text_primitive(layer, provenance, diagnostics, order, source_map_index, &button.control_id, layout, text_bounds, &style);
}

fn label_text_style(theme: &ThemeTokens, font_id: FontId) -> TextStyle {
    let mut style = theme.body_text_style(font_id);
    style.font_size = theme.typography.body.max(1.0);
    style.line_height = TextLineHeightPolicy::Absolute((style.font_size * 1.35).max(1.0));
    style.color = [theme.foreground.r, theme.foreground.g, theme.foreground.b, theme.foreground.a];
    style
}

fn text_layout(
    text: &str,
    bounds: UiRect,
    style: &TextStyle,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    block_id: TextBlockId,
    horizontal_align: TextHorizontalAlign,
) -> TextBlockLayoutResult {
    let block = TextBlock::new(block_id, style.clone())
        .with_run(TextRun::new(TextRunId(1), text).with_semantic_role(TextSemanticRole::Label))
        .with_semantic_role(TextSemanticRole::Label)
        .with_layout(TextLayoutPolicy {
            width_constraint: TextWidthConstraint::Exact(bounds.width.max(0.0)),
            height_constraint: TextHeightConstraint::Unconstrained,
            wrap: TextWrapPolicy::NoWrap,
            whitespace: TextWhitespacePolicy::Preserve,
            horizontal_align,
            vertical_align: TextVerticalAlign::Start,
            overflow: TextOverflowPolicy::Clip,
            max_lines: Some(1),
            text_direction: TextDirectionPolicy::Ltr,
        });
    let mut layout = layouter.layout(atlas_source, TextBlockLayoutRequest::new(block));
    let vertical_offset = ((bounds.height - layout.measured_size.height) * 0.5).max(0.0);
    translate_text_layout(&mut layout, bounds.x, bounds.y + vertical_offset);
    layout
}

fn push_text_primitive(
    layer: &mut UiLayer,
    provenance: &mut Vec<UiRenderPrimitiveProvenance>,
    diagnostics: &mut Vec<UiRenderPrimitiveDiagnostic>,
    order: &mut u32,
    source_map_index: Option<u32>,
    control_id: &str,
    layout: TextBlockLayoutResult,
    clip_bounds: UiRect,
    style: &TextStyle,
) {
    if layout.glyph_count == 0 {
        diagnostics.push(UiRenderPrimitiveDiagnostic::error(
            DIAGNOSTIC_EMPTY_GLYPH_RUN,
            format!("control {control_id} produced an empty glyph run"),
            source_map_index,
        ));
        return;
    }
    if layout.diagnostics.iter().any(|d| d.severity == ui_text::TextDiagnosticSeverity::Error) {
        diagnostics.push(UiRenderPrimitiveDiagnostic::error(
            DIAGNOSTIC_TEXT_LAYOUT_FAILED,
            format!("control {control_id} text could not be shaped with the supplied font atlas"),
            source_map_index,
        ));
        return;
    }
    push_primitive(layer, provenance, 0, source_map_index, UiPrimitive::GlyphRun(GlyphRunPrimitive::new(layout, Some(clip_bounds), paint_from_style(style), UiDrawKey::new(0, Some(style.font_id.0)), sort_key(*order))));
    *order += 1;
}

fn translate_text_layout(layout: &mut TextBlockLayoutResult, dx: f32, dy: f32) {
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

fn push_primitive(
    layer: &mut UiLayer,
    provenance: &mut Vec<UiRenderPrimitiveProvenance>,
    layer_index: usize,
    source_map_index: Option<u32>,
    primitive: UiPrimitive,
) {
    let primitive_index = layer.primitives.len();
    if let Some(source_map_index) = source_map_index {
        provenance.push(UiRenderPrimitiveProvenance {
            surface_index: 0,
            layer_index,
            primitive_index,
            source_map_index,
        });
    }
    layer.push(primitive);
}

fn primitive_diagnostics_from_button_report(diagnostics: &[ButtonRuntimeViewDiagnostic]) -> Vec<UiRenderPrimitiveDiagnostic> {
    diagnostics.iter().map(|diagnostic| UiRenderPrimitiveDiagnostic {
        code: diagnostic.code.clone(),
        message: diagnostic.message.clone(),
        severity: match diagnostic.severity {
            ButtonRuntimeViewDiagnosticSeverity::Info => UiRenderPrimitiveDiagnosticSeverity::Info,
            ButtonRuntimeViewDiagnosticSeverity::Warning => UiRenderPrimitiveDiagnosticSeverity::Warning,
            ButtonRuntimeViewDiagnosticSeverity::Error => UiRenderPrimitiveDiagnosticSeverity::Error,
        },
        source_map_index: diagnostic.source_map_index,
    }).collect()
}

fn button_colors(theme: &ThemeTokens, button: &ButtonRuntimeView) -> (UiColor, UiColor, UiColor) {
    if button.disabled {
        return (alpha(theme.background_panel, 0.72), alpha(theme.border, 0.72), alpha(theme.foreground_muted, 0.55));
    }
    if button.selected {
        return (alpha(theme.status_input, 0.35), theme.status_input, theme.foreground);
    }
    match (button.style_axes.variant.as_str(), button.style_axes.tone.as_str()) {
        ("primary", "accent") => (theme.accent, theme.accent, theme.foreground),
        ("primary", _) => (theme.foreground_muted, theme.border, theme.background),
        ("secondary", _) => (theme.background_panel, theme.border, theme.foreground),
        ("danger", _) => (alpha(theme.status_error, 0.22), theme.status_error, theme.foreground),
        _ => (theme.background_panel, theme.border, theme.foreground),
    }
}

fn alpha(color: UiColor, alpha: f32) -> UiColor {
    UiColor { a: (color.a * alpha).clamp(0.0, 1.0), ..color }
}

fn paint(color: UiColor) -> UiPaint {
    UiPaint::rgba(color.r, color.g, color.b, color.a)
}

fn paint_from_style(style: &TextStyle) -> UiPaint {
    UiPaint::rgba(style.color[0], style.color[1], style.color[2], style.color[3])
}

fn sort_key(order: u32) -> UiSortKey {
    UiSortKey::new(0, 0, order)
}
