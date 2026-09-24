//! File: domain/material_graph/src/ir.rs
//! Purpose: Executable material graph IR shared with render backends.

use graph::{NodeId, PortId};
use resource_ref::ResourceRef;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{MaterialGraphDocumentId, MaterialLiteral, MaterialOutputTarget, MaterialValueType};

pub const MATERIAL_IR_CONTRACT_VERSION: u32 = 1;
pub const MATERIAL_GRAPH_VALUE_TEXTURE_REF: &str = "texture_ref";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialIr {
    pub contract_version: u32,
    pub document_id: MaterialGraphDocumentId,
    pub output_target: MaterialOutputTarget,
    pub nodes: Vec<MaterialIrNode>,
    pub edges: Vec<MaterialIrEdge>,
    pub required_resources: Vec<MaterialResourceBinding>,
}

impl MaterialIr {
    pub fn new(
        document_id: MaterialGraphDocumentId,
        output_target: MaterialOutputTarget,
        nodes: impl IntoIterator<Item = MaterialIrNode>,
        edges: impl IntoIterator<Item = MaterialIrEdge>,
        required_resources: impl IntoIterator<Item = MaterialResourceBinding>,
    ) -> Self {
        let mut nodes = nodes.into_iter().collect::<Vec<_>>();
        nodes = topologically_order_nodes(nodes);
        let mut edges = edges.into_iter().collect::<Vec<_>>();
        edges.sort_by_key(|edge| edge.edge_id);
        let mut required_resources = required_resources.into_iter().collect::<Vec<_>>();
        required_resources.sort_by(|left, right| {
            left.node_id
                .raw()
                .cmp(&right.node_id.raw())
                .then(left.binding_key.cmp(&right.binding_key))
        });
        Self {
            contract_version: MATERIAL_IR_CONTRACT_VERSION,
            document_id,
            output_target,
            nodes,
            edges,
            required_resources,
        }
    }
}

fn topologically_order_nodes(nodes: Vec<MaterialIrNode>) -> Vec<MaterialIrNode> {
    let mut by_id = nodes
        .into_iter()
        .map(|node| (node.node_id, node))
        .collect::<BTreeMap<_, _>>();
    let mut dependencies = BTreeMap::<NodeId, BTreeSet<NodeId>>::new();
    let mut outgoing = BTreeMap::<NodeId, BTreeSet<NodeId>>::new();

    for node_id in by_id.keys().copied() {
        dependencies.insert(node_id, BTreeSet::new());
        outgoing.insert(node_id, BTreeSet::new());
    }

    for node in by_id.values() {
        for input in &node.inputs {
            if let MaterialIrInputSource::Connected { node_id, .. } = input.source
                && by_id.contains_key(&node_id)
            {
                dependencies
                    .entry(node.node_id)
                    .or_default()
                    .insert(node_id);
                outgoing.entry(node_id).or_default().insert(node.node_id);
            }
        }
    }

    let mut ready = dependencies
        .iter()
        .filter(|(_, deps)| deps.is_empty())
        .map(|(node_id, _)| *node_id)
        .collect::<Vec<_>>();
    ready.sort_by_key(|node_id| node_id.raw());
    let mut queue = VecDeque::from(ready);
    let mut ordered = Vec::with_capacity(by_id.len());

    while let Some(node_id) = queue.pop_front() {
        if let Some(node) = by_id.remove(&node_id) {
            ordered.push(node);
        }
        let Some(consumers) = outgoing.get(&node_id) else {
            continue;
        };
        for consumer in consumers {
            let Some(deps) = dependencies.get_mut(consumer) else {
                continue;
            };
            deps.remove(&node_id);
            if deps.is_empty() && by_id.contains_key(consumer) && !queue.contains(consumer) {
                queue.push_back(*consumer);
                let mut staged = queue.drain(..).collect::<Vec<_>>();
                staged.sort_by_key(|node_id| node_id.raw());
                queue = VecDeque::from(staged);
            }
        }
    }

    if !by_id.is_empty() {
        let mut remaining = by_id.into_values().collect::<Vec<_>>();
        remaining.sort_by_key(|node| node.node_id.raw());
        ordered.extend(remaining);
    }

    ordered
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialIrNode {
    pub node_id: NodeId,
    pub op: MaterialNodeOp,
    pub inputs: Vec<MaterialIrInput>,
    pub outputs: Vec<MaterialIrOutput>,
    pub values: Vec<MaterialIrValue>,
}

impl MaterialIrNode {
    pub fn new(
        node_id: NodeId,
        op: MaterialNodeOp,
        inputs: impl IntoIterator<Item = MaterialIrInput>,
        outputs: impl IntoIterator<Item = MaterialIrOutput>,
        values: impl IntoIterator<Item = MaterialIrValue>,
    ) -> Self {
        let mut inputs = inputs.into_iter().collect::<Vec<_>>();
        inputs.sort_by(|left, right| left.name.cmp(&right.name));
        let mut outputs = outputs.into_iter().collect::<Vec<_>>();
        outputs.sort_by(|left, right| left.name.cmp(&right.name));
        let mut values = values.into_iter().collect::<Vec<_>>();
        values.sort_by(|left, right| left.key.cmp(&right.key));
        Self {
            node_id,
            op,
            inputs,
            outputs,
            values,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialIrInput {
    pub name: String,
    pub value_type: MaterialValueType,
    pub source: MaterialIrInputSource,
}

impl MaterialIrInput {
    pub fn new(
        name: impl Into<String>,
        value_type: MaterialValueType,
        source: MaterialIrInputSource,
    ) -> Self {
        Self {
            name: name.into(),
            value_type,
            source,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MaterialIrInputSource {
    Connected {
        node_id: NodeId,
        output_name: String,
    },
    Constant(MaterialLiteral),
    NodeValue {
        key: String,
        canonical_value: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialIrOutput {
    pub name: String,
    pub value_type: MaterialValueType,
}

impl MaterialIrOutput {
    pub fn new(name: impl Into<String>, value_type: MaterialValueType) -> Self {
        Self {
            name: name.into(),
            value_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MaterialNodeOp {
    PbrOutput,
    PbrBaseColor,
    PbrRoughness,
    PbrMetallic,
    PbrNormalStrength,
    PbrEmissive,
    PbrOpacity,
    PbrMaterialChannel,
    SdfPosition,
    SdfNormal,
    SdfDistance,
    SdfMaterialChannel,
    SdfDensity,
    SdfSupport,
    SdfWetness,
    ProcNoise,
    ProcFbm,
    ProcRamp,
    MathRemap,
    MathClamp,
    MathMix,
    MathMask,
    TextureSample2D,
    TextureSample3D,
    CoordTriplanar,
}

impl MaterialNodeOp {
    pub fn from_catalog_key(key: &str) -> Option<Self> {
        Some(match key {
            "pbr.output" => Self::PbrOutput,
            "pbr.base_color" => Self::PbrBaseColor,
            "pbr.roughness" => Self::PbrRoughness,
            "pbr.metallic" => Self::PbrMetallic,
            "pbr.normal_strength" => Self::PbrNormalStrength,
            "pbr.emissive" => Self::PbrEmissive,
            "pbr.opacity" => Self::PbrOpacity,
            "pbr.material_channel" => Self::PbrMaterialChannel,
            "sdf.position" => Self::SdfPosition,
            "sdf.normal" => Self::SdfNormal,
            "sdf.distance" => Self::SdfDistance,
            "sdf.material_channel" => Self::SdfMaterialChannel,
            "sdf.density" => Self::SdfDensity,
            "sdf.support" => Self::SdfSupport,
            "sdf.wetness" => Self::SdfWetness,
            "proc.noise" => Self::ProcNoise,
            "proc.fbm" => Self::ProcFbm,
            "proc.ramp" => Self::ProcRamp,
            "math.remap" => Self::MathRemap,
            "math.clamp" => Self::MathClamp,
            "math.mix" => Self::MathMix,
            "math.mask" => Self::MathMask,
            "texture.sample_2d" => Self::TextureSample2D,
            "texture.sample_3d" => Self::TextureSample3D,
            "coord.triplanar" => Self::CoordTriplanar,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::PbrOutput => "pbr.output",
            Self::PbrBaseColor => "pbr.base_color",
            Self::PbrRoughness => "pbr.roughness",
            Self::PbrMetallic => "pbr.metallic",
            Self::PbrNormalStrength => "pbr.normal_strength",
            Self::PbrEmissive => "pbr.emissive",
            Self::PbrOpacity => "pbr.opacity",
            Self::PbrMaterialChannel => "pbr.material_channel",
            Self::SdfPosition => "sdf.position",
            Self::SdfNormal => "sdf.normal",
            Self::SdfDistance => "sdf.distance",
            Self::SdfMaterialChannel => "sdf.material_channel",
            Self::SdfDensity => "sdf.density",
            Self::SdfSupport => "sdf.support",
            Self::SdfWetness => "sdf.wetness",
            Self::ProcNoise => "proc.noise",
            Self::ProcFbm => "proc.fbm",
            Self::ProcRamp => "proc.ramp",
            Self::MathRemap => "math.remap",
            Self::MathClamp => "math.clamp",
            Self::MathMix => "math.mix",
            Self::MathMask => "math.mask",
            Self::TextureSample2D => "texture.sample_2d",
            Self::TextureSample3D => "texture.sample_3d",
            Self::CoordTriplanar => "coord.triplanar",
        }
    }

    pub const fn requires_texture_resource(self) -> bool {
        matches!(self, Self::TextureSample2D | Self::TextureSample3D)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialIrValue {
    pub key: String,
    pub canonical_value: String,
}

impl MaterialIrValue {
    pub fn new(key: impl Into<String>, canonical_value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            canonical_value: canonical_value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialIrEdge {
    pub edge_id: u64,
    pub from_port: PortId,
    pub to_port: PortId,
}

impl MaterialIrEdge {
    pub fn new(edge_id: u64, from_port: PortId, to_port: PortId) -> Self {
        Self {
            edge_id,
            from_port,
            to_port,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialResourceBinding {
    pub node_id: NodeId,
    pub binding_key: String,
    pub reference: ResourceRef,
}

impl MaterialResourceBinding {
    pub fn new(node_id: NodeId, binding_key: impl Into<String>, reference: ResourceRef) -> Self {
        Self {
            node_id,
            binding_key: binding_key.into(),
            reference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_ir_orders_producers_before_consumers_independent_of_source_order() {
        let consumer = MaterialIrNode::new(
            NodeId::new(10),
            MaterialNodeOp::PbrOutput,
            [MaterialIrInput::new(
                "base_color",
                MaterialValueType::Color,
                MaterialIrInputSource::Connected {
                    node_id: NodeId::new(20),
                    output_name: "color".to_string(),
                },
            )],
            [],
            [],
        );
        let producer = MaterialIrNode::new(
            NodeId::new(20),
            MaterialNodeOp::PbrBaseColor,
            [],
            [MaterialIrOutput::new("color", MaterialValueType::Color)],
            [MaterialIrValue::new("color", "color:1:0:0:1")],
        );

        let ir = MaterialIr::new(
            MaterialGraphDocumentId::new(1),
            MaterialOutputTarget::RenderMaterial,
            [consumer, producer],
            [],
            [],
        );

        assert_eq!(ir.nodes[0].node_id, NodeId::new(20));
        assert_eq!(ir.nodes[1].node_id, NodeId::new(10));
    }
}
