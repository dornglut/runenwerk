//! File: apps/runenwerk_editor/src/shell/compositions/mod.rs
//! Purpose: Built-in Runenwerk Workbench composition manifests.

pub mod full_editor;
pub mod headless_validation;
pub mod material_lab;
pub mod profiles;
pub mod ui_designer;

use editor_shell::{HostCapabilityPolicy, ProfileRef, ToolSuiteId, WorkbenchCompositionManifest};

use super::workbench_host::RunenwerkWorkbenchComposition;

pub(crate) fn composition_manifest_for(
    composition: RunenwerkWorkbenchComposition,
    installed_suites: Vec<ToolSuiteId>,
) -> WorkbenchCompositionManifest {
    match composition {
        RunenwerkWorkbenchComposition::FullEditor | RunenwerkWorkbenchComposition::Constrained => {
            full_editor::composition_manifest(installed_suites, host_policy_for(composition))
        }
        RunenwerkWorkbenchComposition::MaterialLab => {
            material_lab::composition_manifest(installed_suites, host_policy_for(composition))
        }
        RunenwerkWorkbenchComposition::UiDesigner => {
            ui_designer::composition_manifest(installed_suites, host_policy_for(composition))
        }
        RunenwerkWorkbenchComposition::HeadlessValidation => {
            headless_validation::composition_manifest(
                installed_suites,
                host_policy_for(composition),
            )
        }
        RunenwerkWorkbenchComposition::Custom => WorkbenchCompositionManifest {
            composition_ref: profile_ref("runenwerk.workbench.custom"),
            label: "Custom Workbench".to_string(),
            installed_suites,
            profile_refs: vec![profiles::custom_profile_ref()],
            default_profile_ref: profiles::custom_profile_ref(),
            host_policy: host_policy_for(composition),
        },
    }
}

pub(crate) fn host_policy_for(composition: RunenwerkWorkbenchComposition) -> HostCapabilityPolicy {
    match composition {
        RunenwerkWorkbenchComposition::FullEditor
        | RunenwerkWorkbenchComposition::MaterialLab
        | RunenwerkWorkbenchComposition::UiDesigner
        | RunenwerkWorkbenchComposition::Custom => HostCapabilityPolicy::allow_all(),
        RunenwerkWorkbenchComposition::HeadlessValidation
        | RunenwerkWorkbenchComposition::Constrained => HostCapabilityPolicy::deny_all(),
    }
}

fn profile_ref(value: &str) -> ProfileRef {
    ProfileRef::new(value).expect("compiled-in Workbench profile ref should be valid")
}
