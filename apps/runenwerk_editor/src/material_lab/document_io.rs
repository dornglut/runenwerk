use std::path::Path;

use anyhow::{Context, Result};
use material_graph::{MaterialGraphDocument, MaterialGraphSourceFileV1};

pub fn read_material_graph_document(path: &Path) -> Result<MaterialGraphDocument> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read material graph source: {}", path.display()))?;
    let source_file: MaterialGraphSourceFileV1 = ron::from_str(&source)
        .with_context(|| format!("failed to decode material graph source: {}", path.display()))?;
    source_file
        .into_document()
        .map_err(|issue| anyhow::anyhow!("material graph source rejected: {issue:?}"))
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
    let source = MaterialGraphSourceFileV1::from_document(document);
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
