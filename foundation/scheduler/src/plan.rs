use crate::access::AccessConflict;
use crate::label::{ScheduleKey, ScheduleLabel, SystemSetKey};
use crate::system::RegisteredSystem;
use anyhow::{Context, Result, anyhow};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionConflict {
    pub first_system: String,
    pub second_system: String,
    pub conflict: AccessConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionStage {
    pub index: usize,
    pub system_indices: Vec<usize>,
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
        }
    }

    pub fn add_system(&mut self, system: RegisteredSystem<C>) -> usize {
        let index = self.systems.len();
        self.systems.push(system);
        self.dirty = true;
        index
    }

    pub fn systems(&self) -> &[RegisteredSystem<C>] {
        &self.systems
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

        let mut ordered_positions = Vec::with_capacity(scheduled_indices.len());
        while let Some(position) = ready.pop_front() {
            ordered_positions.push(position);
            for dependent in outgoing[position].iter().copied() {
                incoming[dependent] = incoming[dependent].saturating_sub(1);
                if incoming[dependent] == 0 {
                    ready.push_back(dependent);
                }
            }
        }

        if ordered_positions.len() != scheduled_indices.len() {
            return Err(anyhow!(
                "schedule '{}' has cyclic system ordering constraints",
                label.name()
            ));
        }

        let stages = ordered_positions
            .into_iter()
            .enumerate()
            .map(|(stage_index, position)| ExecutionStage {
                index: stage_index,
                system_indices: vec![scheduled_indices[position]],
            })
            .collect();

        let mut conflicts = Vec::new();
        for (left_pos, left_index) in scheduled_indices.iter().enumerate() {
            let left = &self.systems[*left_index];
            for right_index in scheduled_indices.iter().skip(left_pos + 1) {
                let right = &self.systems[*right_index];
                for conflict in left.access().conflicts_with(right.access()) {
                    conflicts.push(ExecutionConflict {
                        first_system: left.name().to_string(),
                        second_system: right.name().to_string(),
                        conflict,
                    });
                }
            }
        }

        Ok(ExecutionPlan {
            label,
            stages,
            conflicts,
        })
    }
}

fn depends_on_set(required_sets: &[SystemSetKey], assigned_sets: &[SystemSetKey]) -> bool {
    required_sets
        .iter()
        .any(|required| assigned_sets.iter().any(|assigned| assigned == required))
}
