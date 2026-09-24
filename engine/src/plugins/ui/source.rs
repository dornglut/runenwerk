use super::screen::UiTypedScreenId;

use ui_artifacts::UiRuntimeArtifact;
use ui_controls::ControlPackageRegistrySnapshot;
use ui_definition::UiNodeDefinition;
use ui_program::{RouteId, UiProgram, UiProgramSource, UiProgramSourceId, UiProgramSourceMapEntry};
use ui_program_lowering::{
    UiProgramFormationReport, form_ui_program_report_from_node_with_registry_snapshot,
};

#[derive(Clone, Debug, PartialEq)]
pub struct UiTypedSource {
    screen_id: UiTypedScreenId,
    source_id: UiProgramSourceId,
    root: UiNodeDefinition,
}

impl UiTypedSource {
    pub fn new(
        screen_id: UiTypedScreenId,
        source_id: UiProgramSourceId,
        root: UiNodeDefinition,
    ) -> Self {
        Self {
            screen_id,
            source_id,
            root,
        }
    }

    pub fn screen_id(&self) -> &UiTypedScreenId {
        &self.screen_id
    }

    pub fn source_id(&self) -> &UiProgramSourceId {
        &self.source_id
    }

    pub fn root(&self) -> &UiNodeDefinition {
        &self.root
    }

    pub fn into_root(self) -> UiNodeDefinition {
        self.root
    }

    pub fn program_id(&self) -> String {
        format!("{}.program", self.screen_id.as_str())
    }

    pub fn lower_with_registry_snapshot(
        &self,
        snapshot: &ControlPackageRegistrySnapshot,
    ) -> UiTypedSourceLoweringReport {
        let formation = form_ui_program_report_from_node_with_registry_snapshot(
            self.program_id(),
            self.source_id.as_str(),
            &self.root,
            snapshot,
        );

        UiTypedSourceLoweringReport {
            screen_id: self.screen_id.clone(),
            source_id: self.source_id.clone(),
            formation,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiTypedSourceLoweringReport {
    screen_id: UiTypedScreenId,
    source_id: UiProgramSourceId,
    formation: UiProgramFormationReport,
}

impl UiTypedSourceLoweringReport {
    pub fn screen_id(&self) -> &UiTypedScreenId {
        &self.screen_id
    }

    pub fn source_id(&self) -> &UiProgramSourceId {
        &self.source_id
    }

    pub fn formation(&self) -> &UiProgramFormationReport {
        &self.formation
    }

    pub fn program(&self) -> &UiProgram {
        &self.formation.program
    }

    pub fn sources(&self) -> &[UiProgramSource] {
        &self.formation.program.sources
    }

    pub fn source_map_entries(&self) -> &[UiProgramSourceMapEntry] {
        &self.formation.program.source_map
    }

    pub fn route_ids(&self) -> Vec<&RouteId> {
        self.formation
            .program
            .graphs
            .interaction
            .handlers
            .iter()
            .map(|handler| &handler.route)
            .collect()
    }

    pub fn has_route(&self, route: &RouteId) -> bool {
        self.formation
            .program
            .graphs
            .interaction
            .handlers
            .iter()
            .any(|handler| &handler.route == route)
    }

    pub fn passed(&self) -> bool {
        self.formation.passed()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiRuntimeSourceProgramFacts {
    screen_id: String,
    source_id: String,
    program_id: String,
    artifact_id: String,
    source_count: usize,
    source_map_count: usize,
    route_count: usize,
    control_count: usize,
    state_requirement_count: usize,
    binding_count: usize,
}

impl UiRuntimeSourceProgramFacts {
    pub fn from_lowering_report(
        report: &UiTypedSourceLoweringReport,
        artifact: &UiRuntimeArtifact,
    ) -> Self {
        Self {
            screen_id: report.screen_id().as_str().to_owned(),
            source_id: report.source_id().as_str().to_owned(),
            program_id: report.program().id.as_str().to_owned(),
            artifact_id: artifact.manifest.artifact_id.as_str().to_owned(),
            source_count: report.sources().len(),
            source_map_count: report.source_map_entries().len(),
            route_count: report.route_ids().len(),
            control_count: artifact.tables.controls.rows.len(),
            state_requirement_count: artifact.tables.state.rows.len(),
            binding_count: artifact.tables.binding_snapshots.rows.len(),
        }
    }

    pub fn screen_id(&self) -> &str {
        &self.screen_id
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn program_id(&self) -> &str {
        &self.program_id
    }

    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    pub fn source_count(&self) -> usize {
        self.source_count
    }

    pub fn source_map_count(&self) -> usize {
        self.source_map_count
    }

    pub fn route_count(&self) -> usize {
        self.route_count
    }

    pub fn control_count(&self) -> usize {
        self.control_count
    }

    pub fn state_requirement_count(&self) -> usize {
        self.state_requirement_count
    }

    pub fn binding_count(&self) -> usize {
        self.binding_count
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiRuntimeEvaluationInput {
    facts: UiRuntimeSourceProgramFacts,
    artifact: UiRuntimeArtifact,
}

impl UiRuntimeEvaluationInput {
    pub fn from_lowering_report(report: &UiTypedSourceLoweringReport) -> Self {
        let artifact = UiRuntimeArtifact::from_program(report.program());
        let facts = UiRuntimeSourceProgramFacts::from_lowering_report(report, &artifact);
        Self { facts, artifact }
    }

    pub fn facts(&self) -> &UiRuntimeSourceProgramFacts {
        &self.facts
    }

    pub fn artifact(&self) -> &UiRuntimeArtifact {
        &self.artifact
    }
}
