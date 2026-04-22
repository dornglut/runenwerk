//! File: domain/ui/ui_widgets/src/numeric_input.rs
//! Purpose: Numeric input widget constructor.

use crate::{NumericInputNode, UiNode, UiNodeKind, WidgetId};
use ui_text::TextStyle;
use ui_theme::ThemeTokens;

pub fn numeric_input(
    id: WidgetId,
    value: f64,
    step: f64,
    min: Option<f64>,
    max: Option<f64>,
    precision: u8,
    text_style: TextStyle,
    theme: ThemeTokens,
) -> UiNode {
    UiNode::new(
        id,
        UiNodeKind::NumericInput(NumericInputNode::new(
            value, step, min, max, precision, text_style, theme,
        )),
    )
}
