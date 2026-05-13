//! File: domain/procgen/src/catalog.rs
//! Purpose: Procgen-owned semantic node catalog for the first terrain/material slice.

use std::collections::BTreeMap;

use crate::ProcgenNodeKind;

pub const HEIGHT_NOISE_NODE: &str = "procgen.height_noise";
pub const MATERIAL_RULE_NODE: &str = "procgen.material_rule";
pub const WORLD_OPS_OUTPUT_NODE: &str = "procgen.output.world_ops";
pub const FIELD_PRODUCT_OUTPUT_NODE: &str = "procgen.output.field_product";
pub const DIAGNOSTIC_NODE: &str = "procgen.diagnostic";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenNodeDescriptor {
    pub name: String,
    pub kind: ProcgenNodeKind,
    pub lowers_to_world_ops: bool,
}

impl ProcgenNodeDescriptor {
    pub fn new(name: impl Into<String>, kind: ProcgenNodeKind, lowers_to_world_ops: bool) -> Self {
        Self {
            name: name.into(),
            kind,
            lowers_to_world_ops,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenNodeCatalog {
    nodes: BTreeMap<String, ProcgenNodeDescriptor>,
}

impl ProcgenNodeCatalog {
    pub fn first_slice() -> Self {
        Self::from_descriptors([
            ProcgenNodeDescriptor::new(HEIGHT_NOISE_NODE, ProcgenNodeKind::HeightNoise, true),
            ProcgenNodeDescriptor::new(MATERIAL_RULE_NODE, ProcgenNodeKind::MaterialRule, true),
            ProcgenNodeDescriptor::new(
                WORLD_OPS_OUTPUT_NODE,
                ProcgenNodeKind::WorldOpsOutput,
                true,
            ),
            ProcgenNodeDescriptor::new(
                FIELD_PRODUCT_OUTPUT_NODE,
                ProcgenNodeKind::FieldProductOutput,
                false,
            ),
            ProcgenNodeDescriptor::new(DIAGNOSTIC_NODE, ProcgenNodeKind::Diagnostic, false),
        ])
    }

    pub fn from_descriptors(descriptors: impl IntoIterator<Item = ProcgenNodeDescriptor>) -> Self {
        Self {
            nodes: descriptors
                .into_iter()
                .map(|descriptor| (descriptor.name.clone(), descriptor))
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&ProcgenNodeDescriptor> {
        self.nodes.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }
}
