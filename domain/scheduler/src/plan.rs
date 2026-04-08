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
pub struct ExecutionPlan {
    pub label: ScheduleKey,
    pub stages: Vec<ExecutionStage>,
    pub conflicts: Vec<ExecutionConflict>,
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

        for stage in plan.stages {
            for system_index in stage.system_indices {
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

        let plan = ExecutionPlan {
            label,
            stages,
            conflicts,
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
