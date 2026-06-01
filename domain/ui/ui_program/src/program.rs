//! File: domain/ui/ui_program/src/program.rs
//! Crate: ui_program

use serde::{Deserialize, Serialize};

use crate::graphs::UiProgramGraphs;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UiProgramId(String);

impl UiProgramId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiProgramVersion(u32);

impl UiProgramVersion {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiProgramDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiProgramSourceMapEntry {
    pub source_id: String,
    pub target_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiProgram {
    pub id: UiProgramId,
    pub version: UiProgramVersion,
    pub graphs: UiProgramGraphs,
    pub source_map: Vec<UiProgramSourceMapEntry>,
    pub diagnostics: Vec<UiProgramDiagnostic>,
}

impl UiProgram {
    pub fn new(id: UiProgramId, version: UiProgramVersion) -> Self {
        Self {
            id,
            version,
            graphs: UiProgramGraphs::default(),
            source_map: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}
