//! Persisted authored automation scenario V1.
//!
//! The shared schema owns one-product envelope structure, strict resource bounds, and validated
//! relative artifact references. Product-owned persisted step DTOs retain their own semantics and
//! validation, while runtime execution remains owned by product callers and `AutomationSession`.

use std::fmt;

use ron::error::Error as RonError;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::AutomationSession;

pub const AUTOMATION_SCENARIO_V1_ARTIFACT_KIND: &str = "runenwerk.automation.authored-scenario";
pub const AUTOMATION_SCENARIO_V1_SCHEMA_VERSION: u32 = 1;

pub const MAX_AUTOMATION_SCENARIO_BYTES: usize = 1024 * 1024;
pub const MAX_AUTOMATION_SCENARIO_RON_RECURSION_DEPTH: usize = 64;
pub const MAX_AUTOMATION_SCENARIO_STEPS: usize = 4_096;
pub const MAX_AUTOMATION_SCENARIO_PRODUCT_ID_BYTES: usize = 128;
pub const MAX_AUTOMATION_SCENARIO_RELATIVE_ARTIFACT_BYTES: usize = 1_024;
pub const MAX_AUTOMATION_SCENARIO_REPLAY_STEPS: usize = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationScenarioReferenceError {
    detail: String,
}

impl AutomationScenarioReferenceError {
    fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl fmt::Display for AutomationScenarioReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for AutomationScenarioReferenceError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct AutomationScenarioRelativeArtifactRef(String);

impl AutomationScenarioRelativeArtifactRef {
    pub fn new(value: impl Into<String>) -> Result<Self, AutomationScenarioReferenceError> {
        let value = value.into();
        validate_relative_artifact_reference(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AutomationScenarioStepV1<ProductStep> {
    Owner(ProductStep),
    ReplayNormalizedTrace(AutomationScenarioRelativeArtifactRef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedAutomationScenarioV1<ProductStep> {
    product_contract_id: String,
    product_contract_version: u32,
    steps: Vec<AutomationScenarioStepV1<ProductStep>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutomationScenarioStepHistoryRange {
    step_ordinal: usize,
    history_start: usize,
    history_end: usize,
}

impl AutomationScenarioStepHistoryRange {
    pub const fn new(step_ordinal: usize, history_start: usize, history_end: usize) -> Self {
        Self {
            step_ordinal,
            history_start,
            history_end,
        }
    }

    pub const fn step_ordinal(self) -> usize {
        self.step_ordinal
    }

    pub const fn history_start(self) -> usize {
        self.history_start
    }

    pub const fn history_end(self) -> usize {
        self.history_end
    }
}

pub fn execute_automation_scenario_step_with_history<T>(
    step_ordinal: usize,
    session: &mut AutomationSession,
    execute: impl FnOnce(&mut AutomationSession) -> T,
) -> (T, AutomationScenarioStepHistoryRange) {
    let history_start = session.history().len();
    let result = execute(session);
    let history_end = session.history().len();
    (
        result,
        AutomationScenarioStepHistoryRange::new(step_ordinal, history_start, history_end),
    )
}

impl<ProductStep> ImportedAutomationScenarioV1<ProductStep> {
    pub fn product_contract_id(&self) -> &str {
        &self.product_contract_id
    }

    pub const fn product_contract_version(&self) -> u32 {
        self.product_contract_version
    }

    pub fn steps(&self) -> &[AutomationScenarioStepV1<ProductStep>] {
        &self.steps
    }

    pub fn into_steps(self) -> Vec<AutomationScenarioStepV1<ProductStep>> {
        self.steps
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationScenarioExportError {
    InvalidProductContractId(String),
    InvalidProductContractVersion,
    EmptyScenario,
    ResourceLimitExceeded(&'static str),
    TooManyReplaySteps,
    InvalidProductStep(String),
    SerializationFailure(String),
}

impl fmt::Display for AutomationScenarioExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProductContractId(id) => {
                write!(
                    formatter,
                    "invalid automation scenario product contract id: {id}"
                )
            }
            Self::InvalidProductContractVersion => {
                formatter.write_str("automation scenario product contract version must be nonzero")
            }
            Self::EmptyScenario => {
                formatter.write_str("automation scenario must contain at least one step")
            }
            Self::ResourceLimitExceeded(limit) => {
                write!(
                    formatter,
                    "automation scenario resource limit exceeded: {limit}"
                )
            }
            Self::TooManyReplaySteps => formatter
                .write_str("automation scenario V1 permits at most one normalized replay step"),
            Self::InvalidProductStep(detail) => {
                write!(
                    formatter,
                    "invalid product-owned automation scenario step: {detail}"
                )
            }
            Self::SerializationFailure(detail) => {
                write!(
                    formatter,
                    "failed to serialize automation scenario V1: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for AutomationScenarioExportError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationScenarioImportError {
    ArtifactTooLarge,
    ParseFailure(String),
    WrongArtifactKind(String),
    UnsupportedSchemaVersion(u32),
    InvalidProductContractId(String),
    InvalidProductContractVersion,
    ProductContractMismatch {
        expected_id: String,
        expected_version: u32,
        found_id: String,
        found_version: u32,
    },
    EmptyScenario,
    ResourceLimitExceeded(&'static str),
    TooManyReplaySteps,
    UnknownField(String),
    UnknownVariant(String),
    MalformedArtifact(String),
    InvalidArtifactReference(String),
    InvalidProductStep(String),
}

impl fmt::Display for AutomationScenarioImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArtifactTooLarge => {
                formatter.write_str("automation scenario V1 exceeds byte limit")
            }
            Self::ParseFailure(detail) => {
                write!(
                    formatter,
                    "automation scenario V1 RON parse failed: {detail}"
                )
            }
            Self::WrongArtifactKind(found) => {
                write!(
                    formatter,
                    "wrong persisted automation artifact kind: {found}"
                )
            }
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported automation scenario schema version: {version}"
                )
            }
            Self::InvalidProductContractId(id) => {
                write!(
                    formatter,
                    "invalid automation scenario product contract id: {id}"
                )
            }
            Self::InvalidProductContractVersion => {
                formatter.write_str("automation scenario product contract version must be nonzero")
            }
            Self::ProductContractMismatch {
                expected_id,
                expected_version,
                found_id,
                found_version,
            } => write!(
                formatter,
                "automation scenario product contract mismatch: expected {expected_id}/v{expected_version}, found {found_id}/v{found_version}",
            ),
            Self::EmptyScenario => {
                formatter.write_str("automation scenario must contain at least one step")
            }
            Self::ResourceLimitExceeded(limit) => {
                write!(
                    formatter,
                    "automation scenario resource limit exceeded: {limit}"
                )
            }
            Self::TooManyReplaySteps => formatter
                .write_str("automation scenario V1 permits at most one normalized replay step"),
            Self::UnknownField(detail) => {
                write!(formatter, "unknown automation scenario V1 field: {detail}")
            }
            Self::UnknownVariant(detail) => {
                write!(
                    formatter,
                    "unknown automation scenario V1 variant: {detail}"
                )
            }
            Self::MalformedArtifact(detail) => {
                write!(
                    formatter,
                    "malformed automation scenario V1 artifact: {detail}"
                )
            }
            Self::InvalidArtifactReference(detail) => {
                write!(
                    formatter,
                    "invalid automation scenario artifact reference: {detail}"
                )
            }
            Self::InvalidProductStep(detail) => {
                write!(
                    formatter,
                    "invalid product-owned automation scenario step: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for AutomationScenarioImportError {}

#[derive(Debug, Deserialize)]
struct EnvelopeProbe {
    artifact_kind: String,
    schema_version: u32,
}

#[derive(Debug, Deserialize)]
struct ProductContractProbe {
    product_contract_id: String,
    product_contract_version: u32,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct PersistedScenarioV1Ref<'a, ProductStep> {
    artifact_kind: &'static str,
    schema_version: u32,
    product_contract_id: &'a str,
    product_contract_version: u32,
    steps: &'a [AutomationScenarioStepV1<ProductStep>],
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedScenarioV1<ProductStep> {
    artifact_kind: String,
    schema_version: u32,
    product_contract_id: String,
    product_contract_version: u32,
    steps: Vec<PersistedScenarioStepV1<ProductStep>>,
}

#[derive(Debug, Deserialize)]
enum PersistedScenarioStepV1<ProductStep> {
    Owner(ProductStep),
    ReplayNormalizedTrace(String),
}

pub fn export_automation_scenario_v1<ProductStep, ValidateProductStep>(
    product_contract_id: &str,
    product_contract_version: u32,
    steps: &[AutomationScenarioStepV1<ProductStep>],
    validate_product_step: ValidateProductStep,
) -> Result<String, AutomationScenarioExportError>
where
    ProductStep: Serialize,
    ValidateProductStep: Fn(&ProductStep) -> Result<(), String>,
{
    validate_product_contract_export(product_contract_id, product_contract_version)?;
    validate_export_steps(steps, &validate_product_step)?;

    let persisted = PersistedScenarioV1Ref {
        artifact_kind: AUTOMATION_SCENARIO_V1_ARTIFACT_KIND,
        schema_version: AUTOMATION_SCENARIO_V1_SCHEMA_VERSION,
        product_contract_id,
        product_contract_version,
        steps,
    };
    let encoded = ron_options()
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .map_err(|error| AutomationScenarioExportError::SerializationFailure(error.to_string()))?;
    if encoded.len() > MAX_AUTOMATION_SCENARIO_BYTES {
        return Err(AutomationScenarioExportError::ResourceLimitExceeded(
            "scenario_bytes",
        ));
    }
    Ok(encoded)
}

pub fn import_automation_scenario_v1<ProductStep, ValidateProductStep>(
    bytes: &[u8],
    expected_product_contract_id: &str,
    expected_product_contract_version: u32,
    validate_product_step: ValidateProductStep,
) -> Result<ImportedAutomationScenarioV1<ProductStep>, AutomationScenarioImportError>
where
    ProductStep: DeserializeOwned,
    ValidateProductStep: Fn(&ProductStep) -> Result<(), String>,
{
    if bytes.len() > MAX_AUTOMATION_SCENARIO_BYTES {
        return Err(AutomationScenarioImportError::ArtifactTooLarge);
    }
    validate_expected_product_contract(
        expected_product_contract_id,
        expected_product_contract_version,
    )?;

    let source = std::str::from_utf8(bytes)
        .map_err(|error| AutomationScenarioImportError::ParseFailure(error.to_string()))?;
    let options = ron_options();

    let envelope_probe: EnvelopeProbe = options.from_str(source).map_err(classify_ron_error)?;
    validate_envelope_probe(&envelope_probe)?;

    let product_probe: ProductContractProbe =
        options.from_str(source).map_err(classify_ron_error)?;
    validate_product_contract_probe(
        &product_probe,
        expected_product_contract_id,
        expected_product_contract_version,
    )?;

    let persisted: PersistedScenarioV1<ProductStep> =
        options.from_str(source).map_err(classify_ron_error)?;
    validate_persisted_header(
        &persisted,
        expected_product_contract_id,
        expected_product_contract_version,
    )?;

    let steps = convert_imported_steps(persisted.steps, &validate_product_step)?;

    Ok(ImportedAutomationScenarioV1 {
        product_contract_id: persisted.product_contract_id,
        product_contract_version: persisted.product_contract_version,
        steps,
    })
}

fn validate_export_steps<ProductStep, ValidateProductStep>(
    steps: &[AutomationScenarioStepV1<ProductStep>],
    validate_product_step: &ValidateProductStep,
) -> Result<(), AutomationScenarioExportError>
where
    ValidateProductStep: Fn(&ProductStep) -> Result<(), String>,
{
    if steps.is_empty() {
        return Err(AutomationScenarioExportError::EmptyScenario);
    }
    if steps.len() > MAX_AUTOMATION_SCENARIO_STEPS {
        return Err(AutomationScenarioExportError::ResourceLimitExceeded(
            "steps",
        ));
    }

    let mut replay_steps = 0usize;
    for step in steps {
        match step {
            AutomationScenarioStepV1::Owner(step) => validate_product_step(step)
                .map_err(AutomationScenarioExportError::InvalidProductStep)?,
            AutomationScenarioStepV1::ReplayNormalizedTrace(_) => {
                replay_steps += 1;
                if replay_steps > MAX_AUTOMATION_SCENARIO_REPLAY_STEPS {
                    return Err(AutomationScenarioExportError::TooManyReplaySteps);
                }
            }
        }
    }
    Ok(())
}

fn convert_imported_steps<ProductStep, ValidateProductStep>(
    steps: Vec<PersistedScenarioStepV1<ProductStep>>,
    validate_product_step: &ValidateProductStep,
) -> Result<Vec<AutomationScenarioStepV1<ProductStep>>, AutomationScenarioImportError>
where
    ValidateProductStep: Fn(&ProductStep) -> Result<(), String>,
{
    if steps.is_empty() {
        return Err(AutomationScenarioImportError::EmptyScenario);
    }
    if steps.len() > MAX_AUTOMATION_SCENARIO_STEPS {
        return Err(AutomationScenarioImportError::ResourceLimitExceeded(
            "steps",
        ));
    }

    let mut replay_steps = 0usize;
    let mut imported = Vec::with_capacity(steps.len());
    for step in steps {
        match step {
            PersistedScenarioStepV1::Owner(step) => {
                validate_product_step(&step)
                    .map_err(AutomationScenarioImportError::InvalidProductStep)?;
                imported.push(AutomationScenarioStepV1::Owner(step));
            }
            PersistedScenarioStepV1::ReplayNormalizedTrace(reference) => {
                replay_steps += 1;
                if replay_steps > MAX_AUTOMATION_SCENARIO_REPLAY_STEPS {
                    return Err(AutomationScenarioImportError::TooManyReplaySteps);
                }
                let reference =
                    AutomationScenarioRelativeArtifactRef::new(reference).map_err(|error| {
                        AutomationScenarioImportError::InvalidArtifactReference(error.to_string())
                    })?;
                imported.push(AutomationScenarioStepV1::ReplayNormalizedTrace(reference));
            }
        }
    }
    Ok(imported)
}

fn validate_product_contract_export(
    product_contract_id: &str,
    product_contract_version: u32,
) -> Result<(), AutomationScenarioExportError> {
    if !valid_product_contract_id(product_contract_id) {
        return Err(AutomationScenarioExportError::InvalidProductContractId(
            product_contract_id.to_owned(),
        ));
    }
    if product_contract_version == 0 {
        return Err(AutomationScenarioExportError::InvalidProductContractVersion);
    }
    Ok(())
}

fn validate_expected_product_contract(
    product_contract_id: &str,
    product_contract_version: u32,
) -> Result<(), AutomationScenarioImportError> {
    if !valid_product_contract_id(product_contract_id) {
        return Err(AutomationScenarioImportError::InvalidProductContractId(
            product_contract_id.to_owned(),
        ));
    }
    if product_contract_version == 0 {
        return Err(AutomationScenarioImportError::InvalidProductContractVersion);
    }
    Ok(())
}

fn validate_envelope_probe(probe: &EnvelopeProbe) -> Result<(), AutomationScenarioImportError> {
    if probe.artifact_kind != AUTOMATION_SCENARIO_V1_ARTIFACT_KIND {
        return Err(AutomationScenarioImportError::WrongArtifactKind(
            probe.artifact_kind.clone(),
        ));
    }
    if probe.schema_version != AUTOMATION_SCENARIO_V1_SCHEMA_VERSION {
        return Err(AutomationScenarioImportError::UnsupportedSchemaVersion(
            probe.schema_version,
        ));
    }
    Ok(())
}

fn validate_product_contract_probe(
    probe: &ProductContractProbe,
    expected_product_contract_id: &str,
    expected_product_contract_version: u32,
) -> Result<(), AutomationScenarioImportError> {
    if !valid_product_contract_id(&probe.product_contract_id) {
        return Err(AutomationScenarioImportError::InvalidProductContractId(
            probe.product_contract_id.clone(),
        ));
    }
    if probe.product_contract_version == 0 {
        return Err(AutomationScenarioImportError::InvalidProductContractVersion);
    }
    if probe.product_contract_id != expected_product_contract_id
        || probe.product_contract_version != expected_product_contract_version
    {
        return Err(AutomationScenarioImportError::ProductContractMismatch {
            expected_id: expected_product_contract_id.to_owned(),
            expected_version: expected_product_contract_version,
            found_id: probe.product_contract_id.clone(),
            found_version: probe.product_contract_version,
        });
    }
    Ok(())
}

fn validate_persisted_header<ProductStep>(
    persisted: &PersistedScenarioV1<ProductStep>,
    expected_product_contract_id: &str,
    expected_product_contract_version: u32,
) -> Result<(), AutomationScenarioImportError> {
    if persisted.artifact_kind != AUTOMATION_SCENARIO_V1_ARTIFACT_KIND {
        return Err(AutomationScenarioImportError::WrongArtifactKind(
            persisted.artifact_kind.clone(),
        ));
    }
    if persisted.schema_version != AUTOMATION_SCENARIO_V1_SCHEMA_VERSION {
        return Err(AutomationScenarioImportError::UnsupportedSchemaVersion(
            persisted.schema_version,
        ));
    }
    if persisted.product_contract_id != expected_product_contract_id
        || persisted.product_contract_version != expected_product_contract_version
    {
        return Err(AutomationScenarioImportError::ProductContractMismatch {
            expected_id: expected_product_contract_id.to_owned(),
            expected_version: expected_product_contract_version,
            found_id: persisted.product_contract_id.clone(),
            found_version: persisted.product_contract_version,
        });
    }
    Ok(())
}

fn valid_product_contract_id(value: &str) -> bool {
    if value.is_empty()
        || value.len() > MAX_AUTOMATION_SCENARIO_PRODUCT_ID_BYTES
        || !value.is_ascii()
    {
        return false;
    }

    value.split('.').all(|segment| {
        let mut bytes = segment.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };
        (first.is_ascii_lowercase() || first.is_ascii_digit())
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    })
}

fn validate_relative_artifact_reference(
    value: &str,
) -> Result<(), AutomationScenarioReferenceError> {
    if value.is_empty() {
        return Err(AutomationScenarioReferenceError::new(
            "relative artifact reference is empty",
        ));
    }
    if value.len() > MAX_AUTOMATION_SCENARIO_RELATIVE_ARTIFACT_BYTES {
        return Err(AutomationScenarioReferenceError::new(
            "relative artifact reference exceeds byte limit",
        ));
    }
    if value.starts_with('/') || value.starts_with('\\') {
        return Err(AutomationScenarioReferenceError::new(
            "absolute artifact references are unsupported",
        ));
    }
    if value.contains('\\') {
        return Err(AutomationScenarioReferenceError::new(
            "artifact reference must use forward slashes",
        ));
    }
    if value.contains(':') {
        return Err(AutomationScenarioReferenceError::new(
            "artifact reference contains platform prefix syntax",
        ));
    }
    if value.chars().any(char::is_control) {
        return Err(AutomationScenarioReferenceError::new(
            "artifact reference contains control characters",
        ));
    }
    if value
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(AutomationScenarioReferenceError::new(
            "artifact reference contains an invalid path segment",
        ));
    }
    Ok(())
}

fn ron_options() -> ron::Options {
    ron::Options::default().with_recursion_limit(MAX_AUTOMATION_SCENARIO_RON_RECURSION_DEPTH)
}

fn classify_ron_error(error: ron::error::SpannedError) -> AutomationScenarioImportError {
    let detail = error.to_string();
    match error.code {
        RonError::ExceededRecursionLimit => {
            AutomationScenarioImportError::ResourceLimitExceeded("ron_recursion_depth")
        }
        RonError::NoSuchStructField { .. } => AutomationScenarioImportError::UnknownField(detail),
        RonError::NoSuchEnumVariant { .. } => AutomationScenarioImportError::UnknownVariant(detail),
        RonError::MissingStructField { .. } | RonError::DuplicateStructField { .. } => {
            AutomationScenarioImportError::MalformedArtifact(detail)
        }
        _ => AutomationScenarioImportError::ParseFailure(detail),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    enum TestProductStep {
        SetValue { value: u32 },
    }

    const PRODUCT_ID: &str = "runenwerk.test.automation";
    const PRODUCT_VERSION: u32 = 1;

    fn validate_step(_step: &TestProductStep) -> Result<(), String> {
        Ok(())
    }

    fn owner_step() -> AutomationScenarioStepV1<TestProductStep> {
        AutomationScenarioStepV1::Owner(TestProductStep::SetValue { value: 7 })
    }

    #[test]
    fn authored_step_history_range_captures_exact_session_records() {
        use super::super::{AutomationSessionId, AutomationStepResult, InputSourceId};

        let mut session =
            AutomationSession::new(AutomationSessionId::new(77), InputSourceId::new(77_001));
        let (result, range) =
            execute_automation_scenario_step_with_history(3, &mut session, |session| {
                assert_eq!(
                    session.assert_observation(&1_u32, |value| *value == 1),
                    AutomationStepResult::AssertionPassed
                );
                session.assert_observation(&2_u32, |value| *value == 2)
            });

        assert_eq!(result, AutomationStepResult::AssertionPassed);
        assert_eq!(range.step_ordinal(), 3);
        assert_eq!(range.history_start(), 0);
        assert_eq!(range.history_end(), 2);
        assert_eq!(session.history().len(), 2);
    }

    #[test]
    fn strict_scenario_round_trip_preserves_product_and_replay_steps() {
        let trace_ref =
            AutomationScenarioRelativeArtifactRef::new("traces/input.ron").expect("valid ref");
        let encoded = export_automation_scenario_v1(
            PRODUCT_ID,
            PRODUCT_VERSION,
            &[
                AutomationScenarioStepV1::ReplayNormalizedTrace(trace_ref),
                owner_step(),
            ],
            validate_step,
        )
        .expect("scenario should export");

        let imported = import_automation_scenario_v1::<TestProductStep, _>(
            encoded.as_bytes(),
            PRODUCT_ID,
            PRODUCT_VERSION,
            validate_step,
        )
        .expect("scenario should import");

        assert_eq!(imported.product_contract_id(), PRODUCT_ID);
        assert_eq!(imported.product_contract_version(), PRODUCT_VERSION);
        assert_eq!(imported.steps().len(), 2);
    }

    #[test]
    fn product_identity_and_version_are_strict() {
        assert!(matches!(
            export_automation_scenario_v1(
                "Runenwerk.Editor",
                PRODUCT_VERSION,
                &[owner_step()],
                validate_step,
            ),
            Err(AutomationScenarioExportError::InvalidProductContractId(_))
        ));
        assert!(matches!(
            export_automation_scenario_v1(PRODUCT_ID, 0, &[owner_step()], validate_step),
            Err(AutomationScenarioExportError::InvalidProductContractVersion)
        ));
    }

    #[test]
    fn scenario_requires_steps_and_permits_at_most_one_replay() {
        assert!(matches!(
            export_automation_scenario_v1::<TestProductStep, _>(
                PRODUCT_ID,
                PRODUCT_VERSION,
                &[],
                validate_step,
            ),
            Err(AutomationScenarioExportError::EmptyScenario)
        ));

        let first = AutomationScenarioRelativeArtifactRef::new("a.ron").unwrap();
        let second = AutomationScenarioRelativeArtifactRef::new("b.ron").unwrap();
        assert!(matches!(
            export_automation_scenario_v1::<TestProductStep, _>(
                PRODUCT_ID,
                PRODUCT_VERSION,
                &[
                    AutomationScenarioStepV1::ReplayNormalizedTrace(first),
                    AutomationScenarioStepV1::ReplayNormalizedTrace(second),
                ],
                validate_step,
            ),
            Err(AutomationScenarioExportError::TooManyReplaySteps)
        ));
    }

    #[test]
    fn relative_artifact_reference_rejects_escape_and_platform_prefix_forms() {
        for value in [
            "",
            "/absolute.ron",
            "../escape.ron",
            "traces/../escape.ron",
            "traces//input.ron",
            "traces/./input.ron",
            "C:/input.ron",
            "traces\\input.ron",
        ] {
            assert!(
                AutomationScenarioRelativeArtifactRef::new(value).is_err(),
                "{value:?} should be rejected"
            );
        }
    }

    #[test]
    fn import_rejects_wrong_kind_future_version_product_mismatch_and_unknown_step() {
        let encoded = export_automation_scenario_v1(
            PRODUCT_ID,
            PRODUCT_VERSION,
            &[owner_step()],
            validate_step,
        )
        .unwrap();

        let wrong_kind = encoded.replace(
            AUTOMATION_SCENARIO_V1_ARTIFACT_KIND,
            "runenwerk.automation.normalized-replay-trace",
        );
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                wrong_kind.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::WrongArtifactKind(_))
        ));

        let future = encoded.replace("schema_version: 1", "schema_version: 2");
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                future.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::UnsupportedSchemaVersion(2))
        ));

        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                encoded.as_bytes(),
                "runenwerk.other.automation",
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::ProductContractMismatch { .. })
        ));

        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                encoded.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION + 1,
                validate_step,
            ),
            Err(AutomationScenarioImportError::ProductContractMismatch { .. })
        ));

        let unknown = encoded.replace("SetValue", "UnknownStep");
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                unknown.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::UnknownVariant(_))
        ));
    }

    #[test]
    fn import_rejects_unknown_envelope_and_product_fields() {
        let encoded = export_automation_scenario_v1(
            PRODUCT_ID,
            PRODUCT_VERSION,
            &[owner_step()],
            validate_step,
        )
        .expect("scenario should export");

        let unknown_envelope =
            encoded.replace("schema_version: 1,", "schema_version: 1,\n    extra: 1,");
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                unknown_envelope.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::UnknownField(_))
        ));

        let unknown_product_field =
            encoded.replace("value: 7,", "value: 7,\n                extra: 1,");
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                unknown_product_field.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::UnknownField(_))
        ));
    }

    #[test]
    fn product_step_validation_runs_on_export_and_import() {
        let reject_large_value = |step: &TestProductStep| match step {
            TestProductStep::SetValue { value } if *value > 10 => {
                Err("value must not exceed 10".to_owned())
            }
            _ => Ok(()),
        };
        let invalid_step = AutomationScenarioStepV1::Owner(TestProductStep::SetValue { value: 11 });
        assert!(matches!(
            export_automation_scenario_v1(
                PRODUCT_ID,
                PRODUCT_VERSION,
                &[invalid_step],
                reject_large_value,
            ),
            Err(AutomationScenarioExportError::InvalidProductStep(_))
        ));

        let encoded = export_automation_scenario_v1(
            PRODUCT_ID,
            PRODUCT_VERSION,
            &[AutomationScenarioStepV1::Owner(TestProductStep::SetValue {
                value: 11,
            })],
            validate_step,
        )
        .expect("permissive exporter should encode fixture");
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                encoded.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                reject_large_value,
            ),
            Err(AutomationScenarioImportError::InvalidProductStep(_))
        ));
    }

    #[test]
    fn import_rejects_raw_trace_artifact_kind_before_scenario_fields_are_required() {
        let raw_trace_header = r#"(
            artifact_kind: "runenwerk.automation.normalized-replay-trace",
            schema_version: 1,
        )"#;
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                raw_trace_header.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::WrongArtifactKind(_))
        ));
    }

    #[test]
    fn import_enforces_empty_step_replay_count_and_reference_rules() {
        let empty = format!(
            r#"(
                artifact_kind: "{AUTOMATION_SCENARIO_V1_ARTIFACT_KIND}",
                schema_version: 1,
                product_contract_id: "{PRODUCT_ID}",
                product_contract_version: 1,
                steps: [],
            )"#
        );
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                empty.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::EmptyScenario)
        ));

        let two_replays = format!(
            r#"(
                artifact_kind: "{AUTOMATION_SCENARIO_V1_ARTIFACT_KIND}",
                schema_version: 1,
                product_contract_id: "{PRODUCT_ID}",
                product_contract_version: 1,
                steps: [
                    ReplayNormalizedTrace("a.ron"),
                    ReplayNormalizedTrace("b.ron"),
                ],
            )"#
        );
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                two_replays.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::TooManyReplaySteps)
        ));

        let escaping_reference = format!(
            r#"(
                artifact_kind: "{AUTOMATION_SCENARIO_V1_ARTIFACT_KIND}",
                schema_version: 1,
                product_contract_id: "{PRODUCT_ID}",
                product_contract_version: 1,
                steps: [ReplayNormalizedTrace("../escape.ron")],
            )"#
        );
        assert!(matches!(
            import_automation_scenario_v1::<TestProductStep, _>(
                escaping_reference.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::InvalidArtifactReference(_))
        ));
    }

    #[test]
    fn import_enforces_scenario_byte_and_step_limits() {
        let oversized = vec![b'x'; MAX_AUTOMATION_SCENARIO_BYTES + 1];
        assert_eq!(
            import_automation_scenario_v1::<TestProductStep, _>(
                &oversized,
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::ArtifactTooLarge)
        );

        let too_many_steps = (0..=MAX_AUTOMATION_SCENARIO_STEPS)
            .map(|_| "Owner(SetValue(value: 1))")
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            r#"(
                artifact_kind: "{AUTOMATION_SCENARIO_V1_ARTIFACT_KIND}",
                schema_version: 1,
                product_contract_id: "{PRODUCT_ID}",
                product_contract_version: 1,
                steps: [{too_many_steps}],
            )"#
        );
        assert_eq!(
            import_automation_scenario_v1::<TestProductStep, _>(
                source.as_bytes(),
                PRODUCT_ID,
                PRODUCT_VERSION,
                validate_step,
            ),
            Err(AutomationScenarioImportError::ResourceLimitExceeded(
                "steps"
            ))
        );
    }
}
