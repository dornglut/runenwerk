//! Persisted normalized automation replay-trace V1.
//!
//! Storage schema and untrusted-input policy live here. Runtime automation traces and RunenInput
//! types remain separate semantic owners.

use std::collections::{HashMap, HashSet};
use std::fmt;

use ron::error::Error as RonError;
use serde::{Deserialize, Serialize};

use runen_input::{
    AnalogMeasurement, CapabilityKnowledge, ContactId, ContactPhase, ContactPresence,
    CoordinateSpace, DeliveryRole, DigitalState, EvidenceStatus, InputContext, InputDeviceId,
    InputObservation, InputObservationGroup, InputSourceId, InputToolKind, MeasurementDomain,
    ObservationOrigin, PhysicalTabletControls, Point2, PointerButton, PointerButtonInput,
    RelativeMotionUnit, ScrollDelta, ScrollDomain, ScrollInput, ScrollPhase, SourceTime,
    SourceTimeUnit, StylusTilt, TabletCapabilities, TabletObservation, ToolId, Vector2,
};

use super::{AutomationInputTrace, AutomationInputTraceFrame};

pub const AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND: &str =
    "runenwerk.automation.normalized-replay-trace";
pub const AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION: u32 = 1;

pub const MAX_ARTIFACT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_RON_RECURSION_DEPTH: usize = 64;
pub const MAX_FRAMES: usize = 16_384;
pub const MAX_GROUPS_PER_FRAME: usize = 4_096;
pub const MAX_TOTAL_GROUPS: usize = 65_536;
pub const MAX_OBSERVATIONS_PER_GROUP: usize = 256;
pub const MAX_SOURCES: usize = 1_024;
pub const MAX_DEVICES_PER_SOURCE: usize = 1_024;
pub const MAX_CONTACTS_PER_CONTEXT: usize = 4_096;
pub const MAX_TOOLS_PER_CONTEXT: usize = 4_096;
pub const MAX_METADATA_STRING_BYTES: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomationInputTraceRecordingWitness {
    RecordedSourcesPristineAtCaptureStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutomationInputTraceCaptureHostClass {
    Headless,
    NativeWindow,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AutomationInputTraceProvenance {
    pub runenwerk_revision: Option<String>,
    pub runen_input_revision: Option<String>,
    pub capture_host: Option<AutomationInputTraceCaptureHostClass>,
    pub label: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedAutomationInputTraceV1 {
    trace: AutomationInputTrace,
    recording_witness: AutomationInputTraceRecordingWitness,
    provenance: Option<AutomationInputTraceProvenance>,
}

impl ImportedAutomationInputTraceV1 {
    pub fn trace(&self) -> &AutomationInputTrace {
        &self.trace
    }

    pub fn into_trace(self) -> AutomationInputTrace {
        self.trace
    }

    pub fn recording_witness(&self) -> AutomationInputTraceRecordingWitness {
        self.recording_witness
    }

    pub fn provenance(&self) -> Option<&AutomationInputTraceProvenance> {
        self.provenance.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationInputTraceExportError {
    UnsupportedTraceShape(String),
    UnframedTrailingGroups,
    ResourceLimitExceeded(&'static str),
    SerializationFailure(String),
}

impl fmt::Display for AutomationInputTraceExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedTraceShape(detail) => {
                write!(formatter, "unsupported normalized trace shape: {detail}")
            }
            Self::UnframedTrailingGroups => {
                formatter.write_str("persisted V1 rejects unframed trailing input groups")
            }
            Self::ResourceLimitExceeded(limit) => {
                write!(formatter, "persisted V1 resource limit exceeded: {limit}")
            }
            Self::SerializationFailure(detail) => {
                write!(
                    formatter,
                    "failed to serialize persisted V1 trace: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for AutomationInputTraceExportError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutomationInputTraceImportError {
    ArtifactTooLarge,
    ParseFailure(String),
    WrongArtifactKind(String),
    UnsupportedSchemaVersion(u32),
    MalformedArtifact(String),
    UnknownField(String),
    UnknownVariant(String),
    ResourceLimitExceeded(&'static str),
    InvalidIdentityReference(String),
    UnsupportedTraceShape(String),
    InvalidNormalizedInput(String),
    UnsupportedRecordingWitness,
}

impl fmt::Display for AutomationInputTraceImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArtifactTooLarge => {
                formatter.write_str("persisted V1 artifact exceeds byte limit")
            }
            Self::ParseFailure(detail) => {
                write!(formatter, "persisted V1 RON parse failed: {detail}")
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
                    "unsupported persisted automation schema version: {version}"
                )
            }
            Self::MalformedArtifact(detail) => {
                write!(formatter, "malformed persisted V1 artifact: {detail}")
            }
            Self::UnknownField(detail) => write!(formatter, "unknown persisted V1 field: {detail}"),
            Self::UnknownVariant(detail) => {
                write!(formatter, "unknown persisted V1 enum variant: {detail}")
            }
            Self::ResourceLimitExceeded(limit) => {
                write!(formatter, "persisted V1 resource limit exceeded: {limit}")
            }
            Self::InvalidIdentityReference(detail) => {
                write!(
                    formatter,
                    "invalid persisted V1 identity reference: {detail}"
                )
            }
            Self::UnsupportedTraceShape(detail) => {
                write!(formatter, "unsupported persisted V1 trace shape: {detail}")
            }
            Self::InvalidNormalizedInput(detail) => {
                write!(
                    formatter,
                    "persisted V1 contains invalid normalized input: {detail}"
                )
            }
            Self::UnsupportedRecordingWitness => {
                formatter.write_str("persisted V1 recording witness is unsupported")
            }
        }
    }
}

impl std::error::Error for AutomationInputTraceImportError {}

#[derive(Debug, Deserialize)]
struct EnvelopeProbe {
    artifact_kind: String,
    schema_version: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedTraceV1 {
    artifact_kind: String,
    schema_version: u32,
    recorded_sources_pristine_at_capture_start: bool,
    #[serde(default)]
    provenance: Option<AutomationInputTraceProvenance>,
    frames: Vec<PersistedFrameV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedFrameV1 {
    frame_ordinal: u64,
    groups: Vec<PersistedGroupV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedGroupV1 {
    source_slot: u32,
    device_slot: Option<u32>,
    observations: Vec<PersistedObservationV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum PersistedObservationV1 {
    PointerButton(PersistedPointerButtonInputV1),
    RelativeMotion(PersistedRelativeMotionV1),
    Scroll(PersistedScrollInputV1),
    Tablet(PersistedTabletObservationV1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum PersistedPointerButtonV1 {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedDigitalStateV1 {
    Pressed,
    Released,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedPointerButtonInputV1 {
    button: PersistedPointerButtonV1,
    state: PersistedDigitalStateV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedRelativeMotionV1 {
    delta: PersistedVector2V1,
    unit: PersistedRelativeMotionUnitV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedRelativeMotionUnitV1 {
    BackendDeviceUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedScrollInputV1 {
    horizontal: Option<f32>,
    vertical: Option<f32>,
    domain: PersistedScrollDomainV1,
    phase: Option<PersistedScrollPhaseV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedScrollDomainV1 {
    Unspecified,
    Lines,
    WindowPhysicalPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedScrollPhaseV1 {
    Begin,
    Update,
    End,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedTabletObservationV1 {
    contact_slot: u32,
    tool_slot: Option<u32>,
    tool_kind: PersistedInputToolKindV1,
    phase: PersistedContactPhaseV1,
    presence: PersistedContactPresenceV1,
    position: PersistedPoint2V1,
    delta: PersistedVector2V1,
    pressure: Option<PersistedAnalogMeasurementV1>,
    tangential_pressure: Option<PersistedAnalogMeasurementV1>,
    tilt: Option<PersistedStylusTiltV1>,
    twist: Option<PersistedAnalogMeasurementV1>,
    controls: PersistedPhysicalTabletControlsV1,
    capabilities: PersistedTabletCapabilitiesV1,
    source_time: Option<PersistedSourceTimeV1>,
    evidence: PersistedEvidenceStatusV1,
    delivery: PersistedDeliveryRoleV1,
    origin: PersistedObservationOriginV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedInputToolKindV1 {
    Mouse,
    Pen,
    Brush,
    Marker,
    Airbrush,
    Eraser,
    Finger,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedContactPhaseV1 {
    Begin,
    Update,
    End,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedContactPresenceV1 {
    Hover,
    Contact,
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedPoint2V1 {
    x: f32,
    y: f32,
    space: PersistedCoordinateSpaceV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedCoordinateSpaceV1 {
    UnspecifiedTargetUnits,
    WindowPhysicalPixels,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedVector2V1 {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedAnalogMeasurementV1 {
    value: f32,
    domain: PersistedMeasurementDomainV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum PersistedMeasurementDomainV1 {
    UnspecifiedScalar,
    NormalizedUnitInterval,
    SignedNormalizedUnitInterval,
    CalibratedForce { max_possible_force: f32 },
    Bounded { min: f32, max: f32 },
    Degrees { min: f32, max: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedStylusTiltV1 {
    x_degrees: f32,
    y_degrees: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedPhysicalTabletControlsV1 {
    eraser: bool,
    barrel_primary: bool,
    barrel_secondary: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedTabletCapabilitiesV1 {
    pressure: PersistedCapabilityKnowledgeV1,
    tilt: PersistedCapabilityKnowledgeV1,
    twist: PersistedCapabilityKnowledgeV1,
    tangential_pressure: PersistedCapabilityKnowledgeV1,
    hover: PersistedCapabilityKnowledgeV1,
    eraser: PersistedCapabilityKnowledgeV1,
    barrel_controls: PersistedCapabilityKnowledgeV1,
    historical_samples: PersistedCapabilityKnowledgeV1,
    predicted_samples: PersistedCapabilityKnowledgeV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedCapabilityKnowledgeV1 {
    Unknown,
    Supported,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PersistedSourceTimeV1 {
    value: u64,
    unit: PersistedSourceTimeUnitV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedSourceTimeUnitV1 {
    Microseconds,
    Milliseconds,
    Nanoseconds,
    NativeTicks { ticks_per_second: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedEvidenceStatusV1 {
    ObservedConfirmed,
    EstimatedRevisable,
    PredictedProvisional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedDeliveryRoleV1 {
    OrdinaryCurrent,
    HistoricalCoalesced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedObservationOriginV1 {
    SourceReport,
    BackendSyntheticReconciliation,
}

pub fn export_automation_input_trace_v1(
    trace: &AutomationInputTrace,
    recording_witness: AutomationInputTraceRecordingWitness,
    provenance: Option<&AutomationInputTraceProvenance>,
) -> Result<String, AutomationInputTraceExportError> {
    if !trace.trailing_groups.is_empty() {
        return Err(AutomationInputTraceExportError::UnframedTrailingGroups);
    }
    validate_export_normalized_input(trace)?;
    validate_provenance_export(provenance)?;
    let persisted = ExportBuilder::new().build(trace, recording_witness, provenance.cloned())?;
    let options = ron_options();
    let encoded = options
        .to_string_pretty(&persisted, ron::ser::PrettyConfig::new())
        .map_err(|error| {
            AutomationInputTraceExportError::SerializationFailure(error.to_string())
        })?;
    if encoded.len() > MAX_ARTIFACT_BYTES {
        return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
            "artifact_bytes",
        ));
    }
    Ok(encoded)
}

pub fn import_automation_input_trace_v1(
    bytes: &[u8],
) -> Result<ImportedAutomationInputTraceV1, AutomationInputTraceImportError> {
    if bytes.len() > MAX_ARTIFACT_BYTES {
        return Err(AutomationInputTraceImportError::ArtifactTooLarge);
    }
    let source = std::str::from_utf8(bytes)
        .map_err(|error| AutomationInputTraceImportError::ParseFailure(error.to_string()))?;

    let options = ron_options();
    let probe: EnvelopeProbe = options.from_str(source).map_err(classify_ron_error)?;
    if probe.artifact_kind != AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND {
        return Err(AutomationInputTraceImportError::WrongArtifactKind(
            probe.artifact_kind,
        ));
    }
    if probe.schema_version != AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION {
        return Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(
            probe.schema_version,
        ));
    }

    let persisted: PersistedTraceV1 = options.from_str(source).map_err(classify_ron_error)?;
    validate_persisted_header(&persisted)?;
    validate_provenance_import(persisted.provenance.as_ref())?;
    if !persisted.recorded_sources_pristine_at_capture_start {
        return Err(AutomationInputTraceImportError::UnsupportedRecordingWitness);
    }
    let recording_witness =
        AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart;

    let trace = ImportBuilder::new().build(&persisted)?;
    validate_imported_normalized_input(&trace)?;

    Ok(ImportedAutomationInputTraceV1 {
        trace,
        recording_witness,
        provenance: persisted.provenance,
    })
}

fn ron_options() -> ron::Options {
    ron::Options::default().with_recursion_limit(MAX_RON_RECURSION_DEPTH)
}

fn classify_ron_error(error: ron::error::SpannedError) -> AutomationInputTraceImportError {
    let detail = error.to_string();
    match error.code {
        RonError::ExceededRecursionLimit => {
            AutomationInputTraceImportError::ResourceLimitExceeded("ron_recursion_depth")
        }
        RonError::NoSuchStructField { .. } => AutomationInputTraceImportError::UnknownField(detail),
        RonError::NoSuchEnumVariant { .. } => {
            AutomationInputTraceImportError::UnknownVariant(detail)
        }
        RonError::MissingStructField { .. } | RonError::DuplicateStructField { .. } => {
            AutomationInputTraceImportError::MalformedArtifact(detail)
        }
        _ => AutomationInputTraceImportError::ParseFailure(detail),
    }
}

fn validate_persisted_header(
    persisted: &PersistedTraceV1,
) -> Result<(), AutomationInputTraceImportError> {
    if persisted.artifact_kind != AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND {
        return Err(AutomationInputTraceImportError::WrongArtifactKind(
            persisted.artifact_kind.clone(),
        ));
    }
    if persisted.schema_version != AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION {
        return Err(AutomationInputTraceImportError::UnsupportedSchemaVersion(
            persisted.schema_version,
        ));
    }
    Ok(())
}

fn metadata_strings(provenance: &AutomationInputTraceProvenance) -> impl Iterator<Item = &String> {
    [
        provenance.runenwerk_revision.as_ref(),
        provenance.runen_input_revision.as_ref(),
        provenance.label.as_ref(),
        provenance.description.as_ref(),
    ]
    .into_iter()
    .flatten()
}

fn validate_provenance_export(
    provenance: Option<&AutomationInputTraceProvenance>,
) -> Result<(), AutomationInputTraceExportError> {
    if provenance
        .into_iter()
        .flat_map(metadata_strings)
        .any(|value| value.len() > MAX_METADATA_STRING_BYTES)
    {
        return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
            "metadata_string_bytes",
        ));
    }
    Ok(())
}

fn validate_provenance_import(
    provenance: Option<&AutomationInputTraceProvenance>,
) -> Result<(), AutomationInputTraceImportError> {
    if provenance
        .into_iter()
        .flat_map(metadata_strings)
        .any(|value| value.len() > MAX_METADATA_STRING_BYTES)
    {
        return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
            "metadata_string_bytes",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct ExportBuilder {
    sources: HashMap<InputSourceId, u32>,
    next_source: u32,
    devices: HashMap<(InputSourceId, InputDeviceId), u32>,
    next_device: HashMap<InputSourceId, u32>,
    contacts: HashMap<(InputContext, ContactId), u32>,
    next_contact: HashMap<InputContext, u32>,
    tools: HashMap<(InputContext, ToolId), u32>,
    next_tool: HashMap<InputContext, u32>,
    first_pointer_state: HashSet<(InputContext, PointerButton)>,
    total_groups: usize,
}

impl ExportBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn build(
        mut self,
        trace: &AutomationInputTrace,
        recording_witness: AutomationInputTraceRecordingWitness,
        provenance: Option<AutomationInputTraceProvenance>,
    ) -> Result<PersistedTraceV1, AutomationInputTraceExportError> {
        if trace.frames.len() > MAX_FRAMES {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "frames",
            ));
        }

        let mut frames = Vec::with_capacity(trace.frames.len());
        for (frame_index, frame) in trace.frames.iter().enumerate() {
            if frame.frame_ordinal != frame_index as u64 {
                return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                    "frame ordinals must be contiguous from zero".to_owned(),
                ));
            }
            if frame.groups.len() > MAX_GROUPS_PER_FRAME {
                return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                    "groups_per_frame",
                ));
            }
            self.total_groups = self.total_groups.checked_add(frame.groups.len()).ok_or(
                AutomationInputTraceExportError::ResourceLimitExceeded("total_groups"),
            )?;
            if self.total_groups > MAX_TOTAL_GROUPS {
                return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                    "total_groups",
                ));
            }

            let mut groups = Vec::with_capacity(frame.groups.len());
            for group in &frame.groups {
                groups.push(self.convert_group(group)?);
            }
            frames.push(PersistedFrameV1 {
                frame_ordinal: frame.frame_ordinal,
                groups,
            });
        }

        Ok(PersistedTraceV1 {
            artifact_kind: AUTOMATION_INPUT_TRACE_V1_ARTIFACT_KIND.to_owned(),
            schema_version: AUTOMATION_INPUT_TRACE_V1_SCHEMA_VERSION,
            recorded_sources_pristine_at_capture_start: match recording_witness {
                AutomationInputTraceRecordingWitness::RecordedSourcesPristineAtCaptureStart => true,
            },
            provenance,
            frames,
        })
    }

    fn convert_group(
        &mut self,
        group: &InputObservationGroup,
    ) -> Result<PersistedGroupV1, AutomationInputTraceExportError> {
        if group.observations.is_empty() {
            return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                "empty observation groups are not persisted".to_owned(),
            ));
        }
        if group.observations.len() > MAX_OBSERVATIONS_PER_GROUP {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "observations_per_group",
            ));
        }

        let source_slot = self.source_slot(group.context.source)?;
        let device_slot = group
            .context
            .device
            .map(|device| self.device_slot(group.context.source, device))
            .transpose()?;

        let all_tablet = group
            .observations
            .iter()
            .all(|observation| matches!(observation, InputObservation::Tablet(_)));
        if !all_tablet && group.observations.len() != 1 {
            return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                "multi-observation non-tablet groups are not persisted".to_owned(),
            ));
        }

        let mut observations = Vec::with_capacity(group.observations.len());
        for observation in &group.observations {
            observations.push(self.convert_observation(group.context, observation, all_tablet)?);
        }

        Ok(PersistedGroupV1 {
            source_slot,
            device_slot,
            observations,
        })
    }

    fn source_slot(
        &mut self,
        source: InputSourceId,
    ) -> Result<u32, AutomationInputTraceExportError> {
        if let Some(slot) = self.sources.get(&source) {
            return Ok(*slot);
        }
        if self.sources.len() >= MAX_SOURCES {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "sources",
            ));
        }
        let slot = self.next_source;
        self.next_source += 1;
        self.sources.insert(source, slot);
        Ok(slot)
    }

    fn device_slot(
        &mut self,
        source: InputSourceId,
        device: InputDeviceId,
    ) -> Result<u32, AutomationInputTraceExportError> {
        if let Some(slot) = self.devices.get(&(source, device)) {
            return Ok(*slot);
        }
        let next = self.next_device.entry(source).or_default();
        if *next as usize >= MAX_DEVICES_PER_SOURCE {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "devices_per_source",
            ));
        }
        let slot = *next;
        *next += 1;
        self.devices.insert((source, device), slot);
        Ok(slot)
    }

    fn contact_slot(
        &mut self,
        context: InputContext,
        contact: ContactId,
    ) -> Result<u32, AutomationInputTraceExportError> {
        if let Some(slot) = self.contacts.get(&(context, contact)) {
            return Ok(*slot);
        }
        let next = self.next_contact.entry(context).or_default();
        if *next as usize >= MAX_CONTACTS_PER_CONTEXT {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "contacts_per_context",
            ));
        }
        let slot = *next;
        *next += 1;
        self.contacts.insert((context, contact), slot);
        Ok(slot)
    }

    fn tool_slot(
        &mut self,
        context: InputContext,
        tool: ToolId,
    ) -> Result<u32, AutomationInputTraceExportError> {
        if let Some(slot) = self.tools.get(&(context, tool)) {
            return Ok(*slot);
        }
        let next = self.next_tool.entry(context).or_default();
        if *next as usize >= MAX_TOOLS_PER_CONTEXT {
            return Err(AutomationInputTraceExportError::ResourceLimitExceeded(
                "tools_per_context",
            ));
        }
        let slot = *next;
        *next += 1;
        self.tools.insert((context, tool), slot);
        Ok(slot)
    }

    fn convert_observation(
        &mut self,
        context: InputContext,
        observation: &InputObservation,
        all_tablet: bool,
    ) -> Result<PersistedObservationV1, AutomationInputTraceExportError> {
        match observation {
            InputObservation::PointerButton(input) if !all_tablet => {
                if self.first_pointer_state.insert((context, input.button))
                    && input.state != DigitalState::Pressed
                {
                    return Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                        "first pointer-button state must establish a press".to_owned(),
                    ));
                }
                Ok(PersistedObservationV1::PointerButton(
                    PersistedPointerButtonInputV1 {
                        button: pointer_button_to_persisted(input.button),
                        state: digital_state_to_persisted(input.state),
                    },
                ))
            }
            InputObservation::RelativeMotion { delta, unit } if !all_tablet => Ok(
                PersistedObservationV1::RelativeMotion(PersistedRelativeMotionV1 {
                    delta: vector_to_persisted(*delta),
                    unit: relative_unit_to_persisted(*unit),
                }),
            ),
            InputObservation::Scroll(input) if !all_tablet => {
                Ok(PersistedObservationV1::Scroll(scroll_to_persisted(*input)))
            }
            InputObservation::Tablet(tablet) if all_tablet => Ok(PersistedObservationV1::Tablet(
                self.tablet_to_persisted(context, tablet)?,
            )),
            _ => Err(AutomationInputTraceExportError::UnsupportedTraceShape(
                "trace contains an observation outside persisted V1 replay scope".to_owned(),
            )),
        }
    }

    fn tablet_to_persisted(
        &mut self,
        context: InputContext,
        tablet: &TabletObservation,
    ) -> Result<PersistedTabletObservationV1, AutomationInputTraceExportError> {
        Ok(PersistedTabletObservationV1 {
            contact_slot: self.contact_slot(context, tablet.contact)?,
            tool_slot: tablet
                .tool
                .map(|tool| self.tool_slot(context, tool))
                .transpose()?,
            tool_kind: tool_kind_to_persisted(tablet.tool_kind),
            phase: contact_phase_to_persisted(tablet.phase),
            presence: contact_presence_to_persisted(tablet.presence),
            position: point_to_persisted(tablet.position),
            delta: vector_to_persisted(tablet.delta),
            pressure: tablet.pressure.map(measurement_to_persisted),
            tangential_pressure: tablet.tangential_pressure.map(measurement_to_persisted),
            tilt: tablet.tilt.map(tilt_to_persisted),
            twist: tablet.twist.map(measurement_to_persisted),
            controls: controls_to_persisted(tablet.controls),
            capabilities: capabilities_to_persisted(tablet.capabilities),
            source_time: tablet.source_time.map(|time| PersistedSourceTimeV1 {
                value: time.value,
                unit: source_time_unit_to_persisted(time.unit),
            }),
            evidence: evidence_to_persisted(tablet.evidence),
            delivery: delivery_to_persisted(tablet.delivery),
            origin: origin_to_persisted(tablet.origin),
        })
    }
}

#[derive(Default)]
struct ImportBuilder {
    next_source: u32,
    next_device: HashMap<u32, u32>,
    next_contact: HashMap<(u32, Option<u32>), u32>,
    next_tool: HashMap<(u32, Option<u32>), u32>,
    first_pointer_state: HashSet<(InputContext, PointerButton)>,
    total_groups: usize,
}

impl ImportBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn build(
        mut self,
        persisted: &PersistedTraceV1,
    ) -> Result<AutomationInputTrace, AutomationInputTraceImportError> {
        if persisted.frames.len() > MAX_FRAMES {
            return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                "frames",
            ));
        }

        let mut frames = Vec::with_capacity(persisted.frames.len());
        for (frame_index, frame) in persisted.frames.iter().enumerate() {
            if frame.frame_ordinal != frame_index as u64 {
                return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                    "frame ordinals must be contiguous from zero".to_owned(),
                ));
            }
            if frame.groups.len() > MAX_GROUPS_PER_FRAME {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "groups_per_frame",
                ));
            }
            self.total_groups = self.total_groups.checked_add(frame.groups.len()).ok_or(
                AutomationInputTraceImportError::ResourceLimitExceeded("total_groups"),
            )?;
            if self.total_groups > MAX_TOTAL_GROUPS {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "total_groups",
                ));
            }

            let mut groups = Vec::with_capacity(frame.groups.len());
            for group in &frame.groups {
                groups.push(self.convert_group(group)?);
            }
            frames.push(AutomationInputTraceFrame {
                frame_ordinal: frame.frame_ordinal,
                groups,
            });
        }

        Ok(AutomationInputTrace {
            frames,
            trailing_groups: Vec::new(),
        })
    }

    fn convert_group(
        &mut self,
        group: &PersistedGroupV1,
    ) -> Result<InputObservationGroup, AutomationInputTraceImportError> {
        if group.observations.is_empty() {
            return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                "empty observation groups are not replayable".to_owned(),
            ));
        }
        if group.observations.len() > MAX_OBSERVATIONS_PER_GROUP {
            return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                "observations_per_group",
            ));
        }

        let source = self.materialize_source(group.source_slot)?;
        let device = group
            .device_slot
            .map(|slot| self.materialize_device(group.source_slot, slot))
            .transpose()?;
        let context = InputContext::new(source, device);

        let all_tablet = group
            .observations
            .iter()
            .all(|observation| matches!(observation, PersistedObservationV1::Tablet(_)));
        if !all_tablet && group.observations.len() != 1 {
            return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                "multi-observation non-tablet groups are not replayable".to_owned(),
            ));
        }

        let mut observations = Vec::with_capacity(group.observations.len());
        for observation in &group.observations {
            let converted = self.convert_observation(
                group.source_slot,
                group.device_slot,
                context,
                observation,
                all_tablet,
            )?;
            observations.push(converted);
        }

        Ok(InputObservationGroup::new(context, observations))
    }

    fn materialize_source(
        &mut self,
        slot: u32,
    ) -> Result<InputSourceId, AutomationInputTraceImportError> {
        if slot > self.next_source {
            return Err(AutomationInputTraceImportError::InvalidIdentityReference(
                format!(
                    "source slot {slot} skipped first-appearance slot {}",
                    self.next_source
                ),
            ));
        }
        if slot == self.next_source {
            if self.next_source as usize >= MAX_SOURCES {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "sources",
                ));
            }
            self.next_source += 1;
        }
        Ok(InputSourceId::new(slot as u64 + 1))
    }

    fn materialize_device(
        &mut self,
        source_slot: u32,
        slot: u32,
    ) -> Result<InputDeviceId, AutomationInputTraceImportError> {
        let next = self.next_device.entry(source_slot).or_default();
        if slot > *next {
            return Err(AutomationInputTraceImportError::InvalidIdentityReference(
                format!("device slot {slot} skipped first-appearance slot {next}"),
            ));
        }
        if slot == *next {
            if *next as usize >= MAX_DEVICES_PER_SOURCE {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "devices_per_source",
                ));
            }
            *next += 1;
        }
        Ok(InputDeviceId::new(slot as u64 + 1))
    }

    fn materialize_contact(
        &mut self,
        source_slot: u32,
        device_slot: Option<u32>,
        slot: u32,
    ) -> Result<ContactId, AutomationInputTraceImportError> {
        let next = self
            .next_contact
            .entry((source_slot, device_slot))
            .or_default();
        if slot > *next {
            return Err(AutomationInputTraceImportError::InvalidIdentityReference(
                format!("contact slot {slot} skipped first-appearance slot {next}"),
            ));
        }
        if slot == *next {
            if *next as usize >= MAX_CONTACTS_PER_CONTEXT {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "contacts_per_context",
                ));
            }
            *next += 1;
        }
        Ok(ContactId::new(slot as u64 + 1))
    }

    fn materialize_tool(
        &mut self,
        source_slot: u32,
        device_slot: Option<u32>,
        slot: u32,
    ) -> Result<ToolId, AutomationInputTraceImportError> {
        let next = self
            .next_tool
            .entry((source_slot, device_slot))
            .or_default();
        if slot > *next {
            return Err(AutomationInputTraceImportError::InvalidIdentityReference(
                format!("tool slot {slot} skipped first-appearance slot {next}"),
            ));
        }
        if slot == *next {
            if *next as usize >= MAX_TOOLS_PER_CONTEXT {
                return Err(AutomationInputTraceImportError::ResourceLimitExceeded(
                    "tools_per_context",
                ));
            }
            *next += 1;
        }
        Ok(ToolId::new(slot as u64 + 1))
    }

    fn convert_observation(
        &mut self,
        source_slot: u32,
        device_slot: Option<u32>,
        context: InputContext,
        observation: &PersistedObservationV1,
        all_tablet: bool,
    ) -> Result<InputObservation, AutomationInputTraceImportError> {
        match observation {
            PersistedObservationV1::PointerButton(input) if !all_tablet => {
                let button = pointer_button_from_persisted(input.button);
                let state = digital_state_from_persisted(input.state);
                if self.first_pointer_state.insert((context, button))
                    && state != DigitalState::Pressed
                {
                    return Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                        "first pointer-button state must establish a press".to_owned(),
                    ));
                }
                Ok(InputObservation::PointerButton(PointerButtonInput {
                    button,
                    state,
                }))
            }
            PersistedObservationV1::RelativeMotion(input) if !all_tablet => {
                Ok(InputObservation::RelativeMotion {
                    delta: vector_from_persisted(input.delta),
                    unit: relative_unit_from_persisted(input.unit),
                })
            }
            PersistedObservationV1::Scroll(input) if !all_tablet => {
                Ok(InputObservation::Scroll(scroll_from_persisted(*input)))
            }
            PersistedObservationV1::Tablet(tablet) if all_tablet => Ok(InputObservation::Tablet(
                self.tablet_from_persisted(source_slot, device_slot, context, tablet)?,
            )),
            _ => Err(AutomationInputTraceImportError::UnsupportedTraceShape(
                "artifact contains an observation outside persisted V1 replay scope".to_owned(),
            )),
        }
    }

    fn tablet_from_persisted(
        &mut self,
        source_slot: u32,
        device_slot: Option<u32>,
        context: InputContext,
        tablet: &PersistedTabletObservationV1,
    ) -> Result<TabletObservation, AutomationInputTraceImportError> {
        Ok(TabletObservation {
            contact: self.materialize_contact(source_slot, device_slot, tablet.contact_slot)?,
            tool: tablet
                .tool_slot
                .map(|slot| self.materialize_tool(source_slot, device_slot, slot))
                .transpose()?,
            tool_kind: tool_kind_from_persisted(tablet.tool_kind),
            phase: contact_phase_from_persisted(tablet.phase),
            presence: contact_presence_from_persisted(tablet.presence),
            position: point_from_persisted(tablet.position),
            delta: vector_from_persisted(tablet.delta),
            pressure: tablet.pressure.map(measurement_from_persisted),
            tangential_pressure: tablet.tangential_pressure.map(measurement_from_persisted),
            tilt: tablet.tilt.map(tilt_from_persisted),
            twist: tablet.twist.map(measurement_from_persisted),
            controls: controls_from_persisted(tablet.controls),
            capabilities: capabilities_from_persisted(tablet.capabilities),
            source_time: tablet.source_time.map(|time| {
                SourceTime::new(
                    context,
                    time.value,
                    source_time_unit_from_persisted(time.unit),
                )
            }),
            evidence: evidence_from_persisted(tablet.evidence),
            delivery: delivery_from_persisted(tablet.delivery),
            origin: origin_from_persisted(tablet.origin),
        })
    }
}

fn validate_export_normalized_input(
    trace: &AutomationInputTrace,
) -> Result<(), AutomationInputTraceExportError> {
    let mut neutral = runen_input::InputState::default();
    for frame in &trace.frames {
        for group in &frame.groups {
            neutral.admit(group).map_err(|error| {
                AutomationInputTraceExportError::UnsupportedTraceShape(format!(
                    "runtime trace contains invalid normalized input: {error}"
                ))
            })?;
        }
    }
    Ok(())
}

fn validate_imported_normalized_input(
    trace: &AutomationInputTrace,
) -> Result<(), AutomationInputTraceImportError> {
    let mut neutral = runen_input::InputState::default();
    for frame in &trace.frames {
        for group in &frame.groups {
            neutral.admit(group).map_err(|error| {
                AutomationInputTraceImportError::InvalidNormalizedInput(error.to_string())
            })?;
        }
    }
    Ok(())
}

fn pointer_button_to_persisted(button: PointerButton) -> PersistedPointerButtonV1 {
    match button {
        PointerButton::Left => PersistedPointerButtonV1::Left,
        PointerButton::Right => PersistedPointerButtonV1::Right,
        PointerButton::Middle => PersistedPointerButtonV1::Middle,
        PointerButton::Back => PersistedPointerButtonV1::Back,
        PointerButton::Forward => PersistedPointerButtonV1::Forward,
        PointerButton::Other(value) => PersistedPointerButtonV1::Other(value),
    }
}

fn pointer_button_from_persisted(button: PersistedPointerButtonV1) -> PointerButton {
    match button {
        PersistedPointerButtonV1::Left => PointerButton::Left,
        PersistedPointerButtonV1::Right => PointerButton::Right,
        PersistedPointerButtonV1::Middle => PointerButton::Middle,
        PersistedPointerButtonV1::Back => PointerButton::Back,
        PersistedPointerButtonV1::Forward => PointerButton::Forward,
        PersistedPointerButtonV1::Other(value) => PointerButton::Other(value),
    }
}

fn digital_state_to_persisted(state: DigitalState) -> PersistedDigitalStateV1 {
    match state {
        DigitalState::Pressed => PersistedDigitalStateV1::Pressed,
        DigitalState::Released => PersistedDigitalStateV1::Released,
    }
}

fn digital_state_from_persisted(state: PersistedDigitalStateV1) -> DigitalState {
    match state {
        PersistedDigitalStateV1::Pressed => DigitalState::Pressed,
        PersistedDigitalStateV1::Released => DigitalState::Released,
    }
}

fn relative_unit_to_persisted(unit: RelativeMotionUnit) -> PersistedRelativeMotionUnitV1 {
    match unit {
        RelativeMotionUnit::BackendDeviceUnits => PersistedRelativeMotionUnitV1::BackendDeviceUnits,
    }
}

fn relative_unit_from_persisted(unit: PersistedRelativeMotionUnitV1) -> RelativeMotionUnit {
    match unit {
        PersistedRelativeMotionUnitV1::BackendDeviceUnits => RelativeMotionUnit::BackendDeviceUnits,
    }
}

fn scroll_to_persisted(input: ScrollInput) -> PersistedScrollInputV1 {
    PersistedScrollInputV1 {
        horizontal: input.delta.horizontal,
        vertical: input.delta.vertical,
        domain: match input.domain {
            ScrollDomain::Unspecified => PersistedScrollDomainV1::Unspecified,
            ScrollDomain::Lines => PersistedScrollDomainV1::Lines,
            ScrollDomain::WindowPhysicalPixels => PersistedScrollDomainV1::WindowPhysicalPixels,
        },
        phase: input.phase.map(|phase| match phase {
            ScrollPhase::Begin => PersistedScrollPhaseV1::Begin,
            ScrollPhase::Update => PersistedScrollPhaseV1::Update,
            ScrollPhase::End => PersistedScrollPhaseV1::End,
            ScrollPhase::Cancel => PersistedScrollPhaseV1::Cancel,
        }),
    }
}

fn scroll_from_persisted(input: PersistedScrollInputV1) -> ScrollInput {
    ScrollInput {
        delta: ScrollDelta {
            horizontal: input.horizontal,
            vertical: input.vertical,
        },
        domain: match input.domain {
            PersistedScrollDomainV1::Unspecified => ScrollDomain::Unspecified,
            PersistedScrollDomainV1::Lines => ScrollDomain::Lines,
            PersistedScrollDomainV1::WindowPhysicalPixels => ScrollDomain::WindowPhysicalPixels,
        },
        phase: input.phase.map(|phase| match phase {
            PersistedScrollPhaseV1::Begin => ScrollPhase::Begin,
            PersistedScrollPhaseV1::Update => ScrollPhase::Update,
            PersistedScrollPhaseV1::End => ScrollPhase::End,
            PersistedScrollPhaseV1::Cancel => ScrollPhase::Cancel,
        }),
    }
}

fn vector_to_persisted(value: Vector2) -> PersistedVector2V1 {
    PersistedVector2V1 {
        x: value.x,
        y: value.y,
    }
}

fn vector_from_persisted(value: PersistedVector2V1) -> Vector2 {
    Vector2::new(value.x, value.y)
}

fn point_to_persisted(value: Point2) -> PersistedPoint2V1 {
    PersistedPoint2V1 {
        x: value.x,
        y: value.y,
        space: match value.space {
            CoordinateSpace::UnspecifiedTargetUnits => {
                PersistedCoordinateSpaceV1::UnspecifiedTargetUnits
            }
            CoordinateSpace::WindowPhysicalPixels => {
                PersistedCoordinateSpaceV1::WindowPhysicalPixels
            }
        },
    }
}

fn point_from_persisted(value: PersistedPoint2V1) -> Point2 {
    Point2::new(
        value.x,
        value.y,
        match value.space {
            PersistedCoordinateSpaceV1::UnspecifiedTargetUnits => {
                CoordinateSpace::UnspecifiedTargetUnits
            }
            PersistedCoordinateSpaceV1::WindowPhysicalPixels => {
                CoordinateSpace::WindowPhysicalPixels
            }
        },
    )
}

fn measurement_to_persisted(value: AnalogMeasurement) -> PersistedAnalogMeasurementV1 {
    PersistedAnalogMeasurementV1 {
        value: value.value,
        domain: match value.domain {
            MeasurementDomain::UnspecifiedScalar => PersistedMeasurementDomainV1::UnspecifiedScalar,
            MeasurementDomain::NormalizedUnitInterval => {
                PersistedMeasurementDomainV1::NormalizedUnitInterval
            }
            MeasurementDomain::SignedNormalizedUnitInterval => {
                PersistedMeasurementDomainV1::SignedNormalizedUnitInterval
            }
            MeasurementDomain::CalibratedForce { max_possible_force } => {
                PersistedMeasurementDomainV1::CalibratedForce { max_possible_force }
            }
            MeasurementDomain::Bounded { min, max } => {
                PersistedMeasurementDomainV1::Bounded { min, max }
            }
            MeasurementDomain::Degrees { min, max } => {
                PersistedMeasurementDomainV1::Degrees { min, max }
            }
        },
    }
}

fn measurement_from_persisted(value: PersistedAnalogMeasurementV1) -> AnalogMeasurement {
    AnalogMeasurement::new(
        value.value,
        match value.domain {
            PersistedMeasurementDomainV1::UnspecifiedScalar => MeasurementDomain::UnspecifiedScalar,
            PersistedMeasurementDomainV1::NormalizedUnitInterval => {
                MeasurementDomain::NormalizedUnitInterval
            }
            PersistedMeasurementDomainV1::SignedNormalizedUnitInterval => {
                MeasurementDomain::SignedNormalizedUnitInterval
            }
            PersistedMeasurementDomainV1::CalibratedForce { max_possible_force } => {
                MeasurementDomain::CalibratedForce { max_possible_force }
            }
            PersistedMeasurementDomainV1::Bounded { min, max } => {
                MeasurementDomain::Bounded { min, max }
            }
            PersistedMeasurementDomainV1::Degrees { min, max } => {
                MeasurementDomain::Degrees { min, max }
            }
        },
    )
}

fn tilt_to_persisted(value: StylusTilt) -> PersistedStylusTiltV1 {
    PersistedStylusTiltV1 {
        x_degrees: value.x_degrees,
        y_degrees: value.y_degrees,
    }
}

fn tilt_from_persisted(value: PersistedStylusTiltV1) -> StylusTilt {
    StylusTilt::new(value.x_degrees, value.y_degrees)
}

fn controls_to_persisted(value: PhysicalTabletControls) -> PersistedPhysicalTabletControlsV1 {
    PersistedPhysicalTabletControlsV1 {
        eraser: value.eraser,
        barrel_primary: value.barrel_primary,
        barrel_secondary: value.barrel_secondary,
    }
}

fn controls_from_persisted(value: PersistedPhysicalTabletControlsV1) -> PhysicalTabletControls {
    PhysicalTabletControls {
        eraser: value.eraser,
        barrel_primary: value.barrel_primary,
        barrel_secondary: value.barrel_secondary,
    }
}

fn capability_to_persisted(value: CapabilityKnowledge) -> PersistedCapabilityKnowledgeV1 {
    match value {
        CapabilityKnowledge::Unknown => PersistedCapabilityKnowledgeV1::Unknown,
        CapabilityKnowledge::Supported => PersistedCapabilityKnowledgeV1::Supported,
        CapabilityKnowledge::Unsupported => PersistedCapabilityKnowledgeV1::Unsupported,
    }
}

fn capability_from_persisted(value: PersistedCapabilityKnowledgeV1) -> CapabilityKnowledge {
    match value {
        PersistedCapabilityKnowledgeV1::Unknown => CapabilityKnowledge::Unknown,
        PersistedCapabilityKnowledgeV1::Supported => CapabilityKnowledge::Supported,
        PersistedCapabilityKnowledgeV1::Unsupported => CapabilityKnowledge::Unsupported,
    }
}

fn capabilities_to_persisted(value: TabletCapabilities) -> PersistedTabletCapabilitiesV1 {
    PersistedTabletCapabilitiesV1 {
        pressure: capability_to_persisted(value.pressure),
        tilt: capability_to_persisted(value.tilt),
        twist: capability_to_persisted(value.twist),
        tangential_pressure: capability_to_persisted(value.tangential_pressure),
        hover: capability_to_persisted(value.hover),
        eraser: capability_to_persisted(value.eraser),
        barrel_controls: capability_to_persisted(value.barrel_controls),
        historical_samples: capability_to_persisted(value.historical_samples),
        predicted_samples: capability_to_persisted(value.predicted_samples),
    }
}

fn capabilities_from_persisted(value: PersistedTabletCapabilitiesV1) -> TabletCapabilities {
    TabletCapabilities {
        pressure: capability_from_persisted(value.pressure),
        tilt: capability_from_persisted(value.tilt),
        twist: capability_from_persisted(value.twist),
        tangential_pressure: capability_from_persisted(value.tangential_pressure),
        hover: capability_from_persisted(value.hover),
        eraser: capability_from_persisted(value.eraser),
        barrel_controls: capability_from_persisted(value.barrel_controls),
        historical_samples: capability_from_persisted(value.historical_samples),
        predicted_samples: capability_from_persisted(value.predicted_samples),
    }
}

fn source_time_unit_to_persisted(value: SourceTimeUnit) -> PersistedSourceTimeUnitV1 {
    match value {
        SourceTimeUnit::Microseconds => PersistedSourceTimeUnitV1::Microseconds,
        SourceTimeUnit::Milliseconds => PersistedSourceTimeUnitV1::Milliseconds,
        SourceTimeUnit::Nanoseconds => PersistedSourceTimeUnitV1::Nanoseconds,
        SourceTimeUnit::NativeTicks { ticks_per_second } => {
            PersistedSourceTimeUnitV1::NativeTicks { ticks_per_second }
        }
    }
}

fn source_time_unit_from_persisted(value: PersistedSourceTimeUnitV1) -> SourceTimeUnit {
    match value {
        PersistedSourceTimeUnitV1::Microseconds => SourceTimeUnit::Microseconds,
        PersistedSourceTimeUnitV1::Milliseconds => SourceTimeUnit::Milliseconds,
        PersistedSourceTimeUnitV1::Nanoseconds => SourceTimeUnit::Nanoseconds,
        PersistedSourceTimeUnitV1::NativeTicks { ticks_per_second } => {
            SourceTimeUnit::NativeTicks { ticks_per_second }
        }
    }
}

fn evidence_to_persisted(value: EvidenceStatus) -> PersistedEvidenceStatusV1 {
    match value {
        EvidenceStatus::ObservedConfirmed => PersistedEvidenceStatusV1::ObservedConfirmed,
        EvidenceStatus::EstimatedRevisable => PersistedEvidenceStatusV1::EstimatedRevisable,
        EvidenceStatus::PredictedProvisional => PersistedEvidenceStatusV1::PredictedProvisional,
    }
}

fn evidence_from_persisted(value: PersistedEvidenceStatusV1) -> EvidenceStatus {
    match value {
        PersistedEvidenceStatusV1::ObservedConfirmed => EvidenceStatus::ObservedConfirmed,
        PersistedEvidenceStatusV1::EstimatedRevisable => EvidenceStatus::EstimatedRevisable,
        PersistedEvidenceStatusV1::PredictedProvisional => EvidenceStatus::PredictedProvisional,
    }
}

fn delivery_to_persisted(value: DeliveryRole) -> PersistedDeliveryRoleV1 {
    match value {
        DeliveryRole::OrdinaryCurrent => PersistedDeliveryRoleV1::OrdinaryCurrent,
        DeliveryRole::HistoricalCoalesced => PersistedDeliveryRoleV1::HistoricalCoalesced,
    }
}

fn delivery_from_persisted(value: PersistedDeliveryRoleV1) -> DeliveryRole {
    match value {
        PersistedDeliveryRoleV1::OrdinaryCurrent => DeliveryRole::OrdinaryCurrent,
        PersistedDeliveryRoleV1::HistoricalCoalesced => DeliveryRole::HistoricalCoalesced,
    }
}

fn origin_to_persisted(value: ObservationOrigin) -> PersistedObservationOriginV1 {
    match value {
        ObservationOrigin::SourceReport => PersistedObservationOriginV1::SourceReport,
        ObservationOrigin::BackendSyntheticReconciliation => {
            PersistedObservationOriginV1::BackendSyntheticReconciliation
        }
    }
}

fn origin_from_persisted(value: PersistedObservationOriginV1) -> ObservationOrigin {
    match value {
        PersistedObservationOriginV1::SourceReport => ObservationOrigin::SourceReport,
        PersistedObservationOriginV1::BackendSyntheticReconciliation => {
            ObservationOrigin::BackendSyntheticReconciliation
        }
    }
}

fn tool_kind_to_persisted(value: InputToolKind) -> PersistedInputToolKindV1 {
    match value {
        InputToolKind::Mouse => PersistedInputToolKindV1::Mouse,
        InputToolKind::Pen => PersistedInputToolKindV1::Pen,
        InputToolKind::Brush => PersistedInputToolKindV1::Brush,
        InputToolKind::Marker => PersistedInputToolKindV1::Marker,
        InputToolKind::Airbrush => PersistedInputToolKindV1::Airbrush,
        InputToolKind::Eraser => PersistedInputToolKindV1::Eraser,
        InputToolKind::Finger => PersistedInputToolKindV1::Finger,
        InputToolKind::Unknown => PersistedInputToolKindV1::Unknown,
    }
}

fn tool_kind_from_persisted(value: PersistedInputToolKindV1) -> InputToolKind {
    match value {
        PersistedInputToolKindV1::Mouse => InputToolKind::Mouse,
        PersistedInputToolKindV1::Pen => InputToolKind::Pen,
        PersistedInputToolKindV1::Brush => InputToolKind::Brush,
        PersistedInputToolKindV1::Marker => InputToolKind::Marker,
        PersistedInputToolKindV1::Airbrush => InputToolKind::Airbrush,
        PersistedInputToolKindV1::Eraser => InputToolKind::Eraser,
        PersistedInputToolKindV1::Finger => InputToolKind::Finger,
        PersistedInputToolKindV1::Unknown => InputToolKind::Unknown,
    }
}

fn contact_phase_to_persisted(value: ContactPhase) -> PersistedContactPhaseV1 {
    match value {
        ContactPhase::Begin => PersistedContactPhaseV1::Begin,
        ContactPhase::Update => PersistedContactPhaseV1::Update,
        ContactPhase::End => PersistedContactPhaseV1::End,
        ContactPhase::Cancel => PersistedContactPhaseV1::Cancel,
    }
}

fn contact_phase_from_persisted(value: PersistedContactPhaseV1) -> ContactPhase {
    match value {
        PersistedContactPhaseV1::Begin => ContactPhase::Begin,
        PersistedContactPhaseV1::Update => ContactPhase::Update,
        PersistedContactPhaseV1::End => ContactPhase::End,
        PersistedContactPhaseV1::Cancel => ContactPhase::Cancel,
    }
}

fn contact_presence_to_persisted(value: ContactPresence) -> PersistedContactPresenceV1 {
    match value {
        ContactPresence::Hover => PersistedContactPresenceV1::Hover,
        ContactPresence::Contact => PersistedContactPresenceV1::Contact,
        ContactPresence::OutOfRange => PersistedContactPresenceV1::OutOfRange,
    }
}

fn contact_presence_from_persisted(value: PersistedContactPresenceV1) -> ContactPresence {
    match value {
        PersistedContactPresenceV1::Hover => ContactPresence::Hover,
        PersistedContactPresenceV1::Contact => ContactPresence::Contact,
        PersistedContactPresenceV1::OutOfRange => ContactPresence::OutOfRange,
    }
}

#[cfg(test)]
mod tests;
