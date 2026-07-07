use super::screen::UiTypedScreenId;

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
