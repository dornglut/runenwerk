use std::path::Path;

use editor_core::{EditorMutationError, MigrationFailureClass, MigrationPathId};

use super::RunenwerkEditorApp;
use crate::editor_runtime::register_mvp_component_types;
use crate::persistence::{load_scene_file_into_runtime_classified, write_scene_file};

impl RunenwerkEditorApp {
    pub(crate) fn save_scene_persistence_to_path(
        &mut self,
        path: &Path,
    ) -> Result<(), EditorMutationError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| {
                EditorMutationError::runtime_rejected("failed to create editor scene folder")
            })?;
        }

        let persisted_scene = write_scene_file(path, self.runtime())
            .map_err(|_| EditorMutationError::runtime_rejected("failed to save editor scene"))?;
        self.establish_scene_persistence(path.to_path_buf(), persisted_scene);
        Ok(())
    }

    pub(crate) fn load_scene_persistence_from_path(
        &mut self,
        path: &Path,
    ) -> Result<Option<MigrationPathId>, MigrationFailureClass> {
        {
            let runtime = self.runtime_mut();
            runtime.prepare_for_scene_load();
            register_mvp_component_types(runtime);
        }

        let (migration, persisted_scene) =
            load_scene_file_into_runtime_classified(path, self.runtime_mut())?;
        self.establish_scene_persistence(path.to_path_buf(), persisted_scene);
        Ok(migration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_core::{ChangeOrigin, ComponentTypeId};
    use editor_scene::SceneCommandIntent;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::editor_features::{
        execute_intent_with_history, redo_last_scene_change, undo_last_scene_change,
    };
    use crate::editor_runtime::bootstrap_mvp_scene_if_empty;
    use crate::persistence::read_scene_file_v2;

    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

    #[derive(Debug, Clone, Default, runen_ecs::Component, runen_ecs::Reflect)]
    struct RuntimeOnlyComponent;

    fn temp_root(name: &str) -> std::path::PathBuf {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "runenwerk_scene_persistence_{name}_{}_{}",
            std::process::id(),
            id
        ))
    }

    fn saved_mvp_scene(name: &str) -> (RunenwerkEditorApp, std::path::PathBuf) {
        let mut app = RunenwerkEditorApp::new();
        register_mvp_component_types(app.runtime_mut());
        bootstrap_mvp_scene_if_empty(app.runtime_mut()).expect("MVP scene bootstrap should succeed");
        let root = temp_root(name);
        let path = root.join("scene.ron");
        app.save_scene_persistence_to_path(&path)
            .expect("scene save should establish a persistence baseline");
        (app, path)
    }

    #[test]
    fn fresh_unbound_scene_is_clean_without_claiming_a_persisted_baseline() {
        let app = RunenwerkEditorApp::new();

        assert!(!app
            .scene_persistence_is_dirty()
            .expect("fresh scene projection should normalize"));
        assert_eq!(app.scene_persistence().target(), None);
        assert_eq!(app.scene_persistence().persisted_scene(), None);
    }

    #[test]
    fn successful_save_tracks_exact_written_projection_and_undo_redo_content() {
        let (mut app, path) = saved_mvp_scene("undo_redo");
        let persisted = read_scene_file_v2(&path).expect("saved scene should be readable");

        assert_eq!(app.scene_persistence().target(), Some(path.as_path()));
        assert_eq!(app.scene_persistence().persisted_scene(), Some(&persisted));
        assert!(!app
            .scene_persistence_is_dirty()
            .expect("saved scene projection should normalize"));

        let entity = app
            .runtime()
            .document()
            .entity_ids()
            .next()
            .expect("MVP scene should contain an entity");
        let original_name = app
            .runtime()
            .document()
            .entity_snapshot(entity)
            .expect("entity snapshot should exist")
            .display_name;

        execute_intent_with_history(
            app.runtime_mut(),
            "Persistence No-op Rename",
            SceneCommandIntent::RenameEntity {
                entity,
                new_display_name: original_name,
            },
        )
        .expect("same-name rename should execute");
        assert!(!app
            .scene_persistence_is_dirty()
            .expect("no-op scene projection should normalize"));

        execute_intent_with_history(
            app.runtime_mut(),
            "Persistence Rename",
            SceneCommandIntent::RenameEntity {
                entity,
                new_display_name: "Persistence Changed".to_string(),
            },
        )
        .expect("scene edit should execute");
        assert!(app
            .scene_persistence_is_dirty()
            .expect("edited scene projection should normalize"));

        undo_last_scene_change(app.runtime_mut(), ChangeOrigin::EditorShell)
            .expect("scene undo should succeed")
            .expect("scene undo should consume the edit");
        assert!(!app
            .scene_persistence_is_dirty()
            .expect("undone scene projection should normalize"));

        redo_last_scene_change(app.runtime_mut(), ChangeOrigin::EditorShell)
            .expect("scene redo should succeed")
            .expect("scene redo should restore the edit");
        assert!(app
            .scene_persistence_is_dirty()
            .expect("redone scene projection should normalize"));

        let _ = std::fs::remove_dir_all(path.parent().expect("temp scene has parent"));
    }

    #[test]
    fn runtime_only_reflected_state_is_outside_scene_persistence_cleanliness() {
        let (mut app, path) = saved_mvp_scene("runtime_only");
        let entity = app
            .runtime()
            .document()
            .entity_ids()
            .next()
            .expect("MVP scene should contain an entity");
        let runtime_only_type = ComponentTypeId(99_901);
        app.runtime_mut()
            .register_component_type::<RuntimeOnlyComponent>(runtime_only_type);
        app.runtime_mut()
            .insert_component_for_editor_entity(entity, RuntimeOnlyComponent)
            .expect("runtime-only component insertion should succeed");

        assert!(!app
            .scene_persistence_is_dirty()
            .expect("persisted scene projection should ignore runtime-only component"));

        let _ = std::fs::remove_dir_all(path.parent().expect("temp scene has parent"));
    }

    #[test]
    fn failed_save_preserves_prior_baseline_and_dirty_knowledge() {
        let (mut app, path) = saved_mvp_scene("failed_save");
        let entity = app
            .runtime()
            .document()
            .entity_ids()
            .next()
            .expect("MVP scene should contain an entity");
        execute_intent_with_history(
            app.runtime_mut(),
            "Dirty Before Failed Save",
            SceneCommandIntent::RenameEntity {
                entity,
                new_display_name: "Dirty Before Failed Save".to_string(),
            },
        )
        .expect("scene edit should execute");
        assert!(app
            .scene_persistence_is_dirty()
            .expect("edited scene projection should normalize"));
        let baseline_before_failure = app.scene_persistence().clone();

        let blocker = temp_root("save_blocker");
        std::fs::write(&blocker, b"not a directory").expect("blocker file should be writable");
        let failing_path = blocker.join("scene.ron");
        assert!(app.save_scene_persistence_to_path(&failing_path).is_err());

        assert_eq!(app.scene_persistence(), &baseline_before_failure);
        assert!(app
            .scene_persistence_is_dirty()
            .expect("dirty scene projection should remain observable"));

        let _ = std::fs::remove_file(blocker);
        let _ = std::fs::remove_dir_all(path.parent().expect("temp scene has parent"));
    }

    #[test]
    fn successful_empty_load_establishes_the_exact_loaded_projection_as_clean() {
        let root = temp_root("empty_load");
        std::fs::create_dir_all(&root).expect("empty-load root should be creatable");
        let path = root.join("scene.ron");
        let empty_scene = editor_persistence::SceneFileV2::new(Vec::new());
        let source = editor_persistence::encode_ron_pretty(&empty_scene)
            .expect("empty scene should encode");
        std::fs::write(&path, source).expect("empty scene fixture should be writable");
        let mut app = RunenwerkEditorApp::new();

        app.load_scene_persistence_from_path(&path)
            .expect("empty scene load should succeed");

        assert_eq!(app.runtime().document().entity_ids().count(), 0);
        assert_eq!(app.scene_persistence().target(), Some(path.as_path()));
        assert_eq!(app.scene_persistence().persisted_scene(), Some(&empty_scene));
        assert!(!app
            .scene_persistence_is_dirty()
            .expect("loaded empty scene projection should normalize"));

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn successful_load_establishes_baseline_and_failed_load_preserves_it() {
        let (_source, path) = saved_mvp_scene("load_source");
        let mut app = RunenwerkEditorApp::new();

        app.load_scene_persistence_from_path(&path)
            .expect("scene load should succeed");
        assert_eq!(app.scene_persistence().target(), Some(path.as_path()));
        assert!(!app
            .scene_persistence_is_dirty()
            .expect("loaded scene projection should normalize"));
        let baseline_before_failure = app.scene_persistence().clone();

        let invalid_root = temp_root("invalid_load");
        std::fs::create_dir_all(&invalid_root).expect("invalid-load root should be creatable");
        let invalid_path = invalid_root.join("scene.ron");
        std::fs::write(&invalid_path, "not valid scene RON")
            .expect("invalid scene fixture should be writable");

        assert!(app.load_scene_persistence_from_path(&invalid_path).is_err());
        assert_eq!(app.scene_persistence(), &baseline_before_failure);
        assert!(app
            .scene_persistence_is_dirty()
            .expect("failed load should leave current state distinguishable from prior baseline"));

        let _ = std::fs::remove_dir_all(invalid_root);
        let _ = std::fs::remove_dir_all(path.parent().expect("temp scene has parent"));
    }
}
