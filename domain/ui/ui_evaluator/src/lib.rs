//! File: domain/ui/ui_evaluator/src/lib.rs
//! Crate: ui_evaluator

use serde::{Deserialize, Serialize};
use ui_artifacts::UiRuntimeArtifact;
use ui_state::UiStateModel;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiOutput {
    pub visual_packets: Vec<String>,
    pub diagnostics: Vec<String>,
    pub inspection_reports: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct UiEvaluator;

impl UiEvaluator {
    pub fn evaluate(&self, artifact: &UiRuntimeArtifact, _state: &mut UiStateModel) -> UiOutput {
        UiOutput {
            visual_packets: artifact.tables.visual.clone(),
            diagnostics: artifact.manifest.diagnostics.clone(),
            inspection_reports: artifact.tables.inspection.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_artifacts::{UiRuntimeArtifact, UiRuntimeArtifactManifest, UiRuntimeArtifactTables};
    use ui_program::{UiProgram, UiProgramId, UiProgramVersion};

    #[test]
    fn evaluator_contract_reads_artifact_tables_without_host_truth() {
        let program = UiProgram::new(UiProgramId::new("hud"), UiProgramVersion::new(1));
        let manifest = UiRuntimeArtifactManifest::from_program(&program);
        let artifact = UiRuntimeArtifact::new(
            manifest,
            UiRuntimeArtifactTables {
                visual: vec!["label.score".to_owned()],
                inspection: vec!["node.score".to_owned()],
                ..UiRuntimeArtifactTables::default()
            },
        );
        let output = UiEvaluator.evaluate(&artifact, &mut UiStateModel::default());

        assert_eq!(output.visual_packets, ["label.score"]);
        assert_eq!(output.inspection_reports, ["node.score"]);
    }
}
