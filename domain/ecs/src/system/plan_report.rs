use scheduler::access::{AccessDomain, ConflictKind};
use scheduler::plan::{
    BarrierKind, ExecutionBarrier, ExecutionPhaseKind, ExecutionPlan, ExecutionPlanDiagnosticKind,
};
use scheduler::system::{RegisteredSystem, SystemId};

use super::runtime::{ParamSlotId, ParamSlotMetadata};
use crate::World;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanReport {
    pub schedule_label: &'static str,
    pub phase: RuntimePlanPhaseReport,
    pub stages: Vec<RuntimePlanStageReport>,
    pub waves: Vec<RuntimePlanWaveReport>,
    pub barriers: Vec<RuntimePlanBarrierReport>,
    pub conflicts: Vec<RuntimePlanConflictReport>,
    pub diagnostics: Vec<RuntimePlanDiagnosticReport>,
}

impl RuntimePlanReport {
    pub(crate) fn from_plan(plan: &ExecutionPlan, systems: &[RegisteredSystem<World>]) -> Self {
        let barriers = plan
            .barriers
            .iter()
            .map(RuntimePlanBarrierReport::from_barrier)
            .collect::<Vec<_>>();

        Self {
            schedule_label: plan.label.name(),
            phase: RuntimePlanPhaseReport {
                index: plan.phase.index,
                label: plan.phase.label.name(),
                kind: plan.phase.kind.clone(),
            },
            stages: plan
                .stages
                .iter()
                .map(|stage| RuntimePlanStageReport {
                    index: stage.index,
                    systems: system_reports_for_indices(systems, &stage.system_indices),
                    missing_system_indices: missing_system_indices(systems, &stage.system_indices),
                })
                .collect(),
            waves: plan
                .waves
                .iter()
                .map(|wave| RuntimePlanWaveReport {
                    index: wave.index,
                    phase_index: wave.phase_index,
                    stage_index: wave.stage_index,
                    system_indices: wave.system_indices.clone(),
                    system_ids: wave.system_ids.clone(),
                    barriers: barriers
                        .iter()
                        .filter(|barrier| barrier.after_wave_index == Some(wave.index))
                        .cloned()
                        .collect(),
                    missing_system_indices: missing_system_indices(systems, &wave.system_indices),
                })
                .collect(),
            barriers,
            conflicts: plan
                .conflicts
                .iter()
                .map(|conflict| RuntimePlanConflictReport {
                    first_system_id: conflict.first_system_id,
                    first_system: conflict.first_system.clone(),
                    second_system_id: conflict.second_system_id,
                    second_system: conflict.second_system.clone(),
                    access_domain: conflict.conflict.key.domain(),
                    access_name: conflict.conflict.key.name(),
                    conflict_kind: conflict.conflict.kind,
                    message: conflict.conflict.diagnostic_message(),
                })
                .collect(),
            diagnostics: plan
                .diagnostics
                .iter()
                .map(|diagnostic| RuntimePlanDiagnosticReport {
                    kind: diagnostic.kind.clone(),
                    message: diagnostic.message.clone(),
                })
                .collect(),
        }
    }

    pub fn product_publication_barriers(&self) -> impl Iterator<Item = &RuntimePlanBarrierReport> {
        self.barriers
            .iter()
            .filter(|barrier| barrier.kind == BarrierKind::ProductPublication)
    }

    pub fn query_snapshot_publication_barriers(
        &self,
    ) -> impl Iterator<Item = &RuntimePlanBarrierReport> {
        self.barriers
            .iter()
            .filter(|barrier| barrier.kind == BarrierKind::QuerySnapshotPublication)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanPhaseReport {
    pub index: usize,
    pub label: &'static str,
    pub kind: ExecutionPhaseKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanStageReport {
    pub index: usize,
    pub systems: Vec<RuntimePlanSystemReport>,
    pub missing_system_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanWaveReport {
    pub index: usize,
    pub phase_index: usize,
    pub stage_index: usize,
    pub system_indices: Vec<usize>,
    pub system_ids: Vec<SystemId>,
    pub barriers: Vec<RuntimePlanBarrierReport>,
    pub missing_system_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanSystemReport {
    pub system_index: usize,
    pub system_id: SystemId,
    pub name: String,
    pub param_slots: Vec<ParamSlotMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanBarrierReport {
    pub index: usize,
    pub phase_index: usize,
    pub after_wave_index: Option<usize>,
    pub kind: BarrierKind,
}

impl RuntimePlanBarrierReport {
    fn from_barrier(barrier: &ExecutionBarrier) -> Self {
        Self {
            index: barrier.index,
            phase_index: barrier.phase_index,
            after_wave_index: barrier.after_wave_index,
            kind: barrier.kind.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanConflictReport {
    pub first_system_id: SystemId,
    pub first_system: String,
    pub second_system_id: SystemId,
    pub second_system: String,
    pub access_domain: AccessDomain,
    pub access_name: &'static str,
    pub conflict_kind: ConflictKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePlanDiagnosticReport {
    pub kind: ExecutionPlanDiagnosticKind,
    pub message: String,
}

fn system_reports_for_indices(
    systems: &[RegisteredSystem<World>],
    indices: &[usize],
) -> Vec<RuntimePlanSystemReport> {
    indices
        .iter()
        .filter_map(|system_index| system_report_for_index(systems, *system_index))
        .collect()
}

fn system_report_for_index(
    systems: &[RegisteredSystem<World>],
    system_index: usize,
) -> Option<RuntimePlanSystemReport> {
    let system = systems.get(system_index)?;
    let system_id = system.id();
    Some(RuntimePlanSystemReport {
        system_index,
        system_id,
        name: system.name().to_string(),
        param_slots: system
            .param_slots()
            .iter()
            .enumerate()
            .map(|(slot_index, slot)| ParamSlotMetadata {
                id: ParamSlotId {
                    system_id,
                    slot_index,
                },
                kind: slot.kind,
                label: slot.label,
                type_name: slot.type_name,
            })
            .collect(),
    })
}

fn missing_system_indices(systems: &[RegisteredSystem<World>], indices: &[usize]) -> Vec<usize> {
    indices
        .iter()
        .copied()
        .filter(|system_index| systems.get(*system_index).is_none())
        .collect()
}
