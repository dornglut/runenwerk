use super::{SceneLayerDescriptor, SceneManifestDescriptor, normalize_scene_label};
use asset::{AssetCatalog, AssetKind};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const SCENE_MANIFEST_DIR: &str = "assets/scenes";
pub const GAME_SCENE_MANIFEST_DIR: &str = "game/assets/scenes";

pub fn load_scene_manifest_descriptors() -> Vec<SceneManifestDescriptor> {
    load_scene_manifest_descriptors_with_asset_catalog(None)
}

pub fn load_scene_manifest_descriptors_with_asset_catalog(
    asset_catalog: Option<&AssetCatalog>,
) -> Vec<SceneManifestDescriptor> {
    let mut by_id = BTreeMap::<String, SceneManifestDescriptor>::new();
    for descriptor in default_scene_manifest_descriptors() {
        by_id.insert(descriptor.id.clone(), descriptor);
    }

    let mut discovered_files = Vec::new();
    discovered_files.extend(discover_manifest_files(Path::new(SCENE_MANIFEST_DIR)));
    discovered_files.extend(discover_manifest_files(Path::new(GAME_SCENE_MANIFEST_DIR)));
    discovered_files.sort();
    discovered_files.dedup();

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

    if let Some(asset_catalog) = asset_catalog {
        for descriptor in scene_manifest_descriptors_from_asset_catalog(asset_catalog) {
            by_id.insert(descriptor.id.clone(), descriptor);
        }
    }

    by_id.into_values().collect()
}

pub fn scene_manifest_descriptors_from_asset_catalog(
    asset_catalog: &AssetCatalog,
) -> Vec<SceneManifestDescriptor> {
    asset_catalog
        .assets()
        .filter(|record| record.kind == AssetKind::Scene)
        .filter_map(|record| {
            let id = normalize_scene_label(&record.stable_name);
            if id.is_empty() {
                return None;
            }
            Some(SceneManifestDescriptor {
                id,
                layer: Some(SceneLayerDescriptor::World),
                ui_template: None,
                render_graph_append_passes: Vec::new(),
            })
        })
        .collect()
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
