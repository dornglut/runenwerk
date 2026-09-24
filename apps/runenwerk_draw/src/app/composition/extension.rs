//! Typed Draw interpretation of app-neutral mounted content.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use ui_composition::{
    AppProfileId, CanonicalExtensionPayload, CompositionDefinitionId, CompositionState,
    DefinitionRevision, ExtensionProfileId, ExtensionSchemaVersion, MountedUnitId,
};

use super::{
    DRAWING_APP_PROFILE, DRAWING_CANVAS_UNIT_ID, DRAWING_EXTENSION_PROFILE,
    DRAWING_SUPPORT_UNIT_ID, DRAWING_TOOL_RAIL_UNIT_ID, DRAWING_TOP_BAR_UNIT_ID,
    DrawingCompositionDiagnosticCode as Code, DrawingCompositionDiagnosticRecord as Record,
    DrawingCompositionDiagnosticStage as Stage, DrawingCompositionDiagnosticSubject as Subject,
    DrawingCompositionRejection,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DrawingContentRole {
    TopBar,
    ToolRail,
    Canvas,
    SupportPanel,
}

impl DrawingContentRole {
    pub const ALL: [Self; 4] = [
        Self::TopBar,
        Self::ToolRail,
        Self::Canvas,
        Self::SupportPanel,
    ];

    pub const fn expected_content_profile(self) -> &'static str {
        match self {
            Self::TopBar => "runenwerk.draw.top_bar",
            Self::ToolRail => "runenwerk.draw.tool_rail",
            Self::Canvas => "runenwerk.draw.canvas",
            Self::SupportPanel => "runenwerk.draw.support_panel",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DrawingUnavailableProjectionPolicy {
    AppProvided,
    NeutralOnly,
}

impl DrawingUnavailableProjectionPolicy {
    pub const fn app_projection_available(self) -> bool {
        matches!(self, Self::AppProvided)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrawingMountedUnitExtensionV1 {
    pub mounted_unit_id: MountedUnitId,
    pub role: DrawingContentRole,
    pub unavailable_projection: DrawingUnavailableProjectionPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DrawingCompositionExtensionV1 {
    schema_version: u16,
    layout_id: CompositionDefinitionId,
    core_schema_version: u16,
    definition_revision: DefinitionRevision,
    app_profile: AppProfileId,
    mounted_units: Vec<DrawingMountedUnitExtensionV1>,
}

impl DrawingCompositionExtensionV1 {
    pub const SCHEMA_VERSION: u16 = 1;

    pub fn new(
        layout_id: CompositionDefinitionId,
        core_schema_version: u16,
        definition_revision: DefinitionRevision,
        app_profile: AppProfileId,
        mounted_units: Vec<DrawingMountedUnitExtensionV1>,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            layout_id,
            core_schema_version,
            definition_revision,
            app_profile,
            mounted_units,
        }
        .normalized()
    }

    pub const fn schema_version(&self) -> u16 {
        self.schema_version
    }

    pub const fn layout_id(&self) -> CompositionDefinitionId {
        self.layout_id
    }

    pub const fn core_schema_version(&self) -> u16 {
        self.core_schema_version
    }

    pub const fn definition_revision(&self) -> DefinitionRevision {
        self.definition_revision
    }

    pub fn app_profile(&self) -> &AppProfileId {
        &self.app_profile
    }

    pub fn mounted_units(&self) -> &[DrawingMountedUnitExtensionV1] {
        &self.mounted_units
    }

    pub fn mounted_unit(&self, id: MountedUnitId) -> Option<&DrawingMountedUnitExtensionV1> {
        self.mounted_units
            .binary_search_by_key(&id, |record| record.mounted_unit_id)
            .ok()
            .map(|index| &self.mounted_units[index])
    }

    pub fn validate_against(
        &self,
        state: &CompositionState,
    ) -> Result<(), DrawingCompositionRejection> {
        let mut diagnostics = Vec::new();
        if self.schema_version != Self::SCHEMA_VERSION
            || self.core_schema_version != ui_composition::CompositionDefinitionV1::SCHEMA_VERSION
        {
            diagnostics.push(Record::error(
                Code::ExtensionSchemaUnsupported,
                Stage::Extension,
                Subject::Layout(self.layout_id),
                "Use Draw extension schema version 1 with composition definition schema version 1.",
            ));
        }
        if self.layout_id != state.definition().id()
            || self.definition_revision != state.definition().revision()
            || self.app_profile.as_str() != DRAWING_APP_PROFILE
        {
            diagnostics.push(Record::error(
                Code::ExtensionCoreMismatch,
                Stage::Extension,
                Subject::Layout(self.layout_id),
                "Match the Draw extension layout, revision, and app profile to the composition definition.",
            ));
        }

        let expected_units = state
            .definition()
            .mounted_units()
            .iter()
            .map(|unit| unit.id)
            .collect::<BTreeSet<_>>();
        let actual_units = self
            .mounted_units
            .iter()
            .map(|unit| unit.mounted_unit_id)
            .collect::<BTreeSet<_>>();
        if expected_units != actual_units || actual_units.len() != self.mounted_units.len() {
            diagnostics.push(Record::error(
                Code::ExtensionCoverageMismatch,
                Stage::Extension,
                Subject::Layout(self.layout_id),
                "Declare exactly one Draw extension record for every core mounted unit.",
            ));
        }

        let actual_roles = self
            .mounted_units
            .iter()
            .map(|unit| unit.role)
            .collect::<BTreeSet<_>>();
        let expected_roles = DrawingContentRole::ALL.into_iter().collect::<BTreeSet<_>>();
        if actual_roles != expected_roles || actual_roles.len() != self.mounted_units.len() {
            diagnostics.push(Record::error(
                Code::ExtensionRoleDuplicate,
                Stage::Extension,
                Subject::Layout(self.layout_id),
                "Bind each required Draw content role to exactly one mounted unit.",
            ));
        }

        for unit in state.definition().mounted_units() {
            let Some(extension) = self.mounted_unit(unit.id) else {
                continue;
            };
            if unit.content().profile().as_str() != extension.role.expected_content_profile() {
                diagnostics.push(Record::error(
                    Code::ExtensionContentProfileMismatch,
                    Stage::Extension,
                    Subject::MountedUnit(unit.id),
                    "Match the Draw content role to the mounted content profile declared by the core definition.",
                ));
            }
        }

        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(DrawingCompositionRejection::new(diagnostics))
        }
    }

    pub fn canonical_ron(&self) -> Result<String, DrawingCompositionRejection> {
        let config = ron::ser::PrettyConfig::new()
            .new_line("\n".to_owned())
            .indentor("  ".to_owned())
            .separator(" ".to_owned())
            .struct_names(false)
            .separate_tuple_members(false)
            .enumerate_arrays(false)
            .compact_arrays(false);
        let mut source =
            ron::ser::to_string_pretty(&self.clone().normalized(), config).map_err(|error| {
                DrawingCompositionRejection::single(
                    Record::error(
                        Code::ExtensionNonCanonical,
                        Stage::Extension,
                        Subject::Layout(self.layout_id),
                        "Fix the Draw extension before canonical encoding.",
                    )
                    .with_context("source_error", error.to_string()),
                )
            })?;
        while source.ends_with('\n') {
            source.pop();
        }
        source.push('\n');
        Ok(source)
    }

    pub fn decode_canonical(source: &str) -> Result<Self, DrawingCompositionRejection> {
        let extension: Self = ron::from_str(source).map_err(|error| {
            DrawingCompositionRejection::single(
                Record::error(
                    Code::ExtensionNonCanonical,
                    Stage::Extension,
                    Subject::General("draw_extension".to_owned()),
                    "Use canonical Draw composition extension RON schema version 1.",
                )
                .with_context("source_error", error.to_string()),
            )
        })?;
        if extension.schema_version != Self::SCHEMA_VERSION
            || extension.clone().normalized() != extension
            || extension.canonical_ron()? != source
        {
            return Err(DrawingCompositionRejection::single(Record::error(
                Code::ExtensionNonCanonical,
                Stage::Extension,
                Subject::Layout(extension.layout_id),
                "Re-encode the Draw extension with its typed canonical codec.",
            )));
        }
        Ok(extension)
    }

    pub fn canonical_payload(
        &self,
    ) -> Result<CanonicalExtensionPayload, DrawingCompositionRejection> {
        let profile = ExtensionProfileId::new(DRAWING_EXTENSION_PROFILE).map_err(|error| {
            extension_error(
                Code::ExtensionSchemaUnsupported,
                self.layout_id,
                "Use the compiled-in Draw extension profile identifier.",
                error,
            )
        })?;
        let version =
            ExtensionSchemaVersion::new(Self::SCHEMA_VERSION.into()).map_err(|error| {
                extension_error(
                    Code::ExtensionSchemaUnsupported,
                    self.layout_id,
                    "Use a non-zero Draw extension schema version.",
                    error,
                )
            })?;
        CanonicalExtensionPayload::new(profile, version, self.canonical_ron()?).map_err(|error| {
            extension_error(
                Code::ExtensionNonCanonical,
                self.layout_id,
                "Provide a canonical Draw extension payload.",
                error,
            )
        })
    }

    fn normalized(mut self) -> Self {
        self.mounted_units
            .sort_by_key(|record| record.mounted_unit_id);
        self
    }
}

pub fn builtin_drawing_composition_extension(
    state: &CompositionState,
) -> Result<DrawingCompositionExtensionV1, DrawingCompositionRejection> {
    let app_profile = AppProfileId::new(DRAWING_APP_PROFILE).map_err(|error| {
        extension_error(
            Code::ExtensionSchemaUnsupported,
            state.definition().id(),
            "Use the compiled-in Draw app compatibility profile.",
            error,
        )
    })?;
    Ok(DrawingCompositionExtensionV1::new(
        state.definition().id(),
        state.definition().schema_version(),
        state.definition().revision(),
        app_profile,
        vec![
            DrawingMountedUnitExtensionV1 {
                mounted_unit_id: DRAWING_TOP_BAR_UNIT_ID,
                role: DrawingContentRole::TopBar,
                unavailable_projection: DrawingUnavailableProjectionPolicy::NeutralOnly,
            },
            DrawingMountedUnitExtensionV1 {
                mounted_unit_id: DRAWING_TOOL_RAIL_UNIT_ID,
                role: DrawingContentRole::ToolRail,
                unavailable_projection: DrawingUnavailableProjectionPolicy::NeutralOnly,
            },
            DrawingMountedUnitExtensionV1 {
                mounted_unit_id: DRAWING_CANVAS_UNIT_ID,
                role: DrawingContentRole::Canvas,
                unavailable_projection: DrawingUnavailableProjectionPolicy::AppProvided,
            },
            DrawingMountedUnitExtensionV1 {
                mounted_unit_id: DRAWING_SUPPORT_UNIT_ID,
                role: DrawingContentRole::SupportPanel,
                unavailable_projection: DrawingUnavailableProjectionPolicy::AppProvided,
            },
        ],
    ))
}

fn extension_error(
    code: Code,
    layout_id: CompositionDefinitionId,
    message: &'static str,
    error: impl std::fmt::Display,
) -> DrawingCompositionRejection {
    DrawingCompositionRejection::single(
        Record::error(code, Stage::Extension, Subject::Layout(layout_id), message)
            .with_context("source_error", error.to_string()),
    )
}
