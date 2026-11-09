use std::path::Path;

use anyhow::{Context, Result};

use crate::asset_pipeline::{EditorAssetProjectSession, load_project_catalog};
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
        // A hard read failure in B must not discard the active A session,
        // catalog, Material selection, or history before returning an error.
        // Publish this exact staged result instead of reading B again.
        let catalog_outcome = load_project_catalog(&session).with_context(|| {
            format!(
                "failed to load asset catalog for project: {}",
                path.display()
            )
        })?;
        // Only a reload of the same logical project/catalog context may
        // preserve its in-memory last-known-good fallback. Distinct project
        // IDs do not share that state just because their catalog path matches.
        // Explicit catalog reload remains the separate restore operation.
        let same_catalog_fallback_context = self
            .asset_project_session()
            .is_some_and(|current| current.shares_catalog_fallback_context(&session));
        if same_catalog_fallback_context
            && let Some(persisted) = catalog_outcome.accepted_catalog.as_ref()
        {
            anyhow::ensure!(
                self.asset_catalog_runtime().catalog() == persisted,
                "project reload rejected: the in-memory asset catalog differs from the accepted on-disk catalog; save or explicitly reload the asset catalog first"
            );
        }
        self.set_asset_project_session(session)?;
        self.apply_asset_project_catalog_load_outcome(catalog_outcome);
        Ok(())
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

        // A hard B catalog read failure must not partially commit the
        // switch after clearing A's Material and catalog state.
        let invalid_b_catalog = second_root.join("assets/catalog.ron");
        std::fs::create_dir_all(&invalid_b_catalog).expect("create B catalog path as a directory");
        let origin_before_failure = app
            .material_source_origin_for_selected(None)
            .expect("A origin remains active after restoring its catalog");
        let failed = app
            .load_project_file(&second_project)
            .expect_err("unreadable B catalog must fail without switching projects");
        assert!(
            format!("{failed:#}").contains("failed to read asset catalog"),
            "failure must come from the incoming catalog read: {failed:#}"
        );
        assert_eq!(
            app.asset_project_session()
                .expect("A session preserved after B read failure")
                .project_root(),
            first_root.as_path()
        );
        assert_eq!(
            app.material_source_origin_for_selected(None)
                .expect("A Material origin preserved after B read failure"),
            origin_before_failure
        );
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(asset_id)
                .is_some(),
            "A catalog must survive a hard B catalog read error"
        );
        std::fs::remove_dir(&invalid_b_catalog)
            .expect("remove B directory fixture before successful switch");

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

    #[test]
    fn same_project_reload_rejects_newer_in_memory_catalog_without_discarding_it() {
        let root = unique_temp_dir("same_project_catalog_reload_guard");
        let project_path = root.join("project.rune.ron");
        let project = ProjectFileV3::new("project.reload", "Reload");
        std::fs::write(
            &project_path,
            encode_ron_pretty(&project).expect("project should encode"),
        )
        .expect("project file should be writable");

        let mut app = RunenwerkEditorApp::new();
        app.load_project_file(&project_path)
            .expect("initial project load should succeed");
        let initial_id = asset::asset_id(2501);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(asset::AssetRecord::new(
                initial_id,
                "persisted-material",
                "Persisted Material",
                asset::AssetKind::MaterialGraph,
            ));
        app.save_asset_project_catalog()
            .expect("valid catalog should be saved");
        let catalog_path = root.join("assets/catalog.ron");
        let saved_bytes = std::fs::read(&catalog_path).expect("catalog file exists");

        let unsaved_id = asset::asset_id(2502);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(asset::AssetRecord::new(
                unsaved_id,
                "unsaved-material",
                "Unsaved Material",
                asset::AssetKind::MaterialGraph,
            ));
        let unsaved_catalog = app.asset_catalog_runtime().catalog().clone();
        let err = app
            .load_project_file(&project_path)
            .expect_err("reopening the project cannot silently lose newer catalog edits");
        assert!(
            err.to_string().contains("project reload rejected"),
            "failure must identify the guarded reload: {err:#}"
        );
        assert_eq!(
            app.asset_project_session()
                .expect("project session must remain active")
                .project_root(),
            root.as_path()
        );
        assert_eq!(
            app.asset_catalog_runtime().catalog(),
            &unsaved_catalog,
            "rejection must retain the in-memory catalog"
        );
        assert_eq!(
            std::fs::read(&catalog_path).expect("accepted disk catalog remains readable"),
            saved_bytes,
            "project reload rejection cannot write a catalog"
        );

        // This explicit action is the existing discard/restore path.
        app.load_asset_project_catalog()
            .expect("explicit catalog reload should restore disk state");
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(unsaved_id)
                .is_none(),
            "explicit restore discards only the unsaved test entry"
        );
        app.load_project_file(&project_path)
            .expect("a clean same-project reload should still succeed");
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(initial_id)
                .is_some()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn distinct_project_ids_at_same_catalog_path_cannot_share_unsaved_fallback() {
        let root = unique_temp_dir("catalog_distinct_logical_project_id");
        let project_a = root.join("a.project.ron");
        let project_b = root.join("b.project.ron");
        std::fs::write(
            &project_a,
            encode_ron_pretty(&ProjectFileV3::new("project.A", "Project A"))
                .expect("encode project A"),
        )
        .expect("write A project file");
        std::fs::write(
            &project_b,
            encode_ron_pretty(&ProjectFileV3::new("project.B", "Project B"))
                .expect("encode project B"),
        )
        .expect("write B project file");

        let mut app = RunenwerkEditorApp::new();
        app.load_project_file(&project_a)
            .expect("A may open without a catalog");
        let persisted_id = asset::asset_id(18811);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(asset::AssetRecord::new(
                persisted_id,
                "a-material",
                "A Material",
                asset::AssetKind::MaterialGraph,
            ));
        let unsaved_a = app.asset_catalog_runtime().catalog().clone();
        assert!(
            app.load_project_file(&project_b).is_err(),
            "different project_id at the same catalog path must not inherit unsaved A"
        );
        assert_eq!(
            app.asset_project_session()
                .and_then(|session| session.project_id()),
            Some("project.A")
        );
        assert_eq!(app.asset_catalog_runtime().catalog(), &unsaved_a);

        app.save_asset_project_catalog()
            .expect("persist the approved shared physical catalog");
        let disk_path = root.join("assets/catalog.ron");
        let accepted_bytes = std::fs::read(&disk_path).expect("accepted catalog bytes");

        let unpersisted_id = asset::asset_id(18812);
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(asset::AssetRecord::new(
                unpersisted_id,
                "unpersisted-material",
                "Unpersisted Material",
                asset::AssetKind::MaterialGraph,
            ));
        assert!(
            app.load_project_file(&project_b).is_err(),
            "accepted physical catalog cannot hide newer A memory edits"
        );
        assert_eq!(
            std::fs::read(&disk_path).expect("disk catalog retained"),
            accepted_bytes
        );
        app.load_asset_project_catalog()
            .expect("explicit restore of A's persisted catalog");
        app.load_project_file(&project_b)
            .expect("a clean A can switch to B using the shared accepted physical catalog");
        assert_eq!(
            app.asset_project_session()
                .and_then(|session| session.project_id()),
            Some("project.B")
        );
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(persisted_id)
                .is_some()
        );
        assert!(
            app.asset_catalog_runtime()
                .catalog()
                .asset(unpersisted_id)
                .is_none(),
            "B must see only the accepted catalog bytes, not unsaved A state"
        );
        assert_eq!(
            std::fs::read(&disk_path).expect("shared disk catalog unchanged"),
            accepted_bytes
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejected_catalog_save_reports_error_without_overwriting_valid_source() {
        let root = unique_temp_dir("material_catalog_save_rejection");
        let project_path = root.join("project.rune.ron");
        let project = ProjectFileV3::new("project.catalog", "Catalog");
        std::fs::write(
            &project_path,
            encode_ron_pretty(&project).expect("project should encode"),
        )
        .expect("project file should be writable");

        let mut app = RunenwerkEditorApp::new();
        app.load_project_file(&project_path)
            .expect("new project catalog may be absent");
        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(asset::AssetRecord::new(
                asset::asset_id(1981),
                "valid-material",
                "Valid Material",
                asset::AssetKind::MaterialGraph,
            ));
        app.save_asset_project_catalog()
            .expect("valid catalog save must succeed");
        let catalog_path = root.join("assets/catalog.ron");
        let persisted = std::fs::read(&catalog_path).expect("saved catalog bytes");

        app.asset_catalog_runtime_mut()
            .catalog_mut()
            .insert_asset_record(
                asset::AssetRecord::new(
                    asset::asset_id(1982),
                    "invalid-material",
                    "Invalid Material",
                    asset::AssetKind::MaterialGraph,
                )
                .with_primary_source(asset::asset_source_id(1983)),
            );
        assert!(
            app.save_asset_project_catalog().is_err(),
            "ratification-rejected save must return an error"
        );
        assert_eq!(
            std::fs::read(&catalog_path).expect("previously valid catalog remains readable"),
            persisted,
            "rejected save cannot overwrite the accepted disk catalog"
        );
        assert!(
            app.asset_catalog_status_lines()
                .iter()
                .any(|line| line.contains("rejected; invalid catalog not written")),
            "save status must explicitly report rejection"
        );

        let _ = std::fs::remove_dir_all(root);
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
