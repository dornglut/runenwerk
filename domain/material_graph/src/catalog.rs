//! File: domain/material_graph/src/catalog.rs
//! Purpose: Material graph node catalog boundaries.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialNodeDescriptor {
    pub key: String,
    pub label: String,
}

impl MaterialNodeDescriptor {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialNodeCatalog {
    nodes_by_key: BTreeMap<String, MaterialNodeDescriptor>,
}

impl MaterialNodeCatalog {
    pub fn new(nodes: impl IntoIterator<Item = MaterialNodeDescriptor>) -> Self {
        Self {
            nodes_by_key: nodes
                .into_iter()
                .map(|node| (node.key.clone(), node))
                .collect(),
        }
    }

    pub fn first_slice() -> Self {
        Self::new([
            MaterialNodeDescriptor::new("pbr.output", "PBR Output"),
            MaterialNodeDescriptor::new("pbr.base_color", "Base Color"),
            MaterialNodeDescriptor::new("pbr.roughness", "Roughness"),
            MaterialNodeDescriptor::new("pbr.metallic", "Metallic"),
            MaterialNodeDescriptor::new("pbr.normal_strength", "Normal Strength"),
            MaterialNodeDescriptor::new("pbr.emissive", "Emissive"),
            MaterialNodeDescriptor::new("pbr.opacity", "Opacity"),
            MaterialNodeDescriptor::new("pbr.material_channel", "Material Channel"),
            MaterialNodeDescriptor::new("sdf.position", "SDF Position"),
            MaterialNodeDescriptor::new("sdf.normal", "SDF Normal"),
            MaterialNodeDescriptor::new("sdf.distance", "SDF Distance"),
            MaterialNodeDescriptor::new("sdf.material_channel", "SDF Material Channel"),
            MaterialNodeDescriptor::new("sdf.density", "SDF Density"),
            MaterialNodeDescriptor::new("sdf.support", "SDF Support"),
            MaterialNodeDescriptor::new("sdf.wetness", "SDF Wetness"),
            MaterialNodeDescriptor::new("proc.noise", "Noise"),
            MaterialNodeDescriptor::new("proc.fbm", "FBM"),
            MaterialNodeDescriptor::new("proc.ramp", "Ramp"),
            MaterialNodeDescriptor::new("math.remap", "Remap"),
            MaterialNodeDescriptor::new("math.clamp", "Clamp"),
            MaterialNodeDescriptor::new("math.mix", "Mix"),
            MaterialNodeDescriptor::new("math.mask", "Mask"),
            MaterialNodeDescriptor::new("texture.sample_2d", "Texture2D Sample"),
            MaterialNodeDescriptor::new("texture.sample_3d", "Texture3D Sample"),
            MaterialNodeDescriptor::new("coord.triplanar", "Triplanar Coordinates"),
        ])
    }

    pub fn contains(&self, key: &str) -> bool {
        self.nodes_by_key.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.nodes_by_key.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes_by_key.is_empty()
    }

    pub fn descriptors(&self) -> impl Iterator<Item = &MaterialNodeDescriptor> {
        self.nodes_by_key.values()
    }
}
