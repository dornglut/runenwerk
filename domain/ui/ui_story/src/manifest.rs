//! Story manifest inputs for domain-owned UI proof runs.

use serde::{Deserialize, Serialize};

use crate::report::{UiStoryDiagnosticSeverity, UiStoryStageKind};

pub const DIAGNOSTIC_MANIFEST_FIELD_MISSING: &str = "ui.story.manifest.field_missing";
pub const DIAGNOSTIC_MANIFEST_VIEWPORT_INVALID: &str = "ui.story.manifest.viewport_invalid";
pub const DIAGNOSTIC_MANIFEST_REQUIRED_STAGE_MISSING: &str =
    "ui.story.manifest.required_stage_missing";

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
pub struct UiStoryDiagnosticExpectation {
    pub code: String,
    pub stage: UiStoryStageKind,
    pub severity: UiStoryDiagnosticSeverity,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiStoryManifest {
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
}

impl UiStoryManifest {
    pub fn validate(&self) -> Vec<UiStoryManifestDiagnostic> {
        let mut diagnostics = Vec::new();
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

    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
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
