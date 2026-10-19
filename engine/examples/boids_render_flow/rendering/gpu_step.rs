//! Immutable, source-tick-specific GPU inputs for all eight existing Boids compute stages.
//! This is a producer-local projection, not a new scheduling or physical-admission authority.
use crate::rendering::{BoidsComputeParams, BoidsRenderState};
use anyhow::{Context, Result};
use engine::plugins::render::{
    GpuParams, RenderFixedStepIterationUniform, RenderFlow, RenderFlowId, RenderGpuFixedStepInput,
    RenderPassId,
};
use runen_ecs::Resource;
use runen_gpu::{GpuBufferHandle, GpuWorkResourceId};
use std::collections::BTreeMap;

const STAGE_LABELS: [&str; 8] = [
    "boids.seed_or_hold",
    "boids.grid.clear_counts",
    "boids.grid.count_cells",
    "boids.grid.scan_counts",
    "boids.grid.reset_cursors",
    "boids.grid.scatter_sorted_indices",
    "boids.grid.simulate_neighbors",
    "boids.grid.publish_draw",
];
const REGION_MAX_SUBSTEPS: u32 = 4;

/// The IDs come from the validated ordinary RenderFlow, not independently allocated
/// copies of its GPU resources. Every authored state uniform must be captured.
#[derive(Debug, Clone, Resource)]
pub(crate) struct BoidsGpuStepSchema {
    flow_id: RenderFlowId,
    uniforms: [GpuWorkResourceId; 8],
    passes: [RenderPassId; 8],
    draw_pass: RenderPassId,
    cell_counts: GpuBufferHandle,
}

impl BoidsGpuStepSchema {
    pub(crate) fn from_flow(flow: &RenderFlow) -> Result<Self> {
        let mut uniforms = Vec::with_capacity(STAGE_LABELS.len());
        let mut passes = Vec::with_capacity(STAGE_LABELS.len());
        for label in STAGE_LABELS {
            let pass = flow
                .pass_id(label)
                .with_context(|| format!("missing Boids GPU stage '{label}'"))?;
            let uniform = flow
                .resource_id(&format!("{label}.uniform.0"))
                .with_context(|| {
                    format!("missing source-projected Boids GPU uniform for '{label}'")
                })?;
            passes.push(pass);
            uniforms.push(uniform);
        }
        let draw_pass = flow
            .pass_id("boids.draw")
            .context("missing Boids graph terminal draw pass for required GPU evidence")?;
        let grid_counts = flow
            .resource_id("boids.grid.cell_counts")
            .context("missing Boids GPU grid counter resource")?;
        let cell_counts = flow
            .graph()
            .resources
            .resources
            .iter()
            .find(|resource| *resource.id() == grid_counts)
            .and_then(|resource| resource.buffer_handle())
            .cloned()
            .context("Boids GPU grid counter has no actual G5 buffer handle")?;
        Ok(Self {
            flow_id: flow.id(),
            uniforms: uniforms
                .try_into()
                .expect("eight checked Boids uniform IDs"),
            passes: passes.try_into().expect("eight checked Boids pass IDs"),
            draw_pass,
            cell_counts,
        })
    }

    pub(crate) const fn flow_id(&self) -> RenderFlowId {
        self.flow_id
    }

    pub(crate) const fn draw_pass_id(&self) -> RenderPassId {
        self.draw_pass
    }

    pub(crate) fn cell_counts_buffer(&self) -> &GpuBufferHandle {
        &self.cell_counts
    }

    pub(crate) const fn max_substeps(&self) -> u32 {
        REGION_MAX_SUBSTEPS
    }

    /// source_tick is zero-based: first source tick 0 seeds the initial GPU
    /// buffers. The producer time-dilates source ticks under backpressure;
    /// elapsed host fixed ticks are not replayed. Already captured steps retain
    /// their original immutable source-tick inputs and identities.
    /// Mutable sources must snapshot inputs at source time instead.
    pub(crate) fn capture(
        &self,
        state: &BoidsRenderState,
        source_tick: u64,
    ) -> Result<RenderGpuFixedStepInput> {
        let tick =
            u32::try_from(source_tick).context("Boids GPU tick exceeds shader u32 domain")?;
        let state = state.at_source_tick(tick);
        let projections: [BoidsComputeParams; 8] = [
            state.seed_params(),
            state.clear_counts_params(),
            state.count_cells_params(),
            state.scan_counts_params(),
            state.reset_cursors_params(),
            state.scatter_indices_params(),
            state.compute_params(),
            state.publish_params(),
        ];
        let uniforms = self
            .uniforms
            .iter()
            .copied()
            .zip(projections)
            .map(|(id, value)| {
                (
                    id,
                    engine::plugins::render::bytemuck::bytes_of(&value.to_gpu()).to_vec(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let dispatches = [
            state.dispatch_workgroups(),
            state.dispatch_grid_workgroups(),
            state.dispatch_workgroups(),
            state.dispatch_scan_workgroups(),
            state.dispatch_grid_workgroups(),
            state.dispatch_workgroups(),
            state.dispatch_workgroups(),
            state.dispatch_workgroups(),
        ];
        let dispatch_workgroups = self
            .passes
            .iter()
            .copied()
            .zip(dispatches)
            .collect::<BTreeMap<_, _>>();
        anyhow::ensure!(
            uniforms.len() == STAGE_LABELS.len() && dispatch_workgroups.len() == STAGE_LABELS.len(),
            "Boids GPU source has duplicate resource or pass identity"
        );
        Ok(RenderGpuFixedStepInput {
            uniforms,
            dispatch_workgroups,
            iteration: RenderFixedStepIterationUniform::new(
                0,
                1,
                REGION_MAX_SUBSTEPS,
                0,
                state.fixed_delta_seconds(),
                0.0,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::{BoidsRuleFamily, build_render_flow};

    #[test]
    fn boids_captures_every_stage_from_real_flow_at_each_source_tick() {
        let flow = build_render_flow();
        let schema = BoidsGpuStepSchema::from_flow(&flow).expect("all eight authored stages");
        assert_eq!(schema.flow_id(), flow.id());
        assert_eq!(schema.draw_pass_id(), flow.pass_id("boids.draw").unwrap());
        let state = BoidsRenderState::default();
        let step0 = schema.capture(&state, 0).expect("seed step");
        let step1 = schema.capture(&state, 1).expect("subsequent step");
        assert_eq!(step0.uniforms.len(), 8);
        assert_eq!(step0.dispatch_workgroups.len(), 8);
        assert_eq!(step0.iteration.submitted_substeps, 1);
        assert_eq!(step1.iteration.substep_index, 0);
        assert_eq!(step0.iteration.max_substeps, 4);
        assert_eq!(step0.dispatch_workgroups, step1.dispatch_workgroups);
        assert_ne!(
            step0.uniforms, step1.uniforms,
            "a new source tick needs new immutable uniforms"
        );
        assert!(schema.capture(&state, u64::MAX).is_err());
    }

    #[test]
    fn species_law_changes_source_bytes_but_not_physical_stage_shape() {
        let flow = build_render_flow();
        let schema = BoidsGpuStepSchema::from_flow(&flow).unwrap();
        let flock = schema.capture(&BoidsRenderState::default(), 9).unwrap();
        let species = schema
            .capture(
                &BoidsRenderState::with_rule(BoidsRuleFamily::SpeciesInteraction),
                9,
            )
            .unwrap();
        assert_eq!(flock.dispatch_workgroups, species.dispatch_workgroups);
        assert_ne!(flock.uniforms, species.uniforms);
    }
}
