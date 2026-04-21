use std::path::Path;

use anyhow::{Context, Result};
use editor_persistence::{
    ProjectFileV1, SceneFileV2, SceneLoadResult, SceneMigrationPath, decode_ron,
    decode_scene_file_with_migration, encode_ron_pretty, form_scene_for_runtime,
    normalize_scene_file,
};
use editor_shell::{PersistedWorkspaceStateV1, WorkspaceState};

use crate::editor_runtime::RunenwerkEditorRuntime;
use crate::persistence::{apply_formed_scene_to_runtime, scene_file_from_runtime};

pub fn write_scene_file(path: &Path, runtime: &RunenwerkEditorRuntime) -> Result<()> {
    let scene_file = scene_file_from_runtime(runtime);
    let ron = encode_ron_pretty(&scene_file).context("failed to encode SceneFileV2 as RON")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write scene file: {}", path.display()))
}

pub fn read_scene_file(path: &Path) -> Result<SceneLoadResult> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read scene file: {}", path.display()))?;
    decode_scene_file_with_migration(&source)
        .map_err(|error| anyhow::Error::msg(error.as_static_str()))
        .context("failed to decode scene file from RON")
}

pub fn read_scene_file_v2(path: &Path) -> Result<SceneFileV2> {
    Ok(read_scene_file(path)?.scene)
}

pub fn load_scene_file_into_runtime(
    path: &Path,
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<Option<editor_core::MigrationPathId>> {
    load_scene_file_into_runtime_classified(path, runtime)
        .map_err(|class| anyhow::Error::msg(classification_message(class)))
}

pub fn load_scene_file_into_runtime_classified(
    path: &Path,
    runtime: &mut RunenwerkEditorRuntime,
) -> std::result::Result<Option<editor_core::MigrationPathId>, editor_core::MigrationFailureClass> {
    let loaded =
        read_scene_file(path).map_err(|_| editor_core::MigrationFailureClass::DecodeFailure)?;
    let normalized = normalize_scene_file(loaded.scene)
        .map_err(|_| editor_core::MigrationFailureClass::NormalizationFailure)?;
    let formed = form_scene_for_runtime(normalized);
    let migration = scene_migration_path_id(loaded.migration);
    apply_formed_scene_to_runtime(runtime, &formed)
        .map_err(|_| editor_core::MigrationFailureClass::ApplyFailure)?;
    Ok(migration)
}

fn scene_migration_path_id(path: SceneMigrationPath) -> Option<editor_core::MigrationPathId> {
    match path {
        SceneMigrationPath::IdentityV2 => None,
        SceneMigrationPath::V1ToV2DefaultPrimitive => {
            Some(editor_core::SCENE_MIGRATION_V1_TO_V2_DEFAULT_PRIMITIVE)
        }
    }
}

fn classification_message(class: editor_core::MigrationFailureClass) -> &'static str {
    match class {
        editor_core::MigrationFailureClass::DecodeFailure => "failed to decode scene file",
        editor_core::MigrationFailureClass::NormalizationFailure => {
            "failed to normalize scene file"
        }
        editor_core::MigrationFailureClass::FormationFailure => "failed to form scene package",
        editor_core::MigrationFailureClass::ApplyFailure => "failed to apply scene package",
    }
}

pub fn write_project_file(path: &Path, project: &ProjectFileV1) -> Result<()> {
    let ron = encode_ron_pretty(project).context("failed to encode ProjectFileV1 as RON")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write project file: {}", path.display()))
}

pub fn read_project_file(path: &Path) -> Result<ProjectFileV1> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read project file: {}", path.display()))?;
    decode_ron(&source).context("failed to decode ProjectFileV1 from RON")
}

pub fn write_workspace_state_file(path: &Path, workspace_state: &WorkspaceState) -> Result<()> {
    let persisted = workspace_state.to_persisted_v1();
    let ron = encode_ron_pretty(&persisted).context("failed to encode workspace state as RON")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write workspace file: {}", path.display()))
}

pub fn read_workspace_state_file(path: &Path) -> Result<WorkspaceState> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read workspace file: {}", path.display()))?;
    let persisted: PersistedWorkspaceStateV1 =
        decode_ron(&source).context("failed to decode workspace RON")?;
    WorkspaceState::from_persisted_v1(persisted)
        .map_err(|error| anyhow::Error::msg(error.to_string()))
        .context("failed to restore workspace state from persisted payload")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_runtime::{RunenwerkEditorRuntime, register_mvp_component_types};
    use editor_shell::WorkspaceIdentityAllocator;

    fn temp_scene_path(name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        path.push(format!("runenwerk_{name}_{nanos}.scene.ron"));
        path
    }

    fn temp_workspace_path(name: &str) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        path.push(format!("runenwerk_{name}_{nanos}.workspace.ron"));
        path
    }

    #[test]
    fn load_scene_file_into_runtime_migrates_v1_payload() {
        let path = temp_scene_path("migrate_v1");
        let source = r#"
(
    version: 1,
    entities: [
        (
            id: 1,
            display_name: "Entity",
            parent: None,
            transform: (
                translation: (
                    0.0,
                    0.0,
                    0.0,
                ),
                rotation: (
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ),
                scale: (
                    1.0,
                    1.0,
                    1.0,
                ),
            ),
        ),
    ],
)
"#;
        std::fs::write(&path, source).expect("test scene file should be writable");

        let mut runtime = RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut runtime);

        let migration =
            load_scene_file_into_runtime(&path, &mut runtime).expect("scene load should succeed");
        assert_eq!(
            migration,
            Some(editor_core::SCENE_MIGRATION_V1_TO_V2_DEFAULT_PRIMITIVE)
        );
        assert_eq!(runtime.document().entity_ids().count(), 1);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn load_scene_file_into_runtime_rejects_invalid_normalized_structure() {
        let path = temp_scene_path("reject_duplicate");
        let source = r#"
(
    version: 2,
    entities: [
        (
            id: 1,
            display_name: "A",
            parent: None,
            transform: (
                translation: (
                    0.0,
                    0.0,
                    0.0,
                ),
                rotation: (
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ),
                scale: (
                    1.0,
                    1.0,
                    1.0,
                ),
            ),
            primitive: (
                kind: Box,
                box_half_extents: (
                    0.5,
                    0.5,
                    0.5,
                ),
                sphere_radius: 0.6,
                capsule_radius: 0.35,
                capsule_half_height: 0.75,
            ),
        ),
        (
            id: 1,
            display_name: "B",
            parent: None,
            transform: (
                translation: (
                    0.0,
                    0.0,
                    0.0,
                ),
                rotation: (
                    0.0,
                    0.0,
                    0.0,
                    1.0,
                ),
                scale: (
                    1.0,
                    1.0,
                    1.0,
                ),
            ),
            primitive: (
                kind: Box,
                box_half_extents: (
                    0.5,
                    0.5,
                    0.5,
                ),
                sphere_radius: 0.6,
                capsule_radius: 0.35,
                capsule_half_height: 0.75,
            ),
        ),
    ],
)
"#;
        std::fs::write(&path, source).expect("test scene file should be writable");

        let mut runtime = RunenwerkEditorRuntime::new();
        register_mvp_component_types(&mut runtime);

        let error = load_scene_file_into_runtime(&path, &mut runtime)
            .expect_err("invalid scene structure should fail normalization");
        assert!(
            error.to_string().contains("failed to normalize scene file"),
            "error should surface normalization failure context"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn workspace_state_file_roundtrip_preserves_structural_identity() {
        let path = temp_workspace_path("workspace_roundtrip");
        let mut allocator = WorkspaceIdentityAllocator::new();
        let workspace_id = allocator.allocate_workspace_id();
        let state = WorkspaceState::bootstrap_current_layout(workspace_id, &mut allocator);

        write_workspace_state_file(&path, &state).expect("workspace write should succeed");
        let restored = read_workspace_state_file(&path).expect("workspace read should succeed");

        assert_eq!(restored, state);

        let _ = std::fs::remove_file(path);
    }
}
