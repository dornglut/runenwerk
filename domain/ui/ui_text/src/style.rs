//! File: domain/ui/ui_text/src/style.rs
//! Purpose: Text styling contracts.

use crate::FontId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    NoWrap,
    WordWrap,
    CharacterWrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    Clip,
    Ellipsis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextVerticalAlign {
    LineBoxCenter,
    InkBoundsCenter,
    CapHeightCenter,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub font_id: FontId,
    pub font_size: f32,
    pub color: [f32; 4],
    pub line_height: Option<f32>,
    pub align: TextAlign,
    pub vertical_align: TextVerticalAlign,
    pub wrap: TextWrap,
    pub overflow: TextOverflow,
}

impl TextStyle {
    pub fn line_height_or_default(&self, default: f32) -> f32 {
        self.line_height.unwrap_or(default)
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_id: FontId(0),
            font_size: 14.0,
            color: [0.92, 0.93, 0.95, 1.0],
            line_height: None,
            align: TextAlign::Start,
            vertical_align: TextVerticalAlign::LineBoxCenter,
            wrap: TextWrap::NoWrap,
            overflow: TextOverflow::Clip,
        }
    }
}
