use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use editor_persistence::{decode_ron, encode_ron_pretty};
use serde::Deserialize;

use editor_shell::{
    PERSISTED_WORKSPACE_STATE_VERSION_V1, PERSISTED_WORKSPACE_STATE_VERSION_V2,
    PERSISTED_WORKSPACE_STATE_VERSION_V3, PERSISTED_WORKSPACE_STATE_VERSION_V4,
    PersistedWorkspaceStateV1, PersistedWorkspaceStateV2, PersistedWorkspaceStateV3,
    PersistedWorkspaceStateV4, WorkspaceProfileId, WorkspaceState, compact_empty_tab_stack_areas,
    default_workspace_profile_registry,
};

#[derive(Debug, Deserialize)]
struct PersistedWorkspaceVersionProbe {
    version: u32,
}

#[derive(Debug)]
pub struct WorkspaceLayoutReadResult {
    pub workspace_state: WorkspaceState,
    pub workspace_profile_id: Option<WorkspaceProfileId>,
    pub layout_template: Option<String>,
    pub layout_template_version: Option<u32>,
    pub last_saved_at_unix_seconds: Option<u64>,
}

const DEFAULT_WORKSPACE_LAYOUT_DIR: &str = "editor-scenes/workspaces";

pub fn default_workspace_layout_dir() -> PathBuf {
    PathBuf::from(DEFAULT_WORKSPACE_LAYOUT_DIR)
}

pub fn workspace_layout_path_for_profile(root: &Path, profile_id: WorkspaceProfileId) -> PathBuf {
    root.join(format!("profile-{}.workspace.ron", profile_id.raw()))
}

pub fn default_workspace_layout_path_for_profile(profile_id: WorkspaceProfileId) -> PathBuf {
    workspace_layout_path_for_profile(&default_workspace_layout_dir(), profile_id)
}

pub fn legacy_workspace_layout_path_for_scene(scene_path: &Path) -> PathBuf {
    let file_name = scene_path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.workspace.ron"))
        .unwrap_or_else(|| "scene.workspace.ron".to_string());
    scene_path.with_file_name(file_name)
}

pub fn write_workspace_layout(path: &Path, workspace_state: &WorkspaceState) -> Result<()> {
    write_workspace_layout_with_profile(path, workspace_state, None)
}

pub fn write_workspace_layout_for_profile(
    path: &Path,
    workspace_state: &WorkspaceState,
    profile_id: WorkspaceProfileId,
) -> Result<()> {
    write_workspace_layout_with_profile(path, workspace_state, Some(profile_id))
}

fn write_workspace_layout_with_profile(
    path: &Path,
    workspace_state: &WorkspaceState,
    profile_id: Option<WorkspaceProfileId>,
) -> Result<()> {
    let workspace_state = compact_empty_tab_stack_areas(workspace_state)
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to normalize workspace layout before saving")?;
    let mut persisted = workspace_state.to_persisted_v4();
    persisted.workspace_profile_id = profile_id.map(|id| id.raw());
    if let Some(profile_id) = profile_id
        && let Some(profile) = default_workspace_profile_registry().profile(profile_id)
    {
        persisted.layout_template = Some(profile.default_layout_template.contract_id().to_string());
        persisted.layout_template_version =
            Some(profile.default_layout_template.contract_version());
    }
    persisted.last_saved_at_unix_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|duration| duration.as_secs());
    let ron =
        encode_ron_pretty(&persisted).context("failed to encode persisted workspace layout")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write workspace layout: {}", path.display()))
}

pub fn read_workspace_layout(path: &Path) -> Result<WorkspaceState> {
    Ok(read_workspace_layout_with_metadata(path)?.workspace_state)
}

pub fn read_workspace_layout_with_metadata(path: &Path) -> Result<WorkspaceLayoutReadResult> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read workspace layout: {}", path.display()))?;
    let probe: PersistedWorkspaceVersionProbe =
        decode_ron(&source).context("failed to decode persisted workspace layout version")?;
    let (
        workspace_state,
        workspace_profile_id,
        layout_template,
        layout_template_version,
        last_saved_at_unix_seconds,
    ) = match probe.version {
        PERSISTED_WORKSPACE_STATE_VERSION_V1 => {
            let persisted: PersistedWorkspaceStateV1 =
                decode_ron(&source).context("failed to decode v1 persisted workspace layout")?;
            (
                WorkspaceState::from_persisted_v1(persisted),
                None,
                None,
                None,
                None,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V2 => {
            let persisted: PersistedWorkspaceStateV2 =
                decode_ron(&source).context("failed to decode v2 persisted workspace layout")?;
            (
                WorkspaceState::from_persisted_v2(persisted),
                None,
                None,
                None,
                None,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V3 => {
            let persisted: PersistedWorkspaceStateV3 =
                decode_ron(&source).context("failed to decode v3 persisted workspace layout")?;
            let workspace_profile_id = persisted
                .workspace_profile_id
                .and_then(|raw| WorkspaceProfileId::try_from_raw(raw).ok());
            let layout_template = persisted.layout_template.clone();
            let layout_template_version = persisted.layout_template_version;
            let last_saved_at_unix_seconds = persisted.last_saved_at_unix_seconds;
            (
                WorkspaceState::from_persisted_v3(persisted),
                workspace_profile_id,
                layout_template,
                layout_template_version,
                last_saved_at_unix_seconds,
            )
        }
        PERSISTED_WORKSPACE_STATE_VERSION_V4 => {
            let persisted: PersistedWorkspaceStateV4 =
                decode_ron(&source).context("failed to decode v4 persisted workspace layout")?;
            let workspace_profile_id = persisted
                .workspace_profile_id
                .and_then(|raw| WorkspaceProfileId::try_from_raw(raw).ok());
            let layout_template = persisted.layout_template.clone();
            let layout_template_version = persisted.layout_template_version;
            let last_saved_at_unix_seconds = persisted.last_saved_at_unix_seconds;
            (
                WorkspaceState::from_persisted_v4(persisted),
                workspace_profile_id,
                layout_template,
                layout_template_version,
                last_saved_at_unix_seconds,
            )
        }
        version => (
            Err(editor_shell::WorkspaceStateError::PersistedVersionUnsupported(version)),
            None,
            None,
            None,
            None,
        ),
    };
    let workspace_state = workspace_state
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to validate persisted workspace layout")?;
    let workspace_state = compact_empty_tab_stack_areas(&workspace_state)
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to normalize persisted workspace layout")?;
    Ok(WorkspaceLayoutReadResult {
        workspace_state,
        workspace_profile_id,
        layout_template,
        layout_template_version,
        last_saved_at_unix_seconds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_shell::{LAYOUT_WORKSPACE_PROFILE_ID, WorkspaceIdentityAllocator};

    fn temp_workspace_layout_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        path.push(format!("runenwerk_workspace_layout_{nanos}.ron"));
        path
    }

    #[test]
    fn workspace_layout_roundtrip_preserves_workspace_graph() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);

        let path = temp_workspace_layout_path();
        write_workspace_layout(&path, &workspace)
            .expect("workspace layout should write successfully");
        let loaded = read_workspace_layout(&path).expect("workspace layout should decode");
        assert_eq!(workspace, loaded);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn profile_layout_path_is_not_derived_from_scene_path() {
        let path = default_workspace_layout_path_for_profile(LAYOUT_WORKSPACE_PROFILE_ID);

        assert_eq!(
            path,
            PathBuf::from("editor-scenes/workspaces/profile-1.workspace.ron")
        );
    }

    #[test]
    fn legacy_scene_layout_path_remains_available_for_load_migration() {
        let path =
            legacy_workspace_layout_path_for_scene(Path::new("editor-scenes/default.scene.ron"));

        assert_eq!(
            path,
            PathBuf::from("editor-scenes/default.scene.ron.workspace.ron")
        );
    }

    #[test]
    fn profile_workspace_layout_roundtrip_preserves_profile_metadata() {
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let workspace = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);
        let path = temp_workspace_layout_path();

        write_workspace_layout_for_profile(&path, &workspace, LAYOUT_WORKSPACE_PROFILE_ID)
            .expect("workspace layout should write with profile metadata");
        let loaded = read_workspace_layout_with_metadata(&path)
            .expect("workspace layout should decode with profile metadata");

        assert_eq!(loaded.workspace_state, workspace);
        assert_eq!(
            loaded.workspace_profile_id,
            Some(LAYOUT_WORKSPACE_PROFILE_ID)
        );
        assert_eq!(loaded.layout_template.as_deref(), Some("scene"));
        assert_eq!(loaded.layout_template_version, Some(1));
        assert!(loaded.last_saved_at_unix_seconds.is_some());

        let _ = std::fs::remove_file(path);
    }
}
