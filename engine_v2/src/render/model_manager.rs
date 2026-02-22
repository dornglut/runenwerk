use super::WorldRenderModelProxy;
use glam::{Mat4, Vec3, Vec4};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

const MODELS_DIR: &str = "assets/models";
const EDITOR_CONFIG_PATH: &str = "assets/editor/config.ron";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct EditorConfig {
    blender_bin: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceKind {
    Glb,
    Gltf,
    Blend,
}

impl SourceKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Glb => "glb",
            Self::Gltf => "gltf",
            Self::Blend => "blend",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReloadStamp {
    source_modified: Option<SystemTime>,
    glb_modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct DiscoveredModel {
    id: String,
    source_kind: SourceKind,
    source_path: PathBuf,
    glb_path: PathBuf,
    source_modified: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct ModelAsset {
    source_kind: SourceKind,
    source_path: PathBuf,
    glb_path: PathBuf,
    stamp: ReloadStamp,
    revision: u64,
    proxies: Vec<WorldRenderModelProxy>,
    meshes: Vec<ModelMesh>,
    last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelStatus {
    pub id: String,
    pub path: String,
    pub source_kind: String,
    pub glb_path: String,
    pub revision: u64,
    pub proxy_count: usize,
    pub mesh_count: usize,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct ModelMeshVertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct ModelTextureData {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ModelMaterial {
    pub base_color_factor: [f32; 4],
    pub base_color_texture: Option<ModelTextureData>,
}

#[derive(Debug, Clone)]
pub struct ModelMesh {
    pub vertices: Vec<ModelMeshVertex>,
    pub indices: Vec<u32>,
    pub material: ModelMaterial,
}

#[derive(Debug)]
struct ImportedModelData {
    proxies: Vec<WorldRenderModelProxy>,
    meshes: Vec<ModelMesh>,
}

#[derive(Debug, Clone)]
pub struct ModelManager {
    models: BTreeMap<String, ModelAsset>,
    watch_enabled: bool,
    force_reload: bool,
}

impl Default for ModelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelManager {
    pub fn new() -> Self {
        let mut manager = Self {
            models: BTreeMap::new(),
            watch_enabled: true,
            force_reload: true,
        };
        let _ = manager.poll_updates();
        manager
    }

    pub fn watch_enabled(&self) -> bool {
        self.watch_enabled
    }

    pub fn set_watch_enabled(&mut self, enabled: bool) {
        self.watch_enabled = enabled;
    }

    pub fn request_reload(&mut self) {
        self.force_reload = true;
    }

    pub fn collect_sdf_proxies(&self) -> Vec<WorldRenderModelProxy> {
        let mut out = Vec::new();
        for asset in self.models.values() {
            out.extend(asset.proxies.iter().copied());
        }
        out
    }

    pub fn collect_meshes(&self) -> Vec<ModelMesh> {
        let mut out = Vec::new();
        for asset in self.models.values() {
            out.extend(asset.meshes.iter().cloned());
        }
        out
    }

    pub fn status_lines(&self) -> Vec<String> {
        let mut lines = vec![format!(
            "model_watch={} (assets/models/*.blend|*.glb|*.gltf)",
            if self.watch_enabled { "on" } else { "off" }
        )];
        if self.models.is_empty() {
            lines.push("models: none loaded".to_string());
            return lines;
        }
        for (id, asset) in &self.models {
            let textured_meshes = asset
                .meshes
                .iter()
                .filter(|m| m.material.base_color_texture.is_some())
                .count();
            if let Some(err) = &asset.last_error {
                lines.push(format!(
                    "model {} rev={} proxies={} meshes={} textured={} error={}",
                    id,
                    asset.revision,
                    asset.proxies.len(),
                    asset.meshes.len(),
                    textured_meshes,
                    err
                ));
            } else {
                lines.push(format!(
                    "model {} rev={} proxies={} meshes={} textured={} source={} path={} glb={}",
                    id,
                    asset.revision,
                    asset.proxies.len(),
                    asset.meshes.len(),
                    textured_meshes,
                    asset.source_kind.as_str(),
                    asset.source_path.display(),
                    asset.glb_path.display()
                ));
            }
        }
        lines
    }

    pub fn statuses(&self) -> Vec<ModelStatus> {
        self.models
            .iter()
            .map(|(id, asset)| ModelStatus {
                id: id.clone(),
                path: asset.source_path.to_string_lossy().to_string(),
                source_kind: asset.source_kind.as_str().to_string(),
                glb_path: asset.glb_path.to_string_lossy().to_string(),
                revision: asset.revision,
                proxy_count: asset.proxies.len(),
                mesh_count: asset.meshes.len(),
                last_error: asset.last_error.clone(),
            })
            .collect()
    }

    pub fn poll_updates(&mut self) -> Vec<String> {
        match self.poll_updates_impl() {
            Ok(messages) => messages,
            Err(err) => vec![format!("reload poll failed: {err}")],
        }
    }

    fn poll_updates_impl(&mut self) -> Result<Vec<String>, String> {
        if !self.watch_enabled && !self.force_reload {
            return Ok(Vec::new());
        }
        let mut messages = Vec::new();
        let force = self.force_reload;
        self.force_reload = false;

        let model_files = scan_model_files();
        if model_files.is_empty() && self.models.is_empty() {
            return Ok(Vec::new());
        }

        let discovered_ids: BTreeSet<String> = model_files.iter().map(|m| m.id.clone()).collect();

        let existing_ids: Vec<String> = self.models.keys().cloned().collect();
        for id in existing_ids {
            if !discovered_ids.contains(&id) {
                self.models.remove(&id);
                messages.push(format!("model {id} removed from watch list"));
            }
        }

        for model in model_files {
            let id = model.id.clone();
            let entry = self.models.entry(id.clone()).or_insert_with(|| ModelAsset {
                source_kind: model.source_kind,
                source_path: model.source_path.clone(),
                glb_path: model.glb_path.clone(),
                stamp: ReloadStamp {
                    source_modified: None,
                    glb_modified: None,
                },
                revision: 0,
                proxies: Vec::new(),
                meshes: Vec::new(),
                last_error: None,
            });
            entry.source_kind = model.source_kind;
            entry.source_path = model.source_path.clone();
            entry.glb_path = model.glb_path.clone();

            if model.source_kind == SourceKind::Blend {
                if let Some(msg) = ensure_blend_export(&model, force)? {
                    messages.push(msg);
                }
            }

            let stamp = ReloadStamp {
                source_modified: model.source_modified,
                glb_modified: fs::metadata(&model.glb_path)
                    .ok()
                    .and_then(|m| m.modified().ok()),
            };

            if !force && entry.stamp == stamp {
                continue;
            }

            match import_model_data(&model.glb_path) {
                Ok(imported) => {
                    entry.proxies = imported.proxies;
                    entry.meshes = imported.meshes;
                    entry.stamp = stamp;
                    entry.revision = entry.revision.saturating_add(1);
                    entry.last_error = None;
                    messages.push(format!(
                        "model {} reloaded rev={} proxies={} meshes={} ({})",
                        id,
                        entry.revision,
                        entry.proxies.len(),
                        entry.meshes.len(),
                        model.glb_path.display()
                    ));
                }
                Err(err) => {
                    entry.last_error = Some(err.clone());
                    messages.push(format!("model {} reload failed: {}", id, err));
                }
            }
        }

        Ok(messages)
    }
}

fn scan_model_files() -> Vec<DiscoveredModel> {
    let mut files = BTreeMap::<String, DiscoveredModel>::new();
    let root = Path::new(MODELS_DIR);
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
            continue;
        };
        let id = model_id_from_path(&path);
        let source_modified = fs::metadata(&path).ok().and_then(|m| m.modified().ok());
        if ext.eq_ignore_ascii_case("blend") {
            upsert_model_candidate(
                &mut files,
                DiscoveredModel {
                    id,
                    source_kind: SourceKind::Blend,
                    source_path: path.clone(),
                    glb_path: path.with_extension("glb"),
                    source_modified,
                },
            );
        } else if ext.eq_ignore_ascii_case("glb") {
            upsert_model_candidate(
                &mut files,
                DiscoveredModel {
                    id,
                    source_kind: SourceKind::Glb,
                    source_path: path.clone(),
                    glb_path: path.clone(),
                    source_modified,
                },
            );
        } else if ext.eq_ignore_ascii_case("gltf") {
            upsert_model_candidate(
                &mut files,
                DiscoveredModel {
                    id,
                    source_kind: SourceKind::Gltf,
                    source_path: path.clone(),
                    glb_path: path.clone(),
                    source_modified,
                },
            );
        }
    }
    files.into_values().collect()
}

fn upsert_model_candidate(
    files: &mut BTreeMap<String, DiscoveredModel>,
    candidate: DiscoveredModel,
) {
    let replace = match files.get(&candidate.id) {
        Some(existing) => source_priority(candidate.source_kind) > source_priority(existing.source_kind),
        None => true,
    };
    if replace {
        files.insert(candidate.id.clone(), candidate);
    }
}

fn source_priority(kind: SourceKind) -> u8 {
    match kind {
        SourceKind::Blend => 3,
        SourceKind::Glb => 2,
        SourceKind::Gltf => 1,
    }
}

fn model_id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(ToString::to_string)
        .unwrap_or_else(|| path.display().to_string())
}

fn import_model_data(path: &Path) -> Result<ImportedModelData, String> {
    let (document, buffers, images) =
        gltf::import(path).map_err(|e| format!("gltf import failed: {e}"))?;
    let mut proxies = Vec::new();
    let mut meshes = Vec::new();

    let default_scene = document.default_scene().or_else(|| document.scenes().next());
    let Some(scene) = default_scene else {
        return Err("model has no scene".to_string());
    };

    for node in scene.nodes() {
        collect_node_geometry(
            node,
            Mat4::IDENTITY,
            &buffers,
            &images,
            &mut proxies,
            &mut meshes,
        );
    }

    if meshes.is_empty() {
        return Err("no POSITION streams found in model primitives".to_string());
    }

    Ok(ImportedModelData { proxies, meshes })
}

fn collect_node_geometry(
    node: gltf::Node<'_>,
    parent_transform: Mat4,
    buffers: &[gltf::buffer::Data],
    images: &[gltf::image::Data],
    proxies: &mut Vec<WorldRenderModelProxy>,
    meshes: &mut Vec<ModelMesh>,
) {
    let local_transform = Mat4::from_cols_array_2d(&node.transform().matrix());
    let world_transform = parent_transform * local_transform;

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| b.0.as_slice()));
            let Some(positions) = reader.read_positions() else {
                continue;
            };
            let positions: Vec<[f32; 3]> = positions.collect();
            if positions.is_empty() {
                continue;
            }
            let tex_coord_set = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .map(|info| info.tex_coord() as u32)
                .unwrap_or(0);
            let tex_coords: Vec<[f32; 2]> = reader
                .read_tex_coords(tex_coord_set)
                .map(|coords| coords.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
            let indices: Vec<u32> = if let Some(read_indices) = reader.read_indices() {
                read_indices.into_u32().collect()
            } else {
                (0..positions.len() as u32).collect()
            };

            let pbr = primitive.material().pbr_metallic_roughness();
            let base = pbr.base_color_factor();
            let texture_index = pbr
                .base_color_texture()
                .map(|info| info.texture().source().index());
            let base_color_texture =
                texture_index.and_then(|idx| images.get(idx).and_then(convert_image_to_rgba8));
            let material = ModelMaterial {
                base_color_factor: [base[0], base[1], base[2], base[3]],
                base_color_texture,
            };

            let mut min = Vec3::splat(f32::INFINITY);
            let mut max = Vec3::splat(f32::NEG_INFINITY);
            let mut vertices = Vec::with_capacity(positions.len());
            for (idx, p) in positions.iter().enumerate() {
                let world_pos = world_transform * Vec4::new(p[0], p[1], p[2], 1.0);
                let wp = [world_pos.x, world_pos.y, world_pos.z];
                min = min.min(Vec3::new(wp[0], wp[1], wp[2]));
                max = max.max(Vec3::new(wp[0], wp[1], wp[2]));
                let uv = tex_coords.get(idx).copied().unwrap_or([0.0, 0.0]);
                vertices.push(ModelMeshVertex { position: wp, uv });
            }

            let center = (min + max) * 0.5;
            let extent = max - min;
            let radius = (extent.length() * 0.5).max(0.25);
            proxies.push(WorldRenderModelProxy {
                x: center.x,
                y: center.z,
                radius,
                color: [0.64, 0.56, 0.44, 0.95],
            });

            meshes.push(ModelMesh {
                vertices,
                indices,
                material,
            });
        }
    }

    for child in node.children() {
        collect_node_geometry(child, world_transform, buffers, images, proxies, meshes);
    }
}

fn convert_image_to_rgba8(image: &gltf::image::Data) -> Option<ModelTextureData> {
    use gltf::image::Format;
    match image.format {
        Format::R8G8B8A8 => Some(ModelTextureData {
            width: image.width,
            height: image.height,
            rgba8: image.pixels.clone(),
        }),
        Format::R8 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for v in &image.pixels {
                rgba.extend_from_slice(&[*v, *v, *v, 255]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        Format::R8G8 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for rg in image.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[rg[0], rg[0], rg[0], rg[1]]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for rgb in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        Format::R16 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for v in image.pixels.chunks_exact(2) {
                let g = v[1];
                rgba.extend_from_slice(&[g, g, g, 255]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        Format::R16G16 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for rg in image.pixels.chunks_exact(4) {
                let r = rg[1];
                let a = rg[3];
                rgba.extend_from_slice(&[r, r, r, a]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        Format::R16G16B16 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for rgb in image.pixels.chunks_exact(6) {
                rgba.extend_from_slice(&[rgb[1], rgb[3], rgb[5], 255]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        Format::R16G16B16A16 => {
            let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
            for rgba16 in image.pixels.chunks_exact(8) {
                rgba.extend_from_slice(&[rgba16[1], rgba16[3], rgba16[5], rgba16[7]]);
            }
            Some(ModelTextureData {
                width: image.width,
                height: image.height,
                rgba8: rgba,
            })
        }
        _ => None,
    }
}

fn ensure_blend_export(model: &DiscoveredModel, force: bool) -> Result<Option<String>, String> {
    let source_path = &model.source_path;
    let glb_path = &model.glb_path;

    let source_modified = model.source_modified;
    let target_modified = fs::metadata(glb_path).ok().and_then(|m| m.modified().ok());

    let needs_export = force
        || target_modified.is_none()
        || match (source_modified, target_modified) {
            (Some(source), Some(target)) => source > target,
            _ => false,
        };

    if !needs_export {
        return Ok(None);
    }

    if let Some(parent) = glb_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed creating model export directory {}: {e}",
                parent.display()
            )
        })?;
    }

    export_blend_to_glb(source_path, glb_path)?;
    Ok(Some(format!(
        "model {} converted to {}",
        source_path.display(),
        glb_path.display()
    )))
}

fn export_blend_to_glb(source_path: &Path, glb_path: &Path) -> Result<(), String> {
    let target = python_escaped_path(glb_path);
    let expr = format!(
        "import bpy; bpy.ops.export_scene.gltf(filepath='{}', export_format='GLB', export_apply=True)",
        target
    );

    let blender_bin = resolve_blender_binary();
    let output = Command::new(&blender_bin)
        .arg("-b")
        .arg(source_path)
        .arg("--python-expr")
        .arg(expr)
        .output()
        .map_err(|e| format_blender_exec_error(e, source_path, &blender_bin))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let reason = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("exit status {}", output.status)
    };

    Err(format!(
        "blender export failed for {} -> {}: {}",
        source_path.display(),
        glb_path.display(),
        reason
    ))
}

fn format_blender_exec_error(err: io::Error, source_path: &Path, blender_bin: &Path) -> String {
    if err.kind() == io::ErrorKind::NotFound {
        return format!(
            "blender CLI not found at {} while exporting {}. Install Blender or set BLENDER_BIN",
            blender_bin.display(),
            source_path.display(),
        );
    }
    format!(
        "failed to launch blender ({}) for {}: {err}",
        blender_bin.display(),
        source_path.display(),
    )
}

fn python_escaped_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
}

fn resolve_blender_binary() -> PathBuf {
    if let Some(bin) = env::var_os("BLENDER_BIN") {
        return PathBuf::from(bin);
    }
    if let Some(bin) = load_editor_config_blender_bin() {
        return bin;
    }
    let mac_default = Path::new("/Applications/Blender.app/Contents/MacOS/Blender");
    if mac_default.exists() {
        return mac_default.to_path_buf();
    }
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        let home_apps = home.join("Applications/Blender.app/Contents/MacOS/Blender");
        if home_apps.exists() {
            return home_apps;
        }
        let steam_default = home.join(
            "Library/Application Support/Steam/steamapps/common/Blender/Blender.app/Contents/MacOS/Blender",
        );
        if steam_default.exists() {
            return steam_default;
        }
    }
    PathBuf::from("blender")
}

fn load_editor_config_blender_bin() -> Option<PathBuf> {
    let path = Path::new(EDITOR_CONFIG_PATH);
    if !path.exists() {
        return None;
    }
    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(err) => {
            tracing::warn!(?err, path = EDITOR_CONFIG_PATH, "failed reading editor config");
            return None;
        }
    };
    let cfg = match ron::from_str::<EditorConfig>(&raw) {
        Ok(cfg) => cfg,
        Err(err) => {
            tracing::warn!(?err, path = EDITOR_CONFIG_PATH, "failed parsing editor config");
            return None;
        }
    };
    Some(cfg.blender_bin.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
}
