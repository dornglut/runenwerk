//! Backend-neutral primitive generation from artifact-backed runtime views.

use serde::{Deserialize, Serialize};
use ui_math::{UiRect, UiSize};
use ui_render_data::{
    BorderPrimitive, GlyphRunPrimitive, RectPrimitive, UiDrawKey, UiFrame, UiLayer, UiLayerId,
    UiPaint, UiPrimitive, UiSortKey, UiSurface, UiSurfaceId,
};
use ui_runtime_view::{
    ButtonRuntimeHostData, ButtonRuntimeView, ButtonRuntimeViewReport, UiRuntimeViewReport,
};
use ui_text::{
    AtlasTextLayouter, FontAtlasSource, FontId, GlyphRun, TextAlign, TextLayoutRequest,
    TextLayouter, TextOverflow, TextStyle,
};
use ui_theme::{ThemeTokens, UiColor};

pub const DIAGNOSTIC_RUNTIME_VIEW_FAILED: &str = "ui.render_primitives.runtime_view_failed";
pub const DIAGNOSTIC_BUTTON_VIEW_FAILED: &str = "ui.render_primitives.button_view_failed";
pub const DIAGNOSTIC_NO_BUTTONS: &str = "ui.render_primitives.no_buttons";
pub const DIAGNOSTIC_TEXT_LAYOUT_FAILED: &str = "ui.render_primitives.text.layout_failed";
pub const DIAGNOSTIC_EMPTY_GLYPH_RUN: &str = "ui.render_primitives.text.empty_glyph_run";

#[derive(Clone, Debug, PartialEq)]
pub struct UiRenderPrimitiveReport {
    frame: Option<UiFrame>,
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
        Self::from_button_report(button_report, viewport, theme, atlas_source, font_id)
    }

    pub fn from_button_report(
        button_report: ButtonRuntimeViewReport,
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
                diagnostics,
                provenance: Vec::new(),
                button_report,
            };
        }
        if button_report.buttons.is_empty() {
            diagnostics.push(UiRenderPrimitiveDiagnostic::error(
                DIAGNOSTIC_NO_BUTTONS,
                "render primitive generation requires at least one button view",
                None,
            ));
            return Self {
                frame: None,
                diagnostics,
                provenance: Vec::new(),
                button_report,
            };
        }

        let mut layer = UiLayer::new(UiLayerId(0));
        let mut provenance = Vec::new();
        let mut primitive_order = 0_u32;
        let layouter = AtlasTextLayouter;

        for (index, button) in button_report.buttons.iter().enumerate() {
            let style = ResolvedButtonStyle::from_button(theme, button, font_id);
            let layout = ButtonPrimitiveLayout::for_button_index(index, viewport, &style);
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

            match label_glyph_run(
                button,
                layout.text_bounds,
                &style.text_style,
                atlas_source,
                &layouter,
            ) {
                Some(glyph_run) if glyph_run.glyphs.is_empty() => {
                    diagnostics.push(UiRenderPrimitiveDiagnostic::error(
                        DIAGNOSTIC_EMPTY_GLYPH_RUN,
                        format!(
                            "button {} produced an empty glyph run for label text",
                            button.control_id
                        ),
                        source_map_index,
                    ));
                }
                Some(glyph_run) => {
                    push_primitive(
                        &mut layer,
                        &mut provenance,
                        0,
                        source_map_index,
                        UiPrimitive::GlyphRun(GlyphRunPrimitive::new(
                            glyph_run,
                            Some(layout.text_bounds),
                            paint_from_text_style(&style.text_style),
                            UiDrawKey::new(0, Some(style.text_style.font_id.0)),
                            sort_key(primitive_order),
                        )),
                    );
                    primitive_order += 1;
                }
                None => {
                    diagnostics.push(UiRenderPrimitiveDiagnostic::error(
                        DIAGNOSTIC_TEXT_LAYOUT_FAILED,
                        format!(
                            "button {} label could not be shaped with the supplied font atlas",
                            button.control_id
                        ),
                        source_map_index,
                    ));
                }
            }
        }

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == UiRenderPrimitiveDiagnosticSeverity::Error)
        {
            return Self {
                frame: None,
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
        text_style.align = TextAlign::Center;
        text_style.overflow = TextOverflow::Ellipsis;
        text_style.font_size = (theme.typography.body * size_factor).max(1.0);
        text_style.line_height = Some((text_style.font_size * 1.35).max(1.0));

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
struct ButtonPrimitiveLayout {
    bounds: UiRect,
    text_bounds: UiRect,
}

impl ButtonPrimitiveLayout {
    fn for_button_index(index: usize, viewport: UiSize, style: &ResolvedButtonStyle) -> Self {
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

fn label_glyph_run(
    button: &ButtonRuntimeView,
    bounds: UiRect,
    text_style: &TextStyle,
    atlas_source: &dyn FontAtlasSource,
    layouter: &dyn TextLayouter,
) -> Option<GlyphRun> {
    let mut glyph_run = layouter.layout(
        atlas_source,
        TextLayoutRequest {
            text: &button.label,
            style: text_style,
            max_width: Some(bounds.width.max(0.0)),
        },
    )?;
    let align_offset = match text_style.align {
        TextAlign::Start => 0.0,
        TextAlign::Center => ((bounds.width - glyph_run.size.width) * 0.5).max(0.0),
        TextAlign::End => (bounds.width - glyph_run.size.width).max(0.0),
    };
    let vertical_offset =
        vertical_alignment_offset(&glyph_run, text_style, bounds.height, atlas_source);

    for glyph in &mut glyph_run.glyphs {
        glyph.origin.x += bounds.x + align_offset;
        glyph.origin.y += bounds.y + vertical_offset;
    }

    Some(glyph_run)
}

fn vertical_alignment_offset(
    glyph_run: &GlyphRun,
    text_style: &TextStyle,
    bounds_height: f32,
    atlas_source: &dyn FontAtlasSource,
) -> f32 {
    match text_style.vertical_align {
        ui_text::TextVerticalAlign::LineBoxCenter => {
            ((bounds_height - glyph_run.size.height) * 0.5).max(0.0)
        }
        ui_text::TextVerticalAlign::InkBoundsCenter
        | ui_text::TextVerticalAlign::CapHeightCenter => {
            ink_bounds_vertical_offset(glyph_run, text_style, bounds_height, atlas_source)
                .unwrap_or_else(|| ((bounds_height - glyph_run.size.height) * 0.5).max(0.0))
        }
    }
}

fn ink_bounds_vertical_offset(
    glyph_run: &GlyphRun,
    text_style: &TextStyle,
    bounds_height: f32,
    atlas_source: &dyn FontAtlasSource,
) -> Option<f32> {
    let atlas = atlas_source.atlas(text_style.font_id)?;
    let scale = text_style.font_size / atlas.metrics.base_size.max(f32::EPSILON);
    let mut top = f32::INFINITY;
    let mut bottom = f32::NEG_INFINITY;
    for glyph in &glyph_run.glyphs {
        let metrics = atlas
            .glyphs
            .get(&glyph.ch)
            .or_else(|| atlas.glyphs.get(&'?'))?;
        top = top.min(glyph.origin.y - metrics.plane_top * scale);
        bottom = bottom.max(glyph.origin.y - metrics.plane_bottom * scale);
    }
    if !top.is_finite() || !bottom.is_finite() {
        return None;
    }

    Some(bounds_height * 0.5 - (top + bottom) * 0.5)
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
    use std::{collections::HashMap, fs, path::PathBuf};

    use super::*;
    use ui_artifacts::{UiRuntimeArtifact, UiRuntimeArtifactDiagnostic};
    use ui_compiler::UiCompiler;
    use ui_controls::{ControlPackageRegistry, runenwerk_control_package};
    use ui_definition::UiNodeDefinition;
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

    fn compiled_button_artifact(relative_repo_path: &str) -> UiRuntimeArtifact {
        let node = load_node(relative_repo_path);
        let registry = ControlPackageRegistry::new()
            .with_package(runenwerk_control_package())
            .expect("runenwerk controls package should register");

        let formation_report = form_ui_program_report_from_node_with_registry_snapshot(
            "ui_gallery.button.selected",
            "assets.ui_gallery.button.selected",
            &node,
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
