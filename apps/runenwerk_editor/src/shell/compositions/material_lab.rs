//! File: apps/runenwerk_editor/src/shell/compositions/material_lab.rs
//! Purpose: Material Lab Workbench composition manifest.

use editor_shell::{HostCapabilityPolicy, ProfileRef, ToolSuiteId, WorkbenchCompositionManifest};

use super::profiles;

pub(crate) fn composition_manifest(
    installed_suites: Vec<ToolSuiteId>,
    host_policy: HostCapabilityPolicy,
) -> WorkbenchCompositionManifest {
    WorkbenchCompositionManifest {
        composition_ref: profile_ref("runenwerk.workbench.material_lab"),
        label: "Material Lab".to_string(),
        installed_suites,
        profile_refs: vec![profiles::material_profile_ref()],
        default_profile_ref: profiles::material_profile_ref(),
        host_policy,
    }
}

fn profile_ref(value: &str) -> ProfileRef {
    ProfileRef::new(value).expect("compiled-in Workbench composition ref should be valid")
}
