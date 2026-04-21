use engine::plugins::render::{
    GpuStorage, GpuUniform, RenderFlow, RenderFrameDataRegistry, RenderPassKind,
    RenderPassId, ShaderRegistryResource,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, GpuStorage)]
struct Cell {
    alive: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComputeParams {
    tick: u32,
    grid: [u32; 2],
    step: u32,
}

#[derive(Debug, Clone, Copy, GpuUniform)]
struct ComposeParams {
    grid: [u32; 2],
    surface: [f32; 2],
}

#[derive(Debug, Clone, ecs::Resource)]
struct FlowState {
    tick: u32,
    grid: [u32; 2],
}

impl Default for FlowState {
    fn default() -> Self {
        Self {
            tick: 0,
            grid: [16, 9],
        }
    }
}

impl FlowState {
    fn compute_params(&self) -> ComputeParams {
        ComputeParams {
            tick: self.tick,
            grid: self.grid,
            step: 1,
        }
    }

    fn compose_params(&self, surface: (u32, u32)) -> ComposeParams {
        ComposeParams {
            grid: self.grid,
            surface: [surface.0 as f32, surface.1 as f32],
        }
    }

    fn dispatch(&self) -> [u32; 3] {
        [self.grid[0].div_ceil(8), self.grid[1].div_ceil(8), 1]
    }
}

fn build_flow() -> RenderFlow {
    RenderFlow::new("v2.flow")
        .with_state::<FlowState>()
        .with_surface_color()
        .with_builtin_ui()
        .double_buffer_storage_array::<Cell>("cells", 16 * 9)
        .compute_pass("simulate")
        .shader_asset("assets/shaders/game_of_life_compute.wgsl")
        .uniform_from_state(FlowState::compute_params)
        .bind_ping_pong_storage("cells")
        .dispatch_from_state(FlowState::dispatch)
        .finish()
        .fullscreen_pass("compose")
        .shader_asset("assets/shaders/game_of_life_compose.wgsl")
        .uniform_from_state_with_surface(FlowState::compose_params)
        .bind_ping_pong_storage("cells")
        .write_surface_color()
        .depends_on("simulate")
        .finish()
        .builtin_ui_composite_pass("ui")
        .depends_on("compose")
        .finish()
        .validate()
        .expect("flow should validate")
}

fn pass_id_by_label(flow: &RenderFlow, label: &str) -> RenderPassId {
    flow.graph()
        .passes
        .passes
        .iter()
        .find(|pass| pass.label == label)
        .map(|pass| pass.id)
        .expect("pass label should exist")
}

#[test]
fn v2_flow_keeps_graph_contract_inspectable() {
    let flow = build_flow();
    let report = flow.validation_report().expect("report should validate");
    let pass_labels_by_id = flow
        .graph()
        .passes
        .passes
        .iter()
        .map(|pass| (pass.id, pass.label.as_str()))
        .collect::<BTreeMap<_, _>>();
    let ordered_labels = report
        .pass_order
        .iter()
        .map(|id| pass_labels_by_id.get(id).copied().expect("pass should exist"))
        .collect::<Vec<_>>();
    assert_eq!(ordered_labels, vec!["simulate", "compose", "ui"]);

    let simulate = flow
        .graph()
        .passes
        .passes
        .iter()
        .find(|pass| pass.label == "simulate")
        .expect("simulate pass should exist");
    assert_eq!(simulate.kind, RenderPassKind::Compute);
    let read_ids = simulate.reads.iter().copied().collect::<BTreeSet<_>>();
    let write_ids = simulate.writes.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(read_ids.len(), 2);
    assert_eq!(write_ids.len(), 2);
    assert_eq!(read_ids, write_ids);
}

#[test]
fn v2_uniform_projection_uses_state_bindings() {
    let flow = build_flow();
    let state = FlowState::default();
    let frame_data = RenderFrameDataRegistry::new().with(&state);
    let projections = flow
        .project_uniforms(&frame_data, (1280, 720))
        .expect("projection should succeed");
    assert!(projections.pass(pass_id_by_label(&flow, "simulate")).is_some());
    assert!(projections.pass(pass_id_by_label(&flow, "compose")).is_some());
}

#[test]
fn v2_uniform_projection_infers_types_from_method_items() {
    let flow = RenderFlow::new("v2.inference")
        .with_state::<FlowState>()
        .with_surface_color()
        .double_buffer_storage_array::<Cell>("cells", 16 * 9)
        .compute_pass("simulate")
        .uniform_from_state(FlowState::compute_params)
        .bind_ping_pong_storage("cells")
        .dispatch_from_state(FlowState::dispatch)
        .finish()
        .fullscreen_pass("compose")
        .uniform_from_state_with_surface(FlowState::compose_params)
        .bind_ping_pong_storage("cells")
        .write_surface_color()
        .depends_on("simulate")
        .finish()
        .validate()
        .expect("flow should validate");

    let state = FlowState::default();
    let frame_data = RenderFrameDataRegistry::new().with(&state);
    let projections = flow
        .project_uniforms(&frame_data, (1920, 1080))
        .expect("projection should succeed");
    assert!(projections.pass(pass_id_by_label(&flow, "simulate")).is_some());
    assert!(projections.pass(pass_id_by_label(&flow, "compose")).is_some());
}

#[test]
fn shader_registry_supports_asset_and_explicit_registration() {
    let mut registry = ShaderRegistryResource::default();
    let a = registry.register_shader("assets/shaders/game_of_life_compute.wgsl");
    let b = registry
        .register_shader_with_id("custom.compose", "assets/shaders/game_of_life_compose.wgsl");
    assert!(registry.shader_count() >= 2);
    assert_eq!(
        registry.handle("custom.compose"),
        Some(b),
        "explicit id should resolve to registered shader handle"
    );
    assert_ne!(a, b);
}
