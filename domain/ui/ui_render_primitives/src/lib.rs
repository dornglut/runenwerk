//! Backend-neutral primitive generation from artifact-backed runtime views.

use serde::{Deserialize, Serialize};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_runtime_view::{
    ButtonRuntimeHostData, ButtonRuntimeView, ButtonRuntimeViewReport, RuntimeControlView,
    UiRuntimeViewReport,
};
use ui_text::{
    AtlasTextLayouter, FontAtlasSource, FontId, TextBlock, TextBlockId, TextBlockLayoutRequest,
    TextBlockLayoutResult, TextDirectionPolicy, TextHeightConstraint, TextHorizontalAlign,
    TextLayoutPolicy, TextLayouter, TextLineHeightPolicy, TextRun, TextRunId, TextSemanticRole,
    TextStyle, TextVerticalAlign, TextWhitespacePolicy, TextWidthConstraint, TextWrapPolicy,
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
            return Self {
                frame: None,
                labels: Vec::new(),
                hit_targets: Vec::new(),
                diagnostics: vec![UiRenderPrimitiveDiagnostic::error(
                    DIAGNOSTIC_RUNTIME_VIEW_FAILED,
                    "render primitive generation refused failed runtime view report",
                    None,
                )],
                provenance: Vec::new(),
                button_report: ButtonRuntimeViewReport::default(),
            };
        }

        let button_report =
            ButtonRuntimeViewReport::from_runtime_view_report_with_host_data(report, host_data);
        let labels = label_views_from_runtime_view_report(report);
        Self::from_button_and_label_views(
            button_report,
            labels,
            viewport,
            theme,
            atlas_source,
            font_id,
        )
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
        let mut diagnostics =
            primitive_diagnostics_from_button_report(button_report.diagnostics.as_slice());
        if !button_report.passed() {
            diagnostics.push(UiRenderPrimitiveDiagnostic::error(
                DIAGNOSTIC_BUTTON_VIEW_FAILED,
                "render primitive generation refused failed button runtime view report",
                None,
            ));
            return Self {
                frame: None,
                labels: label_views.iter().map(|label| label.text.clone()).collect(),
                hit_targets: Vec::new(),
                diagnostics,
                provenance: Vec::new(),
                button_report,
            };
        }

        let mut layer = UiLayer::new(UiLayerId(0));
        let mut provenance = Vec::new();
        let mut hit_targets = Vec::new();
        let mut labels = Vec::new();
        let mut primitive_order = 0_u32;
        let mut stack_index = 0_usize;
        let layouter = AtlasTextLayouter;
        let label_style = ResolvedLabelStyle::from_theme(theme, font_id);

        for control in ControlPrimitiveView::from_views(&label_views, &button_report.buttons) {
            match control {
                ControlPrimitiveView::Label(label) => {
                    labels.push(label.text.clone());
                    let layout =
                        LabelPrimitiveLayout::for_stack_index(stack_index, viewport, &label_style);
                    let text_layout = text_layout_for_string(
                        &label.text,
                        layout.bounds,
                        &label_style.text_style,
                        atlas_source,
                        &layouter,
                        TextBlockId(stack_index as u64 + 1),
                        TextHorizontalAlign::Start,
                    );
                    push_text_primitive(
                        &mut layer,
                        &mut provenance,
                        &mut diagnostics,
                        &mut primitive_order,
                        label.source_map_index,
                        &label.control_id,
                        text_layout,
                        layout.bounds,
                        &label_style.text_style,
                    );
                    stack_index += 1;
                }
                ControlPrimitiveView::Button(button) => {
                    labels.push(button.label.clone());
                    let style = ResolvedButtonStyle::from_button(theme, button, font_id);
                    let layout =
                        ButtonPrimitiveLayout::for_stack_index(stack_index, viewport, &style);
                    let source_map_index = button
                        .source_map_indexes
                        .property
                        .or(button.source_map_indexes.control);

                    push_primitive(
                        &mut layer,
                        &mut provenance,
                        0,
                        source_map_index,
                        UiPrimitive::Rect(RectPrimitive::new(
                            layout.bounds,
                            style.radius,
                            paint_from_color(style.background),
                            UiDrawKey::new(1, None),
                            sort_key(primitive_order),
                        )),
                    );
                    primitive_order += 1;

                    push_primitive(
                        &mut layer,
                        &mut provenance,
                        0,
                        source_map_index,
                        UiPrimitive::Border(BorderPrimitive::new(
                            layout.bounds,
                            style.radius,
                            style.border_width,
                            paint_from_color(style.border),
                            UiDrawKey::new(2, None),
                            sort_key(primitive_order),
                        )),
                    );
                    primitive_order += 1;

                    let text_layout = text_layout_for_string(
                        &button.label,
                        layout.text_bounds,
                        &style.text_style,
                        atlas_source,
                        &layouter,
                        TextBlockId(stack_index as u64 + 1),
                        TextHorizontalAlign::Center,
                    );
                    push_text_primitive(
                        &mut layer,
                        &mut provenance,
                        &mut diagnostics,
                        &mut primitive_order,
                        source_map_index,
                        &button.control_id,
                        text_layout,
                        layout.text_bounds,
                        &style.text_style,
                    );

                    if button.route.is_some() {
                        hit_targets.push(UiRenderHitTarget::new(
                            button.control_id.clone(),
                            button.label.clone(),
                            button.route.clone(),
                            button.capability.clone(),
                            layout.bounds,
                            !button.disabled,
                        ));
                    }
                    stack_index += 1;
                }
            }
        }

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiRenderPrimitiveDiagnosticSeverity::Error)
        {
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

    pub fn passed(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != UiRenderPrimitiveDiagnosticSeverity::Error)
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRenderPrimitiveDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: UiRenderPrimitiveDiagnosticSeverity,
    #[serde(default)]
    pub source_map_index: Option<u32>,
}

impl UiRenderPrimitiveDiagnostic {
    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        source_map_index: Option<u32>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: UiRenderPrimitiveDiagnosticSeverity::Error,
            source_map_index,
        }
    }

    pub fn warning(
        code: impl Into<String>,
        message: impl Into<String>,
        source_map_index: Option<u32>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            severity: UiRenderPrimitiveDiagnosticSeverity::Warning,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlPrimitiveView<'a> {
    Label(&'a LabelRuntimePrimitiveView),
    Button(&'a ButtonRuntimeView),
}

impl<'a> ControlPrimitiveView<'a> {
    fn from_views(
        labels: &'a [LabelRuntimePrimitiveView],
        buttons: &'a [ButtonRuntimeView],
    ) -> Vec<Self> {
        let mut controls = Vec::with_capacity(labels.len() + buttons.len());
        controls.extend(labels.iter().map(Self::Label));
        controls.extend(buttons.iter().map(Self::Button));
        controls.sort_by_key(|control| match control {
            Self::Label(label) => label.ordinal,
            Self::Button(button) => button
                .source_map_indexes
                .control
                .or(button.source_map_indexes.property)
                .unwrap_or(u32::MAX) as usize,
        });
        controls
    }
}

fn label_views_from_runtime_view_report(
    report: &UiRuntimeViewReport,
) -> Vec<LabelRuntimePrimitiveView> {
    report
        .view
        .controls()
        .enumerate()
        .filter_map(|(ordinal, control)| label_view_from_control(control, ordinal))
        .collect()
}

fn label_view_from_control(
    control: &RuntimeControlView,
    ordinal: usize,
) -> Option<LabelRuntimePrimitiveView> {
    if control.control.node.control_kind.as_str() != LABEL_CONTROL_KIND_ID {
        return None;
    }
    let Some(property) = control.property() else {
        return Some(LabelRuntimePrimitiveView {
            control_id: control.control_id().as_str().to_owned(),
            text: String::new(),
            source_map_index: control.control.source_map_index,
            ordinal,
        });
    };
    let text = property
        .snapshot
        .get("text")
        .or_else(|| property.snapshot.get("label"))
        .and_then(ui_schema::UiSchemaValue::as_str)
        .unwrap_or_default()
        .to_owned();
    Some(LabelRuntimePrimitiveView {
        control_id: control.control_id().as_str().to_owned(),
        text,
        source_map_index: property
            .source_map_index
            .or(control.control.source_map_index),
        ordinal,
    })
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedLabelStyle {
    text_style: TextStyle,
    horizontal_padding: f32,
    height: f32,
    stack_gap: f32,
}

impl ResolvedLabelStyle {
    fn from_theme(theme: &ThemeTokens, font_id: FontId) -> Self {
        let mut text_style = theme.body_text_style(font_id);
        text_style.font_size = theme.typography.body.max(1.0);
        text_style.line_height =
            TextLineHeightPolicy::Absolute((text_style.font_size * 1.35).max(1.0));
        text_style.color = [
            theme.foreground.r,
            theme.foreground.g,
            theme.foreground.b,
            theme.foreground.a,
        ];
        let font_size = text_style.font_size;
        Self {
            text_style,
            horizontal_padding: theme.spacing.lg.max(1.0),
            height: (font_size * 1.6).max(font_size + 2.0),
            stack_gap: theme.spacing.md.max(1.0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ResolvedButtonStyle {
    background: UiColor,
    border: UiColor,
    border_width: f32,
    radius: f32,
    text_style: TextStyle,
    horizontal_padding: f32,
    height: f32,
    min_width: f32,
    stack_gap: f32,
}

impl ResolvedButtonStyle {
    fn from_button(theme: &ThemeTokens, button: &ButtonRuntimeView, font_id: FontId) -> Self {
        let density_factor = match button.style_axes.density.as_str() {
            "compact" => 0.8,
            "spacious" => 1.25,
            _ => 1.0,
        };
        let size_factor = match button.style_axes.size.as_str() {
            "xs" => 0.85,
            "sm" => 0.92,
            "lg" => 1.18,
            _ => 1.0,
        };

        let mut text_style = theme.body_text_style(font_id);
        text_style.font_size = (theme.typography.body * size_factor).max(1.0);
        text_style.line_height =
            TextLineHeightPolicy::Absolute((text_style.font_size * 1.35).max(1.0));

        let (background, border, text) = button_colors(theme, button);
        text_style.color = [text.r, text.g, text.b, text.a];
        let font_size = text_style.font_size;

        Self {
            background,
            border,
            border_width: theme.border_width,
            radius: theme.radius.md,
            text_style,
            horizontal_padding: (theme.spacing.xl * density_factor).max(1.0),
            height: (font_size * 1.6 + theme.spacing.lg * density_factor * 2.0)
                .max(font_size + 2.0),
            min_width: (theme.spacing.xl * density_factor * 7.5).max(font_size * 5.0),
            stack_gap: (theme.spacing.lg * density_factor).max(1.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct LabelPrimitiveLayout {
    bounds: UiRect,
}

impl LabelPrimitiveLayout {
    fn for_stack_index(index: usize, viewport: UiSize, style: &ResolvedLabelStyle) -> Self {
        let x = style.horizontal_padding.max(1.0);
        let y = style.horizontal_padding.max(1.0) + index as f32 * (style.height + style.stack_gap);
        let bounds = UiRect::new(x, y, (viewport.width - x * 2.0).max(1.0), style.height);
        Self { bounds }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ButtonPrimitiveLayout {
    bounds: UiRect,
    text_bounds: UiRect,
}

impl ButtonPrimitiveLayout {
    fn for_stack_index(index: usize, viewport: UiSize, style: &ResolvedButtonStyle) -> Self {
        let x = style.horizontal_padding.max(1.0);
        let y = style.horizontal_padding.max(1.0) + index as f32 * (style.height + style.stack_gap);
        let max_width = (viewport.width - x * 2.0).max(style.min_width);
        let bounds = UiRect::new(x, y, style.min_width.min(max_width), style.height);
        let text_bounds = UiRect::new(
            bounds.x + style.horizontal_padding,
            bounds.y,
            (bounds.width - style.horizontal_padding * 2.0).max(0.0),
            bounds.height,
        );
        Self {
            bounds,
            text_bounds,
        }
    }
}

fn text_layout_for_string(
    text: &str,
    bounds: UiRect,
    text_style: &TextStyle,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
    block_id: TextBlockId,
    horizontal_align: TextHorizontalAlign,
) -> TextBlockLayoutResult {
    let block = TextBlock::new(block_id, text_style.clone())
        .with_run(TextRun::new(TextRunId(1), text).with_semantic_role(TextSemanticRole::Label))
        .with_semantic_role(TextSemanticRole::Label)
        .with_layout(TextLayoutPolicy {
            width_constraint: TextWidthConstraint::Exact(bounds.width.max(0.0)),
            height_constraint: TextHeightConstraint::Unconstrained,
            wrap: TextWrapPolicy::NoWrap,
            whitespace: TextWhitespacePolicy::Preserve,
            horizontal_align,
            vertical_align: TextVerticalAlign::Start,
            overflow: ui_text::TextOverflowPolicy::Clip,
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
    primitive_order: &mut u32,
    source_map_index: Option<u32>,
    control_id: &str,
    text_layout: TextBlockLayoutResult,
    clip_bounds: UiRect,
    text_style: &TextStyle,
) {
    if text_layout.glyph_count == 0 {
        diagnostics.push(UiRenderPrimitiveDiagnostic::error(
            DIAGNOSTIC_EMPTY_GLYPH_RUN,
            format!("control {control_id} produced an empty glyph run"),
            source_map_index,
        ));
    } else if text_layout
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == ui_text::TextDiagnosticSeverity::Error)
    {
        diagnostics.push(UiRenderPrimitiveDiagnostic::error(
            DIAGNOSTIC_TEXT_LAYOUT_FAILED,
            format!("control {control_id} text could not be shaped with the supplied font atlas"),
            source_map_index,
        ));
    } else {
        push_primitive(
            layer,
            provenance,
            0,
            source_map_index,
            UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
                text_layout,
                Some(clip_bounds),
                paint_from_text_style(text_style),
                UiDrawKey::new(0, Some(text_style.font_id.0)),
                sort_key(*primitive_order),
            )),
        );
        *primitive_order += 1;
    }
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

fn button_colors(theme: &ThemeTokens, button: &ButtonRuntimeView) -> (UiColor, UiColor, UiColor) {
    if button.disabled {
        return (
            alpha(theme.background_panel, 0.72),
            alpha(theme.border, 0.72),
            alpha(theme.foreground_muted, 0.55),
        );
    }
    if button.selected {
        return (
            alpha(theme.status_input, 0.35),
            theme.status_input,
            theme.foreground,
        );
    }

    match (
        button.style_axes.variant.as_str(),
        button.style_axes.tone.as_str(),
    ) {
        ("primary", "accent") => (theme.accent, theme.accent, theme.foreground),
        ("primary", _) => (theme.foreground_muted, theme.border, theme.background),
        ("secondary", _) => (theme.background_panel, theme.border, theme.foreground),
        ("ghost", _) => (
            alpha(theme.background_panel, 0.0),
            theme.border,
            theme.foreground,
        ),
        ("danger", _) => (
            alpha(theme.status_error, 0.22),
            theme.status_error,
            theme.foreground,
        ),
        _ => (theme.background_panel, theme.border, theme.foreground),
    }
}

fn alpha(color: UiColor, alpha: f32) -> UiColor {
    UiColor {
        a: (color.a * alpha).clamp(0.0, 1.0),
        ..color
    }
}

fn paint_from_color(color: UiColor) -> UiPaint {
    UiPaint::rgba(color.r, color.g, color.b, color.a)
}

fn paint_from_text_style(style: &TextStyle) -> UiPaint {
    UiPaint::rgba(
        style.color[0],
        style.color[1],
        style.color[2],
        style.color[3],
    )
}

fn sort_key(primitive_order: u32) -> UiSortKey {
    UiSortKey::new(0, 0, primitive_order)
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

fn primitive_diagnostics_from_button_report(
    diagnostics: &[ui_runtime_view::ButtonRuntimeViewDiagnostic],
) -> Vec<UiRenderPrimitiveDiagnostic> {
    diagnostics
        .iter()
        .map(|diagnostic| UiRenderPrimitiveDiagnostic {
            code: diagnostic.code.clone(),
            message: diagnostic.message.clone(),
            severity: match diagnostic.severity {
                ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Info => {
                    UiRenderPrimitiveDiagnosticSeverity::Info
                }
                ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Warning => {
                    UiRenderPrimitiveDiagnosticSeverity::Warning
                }
                ui_runtime_view::ButtonRuntimeViewDiagnosticSeverity::Error => {
                    UiRenderPrimitiveDiagnosticSeverity::Error
                }
            },
            source_map_index: diagnostic.source_map_index,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        fs,
        path::PathBuf,
    };

    use super::*;
    use ui_artifacts::{UiRuntimeArtifact, UiRuntimeArtifactDiagnostic};
    use ui_compiler::UiCompiler;
    use ui_controls::{
        BUTTON_CONTROL_KIND_ID as TEST_BUTTON_CONTROL_KIND_ID, ControlPackageRegistry,
        LABEL_CONTROL_KIND_ID as TEST_LABEL_CONTROL_KIND_ID, runenwerk_control_package,
    };
    use ui_definition::{
        AuthoredControlAccessibilityDefinition, AuthoredControlKindId, AuthoredControlValue,
        AuthoredId, AuthoredRouteId, UiNodeDefinition,
    };
    use ui_program_lowering::form_ui_program_report_from_node_with_registry_snapshot;
    use ui_runtime_view::UiRuntimeView;
    use ui_text::{FontFaceMetrics, GlyphMetrics, MsdfFontAtlas};

    #[test]
    fn render_primitives_emit_glyph_run_for_selected_button() {
        let report = selected_button_render_primitive_report();

        assert!(report.passed(), "{:?}", report.diagnostics());
        assert!(
            report
                .button_report()
                .buttons
                .iter()
                .any(|button| button.selected)
        );

        let frame = report.frame().expect("render primitive frame should exist");
        assert!(has_primitive(frame, |primitive| matches!(
            primitive,
            UiPrimitive::Rect(_)
        )));
        assert!(has_primitive(frame, |primitive| matches!(
            primitive,
            UiPrimitive::Border(_)
        )));
        assert!(has_primitive(frame, |primitive| matches!(
            primitive,
            UiPrimitive::GlyphRun(_)
        )));
        assert_eq!(frame.surfaces[0].size, UiSize::new(320.0, 200.0));
        assert_eq!(primitive_count(frame), report.provenance().len());
        assert_eq!(report.hit_targets().len(), 1);
        assert_eq!(report.hit_targets()[0].label(), "Selected");
    }

    #[test]
    fn mixed_label_and_button_screen_emits_label_glyphs_button_primitives_and_hit_targets() {
        let report = primitive_report_for_node(UiNodeDefinition::Column {
            id: AuthoredId::new("mixed.root"),
            children: vec![
                label_control("mixed.title", "Runenwerk UI Counter Runtime"),
                button_control("mixed.increment", "Increment", "counter.increment"),
            ],
        });

        assert!(report.passed(), "{:?}", report.diagnostics());
        assert!(
            report
                .labels()
                .iter()
                .any(|label| label == "Runenwerk UI Counter Runtime")
        );
        assert!(report.labels().iter().any(|label| label == "Increment"));
        assert_eq!(report.button_report().buttons.len(), 1);
        assert_eq!(report.hit_targets().len(), 1);
        assert_eq!(report.hit_targets()[0].route(), Some("counter.increment"));
        let frame = report.frame().expect("mixed screen should produce frame");
        assert!(has_primitive(frame, |primitive| matches!(
            primitive,
            UiPrimitive::GlyphRun(_)
        )));
        assert!(has_primitive(frame, |primitive| matches!(
            primitive,
            UiPrimitive::Rect(_)
        )));
        assert!(has_primitive(frame, |primitive| matches!(
            primitive,
            UiPrimitive::Border(_)
        )));
        assert!(report.hit_targets()[0].bounds().width > 1.0);
        assert!(report.hit_targets()[0].bounds().height > 1.0);
    }

    #[test]
    fn label_only_screen_does_not_fail_with_no_buttons() {
        let report = primitive_report_for_node(UiNodeDefinition::Column {
            id: AuthoredId::new("labels.root"),
            children: vec![label_control("labels.title", "Only labels")],
        });

        assert!(report.passed(), "{:?}", report.diagnostics());
        assert!(report.frame().is_some());
        assert!(report.button_report().buttons.is_empty());
        assert!(report.hit_targets().is_empty());
        assert!(report.labels().iter().any(|label| label == "Only labels"));
    }

    #[test]
    fn render_primitives_fail_closed_when_runtime_view_fails() {
        let mut artifact = compiled_button_artifact("assets/ui_gallery/button/selected.ron");
        artifact
            .manifest
            .push_diagnostic(UiRuntimeArtifactDiagnostic::error(
                "fixture.artifact.error",
                "fixture artifact error",
            ));
        let runtime_report = UiRuntimeView::from_artifact_report(&artifact);
        let report = UiRenderPrimitiveReport::from_runtime_view_report_with_host_data(
            &runtime_report,
            UiSize::new(320.0, 200.0),
            &ThemeTokens::default(),
            &TestFontAtlasSource::default(),
            TEST_FONT_ID,
            &ButtonRuntimeHostData::default(),
        );

        assert!(!runtime_report.passed());
        assert!(!report.passed());
        assert!(report.frame().is_none());
        assert_has_diagnostic(&report, DIAGNOSTIC_RUNTIME_VIEW_FAILED);
    }

    fn selected_button_render_primitive_report() -> UiRenderPrimitiveReport {
        let artifact = compiled_button_artifact("assets/ui_gallery/button/selected.ron");
        let runtime_report = UiRuntimeView::from_artifact_report(&artifact);
        assert!(
            runtime_report.passed(),
            "{:?}",
            runtime_report.view.diagnostics
        );
        UiRenderPrimitiveReport::from_runtime_view_report_with_host_data(
            &runtime_report,
            UiSize::new(320.0, 200.0),
            &ThemeTokens::default(),
            &TestFontAtlasSource::default(),
            TEST_FONT_ID,
            &ButtonRuntimeHostData::new().with_bool("ui_gallery.button.selected.active", true),
        )
    }

    fn primitive_report_for_node(node: UiNodeDefinition) -> UiRenderPrimitiveReport {
        let artifact = compiled_artifact_from_node("mixed.screen", "mixed.source", &node);
        let runtime_report = UiRuntimeView::from_artifact_report(&artifact);
        assert!(
            runtime_report.passed(),
            "{:?}",
            runtime_report.view.diagnostics
        );
        UiRenderPrimitiveReport::from_runtime_view_report(
            &runtime_report,
            UiSize::new(320.0, 200.0),
            &ThemeTokens::default(),
            &TestFontAtlasSource::default(),
            TEST_FONT_ID,
        )
    }

    fn label_control(id: &str, text: &str) -> UiNodeDefinition {
        let mut properties = BTreeMap::new();
        properties.insert(
            "text".to_owned(),
            AuthoredControlValue::String(text.to_owned()),
        );
        UiNodeDefinition::Control {
            id: AuthoredId::new(id),
            kind: AuthoredControlKindId::new(TEST_LABEL_CONTROL_KIND_ID),
            properties,
            bindings: BTreeMap::new(),
            route: None,
            accessibility: Some(AuthoredControlAccessibilityDefinition {
                role: "label".to_owned(),
                label: Some(text.to_owned()),
            }),
            children: Vec::new(),
        }
    }

    fn button_control(id: &str, label: &str, route: &str) -> UiNodeDefinition {
        let mut properties = BTreeMap::new();
        properties.insert(
            "label".to_owned(),
            AuthoredControlValue::String(label.to_owned()),
        );
        UiNodeDefinition::Control {
            id: AuthoredId::new(id),
            kind: AuthoredControlKindId::new(TEST_BUTTON_CONTROL_KIND_ID),
            properties,
            bindings: BTreeMap::new(),
            route: Some(AuthoredRouteId::new(route)),
            accessibility: Some(AuthoredControlAccessibilityDefinition {
                role: "button".to_owned(),
                label: Some(label.to_owned()),
            }),
            children: Vec::new(),
        }
    }

    fn compiled_button_artifact(relative_repo_path: &str) -> UiRuntimeArtifact {
        let node = load_node(relative_repo_path);
        compiled_artifact_from_node(
            "ui_gallery.button.selected",
            "assets.ui_gallery.button.selected",
            &node,
        )
    }

    fn compiled_artifact_from_node(
        program_id: &str,
        source_id: &str,
        node: &UiNodeDefinition,
    ) -> UiRuntimeArtifact {
        let registry = ControlPackageRegistry::new()
            .with_package(runenwerk_control_package())
            .expect("runenwerk controls package should register");

        let formation_report = form_ui_program_report_from_node_with_registry_snapshot(
            program_id,
            source_id,
            node,
            &registry.snapshot(),
        );
        assert!(
            formation_report.passed(),
            "{:?}",
            formation_report.diagnostics
        );

        let report = UiCompiler.compile_report(&formation_report.program);
        assert!(
            report.passed(),
            "{:?}",
            report.artifact.manifest.diagnostics
        );
        report.artifact
    }

    fn has_primitive(frame: &UiFrame, predicate: impl Fn(&UiPrimitive) -> bool) -> bool {
        frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .flat_map(|layer| layer.primitives.iter())
            .any(predicate)
    }

    fn primitive_count(frame: &UiFrame) -> usize {
        frame
            .surfaces
            .iter()
            .flat_map(|surface| surface.layers.iter())
            .map(|layer| layer.primitives.len())
            .sum()
    }

    fn assert_has_diagnostic(report: &UiRenderPrimitiveReport, code: &str) {
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code == code),
            "{:?}",
            report.diagnostics()
        );
    }

    fn load_node(relative_repo_path: &str) -> UiNodeDefinition {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("ui_render_primitives should live under domain/ui/ui_render_primitives")
            .to_path_buf();
        let path = repo_root.join(relative_repo_path);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {:?}: {error}", path));
        ron::from_str(&source).expect("fixture should parse as UiNodeDefinition")
    }

    const TEST_FONT_ID: FontId = FontId(77);

    #[derive(Clone, Debug)]
    struct TestFontAtlasSource {
        atlas: MsdfFontAtlas,
    }

    impl Default for TestFontAtlasSource {
        fn default() -> Self {
            let glyphs = (32_u8..=126_u8)
                .map(|byte| {
                    (
                        char::from(byte),
                        GlyphMetrics {
                            advance: 8.0,
                            plane_left: 0.0,
                            plane_top: 10.0,
                            plane_right: 7.0,
                            plane_bottom: -2.0,
                            atlas_left: 0.0,
                            atlas_top: 0.0,
                            atlas_right: 1.0,
                            atlas_bottom: 1.0,
                        },
                    )
                })
                .collect::<HashMap<_, _>>();

            Self {
                atlas: MsdfFontAtlas {
                    font_id: TEST_FONT_ID,
                    texture_width: 1,
                    texture_height: 1,
                    metrics: FontFaceMetrics {
                        ascender: 10.0,
                        descender: -2.0,
                        line_height: 14.0,
                        base_size: 14.0,
                    },
                    glyphs,
                },
            }
        }
    }

    impl FontAtlasSource for TestFontAtlasSource {
        fn atlas(&self, font_id: FontId) -> Option<&MsdfFontAtlas> {
            (font_id == TEST_FONT_ID).then_some(&self.atlas)
        }
    }
}
