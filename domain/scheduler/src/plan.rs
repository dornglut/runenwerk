use crate::access::AccessConflict;
use crate::label::{ScheduleKey, ScheduleLabel, SystemSetKey};
use crate::system::{RegisteredSystem, SystemId};
use crate::telemetry;
use anyhow::{Context, Result, anyhow};
use std::collections::{BTreeSet, VecDeque};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionConflict {
    pub first_system_id: SystemId,
    pub first_system: String,
    pub second_system_id: SystemId,
    pub second_system: String,
    pub conflict: AccessConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionStage {
    pub index: usize,
    pub system_indices: Vec<usize>,
    pub system_ids: Vec<SystemId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionPhaseKind {
    PreUpdate,
    FixedUpdate,
    Update,
    RenderPrepare,
    RenderSubmit,
    FrameEnd,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPhase {
    pub index: usize,
    pub label: ScheduleKey,
    pub kind: ExecutionPhaseKind,
}

impl ExecutionPhase {
    pub fn from_schedule_label(index: usize, label: ScheduleKey) -> Self {
        let short_name = label.name().rsplit("::").next().unwrap_or(label.name());
        let kind = match short_name {
            "PreUpdate" => ExecutionPhaseKind::PreUpdate,
            "FixedUpdate" => ExecutionPhaseKind::FixedUpdate,
            "Update" => ExecutionPhaseKind::Update,
            "RenderPrepare" => ExecutionPhaseKind::RenderPrepare,
            "RenderSubmit" => ExecutionPhaseKind::RenderSubmit,
            "FrameEnd" => ExecutionPhaseKind::FrameEnd,
            _ => ExecutionPhaseKind::Custom(short_name.to_string()),
        };
        Self { index, label, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionWave {
    pub index: usize,
    pub phase_index: usize,
    pub stage_index: usize,
    pub system_indices: Vec<usize>,
    pub system_ids: Vec<SystemId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarrierKind {
    ApplyDeferredCommands,
    ProductPublication,
    QuerySnapshotPublication,
    RenderSubmit,
    GenerationFinalization,
    ReplayNetworkCapture,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionBarrier {
    pub index: usize,
    pub phase_index: usize,
    pub after_wave_index: Option<usize>,
    pub kind: BarrierKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionPlanDiagnosticKind {
    EmptyPhase,
    SerialWaveMirrorsStage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlanDiagnostic {
    pub kind: ExecutionPlanDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    pub label: ScheduleKey,
    pub phase: ExecutionPhase,
    pub stages: Vec<ExecutionStage>,
    pub waves: Vec<ExecutionWave>,
    pub barriers: Vec<ExecutionBarrier>,
    pub conflicts: Vec<ExecutionConflict>,
    pub diagnostics: Vec<ExecutionPlanDiagnostic>,
}

impl ExecutionPlan {
    pub fn barriers_after_wave(
        &self,
        wave_index: usize,
    ) -> impl Iterator<Item = &ExecutionBarrier> {
        self.barriers
            .iter()
            .filter(move |barrier| barrier.after_wave_index == Some(wave_index))
    }

    pub fn apply_deferred_barrier_after_wave(&self, wave_index: usize) -> bool {
        self.barriers_after_wave(wave_index)
            .any(|barrier| barrier.kind == BarrierKind::ApplyDeferredCommands)
    }
}

pub struct ExecutionScheduler<C> {
    systems: Vec<RegisteredSystem<C>>,
    plans: Vec<ExecutionPlan>,
    dirty: bool,
    next_system_id: u64,
}

impl<C> Default for ExecutionScheduler<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> ExecutionScheduler<C> {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
            plans: Vec::new(),
            dirty: true,
            next_system_id: 0,
        }
    }

    pub fn add_system(&mut self, mut system: RegisteredSystem<C>) -> usize {
        let system_id = SystemId::from_raw(self.next_system_id);
        self.next_system_id = self.next_system_id.saturating_add(1);
        system.assign_id(system_id);
        let index = self.systems.len();
        self.systems.push(system);
        self.dirty = true;
        index
    }

    pub fn systems(&self) -> &[RegisteredSystem<C>] {
        &self.systems
    }

    pub fn systems_mut(&mut self) -> &mut [RegisteredSystem<C>] {
        &mut self.systems
    }

    pub fn plans(&mut self) -> &[ExecutionPlan] {
        self.rebuild_if_dirty()
            .expect("execution plan rebuild failed");
        &self.plans
    }

    pub fn plan_for<L: ScheduleLabel>(&mut self) -> Option<&ExecutionPlan> {
        self.rebuild_if_dirty().ok()?;
        self.plans.iter().find(|plan| plan.label == L::key())
    }

    pub fn run_schedule<L: ScheduleLabel>(&mut self, ctx: &mut C) -> Result<()> {
        self.rebuild_if_dirty()?;
        let Some(plan) = self
            .plans
            .iter()
            .find(|plan| plan.label == L::key())
            .cloned()
        else {
            return Ok(());
        };

        for wave in plan.waves {
            for system_index in wave.system_indices {
                let system = self
                    .systems
                    .get_mut(system_index)
                    .context("execution plan referenced missing system")?;
                system
                    .run(ctx)
                    .with_context(|| format!("failed to run system '{}'", system.name()))?;
            }
        }

        Ok(())
    }

    fn rebuild_if_dirty(&mut self) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let mut labels: Vec<ScheduleKey> = Vec::new();
        for system in &self.systems {
            if !labels.iter().any(|label| *label == system.label()) {
                labels.push(system.label());
            }
        }

        self.plans = labels
            .into_iter()
            .map(|label| self.build_plan(label))
            .collect::<Result<Vec<_>>>()?;
        self.dirty = false;
        Ok(())
    }

    fn build_plan(&self, label: ScheduleKey) -> Result<ExecutionPlan> {
        let build_start = Instant::now();
        let mut conflict_check_count = 0_u64;
        let scheduled_indices: Vec<_> = self
            .systems
            .iter()
            .enumerate()
            .filter_map(|(index, system)| (system.label() == label).then_some(index))
            .collect();

        let mut outgoing = vec![BTreeSet::<usize>::new(); scheduled_indices.len()];
        let mut incoming = vec![0usize; scheduled_indices.len()];

        for (source_pos, source_index) in scheduled_indices.iter().enumerate() {
            let source = &self.systems[*source_index];
            for (target_pos, target_index) in scheduled_indices.iter().enumerate() {
                if source_pos == target_pos {
                    continue;
                }
                let target = &self.systems[*target_index];

                if depends_on_set(source.after_sets(), target.sets())
                    && outgoing[target_pos].insert(source_pos)
                {
                    incoming[source_pos] = incoming[source_pos].saturating_add(1);
                }

                if depends_on_set(source.before_sets(), target.sets())
                    && outgoing[source_pos].insert(target_pos)
                {
                    incoming[target_pos] = incoming[target_pos].saturating_add(1);
                }
            }
        }

        let mut ready = VecDeque::new();
        for (position, indegree) in incoming.iter().enumerate() {
            if *indegree == 0 {
                ready.push_back(position);
            }
        }

        let mut ready_set = BTreeSet::new();
        while let Some(position) = ready.pop_front() {
            ready_set.insert(position);
        }

        let mut stages = Vec::new();
        let mut stage_index = 0usize;
        let mut scheduled_count = 0usize;

        while !ready_set.is_empty() {
            let mut stage_positions: Vec<usize> = Vec::new();

            for position in ready_set.iter().copied() {
                let candidate_index = scheduled_indices[position];
                let candidate = &self.systems[candidate_index];
                let compatible = stage_positions.iter().all(|existing_position| {
                    let existing_index = scheduled_indices[*existing_position];
                    let existing = &self.systems[existing_index];
                    conflict_check_count = conflict_check_count.saturating_add(1);
                    existing
                        .access()
                        .conflicts_with(candidate.access())
                        .is_empty()
                });
                if compatible {
                    stage_positions.push(position);
                }
            }

            if stage_positions.is_empty() {
                let fallback = *ready_set
                    .iter()
                    .next()
                    .expect("ready set should contain at least one system");
                stage_positions.push(fallback);
            }

            let mut stage_system_indices = Vec::with_capacity(stage_positions.len());
            let mut stage_system_ids = Vec::with_capacity(stage_positions.len());
            for position in &stage_positions {
                ready_set.remove(position);
                let system_index = scheduled_indices[*position];
                stage_system_indices.push(system_index);
                stage_system_ids.push(self.systems[system_index].id());
            }

            scheduled_count = scheduled_count.saturating_add(stage_system_indices.len());

            for position in stage_positions {
                for dependent in outgoing[position].iter().copied() {
                    incoming[dependent] = incoming[dependent].saturating_sub(1);
                    if incoming[dependent] == 0 {
                        ready_set.insert(dependent);
                    }
                }
            }

            stages.push(ExecutionStage {
                index: stage_index,
                system_indices: stage_system_indices,
                system_ids: stage_system_ids,
            });
            stage_index = stage_index.saturating_add(1);
        }

        if scheduled_count != scheduled_indices.len() {
            return Err(anyhow!(
                "schedule '{}' has cyclic system ordering constraints",
                label.name()
            ));
        }

        let mut conflicts = Vec::new();
        for (left_pos, left_index) in scheduled_indices.iter().enumerate() {
            let left = &self.systems[*left_index];
            for right_index in scheduled_indices.iter().skip(left_pos + 1) {
                let right = &self.systems[*right_index];
                conflict_check_count = conflict_check_count.saturating_add(1);
                for conflict in left.access().conflicts_with(right.access()) {
                    conflicts.push(ExecutionConflict {
                        first_system_id: left.id(),
                        first_system: left.name().to_string(),
                        second_system_id: right.id(),
                        second_system: right.name().to_string(),
                        conflict,
                    });
                }
            }
        }

        let phase = ExecutionPhase::from_schedule_label(0, label);
        let waves = stages
            .iter()
            .enumerate()
            .map(|(wave_index, stage)| ExecutionWave {
                index: wave_index,
                phase_index: phase.index,
                stage_index: stage.index,
                system_indices: stage.system_indices.clone(),
                system_ids: stage.system_ids.clone(),
            })
            .collect::<Vec<_>>();
        let barriers = waves
            .iter()
            .flat_map(|wave| {
                [
                    BarrierKind::ApplyDeferredCommands,
                    BarrierKind::ProductPublication,
                    BarrierKind::QuerySnapshotPublication,
                ]
                .into_iter()
                .map(move |kind| (wave.index, kind))
            })
            .enumerate()
            .map(|(barrier_index, (wave_index, kind))| ExecutionBarrier {
                index: barrier_index,
                phase_index: phase.index,
                after_wave_index: Some(wave_index),
                kind,
            })
            .collect::<Vec<_>>();
        let mut diagnostics = Vec::new();
        if waves.is_empty() {
            diagnostics.push(ExecutionPlanDiagnostic {
                kind: ExecutionPlanDiagnosticKind::EmptyPhase,
                message: format!(
                    "schedule '{}' produced an empty execution phase",
                    label.name()
                ),
            });
        } else {
            diagnostics.push(ExecutionPlanDiagnostic {
                kind: ExecutionPlanDiagnosticKind::SerialWaveMirrorsStage,
                message:
                    "serial fabric compatibility: each execution wave mirrors one legacy stage"
                        .to_string(),
            });
        }

        let plan = ExecutionPlan {
            label,
            phase,
            stages,
            waves,
            barriers,
            conflicts,
            diagnostics,
        };
        telemetry::record_plan_build(
            build_start.elapsed().as_nanos() as u64,
            conflict_check_count,
            plan.stages.len() as u64,
        );
        Ok(plan)
    }
}

fn depends_on_set(required_sets: &[SystemSetKey], assigned_sets: &[SystemSetKey]) -> bool {
    required_sets
        .iter()
        .any(|required| assigned_sets.iter().any(|assigned| assigned == required))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access::SystemAccess;
    use crate::label::{ScheduleLabel, SystemSet};
    use crate::system::RegisteredSystem;

    #[derive(Debug, Copy, Clone)]
    struct Update;
    impl ScheduleLabel for Update {}

    #[derive(Debug, Copy, Clone)]
    struct GameplaySet;
    impl SystemSet for GameplaySet {}

    #[derive(Debug, Copy, Clone)]
    struct PostGameplaySet;
    impl SystemSet for PostGameplaySet {}

    fn test_system(name: &'static str) -> RegisteredSystem<Vec<&'static str>> {
        RegisteredSystem::new::<Update>(
            name,
            SystemAccess::new(),
            move |ctx: &mut Vec<&'static str>| {
                ctx.push(name);
                Ok(())
            },
        )
        .expect("test system should register")
    }

    #[test]
    fn plan_exposes_serial_waves_apply_deferred_product_and_query_snapshot_barriers() {
        let mut before = test_system("before");
        before.before_set_key(GameplaySet::key());
        let mut gameplay = test_system("gameplay");
        gameplay.with_set_key(GameplaySet::key());
        let mut after = test_system("after");
        after.with_set_key(PostGameplaySet::key());
        after.after_set_key(GameplaySet::key());

        let mut scheduler = ExecutionScheduler::new();
        scheduler.add_system(gameplay);
        scheduler.add_system(before);
        scheduler.add_system(after);

        let plan = scheduler
            .plan_for::<Update>()
            .expect("plan should exist")
            .clone();

        assert_eq!(plan.phase.kind, ExecutionPhaseKind::Update);
        assert_eq!(plan.stages.len(), 3);
        assert_eq!(plan.waves.len(), plan.stages.len());
        assert_eq!(plan.barriers.len(), plan.waves.len() * 3);
        for wave in &plan.waves {
            assert_eq!(wave.stage_index, wave.index);
            assert!(plan.apply_deferred_barrier_after_wave(wave.index));
            let barriers = plan.barriers_after_wave(wave.index).collect::<Vec<_>>();
            assert_eq!(barriers.len(), 3);
            assert_eq!(barriers[0].kind, BarrierKind::ApplyDeferredCommands);
            assert_eq!(barriers[1].kind, BarrierKind::ProductPublication);
            assert_eq!(barriers[2].kind, BarrierKind::QuerySnapshotPublication);
        }

        let mut order = Vec::new();
        scheduler
            .run_schedule::<Update>(&mut order)
            .expect("serial wave execution should succeed");
        assert_eq!(order, vec!["before", "gameplay", "after"]);
    }
}
