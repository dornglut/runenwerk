use engine::plugins::render::resources::RenderFrameDataRegistry;
use engine::plugins::render::{GpuUniform, RenderFlow};

#[derive(Debug, Clone, ecs::Component)]
struct BindingState {
    tick: u32,
    exposure: f32,
}

impl BindingState {
    fn compute_params(&self) -> ComputeParams {
        ComputeParams { tick: self.tick }
    }

    fn compose_params(&self, surface: (u32, u32)) -> ComposeParams {
        ComposeParams {
            surface_size: [surface.0 as f32, surface.1 as f32],
            exposure: self.exposure,
        }
    }
}

#[derive(Debug, Clone, ecs::Component)]
struct SecondaryBindingState {
    tick: u32,
}

impl SecondaryBindingState {
    fn compute_params(&self) -> ComputeParams {
        ComputeParams { tick: self.tick }
    }
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComputeParams {
    tick: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    surface_size: [f32; 2],
    exposure: f32,
}

#[test]
fn validate_reports_missing_ecs_resource_for_uniform_state() {
    let flow = RenderFlow::new("demo.flow")
        .uniform_buffer::<ComputeParams>("demo.params")
        .compute_pass("demo.compute")
        .uniform_state(BindingState::compute_params)
        .finish();

    let err = flow
        .validate()
        .expect_err("missing ecs_resource declaration must fail");
    assert!(
        err.to_string()
            .contains("ecs_resource::<...>() was not declared"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_reports_missing_uniform_buffer_for_uniform_state() {
    let flow = RenderFlow::new("demo.flow")
        .ecs_resource::<BindingState>()
        .compute_pass("demo.compute")
        .uniform_state(BindingState::compute_params)
        .finish();

    let err = flow
        .validate()
        .expect_err("missing uniform_buffer declaration must fail");
    assert!(
        err.to_string().contains("no matching uniform_buffer"),
        "unexpected error: {err}"
    );
}

#[test]
fn project_uniform_buffers_builds_gpu_bytes_for_state_methods() {
    let flow = RenderFlow::new("demo.flow")
        .ecs_resource::<BindingState>()
        .uniform_buffer::<ComputeParams>("demo.compute.params")
        .uniform_buffer::<ComposeParams>("demo.compose.params")
        .compute_pass("demo.compute")
        .uniform_state(BindingState::compute_params)
        .finish()
        .fullscreen_pass("demo.compose")
        .uniform_state_with_surface(BindingState::compose_params)
        .finish();

    flow.validate().expect("flow should validate");

    let state = BindingState {
        tick: 9,
        exposure: 1.25,
    };
    let frame_data = RenderFrameDataRegistry::new().with(&state);
    let projections = flow
        .project_uniform_buffers(&frame_data, (800, 600))
        .expect("projection should succeed");

    assert_eq!(projections.len(), 2);
    assert!(projections.iter().any(|pass| pass.pass_id == "demo.compute"
        && pass.buffers[0].buffer_id.as_str() == "demo.compute.params"
        && !pass.buffers[0].bytes.is_empty()));
    assert!(projections.iter().any(|pass| pass.pass_id == "demo.compose"
        && pass.buffers[0].buffer_id.as_str() == "demo.compose.params"
        && !pass.buffers[0].bytes.is_empty()));
}

#[test]
fn project_uniform_buffers_deduplicates_identical_target_buffer_uploads() {
    let flow = RenderFlow::new("dedupe.flow")
        .ecs_resource::<BindingState>()
        .uniform_buffer::<ComputeParams>("dedupe.compute.params")
        .compute_pass("dedupe.compute")
        .uniform_state(BindingState::compute_params)
        .uniform_state(BindingState::compute_params)
        .finish();

    flow.validate().expect("flow should validate");

    let state = BindingState {
        tick: 17,
        exposure: 0.5,
    };
    let frame_data = RenderFrameDataRegistry::new().with(&state);
    let projections = flow
        .project_uniform_buffers(&frame_data, (1, 1))
        .expect("projection should succeed");

    assert_eq!(projections.len(), 1);
    assert_eq!(projections[0].pass_id, "dedupe.compute");
    assert_eq!(projections[0].buffers.len(), 1);
    assert_eq!(
        projections[0].buffers[0].buffer_id.as_str(),
        "dedupe.compute.params"
    );
}

#[test]
fn project_uniform_buffers_reports_conflicting_writes_to_same_buffer() {
    let flow = RenderFlow::new("conflict.flow")
        .ecs_resource::<BindingState>()
        .ecs_resource::<SecondaryBindingState>()
        .uniform_buffer::<ComputeParams>("conflict.compute.params")
        .compute_pass("conflict.compute")
        .uniform_state(BindingState::compute_params)
        .uniform_state(SecondaryBindingState::compute_params)
        .finish();

    flow.validate().expect("flow should validate");

    let primary = BindingState {
        tick: 1,
        exposure: 0.0,
    };
    let secondary = SecondaryBindingState { tick: 999 };
    let frame_data = RenderFrameDataRegistry::new()
        .with(&primary)
        .with(&secondary);
    let err = flow
        .project_uniform_buffers(&frame_data, (1, 1))
        .expect_err("conflicting buffer projections must fail");

    assert!(err.iter().any(|item| {
        item.details
            .contains("conflicting uniform_state projections")
    }));
}
