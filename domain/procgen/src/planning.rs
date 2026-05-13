//! File: domain/procgen/src/planning.rs
//! Purpose: Procgen-owned planning lifecycle DTOs and realization evidence.

use product::ProductIdentity;
use world_ops::{OperationRecord, QuantizedAabb};

use crate::{
    ProcgenCandidateId, ProcgenDocumentId, ProcgenRealizationId, ProcgenReservationId,
    ProcgenWriteTarget, ProcgenWriteTargetKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenPrototype {
    pub document_id: ProcgenDocumentId,
    pub label: String,
}

impl ProcgenPrototype {
    pub fn new(document_id: ProcgenDocumentId, label: impl Into<String>) -> Self {
        Self {
            document_id,
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenReservation {
    pub reservation_id: ProcgenReservationId,
    pub target_id: String,
    pub kind: ProcgenWriteTargetKind,
    pub bounds_q: QuantizedAabb,
    pub material_channel: Option<u16>,
}

impl ProcgenReservation {
    pub fn from_target(reservation_id: ProcgenReservationId, target: &ProcgenWriteTarget) -> Self {
        Self {
            reservation_id,
            target_id: target.target_id.clone(),
            kind: target.kind,
            bounds_q: target.bounds_q,
            material_channel: target.material_channel,
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenChangedRegion {
    pub target_id: String,
    pub bounds_q: QuantizedAabb,
    pub product_id: Option<ProductIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenExplanationEntry {
    pub subject: String,
    pub message: String,
}

impl ProcgenExplanationEntry {
    pub fn new(subject: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenInstancePlan {
    pub candidate_id: ProcgenCandidateId,
    pub reservations: Vec<ProcgenReservation>,
    pub explanations: Vec<ProcgenExplanationEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcgenRealization {
    pub realization_id: ProcgenRealizationId,
    pub candidate_id: ProcgenCandidateId,
    pub operation_records: Vec<OperationRecord>,
    pub changed_regions: Vec<ProcgenChangedRegion>,
    pub explanations: Vec<ProcgenExplanationEntry>,
    pub determinism_key: String,
}
