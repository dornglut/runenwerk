//! File: apps/runenwerk_editor/src/material_lab/default_material.rs
//! Purpose: App-owned generated default material bootstrap for renderer startup.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use asset::{
    ArtifactCacheKey, AssetKind, AssetSourceDescriptor, asset_artifact_id, asset_id,
    asset_source_id,
};
use engine::plugins::render::ShaderRegistryResource;
use engine::plugins::render::material_compiler::{
    MaterialPreviewFixture, MaterialShaderCompileRequest, compile_material_shader,
};
use material_graph::{MaterialNodeCatalog, MaterialOutputTarget, lower_material_graph};

use crate::editor_app::RunenwerkEditorApp;
use crate::material_lab::{
    EditorMaterialPreviewProduct, MaterialRendererParameterProfile,
    default_material_graph_document_for_source_with_target,
};
use crate::runtime::app::EDITOR_MATERIAL_PREVIEW_SHADER_ID;

const DEFAULT_MATERIAL_ASSET_ID: u64 = 9_021_000;
const DEFAULT_MATERIAL_SOURCE_ID: u64 = 9_021_001;
const DEFAULT_MATERIAL_ARTIFACT_ID: u64 = 9_021_002;
const DEFAULT_MATERIAL_PREVIEW_SHADER_ARTIFACT_ID: u64 = 9_021_003;
const DEFAULT_MATERIAL_SCENE_SHADER_ARTIFACT_ID: u64 = 9_021_004;

pub fn ensure_default_scene_material_preview(
    app: &mut RunenwerkEditorApp,
    shader_registry: &mut ShaderRegistryResource,
) -> Result<()> {
    let project_root = default_material_runtime_cache_root();
    ensure_default_scene_material_preview_at(app, shader_registry, project_root.as_path())
}

pub(crate) fn ensure_default_scene_material_preview_at(
    app: &mut RunenwerkEditorApp,
    shader_registry: &mut ShaderRegistryResource,
    project_root: &Path,
) -> Result<()> {
    if app.material_lab_runtime().active_preview().is_some() {
        return Ok(());
    }

    let asset_id = asset_id(DEFAULT_MATERIAL_ASSET_ID);
    let source = AssetSourceDescriptor::new(
        asset_source_id(DEFAULT_MATERIAL_SOURCE_ID),
        asset_id,
        AssetKind::MaterialGraph,
        ".runenwerk/generated/default-material/source.material.ron",
    );
    let document = default_material_graph_document_for_source_with_target(
        asset_id,
        &source,
        "Generated Default Material",
        MaterialOutputTarget::RenderMaterial,
    );
    let catalog = MaterialNodeCatalog::first_slice();
    let lowering = lower_material_graph(&document, &catalog);
    if lowering.report.has_blocking_issues() {
        bail!(
            "generated default material graph failed ratification: {:?}",
            lowering.report.issues()
        );
    }
    let product = lowering
        .product
        .ok_or_else(|| anyhow!("default material lowering produced no product"))?;
    let executable_ir = product
        .executable_ir
        .as_ref()
        .ok_or_else(|| anyhow!("default material product has no executable IR"))?;
    let compiled = compile_material_shader(MaterialShaderCompileRequest {
        ir: executable_ir,
        fixture: MaterialPreviewFixture::Sphere,
    })
    .map_err(|error| anyhow!("default material shader compilation failed: {error}"))?;

    let artifact_cache_key =
        ArtifactCacheKey::new(format!("default-material:{}", product.cache_key.as_str()));
    let preview_shader_cache_key = ArtifactCacheKey::new(format!(
        "default-material:preview-shader:{}",
        compiled.identity
    ));
    let scene_shader_cache_key = ArtifactCacheKey::new(format!(
        "default-material:scene-shader:{}",
        compiled.scene_identity
    ));
    let preview_shader_relative_path =
        default_material_generated_path("preview", &preview_shader_cache_key);
    let scene_shader_relative_path =
        default_material_generated_path("scene", &scene_shader_cache_key);
    write_generated_shader(
        project_root.join(&preview_shader_relative_path),
        &compiled.wgsl,
    )?;
    write_generated_shader(
        project_root.join(&scene_shader_relative_path),
        &compiled.scene_wgsl,
    )?;

    let preview_shader_path =
        canonical_shader_registry_path(project_root, preview_shader_relative_path.as_str());
    let scene_shader_path =
        canonical_shader_registry_path(project_root, scene_shader_relative_path.as_str());
    let preview = EditorMaterialPreviewProduct::new(
        asset_id,
        source.source_id,
        asset_artifact_id(DEFAULT_MATERIAL_ARTIFACT_ID),
        artifact_cache_key,
        product,
        MaterialRendererParameterProfile::RenderMaterial,
        asset_artifact_id(DEFAULT_MATERIAL_PREVIEW_SHADER_ARTIFACT_ID),
        preview_shader_cache_key,
        preview_shader_path.clone(),
        compiled.identity,
        asset_artifact_id(DEFAULT_MATERIAL_SCENE_SHADER_ARTIFACT_ID),
        scene_shader_cache_key,
        scene_shader_path.clone(),
        compiled.scene_identity,
        Vec::new(),
    );

    shader_registry.register_shader_with_id(EDITOR_MATERIAL_PREVIEW_SHADER_ID, preview_shader_path);
    shader_registry.register_shader_with_id(scene_shader_path.clone(), scene_shader_path);
    let _ = shader_registry.poll_updates();
    app.material_lab_runtime_mut().set_active_preview(preview);
    Ok(())
}

fn default_material_generated_path(prefix: &str, cache_key: &ArtifactCacheKey) -> String {
    let digest = blake3::hash(cache_key.as_str().as_bytes());
    format!(
        ".runenwerk/artifacts/generated/default-material/{prefix}/{}.wgsl",
        digest.to_hex()
    )
}

fn write_generated_shader(path: PathBuf, wgsl: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create generated shader directory {}", parent.display()))?;
    }
    std::fs::write(&path, wgsl)
        .with_context(|| format!("write generated shader {}", path.display()))?;
    Ok(())
}

fn canonical_shader_registry_path(project_root: &Path, relative_path: &str) -> String {
    project_root
        .join(relative_path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn default_material_runtime_cache_root() -> PathBuf {
    std::env::temp_dir()
        .join("runenwerk")
        .join("default-material-runtime-cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scene_material_bootstrap_generates_and_loads_shaders() {
        let root = unique_temp_dir("runenwerk-default-material-bootstrap");
        let mut app = RunenwerkEditorApp::new();
        let mut shader_registry = ShaderRegistryResource::new();

        ensure_default_scene_material_preview_at(&mut app, &mut shader_registry, root.as_path())
            .expect("default material bootstrap should succeed");

        let preview = app
            .material_lab_runtime()
            .active_preview()
            .expect("default material preview should be active");
        assert!(shader_registry.is_loaded(EDITOR_MATERIAL_PREVIEW_SHADER_ID));
        assert!(shader_registry.is_loaded(preview.scene_shader_path.as_str()));
        assert!(preview.shader_path.ends_with(".wgsl"));
        assert!(preview.scene_shader_path.ends_with(".wgsl"));
        let _ = std::fs::remove_dir_all(root);
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be valid")
            .as_nanos();
        root.push(format!("{label}-{nanos}"));
        std::fs::create_dir_all(&root).expect("temp dir should be created");
        root
    }
}
