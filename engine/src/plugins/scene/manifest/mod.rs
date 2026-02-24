use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const SCENE_MANIFEST_DIR: &str = "assets/scenes";
pub const GAME_SCENE_MANIFEST_DIR: &str = "game/assets/scenes";

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SceneLayerDescriptor {
    World,
    OverlayUi,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FramePassKindDescriptor {
    Compute,
    Render,
}

impl Default for FramePassKindDescriptor {
    fn default() -> Self {
        Self::Render
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FrameResourceDescriptor {
    SurfaceColor,
    WorldColor,
    WorldParams,
    WorldAgents,
    MeshData,
    UiDrawList,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct FramePassDescriptor {
    pub name: String,
    pub kind: FramePassKindDescriptor,
    pub pipeline: Option<String>,
    pub reads: Vec<FrameResourceDescriptor>,
    pub writes: Vec<FrameResourceDescriptor>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SceneManifestDescriptor {
    pub id: String,
    pub layer: Option<SceneLayerDescriptor>,
    pub ui_template: Option<String>,
    pub render_graph_append_passes: Vec<FramePassDescriptor>,
}

pub fn normalize_scene_label(label: &str) -> String {
    label.trim().to_ascii_lowercase()
}

pub fn load_scene_manifest_descriptors() -> Vec<SceneManifestDescriptor> {
    let mut by_id = BTreeMap::<String, SceneManifestDescriptor>::new();
    for descriptor in default_scene_manifest_descriptors() {
        by_id.insert(descriptor.id.clone(), descriptor);
    }

    let mut discovered_files = Vec::new();
    discovered_files.extend(discover_manifest_files(Path::new(SCENE_MANIFEST_DIR)));
    discovered_files.extend(discover_manifest_files(Path::new(GAME_SCENE_MANIFEST_DIR)));
    discovered_files.sort();
    discovered_files.dedup();
    if discovered_files.is_empty() {
        return by_id.into_values().collect();
    }

    for file in discovered_files {
        let raw = match fs::read_to_string(&file) {
            Ok(raw) => raw,
            Err(err) => {
                tracing::warn!(?err, path = %file.display(), "failed reading scene manifest");
                continue;
            }
        };
        let mut descriptor = match ron::from_str::<SceneManifestDescriptor>(&raw) {
            Ok(descriptor) => descriptor,
            Err(err) => {
                tracing::warn!(?err, path = %file.display(), "failed parsing scene manifest");
                continue;
            }
        };
        descriptor.id = normalize_scene_label(&descriptor.id);
        if descriptor.id.is_empty() {
            tracing::warn!(path = %file.display(), "scene manifest has empty id; skipping");
            continue;
        }
        if let Some(path) = descriptor.ui_template.as_mut() {
            *path = path.trim().to_string();
        }
        by_id.insert(descriptor.id.clone(), descriptor);
    }

    by_id.into_values().collect()
}

fn discover_manifest_files(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut files: Vec<_> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ron"))
        })
        .collect();
    files.sort();
    files
}

fn default_scene_manifest_descriptors() -> Vec<SceneManifestDescriptor> {
    vec![
        SceneManifestDescriptor {
            id: "gameplay_stub".to_string(),
            layer: Some(SceneLayerDescriptor::World),
            ui_template: None,
            render_graph_append_passes: Vec::new(),
        },
        SceneManifestDescriptor {
            id: "hub_stub".to_string(),
            layer: Some(SceneLayerDescriptor::World),
            ui_template: None,
            render_graph_append_passes: Vec::new(),
        },
        SceneManifestDescriptor {
            id: "console_ui".to_string(),
            layer: Some(SceneLayerDescriptor::OverlayUi),
            ui_template: Some("game/assets/ui/console.ron".to_string()),
            render_graph_append_passes: Vec::new(),
        },
        SceneManifestDescriptor {
            id: "hud_ui".to_string(),
            layer: Some(SceneLayerDescriptor::OverlayUi),
            ui_template: Some("game/assets/ui/hud.ron".to_string()),
            render_graph_append_passes: Vec::new(),
        },
        SceneManifestDescriptor {
            id: "inventory_ui".to_string(),
            layer: Some(SceneLayerDescriptor::OverlayUi),
            ui_template: Some("game/assets/ui/inventory.ron".to_string()),
            render_graph_append_passes: Vec::new(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        FramePassKindDescriptor, SceneLayerDescriptor, SceneManifestDescriptor,
        normalize_scene_label,
    };

    #[test]
    fn scene_manifest_descriptor_parses() {
        let raw = r#"
(
  id: "hud_ui",
  layer: Some(overlay_ui),
  ui_template: Some("assets/ui/hud.ron"),
  render_graph_append_passes: [
    (
      name: "debug_pass",
      kind: render,
      reads: [ui_draw_list],
      writes: [surface_color],
      depends_on: ["ui_composite"],
    ),
  ],
)
"#;
        let manifest: SceneManifestDescriptor =
            ron::from_str(raw).expect("scene manifest should parse");
        assert_eq!(manifest.id, "hud_ui");
        assert_eq!(manifest.layer, Some(SceneLayerDescriptor::OverlayUi));
        assert_eq!(manifest.render_graph_append_passes.len(), 1);
        assert_eq!(manifest.render_graph_append_passes[0].name, "debug_pass");
        assert_eq!(
            manifest.render_graph_append_passes[0].kind,
            FramePassKindDescriptor::Render
        );
    }

    #[test]
    fn normalize_scene_label_trims_and_lowercases() {
        assert_eq!(normalize_scene_label("  HUD_UI "), "hud_ui");
    }
}
