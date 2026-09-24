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
        self.set_asset_project_session(session);
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
