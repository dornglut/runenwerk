#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SdfSample {
    pub distance: f32,
}

impl SdfSample {
    pub fn new(distance: f32) -> Self {
        Self { distance }
    }
}
