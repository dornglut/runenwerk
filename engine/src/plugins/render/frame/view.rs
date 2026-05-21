#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedViewKind {
    MainSurface,
    OffscreenProduct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedViewFrame {
    pub view_id: String,
    pub kind: PreparedViewKind,
    pub target_size_px: (u32, u32),
    pub history_signature: Option<String>,
}

impl PreparedViewFrame {
    pub fn new(view_id: impl Into<String>, target_size_px: (u32, u32)) -> Self {
        Self {
            view_id: view_id.into(),
            kind: PreparedViewKind::OffscreenProduct,
            target_size_px,
            history_signature: None,
        }
    }

    pub fn main(target_size_px: (u32, u32)) -> Self {
        Self {
            view_id: "main".to_string(),
            kind: PreparedViewKind::MainSurface,
            target_size_px,
            history_signature: None,
        }
    }

    pub fn offscreen_product(view_id: impl Into<String>, target_size_px: (u32, u32)) -> Self {
        Self {
            view_id: view_id.into(),
            kind: PreparedViewKind::OffscreenProduct,
            target_size_px,
            history_signature: None,
        }
    }

    pub fn with_history_signature(mut self, signature: impl Into<String>) -> Self {
        self.history_signature = Some(signature.into());
        self
    }
}
