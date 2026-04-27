//! File: domain/ui/ui_runtime/src/tree/node.rs
//! Purpose: Retained UI node and node-kind contracts.

use ui_layout::{LayoutConstraints, SizePolicy};
use ui_math::{Axis, UiInsets, UiSize};
use ui_render_data::ViewportSurfaceEmbedSlotId;
use ui_text::TextStyle;
use ui_theme::ThemeTokens;

use crate::WidgetId;

#[derive(Debug, Clone, PartialEq)]
pub struct UiNode {
    pub id: WidgetId,
    pub kind: UiNodeKind,
    pub children: Vec<UiNode>,
}

impl UiNode {
    pub fn new(id: WidgetId, kind: UiNodeKind) -> Self {
        Self {
            id,
            kind,
            children: Vec::new(),
        }
    }

    pub fn with_children(id: WidgetId, kind: UiNodeKind, children: Vec<UiNode>) -> Self {
        Self { id, kind, children }
    }

    pub fn push_child(&mut self, child: UiNode) {
        self.children.push(child);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiNodeKind {
    Panel(PanelNode),
    Label(LabelNode),
    Button(ButtonNode),
    TextInput(TextInputNode),
    Toggle(ToggleNode),
    NumericInput(NumericInputNode),
    Tabs(TabsNode),
    ViewportSurfaceEmbed(ViewportSurfaceEmbedNode),
    Scroll(ScrollNode),
    Stack(StackNode),
    Split(SplitNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanelNode {
    pub padding: UiInsets,
    pub gap: f32,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
}

impl PanelNode {
    pub fn new(theme: ThemeTokens) -> Self {
        Self {
            padding: UiInsets::all(theme.spacing.sm),
            gap: theme.spacing.xs,
            min_size: UiSize::ZERO,
            theme,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelNode {
    pub text: String,
    pub text_style: TextStyle,
    pub constraints: LayoutConstraints,
}

impl LabelNode {
    pub fn new(text: impl Into<String>, text_style: TextStyle) -> Self {
        Self {
            text: text.into(),
            text_style,
            constraints: LayoutConstraints::loose(UiSize::new(f32::MAX, f32::MAX)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ButtonNode {
    pub label: String,
    pub text_style: TextStyle,
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
    pub enabled: bool,
}

impl ButtonNode {
    pub fn new(label: impl Into<String>, text_style: TextStyle, theme: ThemeTokens) -> Self {
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let padding = UiInsets::new(
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
        );
        Self {
            label: label.into(),
            text_style,
            padding,
            min_size: UiSize::new(
                (theme.spacing.xl * 2.0).max(32.0),
                (line_height + padding.vertical()).max(theme.spacing.lg + theme.spacing.sm),
            ),
            theme,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextInputNode {
    pub value: String,
    pub placeholder: String,
    pub text_style: TextStyle,
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
    pub editable: bool,
}

impl TextInputNode {
    pub fn new(
        value: impl Into<String>,
        placeholder: impl Into<String>,
        text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let padding = UiInsets::new(
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
        );
        Self {
            value: value.into(),
            placeholder: placeholder.into(),
            text_style,
            padding,
            min_size: UiSize::new(
                (theme.spacing.xl * 3.0).max(72.0),
                line_height + padding.vertical(),
            ),
            theme,
            editable: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ToggleNode {
    pub label: String,
    pub text_style: TextStyle,
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
    pub checked: bool,
    pub enabled: bool,
}

impl ToggleNode {
    pub fn new(
        label: impl Into<String>,
        checked: bool,
        text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let padding = UiInsets::new(
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
        );
        let min_toggle = (line_height + padding.vertical()).max(18.0);
        Self {
            label: label.into(),
            text_style,
            padding,
            min_size: UiSize::new((min_toggle * 2.0).max(48.0), min_toggle),
            theme,
            checked,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumericInputNode {
    pub value: f64,
    pub step: f64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub precision: u8,
    pub text_style: TextStyle,
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
    pub enabled: bool,
}

impl NumericInputNode {
    pub fn new(
        value: f64,
        step: f64,
        min: Option<f64>,
        max: Option<f64>,
        precision: u8,
        text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let padding = UiInsets::new(
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
        );
        Self {
            value,
            step: step.abs().max(0.000_001),
            min,
            max,
            precision,
            text_style,
            padding,
            min_size: UiSize::new(
                (theme.spacing.xl * 2.5).max(64.0),
                line_height + padding.vertical(),
            ),
            theme,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TabsNode {
    pub labels: Vec<String>,
    pub selected_index: usize,
    pub text_style: TextStyle,
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
}

impl TabsNode {
    pub fn new(
        labels: impl IntoIterator<Item = impl Into<String>>,
        selected_index: usize,
        text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let labels = labels.into_iter().map(Into::into).collect::<Vec<_>>();
        let clamped_index = if labels.is_empty() {
            0
        } else {
            selected_index.min(labels.len() - 1)
        };
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let padding = UiInsets::new(
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
        );
        let width_estimate = labels
            .iter()
            .map(|label| {
                label.chars().count() as f32 * text_style.font_size * 0.65 + padding.horizontal()
            })
            .sum::<f32>()
            .max((theme.spacing.xl * 3.0).max(72.0));
        Self {
            labels,
            selected_index: clamped_index,
            text_style,
            padding,
            min_size: UiSize::new(width_estimate, line_height + padding.vertical()),
            theme,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewportSurfaceEmbedNode {
    pub viewport_id: u64,
    pub slot: ViewportSurfaceEmbedSlotId,
    pub min_size: UiSize,
}

impl ViewportSurfaceEmbedNode {
    pub fn new(viewport_id: u64, slot: ViewportSurfaceEmbedSlotId) -> Self {
        Self {
            viewport_id,
            slot,
            min_size: UiSize::new(64.0, 64.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollNode {
    pub axis: Axis,
    pub bar_thickness: f32,
    pub min_thumb_main_size: f32,
    pub theme: ThemeTokens,
}

impl ScrollNode {
    pub fn new(theme: ThemeTokens) -> Self {
        Self::vertical(theme)
    }

    pub fn vertical(theme: ThemeTokens) -> Self {
        Self {
            axis: Axis::Vertical,
            bar_thickness: (theme.spacing.xs * 1.5).clamp(6.0, 18.0),
            min_thumb_main_size: (theme.spacing.lg + theme.spacing.xs).max(18.0),
            theme,
        }
    }

    pub fn horizontal(theme: ThemeTokens) -> Self {
        Self {
            axis: Axis::Horizontal,
            bar_thickness: (theme.spacing.xs * 1.5).clamp(6.0, 18.0),
            min_thumb_main_size: (theme.spacing.lg + theme.spacing.xs).max(18.0),
            theme,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StackNode {
    pub axis: Axis,
    pub gap: f32,
    pub padding: UiInsets,
    pub child_main_policies: Vec<SizePolicy>,
}

impl StackNode {
    pub fn vertical(gap: f32) -> Self {
        Self {
            axis: Axis::Vertical,
            gap,
            padding: UiInsets::ZERO,
            child_main_policies: Vec::new(),
        }
    }

    pub fn horizontal(gap: f32) -> Self {
        Self {
            axis: Axis::Horizontal,
            gap,
            padding: UiInsets::ZERO,
            child_main_policies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitNode {
    pub axis: Axis,
    pub ratio: f32,
    pub gap: f32,
}

impl SplitNode {
    pub fn new(axis: Axis, ratio: f32, gap: f32) -> Self {
        Self { axis, ratio, gap }
    }
}
