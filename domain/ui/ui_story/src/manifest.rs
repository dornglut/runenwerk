//! Story manifest inputs for domain-owned UI proof runs.

use serde::{Deserialize, Serialize};

use crate::{
    proof::{
        UI_STORY_PROOF_CONTRACT_VERSION, UiStoryProofContract, UiStoryProofDiagnosticExpectation,
    },
    report::UiStoryStageKind,
};

pub const UI_STORY_MANIFEST_SCHEMA_VERSION: u32 = 1;
pub const DIAGNOSTIC_MANIFEST_FIELD_MISSING: &str = "ui.story.manifest.field_missing";
pub const DIAGNOSTIC_MANIFEST_VIEWPORT_INVALID: &str = "ui.story.manifest.viewport_invalid";
pub const DIAGNOSTIC_MANIFEST_REQUIRED_STAGE_MISSING: &str =
    "ui.story.manifest.required_stage_missing";
pub const DIAGNOSTIC_MANIFEST_SCHEMA_UNSUPPORTED: &str = "ui.story.manifest.schema_unsupported";
pub const DIAGNOSTIC_MANIFEST_PROOF_CONTRACT_UNSUPPORTED: &str =
    "ui.story.manifest.proof_contract_unsupported";
pub const DIAGNOSTIC_MANIFEST_EXPECTED_FAILURE_EXPECTATION_MISSING: &str =
    "ui.story.manifest.expected_failure_expectation_missing";

pub type UiStoryDiagnosticExpectation = UiStoryProofDiagnosticExpectation;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UiStoryId(String);

impl UiStoryId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryCategory(String);

impl UiStoryCategory {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStorySourceKind {
    Node,
    Template,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStorySource {
    pub kind: UiStorySourceKind,
    pub path: String,
    pub source_id: String,
}

impl UiStorySource {
    pub fn node(path: impl Into<String>, source_id: impl Into<String>) -> Self {
        Self {
            kind: UiStorySourceKind::Node,
            path: path.into(),
            source_id: source_id.into(),
        }
    }

    pub fn template(path: impl Into<String>, source_id: impl Into<String>) -> Self {
        Self {
            kind: UiStorySourceKind::Template,
            path: path.into(),
            source_id: source_id.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryHostKind {
    Headless,
    Editor,
    Game,
    WorldSpace,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryRoutePolicy {
    VisualUnmappedAllowed,
    AllRoutesMapped,
    NoRoutesAllowed,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryHostProfile {
    pub kind: UiStoryHostKind,
    pub route_policy: UiStoryRoutePolicy,
}

impl UiStoryHostProfile {
    pub fn headless() -> Self {
        Self {
            kind: UiStoryHostKind::Headless,
            route_policy: UiStoryRoutePolicy::VisualUnmappedAllowed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStoryViewportProfile {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}

impl UiStoryViewportProfile {
    pub fn new(id: impl Into<String>, width: u32, height: u32, scale: f32) -> Self {
        Self {
            id: id.into(),
            width,
            height,
            scale,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryThemeProfile(String);

impl UiStoryThemeProfile {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryExpectedVerdict {
    Pass,
    Fail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryExpectedOutcome {
    pub verdict: UiStoryExpectedVerdict,
    pub required_stages: Vec<UiStoryStageKind>,
}

impl UiStoryExpectedOutcome {
    pub fn pass(required_stages: impl Into<Vec<UiStoryStageKind>>) -> Self {
        Self {
            verdict: UiStoryExpectedVerdict::Pass,
            required_stages: required_stages.into(),
        }
    }

    pub fn fail(required_stages: impl Into<Vec<UiStoryStageKind>>) -> Self {
        Self {
            verdict: UiStoryExpectedVerdict::Fail,
            required_stages: required_stages.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryMountPolicy {
    GalleryOnly,
    EligibleWhenPassed,
    Never,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryCompatibilityPolicy {
    ExactVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryMigrationPolicy {
    RejectUnsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiStoryHostInputValue {
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStoryHostInput {
    pub endpoint: String,
    pub value: UiStoryHostInputValue,
}

impl UiStoryHostInput {
    pub fn bool(endpoint: impl Into<String>, value: bool) -> Self {
        Self {
            endpoint: endpoint.into(),
            value: UiStoryHostInputValue::Bool(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStoryManifest {
    pub schema_version: u32,
    pub story_revision: u32,
    pub proof_contract_version: u32,
    pub compatibility_policy: UiStoryCompatibilityPolicy,
    pub migration_policy: UiStoryMigrationPolicy,
    pub story_id: UiStoryId,
    pub category: UiStoryCategory,
    pub title: String,
    pub source: UiStorySource,
    pub program_id: String,
    pub control_package: String,
    pub host_profile: UiStoryHostProfile,
    pub viewport_matrix: Vec<UiStoryViewportProfile>,
    pub theme_profile: UiStoryThemeProfile,
    pub expected: UiStoryExpectedOutcome,
    pub mount_policy: UiStoryMountPolicy,
    #[serde(default)]
    pub diagnostic_expectations: Vec<UiStoryDiagnosticExpectation>,
    #[serde(default)]
    pub host_inputs: Vec<UiStoryHostInput>,
    #[serde(default)]
    pub proof_contract: UiStoryProofContract,
}

impl UiStoryManifest {
    pub fn validate(&self) -> Vec<UiStoryManifestDiagnostic> {
        let mut diagnostics = Vec::new();
        if self.schema_version != UI_STORY_MANIFEST_SCHEMA_VERSION {
            diagnostics.push(UiStoryManifestDiagnostic::new(
                DIAGNOSTIC_MANIFEST_SCHEMA_UNSUPPORTED,
                format!(
                    "schema_version {} is unsupported; expected {}",
                    self.schema_version, UI_STORY_MANIFEST_SCHEMA_VERSION
                ),
            ));
        }
        if self.story_revision == 0 {
            diagnostics.push(UiStoryManifestDiagnostic::new(
                DIAGNOSTIC_MANIFEST_FIELD_MISSING,
                "story_revision must be greater than zero",
            ));
        }
        if self.proof_contract_version != UI_STORY_PROOF_CONTRACT_VERSION
            || self.proof_contract.version != self.proof_contract_version
        {
            diagnostics.push(UiStoryManifestDiagnostic::new(
                DIAGNOSTIC_MANIFEST_PROOF_CONTRACT_UNSUPPORTED,
                format!(
                    "proof contract version {} with contract {} is unsupported; expected {}",
                    self.proof_contract_version,
                    self.proof_contract.version,
                    UI_STORY_PROOF_CONTRACT_VERSION
                ),
            ));
        }
        push_missing(&mut diagnostics, self.story_id.is_empty(), "story_id");
        push_missing(
            &mut diagnostics,
            self.category.as_str().trim().is_empty(),
            "category",
        );
        push_missing(&mut diagnostics, self.title.trim().is_empty(), "title");
        push_missing(
            &mut diagnostics,
            self.source.path.trim().is_empty(),
            "source.path",
        );
        push_missing(
            &mut diagnostics,
            self.source.source_id.trim().is_empty(),
            "source.source_id",
        );
        push_missing(
            &mut diagnostics,
            self.program_id.trim().is_empty(),
            "program_id",
        );
        push_missing(
            &mut diagnostics,
            self.control_package.trim().is_empty(),
            "control_package",
        );
        push_missing(
            &mut diagnostics,
            self.theme_profile.as_str().trim().is_empty(),
            "theme_profile",
        );
        if self.viewport_matrix.is_empty() {
            diagnostics.push(UiStoryManifestDiagnostic::new(
                DIAGNOSTIC_MANIFEST_FIELD_MISSING,
                "viewport_matrix must contain at least one viewport",
            ));
        }
        if self.expected.required_stages.is_empty() {
            diagnostics.push(UiStoryManifestDiagnostic::new(
                DIAGNOSTIC_MANIFEST_REQUIRED_STAGE_MISSING,
                "expected.required_stages must name at least one proof stage",
            ));
        }
        if self.expected.verdict == UiStoryExpectedVerdict::Fail
            && self.diagnostic_expectations.is_empty()
        {
            diagnostics.push(UiStoryManifestDiagnostic::new(
                DIAGNOSTIC_MANIFEST_EXPECTED_FAILURE_EXPECTATION_MISSING,
                "expected-failure stories must declare exact diagnostic_expectations",
            ));
        }
        for expectation in &self.diagnostic_expectations {
            push_missing(
                &mut diagnostics,
                expectation.producer.is_empty(),
                "diagnostic_expectations.producer",
            );
            push_missing(
                &mut diagnostics,
                expectation.proof_key.is_empty(),
                "diagnostic_expectations.proof_key",
            );
            push_missing(
                &mut diagnostics,
                expectation.code.trim().is_empty(),
                "diagnostic_expectations.code",
            );
        }
        for input in &self.host_inputs {
            push_missing(
                &mut diagnostics,
                input.endpoint.trim().is_empty(),
                "host_inputs.endpoint",
            );
        }
        for requirement in &self.proof_contract.requirements {
            push_missing(
                &mut diagnostics,
                requirement.producer.is_empty(),
                "proof_contract.requirements.producer",
            );
            push_missing(
                &mut diagnostics,
                requirement.proof_key.is_empty(),
                "proof_contract.requirements.proof_key",
            );
        }
        for viewport in &self.viewport_matrix {
            if viewport.id.trim().is_empty()
                || viewport.width == 0
                || viewport.height == 0
                || viewport.scale <= 0.0
            {
                diagnostics.push(UiStoryManifestDiagnostic::new(
                    DIAGNOSTIC_MANIFEST_VIEWPORT_INVALID,
                    format!(
                        "viewport {:?} must have id, positive width, positive height, and positive scale",
                        viewport.id
                    ),
                ));
            }
        }
        diagnostics
    }

    pub fn from_ron_str(source: &str) -> Result<Self, UiStoryManifestParseError> {
        let manifest = ron::from_str::<Self>(source).map_err(|error| {
            UiStoryManifestParseError::new(
                DIAGNOSTIC_MANIFEST_SCHEMA_UNSUPPORTED,
                format!("failed to parse story manifest: {error}"),
            )
        })?;

        if let Some(diagnostic) = manifest.validate().into_iter().find(|diagnostic| {
            diagnostic.code == DIAGNOSTIC_MANIFEST_SCHEMA_UNSUPPORTED
                || diagnostic.code == DIAGNOSTIC_MANIFEST_PROOF_CONTRACT_UNSUPPORTED
        }) {
            return Err(UiStoryManifestParseError::new(
                diagnostic.code,
                diagnostic.message,
            ));
        }

        Ok(manifest)
    }

    pub fn to_ron_string_pretty(&self) -> Result<String, UiStoryManifestParseError> {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).map_err(|error| {
            UiStoryManifestParseError::new(
                DIAGNOSTIC_MANIFEST_SCHEMA_UNSUPPORTED,
                format!("failed to serialize story manifest: {error}"),
            )
        })
    }

    pub fn required_stage_kinds(&self) -> Vec<UiStoryStageKind> {
        let mut stages = self.expected.required_stages.clone();
        for stage in self.proof_contract.required_stages() {
            if !stages.contains(&stage) {
                stages.push(stage);
            }
        }
        stages
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryManifestParseError {
    pub code: String,
    pub message: String,
}

impl UiStoryManifestParseError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiStoryManifestDiagnostic {
    pub code: String,
    pub message: String,
}

impl UiStoryManifestDiagnostic {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

fn push_missing(diagnostics: &mut Vec<UiStoryManifestDiagnostic>, missing: bool, field: &str) {
    if missing {
        diagnostics.push(UiStoryManifestDiagnostic::new(
            DIAGNOSTIC_MANIFEST_FIELD_MISSING,
            format!("{field} is required"),
        ));
    }
}
