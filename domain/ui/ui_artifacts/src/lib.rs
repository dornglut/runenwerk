//! File: domain/ui/ui_artifacts/src/lib.rs
//! Crate: ui_artifacts

use serde::{Deserialize, Serialize};
use ui_program::{UiProgram, UiProgramSourceMapEntry};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UiRuntimeArtifactId(String);

impl UiRuntimeArtifactId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRuntimeArtifactManifest {
    pub artifact_id: UiRuntimeArtifactId,
    pub program_id: String,
    pub program_version: u32,
    pub cache_key: String,
    pub package_ids: Vec<String>,
    pub schema_ids: Vec<String>,
    pub source_map: Vec<UiProgramSourceMapEntry>,
    pub diagnostics: Vec<String>,
}

impl UiRuntimeArtifactManifest {
    pub fn from_program(program: &UiProgram) -> Self {
        let program_id = program.id.as_str().to_owned();
        let program_version = program.version.value();
        Self {
            artifact_id: UiRuntimeArtifactId::new(format!("{program_id}@{program_version}")),
            program_id: program_id.clone(),
            program_version,
            cache_key: format!("ui-program:{program_id}:{program_version}"),
            package_ids: Vec::new(),
            schema_ids: Vec::new(),
            source_map: program.source_map.clone(),
            diagnostics: program
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code.clone())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRuntimeArtifactTables {
    pub layout: Vec<String>,
    pub state: Vec<String>,
    pub binding: Vec<String>,
    pub interaction: Vec<String>,
    pub visual: Vec<String>,
    pub accessibility: Vec<String>,
    pub inspection: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRuntimeArtifact {
    pub manifest: UiRuntimeArtifactManifest,
    pub tables: UiRuntimeArtifactTables,
}

impl UiRuntimeArtifact {
    pub fn new(manifest: UiRuntimeArtifactManifest, tables: UiRuntimeArtifactTables) -> Self {
        Self { manifest, tables }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_program::{UiProgram, UiProgramId, UiProgramVersion};

    #[test]
    fn artifact_contract_splits_manifest_from_tables() {
        let program = UiProgram::new(UiProgramId::new("hud"), UiProgramVersion::new(7));
        let manifest = UiRuntimeArtifactManifest::from_program(&program);
        let artifact = UiRuntimeArtifact::new(manifest, UiRuntimeArtifactTables::default());

        assert_eq!(artifact.manifest.program_id, "hud");
        assert_eq!(artifact.manifest.program_version, 7);
        assert_eq!(artifact.manifest.cache_key, "ui-program:hud:7");
        assert!(artifact.tables.layout.is_empty());
    }
}
