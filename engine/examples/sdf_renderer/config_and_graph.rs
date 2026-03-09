// Owner: SDF Renderer Example - Config and Render Graph Wiring
use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(super) struct SdfParamsConfig {
    pub(super) world_scene_label: String,
    pub(super) overlay_scene_label: String,
    pub(super) world_bounds: [f32; 4],
    pub(super) camera: SdfCameraConfig,
    pub(super) controls: SdfControlsConfig,
    pub(super) debug_view_mode: u32,
    pub(super) world_paused: bool,
    pub(super) render_mesh_overlay: bool,
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
pub(super) struct SdfCameraConfig {
    pub(super) target: [f32; 3],
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) distance: f32,
    pub(super) pitch_min: f32,
    pub(super) pitch_max: f32,
    pub(super) distance_min: f32,
    pub(super) distance_max: f32,
    pub(super) fov_y_radians: f32,
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
pub(super) struct SdfControlsConfig {
    pub(super) base_move_speed: f32,
    pub(super) speed_up_multiplier: f32,
    pub(super) speed_down_multiplier: f32,
    pub(super) mouse_rotate_sensitivity: f32,
    pub(super) scroll_zoom_sensitivity: f32,
    pub(super) camera_target_y_min: f32,
    pub(super) camera_target_y_max: f32,
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
pub(super) struct SdfInputBindingsConfig {
    pub(super) bindings: Vec<SdfInputBindingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct SdfInputBindingConfig {
    pub(super) action: String,
    pub(super) key: String,
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
pub(super) enum SdfPassKindConfig {
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
pub(super) struct SdfRenderGraphConfig {
    pub(super) feature: String,
    pub(super) resources: Vec<String>,
    pub(super) compute_pipelines: Vec<SdfComputePipelineConfig>,
    pub(super) render_builtin_pipelines: Vec<SdfRenderBuiltinPipelineConfig>,
    pub(super) executor_bindings: Vec<SdfExecutorBindingConfig>,
    pub(super) passes: Vec<SdfRenderPassConfig>,
}

impl SdfRenderGraphConfig {
    pub(super) fn to_spec(&self) -> Result<RenderFeatureGraphSpec> {
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

    pub(super) fn register_custom_executors(
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
pub(super) struct SdfExecutorBindingConfig {
    pub(super) id: String,
    pub(super) builtin: String,
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

