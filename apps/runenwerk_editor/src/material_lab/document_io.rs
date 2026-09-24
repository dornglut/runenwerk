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
    let version = match ron::from_str::<MaterialGraphSourceVersionProbe>(&source) {
        Ok(probe) => probe.version,
        Err(error) => {
            return Ok(MaterialGraphSourceLoadOutcome::DecodeRejected {
                message: error.to_string(),
            });
        }
    };
    if version == 1 {
        return Ok(MaterialGraphSourceLoadOutcome::SupersededV1 { version });
    }
    if version != MATERIAL_GRAPH_SOURCE_FILE_VERSION_V2 {
        return Ok(MaterialGraphSourceLoadOutcome::Unsupported { version });
    }
    let source_file = match ron::from_str::<MaterialGraphSourceFileV2>(&source) {
        Ok(source_file) => source_file,
        Err(error) => {
            return Ok(MaterialGraphSourceLoadOutcome::DecodeRejected {
                message: error.to_string(),
            });
        }
    };
    Ok(match source_file.into_document() {
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
    })
}

pub fn write_material_graph_document(path: &Path, document: &MaterialGraphDocument) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create material graph source parent: {}",
                parent.display()
            )
        })?;
    }
    let config = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    let source = MaterialGraphSourceFileV2::from_document(document);
    let ron = ron::ser::to_string_pretty(&source, config)
        .context("failed to encode material graph source")?;
    std::fs::write(path, ron)
        .with_context(|| format!("failed to write material graph source: {}", path.display()))
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
