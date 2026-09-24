//! Editor composition bundle persistence and read-only legacy source probing.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use editor_shell::{
    EDITOR_COMPOSITION_EXTENSION_PROFILE, EditorCompositionExtensionSnapshot,
    EditorCompositionExtensionV1, EditorCompositionRuntime,
};
use ui_composition::{
    AppProfileId, AppSchemaVersion, CompositionBundleRepository, CompositionCompatibility,
    CompositionCompatibilityRequirement, CompositionLayoutScope, CompositionSourceSchema,
    CompositionState, LayoutDisplayName, probe_composition_source,
};

const EDITOR_COMPOSITION_APP_PROFILE: &str = "runenwerk.editor";
const EDITOR_COMPOSITION_APP_SCHEMA_VERSION: u32 = 1;

pub fn default_composition_layout_root_for_profile(
    profile_id: editor_shell::WorkspaceProfileId,
) -> PathBuf {
    PathBuf::from("editor-scenes")
        .join("compositions")
        .join(format!("profile-{}", profile_id.raw()))
}

pub fn save_editor_composition_layout(
    root: &Path,
    runtime: &EditorCompositionRuntime,
) -> Result<ui_composition::CompositionActivation> {
    let repository = CompositionBundleRepository::new(root);
    let expected_generation = repository
        .current_pointer()
        .context("read active editor composition generation")?
        .map(|pointer| pointer.active);
    let compatibility = editor_compatibility()?;
    let display_name = LayoutDisplayName::new(format!(
        "Runenwerk editor profile {}",
        runtime.extension().workspace_profile_raw()
    ))
    .context("form editor composition display name")?;
    let promotion = runtime.composition().promote_definition(
        runtime.composition().definition().id(),
        display_name,
        CompositionLayoutScope::User,
        compatibility,
    );
    let snapshot = EditorCompositionExtensionSnapshot::new(runtime.extension());
    let candidate = promotion
        .snapshot_bundle(&snapshot)
        .context("snapshot linked editor composition bundle")?;
    repository
        .activate(&candidate, expected_generation.as_ref())
        .map_err(describe_composition_persistence_rejection)
        .context("activate editor composition generation")
}

pub fn load_editor_composition_layout(root: &Path) -> Result<EditorCompositionRuntime> {
    let repository = CompositionBundleRepository::new(root);
    let requirement = editor_compatibility_requirement()?;
    let loaded = repository
        .load_active(Some(&requirement))
        .context("load active editor composition generation")?;
    let bundle = loaded.bundle;
    let extension = bundle
        .extensions()
        .iter()
        .find(|extension| {
            extension.link.identity.profile.as_str() == EDITOR_COMPOSITION_EXTENSION_PROFILE
        })
        .ok_or_else(|| anyhow!("editor composition extension is missing"))?;
    if bundle.extensions().len() != 1 {
        return Err(anyhow!(
            "editor composition bundle must contain exactly one editor extension"
        ));
    }
    let extension = EditorCompositionExtensionV1::decode_canonical(&extension.payload_ron)
        .map_err(|error| anyhow!(error.to_string()))?;
    let state = CompositionState::form(bundle.core().definition.clone())
        .map_err(|error| anyhow!(error.to_string()))?;
    EditorCompositionRuntime::install(state, extension).map_err(|error| anyhow!(error.to_string()))
}

pub fn probe_legacy_layout_path(path: &Path) -> Result<CompositionSourceSchema> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("read composition source probe at {}", path.display()))?;
    probe_composition_source(&source).map_err(|error| anyhow!(error.to_string()))
}

fn editor_compatibility() -> Result<CompositionCompatibility> {
    let profile = AppProfileId::new(EDITOR_COMPOSITION_APP_PROFILE)
        .map_err(|error| anyhow!(error.to_string()))?;
    let version = AppSchemaVersion::new(EDITOR_COMPOSITION_APP_SCHEMA_VERSION)
        .map_err(|error| anyhow!(error.to_string()))?;
    CompositionCompatibility::new(profile, version, version)
        .map_err(|error| anyhow!(error.to_string()))
}

fn editor_compatibility_requirement() -> Result<CompositionCompatibilityRequirement> {
    Ok(CompositionCompatibilityRequirement {
        app_profile: AppProfileId::new(EDITOR_COMPOSITION_APP_PROFILE)
            .map_err(|error| anyhow!(error.to_string()))?,
        app_schema_version: AppSchemaVersion::new(EDITOR_COMPOSITION_APP_SCHEMA_VERSION)
            .map_err(|error| anyhow!(error.to_string()))?,
    })
}

fn describe_composition_persistence_rejection(
    rejection: ui_composition::CompositionPersistenceRejection,
) -> anyhow::Error {
    let diagnostics = rejection
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            format!(
                "{:?} {:?} {:?}: {} {:?}",
                diagnostic.code(),
                diagnostic.stage(),
                diagnostic.subject(),
                diagnostic.message(),
                diagnostic.context()
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    anyhow!("composition persistence rejected: {diagnostics}")
}

#[cfg(test)]
pub fn write_workspace_layout(path: &Path, workspace: &editor_shell::WorkspaceState) -> Result<()> {
    let runtime =
        editor_shell::import_legacy_workspace(editor_shell::SCENE_WORKSPACE_PROFILE_ID, workspace)
            .map_err(|error| anyhow!(error.to_string()))?;
    save_editor_composition_layout(path, &runtime).map(|_| ())
}

#[cfg(test)]
// Legacy/test compatibility reader: reverse composition-to-workspace loading is unsupported.
pub fn read_workspace_layout_legacy_no_registry(
    _path: &Path,
) -> Result<editor_shell::WorkspaceState> {
    Err(anyhow!(
        "reverse composition-to-workspace loading is intentionally unsupported"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{
        SCENE_WORKSPACE_PROFILE_ID, WorkspaceIdentityAllocator, default_workspace_profile_registry,
        import_legacy_workspace,
    };

    #[test]
    fn composition_linked_editor_bundle_round_trips_atomically() {
        let profiles = default_workspace_profile_registry();
        let profile = profiles.profile(SCENE_WORKSPACE_PROFILE_ID).unwrap();
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = profile.build_default_workspace_state(workspace_id, &mut allocator);
        let runtime = import_legacy_workspace(profile.id, &workspace).unwrap();
        let directory = tempfile::tempdir().unwrap();

        save_editor_composition_layout(directory.path(), &runtime).unwrap();
        let loaded = load_editor_composition_layout(directory.path()).unwrap();

        assert_eq!(loaded, runtime);
    }

    #[test]
    fn composition_legacy_probe_rejects_without_modifying_source() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("legacy.workspace.ron");
        let source = "(version:5)\n";
        std::fs::write(&path, source).unwrap();

        assert!(probe_legacy_layout_path(&path).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), source);
    }
}
