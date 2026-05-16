//! File: domain/material_graph/src/lowering.rs
//! Purpose: Deterministic lowering from ratified material graph documents to formed product descriptors.

use graph::{CyclePolicy, PortDirection};

use crate::{
    FormedMaterialProduct, MaterialCacheKey, MaterialGraphDocument,
    MaterialGraphRatificationReport, MaterialNodeCatalog, MaterialOutputTarget,
    MaterialParameterDescriptor, MaterialParameterKind, MaterialProductId, MaterialSourceMap,
    MaterialSpecializationFragment, ratify_material_graph,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialLoweringResult {
    pub report: MaterialGraphRatificationReport,
    pub product: Option<FormedMaterialProduct>,
}

pub fn lower_material_graph(
    document: &MaterialGraphDocument,
    catalog: &MaterialNodeCatalog,
) -> MaterialLoweringResult {
    let report = ratify_material_graph(document, catalog);
    if report.has_blocking_issues() {
        return MaterialLoweringResult {
            report,
            product: None,
        };
    }

    let mut product = FormedMaterialProduct::new(
        MaterialProductId::new(document.document_id.raw()),
        document.document_id,
        document.output_target,
        deterministic_cache_key(document),
    );
    product.parameters = material_parameters_for_output(document.output_target);
    product.source_map =
        MaterialSourceMap::from_nodes(document.graph.nodes.iter().map(|node| node.id));
    product.specialization_fragment = MaterialSpecializationFragment::new(
        specialization_fragment_for_output(document.output_target),
    );

    MaterialLoweringResult {
        report,
        product: Some(product),
    }
}

fn material_parameters_for_output(
    output_target: MaterialOutputTarget,
) -> Vec<MaterialParameterDescriptor> {
    let mut parameters = vec![
        MaterialParameterDescriptor::new("base_color", MaterialParameterKind::Vector4),
        MaterialParameterDescriptor::new("roughness", MaterialParameterKind::Scalar),
        MaterialParameterDescriptor::new("metallic", MaterialParameterKind::Scalar),
        MaterialParameterDescriptor::new("normal_strength", MaterialParameterKind::Scalar),
        MaterialParameterDescriptor::new("emissive", MaterialParameterKind::Vector3),
        MaterialParameterDescriptor::new("opacity", MaterialParameterKind::Scalar),
    ];
    if output_target == MaterialOutputTarget::FieldMaterialChannel {
        parameters.push(MaterialParameterDescriptor::new(
            "material_channel",
            MaterialParameterKind::Scalar,
        ));
    }
    parameters
}

fn specialization_fragment_for_output(output_target: MaterialOutputTarget) -> &'static str {
    match output_target {
        MaterialOutputTarget::PbrPreview => "material.first_slice.pbr_preview",
        MaterialOutputTarget::FieldMaterialChannel => "material.first_slice.field_channel",
        MaterialOutputTarget::RenderMaterial => "material.first_slice.render_material",
    }
}

fn deterministic_cache_key(document: &MaterialGraphDocument) -> MaterialCacheKey {
    let mut node_parts = document
        .graph
        .nodes
        .iter()
        .map(|node| {
            let mut port_parts = node
                .ports
                .iter()
                .map(|port| {
                    format!(
                        "{}:{}:{}:{}",
                        port.id.raw(),
                        port.name,
                        port_direction_label(port.direction),
                        port.port_type.raw()
                    )
                })
                .collect::<Vec<_>>();
            port_parts.sort();
            format!(
                "{}:{}:ports=[{}]",
                node.id.raw(),
                node.name,
                port_parts.join("|")
            )
        })
        .collect::<Vec<_>>();
    node_parts.sort();
    let mut edge_parts = document
        .graph
        .edges
        .iter()
        .map(|edge| {
            format!(
                "{}:{}->{}",
                edge.id.raw(),
                edge.from_port.raw(),
                edge.to_port.raw()
            )
        })
        .collect::<Vec<_>>();
    edge_parts.sort();
    MaterialCacheKey::new(format!(
        "material-graph-v1-document={}:graph={}:graph_name={}:cycle={}:target={}:nodes={}:edges={}",
        document.document_id.raw(),
        document.graph.id.raw(),
        document.graph.name,
        cycle_policy_label(document.graph.cycle_policy),
        output_target_label(document.output_target),
        node_parts.join(","),
        edge_parts.join(","),
    ))
}

fn output_target_label(output_target: MaterialOutputTarget) -> &'static str {
    match output_target {
        MaterialOutputTarget::PbrPreview => "pbr_preview",
        MaterialOutputTarget::FieldMaterialChannel => "field_material_channel",
        MaterialOutputTarget::RenderMaterial => "render_material",
    }
}

fn cycle_policy_label(policy: CyclePolicy) -> &'static str {
    match policy {
        CyclePolicy::AllowDirectedCycles => "allow_directed_cycles",
        CyclePolicy::RejectDirectedCycles => "reject_directed_cycles",
    }
}

fn port_direction_label(direction: PortDirection) -> &'static str {
    match direction {
        PortDirection::Input => "input",
        PortDirection::Output => "output",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graph::{
        CyclePolicy, GraphDefinition, GraphId, NodeDefinition, NodeId, PortDefinition,
        PortDirection, PortId, PortTypeId,
    };

    fn pbr_graph() -> GraphDefinition {
        let color = PortTypeId::new(1);
        GraphDefinition::new(
            GraphId::new(1),
            "pbr",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(
                    NodeId::new(1),
                    "pbr.base_color",
                    [PortDefinition::new(
                        PortId::new(1),
                        "color",
                        PortDirection::Output,
                        color,
                    )],
                ),
                NodeDefinition::new(
                    NodeId::new(2),
                    "pbr.output",
                    [PortDefinition::new(
                        PortId::new(2),
                        "base_color",
                        PortDirection::Input,
                        color,
                    )],
                ),
            ],
            [graph::EdgeDefinition::new(
                graph::EdgeId::new(1),
                PortId::new(1),
                PortId::new(2),
            )],
        )
    }

    #[test]
    fn valid_material_graph_lowers_to_formed_product_with_source_map() {
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(7),
            "rock",
            pbr_graph(),
            MaterialOutputTarget::RenderMaterial,
        );

        let result = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());

        assert!(result.report.is_accepted());
        let product = result.product.expect("valid graph should form a product");
        assert_eq!(product.product_id, MaterialProductId::new(7));
        assert_eq!(product.source_map.entries.len(), 2);
        assert_eq!(
            product.specialization_fragment.0,
            "material.first_slice.render_material"
        );
    }

    #[test]
    fn unsupported_material_node_blocks_lowering() {
        let mut graph = pbr_graph();
        graph.nodes[0].name = "private.renderer_shader".to_string();
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(8),
            "bad",
            graph,
            MaterialOutputTarget::RenderMaterial,
        );

        let result = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());

        assert!(result.report.has_blocking_issues());
        assert!(result.product.is_none());
        assert_eq!(
            result.report.issues()[0].code(),
            &crate::MaterialGraphIssueCode::UnsupportedNode
        );
    }

    #[test]
    fn cache_key_is_deterministic_for_same_graph() {
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&document, &MaterialNodeCatalog::first_slice())
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&document, &MaterialNodeCatalog::first_slice())
            .product
            .expect("valid graph should form");

        assert_eq!(first.cache_key, second.cache_key);
    }

    #[test]
    fn cache_key_changes_when_output_target_changes() {
        let pbr_preview = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );
        let render_material = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::RenderMaterial,
        );

        let first = lower_material_graph(&pbr_preview, &MaterialNodeCatalog::first_slice())
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&render_material, &MaterialNodeCatalog::first_slice())
            .product
            .expect("valid graph should form");

        assert_ne!(first.cache_key, second.cache_key);
    }

    #[test]
    fn cache_key_changes_when_port_contract_changes() {
        let base = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );
        let mut changed_graph = pbr_graph();
        changed_graph.nodes[0].ports[0].name = "albedo".to_string();
        let changed = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            changed_graph,
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&base, &MaterialNodeCatalog::first_slice())
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&changed, &MaterialNodeCatalog::first_slice())
            .product
            .expect("valid graph should form");

        assert_ne!(first.cache_key, second.cache_key);
    }
}
