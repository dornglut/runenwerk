use std::path::Path;

use anyhow::{Context, Result};

use crate::asset_pipeline::EditorAssetProjectSession;
use crate::editor_app::RunenwerkEditorApp;
use crate::persistence::read_project_file;

impl RunenwerkEditorApp {
    pub fn load_project_file(&mut self, path: &Path) -> Result<()> {
        let project = read_project_file(path)?;
        let project_root = path.parent().unwrap_or_else(|| Path::new("."));
        let session = match EditorAssetProjectSession::from_project_file(project_root, &project) {
            Ok(session) => session,
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    self.asset_catalog_runtime_mut()
                        .record_diagnostic(diagnostic);
                }
                anyhow::bail!(
                    "failed to form asset project session for project file: {}",
                    path.display()
                );
            }
        };
        self.set_asset_project_session(session)?;
        self.load_asset_project_catalog().with_context(|| {
            format!(
                "failed to load asset catalog for project: {}",
                path.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_persistence::{ProjectFileV3, encode_ron_pretty};

    #[test]
    fn load_project_file_sets_asset_project_session_and_reports_missing_catalog() {
        let root = unique_temp_dir("project_load_session");
        let project_path = root.join("project.rune.ron");
        let project = ProjectFileV3::new("project.test", "Test");
        let ron = encode_ron_pretty(&project).expect("project should encode");
        std::fs::write(&project_path, ron).expect("project file should be writable");
        let mut app = RunenwerkEditorApp::new();

        app.load_project_file(&project_path)
            .expect("missing catalog should be a controlled diagnostic");

        assert!(app.asset_project_session().is_some());
        assert!(
            app.asset_catalog_runtime()
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.message.contains("asset catalog is missing"))
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn project_catalog_fallback_cannot_follow_a_material_source_into_another_project() {
        let first_root = unique_temp_dir("material_catalog_owner_project_a");
        let second_root = unique_temp_dir("material_catalog_owner_project_b");
        let project = ProjectFileV3::new("project.material", "Material");
        let project_ron = encode_ron_pretty(&project).expect("project file should encode");
        let first_project = first_root.join("project.rune.ron");
        let second_project = second_root.join("project.rune.ron");
        std::fs::write(&first_project, &project_ron).expect("A project file");
        std::fs::write(&second_project, &project_ron).expect("B project file");

        let asset_id = asset::asset_id(1541);
        let source_id = asset::asset_source_id(1542);
        let relative_path = "assets/materials/source.material.ron";
        let source = asset::AssetSourceDescriptor::new(
            source_id,
            asset_id,
            asset::AssetKind::MaterialGraph,
            relative_path,
        );
        let document = crate::material_lab::default_material_graph_document_for_source(
            asset_id,
            &source,
            "Identical source",
        );
        for root in [&first_root, &second_root] {
            crate::material_lab::write_material_graph_document(
                &root.join(relative_path),
                &document,
            )
            .expect("identically addressed Material file should persist");
        }
        let original_a = std::fs::read(first_root.join(relative_path)).expect("A bytes");
        let original_b = std::fs::read(second_root.join(relative_path)).expect("B bytes");
        assert_eq!(original_a, original_b);

        let mut app = RunenwerkEditorApp::new();
        app.load_project_file(&first_project)
            .expect("A missing catalog should be a controlled diagnostic");
        let mut a_catalog = asset::AssetCatalog::new();
        a_catalog.insert_asset_record(
            asset::AssetRecord::new(
                asset_id,
                "catalog-from-A",
                "Material from A",
                asset::AssetKind::MaterialGraph,
            )
            .with_primary_source(source_id),
        );
        a_catalog.insert_source(source);
        app.asset_catalog_runtime_mut().replace_catalog(a_catalog);
        app.select_material_asset(asset_id);
        assert!(
            app.material_published_source_origin_for_selected(None)
                .is_ok(),
            "A source is valid under A's in-memory last-known-good catalog"
        );

        app.load_project_file(&first_project)
            .expect("same project reload may preserve last-known-good catalog");
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(asset_id)
                .is_some(),
            "within-project rejected catalog preserves its last-known-good catalog"
        );
        app.select_material_asset(asset_id);

        let prior_origin = app
            .material_source_origin_for_selected(None)
            .expect("A selected source origin must exist before rejected switch");
        let rejected = app.load_project_file(&second_project);
        assert!(
            rejected.is_err(),
            "switching projects must not silently discard an unsaved A catalog"
        );
        assert_eq!(
            app.asset_project_session()
                .expect("A session retained")
                .project_root(),
            first_root.as_path(),
            "failed switch must not change the active project"
        );
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(asset_id)
                .is_some(),
            "failed switch must preserve the unsaved catalog"
        );
        assert_eq!(
            app.material_source_origin_for_selected(None)
                .expect("A origin retained after rejection"),
            prior_origin,
            "failed switch must preserve Material invocation identity"
        );

        app.save_asset_project_catalog()
            .expect("save of A catalog should complete");
        let catalog_path = first_root.join("assets/catalog.ron");
        assert!(
            catalog_path.is_file(),
            "A catalog must be ratified and persisted before switching projects"
        );
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(asset::AssetRecord::new(
                asset::asset_id(1543),
                "unsaved-catalog-entry",
                "Unsaved Catalog Entry",
                asset::AssetKind::MaterialGraph,
            ));
        assert!(
            app.load_project_file(&second_project).is_err(),
            "even an existing disk catalog must not hide newer in-memory changes"
        );
        assert_eq!(
            app.material_source_origin_for_selected(None)
                .expect("A origin retained after stale-disk rejection"),
            prior_origin
        );
        app.load_asset_project_catalog()
            .expect("discard test mutation by loading A's persisted catalog");

        app.load_project_file(&second_project)
            .expect("clean A catalog permits B switch despite missing B catalog");
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(asset_id)
                .is_none(),
            "project B must not inherit project A's catalog on a missing/invalid B catalog"
        );
        assert_eq!(
            app.material_lab_runtime().selected_material_asset_id(),
            None
        );
        assert!(
            app.material_source_origin_for_selected(None).is_err(),
            "identically named B files cannot be edited through stale A catalog descriptors"
        );
        assert_eq!(
            std::fs::read(first_root.join(relative_path)).expect("A stable"),
            original_a
        );
        assert_eq!(
            std::fs::read(second_root.join(relative_path)).expect("B stable"),
            original_b
        );

        let _ = std::fs::remove_dir_all(first_root);
        let _ = std::fs::remove_dir_all(second_root);
    }

    fn unique_temp_dir(label: &str) -> std::path::PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        root.push(format!("{label}_{nanos}"));
        std::fs::create_dir_all(&root).expect("temp dir should be creatable");
        root
    }
}
