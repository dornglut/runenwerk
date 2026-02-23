use super::{SceneId, SceneLayer};
use crate::scene_manifest::{
    FramePassDescriptor as SceneFramePassDescriptor, SceneLayerDescriptor,
    load_scene_manifest_descriptors, normalize_scene_label,
};

#[derive(Debug, Clone)]
pub struct SceneDescriptor {
    pub id: SceneId,
    pub ui_template: Option<String>,
    pub render_graph_append_passes: Vec<SceneFramePassDescriptor>,
}

#[derive(Debug, Clone)]
pub struct SceneRegistry {
    descriptors: Vec<SceneDescriptor>,
}

impl SceneRegistry {
    pub fn load() -> Self {
        let manifests = load_scene_manifest_descriptors();
        let mut descriptors = vec![
            default_descriptor(SceneId::GameplayStub),
            default_descriptor(SceneId::HubStub),
            default_descriptor(SceneId::ConsoleUi),
            default_descriptor(SceneId::HudUi),
        ];

        for manifest in manifests {
            let id = normalize_scene_label(&manifest.id);
            let Some(scene) = SceneId::from_label(&id) else {
                tracing::warn!(scene_id = id, "scene manifest id is unknown; skipping");
                continue;
            };
            if let Some(layer) = manifest.layer
                && layer != scene_layer_descriptor(scene.layer())
            {
                tracing::warn!(
                    scene = scene.label(),
                    ?layer,
                    expected = ?scene_layer_descriptor(scene.layer()),
                    "scene manifest layer mismatches scene id layer"
                );
            }

            let descriptor = descriptor_mut(&mut descriptors, scene);
            if let Some(ui_template) = manifest.ui_template
                && !ui_template.is_empty()
            {
                descriptor.ui_template = Some(ui_template);
            }
            descriptor.render_graph_append_passes = manifest.render_graph_append_passes;
        }

        Self { descriptors }
    }

    pub fn descriptor(&self, scene: SceneId) -> &SceneDescriptor {
        descriptor_ref(&self.descriptors, scene)
    }

    pub fn ui_template_path(&self, scene: SceneId) -> Option<&str> {
        self.descriptor(scene).ui_template.as_deref()
    }

    pub fn render_graph_contributions(
        &self,
        world_scene: SceneId,
        overlay_scene: SceneId,
    ) -> Vec<SceneFramePassDescriptor> {
        let mut out = Vec::new();
        out.extend(
            self.descriptor(world_scene)
                .render_graph_append_passes
                .iter()
                .cloned(),
        );
        out.extend(
            self.descriptor(overlay_scene)
                .render_graph_append_passes
                .iter()
                .cloned(),
        );
        out
    }
}

fn default_descriptor(id: SceneId) -> SceneDescriptor {
    SceneDescriptor {
        id,
        ui_template: match id {
            SceneId::ConsoleUi => Some("assets/ui/console.ron".to_string()),
            SceneId::HudUi => Some("assets/ui/hud.ron".to_string()),
            SceneId::GameplayStub | SceneId::HubStub => None,
        },
        render_graph_append_passes: Vec::new(),
    }
}

fn scene_layer_descriptor(layer: SceneLayer) -> SceneLayerDescriptor {
    match layer {
        SceneLayer::World => SceneLayerDescriptor::World,
        SceneLayer::OverlayUi => SceneLayerDescriptor::OverlayUi,
    }
}

fn descriptor_ref(descriptors: &[SceneDescriptor], scene: SceneId) -> &SceneDescriptor {
    descriptors
        .iter()
        .find(|descriptor| descriptor.id == scene)
        .expect("scene registry should always contain descriptor for all SceneId values")
}

fn descriptor_mut(descriptors: &mut [SceneDescriptor], scene: SceneId) -> &mut SceneDescriptor {
    descriptors
        .iter_mut()
        .find(|descriptor| descriptor.id == scene)
        .expect("scene registry should always contain descriptor for all SceneId values")
}

#[cfg(test)]
mod tests {
    use super::SceneRegistry;
    use crate::runtime::SceneId;

    #[test]
    fn scene_registry_contains_all_core_scenes() {
        let registry = SceneRegistry::load();
        assert_eq!(
            registry.descriptor(SceneId::GameplayStub).id,
            SceneId::GameplayStub
        );
        assert_eq!(registry.descriptor(SceneId::HubStub).id, SceneId::HubStub);
        assert_eq!(
            registry.descriptor(SceneId::ConsoleUi).id,
            SceneId::ConsoleUi
        );
        assert_eq!(registry.descriptor(SceneId::HudUi).id, SceneId::HudUi);
    }
}
