use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use material_graph::{
    MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2, MaterialGraphDocument, MaterialGraphSourceFileV2,
    MaterialGraphSourceIssue,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MaterialGraphSourceVersionProbe {
    version: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialGraphSourceLoadOutcome {
    Ready(MaterialGraphDocument),
    SupersededV1 { version: u32 },
    Unsupported { version: u32 },
    DecodeRejected { message: String },
}

impl MaterialGraphSourceLoadOutcome {
    pub fn into_document(self) -> Result<MaterialGraphDocument> {
        match self {
            Self::Ready(document) => Ok(document),
            Self::SupersededV1 { version } => Err(anyhow::anyhow!(
                "material graph source v{version} is superseded; create a V2 graph from template before editing"
            )),
            Self::Unsupported { version } => Err(anyhow::anyhow!(
                "material graph source version {version} is unsupported"
            )),
            Self::DecodeRejected { message } => Err(anyhow::anyhow!(
                "material graph source decode rejected: {message}"
            )),
        }
    }
}

pub fn read_material_graph_document(path: &Path) -> Result<MaterialGraphDocument> {
    read_material_graph_source(path)?.into_document()
}

pub fn read_material_graph_source(path: &Path) -> Result<MaterialGraphSourceLoadOutcome> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read material graph source: {}", path.display()))?;
    Ok(classify_material_graph_source(&source))
}

/// Returns the decoded document and a content revision of the *same exact read*.
/// This is an owner admission witness, not an atomic filesystem CAS operation.
pub fn read_material_graph_document_with_digest(
    path: &Path,
) -> Result<(MaterialGraphDocument, [u8; 32])> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("failed to read material graph source: {}", path.display()))?;
    let source = std::str::from_utf8(&bytes).context("material graph source is not UTF-8")?;
    let document = classify_material_graph_source(source).into_document()?;
    Ok((document, *blake3::hash(&bytes).as_bytes()))
}

fn classify_material_graph_source(source: &str) -> MaterialGraphSourceLoadOutcome {
    let version = match ron::from_str::<MaterialGraphSourceVersionProbe>(source) {
        Ok(probe) => probe.version,
        Err(error) => {
            return MaterialGraphSourceLoadOutcome::DecodeRejected {
                message: error.to_string(),
            };
        }
    };
    if version == 1 {
        return MaterialGraphSourceLoadOutcome::SupersededV1 { version };
    }
    if version != MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2 {
        return MaterialGraphSourceLoadOutcome::Unsupported { version };
    }
    let source_file = match ron::from_str::<MaterialGraphSourceFileV2>(source) {
        Ok(source_file) => source_file,
        Err(error) => {
            return MaterialGraphSourceLoadOutcome::DecodeRejected {
                message: error.to_string(),
            };
        }
    };
    match source_file.into_document() {
        Ok(document) => MaterialGraphSourceLoadOutcome::Ready(document),
        Err(MaterialGraphSourceIssue::SupersededVersion(version)) => {
            MaterialGraphSourceLoadOutcome::SupersededV1 { version }
        }
        Err(MaterialGraphSourceIssue::UnsupportedVersion(version)) => {
            MaterialGraphSourceLoadOutcome::Unsupported { version }
        }
        Err(issue) => MaterialGraphSourceLoadOutcome::DecodeRejected {
            message: format!("{issue:?}"),
        },
    }
}

/// Writes a generated or explicitly created Material source without a prior
/// revision requirement. Existing sources still use same-directory staging.
pub fn write_material_graph_document(path: &Path, document: &MaterialGraphDocument) -> Result<()> {
    write_material_graph_document_if_revision(path, document, None)
}

/// An authored edit carries the digest observed at action admission. The live
/// path is rechecked after staging and before replacement. This does not
/// implement cross-process compare-and-swap or power-loss durability.
pub(crate) fn write_material_graph_document_if_revision(
    path: &Path,
    document: &MaterialGraphDocument,
    expected_digest: Option<[u8; 32]>,
) -> Result<()> {
    staged_material_graph_write(
        path,
        document,
        expected_digest,
        |staged, bytes| staged.write_all(bytes),
        |staged| staged.as_file().sync_all(),
        |staged, target| {
            staged
                .persist(target)
                .map(|_| ())
                .map_err(|error| error.error)
        },
    )
}

/// Three explicit I/O seams let tests fail a partial staging write, staging
/// sync, and replacement without ever truncating the accepted source.
fn staged_material_graph_write<W, S, R>(
    path: &Path,
    document: &MaterialGraphDocument,
    expected_digest: Option<[u8; 32]>,
    write_staged: W,
    sync_staged: S,
    replace_staged: R,
) -> Result<()>
where
    W: FnOnce(&mut tempfile::NamedTempFile, &[u8]) -> io::Result<()>,
    S: FnOnce(&tempfile::NamedTempFile) -> io::Result<()>,
    R: FnOnce(tempfile::NamedTempFile, &Path) -> io::Result<()>,
{
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).with_context(|| {
        format!(
            "failed to create material graph source parent: {}",
            parent.display()
        )
    })?;
    let config = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let source = MaterialGraphSourceFileV2::from_document(document);
    let encoded = ron::ser::to_string_pretty(&source, config)
        .context("failed to encode material graph source")?;

    // A failed write/sync/permission operation affects only a disposable
    // staging file. The original source is not opened for writing.
    let mut staged = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to stage material graph source: {}", path.display()))?;
    write_staged(&mut staged, encoded.as_bytes())
        .with_context(|| format!("failed to stage material graph bytes: {}", path.display()))?;
    staged
        .flush()
        .context("failed to flush staged material graph source")?;

    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "material source replacement refuses a symlink target: {}",
                path.display()
            );
            anyhow::ensure!(
                !metadata.permissions().readonly(),
                "material source replacement refuses a read-only target: {}",
                path.display()
            );
            staged
                .as_file()
                .set_permissions(metadata.permissions())
                .context("failed to preserve material graph source permissions")?;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "failed to inspect material graph source: {}",
                    path.display()
                )
            });
        }
    }
    sync_staged(&staged).context("failed to sync staged material graph source")?;

    if let Some(expected) = expected_digest {
        let current = std::fs::read(path).with_context(|| {
            format!(
                "failed to recheck material graph source revision: {}",
                path.display()
            )
        })?;
        anyhow::ensure!(
            blake3::hash(&current).as_bytes() == &expected,
            "material graph source revision changed before replacement: {}",
            path.display()
        );
    }
    // Once this replacement succeeds, no fallible post-commit operation is
    // reported as an uncommitted edit. Parent-directory durability is not
    // claimed here; Windows/native filesystem behavior needs its own proof.
    replace_staged(staged, path).with_context(|| {
        format!(
            "failed to replace material graph source: {}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId, PortDefinition,
        PortDirection, PortId, PortTypeId,
    };
    use material_graph::{MaterialGraphDocumentId, MaterialOutputTarget};

    #[test]
    fn material_graph_document_round_trips_through_project_source_file() {
        let root = unique_temp_dir("material_graph_document_round_trip");
        let path = root.join("assets/materials/rock.material.ron");
        let document = MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(7),
            "Rock",
            GraphDefinition::new(
                GraphId::new(1),
                "rock",
                CyclePolicy::RejectDirectedCycles,
                [NodeDefinition::new(
                    NodeId::new(1),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(1),
                        "base_color",
                        PortDirection::Input,
                        PortTypeId::new(1),
                    )],
                )],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        );

        write_material_graph_document(&path, &document).expect("document should write");
        let restored = read_material_graph_document(&path).expect("document should read");

        assert_eq!(restored, document);
        let (snapshot, digest) =
            read_material_graph_document_with_digest(&path).expect("revision snapshot should read");
        assert_eq!(snapshot, restored);
        assert_eq!(
            digest,
            *blake3::hash(&std::fs::read(&path).expect("snapshot bytes")).as_bytes()
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn failed_staged_io_and_stale_revision_preserve_accepted_material_bytes() {
        let root = unique_temp_dir("material_source_failed_staging");
        let path = root.join("assets/materials/rock.material.ron");
        let mut document = MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(991),
            "Original",
            GraphDefinition::new(
                GraphId::new(1),
                "source",
                CyclePolicy::RejectDirectedCycles,
                [NodeDefinition::new(NodeId::new(1), "pbr.output", [])],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        );
        write_material_graph_document(&path, &document).expect("initial source should persist");
        let initial = std::fs::read(&path).expect("accepted bytes");
        let expected = *blake3::hash(&initial).as_bytes();
        document.label = "Candidate".to_string();

        for failure in ["write", "sync", "replace"] {
            let result = staged_material_graph_write(
                &path,
                &document,
                Some(expected),
                |staged, bytes| {
                    if failure == "write" {
                        staged.write_all(&bytes[..bytes.len() / 2])?;
                        return Err(io::Error::other("injected short staging write"));
                    }
                    staged.write_all(bytes)
                },
                |staged| {
                    if failure == "sync" {
                        return Err(io::Error::other("injected staging sync failure"));
                    }
                    staged.as_file().sync_all()
                },
                |staged, target| {
                    if failure == "replace" {
                        return Err(io::Error::other("injected replacement failure"));
                    }
                    staged
                        .persist(target)
                        .map(|_| ())
                        .map_err(|error| error.error)
                },
            );
            assert!(result.is_err(), "injected {failure} must reject commit");
            assert_eq!(
                std::fs::read(&path).expect("accepted file still readable"),
                initial,
                "failed {failure} must not truncate or overwrite prior bytes"
            );
        }

        let stale = write_material_graph_document_if_revision(&path, &document, Some([42_u8; 32]));
        assert!(
            stale.is_err(),
            "stale source byte digest must reject replacement"
        );
        assert_eq!(std::fs::read(&path).expect("unchanged source"), initial);
        write_material_graph_document_if_revision(&path, &document, Some(expected))
            .expect("current revision permits a staged replacement");
        assert_eq!(
            read_material_graph_document(&path)
                .expect("updated document")
                .label,
            "Candidate"
        );
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn read_only_material_target_rejects_replacement_without_losing_source() {
        let root = unique_temp_dir("material_read_only_source");
        let path = root.join("assets/materials/read_only.material.ron");
        let document = MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(992),
            "Unchanged",
            GraphDefinition::new(
                GraphId::new(1),
                "readonly",
                CyclePolicy::RejectDirectedCycles,
                [NodeDefinition::new(NodeId::new(1), "pbr.output", [])],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        );
        write_material_graph_document(&path, &document).expect("original source should exist");
        let accepted = std::fs::read(&path).expect("accepted source bytes");
        let original_permissions = std::fs::metadata(&path)
            .expect("accepted source metadata")
            .permissions();
        let mut read_only = original_permissions.clone();
        read_only.set_readonly(true);
        std::fs::set_permissions(&path, read_only).expect("make accepted source read-only");

        let mut changed = document.clone();
        changed.label = "Unexpected".to_string();
        assert!(
            write_material_graph_document_if_revision(
                &path,
                &changed,
                Some(*blake3::hash(&accepted).as_bytes()),
            )
            .is_err(),
            "read-only source cannot be replaced by staging around its permission bit"
        );
        assert_eq!(
            std::fs::read(&path).expect("unchanged read-only source"),
            accepted
        );
        std::fs::set_permissions(&path, original_permissions)
            .expect("restore permissions for cleanup");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn v1_source_returns_recoverable_load_outcome() {
        let root = unique_temp_dir("material_graph_v1_recoverable");
        let path = root.join("assets/materials/legacy.material.ron");
        std::fs::create_dir_all(path.parent().expect("legacy source has a parent"))
            .expect("legacy source parent should write");
        std::fs::write(
            &path,
            r#"(
                version: 1,
                document_id: 7,
                label: "Legacy",
                output_target: PbrPreview,
                graph: (
                    id: 1,
                    name: "legacy",
                    cycle_policy: RejectDirectedCycles,
                    nodes: [],
                    edges: [],
                ),
            )"#,
        )
        .expect("legacy source should write");

        let outcome = read_material_graph_source(&path).expect("source should decode");

        assert_eq!(
            outcome,
            MaterialGraphSourceLoadOutcome::SupersededV1 { version: 1 }
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
