use runen_ecs::Component;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneRegistration {
    pub id: String,
    pub template_path: String,
}

impl SceneRegistration {
    pub fn new(id: impl Into<String>, template_path: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            template_path: template_path.into(),
        }
    }

    pub fn from_template_path(template_path: impl Into<String>) -> Self {
        let template_path = template_path.into();
        let id = Self::derive_id_from_template_path(&template_path);
        Self { id, template_path }
    }

    pub fn derive_id_from_template_path(template_path: &str) -> String {
        let raw = Path::new(template_path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("scene");
        let mut out = String::new();
        let mut previous_was_sep = false;
        for ch in raw.chars() {
            if ch.is_ascii_alphanumeric() {
                out.push(ch.to_ascii_lowercase());
                previous_was_sep = false;
            } else if !previous_was_sep {
                out.push('_');
                previous_was_sep = true;
            }
        }
        let normalized = out.trim_matches('_');
        if normalized.is_empty() {
            "scene".to_string()
        } else {
            normalized.to_string()
        }
    }
}

impl From<String> for SceneRegistration {
    fn from(template_path: String) -> Self {
        Self::from_template_path(template_path)
    }
}

impl From<&str> for SceneRegistration {
    fn from(template_path: &str) -> Self {
        Self::from_template_path(template_path)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SceneHandle(u32);

impl SceneHandle {
    pub fn index(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredScene {
    pub handle: SceneHandle,
    pub id: String,
    pub template_path: String,
}

#[derive(Debug, Clone, Default, Component, runen_ecs::Resource)]
pub struct SceneCatalog {
    scenes: Vec<RegisteredScene>,
    by_id: HashMap<String, SceneHandle>,
}

impl SceneCatalog {
    pub fn from_registrations(registrations: &[SceneRegistration]) -> Self {
        let mut catalog = Self::default();
        for registration in registrations {
            catalog.register(registration.id.clone(), registration.template_path.clone());
        }
        catalog
    }

    pub fn register(
        &mut self,
        id: impl Into<String>,
        template_path: impl Into<String>,
    ) -> SceneHandle {
        let id = id.into();
        if let Some(existing) = self.by_id.get(&id).copied() {
            tracing::warn!(scene_id = %id, "duplicate scene registration id ignored");
            return existing;
        }

        let handle = SceneHandle(self.scenes.len() as u32);
        let scene = RegisteredScene {
            handle,
            id: id.clone(),
            template_path: template_path.into(),
        };
        self.by_id.insert(id, handle);
        self.scenes.push(scene);
        handle
    }

    pub fn len(&self) -> usize {
        self.scenes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scenes.is_empty()
    }

    pub fn handle(&self, id: &str) -> Option<SceneHandle> {
        self.by_id.get(id).copied()
    }

    pub fn get(&self, handle: SceneHandle) -> Option<&RegisteredScene> {
        self.scenes.get(handle.index() as usize)
    }

    pub fn iter(&self) -> impl Iterator<Item = &RegisteredScene> {
        self.scenes.iter()
    }
}
