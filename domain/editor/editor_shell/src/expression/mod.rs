//! File: domain/editor/editor_shell/src/expression/mod.rs
//! Purpose: Expression-frame contracts for shell renderer consumers.

use editor_core::RealityVersion;
use ui_render_data::UiFrame;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionConsumerKind {
    UiRenderer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExpressionSourceReality {
    ObservedShell,
}

#[derive(Debug, Clone)]
pub struct ShellExpressionFrame {
    pub source_reality: ExpressionSourceReality,
    pub consumer_kind: ExpressionConsumerKind,
    pub source_version: RealityVersion,
    pub frame: UiFrame,
}

impl ShellExpressionFrame {
    pub fn new(source_version: RealityVersion, frame: UiFrame) -> Self {
        Self {
            source_reality: ExpressionSourceReality::ObservedShell,
            consumer_kind: ExpressionConsumerKind::UiRenderer,
            source_version,
            frame,
        }
    }

    pub fn into_ui_frame(self) -> UiFrame {
        self.frame
    }
}
