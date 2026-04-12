use crate::plugins::render::inspect::{
    RenderCaptureIdentity, RenderCapturePointIdentity, RenderCapturedTexture,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedCaptureArtifact {
    pub frame_identity: RenderCaptureIdentity,
    pub image_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderArtifactExportResult {
    pub manifest_path: PathBuf,
    pub run_manifest_path: PathBuf,
    pub exported_images: Vec<PathBuf>,
    pub exported_capture_images: Vec<ExportedCaptureArtifact>,
}

#[derive(Debug, Serialize)]
struct CaptureManifest {
    frame_index: u64,
    captures: Vec<CaptureManifestEntry>,
}

#[derive(Debug, Serialize)]
struct CaptureManifestEntry {
    frame_index: u64,
    flow_id: String,
    pass_id: String,
    pass_label: String,
    stage: String,
    resource_id: String,
    texture_class: String,
    width: u32,
    height: u32,
    format: String,
    terminal_code: String,
    terminal_reason_code: Option<String>,
    terminal_reason_detail: Option<String>,
    file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct RunManifest {
    frame_manifests: Vec<String>,
}

pub fn export_captured_textures(
    output_root: &Path,
    frame_index: u64,
    captures: &[RenderCapturedTexture],
) -> Result<RenderArtifactExportResult> {
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create render artifact dir {:?}", output_root))?;

    let mut exported_images = Vec::<PathBuf>::new();
    let mut exported_capture_images = Vec::<ExportedCaptureArtifact>::new();
    let mut manifest_entries = Vec::<CaptureManifestEntry>::new();

    for capture in captures {
        let maybe_image_file = match capture.bytes_rgba8.as_ref() {
            Some(bytes) => {
                let file_name = deterministic_capture_filename(&capture.identity, "png");
                let image_path = output_root.join(file_name);
                let image =
                    image::RgbaImage::from_raw(capture.width, capture.height, bytes.clone())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "capture '{}' has invalid rgba byte length {} for {}x{}",
                                capture.identity.pass_id(),
                                bytes.len(),
                                capture.width,
                                capture.height
                            )
                        })?;
                image
                    .save(&image_path)
                    .with_context(|| format!("failed to write capture image {:?}", image_path))?;
                exported_images.push(image_path.clone());
                exported_capture_images.push(ExportedCaptureArtifact {
                    frame_identity: capture.identity.clone(),
                    image_path: image_path.clone(),
                });
                Some(image_path.to_string_lossy().to_string())
            }
            None => None,
        };

        manifest_entries.push(CaptureManifestEntry {
            frame_index: capture.identity.frame_index,
            flow_id: capture.identity.flow_id().to_string(),
            pass_id: capture.identity.pass_id().to_string(),
            pass_label: capture.identity.pass_label.clone(),
            stage: capture.identity.stage().as_str().to_string(),
            resource_id: capture.identity.resource_id().to_string(),
            texture_class: capture.identity.texture_class().as_str().to_string(),
            width: capture.width,
            height: capture.height,
            format: capture.format.clone(),
            terminal_code: capture.terminal.code.as_str().to_string(),
            terminal_reason_code: capture
                .terminal
                .reason
                .as_ref()
                .map(|value| value.code.clone()),
            terminal_reason_detail: capture
                .terminal
                .reason
                .as_ref()
                .map(|value| value.detail.clone()),
            file: maybe_image_file,
        });
    }

    let manifest = CaptureManifest {
        frame_index,
        captures: manifest_entries,
    };

    let manifest_path = output_root.join(format!("frame_{}__manifest.json", frame_index));
    let payload = serde_json::to_vec_pretty(&manifest)?;
    fs::write(&manifest_path, payload)
        .with_context(|| format!("failed to write capture manifest {:?}", manifest_path))?;

    let run_manifest_path = upsert_run_manifest(output_root, manifest_path.as_path())?;

    Ok(RenderArtifactExportResult {
        manifest_path,
        run_manifest_path,
        exported_images,
        exported_capture_images,
    })
}

fn upsert_run_manifest(output_root: &Path, frame_manifest: &Path) -> Result<PathBuf> {
    let run_manifest_path = output_root.join("run_manifest.json");
    let mut run_manifest = if run_manifest_path.exists() {
        let bytes = fs::read(&run_manifest_path)
            .with_context(|| format!("failed to read run manifest {:?}", run_manifest_path))?;
        serde_json::from_slice::<RunManifest>(&bytes)
            .with_context(|| format!("failed to parse run manifest {:?}", run_manifest_path))?
    } else {
        RunManifest::default()
    };

    let frame_manifest = frame_manifest.to_string_lossy().to_string();
    if !run_manifest
        .frame_manifests
        .iter()
        .any(|value| value == &frame_manifest)
    {
        run_manifest.frame_manifests.push(frame_manifest);
    }
    run_manifest.frame_manifests.sort();

    let payload = serde_json::to_vec_pretty(&run_manifest)?;
    fs::write(&run_manifest_path, payload)
        .with_context(|| format!("failed to write run manifest {:?}", run_manifest_path))?;
    Ok(run_manifest_path)
}

pub fn deterministic_capture_filename(identity: &RenderCaptureIdentity, ext: &str) -> String {
    format!(
        "frame_{}__flow_{}__pass_{}__stage_{}__resource_{}.{}",
        identity.frame_index,
        normalize_token(identity.flow_id()),
        normalize_token(identity.pass_id()),
        identity.stage().as_str(),
        normalize_token(identity.resource_id()),
        ext
    )
}

pub fn deterministic_capture_point_filename(
    identity: &RenderCapturePointIdentity,
    ext: &str,
) -> String {
    format!(
        "flow_{}__pass_{}__stage_{}__resource_{}.{}",
        normalize_token(identity.flow_id.as_str()),
        normalize_token(identity.pass_id.as_str()),
        identity.stage.as_str(),
        normalize_token(identity.resource_id.as_str()),
        ext
    )
}

fn normalize_token(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::render::inspect::{
        CaptureStage, CaptureTextureClass, RenderCapturePointIdentity, RenderCaptureTerminal,
    };

    #[test]
    fn deterministic_filename_uses_identity_tuple() {
        let identity = RenderCaptureIdentity {
            frame_index: 4,
            pass_label: "runenwerk.editor.viewport.sdf".to_string(),
            capture_point: RenderCapturePointIdentity {
                flow_id: "runenwerk.editor.main".to_string(),
                pass_id: "runenwerk.editor.viewport.sdf".to_string(),
                stage: CaptureStage::After,
                resource_id: "surface.color".to_string(),
                texture_class: CaptureTextureClass::ImportedTexture,
            },
        };

        let file = deterministic_capture_filename(&identity, "png");
        assert_eq!(
            file,
            "frame_4__flow_runenwerk_editor_main__pass_runenwerk_editor_viewport_sdf__stage_after__resource_surface_color.png"
        );
    }

    #[test]
    fn run_manifest_is_deterministic() {
        let temp = std::env::temp_dir().join(format!(
            "runenwerk_render_artifact_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp).expect("test output dir should be created");
        let capture = RenderCapturedTexture {
            identity: RenderCaptureIdentity {
                frame_index: 1,
                pass_label: "pass".to_string(),
                capture_point: RenderCapturePointIdentity {
                    flow_id: "flow".to_string(),
                    pass_id: "pass".to_string(),
                    stage: CaptureStage::After,
                    resource_id: "surface.color".to_string(),
                    texture_class: CaptureTextureClass::ImportedTexture,
                },
            },
            width: 1,
            height: 1,
            format: "Rgba8Unorm".to_string(),
            bytes_rgba8: Some(vec![255, 0, 0, 255]),
            terminal: RenderCaptureTerminal::completed(),
        };

        let first = export_captured_textures(&temp, 1, std::slice::from_ref(&capture))
            .expect("first export should succeed");
        let second =
            export_captured_textures(&temp, 1, &[capture]).expect("second export should succeed");

        assert_eq!(first.run_manifest_path, second.run_manifest_path);
        let _ = std::fs::remove_dir_all(temp);
    }
}
