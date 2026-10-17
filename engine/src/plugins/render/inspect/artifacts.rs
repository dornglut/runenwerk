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
    surface_id: u64,
    captures: Vec<CaptureManifestEntry>,
}

#[derive(Debug, Serialize)]
struct CaptureManifestEntry {
    frame_index: u64,
    surface_id: u64,
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
    surface_id: u64,
    captures: &[RenderCapturedTexture],
) -> Result<RenderArtifactExportResult> {
    // Fail before writing an image or manifest: this product is one exact source.
    if let Some(foreign) = captures.iter().find(|capture| {
        capture.identity.frame_index != frame_index || capture.identity.surface_id() != surface_id
    }) {
        anyhow::bail!(
            "capture source mismatch: export frame {frame_index} surface {surface_id}, capture frame {} surface {}",
            foreign.identity.frame_index,
            foreign.identity.surface_id()
        );
    }
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
                let expected_len = usize::try_from(capture.width)
                    .ok()
                    .and_then(|width| {
                        usize::try_from(capture.height)
                            .ok()
                            .and_then(|height| width.checked_mul(height))
                    })
                    .and_then(|pixels| pixels.checked_mul(4));
                if expected_len != Some(bytes.len()) {
                    anyhow::bail!(
                        "capture '{}' has invalid rgba byte length {} for {}x{}",
                        capture.identity.pass_id(),
                        bytes.len(),
                        capture.width,
                        capture.height
                    );
                }
                image::save_buffer(
                    &image_path,
                    bytes,
                    capture.width,
                    capture.height,
                    image::ColorType::Rgba8,
                )
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
            surface_id: capture.identity.surface_id(),
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
        surface_id,
        captures: manifest_entries,
    };

    let manifest_path = output_root.join(format!(
        "frame_{frame_index}__surface_{surface_id}__manifest.json"
    ));
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
        "frame_{}__surface_{}__flow_{}__pass_{}__stage_{}__resource_{}.{}",
        identity.frame_index,
        identity.surface_id(),
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
            surface_id: 1,
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
            "frame_4__surface_1__flow_runenwerk_editor_main__pass_runenwerk_editor_viewport_sdf__stage_after__resource_surface_color.png"
        );
        let second_surface = RenderCaptureIdentity {
            surface_id: 2,
            ..identity.clone()
        };
        assert_ne!(
            deterministic_capture_filename(&identity, "png"),
            deterministic_capture_filename(&second_surface, "png"),
            "two windows with one frame ordinal and capture point must not overwrite each other"
        );
    }

    #[test]
    fn same_ordinal_surface_exports_are_distinct_and_reject_foreign_capture() {
        let root = std::env::temp_dir().join(format!(
            "runenwerk_surface_manifest_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos()
        ));
        let identity = RenderCaptureIdentity {
            frame_index: 42,
            surface_id: 1,
            pass_label: "pass".into(),
            capture_point: RenderCapturePointIdentity {
                flow_id: "flow".into(),
                pass_id: "pass".into(),
                stage: CaptureStage::After,
                resource_id: "surface.color".into(),
                texture_class: CaptureTextureClass::ColorTarget,
            },
        };
        let first_capture = RenderCapturedTexture {
            identity: identity.clone(),
            width: 1,
            height: 1,
            format: "Rgba8Unorm".into(),
            bytes_rgba8: Some(vec![255, 0, 0, 255]),
            terminal: RenderCaptureTerminal::completed(),
        };
        let mut second_capture = first_capture.clone();
        second_capture.identity.surface_id = 2;
        second_capture.bytes_rgba8 = Some(vec![0, 255, 0, 255]);

        let mismatched =
            export_captured_textures(&root, 42, 1, std::slice::from_ref(&second_capture));
        assert!(
            mismatched.is_err(),
            "an unrelated surface cannot enter this manifest"
        );
        assert!(
            !root.exists(),
            "source mismatch must fail before any export side effects"
        );
        let mut other_frame = second_capture.clone();
        other_frame.identity.frame_index = 41;
        assert!(
            export_captured_textures(&root, 42, 2, &[other_frame]).is_err(),
            "an unrelated frame cannot enter this manifest"
        );
        assert!(!root.exists());

        let first = export_captured_textures(&root, 42, 1, &[first_capture])
            .expect("primary surface export");
        let second = export_captured_textures(&root, 42, 2, &[second_capture])
            .expect("secondary surface export");
        assert_ne!(first.manifest_path, second.manifest_path);
        assert!(
            first
                .manifest_path
                .ends_with("frame_42__surface_1__manifest.json")
        );
        assert!(
            second
                .manifest_path
                .ends_with("frame_42__surface_2__manifest.json")
        );
        assert!(first.manifest_path.exists() && second.manifest_path.exists());
        let primary_manifest = std::fs::read(&first.manifest_path).expect("primary manifest");
        let secondary_manifest = std::fs::read(&second.manifest_path).expect("secondary manifest");
        let first_json: serde_json::Value =
            serde_json::from_slice(&primary_manifest).expect("valid primary manifest");
        let second_json: serde_json::Value =
            serde_json::from_slice(&secondary_manifest).expect("valid secondary manifest");
        assert_eq!(first_json["surface_id"], 1);
        assert_eq!(second_json["surface_id"], 2);
        assert_eq!(first_json["captures"][0]["surface_id"], 1);
        assert_eq!(second_json["captures"][0]["surface_id"], 2);
        let run_manifest = std::fs::read(&second.run_manifest_path).expect("run manifest");
        let run: serde_json::Value =
            serde_json::from_slice(&run_manifest).expect("valid run manifest");
        assert_eq!(run["frame_manifests"].as_array().unwrap().len(), 2);
        assert_ne!(first.exported_images, second.exported_images);
        let _ = std::fs::remove_dir_all(root);
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
                surface_id: 1,
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

        let first = export_captured_textures(&temp, 1, 1, std::slice::from_ref(&capture))
            .expect("first export should succeed");
        let second = export_captured_textures(&temp, 1, 1, &[capture])
            .expect("second export should succeed");

        assert_eq!(first.run_manifest_path, second.run_manifest_path);
        let _ = std::fs::remove_dir_all(temp);
    }
}
