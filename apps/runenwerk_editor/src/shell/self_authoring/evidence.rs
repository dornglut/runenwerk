//! UI Designer runtime-evidence capture DTOs owned by the self-authoring session.

use super::*;

#[derive(Debug, Clone, Default)]
pub struct EditorLabProductPathEvidenceCapture {
    pub artifacts: Vec<EditorLabEvidenceArtifact>,
    pub performance_baselines: Vec<EditorLabPerformanceBaseline>,
}

impl EditorLabProductPathEvidenceCapture {
    pub fn new(
        artifacts: impl IntoIterator<Item = EditorLabEvidenceArtifact>,
        performance_baselines: impl IntoIterator<Item = EditorLabPerformanceBaseline>,
    ) -> Self {
        Self {
            artifacts: artifacts.into_iter().collect(),
            performance_baselines: performance_baselines.into_iter().collect(),
        }
    }
}
