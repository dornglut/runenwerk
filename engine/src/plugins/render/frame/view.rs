#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedViewFrame {
    pub view_id: String,
    pub target_size_px: (u32, u32),
}

impl PreparedViewFrame {
    pub fn new(view_id: impl Into<String>, target_size_px: (u32, u32)) -> Self {
        Self {
            view_id: view_id.into(),
            target_size_px,
        }
    }

    pub fn main(target_size_px: (u32, u32)) -> Self {
        // Active runtime path is single-view only for now; "main" is the canonical view id.
        Self::new("main", target_size_px)
    }
}
