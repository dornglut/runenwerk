//! File: domain/editor/editor_shell/src/story_lab/readiness.rs
//! Purpose: Editor tool-surface product readiness classification.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolSurfaceReadiness {
    Product,
    FallbackOnly,
    Diagnostic,
    HiddenUntilProductized,
}

impl ToolSurfaceReadiness {
    pub const fn requires_native_evidence(self) -> bool {
        matches!(self, Self::Product)
    }

    pub const fn visible_in_product(self) -> bool {
        matches!(self, Self::Product | Self::FallbackOnly | Self::Diagnostic)
    }
}
