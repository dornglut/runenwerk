use std::path::Path;

use anyhow::{Context, Result};
use editor_persistence::{ProjectFileV1, SceneFileV1, decode_ron, encode_ron_pretty};

use crate::editor_runtime::RunenwerkEditorRuntime;
use crate::persistence::{apply_scene_file_to_runtime, scene_file_from_runtime};

pub fn write_scene_file(path: &Path, runtime: &RunenwerkEditorRuntime) -> Result<()> {
    let scene_file = scene_file_from_runtime(runtime);
    let ron = encode_ron_pretty(&scene_file).context("failed to encode SceneFileV1 as RON")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write scene file: {}", path.display()))
}

pub fn read_scene_file(path: &Path) -> Result<SceneFileV1> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read scene file: {}", path.display()))?;
    decode_ron(&source).context("failed to decode SceneFileV1 from RON")
}

pub fn load_scene_file_into_runtime(
    path: &Path,
    runtime: &mut RunenwerkEditorRuntime,
) -> Result<()> {
    let scene = read_scene_file(path)?;
    apply_scene_file_to_runtime(runtime, &scene).map_err(anyhow::Error::msg)
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
