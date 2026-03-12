#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PipelinePhase {
    Compute,
    Render,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineSpecialization {
    pub phase: PipelinePhase,
    pub label: String,
}

impl PipelineSpecialization {
    pub fn compute(label: impl Into<String>) -> Self {
        Self {
            phase: PipelinePhase::Compute,
            label: label.into(),
        }
    }

    pub fn render(label: impl Into<String>) -> Self {
        Self {
            phase: PipelinePhase::Render,
            label: label.into(),
        }
    }
}
