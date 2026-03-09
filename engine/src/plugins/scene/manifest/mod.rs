mod catalog;

use serde::Deserialize;

pub use catalog::load_scene_manifest_descriptors;

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
