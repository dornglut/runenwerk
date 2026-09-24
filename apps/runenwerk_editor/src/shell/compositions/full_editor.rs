//! File: apps/runenwerk_editor/src/shell/compositions/full_editor.rs
//! Purpose: Full editor Workbench composition manifest.

use editor_shell::{HostCapabilityPolicy, ProfileRef, ToolSuiteId, WorkbenchCompositionManifest};

use super::profiles;

pub(crate) fn composition_manifest(
    installed_suites: Vec<ToolSuiteId>,
    host_policy: HostCapabilityPolicy,
) -> WorkbenchCompositionManifest {
    let profile_manifests = profiles::full_editor_profiles();
    WorkbenchCompositionManifest {
        composition_ref: profile_ref("runenwerk.workbench.full_editor"),
        label: "Full Editor".to_string(),
        installed_suites,
        profile_refs: profile_manifests
            .iter()
            .map(|profile| profile.profile_ref.clone())
            .collect(),
        default_profile_ref: profiles::scene_profile_ref(),
        host_policy,
    }
}

fn profile_ref(value: &str) -> ProfileRef {
    ProfileRef::new(value).expect("compiled-in Workbench composition ref should be valid")
}
