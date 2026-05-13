//! File: domain/procgen/src/document.rs
//! Purpose: Authored procgen document contract and deterministic descriptor vocabulary.

use std::collections::BTreeSet;

use graph::{GraphDefinition, NodeId};
use product::{ProductIdentity, ProductScope};
use spatial::{ChunkId, RegionId, WorldId};
use world_ops::{QuantizedAabb, WorldRevision};

use crate::{
    ProcgenDocumentId, ProcgenGeneratorId, ProcgenReservation,
    determinism::parameter_hash_for_document,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcgenNodeKind {
    HeightNoise,
    MaterialRule,
    WorldOpsOutput,
    FieldProductOutput,
    Diagnostic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenNodeParameters {
    pub node_id: NodeId,
    pub kind: ProcgenNodeKind,
    pub label: String,
    pub seed_salt: String,
    pub material_channel: Option<u16>,
    pub weight: i32,
}

impl ProcgenNodeParameters {
    pub fn new(node_id: NodeId, kind: ProcgenNodeKind, label: impl Into<String>) -> Self {
        Self {
            node_id,
            kind,
            label: label.into(),
            seed_salt: String::new(),
            material_channel: None,
            weight: 0,
        }
    }

    pub fn with_seed_salt(mut self, seed_salt: impl Into<String>) -> Self {
        self.seed_salt = seed_salt.into();
        self
    }

    pub fn with_material_channel(mut self, material_channel: u16) -> Self {
        self.material_channel = Some(material_channel);
        self
    }

    pub fn with_weight(mut self, weight: i32) -> Self {
        self.weight = weight;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcgenInputProduct {
    pub product_id: ProductIdentity,
    pub generation: u64,
}

impl ProcgenInputProduct {
    pub const fn new(product_id: ProductIdentity, generation: u64) -> Self {
        Self {
            product_id,
            generation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenScope {
    pub world_id: WorldId,
    pub chunk_ids: Vec<ChunkId>,
    pub region_ids: Vec<RegionId>,
}

impl ProcgenScope {
    pub fn new(
        world_id: WorldId,
        chunk_ids: impl IntoIterator<Item = ChunkId>,
        region_ids: impl IntoIterator<Item = RegionId>,
    ) -> Self {
        let chunk_ids = chunk_ids.into_iter().collect::<BTreeSet<_>>();
        let region_ids = region_ids.into_iter().collect::<BTreeSet<_>>();
        Self {
            world_id,
            chunk_ids: chunk_ids.into_iter().collect(),
            region_ids: region_ids.into_iter().collect(),
        }
    }

    pub fn is_bounded(&self) -> bool {
        !self.chunk_ids.is_empty() || !self.region_ids.is_empty()
    }

    pub fn all_ids_match_world(&self) -> bool {
        self.chunk_ids
            .iter()
            .all(|chunk| chunk.world_id == self.world_id)
            && self
                .region_ids
                .iter()
                .all(|region| region.world_id == self.world_id)
    }

    pub fn product_scope(&self) -> ProductScope {
        ProductScope::field(self.chunk_labels(), self.region_labels())
    }

    pub fn chunk_labels(&self) -> Vec<String> {
        self.chunk_ids
            .iter()
            .map(|chunk| {
                format!(
                    "world:{}:chunk:{}:{}:{}",
                    chunk.world_id.0, chunk.coord.x, chunk.coord.y, chunk.coord.z
                )
            })
            .collect()
    }

    pub fn region_labels(&self) -> Vec<String> {
        self.region_ids
            .iter()
            .map(|region| {
                format!(
                    "world:{}:region:{}:{}:{}",
                    region.world_id.0, region.coord.x, region.coord.y, region.coord.z
                )
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcgenWriteTargetKind {
    DensityField,
    MaterialChannel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenWriteTarget {
    pub target_id: String,
    pub kind: ProcgenWriteTargetKind,
    pub bounds_q: QuantizedAabb,
    pub material_channel: Option<u16>,
}

impl ProcgenWriteTarget {
    pub fn density(target_id: impl Into<String>, bounds_q: QuantizedAabb) -> Self {
        Self {
            target_id: target_id.into(),
            kind: ProcgenWriteTargetKind::DensityField,
            bounds_q,
            material_channel: None,
        }
    }

    pub fn material_channel(
        target_id: impl Into<String>,
        bounds_q: QuantizedAabb,
        material_channel: u16,
    ) -> Self {
        Self {
            target_id: target_id.into(),
            kind: ProcgenWriteTargetKind::MaterialChannel,
            bounds_q,
            material_channel: Some(material_channel),
        }
    }

    pub fn has_valid_bounds(&self) -> bool {
        self.bounds_q.min.x < self.bounds_q.max.x
            && self.bounds_q.min.y < self.bounds_q.max.y
            && self.bounds_q.min.z < self.bounds_q.max.z
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.bounds_q.min.x < other.bounds_q.max.x
            && self.bounds_q.max.x > other.bounds_q.min.x
            && self.bounds_q.min.y < other.bounds_q.max.y
            && self.bounds_q.max.y > other.bounds_q.min.y
            && self.bounds_q.min.z < other.bounds_q.max.z
            && self.bounds_q.max.z > other.bounds_q.min.z
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcgenOutputKind {
    WorldOpsWindow,
    FieldProductCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcgenOutputProduct {
    pub product_id: ProductIdentity,
    pub kind: ProcgenOutputKind,
    pub label: String,
}

impl ProcgenOutputProduct {
    pub fn new(
        product_id: ProductIdentity,
        kind: ProcgenOutputKind,
        label: impl Into<String>,
    ) -> Self {
        Self {
            product_id,
            kind,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenLoweringPolicy {
    pub lowering_version: String,
    pub fixed_point_scale: i32,
    pub base_world_revision: WorldRevision,
}

impl ProcgenLoweringPolicy {
    pub fn new(
        lowering_version: impl Into<String>,
        fixed_point_scale: i32,
        base_world_revision: WorldRevision,
    ) -> Self {
        Self {
            lowering_version: lowering_version.into(),
            fixed_point_scale,
            base_world_revision,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcgenBudgetClass {
    RuntimePreview,
    OfflineBake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProcgenRetentionClass {
    SessionCandidate,
    RetainedBake,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenExecutionPolicy {
    pub budget_class: ProcgenBudgetClass,
    pub retention_class: ProcgenRetentionClass,
}

impl ProcgenExecutionPolicy {
    pub const fn runtime_preview() -> Self {
        Self {
            budget_class: ProcgenBudgetClass::RuntimePreview,
            retention_class: ProcgenRetentionClass::SessionCandidate,
        }
    }

    pub const fn offline_bake() -> Self {
        Self {
            budget_class: ProcgenBudgetClass::OfflineBake,
            retention_class: ProcgenRetentionClass::RetainedBake,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenDiagnosticsPolicy {
    pub preserve_warnings: bool,
    pub reject_on_conflict: bool,
}

impl ProcgenDiagnosticsPolicy {
    pub const fn strict() -> Self {
        Self {
            preserve_warnings: true,
            reject_on_conflict: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenCacheLineage {
    pub parameter_hash: String,
    pub cache_inputs: Vec<String>,
}

impl ProcgenCacheLineage {
    pub fn empty() -> Self {
        Self {
            parameter_hash: String::new(),
            cache_inputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenDocument {
    pub document_id: ProcgenDocumentId,
    pub label: String,
    pub schema_version: String,
    pub generator_id: ProcgenGeneratorId,
    pub generator_version: String,
    pub world_seed: String,
    pub source_revision: String,
    pub graph: GraphDefinition,
    pub node_parameters: Vec<ProcgenNodeParameters>,
    pub scope: ProcgenScope,
    pub input_products: Vec<ProcgenInputProduct>,
    pub authored_overlay_generation: u64,
    pub write_targets: Vec<ProcgenWriteTarget>,
    pub output_products: Vec<ProcgenOutputProduct>,
    pub lowering_policy: ProcgenLoweringPolicy,
    pub execution_policy: ProcgenExecutionPolicy,
    pub diagnostics_policy: ProcgenDiagnosticsPolicy,
    pub cache_lineage: ProcgenCacheLineage,
    pub reservations: Vec<ProcgenReservation>,
}

impl ProcgenDocument {
    pub fn new(
        document_id: ProcgenDocumentId,
        label: impl Into<String>,
        graph: GraphDefinition,
        scope: ProcgenScope,
    ) -> Self {
        Self {
            document_id,
            label: label.into(),
            schema_version: String::new(),
            generator_id: ProcgenGeneratorId::default(),
            generator_version: String::new(),
            world_seed: String::new(),
            source_revision: String::new(),
            graph,
            node_parameters: Vec::new(),
            scope,
            input_products: Vec::new(),
            authored_overlay_generation: 0,
            write_targets: Vec::new(),
            output_products: Vec::new(),
            lowering_policy: ProcgenLoweringPolicy::new("", 1, WorldRevision::default()),
            execution_policy: ProcgenExecutionPolicy::runtime_preview(),
            diagnostics_policy: ProcgenDiagnosticsPolicy::strict(),
            cache_lineage: ProcgenCacheLineage::empty(),
            reservations: Vec::new(),
        }
    }

    pub fn with_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = schema_version.into();
        self
    }

    pub fn with_generator(
        mut self,
        generator_id: ProcgenGeneratorId,
        generator_version: impl Into<String>,
    ) -> Self {
        self.generator_id = generator_id;
        self.generator_version = generator_version.into();
        self
    }

    pub fn with_world_seed(mut self, world_seed: impl Into<String>) -> Self {
        self.world_seed = world_seed.into();
        self
    }

    pub fn with_source_revision(mut self, source_revision: impl Into<String>) -> Self {
        self.source_revision = source_revision.into();
        self
    }

    pub fn with_authored_overlay_generation(mut self, generation: u64) -> Self {
        self.authored_overlay_generation = generation;
        self
    }

    pub fn with_lowering_policy(mut self, lowering_policy: ProcgenLoweringPolicy) -> Self {
        self.lowering_policy = lowering_policy;
        self
    }

    pub fn with_execution_policy(mut self, execution_policy: ProcgenExecutionPolicy) -> Self {
        self.execution_policy = execution_policy;
        self
    }

    pub fn with_node_parameter(mut self, parameter: ProcgenNodeParameters) -> Self {
        self.node_parameters.push(parameter);
        self
    }

    pub fn with_input_product(mut self, input: ProcgenInputProduct) -> Self {
        self.input_products.push(input);
        self
    }

    pub fn with_write_target(mut self, target: ProcgenWriteTarget) -> Self {
        self.write_targets.push(target);
        self
    }

    pub fn with_output_product(mut self, output: ProcgenOutputProduct) -> Self {
        self.output_products.push(output);
        self
    }

    pub fn with_reservation(mut self, reservation: ProcgenReservation) -> Self {
        self.reservations.push(reservation);
        self
    }

    pub fn refresh_cache_lineage(&mut self) {
        let parameter_hash = parameter_hash_for_document(self);
        let mut cache_inputs = vec![
            format!("document:{}", self.document_id.raw()),
            format!("schema:{}", self.schema_version),
            format!("generator:{}", self.generator_id.raw()),
            format!("generator_version:{}", self.generator_version),
            format!("seed:{}", self.world_seed),
            format!("source_revision:{}", self.source_revision),
            format!("parameter_hash:{}", parameter_hash),
            format!("overlay_generation:{}", self.authored_overlay_generation),
            format!("lowering_version:{}", self.lowering_policy.lowering_version),
        ];
        cache_inputs.extend(
            self.input_products
                .iter()
                .map(|input| format!("input:{}:{}", input.product_id.raw(), input.generation)),
        );
        cache_inputs.sort();
        self.cache_lineage = ProcgenCacheLineage {
            parameter_hash,
            cache_inputs,
        };
    }

    pub fn with_refreshed_cache_lineage(mut self) -> Self {
        self.refresh_cache_lineage();
        self
    }
}
