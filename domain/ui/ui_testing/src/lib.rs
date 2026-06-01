//! File: domain/ui/ui_testing/src/lib.rs
//! Crate: ui_testing

use serde::{Deserialize, Serialize};
use ui_artifacts::UiRuntimeArtifact;
use ui_compiler::UiCompiler;
use ui_program::{UiProgram, UiProgramId, UiProgramSourceMapEntry, UiProgramVersion};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiArchitectureFixture {
    pub fixture_id: String,
    pub program: UiProgram,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSourceMapProof {
    pub source_id: String,
    pub target_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiDiagnosticProof {
    pub code: String,
    pub message: String,
}

impl UiArchitectureFixture {
    pub fn minimal(fixture_id: impl Into<String>) -> Self {
        let mut program = UiProgram::new(
            UiProgramId::new("fixture.minimal"),
            UiProgramVersion::new(1),
        );
        program.source_map.push(UiProgramSourceMapEntry {
            source_id: "definition.root".to_owned(),
            target_id: "program.root".to_owned(),
        });
        program
            .graphs
            .visual
            .operators
            .push("label.title".to_owned());
        Self {
            fixture_id: fixture_id.into(),
            program,
        }
    }

    pub fn compile(&self) -> UiRuntimeArtifact {
        UiCompiler.compile(&self.program)
    }

    pub fn source_map_proofs(&self) -> Vec<UiSourceMapProof> {
        self.program
            .source_map
            .iter()
            .map(|entry| UiSourceMapProof {
                source_id: entry.source_id.clone(),
                target_id: entry.target_id.clone(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn architecture_fixtures_compile_to_runtime_artifacts() {
        let fixture = UiArchitectureFixture::minimal("minimal-label");
        let artifact = fixture.compile();

        assert_eq!(fixture.fixture_id, "minimal-label");
        assert_eq!(artifact.manifest.program_id, "fixture.minimal");
        assert_eq!(artifact.tables.visual, ["label.title"]);
    }

    #[test]
    fn diagnostics_records_structural_codes() {
        let proof = UiDiagnosticProof {
            code: "ui.arch.fixture.ready".to_owned(),
            message: "fixture compiled".to_owned(),
        };

        assert_eq!(proof.code, "ui.arch.fixture.ready");
        assert!(proof.message.contains("compiled"));
    }

    #[test]
    fn source_maps_preserve_definition_to_program_targets() {
        let fixture = UiArchitectureFixture::minimal("minimal-label");
        let maps = fixture.source_map_proofs();

        assert_eq!(maps[0].source_id, "definition.root");
        assert_eq!(maps[0].target_id, "program.root");
    }
}
