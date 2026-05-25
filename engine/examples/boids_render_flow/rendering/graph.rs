use crate::rendering::{
    BoidAgent, BoidsRenderState, DEFAULT_BOID_COUNT, DEFAULT_GRID_CELLS_X, DEFAULT_GRID_CELLS_Y,
};
use engine::plugins::render::{
    BoundedUniformGrid2dBuildPlan, BoundedUniformGrid2dConfig, BoundedUniformGrid2dStage,
    ProceduralBufferBinding, ProceduralPassDescriptor, ProceduralTargetDescriptor, RenderFlow,
    RenderVertexBufferLayout, RenderVertexFormat, U32Counter, U32ScanElement,
};

pub(crate) fn build_render_flow() -> RenderFlow {
    let grid_cell_count = (DEFAULT_GRID_CELLS_X * DEFAULT_GRID_CELLS_Y) as u64;
    let (flow, boid_instances) = RenderFlow::new("boids_render_flow")
        .with_state::<BoidsRenderState>()
        .with_surface_color()
        .with_color_target("boids.color")
        .double_buffer_storage_array_with_handle::<BoidAgent>(
            "boids.instances",
            DEFAULT_BOID_COUNT as u64,
        );
    let (flow, grid_cell_counts) =
        flow.storage_array::<U32Counter>("boids.grid.cell_counts", grid_cell_count);
    let (flow, grid_cell_offsets) =
        flow.storage_array::<U32ScanElement>("boids.grid.cell_offsets", grid_cell_count);
    let (flow, grid_scatter_cursors) =
        flow.storage_array::<U32Counter>("boids.grid.scatter_cursors", grid_cell_count);
    let (flow, grid_sorted_indices) = flow
        .storage_array::<U32ScanElement>("boids.grid.sorted_indices", DEFAULT_BOID_COUNT as u64);
    let grid_plan = BoundedUniformGrid2dBuildPlan::new(
        "boids.grid",
        BoundedUniformGrid2dConfig::new(
            DEFAULT_GRID_CELLS_X,
            DEFAULT_GRID_CELLS_Y,
            DEFAULT_BOID_COUNT,
        ),
        grid_cell_counts.clone(),
        grid_cell_offsets.clone(),
        grid_scatter_cursors.clone(),
        grid_sorted_indices.clone(),
    )
    .expect("boids bounded grid plan should validate");
    let clear_counts = stage_label(&grid_plan, BoundedUniformGrid2dStage::ClearCounts).to_string();
    let count_cells = stage_label(&grid_plan, BoundedUniformGrid2dStage::CountCells).to_string();
    let scan_counts = stage_label(&grid_plan, BoundedUniformGrid2dStage::ScanCounts).to_string();
    let reset_cursors =
        stage_label(&grid_plan, BoundedUniformGrid2dStage::ResetCursors).to_string();
    let scatter_indices =
        stage_label(&grid_plan, BoundedUniformGrid2dStage::ScatterSortedIndices).to_string();
    let simulate_neighbors =
        stage_label(&grid_plan, BoundedUniformGrid2dStage::SimulateNeighbors).to_string();
    let publish_draw = stage_label(&grid_plan, BoundedUniformGrid2dStage::PublishDraw).to_string();

    let instance_layout = boid_instance_layout();
    let instance_buffer =
        ProceduralBufferBinding::storage(boid_instances.a().clone(), instance_layout);

    let flow = flow
        .compute_pass("boids.seed_or_hold")
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::seed_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .finish()
        .compute_pass(clear_counts.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::clear_counts_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_grid_workgroups)
        .depends_on("boids.seed_or_hold")
        .finish()
        .compute_pass(count_cells.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::count_cells_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .depends_on(clear_counts.as_str())
        .finish()
        .compute_pass(scan_counts.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::scan_counts_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_scan_workgroups)
        .depends_on(count_cells.as_str())
        .finish()
        .compute_pass(reset_cursors.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::reset_cursors_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_grid_workgroups)
        .depends_on(scan_counts.as_str())
        .finish()
        .compute_pass(scatter_indices.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::scatter_indices_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .depends_on(reset_cursors.as_str())
        .finish()
        .compute_pass(simulate_neighbors.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::compute_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts.clone())
        .bind_storage(grid_cell_offsets.clone())
        .bind_storage(grid_scatter_cursors.clone())
        .bind_storage(grid_sorted_indices.clone())
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .depends_on(scatter_indices.as_str())
        .finish()
        .compute_pass(publish_draw.clone())
        .shader_asset("assets/shaders/boids_compute.wgsl")
        .uniform_from_state(BoidsRenderState::publish_params)
        .bind_ping_pong_storage(boid_instances.name())
        .bind_storage(grid_cell_counts)
        .bind_storage(grid_cell_offsets)
        .bind_storage(grid_scatter_cursors)
        .bind_storage(grid_sorted_indices)
        .dispatch_from_state(BoidsRenderState::dispatch_workgroups)
        .depends_on(simulate_neighbors.as_str())
        .finish();

    let flow = flow.fixed_step_region(
        "boids.fixed_step",
        4,
        [
            "boids.seed_or_hold",
            clear_counts.as_str(),
            count_cells.as_str(),
            scan_counts.as_str(),
            reset_cursors.as_str(),
            scatter_indices.as_str(),
            simulate_neighbors.as_str(),
            publish_draw.as_str(),
        ],
    );

    let flow = flow
        .procedural_pass_builder(
            ProceduralPassDescriptor::local_sdf_2d_impostors(
                "boids.draw",
                instance_buffer,
                DEFAULT_BOID_COUNT,
            )
            .shader_asset("assets/shaders/boids_compose.wgsl")
            .target(
                ProceduralTargetDescriptor::color("boids.color")
                    .clear_color([0.020, 0.028, 0.040, 1.0]),
            )
            .depends_on(publish_draw.as_str()),
        )
        .expect("boids.draw procedural builder should be valid")
        .uniform_from_state_with_surface(BoidsRenderState::draw_params)
        .finish()
        .expect("boids.draw procedural pass should lower");

    flow.present_pass("boids.present")
        .source("boids.color")
        .depends_on("boids.draw")
        .finish()
        .validate()
        .expect("boids_render_flow should validate")
}

fn stage_label(plan: &BoundedUniformGrid2dBuildPlan, stage: BoundedUniformGrid2dStage) -> &str {
    plan.stages
        .iter()
        .find(|candidate| candidate.stage == stage)
        .map(|candidate| candidate.label.as_str())
        .expect("bounded grid plan should include canonical stage")
}

fn boid_instance_layout() -> RenderVertexBufferLayout {
    RenderVertexBufferLayout::instance(
        0,
        std::mem::size_of::<<BoidAgent as engine::plugins::render::GpuParams>::Raw>() as u64,
    )
    .attribute(0, 0, RenderVertexFormat::Float32x2)
    .attribute(1, 8, RenderVertexFormat::Float32x2)
    .attribute(2, 16, RenderVertexFormat::Float32x2)
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;
    #[allow(deprecated)]
    use engine::plugins::render::{
        RenderBackendCapabilityProfile, RenderFrameDataRegistry, RenderPassId, RenderPassKind,
        compile_flow_plan_checked,
    };

    fn pass_kind(flow: &RenderFlow, pass_id: &str) -> RenderPassKind {
        flow.graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == pass_id)
            .map(|pass| pass.kind)
            .expect("requested pass should exist")
    }

    fn pass_id(flow: &RenderFlow, pass_label: &str) -> RenderPassId {
        flow.graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == pass_label)
            .map(|pass| pass.id)
            .expect("requested pass should exist")
    }

    fn expected_pass_order() -> Vec<&'static str> {
        vec![
            "boids.seed_or_hold",
            "boids.grid.clear_counts",
            "boids.grid.count_cells",
            "boids.grid.scan_counts",
            "boids.grid.reset_cursors",
            "boids.grid.scatter_sorted_indices",
            "boids.grid.simulate_neighbors",
            "boids.grid.publish_draw",
            "boids.draw",
            "boids.present",
        ]
    }

    #[test]
    fn flow_declares_expected_passes() {
        let flow = build_render_flow();
        let graph = flow.graph();
        let pass_ids = graph
            .passes
            .passes
            .iter()
            .map(|pass| pass.label.as_str())
            .collect::<Vec<_>>();
        assert_eq!(pass_ids, expected_pass_order());
        assert_eq!(
            pass_kind(&flow, "boids.seed_or_hold"),
            RenderPassKind::Compute
        );
        assert_eq!(
            pass_kind(&flow, "boids.grid.simulate_neighbors"),
            RenderPassKind::Compute
        );
        assert_eq!(
            pass_kind(&flow, "boids.grid.publish_draw"),
            RenderPassKind::Compute
        );
        assert_eq!(pass_kind(&flow, "boids.draw"), RenderPassKind::Graphics);
        assert_eq!(pass_kind(&flow, "boids.present"), RenderPassKind::Present);
    }

    #[test]
    fn flow_does_not_keep_history_copy_without_history_consumer() {
        let flow = build_render_flow();
        assert!(
            flow.graph()
                .passes
                .passes
                .iter()
                .all(|pass| pass.kind != RenderPassKind::Copy)
        );
        assert!(
            flow.graph()
                .passes
                .passes
                .iter()
                .all(|pass| !pass.label.contains("history"))
        );
        assert!(flow.resource_id("boids.history").is_none());
    }

    #[test]
    fn procedural_draw_pass_binds_local_instance_geometry() {
        let flow = build_render_flow();
        let draw_pass = flow
            .graph()
            .passes
            .passes
            .iter()
            .find(|pass| pass.label == "boids.draw")
            .expect("draw pass should exist");

        assert_eq!(draw_pass.instance_buffers.len(), 1);
        assert_eq!(draw_pass.instance_buffer_layouts.len(), 1);
        assert_eq!(draw_pass.instance_buffer_layouts[0], boid_instance_layout());
        assert_eq!(
            draw_pass
                .draw
                .expect("draw descriptor should exist")
                .vertex_count,
            6
        );
        assert_eq!(
            draw_pass
                .draw
                .expect("draw descriptor should exist")
                .instance_count,
            DEFAULT_BOID_COUNT
        );

        compile_flow_plan_checked(&flow, &RenderBackendCapabilityProfile::runtime_default())
            .expect("boids procedural draw should satisfy pass-shape guards");
    }

    #[test]
    fn flow_orders_compute_publish_procedural_draw_then_present() {
        let flow = build_render_flow();
        let order = flow
            .pass_order()
            .expect("boids_render_flow pass order should validate")
            .into_iter()
            .map(|id| {
                flow.graph()
                    .passes
                    .passes
                    .iter()
                    .find(|pass| pass.id == id)
                    .map(|pass| pass.label.as_str())
                    .expect("ordered pass should exist")
            })
            .collect::<Vec<_>>();

        assert_eq!(order, expected_pass_order());
    }

    #[test]
    fn state_projects_simulation_and_publish_uniforms() {
        let flow = build_render_flow();
        let state = BoidsRenderState::default();
        // Projection-helper compatibility surface; active runtime submission uses PreparedRenderFrame.
        #[allow(deprecated)]
        let frame_data = RenderFrameDataRegistry::new().with(&state);

        let uniforms = flow
            .project_uniforms(&frame_data, (1600, 900))
            .expect("uniform projection should succeed");

        assert!(
            uniforms
                .pass(pass_id(&flow, "boids.seed_or_hold"))
                .is_some()
        );
        assert!(
            uniforms
                .pass(pass_id(&flow, "boids.grid.scan_counts"))
                .is_some()
        );
        assert!(
            uniforms
                .pass(pass_id(&flow, "boids.grid.publish_draw"))
                .is_some()
        );
        assert!(uniforms.pass(pass_id(&flow, "boids.draw")).is_some());
    }

    #[test]
    fn compose_shader_uses_instance_inputs_and_surface_uniform_without_storage_loop() {
        let shader = include_str!("../../../../assets/shaders/boids_compose.wgsl");
        assert!(shader.contains("@location(0) instance_position"));
        assert!(shader.contains("@location(1) instance_velocity"));
        assert!(shader.contains("@location(2) instance_visual_heading"));
        assert!(shader.contains("var<uniform> draw_params"));
        assert!(shader.contains("world_to_clip"));
        assert!(shader.contains("visible_world"));
        assert!(!shader.contains("var<storage"));
        assert!(!shader.contains("for (var i"));
    }

    #[test]
    fn compute_shader_uses_bounded_grid_neighbor_lookup() {
        let shader = include_str!("../../../../assets/shaders/boids_compute.wgsl");
        assert!(shader.contains("cell_counts"));
        assert!(shader.contains("cell_offsets"));
        assert!(shader.contains("sorted_indices"));
        assert!(!shader.contains("i < boid_count"));
    }

    #[test]
    fn boids_shaders_parse_as_wgsl() {
        validate_wgsl(include_str!(
            "../../../../assets/shaders/boids_compute.wgsl"
        ));
        validate_wgsl(include_str!(
            "../../../../assets/shaders/boids_compose.wgsl"
        ));
    }

    fn validate_wgsl(shader: &str) {
        let module = naga::front::wgsl::parse_str(shader).expect("shader should parse as WGSL");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("shader should validate");
    }
}
