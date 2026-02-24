use anyhow::Result;
use engine::plugins::render::domain::{
    OwnerRenderGraphRegistration, PassSlot, PipelineKey, RegisteredPassDescriptor,
    RegisteredPipelineDescriptor, RegisteredPipelineRef,
};
use engine::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use engine::{platform::App, plugins::input::domain::action};
use winit::keyboard::KeyCode;

const ACTION_UP: &str = "sdf.move_up";
const ACTION_DOWN: &str = "sdf.move_down";
const ACTION_DEBUG_NEXT: &str = "sdf.debug_next";
const ACTION_DEBUG_PREV: &str = "sdf.debug_prev";
const ACTION_SPEED_UP: &str = "sdf.speed_up";
const ACTION_SPEED_DOWN: &str = "sdf.speed_down";

fn main() -> Result<()> {
    App::new()
        .set_title("Grotto Quest - 3D SDF Compute Renderer")
        .add_plugin(SdfRendererExamplePlugin)
        .run()
}

struct SdfRendererExamplePlugin;

impl EnginePlugin for SdfRendererExamplePlugin {
    fn name(&self) -> &'static str {
        "sdf_renderer_example"
    }

    fn configure(&self, builder: &mut EngineScheduleBuilder) -> Result<()> {
        builder.add_node_with_edges(
            "sdf_renderer_example_update",
            sdf_renderer_example_update_system,
            &["world_scene_update"],
        );
        builder.add_edge("sdf_renderer_example_update", "frame_render_submit");
        Ok(())
    }

    fn setup(&self, data: &mut EngineData) -> Result<()> {
        data.render_graph_registry.upsert_owner(
            OwnerRenderGraphRegistration::new(self.name())
                .with_pipelines(vec![
                    RegisteredPipelineDescriptor::new(
                        "sdf.compute.raymarch",
                        PipelineKey::WorldComputeSdf3d,
                    )
                    .with_slot(PassSlot::WorldCompute),
                ])
                .with_passes(vec![
                    RegisteredPassDescriptor::compute("world_compute")
                        .with_slot(PassSlot::WorldCompute)
                        .with_pipeline(RegisteredPipelineRef::Named(
                            "sdf.compute.raymarch".to_string(),
                        ))
                        .with_executor("world_compute")
                        .with_reads(["world_params", "world_agents"])
                        .with_writes(["world_color"]),
                    RegisteredPassDescriptor::render("world_compose")
                        .with_slot(PassSlot::WorldCompose)
                        .with_executor("world_compose")
                        .with_reads(["world_color"])
                        .with_writes(["surface_color"])
                        .with_dependencies(["world_compute"]),
                ]),
        );

        data.input.map_key(ACTION_UP, KeyCode::KeyR);
        data.input.map_key(ACTION_DOWN, KeyCode::KeyF);
        data.input.map_key(ACTION_DEBUG_NEXT, KeyCode::Tab);
        data.input.map_key(ACTION_DEBUG_PREV, KeyCode::Backquote);
        data.input.map_key(ACTION_SPEED_UP, KeyCode::KeyE);
        data.input.map_key(ACTION_SPEED_DOWN, KeyCode::KeyQ);

        data.world_render.render_world = true;
        data.world_render.world_scene_label = "gameplay_stub".to_string();
        data.world_render.world_bounds = [-18.0, -18.0, 18.0, 18.0];
        data.world_render.camera_target = [0.0, 0.8, 0.0];
        data.world_render.camera_yaw = 0.4;
        data.world_render.camera_pitch = 0.25;
        data.world_render.camera_distance = 9.5;
        data.world_render.camera_pitch_min = -1.2;
        data.world_render.camera_pitch_max = 1.2;
        data.world_render.camera_distance_min = 2.0;
        data.world_render.camera_distance_max = 30.0;
        data.world_render.camera_fov_y = 58.0f32.to_radians();
        data.world_render.world_paused = false;
        data.world_render.debug_view_mode = 0;
        data.world_render.elapsed_time_seconds = 0.0;
        data.world_render.render_mesh_overlay = false;
        data.world_render.agents.clear();
        data.world_render.model_proxies.clear();

        Ok(())
    }
}

fn sdf_renderer_example_update_system(data: &mut EngineData) -> Result<()> {
    data.world_render.render_world = true;
    data.world_render.agents.clear();
    data.world_render.model_proxies.clear();

    if data.input.toggle_pause_menu {
        data.world_render.world_paused = !data.world_render.world_paused;
    }

    if !data.world_render.world_paused {
        data.world_render.elapsed_time_seconds += data.time.delta_seconds.max(0.0);
    }

    if data.input.left_mouse_down() {
        let rotate_sensitivity = 0.0045;
        data.world_render.camera_yaw -= data.input.mouse_delta.0 * rotate_sensitivity;
        data.world_render.camera_pitch -= data.input.mouse_delta.1 * rotate_sensitivity;
    }

    if data.input.scroll_delta.abs() > f32::EPSILON {
        data.world_render.camera_distance -= data.input.scroll_delta * 0.55;
    }

    let min_pitch = data
        .world_render
        .camera_pitch_min
        .min(data.world_render.camera_pitch_max);
    let max_pitch = data
        .world_render
        .camera_pitch_min
        .max(data.world_render.camera_pitch_max);
    let min_distance = data
        .world_render
        .camera_distance_min
        .min(data.world_render.camera_distance_max)
        .max(0.1);
    let max_distance = data
        .world_render
        .camera_distance_min
        .max(data.world_render.camera_distance_max)
        .max(min_distance);
    data.world_render.camera_pitch = data.world_render.camera_pitch.clamp(min_pitch, max_pitch);
    data.world_render.camera_distance = data
        .world_render
        .camera_distance
        .clamp(min_distance, max_distance);

    let mut speed = 7.5;
    if data.input.action_down(ACTION_SPEED_UP) {
        speed *= 2.0;
    }
    if data.input.action_down(ACTION_SPEED_DOWN) {
        speed *= 0.35;
    }

    let move_dt = speed * data.time.delta_seconds;
    let yaw = data.world_render.camera_yaw;
    let forward = [yaw.sin(), yaw.cos()];
    let right = [forward[1], -forward[0]];

    let forward_axis = (if data.input.action_down(action::WORLD_MOVE_UP) {
        1.0
    } else {
        0.0
    }) - (if data.input.action_down(action::WORLD_MOVE_DOWN) {
        1.0
    } else {
        0.0
    });
    let strafe_axis = (if data.input.action_down(action::WORLD_MOVE_RIGHT) {
        1.0
    } else {
        0.0
    }) - (if data.input.action_down(action::WORLD_MOVE_LEFT) {
        1.0
    } else {
        0.0
    });
    let vertical_axis = (if data.input.action_down(ACTION_UP) {
        1.0
    } else {
        0.0
    }) - (if data.input.action_down(ACTION_DOWN) {
        1.0
    } else {
        0.0
    });

    data.world_render.camera_target[0] +=
        (forward[0] * forward_axis + right[0] * strafe_axis) * move_dt;
    data.world_render.camera_target[2] +=
        (forward[1] * forward_axis + right[1] * strafe_axis) * move_dt;
    data.world_render.camera_target[1] += vertical_axis * move_dt;

    data.world_render.camera_target[1] = data.world_render.camera_target[1].clamp(-4.0, 8.0);

    if data.input.action_pressed(ACTION_DEBUG_NEXT) {
        data.world_render.debug_view_mode = (data.world_render.debug_view_mode + 1) % 4;
    }
    if data.input.action_pressed(ACTION_DEBUG_PREV) {
        data.world_render.debug_view_mode = (data.world_render.debug_view_mode + 3) % 4;
    }

    Ok(())
}
