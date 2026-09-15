use std::path::{Path, PathBuf};

use editor_persistence::SceneFileV2;

#[derive(Debug, Clone, PartialEq)]
enum ScenePersistenceAnchor {
    Unbound { origin_scene: SceneFileV2 },
    Persisted { target: PathBuf, scene: SceneFileV2 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenePersistenceContext {
    anchor: ScenePersistenceAnchor,
}

impl Default for ScenePersistenceContext {
    fn default() -> Self {
        Self::new_unbound(SceneFileV2::new(Vec::new()))
    }
}

impl ScenePersistenceContext {
    pub(crate) fn new_unbound(origin_scene: SceneFileV2) -> Self {
        Self {
            anchor: ScenePersistenceAnchor::Unbound { origin_scene },
        }
    }

    pub fn target(&self) -> Option<&Path> {
        match &self.anchor {
            ScenePersistenceAnchor::Unbound { .. } => None,
            ScenePersistenceAnchor::Persisted { target, .. } => Some(target.as_path()),
        }
    }

    pub fn persisted_scene(&self) -> Option<&SceneFileV2> {
        match &self.anchor {
            ScenePersistenceAnchor::Unbound { .. } => None,
            ScenePersistenceAnchor::Persisted { scene, .. } => Some(scene),
        }
    }

    pub fn is_dirty(&self, current_scene: &SceneFileV2) -> bool {
        self.comparison_scene() != current_scene
    }

    pub(crate) fn establish(
        &mut self,
        target: impl Into<PathBuf>,
        persisted_scene: SceneFileV2,
    ) {
        self.anchor = ScenePersistenceAnchor::Persisted {
            target: target.into(),
            scene: persisted_scene,
        };
    }

    fn comparison_scene(&self) -> &SceneFileV2 {
        match &self.anchor {
            ScenePersistenceAnchor::Unbound { origin_scene } => origin_scene,
            ScenePersistenceAnchor::Persisted { scene, .. } => scene,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbound_context_has_no_persisted_baseline_and_compares_to_its_origin() {
        let origin = SceneFileV2::new(Vec::new());
        let context = ScenePersistenceContext::new_unbound(origin.clone());

        assert_eq!(context.target(), None);
        assert_eq!(context.persisted_scene(), None);
        assert!(!context.is_dirty(&origin));
    }

    #[test]
    fn establish_replaces_unbound_origin_with_persisted_target_and_projection() {
        let mut context = ScenePersistenceContext::default();
        let persisted = SceneFileV2::new(Vec::new());

        context.establish("scene.ron", persisted.clone());

        assert_eq!(context.target(), Some(Path::new("scene.ron")));
        assert_eq!(context.persisted_scene(), Some(&persisted));
        assert!(!context.is_dirty(&persisted));
    }
}
