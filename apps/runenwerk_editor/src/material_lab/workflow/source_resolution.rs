use super::diagnostics::material_diagnostic;
use super::*;

pub fn material_source_for_asset(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Option<&AssetSourceDescriptor> {
    resolve_material_source_for_asset(catalog, asset_id).ok()
}

pub fn resolve_material_source_for_asset(
    catalog: &AssetCatalog,
    asset_id: AssetId,
) -> Result<&AssetSourceDescriptor, AssetDiagnosticRecord> {
    let Some(record) = catalog.asset(asset_id) else {
        return Err(material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            format!("asset {} is not present in the catalog", asset_id.raw()),
        ));
    };
    if record.kind != AssetKind::MaterialGraph {
        return Err(material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            format!(
                "asset {} is {:?}, not a material graph asset",
                asset_id.raw(),
                record.kind
            ),
        ));
    }
    if let Some(primary_source_id) = record.primary_source_id {
        let Some(source) = catalog.source(primary_source_id) else {
            return Err(material_diagnostic(
                AssetDiagnosticCode::SourceMissing,
                format!(
                    "asset {} primary material graph source {} is missing",
                    asset_id.raw(),
                    primary_source_id.raw()
                ),
            ));
        };
        if source.asset_id != asset_id || source.kind != AssetKind::MaterialGraph {
            return Err(material_diagnostic(
                AssetDiagnosticCode::RatificationRejected,
                format!(
                    "asset {} primary source {} is {:?} for asset {}",
                    asset_id.raw(),
                    source.source_id.raw(),
                    source.kind,
                    source.asset_id.raw()
                ),
            ));
        }
        return Ok(source);
    }

    let mut material_sources = catalog
        .sources
        .values()
        .filter(|source| source.asset_id == asset_id && source.kind == AssetKind::MaterialGraph);
    let Some(first) = material_sources.next() else {
        return Err(material_diagnostic(
            AssetDiagnosticCode::SourceMissing,
            format!(
                "asset {} has no material graph source descriptor",
                asset_id.raw()
            ),
        ));
    };
    if material_sources.next().is_some() {
        return Err(material_diagnostic(
            AssetDiagnosticCode::RatificationRejected,
            format!(
                "asset {} has multiple material graph sources and no primary source",
                asset_id.raw()
            ),
        ));
    }
    Ok(first)
}

pub fn default_material_graph_document_for_source(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
    label: impl Into<String>,
) -> MaterialGraphDocument {
    default_material_graph_document_for_source_with_target(
        asset_id,
        source,
        label,
        MaterialOutputTarget::PbrPreview,
    )
}

pub fn default_material_graph_document_for_source_with_target(
    asset_id: AssetId,
    source: &AssetSourceDescriptor,
    label: impl Into<String>,
    output_target: MaterialOutputTarget,
) -> MaterialGraphDocument {
    use graph::{
        CyclePolicy, EdgeDefinition, EdgeId, GraphDefinition, GraphId, GraphMetadataEntry,
        GraphValue, NodeDefinition, NodeId, PortDefinition, PortDirection, PortId,
    };
    let color_port_type = material_graph::MaterialValueType::Color.port_type_id();
    let base_color_node = NodeId::new(1);
    let base_color_port = PortId::new(1);
    let output_node = NodeId::new(2);
    let output_base_color_port = PortId::new(2);
    let mut editor_state = material_graph::MaterialGraphEditorState::default();
    editor_state.selected_fixture = material_graph::MaterialGraphPreviewFixture::SdfPrimitive;
    editor_state.selected_preview = if output_target == MaterialOutputTarget::RenderMaterial {
        material_graph::MaterialGraphPreviewSelection::SceneProduct
    } else {
        material_graph::MaterialGraphPreviewSelection::MaterialPreviewProduct
    };
    editor_state.node_layouts = vec![
        material_graph::MaterialGraphNodeLayout::new(base_color_node, 40, 96),
        material_graph::MaterialGraphNodeLayout::new(output_node, 360, 96),
    ];

    MaterialGraphDocument::new(
        material_document_id_for_source(asset_id, source.source_id),
        label,
        GraphDefinition::new(
            GraphId::new(1),
            "material.preview",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    base_color_node,
                    "pbr.base_color",
                    [PortDefinition::new(
                        base_color_port,
                        "color",
                        PortDirection::Output,
                        color_port_type,
                    )],
                )
                .with_values([GraphMetadataEntry::new(
                    "color",
                    GraphValue::Text("0.08 0.62 0.95 1.0".to_string()),
                )]),
                NodeDefinition::new(
                    output_node,
                    "pbr.output",
                    [PortDefinition::new(
                        output_base_color_port,
                        "base_color",
                        PortDirection::Input,
                        color_port_type,
                    )],
                ),
            ],
            [EdgeDefinition::new(
                EdgeId::new(1),
                base_color_port,
                output_base_color_port,
            )],
        ),
        output_target,
    )
    .with_editor_state(editor_state)
}
