//! File: domain/ui/ui_runtime/src/tree/node.rs
//! Purpose: Retained UI node and node-kind contracts.

use ui_layout::{LayoutConstraints, SizePolicy};
use ui_math::{Axis, UiInsets, UiSize};
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
    Stack(StackNode),
    Split(SplitNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PanelNode {
    pub padding: UiInsets,
    pub min_size: UiSize,
    pub theme: ThemeTokens,
}

impl PanelNode {
    pub fn new(theme: ThemeTokens) -> Self {
        Self {
            padding: UiInsets::ZERO,
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
        Self {
            label: label.into(),
            text_style,
            padding: UiInsets::all(8.0),
            min_size: UiSize::new(48.0, 28.0),
            theme,
            enabled: true,
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
