//! File: domain/ui/ui_compiler/src/lib.rs
//! Crate: ui_compiler

use ui_artifacts::{UiRuntimeArtifact, UiRuntimeArtifactManifest, UiRuntimeArtifactTables};
use ui_program::UiProgram;

#[derive(Clone, Debug, Default)]
pub struct UiCompiler;

impl UiCompiler {
    pub fn compile(&self, program: &UiProgram) -> UiRuntimeArtifact {
        let manifest = UiRuntimeArtifactManifest::from_program(program);
        let tables = UiRuntimeArtifactTables {
            layout: program.graphs.layout.constraints.clone(),
            state: program.graphs.state.requirements.clone(),
            binding: program.graphs.binding.bindings.clone(),
            interaction: program.graphs.interaction.handlers.clone(),
            visual: program.graphs.visual.operators.clone(),
            accessibility: program.graphs.accessibility.nodes.clone(),
            inspection: program.graphs.inspection.entries.clone(),
        };
        UiRuntimeArtifact::new(manifest, tables)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ui_program::{UiProgram, UiProgramId, UiProgramVersion};

    #[test]
    fn compiler_contract_lowers_program_graphs_to_runtime_tables() {
        let mut program = UiProgram::new(UiProgramId::new("inspector"), UiProgramVersion::new(2));
        program
            .graphs
            .layout
            .constraints
            .push("root.fill".to_owned());
        program
            .graphs
            .visual
            .operators
            .push("label.text".to_owned());

        let artifact = UiCompiler.compile(&program);

        assert_eq!(artifact.manifest.cache_key, "ui-program:inspector:2");
        assert_eq!(artifact.tables.layout, ["root.fill"]);
        assert_eq!(artifact.tables.visual, ["label.text"]);
    }
}
