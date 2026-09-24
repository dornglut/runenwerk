use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use ui_composition::{
    CanonicalExtensionPayload, CompositionDefinitionId, CompositionExtensionSnapshotPort,
    CompositionState, DefinitionRevision, ExtensionProfileId, ExtensionSchemaVersion,
    MountedUnitId, RegionId, StateRevision,
};

use super::{
    EditorCompositionDiagnosticCode as Code, EditorCompositionDiagnosticRecord as Record,
    EditorCompositionDiagnosticStage as Stage, EditorCompositionDiagnosticSubject as Subject,
    EditorCompositionRejection,
};
use crate::{ToolSurfaceStableKey, tool_surface_kind_from_definition_key};

pub const EDITOR_COMPOSITION_EXTENSION_PROFILE: &str = "runenwerk.editor.layout";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorMountedUnitExtensionV1 {
    pub mounted_unit_id: MountedUnitId,
    pub panel_instance_raw: u64,
    pub compatibility_surface_raw: u64,
    pub stable_content_key: String,
    pub panel_kind_key: String,
    pub viewport_instance_raw: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorRegionExtensionV1 {
    pub region_id: RegionId,
    pub compatibility_host_raw: u64,
    pub tab_stack_raw: Option<u64>,
    pub locked_content_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorRootExtensionV1 {
    pub root_id_raw: u64,
    pub compatibility_host_raw: u64,
    pub floating_bounds_milli: Option<[i64; 4]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditorCompositionExtensionV1 {
    schema_version: u16,
    layout_id: CompositionDefinitionId,
    definition_revision: DefinitionRevision,
    workspace_profile_raw: u64,
    mounted_units: Vec<EditorMountedUnitExtensionV1>,
    regions: Vec<EditorRegionExtensionV1>,
    roots: Vec<EditorRootExtensionV1>,
}

impl EditorCompositionExtensionV1 {
    pub const SCHEMA_VERSION: u16 = 1;

    pub fn new(
        layout_id: CompositionDefinitionId,
        definition_revision: DefinitionRevision,
        workspace_profile_raw: u64,
        mounted_units: Vec<EditorMountedUnitExtensionV1>,
        regions: Vec<EditorRegionExtensionV1>,
        roots: Vec<EditorRootExtensionV1>,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            layout_id,
            definition_revision,
            workspace_profile_raw,
            mounted_units,
            regions,
            roots,
        }
        .normalized()
    }

    pub const fn layout_id(&self) -> CompositionDefinitionId {
        self.layout_id
    }

    pub const fn definition_revision(&self) -> DefinitionRevision {
        self.definition_revision
    }

    pub const fn workspace_profile_raw(&self) -> u64 {
        self.workspace_profile_raw
    }

    pub fn mounted_units(&self) -> &[EditorMountedUnitExtensionV1] {
        &self.mounted_units
    }

    pub fn regions(&self) -> &[EditorRegionExtensionV1] {
        &self.regions
    }

    pub fn roots(&self) -> &[EditorRootExtensionV1] {
        &self.roots
    }

    pub fn region(&self, id: RegionId) -> Option<&EditorRegionExtensionV1> {
        self.regions
            .binary_search_by_key(&id, |record| record.region_id)
            .ok()
            .map(|index| &self.regions[index])
    }

    pub fn root(&self, raw: u64) -> Option<&EditorRootExtensionV1> {
        self.roots
            .binary_search_by_key(&raw, |record| record.root_id_raw)
            .ok()
            .map(|index| &self.roots[index])
    }

    pub fn mounted_unit(&self, id: MountedUnitId) -> Option<&EditorMountedUnitExtensionV1> {
        self.mounted_units
            .binary_search_by_key(&id, |record| record.mounted_unit_id)
            .ok()
            .map(|index| &self.mounted_units[index])
    }

    pub fn validate_against(
        &self,
        state: &CompositionState,
    ) -> Result<(), EditorCompositionRejection> {
        let mut diagnostics = Vec::new();
        if self.schema_version != Self::SCHEMA_VERSION {
            diagnostics.push(Record::error(
                Code::ExtensionSchemaUnsupported,
                Stage::Extension,
                Subject::Layout(self.layout_id.raw()),
                "Use editor composition extension schema version 1.",
            ));
        }
        if self.layout_id != state.definition().id()
            || self.definition_revision != state.definition().revision()
        {
            diagnostics.push(Record::error(
                Code::ExtensionCoreMismatch,
                Stage::Extension,
                Subject::Layout(self.layout_id.raw()),
                "Match the editor extension layout ID and revision to the composition definition.",
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
        let expected_regions = state
            .definition()
            .regions()
            .iter()
            .map(|region| region.id)
            .collect::<BTreeSet<_>>();
        let actual_regions = self
            .regions
            .iter()
            .map(|region| region.region_id)
            .collect::<BTreeSet<_>>();
        let expected_roots = state
            .definition()
            .roots()
            .iter()
            .map(|root| root.id.raw())
            .collect::<BTreeSet<_>>();
        let actual_roots = self
            .roots
            .iter()
            .map(|root| root.root_id_raw)
            .collect::<BTreeSet<_>>();
        if expected_units != actual_units
            || expected_regions != actual_regions
            || expected_roots != actual_roots
            || actual_units.len() != self.mounted_units.len()
            || actual_regions.len() != self.regions.len()
            || actual_roots.len() != self.roots.len()
        {
            diagnostics.push(Record::error(
                Code::ExtensionCoverageMismatch,
                Stage::Extension,
                Subject::Layout(self.layout_id.raw()),
                "Declare exactly one editor extension record for every core mounted unit, region, and root.",
            ));
        }
        self.validate_compatibility_identities(&mut diagnostics);
        for unit in state.definition().mounted_units() {
            let Some(extension) = self.mounted_unit(unit.id) else {
                continue;
            };
            if unit.content().profile().as_str() != extension.stable_content_key {
                diagnostics.push(Record::error(
                    Code::ExtensionCoreMismatch,
                    Stage::Extension,
                    Subject::MountedUnit(unit.id.raw()),
                    "Match the editor stable content key to the core mounted-content profile.",
                ));
            }
        }
        if diagnostics.is_empty() {
            Ok(())
        } else {
            Err(EditorCompositionRejection::new(diagnostics))
        }
    }

    fn validate_compatibility_identities(&self, diagnostics: &mut Vec<Record>) {
        let mut panel_ids = BTreeSet::new();
        let mut surface_ids = BTreeSet::new();
        let mut viewport_ids = BTreeSet::new();
        for unit in &self.mounted_units {
            let panel_id_valid =
                unit.panel_instance_raw != 0 && panel_ids.insert(unit.panel_instance_raw);
            let surface_id_valid = unit.compatibility_surface_raw != 0
                && surface_ids.insert(unit.compatibility_surface_raw);
            let viewport_id_valid = unit
                .viewport_instance_raw
                .is_none_or(|raw| raw != 0 && viewport_ids.insert(raw));
            let valid = panel_id_valid
                && surface_id_valid
                && viewport_id_valid
                && ToolSurfaceStableKey::new(unit.stable_content_key.clone()).is_ok()
                && tool_surface_kind_from_definition_key(&unit.panel_kind_key).is_some();
            if !valid {
                diagnostics.push(Record::error(
                    Code::ExtensionIdentityInvalid,
                    Stage::Extension,
                    Subject::MountedUnit(unit.mounted_unit_id.raw()),
                    "Use non-zero, unique editor compatibility IDs and registered stable content and panel-kind keys.",
                ));
            }
        }

        let mut host_ids = BTreeSet::new();
        let mut tab_stack_ids = BTreeSet::new();
        for region in &self.regions {
            let host_id_valid = region.compatibility_host_raw != 0
                && host_ids.insert(region.compatibility_host_raw);
            let tab_stack_id_valid = region
                .tab_stack_raw
                .is_none_or(|raw| raw != 0 && tab_stack_ids.insert(raw));
            let valid = host_id_valid
                && tab_stack_id_valid
                && region
                    .locked_content_key
                    .as_ref()
                    .is_none_or(|key| ToolSurfaceStableKey::new(key.clone()).is_ok());
            if !valid {
                diagnostics.push(Record::error(
                    Code::ExtensionIdentityInvalid,
                    Stage::Extension,
                    Subject::Region(region.region_id.raw()),
                    "Use non-zero, unique editor host and tab-stack compatibility IDs and valid optional lock keys.",
                ));
            }
        }

        let mut root_ids = BTreeSet::new();
        let mut root_host_ids = BTreeSet::new();
        for root in &self.roots {
            if root.root_id_raw == 0
                || !root_ids.insert(root.root_id_raw)
                || root.compatibility_host_raw == 0
                || !root_host_ids.insert(root.compatibility_host_raw)
            {
                diagnostics.push(Record::error(
                    Code::ExtensionIdentityInvalid,
                    Stage::Extension,
                    Subject::Layout(self.layout_id.raw()),
                    "Use non-zero, unique editor root and root-host compatibility IDs.",
                ));
            }
        }

        if self.workspace_profile_raw == 0 {
            diagnostics.push(Record::error(
                Code::ExtensionIdentityInvalid,
                Stage::Extension,
                Subject::Profile(self.workspace_profile_raw.to_string()),
                "Use a non-zero editor workspace-profile compatibility ID.",
            ));
        }
    }

    pub fn canonical_ron(&self) -> Result<String, EditorCompositionRejection> {
        let normalized = self.clone().normalized();
        let config = ron::ser::PrettyConfig::new()
            .new_line("\n".to_owned())
            .indentor("  ".to_owned())
            .separator(" ".to_owned())
            .struct_names(false)
            .separate_tuple_members(false)
            .enumerate_arrays(false)
            .compact_arrays(false);
        let mut source = ron::ser::to_string_pretty(&normalized, config).map_err(|error| {
            EditorCompositionRejection::single(
                Record::error(
                    Code::ExtensionNonCanonical,
                    Stage::Extension,
                    Subject::Layout(self.layout_id.raw()),
                    "Fix the editor extension before canonical encoding.",
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

    pub fn decode_canonical(source: &str) -> Result<Self, EditorCompositionRejection> {
        let extension: Self = ron::from_str(source).map_err(|error| {
            EditorCompositionRejection::single(
                Record::error(
                    Code::ExtensionNonCanonical,
                    Stage::Extension,
                    Subject::General("editor_extension".to_owned()),
                    "Use canonical editor composition extension RON schema version 1.",
                )
                .with_context("source_error", error.to_string()),
            )
        })?;
        if extension.schema_version != Self::SCHEMA_VERSION
            || extension.clone().normalized() != extension
            || extension.canonical_ron()? != source
        {
            return Err(EditorCompositionRejection::single(Record::error(
                Code::ExtensionNonCanonical,
                Stage::Extension,
                Subject::Layout(extension.layout_id.raw()),
                "Re-encode the editor extension with its typed canonical codec.",
            )));
        }
        Ok(extension)
    }

    pub fn canonical_payload(
        &self,
    ) -> Result<CanonicalExtensionPayload, EditorCompositionRejection> {
        let profile =
            ExtensionProfileId::new(EDITOR_COMPOSITION_EXTENSION_PROFILE).map_err(|error| {
                EditorCompositionRejection::single(
                    Record::error(
                        Code::ExtensionSchemaUnsupported,
                        Stage::Extension,
                        Subject::Profile(EDITOR_COMPOSITION_EXTENSION_PROFILE.to_owned()),
                        "Use the compiled-in editor extension profile identifier.",
                    )
                    .with_context("source_error", error.to_string()),
                )
            })?;
        let version =
            ExtensionSchemaVersion::new(Self::SCHEMA_VERSION.into()).map_err(|error| {
                EditorCompositionRejection::single(
                    Record::error(
                        Code::ExtensionSchemaUnsupported,
                        Stage::Extension,
                        Subject::Profile(EDITOR_COMPOSITION_EXTENSION_PROFILE.to_owned()),
                        "Use a non-zero editor extension schema version.",
                    )
                    .with_context("source_error", error.to_string()),
                )
            })?;
        CanonicalExtensionPayload::new(profile, version, self.canonical_ron()?).map_err(|error| {
            EditorCompositionRejection::single(
                Record::error(
                    Code::ExtensionNonCanonical,
                    Stage::Extension,
                    Subject::Layout(self.layout_id.raw()),
                    "Provide a canonical editor extension payload.",
                )
                .with_context("source_error", error.to_string()),
            )
        })
    }

    fn normalized(mut self) -> Self {
        self.mounted_units
            .sort_by_key(|record| record.mounted_unit_id);
        self.regions.sort_by_key(|record| record.region_id);
        self.roots.sort_by_key(|record| record.root_id_raw);
        self
    }

    pub(crate) fn relinked_to_definition(
        &self,
        layout_id: CompositionDefinitionId,
        definition_revision: DefinitionRevision,
    ) -> Self {
        let mut candidate = self.clone();
        candidate.layout_id = layout_id;
        candidate.definition_revision = definition_revision;
        candidate.normalized()
    }

    fn relinked_for_promotion(
        &self,
        layout_id: CompositionDefinitionId,
        source_revision: StateRevision,
    ) -> Self {
        self.relinked_to_definition(layout_id, DefinitionRevision::new(source_revision.raw()))
    }
}

pub struct EditorCompositionExtensionSnapshot<'a> {
    extension: &'a EditorCompositionExtensionV1,
}

impl<'a> EditorCompositionExtensionSnapshot<'a> {
    pub const fn new(extension: &'a EditorCompositionExtensionV1) -> Self {
        Self { extension }
    }
}

impl CompositionExtensionSnapshotPort for EditorCompositionExtensionSnapshot<'_> {
    fn snapshot_extensions(
        &self,
        layout_id: CompositionDefinitionId,
        source_revision: StateRevision,
    ) -> Result<Vec<CanonicalExtensionPayload>, ui_composition::CompositionPersistenceRejection>
    {
        self.extension
            .relinked_for_promotion(layout_id, source_revision)
            .canonical_payload()
            .map(|payload| vec![payload])
            .map_err(|rejection| {
                ui_composition::CompositionPersistenceRejection::single(
                    ui_composition::CompositionPersistenceDiagnosticRecord::error(
                        ui_composition::CompositionPersistenceDiagnosticCode::CanonicalEncodeFailed,
                        ui_composition::CompositionPersistenceDiagnosticStage::Promotion,
                        ui_composition::CompositionPersistenceDiagnosticSubject::Layout(layout_id),
                        format!("Snapshot a valid editor extension: {rejection}"),
                    ),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::Code;
    use crate::{WorkspaceIdentityAllocator, default_workspace_profile_registry};

    #[test]
    fn composition_extension_rejects_invalid_and_duplicate_compatibility_identities() {
        let registry = default_workspace_profile_registry();
        let profile = registry.default_profile().unwrap();
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);
        let runtime = super::super::import_legacy_workspace(profile.id, &workspace).unwrap();
        let mut extension = runtime.extension().clone();
        let duplicate_panel = extension.mounted_units[0].panel_instance_raw;
        extension.mounted_units[1].panel_instance_raw = duplicate_panel;
        extension.mounted_units[1].stable_content_key = "invalid-key".to_owned();
        extension.workspace_profile_raw = 0;

        let rejection = extension
            .validate_against(runtime.composition())
            .expect_err("invalid extension identities must reject atomic installation");

        assert!(
            rejection
                .diagnostics()
                .iter()
                .any(|diagnostic| { diagnostic.code() == Code::ExtensionIdentityInvalid })
        );
    }
}
