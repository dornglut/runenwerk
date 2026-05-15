//! File: domain/ui/ui_runtime/src/tree/node.rs
//! Purpose: Retained UI node and node-kind contracts.

use ui_layout::{LayoutConstraints, SizePolicy};
use ui_math::{Axis, UiInsets, UiRect, UiSize};
use ui_render_data::{UiDrawKey, UiPaint, ViewportSurfaceEmbedSlotId};
use ui_text::TextStyle;
use ui_theme::{ThemeTokens, UiColor};

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
    Popup(PopupNode),
    RadialMenu(RadialMenuNode),
    OverlayAdornment(OverlayAdornmentNode),
    Label(LabelNode),
    Button(ButtonNode),
    TextInput(TextInputNode),
    Toggle(ToggleNode),
    NumericInput(NumericInputNode),
    Tabs(TabsNode),
    Select(SelectNode),
    Table(TableNode),
    Tree(TreeNode),
    Spacer(SpacerNode),
    Divider(DividerNode),
    Image(ImageNode),
    ViewportSurfaceEmbed(ViewportSurfaceEmbedNode),
    Scroll(ScrollNode),
    Stack(StackNode),
    Split(SplitNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialMenuNode {
    pub anchor: RadialMenuAnchor,
    pub layer_order: u32,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub item_size: UiSize,
    pub start_angle_radians: f32,
    pub theme: ThemeTokens,
}

impl RadialMenuNode {
    pub fn anchored_to(anchor: WidgetId, theme: ThemeTokens) -> Self {
        Self {
            anchor: RadialMenuAnchor::Widget(anchor),
            layer_order: 2,
            inner_radius: 16.0,
            outer_radius: 72.0,
            item_size: UiSize::new(42.0, 30.0),
            start_angle_radians: -std::f32::consts::FRAC_PI_2,
            theme,
        }
    }

    pub fn anchored_at(point: ui_math::UiPoint, theme: ThemeTokens) -> Self {
        Self {
            anchor: RadialMenuAnchor::Point(point),
            layer_order: 2,
            inner_radius: 16.0,
            outer_radius: 72.0,
            item_size: UiSize::new(42.0, 30.0),
            start_angle_radians: -std::f32::consts::FRAC_PI_2,
            theme,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadialMenuAnchor {
    Widget(WidgetId),
    Point(ui_math::UiPoint),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlayAdornmentNode {
    pub anchor: WidgetId,
    pub placement: PopupPlacement,
    pub offset: f32,
    pub min_size: UiSize,
    pub stretch_child: bool,
}

impl OverlayAdornmentNode {
    pub fn anchored_inside_top_start(anchor: WidgetId, offset: f32) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::TopStart,
            offset,
            min_size: UiSize::ZERO,
            stretch_child: false,
        }
    }

    pub fn anchored_inside_top_end(anchor: WidgetId, offset: f32) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::InsideTopEnd,
            offset,
            min_size: UiSize::ZERO,
            stretch_child: false,
        }
    }

    pub fn anchored_inside_edge(anchor: WidgetId, side: PopupSide, thickness: f32) -> Self {
        let placement = match side {
            PopupSide::Left => PopupPlacement::InsideLeft,
            PopupSide::Right => PopupPlacement::InsideRight,
            PopupSide::Top => PopupPlacement::InsideTop,
            PopupSide::Bottom => PopupPlacement::InsideBottom,
        };
        let min_size = match side {
            PopupSide::Left | PopupSide::Right => UiSize::new(thickness.max(0.0), 0.0),
            PopupSide::Top | PopupSide::Bottom => UiSize::new(0.0, thickness.max(0.0)),
        };
        Self {
            anchor,
            placement,
            offset: 0.0,
            min_size,
            stretch_child: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PopupNode {
    pub anchor: WidgetId,
    pub placement: PopupPlacement,
    pub dismiss_policy: PopupDismissPolicy,
    pub layer_order: u32,
    pub padding: UiInsets,
    pub gap: f32,
    pub offset: f32,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupPlacement {
    BottomStart,
    RightStart,
    InsideTopEnd,
    InsideBottomStart,
    TopStart,
    InsideLeft,
    InsideRight,
    InsideTop,
    InsideBottom,
    Outside {
        preferred_side: PopupSide,
        align: PopupAlign,
        flip_policy: PopupFlipPolicy,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupSide {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupFlipPolicy {
    None,
    FlipToFit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupDismissPolicy {
    None,
    OutsidePointerDown,
}

impl PopupNode {
    pub fn anchored_outside(
        anchor: WidgetId,
        preferred_side: PopupSide,
        align: PopupAlign,
        flip_policy: PopupFlipPolicy,
        theme: ThemeTokens,
    ) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::Outside {
                preferred_side,
                align,
                flip_policy,
            },
            dismiss_policy: PopupDismissPolicy::OutsidePointerDown,
            layer_order: 2,
            padding: UiInsets::all(theme.spacing.xs),
            gap: theme.spacing.xs,
            offset: theme.spacing.xs,
            min_size: UiSize::new(120.0, 0.0),
            theme,
        }
    }

    pub fn anchored_bottom_start(anchor: WidgetId, theme: ThemeTokens) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::BottomStart,
            dismiss_policy: PopupDismissPolicy::OutsidePointerDown,
            layer_order: 2,
            padding: UiInsets::all(theme.spacing.xs),
            gap: theme.spacing.xs,
            offset: theme.spacing.xs,
            min_size: UiSize::new(120.0, 0.0),
            theme,
        }
    }

    pub fn anchored_top_start(anchor: WidgetId, theme: ThemeTokens) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::TopStart,
            dismiss_policy: PopupDismissPolicy::OutsidePointerDown,
            layer_order: 1,
            padding: UiInsets::all(theme.spacing.xs),
            gap: theme.spacing.xs,
            offset: 0.0,
            min_size: UiSize::new(120.0, 0.0),
            theme,
        }
    }

    pub fn anchored_right_start(anchor: WidgetId, theme: ThemeTokens) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::RightStart,
            dismiss_policy: PopupDismissPolicy::OutsidePointerDown,
            layer_order: 2,
            padding: UiInsets::all(theme.spacing.xs),
            gap: theme.spacing.xs,
            offset: theme.spacing.xs,
            min_size: UiSize::new(120.0, 0.0),
            theme,
        }
    }

    pub fn anchored_inside_top_end(anchor: WidgetId, theme: ThemeTokens) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::InsideTopEnd,
            dismiss_policy: PopupDismissPolicy::OutsidePointerDown,
            layer_order: 0,
            padding: UiInsets::all(0.0),
            gap: 0.0,
            offset: 0.0,
            min_size: UiSize::ZERO,
            theme,
        }
    }

    pub fn anchored_inside_bottom_start(anchor: WidgetId, theme: ThemeTokens) -> Self {
        Self {
            anchor,
            placement: PopupPlacement::InsideBottomStart,
            dismiss_policy: PopupDismissPolicy::OutsidePointerDown,
            layer_order: 1,
            padding: UiInsets::all(theme.spacing.xs),
            gap: theme.spacing.xs,
            offset: theme.spacing.xs,
            min_size: UiSize::new(120.0, 0.0),
            theme,
        }
    }

    pub fn with_dismiss_policy(mut self, dismiss_policy: PopupDismissPolicy) -> Self {
        self.dismiss_policy = dismiss_policy;
        self
    }
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
            padding: UiInsets::all(theme.spacing.xs),
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
    pub selected: bool,
    pub selected_fill: Option<UiColor>,
    pub selected_border: Option<UiColor>,
    pub corner_radius: Option<f32>,
    pub fill_width: bool,
    pub reveal_on_hover_anchor: Option<WidgetId>,
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
            selected: false,
            selected_fill: None,
            selected_border: None,
            corner_radius: None,
            fill_width: false,
            reveal_on_hover_anchor: None,
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
    pub fill_width: bool,
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
            fill_width: false,
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
pub struct SelectNode {
    pub options: Vec<String>,
    pub selected_index: Option<usize>,
    pub placeholder: String,
    pub text_style: TextStyle,
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
    pub enabled: bool,
}

impl SelectNode {
    pub fn new(
        options: impl IntoIterator<Item = impl Into<String>>,
        selected_index: Option<usize>,
        placeholder: impl Into<String>,
        text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let options = options.into_iter().map(Into::into).collect::<Vec<_>>();
        let selected_index = selected_index.filter(|index| *index < options.len());
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let padding = UiInsets::new(
            theme.spacing.sm,
            theme.spacing.xs,
            theme.spacing.sm,
            theme.spacing.xs,
        );
        let width_estimate = options
            .iter()
            .map(|option| option.chars().count() as f32 * text_style.font_size * 0.6)
            .fold(0.0, f32::max)
            + padding.horizontal()
            + line_height;
        Self {
            options,
            selected_index,
            placeholder: placeholder.into(),
            text_style,
            padding,
            min_size: UiSize::new(width_estimate.max(96.0), line_height + padding.vertical()),
            theme,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    pub label: String,
    pub min_width: f32,
}

impl TableColumn {
    pub fn new(label: impl Into<String>, min_width: f32) -> Self {
        Self {
            label: label.into(),
            min_width: min_width.max(24.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableRow {
    pub cells: Vec<String>,
    pub selected: bool,
    pub enabled: bool,
}

impl TableRow {
    pub fn new(cells: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            cells: cells.into_iter().map(Into::into).collect(),
            selected: false,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableNode {
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub text_style: TextStyle,
    pub header_text_style: TextStyle,
    pub row_height: f32,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
}

impl TableNode {
    pub fn new(
        columns: impl IntoIterator<Item = TableColumn>,
        rows: impl IntoIterator<Item = TableRow>,
        text_style: TextStyle,
        header_text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let columns = columns.into_iter().collect::<Vec<_>>();
        let rows = rows.into_iter().collect::<Vec<_>>();
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let row_height = (line_height + theme.spacing.xs * 2.0).max(18.0);
        let width = columns
            .iter()
            .map(|column| column.min_width)
            .sum::<f32>()
            .max(96.0);
        let height = row_height * (rows.len() as f32 + 1.0);
        Self {
            columns,
            rows,
            text_style,
            header_text_style,
            row_height,
            min_size: UiSize::new(width, height),
            theme,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeRow {
    pub label: String,
    pub depth: usize,
    pub expanded: bool,
    pub selected: bool,
    pub has_children: bool,
    pub enabled: bool,
}

impl TreeRow {
    pub fn new(label: impl Into<String>, depth: usize, has_children: bool) -> Self {
        Self {
            label: label.into(),
            depth,
            expanded: true,
            selected: false,
            has_children,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    pub rows: Vec<TreeRow>,
    pub text_style: TextStyle,
    pub row_height: f32,
    pub indent_width: f32,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
}

impl TreeNode {
    pub fn new(
        rows: impl IntoIterator<Item = TreeRow>,
        text_style: TextStyle,
        theme: ThemeTokens,
    ) -> Self {
        let rows = rows.into_iter().collect::<Vec<_>>();
        let line_height = text_style.line_height_or_default(text_style.font_size * 1.2);
        let row_height = (line_height + theme.spacing.xs * 1.5).max(18.0);
        let indent_width = theme.spacing.lg.max(16.0);
        let width = rows
            .iter()
            .map(|row| {
                row.depth as f32 * indent_width
                    + row.label.chars().count() as f32 * text_style.font_size * 0.6
                    + theme.spacing.xl
            })
            .fold(96.0, f32::max);
        Self {
            rows,
            text_style,
            row_height,
            indent_width,
            min_size: UiSize::new(width, row_height),
            theme,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpacerNode {
    pub min_size: UiSize,
}

impl SpacerNode {
    pub const fn new(min_size: UiSize) -> Self {
        Self { min_size }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DividerNode {
    pub axis: Axis,
    pub thickness: f32,
    pub length_policy: SizePolicy,
    pub color: UiColor,
}

impl DividerNode {
    pub fn new(axis: Axis, thickness: f32, length_policy: SizePolicy, color: UiColor) -> Self {
        Self {
            axis,
            thickness: thickness.max(0.0),
            length_policy,
            color,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageNode {
    pub draw_key: UiDrawKey,
    pub uv_rect: UiRect,
    pub tint: UiPaint,
    pub min_size: UiSize,
}

impl ImageNode {
    pub const fn new(
        draw_key: UiDrawKey,
        uv_rect: UiRect,
        tint: UiPaint,
        min_size: UiSize,
    ) -> Self {
        Self {
            draw_key,
            uv_rect,
            tint,
            min_size,
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
    pub axes: ScrollAxes,
    pub bar_thickness: f32,
    pub min_thumb_main_size: f32,
    pub input_policies: ScrollInputPolicies,
    pub theme: ThemeTokens,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAxes {
    Horizontal,
    Vertical,
    Both,
}

impl ScrollAxes {
    pub const fn from_axis(axis: Axis) -> Self {
        match axis {
            Axis::Horizontal => Self::Horizontal,
            Axis::Vertical => Self::Vertical,
        }
    }

    pub const fn contains(self, axis: Axis) -> bool {
        matches!(
            (self, axis),
            (Self::Both, _)
                | (Self::Horizontal, Axis::Horizontal)
                | (Self::Vertical, Axis::Vertical)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollInputPolicy {
    WheelOnly,
    MiddleDragOnly,
    WheelAndMiddleDrag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollInputPolicies {
    pub horizontal: ScrollInputPolicy,
    pub vertical: ScrollInputPolicy,
}

impl ScrollInputPolicies {
    pub const fn new(horizontal: ScrollInputPolicy, vertical: ScrollInputPolicy) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    pub const fn for_axis(self, axis: Axis) -> ScrollInputPolicy {
        match axis {
            Axis::Horizontal => self.horizontal,
            Axis::Vertical => self.vertical,
        }
    }
}

impl Default for ScrollInputPolicies {
    fn default() -> Self {
        Self {
            horizontal: ScrollInputPolicy::WheelAndMiddleDrag,
            vertical: ScrollInputPolicy::WheelOnly,
        }
    }
}

impl ScrollNode {
    pub fn new(theme: ThemeTokens) -> Self {
        Self::vertical(theme)
    }

    pub fn vertical(theme: ThemeTokens) -> Self {
        Self {
            axes: ScrollAxes::Vertical,
            bar_thickness: (theme.spacing.xs * 1.5).clamp(6.0, 18.0),
            min_thumb_main_size: (theme.spacing.lg + theme.spacing.xs).max(18.0),
            input_policies: ScrollInputPolicies::default(),
            theme,
        }
    }

    pub fn horizontal(theme: ThemeTokens) -> Self {
        Self {
            axes: ScrollAxes::Horizontal,
            bar_thickness: (theme.spacing.xs * 1.5).clamp(6.0, 18.0),
            min_thumb_main_size: (theme.spacing.lg + theme.spacing.xs).max(18.0),
            input_policies: ScrollInputPolicies::default(),
            theme,
        }
    }

    pub fn both(theme: ThemeTokens) -> Self {
        Self {
            axes: ScrollAxes::Both,
            bar_thickness: (theme.spacing.xs * 1.5).clamp(6.0, 18.0),
            min_thumb_main_size: (theme.spacing.lg + theme.spacing.xs).max(18.0),
            input_policies: ScrollInputPolicies::default(),
            theme,
        }
    }

    pub fn with_input_policies(mut self, input_policies: ScrollInputPolicies) -> Self {
        self.input_policies = input_policies;
        self
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
