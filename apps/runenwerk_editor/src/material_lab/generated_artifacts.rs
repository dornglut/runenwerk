//! File: apps/runenwerk_editor/src/material_lab/generated_artifacts.rs
//! Purpose: Manifest-owned generated material artifact lifecycle contracts.

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GeneratedMaterialArtifactManifest {
    live_paths: BTreeSet<String>,
}

impl GeneratedMaterialArtifactManifest {
    pub fn from_live_paths(paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            live_paths: paths.into_iter().map(Into::into).collect(),
        }
    }

    pub fn contains(&self, path: &str) -> bool {
        self.live_paths.contains(path)
    }

    pub fn stale_paths(
        &self,
        existing_paths: impl IntoIterator<Item = impl Into<String>>,
    ) -> Vec<String> {
        existing_paths
            .into_iter()
            .map(Into::into)
            .filter(|path| !self.live_paths.contains(path))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_gc_keeps_only_live_generated_artifacts() {
        let manifest = GeneratedMaterialArtifactManifest::from_live_paths([
            ".runenwerk/artifacts/generated/material-shader/live.wgsl",
            ".runenwerk/artifacts/generated/material-scene-shader/live.wgsl",
        ]);

        let stale = manifest.stale_paths([
            ".runenwerk/artifacts/generated/material-shader/live.wgsl",
            ".runenwerk/artifacts/generated/material-shader/stale.wgsl",
        ]);

        assert_eq!(
            stale,
            vec![".runenwerk/artifacts/generated/material-shader/stale.wgsl"]
        );
        assert!(manifest.contains(".runenwerk/artifacts/generated/material-shader/live.wgsl"));
    }
}
