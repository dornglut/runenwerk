//! File: domain/procgen/src/determinism.rs
//! Purpose: Stable procgen determinism keys and hash-derived identities.

use crate::{ProcgenDocument, ProcgenWriteTargetKind};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcgenDeterminismKey(pub String);

impl ProcgenDeterminismKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

pub fn stable_nonzero_hash64(parts: impl IntoIterator<Item = impl AsRef<str>>) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET;
    for part in parts {
        for byte in part.as_ref().as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    if hash == 0 { 1 } else { hash }
}

pub fn parameter_hash_for_document(document: &ProcgenDocument) -> String {
    let mut parts = Vec::new();
    for parameter in &document.node_parameters {
        parts.push(format!(
            "node:{}:{:?}:{}:{}:{:?}:{}",
            parameter.node_id.raw(),
            parameter.kind,
            parameter.label,
            parameter.seed_salt,
            parameter.material_channel,
            parameter.weight
        ));
    }
    for target in &document.write_targets {
        parts.push(format!(
            "target:{}:{:?}:{}:{}:{}:{}:{}:{}:{:?}",
            target.target_id,
            target.kind,
            target.bounds_q.min.x,
            target.bounds_q.min.y,
            target.bounds_q.min.z,
            target.bounds_q.max.x,
            target.bounds_q.max.y,
            target.bounds_q.max.z,
            target.material_channel
        ));
    }
    for output in &document.output_products {
        parts.push(format!(
            "output:{}:{:?}:{}",
            output.product_id.raw(),
            output.kind,
            output.label
        ));
    }
    parts.sort();
    format!("{:016x}", stable_nonzero_hash64(parts))
}

pub fn determinism_key_for_document(document: &ProcgenDocument) -> ProcgenDeterminismKey {
    let mut parts = vec![
        format!("document:{}", document.document_id.raw()),
        format!("schema:{}", document.schema_version),
        format!("generator:{}", document.generator_id.raw()),
        format!("generator_version:{}", document.generator_version),
        format!("seed:{}", document.world_seed),
        format!("source_revision:{}", document.source_revision),
        format!("graph:{}", document.graph.id.raw()),
        format!("parameter_hash:{}", parameter_hash_for_document(document)),
        format!(
            "overlay_generation:{}",
            document.authored_overlay_generation
        ),
        format!(
            "lowering_version:{}",
            document.lowering_policy.lowering_version
        ),
    ];
    parts.extend(
        document
            .scope
            .chunk_labels()
            .into_iter()
            .map(|label| format!("scope:{}", label)),
    );
    parts.extend(
        document
            .scope
            .region_labels()
            .into_iter()
            .map(|label| format!("scope:{}", label)),
    );
    parts.extend(
        document
            .input_products
            .iter()
            .map(|input| format!("input:{}:{}", input.product_id.raw(), input.generation)),
    );
    parts.sort();
    ProcgenDeterminismKey::new(format!("{:016x}", stable_nonzero_hash64(parts)))
}

pub(crate) fn write_target_kind_name(kind: ProcgenWriteTargetKind) -> &'static str {
    match kind {
        ProcgenWriteTargetKind::DensityField => "density",
        ProcgenWriteTargetKind::MaterialChannel => "material",
    }
}
