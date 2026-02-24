use anyhow::{Result, anyhow};
use bytemuck::{Pod, Zeroable};
use engine::plugins::render::domain::{
    BuiltinRenderPassExecutor, MAX_WORLD_RENDER_AGENTS, MAX_WORLD_RENDER_MODELS,
    RenderFeatureGraphSpec, RenderPassEncodeContext, RenderPassExecutor,
    RenderPassExecutorRegistryResource, RenderPassPrepareContext, WorldRenderFrame,
};
use engine::runtime::{EngineData, EnginePlugin, EngineScheduleBuilder};
use engine::{platform::App, plugins::input::domain::action};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use wgpu::*;
use winit::keyboard::KeyCode;

const ACTION_UP: &str = "sdf.move_up";
const ACTION_DOWN: &str = "sdf.move_down";
const ACTION_DEBUG_NEXT: &str = "sdf.debug_next";
const ACTION_DEBUG_PREV: &str = "sdf.debug_prev";
const ACTION_SPEED_UP: &str = "sdf.speed_up";
const ACTION_SPEED_DOWN: &str = "sdf.speed_down";

const SDF_ASSETS_DIR_PRIMARY: &str = "engine/examples/sdf_renderer/assets";
const SDF_ASSETS_DIR_FALLBACK: &str = "examples/sdf_renderer/assets";

const PARAMS_CONFIG_FILE: &str = "sdf_params.ron";
const INPUT_BINDINGS_CONFIG_FILE: &str = "input_bindings.ron";
const RENDER_GRAPH_CONFIG_FILE: &str = "render_graph.ron";
const SDF_COMPUTE_SHADER: &str =
    include_str!("../../../assets/shaders/sdf_compute_3d_example.wgsl");
const SDF_COMPOSE_SHADER: &str =
    include_str!("../../../assets/shaders/world_compose_fullscreen.wgsl");

static SDF_CONTROLS: OnceLock<SdfControlsConfig> = OnceLock::new();

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
        let params_config = load_config_with_default::<SdfParamsConfig>(PARAMS_CONFIG_FILE);
        apply_sdf_params(data, &params_config);
        let _ = SDF_CONTROLS.set(params_config.controls);

        let input_bindings =
            load_config_with_default::<SdfInputBindingsConfig>(INPUT_BINDINGS_CONFIG_FILE);
        let applied_bindings = apply_input_bindings(data, &input_bindings);

        let render_graph_config =
            load_config_with_default::<SdfRenderGraphConfig>(RENDER_GRAPH_CONFIG_FILE);
        let (active_render_graph_config, spec) = match render_graph_config.to_spec() {
            Ok(spec) => (render_graph_config.clone(), spec),
            Err(err) => {
                tracing::error!(
                    config = RENDER_GRAPH_CONFIG_FILE,
                    ?err,
                    "invalid sdf render graph config; using built-in defaults"
                );
                let fallback = SdfRenderGraphConfig::default();
                let spec = fallback.to_spec()?;
                (fallback, spec)
            }
        };
        data.render_graph_registry.register_feature_graph(spec);

        let bound_executors = match active_render_graph_config
            .register_custom_executors(&mut data.render_executor_registry)
        {
            Ok(count) => count,
            Err(err) => {
                tracing::error!(
                    config = RENDER_GRAPH_CONFIG_FILE,
                    ?err,
                    "invalid sdf executor bindings; using built-in defaults"
                );
                let fallback = SdfRenderGraphConfig::default();
                fallback.register_custom_executors(&mut data.render_executor_registry)?
            }
        };

        tracing::info!(
            bindings = applied_bindings,
            graph_passes = active_render_graph_config.passes.len(),
            graph_compute_pipelines = active_render_graph_config.compute_pipelines.len(),
            graph_render_pipelines = active_render_graph_config.render_builtin_pipelines.len(),
            executor_bindings = bound_executors,
            "sdf renderer setup applied"
        );

        Ok(())
    }
}

fn sdf_renderer_example_update_system(data: &mut EngineData) -> Result<()> {
    data.world_render.render_world = false;
    data.world_render.agents.clear();
    data.world_render.model_proxies.clear();
    let controls = SDF_CONTROLS.get().copied().unwrap_or_default();

    if data.input.toggle_pause_menu {
        data.world_render.world_paused = !data.world_render.world_paused;
    }

    if !data.world_render.world_paused {
        data.world_render.elapsed_time_seconds += data.time.delta_seconds.max(0.0);
    }

    if data.input.left_mouse_down() {
        data.world_render.camera_yaw -=
            data.input.mouse_delta.0 * controls.mouse_rotate_sensitivity;
        data.world_render.camera_pitch -=
            data.input.mouse_delta.1 * controls.mouse_rotate_sensitivity;
    }

    if data.input.scroll_delta.abs() > f32::EPSILON {
        data.world_render.camera_distance -=
            data.input.scroll_delta * controls.scroll_zoom_sensitivity;
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

    let mut speed = controls.base_move_speed;
    if data.input.action_down(ACTION_SPEED_UP) {
        speed *= controls.speed_up_multiplier;
    }
    if data.input.action_down(ACTION_SPEED_DOWN) {
        speed *= controls.speed_down_multiplier;
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

    data.world_render.camera_target[1] = data.world_render.camera_target[1]
        .clamp(controls.camera_target_y_min, controls.camera_target_y_max);

    if data.input.action_pressed(ACTION_DEBUG_NEXT) {
        data.world_render.debug_view_mode = (data.world_render.debug_view_mode + 1) % 4;
    }
    if data.input.action_pressed(ACTION_DEBUG_PREV) {
        data.world_render.debug_view_mode = (data.world_render.debug_view_mode + 3) % 4;
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SdfParamsConfig {
    world_scene_label: String,
    overlay_scene_label: String,
    world_bounds: [f32; 4],
    camera: SdfCameraConfig,
    controls: SdfControlsConfig,
    debug_view_mode: u32,
    world_paused: bool,
    render_mesh_overlay: bool,
}

impl Default for SdfParamsConfig {
    fn default() -> Self {
        Self {
            world_scene_label: "gameplay_stub".to_string(),
            overlay_scene_label: "console_ui".to_string(),
            world_bounds: [-18.0, -18.0, 18.0, 18.0],
            camera: SdfCameraConfig::default(),
            controls: SdfControlsConfig::default(),
            debug_view_mode: 0,
            world_paused: false,
            render_mesh_overlay: false,
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default)]
struct SdfCameraConfig {
    target: [f32; 3],
    yaw: f32,
    pitch: f32,
    distance: f32,
    pitch_min: f32,
    pitch_max: f32,
    distance_min: f32,
    distance_max: f32,
    fov_y_radians: f32,
}

impl Default for SdfCameraConfig {
    fn default() -> Self {
        Self {
            target: [0.0, 0.8, 0.0],
            yaw: 0.4,
            pitch: 0.25,
            distance: 9.5,
            pitch_min: -1.2,
            pitch_max: 1.2,
            distance_min: 2.0,
            distance_max: 30.0,
            fov_y_radians: 58.0f32.to_radians(),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default)]
struct SdfControlsConfig {
    base_move_speed: f32,
    speed_up_multiplier: f32,
    speed_down_multiplier: f32,
    mouse_rotate_sensitivity: f32,
    scroll_zoom_sensitivity: f32,
    camera_target_y_min: f32,
    camera_target_y_max: f32,
}

impl Default for SdfControlsConfig {
    fn default() -> Self {
        Self {
            base_move_speed: 7.5,
            speed_up_multiplier: 2.0,
            speed_down_multiplier: 0.35,
            mouse_rotate_sensitivity: 0.0045,
            scroll_zoom_sensitivity: 0.55,
            camera_target_y_min: -4.0,
            camera_target_y_max: 8.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SdfInputBindingsConfig {
    bindings: Vec<SdfInputBindingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
struct SdfInputBindingConfig {
    action: String,
    key: String,
}

impl Default for SdfInputBindingConfig {
    fn default() -> Self {
        Self {
            action: String::new(),
            key: String::new(),
        }
    }
}

impl Default for SdfInputBindingsConfig {
    fn default() -> Self {
        Self {
            bindings: vec![
                SdfInputBindingConfig {
                    action: ACTION_UP.to_string(),
                    key: "KeyR".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_DOWN.to_string(),
                    key: "KeyF".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_DEBUG_NEXT.to_string(),
                    key: "Tab".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_DEBUG_PREV.to_string(),
                    key: "Backquote".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_SPEED_UP.to_string(),
                    key: "KeyE".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_SPEED_DOWN.to_string(),
                    key: "KeyQ".to_string(),
                },
            ],
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SdfPassKindConfig {
    Compute,
    Render,
}

impl Default for SdfPassKindConfig {
    fn default() -> Self {
        Self::Compute
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SdfRenderGraphConfig {
    feature: String,
    resources: Vec<String>,
    compute_pipelines: Vec<SdfComputePipelineConfig>,
    render_builtin_pipelines: Vec<SdfRenderBuiltinPipelineConfig>,
    executor_bindings: Vec<SdfExecutorBindingConfig>,
    passes: Vec<SdfRenderPassConfig>,
}

impl SdfRenderGraphConfig {
    fn to_spec(&self) -> Result<RenderFeatureGraphSpec> {
        let mut builder = RenderFeatureGraphSpec::builder(self.feature.as_str());

        for resource in &self.resources {
            builder = builder.resource(resource.as_str());
        }
        for pipeline in &self.compute_pipelines {
            builder = builder.pipeline_compute(pipeline.id.as_str(), pipeline.shader.as_str());
        }
        for pipeline in &self.render_builtin_pipelines {
            builder =
                builder.pipeline_render_builtin(pipeline.id.as_str(), pipeline.builtin.as_str());
        }

        for pass in &self.passes {
            let mut pass_builder = match pass.kind {
                SdfPassKindConfig::Compute => builder.compute_pass(pass.id.as_str()),
                SdfPassKindConfig::Render => builder.render_pass(pass.id.as_str()),
            };
            pass_builder = pass_builder.pipeline(pass.pipeline.as_str());
            if pass.executor.trim().is_empty() {
                return Err(anyhow!("render pass '{}' has empty executor", pass.id));
            }
            pass_builder = pass_builder.executor(pass.executor.as_str());
            if !pass.reads.is_empty() {
                pass_builder = pass_builder.reads(pass.reads.iter().map(String::as_str));
            }
            if !pass.writes.is_empty() {
                pass_builder = pass_builder.writes(pass.writes.iter().map(String::as_str));
            }
            if !pass.depends_on.is_empty() {
                pass_builder = pass_builder.depends_on(pass.depends_on.iter().map(String::as_str));
            }
            builder = pass_builder.finish();
        }

        builder.build()
    }

    fn register_custom_executors(
        &self,
        registry: &mut RenderPassExecutorRegistryResource,
    ) -> Result<usize> {
        let shared_state = Arc::new(Mutex::new(SdfGpuSharedState::default()));
        let mut applied = 0usize;
        for binding in &self.executor_bindings {
            let executor_id = binding.id.trim();
            if executor_id.is_empty() {
                return Err(anyhow!("executor binding id must not be empty"));
            }
            let builtin = parse_builtin_executor(binding.builtin.as_str()).ok_or_else(|| {
                anyhow!(
                    "executor binding '{}' uses unknown builtin '{}'",
                    executor_id,
                    binding.builtin.trim()
                )
            })?;
            let executor: Arc<dyn RenderPassExecutor> = match builtin {
                BuiltinRenderPassExecutor::Compute => {
                    Arc::new(SdfComputeExecutor::new(Arc::clone(&shared_state)))
                }
                BuiltinRenderPassExecutor::Compose => {
                    Arc::new(SdfComposeExecutor::new(Arc::clone(&shared_state)))
                }
                BuiltinRenderPassExecutor::UiComposite => Arc::new(SdfUiCompositeExecutor),
                BuiltinRenderPassExecutor::MeshOverlay => {
                    Arc::new(BuiltinDelegatingExecutor { builtin })
                }
            };
            registry.register_custom(executor_id.to_string(), executor);
            applied = applied.saturating_add(1);
        }
        Ok(applied)
    }
}

impl Default for SdfRenderGraphConfig {
    fn default() -> Self {
        Self {
            feature: "sdf_renderer_example".to_string(),
            resources: vec![
                "sdf.params".to_string(),
                "world.agents".to_string(),
                "sdf.color".to_string(),
                "surface.color".to_string(),
                "ui.draw_list".to_string(),
            ],
            compute_pipelines: vec![SdfComputePipelineConfig {
                id: "sdf.compute.raymarch".to_string(),
                shader: "assets/shaders/sdf_compute_3d_example.wgsl".to_string(),
            }],
            render_builtin_pipelines: vec![
                SdfRenderBuiltinPipelineConfig {
                    id: "sdf.compose.fullscreen".to_string(),
                    builtin: "compose.fullscreen".to_string(),
                },
                SdfRenderBuiltinPipelineConfig {
                    id: "ui.compose".to_string(),
                    builtin: "ui.composite".to_string(),
                },
            ],
            executor_bindings: vec![
                SdfExecutorBindingConfig {
                    id: "sdf.compute".to_string(),
                    builtin: "builtin_compute".to_string(),
                },
                SdfExecutorBindingConfig {
                    id: "sdf.compose".to_string(),
                    builtin: "builtin_compose".to_string(),
                },
                SdfExecutorBindingConfig {
                    id: "ui_composite".to_string(),
                    builtin: "builtin_ui_composite".to_string(),
                },
            ],
            passes: vec![
                SdfRenderPassConfig {
                    id: "sdf.compute".to_string(),
                    kind: SdfPassKindConfig::Compute,
                    pipeline: "sdf.compute.raymarch".to_string(),
                    executor: "sdf.compute".to_string(),
                    reads: vec!["sdf.params".to_string(), "world.agents".to_string()],
                    writes: vec!["sdf.color".to_string()],
                    depends_on: Vec::new(),
                },
                SdfRenderPassConfig {
                    id: "sdf.compose".to_string(),
                    kind: SdfPassKindConfig::Render,
                    pipeline: "sdf.compose.fullscreen".to_string(),
                    executor: "sdf.compose".to_string(),
                    reads: vec!["sdf.color".to_string()],
                    writes: vec!["surface.color".to_string()],
                    depends_on: vec!["sdf.compute".to_string()],
                },
                SdfRenderPassConfig {
                    id: "ui_composite".to_string(),
                    kind: SdfPassKindConfig::Render,
                    pipeline: "ui.compose".to_string(),
                    executor: "ui_composite".to_string(),
                    reads: vec!["ui.draw_list".to_string()],
                    writes: vec!["surface.color".to_string()],
                    depends_on: vec!["sdf.compose".to_string()],
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SdfExecutorBindingConfig {
    id: String,
    builtin: String,
}

impl Default for SdfExecutorBindingConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            builtin: String::new(),
        }
    }
}

#[derive(Debug)]
struct BuiltinDelegatingExecutor {
    builtin: BuiltinRenderPassExecutor,
}

impl RenderPassExecutor for BuiltinDelegatingExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        ctx.run_builtin(self.builtin)
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        ctx.run_builtin(self.builtin)
    }
}

const SDF_CLEAR_COLOR: Color = Color {
    r: 0.02,
    g: 0.02,
    b: 0.03,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SdfWorldParamsRaw {
    screen_size: [f32; 2],
    _pad0: [f32; 2],
    world_min: [f32; 2],
    _pad1: [f32; 2],
    world_max: [f32; 2],
    _pad2: [f32; 2],
    agent_count: u32,
    model_count: u32,
    paused: u32,
    _pad3: u32,
    camera_target_time: [f32; 4],
    camera_orbit: [f32; 4],
    debug_view_mode: u32,
    _pad4: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SdfWorldAgentRaw {
    pos: [f32; 2],
    radius: f32,
    health: f32,
    team: u32,
    _pad0: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SdfWorldModelRaw {
    pos: [f32; 2],
    radius: f32,
    _pad0: f32,
    color: [f32; 4],
}

#[derive(Default)]
struct SdfGpuSharedState {
    pass: Option<SdfGpuPass>,
}

struct SdfGpuPass {
    surface_format: TextureFormat,
    size: (u32, u32),
    params_buffer: Buffer,
    agents_buffer: Buffer,
    models_buffer: Buffer,
    compute_bind_group: BindGroup,
    compose_bind_group: BindGroup,
    compute_pipeline: ComputePipeline,
    compose_pipeline: RenderPipeline,
    _world_texture: Texture,
    _world_texture_view: TextureView,
    _world_sampler: Sampler,
}

fn build_sdf_gpu_pass(
    device: &Device,
    surface_format: TextureFormat,
    size: (u32, u32),
) -> SdfGpuPass {
    let compute_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("sdf_example_compute_shader"),
        source: ShaderSource::Wgsl(SDF_COMPUTE_SHADER.into()),
    });
    let compose_shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("sdf_example_compose_shader"),
        source: ShaderSource::Wgsl(SDF_COMPOSE_SHADER.into()),
    });

    let params_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_params_buffer"),
        size: std::mem::size_of::<SdfWorldParamsRaw>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let agents_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_agents_buffer"),
        size: (std::mem::size_of::<SdfWorldAgentRaw>() * MAX_WORLD_RENDER_AGENTS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let models_buffer = device.create_buffer(&BufferDescriptor {
        label: Some("sdf_example_models_buffer"),
        size: (std::mem::size_of::<SdfWorldModelRaw>() * MAX_WORLD_RENDER_MODELS) as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let world_texture = device.create_texture(&TextureDescriptor {
        label: Some("sdf_example_world_color"),
        size: Extent3d {
            width: size.0.max(1),
            height: size.1.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let world_texture_view = world_texture.create_view(&TextureViewDescriptor::default());
    let world_sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("sdf_example_world_sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    });

    let compute_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("sdf_example_compute_bind_group_layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::StorageTexture {
                    access: StorageTextureAccess::WriteOnly,
                    format: TextureFormat::Rgba8Unorm,
                    view_dimension: TextureViewDimension::D2,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 3,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });
    let compute_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("sdf_example_compute_bind_group"),
        layout: &compute_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&world_texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: params_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: agents_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: models_buffer.as_entire_binding(),
            },
        ],
    });
    let compute_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("sdf_example_compute_pipeline_layout"),
        bind_group_layouts: &[&compute_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
        label: Some("sdf_example_compute_pipeline"),
        layout: Some(&compute_pipeline_layout),
        module: &compute_shader,
        entry_point: Some("cs_main"),
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    });

    let compose_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("sdf_example_compose_bind_group_layout"),
        entries: &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let compose_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("sdf_example_compose_bind_group"),
        layout: &compose_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&world_texture_view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&world_sampler),
            },
        ],
    });
    let compose_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
        label: Some("sdf_example_compose_pipeline_layout"),
        bind_group_layouts: &[&compose_bind_group_layout],
        push_constant_ranges: &[],
    });
    let compose_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some("sdf_example_compose_pipeline"),
        layout: Some(&compose_pipeline_layout),
        vertex: VertexState {
            module: &compose_shader,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(FragmentState {
            module: &compose_shader,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: surface_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL,
            })],
        }),
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    SdfGpuPass {
        surface_format,
        size,
        params_buffer,
        agents_buffer,
        models_buffer,
        compute_bind_group,
        compose_bind_group,
        compute_pipeline,
        compose_pipeline,
        _world_texture: world_texture,
        _world_texture_view: world_texture_view,
        _world_sampler: world_sampler,
    }
}

fn ensure_sdf_gpu_pass(
    shared: &mut SdfGpuSharedState,
    device: &Device,
    surface_format: TextureFormat,
    surface_size: (u32, u32),
) {
    let size = (surface_size.0.max(1), surface_size.1.max(1));
    let needs_rebuild = shared
        .pass
        .as_ref()
        .is_none_or(|pass| pass.surface_format != surface_format || pass.size != size);
    if needs_rebuild {
        shared.pass = Some(build_sdf_gpu_pass(device, surface_format, size));
    }
}

struct SdfComputeExecutor {
    shared: Arc<Mutex<SdfGpuSharedState>>,
}

impl SdfComputeExecutor {
    fn new(shared: Arc<Mutex<SdfGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for SdfComputeExecutor {
    fn prepare(&self, ctx: &mut RenderPassPrepareContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compute shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compute pass unavailable after setup"))?;

        let world_frame = ctx
            .frame_data::<WorldRenderFrame>()
            .ok_or_else(|| anyhow!("missing WorldRenderFrame in render pass prepare context"))?;
        let agent_count = world_frame.agents.len().min(MAX_WORLD_RENDER_AGENTS);
        let model_count = world_frame.model_proxies.len().min(MAX_WORLD_RENDER_MODELS);
        let params = SdfWorldParamsRaw {
            screen_size: [pass.size.0 as f32, pass.size.1 as f32],
            _pad0: [0.0; 2],
            world_min: [world_frame.world_bounds[0], world_frame.world_bounds[1]],
            _pad1: [0.0; 2],
            world_max: [world_frame.world_bounds[2], world_frame.world_bounds[3]],
            _pad2: [0.0; 2],
            agent_count: agent_count as u32,
            model_count: model_count as u32,
            paused: u32::from(world_frame.world_paused),
            _pad3: 0,
            camera_target_time: [
                world_frame.camera_target[0],
                world_frame.camera_target[1],
                world_frame.camera_target[2],
                world_frame.elapsed_time_seconds.max(0.0),
            ],
            camera_orbit: [
                world_frame.camera_yaw,
                world_frame.camera_pitch,
                world_frame.camera_distance.max(0.1),
                world_frame
                    .camera_fov_y
                    .clamp(0.1, std::f32::consts::PI - 0.1),
            ],
            debug_view_mode: world_frame.debug_view_mode,
            _pad4: [0; 3],
        };
        ctx.queue()
            .write_buffer(&pass.params_buffer, 0, bytemuck::bytes_of(&params));

        let mut agents = Vec::with_capacity(agent_count);
        for agent in world_frame.agents.iter().take(agent_count) {
            agents.push(SdfWorldAgentRaw {
                pos: [agent.x, agent.y],
                radius: agent.radius.max(0.2),
                health: agent.health_ratio.clamp(0.0, 1.0),
                team: agent.team,
                _pad0: [0; 3],
            });
        }
        if !agents.is_empty() {
            ctx.queue()
                .write_buffer(&pass.agents_buffer, 0, bytemuck::cast_slice(&agents));
        }

        let mut models = Vec::with_capacity(model_count);
        for model in world_frame.model_proxies.iter().take(model_count) {
            models.push(SdfWorldModelRaw {
                pos: [model.x, model.y],
                radius: model.radius.max(0.2),
                _pad0: 0.0,
                color: model.color,
            });
        }
        if !models.is_empty() {
            ctx.queue()
                .write_buffer(&pass.models_buffer, 0, bytemuck::cast_slice(&models));
        }
        Ok(())
    }

    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compute shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compute pass unavailable during encode"))?;
        let mut compute = ctx.encoder().begin_compute_pass(&ComputePassDescriptor {
            label: Some("sdf_example.compute"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&pass.compute_pipeline);
        compute.set_bind_group(0, &pass.compute_bind_group, &[]);
        compute.dispatch_workgroups(pass.size.0.div_ceil(8), pass.size.1.div_ceil(8), 1);
        Ok(())
    }
}

struct SdfComposeExecutor {
    shared: Arc<Mutex<SdfGpuSharedState>>,
}

impl SdfComposeExecutor {
    fn new(shared: Arc<Mutex<SdfGpuSharedState>>) -> Self {
        Self { shared }
    }
}

impl RenderPassExecutor for SdfComposeExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        let mut shared = self
            .shared
            .lock()
            .map_err(|_| anyhow!("sdf compose shared gpu state lock poisoned"))?;
        ensure_sdf_gpu_pass(
            &mut shared,
            ctx.device(),
            ctx.surface_format(),
            ctx.surface_size(),
        );
        let pass = shared
            .pass
            .as_ref()
            .ok_or_else(|| anyhow!("sdf compose pass unavailable during encode"))?;
        let frame_view = ctx.frame_view();
        let mut compose = ctx.encoder().begin_render_pass(&RenderPassDescriptor {
            label: Some("sdf_example.compose"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: frame_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(SDF_CLEAR_COLOR),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        compose.set_pipeline(&pass.compose_pipeline);
        compose.set_bind_group(0, &pass.compose_bind_group, &[]);
        compose.draw(0..3, 0..1);
        Ok(())
    }
}

struct SdfUiCompositeExecutor;

impl RenderPassExecutor for SdfUiCompositeExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        ctx.run_ui()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SdfComputePipelineConfig {
    id: String,
    shader: String,
}

impl Default for SdfComputePipelineConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            shader: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct SdfRenderBuiltinPipelineConfig {
    id: String,
    builtin: String,
}

impl Default for SdfRenderBuiltinPipelineConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            builtin: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct SdfRenderPassConfig {
    id: String,
    kind: SdfPassKindConfig,
    pipeline: String,
    executor: String,
    reads: Vec<String>,
    writes: Vec<String>,
    depends_on: Vec<String>,
}

fn apply_sdf_params(data: &mut EngineData, params: &SdfParamsConfig) {
    data.world_render.render_world = false;
    data.world_render.world_scene_label = params.world_scene_label.clone();
    data.world_render.overlay_scene_label = params.overlay_scene_label.clone();
    data.world_render.world_bounds = params.world_bounds;
    data.world_render.camera_target = params.camera.target;
    data.world_render.camera_yaw = params.camera.yaw;
    data.world_render.camera_pitch = params.camera.pitch;
    data.world_render.camera_distance = params.camera.distance;
    data.world_render.camera_pitch_min = params.camera.pitch_min;
    data.world_render.camera_pitch_max = params.camera.pitch_max;
    data.world_render.camera_distance_min = params.camera.distance_min;
    data.world_render.camera_distance_max = params.camera.distance_max;
    data.world_render.camera_fov_y = params.camera.fov_y_radians;
    data.world_render.world_paused = params.world_paused;
    data.world_render.debug_view_mode = params.debug_view_mode;
    data.world_render.elapsed_time_seconds = 0.0;
    data.world_render.render_mesh_overlay = params.render_mesh_overlay;
    data.world_render.agents.clear();
    data.world_render.model_proxies.clear();
}

fn apply_input_bindings(data: &mut EngineData, config: &SdfInputBindingsConfig) -> usize {
    let mut applied = 0usize;
    for (index, binding) in config.bindings.iter().enumerate() {
        let action = binding.action.trim();
        if action.is_empty() {
            tracing::error!(
                index,
                key = binding.key.as_str(),
                "sdf input binding has empty action; skipping"
            );
            continue;
        }
        let Some(key) = parse_key_code(binding.key.as_str()) else {
            tracing::error!(
                index,
                action,
                key = binding.key.as_str(),
                "invalid sdf input key code; skipping binding"
            );
            continue;
        };
        data.input.map_key(action.to_string(), key);
        applied = applied.saturating_add(1);
    }
    applied
}

fn parse_key_code(raw: &str) -> Option<KeyCode> {
    let token = raw.trim();
    if token.is_empty() {
        return None;
    }
    let normalized = token.to_ascii_lowercase();
    match normalized.as_str() {
        "arrowleft" | "left" => Some(KeyCode::ArrowLeft),
        "arrowright" | "right" => Some(KeyCode::ArrowRight),
        "arrowup" | "up" => Some(KeyCode::ArrowUp),
        "arrowdown" | "down" => Some(KeyCode::ArrowDown),
        "tab" => Some(KeyCode::Tab),
        "backquote" | "backtick" | "`" => Some(KeyCode::Backquote),
        "escape" | "esc" => Some(KeyCode::Escape),
        "space" => Some(KeyCode::Space),
        "enter" => Some(KeyCode::Enter),
        "numpadenter" => Some(KeyCode::NumpadEnter),
        "shiftleft" => Some(KeyCode::ShiftLeft),
        "shiftright" => Some(KeyCode::ShiftRight),
        "controlleft" => Some(KeyCode::ControlLeft),
        "controlright" => Some(KeyCode::ControlRight),
        "altleft" => Some(KeyCode::AltLeft),
        "altright" => Some(KeyCode::AltRight),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "delete" => Some(KeyCode::Delete),
        "backspace" => Some(KeyCode::Backspace),
        _ => parse_key_code_compact(token),
    }
}

fn parse_key_code_compact(token: &str) -> Option<KeyCode> {
    if let Some(rest) = token
        .strip_prefix("Key")
        .or_else(|| token.strip_prefix("key"))
        && rest.len() == 1
    {
        return parse_letter_key(rest.chars().next().expect("checked len"));
    }
    if let Some(rest) = token
        .strip_prefix("Digit")
        .or_else(|| token.strip_prefix("digit"))
        && rest.len() == 1
    {
        return parse_digit_key(rest.chars().next().expect("checked len"));
    }
    if token.len() == 1 {
        let ch = token.chars().next().expect("checked len");
        return parse_letter_key(ch).or_else(|| parse_digit_key(ch));
    }
    if let Some(rest) = token.strip_prefix('F').or_else(|| token.strip_prefix('f')) {
        return parse_function_key(rest);
    }
    None
}

fn parse_letter_key(ch: char) -> Option<KeyCode> {
    match ch.to_ascii_uppercase() {
        'A' => Some(KeyCode::KeyA),
        'B' => Some(KeyCode::KeyB),
        'C' => Some(KeyCode::KeyC),
        'D' => Some(KeyCode::KeyD),
        'E' => Some(KeyCode::KeyE),
        'F' => Some(KeyCode::KeyF),
        'G' => Some(KeyCode::KeyG),
        'H' => Some(KeyCode::KeyH),
        'I' => Some(KeyCode::KeyI),
        'J' => Some(KeyCode::KeyJ),
        'K' => Some(KeyCode::KeyK),
        'L' => Some(KeyCode::KeyL),
        'M' => Some(KeyCode::KeyM),
        'N' => Some(KeyCode::KeyN),
        'O' => Some(KeyCode::KeyO),
        'P' => Some(KeyCode::KeyP),
        'Q' => Some(KeyCode::KeyQ),
        'R' => Some(KeyCode::KeyR),
        'S' => Some(KeyCode::KeyS),
        'T' => Some(KeyCode::KeyT),
        'U' => Some(KeyCode::KeyU),
        'V' => Some(KeyCode::KeyV),
        'W' => Some(KeyCode::KeyW),
        'X' => Some(KeyCode::KeyX),
        'Y' => Some(KeyCode::KeyY),
        'Z' => Some(KeyCode::KeyZ),
        _ => None,
    }
}

fn parse_digit_key(ch: char) -> Option<KeyCode> {
    match ch {
        '0' => Some(KeyCode::Digit0),
        '1' => Some(KeyCode::Digit1),
        '2' => Some(KeyCode::Digit2),
        '3' => Some(KeyCode::Digit3),
        '4' => Some(KeyCode::Digit4),
        '5' => Some(KeyCode::Digit5),
        '6' => Some(KeyCode::Digit6),
        '7' => Some(KeyCode::Digit7),
        '8' => Some(KeyCode::Digit8),
        '9' => Some(KeyCode::Digit9),
        _ => None,
    }
}

fn parse_function_key(suffix: &str) -> Option<KeyCode> {
    match suffix {
        "1" => Some(KeyCode::F1),
        "2" => Some(KeyCode::F2),
        "3" => Some(KeyCode::F3),
        "4" => Some(KeyCode::F4),
        "5" => Some(KeyCode::F5),
        "6" => Some(KeyCode::F6),
        "7" => Some(KeyCode::F7),
        "8" => Some(KeyCode::F8),
        "9" => Some(KeyCode::F9),
        "10" => Some(KeyCode::F10),
        "11" => Some(KeyCode::F11),
        "12" => Some(KeyCode::F12),
        _ => None,
    }
}

fn parse_builtin_executor(raw: &str) -> Option<BuiltinRenderPassExecutor> {
    BuiltinRenderPassExecutor::from_label(raw)
}

fn load_config_with_default<T>(file_name: &str) -> T
where
    T: DeserializeOwned + Default,
{
    let config_path = find_config_path(file_name);
    let raw = match fs::read_to_string(&config_path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                config = file_name,
                path = config_path.display().to_string(),
                "sdf config file missing; using built-in defaults"
            );
            return T::default();
        }
        Err(err) => {
            tracing::error!(
                config = file_name,
                path = config_path.display().to_string(),
                ?err,
                "sdf config file read failed; using built-in defaults"
            );
            return T::default();
        }
    };

    match ron::from_str::<T>(&raw) {
        Ok(parsed) => parsed,
        Err(err) => {
            tracing::error!(
                config = file_name,
                path = config_path.display().to_string(),
                ?err,
                "sdf config parse failed; using built-in defaults"
            );
            T::default()
        }
    }
}

fn find_config_path(file_name: &str) -> PathBuf {
    let primary = Path::new(SDF_ASSETS_DIR_PRIMARY).join(file_name);
    if primary.exists() {
        return primary;
    }
    let fallback = Path::new(SDF_ASSETS_DIR_FALLBACK).join(file_name);
    if fallback.exists() {
        return fallback;
    }
    primary
}

#[cfg(test)]
mod tests {
    use super::{SdfRenderGraphConfig, parse_builtin_executor, parse_key_code};
    use engine::plugins::render::domain::RenderPassExecutorRegistryResource;
    use winit::keyboard::KeyCode;

    #[test]
    fn key_code_parser_accepts_named_and_compact_forms() {
        assert_eq!(parse_key_code("KeyR"), Some(KeyCode::KeyR));
        assert_eq!(parse_key_code("R"), Some(KeyCode::KeyR));
        assert_eq!(parse_key_code("Digit2"), Some(KeyCode::Digit2));
        assert_eq!(parse_key_code("2"), Some(KeyCode::Digit2));
        assert_eq!(parse_key_code("F10"), Some(KeyCode::F10));
        assert_eq!(parse_key_code("ArrowLeft"), Some(KeyCode::ArrowLeft));
        assert_eq!(parse_key_code("Backquote"), Some(KeyCode::Backquote));
        assert_eq!(parse_key_code("unknown"), None);
    }

    #[test]
    fn default_render_graph_config_converts_to_spec() {
        let spec = SdfRenderGraphConfig::default()
            .to_spec()
            .expect("default render graph config should convert to a typed spec");
        assert_eq!(spec.feature.as_str(), "sdf_renderer_example");
        assert_eq!(spec.passes.len(), 3);
    }

    #[test]
    fn builtin_executor_parser_accepts_builtin_labels() {
        assert_eq!(
            parse_builtin_executor("builtin_compute"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::Compute)
        );
        assert_eq!(
            parse_builtin_executor("builtin_compose"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::Compose)
        );
        assert_eq!(
            parse_builtin_executor("builtin_mesh_overlay"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::MeshOverlay)
        );
        assert_eq!(
            parse_builtin_executor("builtin_ui_composite"),
            Some(engine::plugins::render::domain::BuiltinRenderPassExecutor::UiComposite)
        );
        assert_eq!(parse_builtin_executor("unknown"), None);
    }

    #[test]
    fn default_render_graph_config_registers_custom_executors() {
        let config = SdfRenderGraphConfig::default();
        let mut registry = RenderPassExecutorRegistryResource::default();
        let count = config
            .register_custom_executors(&mut registry)
            .expect("default executor bindings should apply");
        assert_eq!(count, 3);
        assert!(registry.resolve_custom("sdf.compute").is_some());
        assert!(registry.resolve_custom("sdf.compose").is_some());
        assert!(registry.resolve_custom("ui_composite").is_some());
        assert_eq!(registry.resolve_builtin("sdf.compute"), None);
        assert_eq!(registry.resolve_builtin("sdf.compose"), None);
    }
}
