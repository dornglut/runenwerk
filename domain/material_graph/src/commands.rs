//! File: domain/material_graph/src/commands.rs
//! Purpose: Source-backed material graph mutation contracts.

use crate::{
    MaterialGraphDocument, MaterialGraphNodeLayout, MaterialNodeCatalog, MaterialResourceKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialGraphCommandError {
    MissingNode,
    MissingEdge,
    MissingDescriptor,
    MissingResourceBinding,
    InvalidResourceRef,
    InvalidGraph,
}

impl MaterialGraphCommandError {
    pub const fn as_static_str(&self) -> &'static str {
        match self {
            Self::MissingNode => "material graph node is missing",
            Self::MissingEdge => "material graph edge is missing",
            Self::MissingDescriptor => "material node descriptor is not in the active catalog",
            Self::MissingResourceBinding => "material graph node resource binding is missing",
            Self::InvalidResourceRef => "material graph texture resource reference is invalid",
            Self::InvalidGraph => "material graph command would violate graph validation",
        }
    }
}

pub fn add_catalog_node(
    document: &mut MaterialGraphDocument,
    descriptor_key: &str,
    catalog: &MaterialNodeCatalog,
) -> Result<graph::NodeId, MaterialGraphCommandError> {
    let descriptor = catalog
        .descriptor(descriptor_key)
        .ok_or(MaterialGraphCommandError::MissingDescriptor)?;
    let node_id = next_node_id(document);
    let mut next_port = next_port_id(document).raw();
    let mut ports = Vec::new();
    for input in &descriptor.inputs {
        ports.push(graph::PortDefinition::new(
            graph::PortId::new(next_port),
            input.name.clone(),
            graph::PortDirection::Input,
            input.value_type.port_type_id(),
        ));
        next_port += 1;
    }
    for output in &descriptor.outputs {
        ports.push(graph::PortDefinition::new(
            graph::PortId::new(next_port),
            output.name.clone(),
            graph::PortDirection::Output,
            output.value_type.port_type_id(),
        ));
        next_port += 1;
    }
    let node = graph::NodeDefinition::new(node_id, descriptor.key.clone(), ports);
    document.graph.nodes.push(node);
    document
        .editor_state
        .node_layouts
        .push(MaterialGraphNodeLayout::new(
            node_id,
            (document.graph.nodes.len() as i32 % 4) * 220,
            (document.graph.nodes.len() as i32 / 4) * 120,
        ));
    Ok(node_id)
}

pub fn move_node_layout(
    document: &mut MaterialGraphDocument,
    node_id: graph::NodeId,
    delta_x: i32,
    delta_y: i32,
) -> Result<bool, MaterialGraphCommandError> {
    if !document.graph.nodes.iter().any(|node| node.id == node_id) {
        return Err(MaterialGraphCommandError::MissingNode);
    }
    if delta_x == 0 && delta_y == 0 {
        return Ok(false);
    }

    if let Some(layout) = document
        .editor_state
        .node_layouts
        .iter_mut()
        .find(|layout| layout.node_id == node_id)
    {
        layout.position_x = layout.position_x.saturating_add(delta_x);
        layout.position_y = layout.position_y.saturating_add(delta_y);
        return Ok(true);
    }

    let (base_x, base_y) = fallback_node_position(document, node_id);
    document
        .editor_state
        .node_layouts
        .push(MaterialGraphNodeLayout::new(
            node_id,
            base_x.saturating_add(delta_x),
            base_y.saturating_add(delta_y),
        ));
    Ok(true)
}

pub fn delete_selection(
    document: &mut MaterialGraphDocument,
    node_ids: &[graph::NodeId],
    edge_ids: &[graph::EdgeId],
) -> bool {
    if node_ids.is_empty() && edge_ids.is_empty() {
        return false;
    }

    let node_ids = node_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let edge_ids = edge_ids
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let deleted_ports = document
        .graph
        .nodes
        .iter()
        .filter(|node| node_ids.contains(&node.id))
        .flat_map(|node| node.ports.iter().map(|port| port.id))
        .collect::<std::collections::BTreeSet<_>>();

    let before_nodes = document.graph.nodes.len();
    let before_edges = document.graph.edges.len();
    let before_layouts = document.editor_state.node_layouts.len();

    document
        .graph
        .nodes
        .retain(|node| !node_ids.contains(&node.id));
    document.graph.edges.retain(|edge| {
        !edge_ids.contains(&edge.id)
            && !deleted_ports.contains(&edge.from_port)
            && !deleted_ports.contains(&edge.to_port)
    });
    document
        .editor_state
        .node_layouts
        .retain(|layout| !node_ids.contains(&layout.node_id));

    before_nodes != document.graph.nodes.len()
        || before_edges != document.graph.edges.len()
        || before_layouts != document.editor_state.node_layouts.len()
}

pub fn connect_ports(
    document: &mut MaterialGraphDocument,
    from_port_id: graph::PortId,
    to_port_id: graph::PortId,
) -> Result<graph::EdgeId, MaterialGraphCommandError> {
    let edge_id = next_edge_id(document);
    document.graph.edges.push(graph::EdgeDefinition::new(
        edge_id,
        from_port_id,
        to_port_id,
    ));
    if graph::validate_graph(&document.graph).is_err() {
        document.graph.edges.retain(|edge| edge.id != edge_id);
        return Err(MaterialGraphCommandError::InvalidGraph);
    }
    Ok(edge_id)
}

pub fn disconnect_edge(
    document: &mut MaterialGraphDocument,
    edge_id: graph::EdgeId,
) -> Result<bool, MaterialGraphCommandError> {
    let before = document.graph.edges.len();
    document.graph.edges.retain(|edge| edge.id != edge_id);
    if before == document.graph.edges.len() {
        return Err(MaterialGraphCommandError::MissingEdge);
    }
    Ok(true)
}

pub fn set_node_text_value(
    document: &mut MaterialGraphDocument,
    node_id: graph::NodeId,
    key: &str,
    value: String,
) -> Result<bool, MaterialGraphCommandError> {
    let node = document
        .graph
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or(MaterialGraphCommandError::MissingNode)?;
    set_node_graph_value(node, key, graph::GraphValue::Text(value));
    Ok(true)
}

pub fn set_node_texture_resource(
    document: &mut MaterialGraphDocument,
    node_id: graph::NodeId,
    key: &str,
    stable_id: &str,
    catalog: &MaterialNodeCatalog,
) -> Result<bool, MaterialGraphCommandError> {
    let node = document
        .graph
        .nodes
        .iter_mut()
        .find(|node| node.id == node_id)
        .ok_or(MaterialGraphCommandError::MissingNode)?;
    let descriptor = catalog
        .descriptor(&node.name)
        .ok_or(MaterialGraphCommandError::MissingDescriptor)?;
    let resource_kind = descriptor
        .resources
        .iter()
        .find(|resource| resource.key == key)
        .map(|resource| resource.kind)
        .ok_or(MaterialGraphCommandError::MissingResourceBinding)?;
    let kind = match resource_kind {
        MaterialResourceKind::Texture2D => "asset.catalog.texture2d",
        MaterialResourceKind::Texture3D => "asset.catalog.texture3d",
    };
    let reference = resource_ref::ResourceRef::new(kind, stable_id)
        .map_err(|_| MaterialGraphCommandError::InvalidResourceRef)?;
    set_node_graph_value(node, key, graph::GraphValue::Resource(reference));
    Ok(true)
}

fn set_node_graph_value(node: &mut graph::NodeDefinition, key: &str, value: graph::GraphValue) {
    if let Some(entry) = node.values.iter_mut().find(|entry| entry.key == key) {
        entry.value = value;
    } else {
        node.values
            .push(graph::GraphMetadataEntry::new(key.to_string(), value));
    }
}

fn fallback_node_position(document: &MaterialGraphDocument, node_id: graph::NodeId) -> (i32, i32) {
    let index = document
        .graph
        .nodes
        .iter()
        .position(|node| node.id == node_id)
        .unwrap_or_default();
    ((index as i32 % 4) * 220, (index as i32 / 4) * 120)
}

fn next_node_id(document: &MaterialGraphDocument) -> graph::NodeId {
    graph::NodeId::new(
        document
            .graph
            .nodes
            .iter()
            .map(|node| node.id.raw())
            .max()
            .unwrap_or(0)
            + 1,
    )
}

fn next_port_id(document: &MaterialGraphDocument) -> graph::PortId {
    graph::PortId::new(
        document
            .graph
            .nodes
            .iter()
            .flat_map(|node| node.ports.iter().map(|port| port.id.raw()))
            .max()
            .unwrap_or(0)
            + 1,
    )
}

fn next_edge_id(document: &MaterialGraphDocument) -> graph::EdgeId {
    graph::EdgeId::new(
        document
            .graph
            .edges
            .iter()
            .map(|edge| edge.id.raw())
            .max()
            .unwrap_or(0)
            + 1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MaterialGraphDocumentId, MaterialOutputTarget};

    fn command_document() -> MaterialGraphDocument {
        let float = graph::PortTypeId::new(1);
        let color = graph::PortTypeId::new(2);
        MaterialGraphDocument::new(
            MaterialGraphDocumentId::new(9),
            "commands",
            graph::GraphDefinition::new(
                graph::GraphId::new(1),
                "commands",
                graph::CyclePolicy::RejectDirectedCycles,
                [
                    graph::NodeDefinition::new(
                        graph::NodeId::new(1),
                        "source",
                        [
                            graph::PortDefinition::new(
                                graph::PortId::new(1),
                                "out",
                                graph::PortDirection::Output,
                                float,
                            ),
                            graph::PortDefinition::new(
                                graph::PortId::new(2),
                                "color",
                                graph::PortDirection::Output,
                                color,
                            ),
                        ],
                    ),
                    graph::NodeDefinition::new(
                        graph::NodeId::new(2),
                        "sink",
                        [graph::PortDefinition::new(
                            graph::PortId::new(3),
                            "in",
                            graph::PortDirection::Input,
                            float,
                        )],
                    ),
                ],
                [],
            ),
            MaterialOutputTarget::RenderMaterial,
        )
    }

    #[test]
    fn material_graph_commands_validate_port_compatibility() {
        let mut document = command_document();

        let result = connect_ports(&mut document, graph::PortId::new(2), graph::PortId::new(3));

        assert_eq!(result, Err(MaterialGraphCommandError::InvalidGraph));
        assert!(document.graph.edges.is_empty());
    }

    #[test]
    fn material_graph_node_move_uses_source_owned_layout() {
        let mut document = command_document();

        move_node_layout(&mut document, graph::NodeId::new(1), 12, -4)
            .expect("node move should update source layout");

        assert_eq!(
            document.editor_state.node_layouts,
            vec![MaterialGraphNodeLayout::new(graph::NodeId::new(1), 12, -4)]
        );
    }
}
