//! File: domain/material_graph/src/lowering.rs
//! Purpose: Deterministic lowering from ratified material graph documents to formed product descriptors.

use graph::{CyclePolicy, GraphValue, NodeDefinition, PortDirection, PortId};
use ratification::RatificationIssue;
use std::collections::BTreeMap;

use crate::{
    FormedMaterialProduct, MATERIAL_IR_CONTRACT_VERSION, MaterialCacheKey, MaterialGraphDocument,
    MaterialGraphIssueCode, MaterialGraphIssueSubject, MaterialGraphRatificationReport, MaterialIr,
    MaterialIrEdge, MaterialIrInput, MaterialIrInputSource, MaterialIrNode, MaterialIrOutput,
    MaterialIrValue, MaterialLiteral, MaterialNodeCatalog, MaterialNodeOp, MaterialOutputTarget,
    MaterialParameterDescriptor, MaterialParameterKind, MaterialProductId, MaterialResourceBinding,
    MaterialSourceMap, MaterialSpecializationFragment, MaterialValueContract,
    ratify_material_graph,
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
    let mut report = ratify_material_graph(document, catalog);
    let executable_ir = build_material_ir(document, catalog, &mut report);
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
        deterministic_cache_key(document, catalog),
    );
    product.parameters = material_parameters_for_output(document.output_target);
    product.source_map =
        MaterialSourceMap::from_nodes(document.graph.nodes.iter().map(|node| node.id));
    product.specialization_fragment = MaterialSpecializationFragment::new(
        specialization_fragment_for_output(document.output_target),
    );
    product.executable_ir =
        Some(executable_ir.expect("material IR exists when ratification has no blocking issues"));

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

fn deterministic_cache_key(
    document: &MaterialGraphDocument,
    catalog: &MaterialNodeCatalog,
) -> MaterialCacheKey {
    let mut encoder = CanonicalMaterialCacheEncoder::new();
    encoder.field("contract", "material-graph-cache-v2");
    encoder.number("ir_contract_version", MATERIAL_IR_CONTRACT_VERSION as u64);
    encoder.field("catalog_id", catalog.stable_id());
    encoder.number("catalog_version", catalog.version() as u64);
    let catalog_descriptors = catalog.descriptors().collect::<Vec<_>>();
    encoder.number("catalog_descriptor_count", catalog_descriptors.len() as u64);
    for descriptor in catalog_descriptors {
        encoder.field("catalog_node_key", &descriptor.key);
        encoder.number(
            "catalog_node_semantic_version",
            descriptor.semantic_version as u64,
        );
        encoder.field("catalog_node_op", descriptor.compiler_op.label());
        encoder.number("catalog_input_count", descriptor.inputs.len() as u64);
        for input in &descriptor.inputs {
            encoder.field("catalog_input_name", &input.name);
            encoder.field("catalog_input_type", input.value_type.label());
            encoder.field(
                "catalog_input_default",
                input
                    .default_value
                    .as_ref()
                    .map(MaterialLiteral::canonical_component)
                    .as_deref()
                    .unwrap_or("<required>"),
            );
        }
        encoder.number("catalog_output_count", descriptor.outputs.len() as u64);
        for output in &descriptor.outputs {
            encoder.field("catalog_output_name", &output.name);
            encoder.field("catalog_output_type", output.value_type.label());
        }
        encoder.number("catalog_value_count", descriptor.values.len() as u64);
        for value in &descriptor.values {
            encoder.field("catalog_value_key", &value.key);
            encoder.field("catalog_value_type", value.value_type.label());
            encoder.field(
                "catalog_value_default",
                value
                    .default_value
                    .as_ref()
                    .map(MaterialLiteral::canonical_component)
                    .as_deref()
                    .unwrap_or("<required>"),
            );
        }
        encoder.number("catalog_resource_count", descriptor.resources.len() as u64);
        for resource in &descriptor.resources {
            encoder.field("catalog_resource_key", &resource.key);
            encoder.field("catalog_resource_kind", resource.kind.label());
        }
    }
    encoder.number("document_id", document.document_id.raw());
    encoder.number("graph_id", document.graph.id.raw());
    encoder.field("graph_name", &document.graph.name);
    encoder.field(
        "cycle_policy",
        cycle_policy_label(document.graph.cycle_policy),
    );
    encoder.field("output_target", output_target_label(document.output_target));

    let mut nodes = document.graph.nodes.iter().collect::<Vec<_>>();
    nodes.sort_by(|left, right| {
        left.id
            .raw()
            .cmp(&right.id.raw())
            .then(left.name.cmp(&right.name))
    });
    encoder.number("node_count", nodes.len() as u64);
    for node in nodes {
        encoder.number("node_id", node.id.raw());
        encoder.field("node_name", &node.name);
        encode_entries(&mut encoder, "node_metadata", &node.metadata);
        encode_entries(&mut encoder, "node_values", &node.values);
        let mut ports = node.ports.iter().collect::<Vec<_>>();
        ports.sort_by(|left, right| {
            left.id
                .raw()
                .cmp(&right.id.raw())
                .then(left.name.cmp(&right.name))
                .then(
                    port_direction_label(left.direction).cmp(port_direction_label(right.direction)),
                )
                .then(left.port_type.raw().cmp(&right.port_type.raw()))
        });
        encoder.number("node_port_count", ports.len() as u64);
        for port in ports {
            encoder.number("port_id", port.id.raw());
            encoder.field("port_name", &port.name);
            encoder.field("port_direction", port_direction_label(port.direction));
            encoder.number("port_type", port.port_type.raw());
            encode_entries(&mut encoder, "port_metadata", &port.metadata);
        }
    }

    let mut edges = document.graph.edges.iter().collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        left.id
            .raw()
            .cmp(&right.id.raw())
            .then(left.from_port.raw().cmp(&right.from_port.raw()))
            .then(left.to_port.raw().cmp(&right.to_port.raw()))
    });
    encoder.number("edge_count", edges.len() as u64);
    for edge in edges {
        encoder.number("edge_id", edge.id.raw());
        encoder.number("edge_from_port", edge.from_port.raw());
        encoder.number("edge_to_port", edge.to_port.raw());
    }

    MaterialCacheKey::new(format!(
        "material-graph-cache-v2:{}:v{}:{}",
        catalog.stable_id(),
        catalog.version(),
        encoder.finish_hex(),
    ))
}

fn build_material_ir(
    document: &MaterialGraphDocument,
    catalog: &MaterialNodeCatalog,
    report: &mut MaterialGraphRatificationReport,
) -> Option<MaterialIr> {
    let mut ir_nodes = Vec::<MaterialIrNode>::new();
    let mut required_resources = Vec::<MaterialResourceBinding>::new();
    let port_index = MaterialPortIndex::new(document);

    for node in &document.graph.nodes {
        let Some(op) = MaterialNodeOp::from_catalog_key(&node.name) else {
            continue;
        };
        let descriptor = catalog.descriptor(&node.name).cloned();
        let Some(descriptor) = descriptor else {
            continue;
        };
        for resource in &descriptor.resources {
            match texture_resource_for_node(node, &resource.key) {
                Some(reference) => required_resources.push(MaterialResourceBinding::new(
                    node.id,
                    resource.key.clone(),
                    reference,
                )),
                None => report.push(RatificationIssue::error(
                    MaterialGraphIssueCode::MissingResourceReference,
                    MaterialGraphIssueSubject::Node(node.id),
                    format!(
                        "material node '{}' requires a catalog-backed texture resource reference",
                        node.name
                    ),
                )),
            }
        }
        ir_nodes.push(MaterialIrNode::new(
            node.id,
            op,
            descriptor.inputs.iter().map(|input| {
                let source = port_index
                    .input_source(node, input.name.as_str())
                    .unwrap_or_else(|| {
                        node.value(input.name.as_str())
                            .map(|value| MaterialIrInputSource::NodeValue {
                                key: input.name.clone(),
                                canonical_value: value.canonical_component(),
                            })
                            .or_else(|| {
                                input
                                    .default_value
                                    .clone()
                                    .map(MaterialIrInputSource::Constant)
                            })
                            .unwrap_or(MaterialIrInputSource::Constant(MaterialLiteral::float(
                                "0.0",
                            )))
                    });
                MaterialIrInput::new(input.name.clone(), input.value_type, source)
            }),
            descriptor
                .outputs
                .iter()
                .map(|output| MaterialIrOutput::new(output.name.clone(), output.value_type)),
            material_ir_values(node, &descriptor.values),
        ));
    }

    if report.has_blocking_issues() {
        return None;
    }

    Some(MaterialIr::new(
        document.document_id,
        document.output_target,
        ir_nodes,
        document
            .graph
            .edges
            .iter()
            .map(|edge| MaterialIrEdge::new(edge.id.raw(), edge.from_port, edge.to_port)),
        required_resources,
    ))
}

fn material_ir_values(
    node: &NodeDefinition,
    contracts: &[MaterialValueContract],
) -> Vec<MaterialIrValue> {
    let mut values = Vec::<MaterialIrValue>::new();
    for contract in contracts {
        if let Some(value) = node.value(contract.key.as_str()) {
            values.push(MaterialIrValue::new(
                contract.key.clone(),
                value.canonical_component(),
            ));
        } else if let Some(default) = &contract.default_value {
            values.push(MaterialIrValue::new(
                contract.key.clone(),
                default.canonical_component(),
            ));
        }
    }
    values
}

fn texture_resource_for_node(
    node: &NodeDefinition,
    key: &str,
) -> Option<resource_ref::ResourceRef> {
    match node.value(key)? {
        GraphValue::Resource(reference) => Some(reference.clone()),
        _ => None,
    }
}

struct MaterialPortIndex {
    port_to_output: BTreeMap<PortId, (graph::NodeId, String)>,
    input_edges: BTreeMap<PortId, PortId>,
}

impl MaterialPortIndex {
    fn new(document: &MaterialGraphDocument) -> Self {
        let mut port_to_output = BTreeMap::<PortId, (graph::NodeId, String)>::new();
        for node in &document.graph.nodes {
            for port in &node.ports {
                if port.direction == PortDirection::Output {
                    port_to_output.insert(port.id, (node.id, port.name.clone()));
                }
            }
        }
        let input_edges = document
            .graph
            .edges
            .iter()
            .map(|edge| (edge.to_port, edge.from_port))
            .collect::<BTreeMap<_, _>>();
        Self {
            port_to_output,
            input_edges,
        }
    }

    fn input_source(
        &self,
        node: &NodeDefinition,
        input_name: &str,
    ) -> Option<MaterialIrInputSource> {
        let input_port = node
            .ports
            .iter()
            .find(|port| port.direction == PortDirection::Input && port.name == input_name)?;
        let from_port = self.input_edges.get(&input_port.id)?;
        let (node_id, output_name) = self.port_to_output.get(from_port)?;
        Some(MaterialIrInputSource::Connected {
            node_id: *node_id,
            output_name: output_name.clone(),
        })
    }
}

fn encode_entries(
    encoder: &mut CanonicalMaterialCacheEncoder,
    label: &str,
    entries: &[graph::GraphMetadataEntry],
) {
    let mut entries = entries.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.key.cmp(&right.key));
    encoder.number(&format!("{label}_count"), entries.len() as u64);
    for entry in entries {
        encoder.field(&format!("{label}_key"), &entry.key);
        encoder.field(
            &format!("{label}_value"),
            &entry.value.canonical_component(),
        );
    }
}

struct CanonicalMaterialCacheEncoder {
    bytes: Vec<u8>,
}

impl CanonicalMaterialCacheEncoder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn field(&mut self, label: &str, value: &str) {
        self.bytes.extend_from_slice(label.as_bytes());
        self.bytes.push(b'=');
        self.bytes
            .extend_from_slice(value.as_bytes().len().to_string().as_bytes());
        self.bytes.push(b':');
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(b'\n');
    }

    fn number(&mut self, label: &str, value: u64) {
        self.field(label, &value.to_string());
    }

    fn finish_hex(self) -> String {
        bytes_to_lower_hex(&self.bytes)
    }
}

fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
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
        CyclePolicy, GraphDefinition, GraphId, GraphMetadataEntry, GraphValue, NodeDefinition,
        NodeId, PortDefinition, PortDirection, PortId, PortTypeId,
    };

    fn first_slice_descriptor(key: &str) -> crate::MaterialNodeDescriptor {
        MaterialNodeCatalog::first_slice()
            .descriptor(key)
            .expect("first-slice descriptor")
            .clone()
    }

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
        assert!(product.executable_ir.is_some());
    }

    #[test]
    fn texture_nodes_require_catalog_resource_refs() {
        let graph = GraphDefinition::new(
            GraphId::new(1),
            "pbr",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(NodeId::new(2), "pbr.output", Vec::<PortDefinition>::new()),
                NodeDefinition::new(
                    NodeId::new(3),
                    "texture.sample_2d",
                    Vec::<PortDefinition>::new(),
                ),
            ],
            [],
        );
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(7),
            "rock",
            graph,
            MaterialOutputTarget::RenderMaterial,
        );

        let result = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());

        assert!(result.report.has_blocking_issues());
        assert!(
            result
                .report
                .issues()
                .iter()
                .any(|issue| { issue.code() == &MaterialGraphIssueCode::MissingResourceReference })
        );
    }

    #[test]
    fn texture_resource_refs_are_carried_into_executable_ir() {
        let texture_ref =
            resource_ref::ResourceRef::new("asset.catalog", "texture.rock").expect("resource ref");
        let texture_node = NodeDefinition::new(
            NodeId::new(3),
            "texture.sample_2d",
            Vec::<PortDefinition>::new(),
        )
        .with_values([GraphMetadataEntry::new(
            crate::MATERIAL_GRAPH_VALUE_TEXTURE_REF,
            GraphValue::resource(texture_ref.clone()),
        )]);
        let graph = GraphDefinition::new(
            GraphId::new(1),
            "pbr",
            CyclePolicy::RejectDirectedCycles,
            [
                NodeDefinition::new(NodeId::new(2), "pbr.output", Vec::<PortDefinition>::new()),
                texture_node,
            ],
            [],
        );
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(7),
            "rock",
            graph,
            MaterialOutputTarget::RenderMaterial,
        );

        let result = lower_material_graph(&document, &MaterialNodeCatalog::first_slice());

        assert!(result.report.is_accepted());
        let ir = result.product.expect("formed").executable_ir.expect("ir");
        assert_eq!(ir.required_resources.len(), 1);
        assert_eq!(ir.required_resources[0].reference, texture_ref);
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
        changed_graph.nodes[0].ports[0].id = PortId::new(3);
        changed_graph.edges[0].from_port = PortId::new(3);
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

    #[test]
    fn cache_key_changes_when_node_catalog_version_changes() {
        let descriptors = MaterialNodeCatalog::first_slice()
            .descriptors()
            .cloned()
            .collect::<Vec<_>>();
        let catalog_v1 = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.first_slice",
            1,
            descriptors.clone(),
        )
        .expect("test catalog v1 should be valid");
        let catalog_v2 = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.first_slice",
            2,
            descriptors,
        )
        .expect("test catalog v2 should be valid");
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&document, &catalog_v1)
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&document, &catalog_v2)
            .product
            .expect("valid graph should form");

        assert_ne!(first.cache_key, second.cache_key);
    }

    #[test]
    fn cache_key_changes_when_catalog_descriptor_set_changes() {
        let base_catalog = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.custom",
            1,
            [
                first_slice_descriptor("pbr.base_color"),
                first_slice_descriptor("pbr.output"),
            ],
        )
        .expect("base test catalog should be valid");
        let expanded_catalog = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.custom",
            1,
            [
                first_slice_descriptor("pbr.base_color"),
                first_slice_descriptor("pbr.output"),
                first_slice_descriptor("proc.noise"),
            ],
        )
        .expect("expanded test catalog should be valid");
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&document, &base_catalog)
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&document, &expanded_catalog)
            .product
            .expect("valid graph should form");

        assert_ne!(first.cache_key, second.cache_key);
    }

    #[test]
    fn cache_key_ignores_catalog_descriptor_labels() {
        let base_catalog = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.custom",
            1,
            [
                first_slice_descriptor("pbr.base_color"),
                first_slice_descriptor("pbr.output"),
            ],
        )
        .expect("base test catalog should be valid");
        let mut relabeled_base_color = first_slice_descriptor("pbr.base_color");
        relabeled_base_color.label = "Albedo".to_string();
        let mut relabeled_output = first_slice_descriptor("pbr.output");
        relabeled_output.label = "Output".to_string();
        let relabeled_catalog = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.custom",
            1,
            [relabeled_base_color, relabeled_output],
        )
        .expect("relabeled test catalog should be valid");
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&document, &base_catalog)
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&document, &relabeled_catalog)
            .product
            .expect("valid graph should form");

        assert_eq!(first.cache_key, second.cache_key);
    }

    #[test]
    fn cache_key_changes_when_descriptor_semantic_version_changes() {
        let base_catalog = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.custom",
            1,
            [
                first_slice_descriptor("pbr.base_color"),
                first_slice_descriptor("pbr.output"),
            ],
        )
        .expect("base test catalog should be valid");
        let mut semantic_v2_base_color = first_slice_descriptor("pbr.base_color");
        semantic_v2_base_color.semantic_version = 2;
        let semantic_v2_catalog = MaterialNodeCatalog::new_with_identity(
            "material_node_catalog.custom",
            1,
            [semantic_v2_base_color, first_slice_descriptor("pbr.output")],
        )
        .expect("semantic-v2 test catalog should be valid");
        let document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "deterministic",
            pbr_graph(),
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&document, &base_catalog)
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&document, &semantic_v2_catalog)
            .product
            .expect("valid graph should form");

        assert_ne!(first.cache_key, second.cache_key);
    }

    #[test]
    fn cache_key_uses_structured_identity_for_delimiter_like_names() {
        fn graph_with_names(
            graph_name: &str,
            source_metadata_key: &str,
            source_metadata_value: &str,
        ) -> GraphDefinition {
            let color = PortTypeId::new(1);
            GraphDefinition::new(
                GraphId::new(1),
                graph_name,
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
                    )
                    .with_metadata([GraphMetadataEntry::new(
                        source_metadata_key,
                        GraphValue::text(source_metadata_value),
                    )]),
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

        let catalog = MaterialNodeCatalog::first_slice();
        let first_document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "name:a",
            graph_with_names("graph:a", "a:b", "c"),
            MaterialOutputTarget::PbrPreview,
        );
        let second_document = MaterialGraphDocument::new(
            crate::MaterialGraphDocumentId::new(9),
            "name",
            graph_with_names("graph", "a", "b:c"),
            MaterialOutputTarget::PbrPreview,
        );

        let first = lower_material_graph(&first_document, &catalog)
            .product
            .expect("valid graph should form");
        let second = lower_material_graph(&second_document, &catalog)
            .product
            .expect("valid graph should form");

        assert_ne!(first.cache_key, second.cache_key);
    }
}
