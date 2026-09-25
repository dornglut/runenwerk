use crate::app::App;

use super::{SceneCatalog, SceneRegistration};

pub trait AppSceneExt {
    fn add_scene<S>(&mut self, scene: S) -> &mut Self
    where
        S: Into<SceneRegistration>;

    fn add_scene_template(&mut self, template_path: impl Into<String>) -> &mut Self;

    fn registered_scene_count(&self) -> usize;
}

impl AppSceneExt for App {
    fn add_scene<S>(&mut self, scene: S) -> &mut Self
    where
        S: Into<SceneRegistration>,
    {
        if !self.allow_topology_mutation("add_scene", None) {
            return self;
        }

        let scene = scene.into();
        self.init_resource::<SceneCatalog>();
        if let Ok(catalog) = self.world_mut().resource_mut::<SceneCatalog>() {
            catalog.register(scene.id, scene.template_path);
        }
        self
    }

    fn add_scene_template(&mut self, template_path: impl Into<String>) -> &mut Self {
        if !self.allow_topology_mutation("add_scene_template", None) {
            return self;
        }

        let template_path = template_path.into();
        let mut id = SceneRegistration::derive_id_from_template_path(&template_path);
        self.init_resource::<SceneCatalog>();
        if let Ok(catalog) = self.world_mut().resource_mut::<SceneCatalog>() {
            if catalog.handle(&id).is_some() {
                let mut suffix = 2usize;
                let base = id.clone();
                while catalog.handle(&format!("{base}_{suffix}")).is_some() {
                    suffix = suffix.saturating_add(1);
                }
                id = format!("{base}_{suffix}");
            }
            catalog.register(id, template_path);
        }
        self
    }

    fn registered_scene_count(&self) -> usize {
        self.world()
            .resource::<SceneCatalog>()
            .map(|catalog| catalog.len())
            .unwrap_or(0)
    }
}
