// Owner: SDF Renderer Example - Render Graph Config
use crate::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SdfComputePipelineConfig {
    pub(crate) id: String,
    pub(crate) shader: String,
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
pub(crate) struct SdfRenderBuiltinPipelineConfig {
    pub(crate) id: String,
    pub(crate) builtin: String,
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
pub(crate) struct SdfRenderPassConfig {
    pub(crate) id: String,
    pub(crate) kind: SdfPassKindConfig,
    pub(crate) pipeline: String,
    pub(crate) executor: String,
    pub(crate) reads: Vec<String>,
    pub(crate) writes: Vec<String>,
    pub(crate) depends_on: Vec<String>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SdfPassKindConfig {
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
pub(crate) struct SdfRenderGraphConfig {
    pub(crate) feature: String,
    pub(crate) resources: Vec<String>,
    pub(crate) compute_pipelines: Vec<SdfComputePipelineConfig>,
    pub(crate) render_builtin_pipelines: Vec<SdfRenderBuiltinPipelineConfig>,
    pub(crate) executor_bindings: Vec<SdfExecutorBindingConfig>,
    pub(crate) passes: Vec<SdfRenderPassConfig>,
}

impl SdfRenderGraphConfig {
    pub(crate) fn to_spec(&self) -> Result<RenderFeatureGraphSpec> {
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

    pub(crate) fn register_custom_executors(
        &self,
        registry: &mut RenderPassExecutorRegistryResource,
    ) -> Result<usize> {
        if self.executor_bindings.is_empty() {
            return Ok(0);
        }
        let shared = Arc::new(Mutex::new(SdfGpuSharedState::default()));
        let mut count = 0usize;
        for binding in &self.executor_bindings {
            let id = binding.id.trim();
            let builtin_label = binding.builtin.trim();
            if id.is_empty() || builtin_label.is_empty() {
                return Err(anyhow!(
                    "executor binding has empty id or builtin label (id='{}', builtin='{}')",
                    binding.id,
                    binding.builtin
                ));
            }
            let executor: Arc<dyn RenderPassExecutor> = match parse_builtin_executor(builtin_label) {
                Some(BuiltinRenderPassExecutor::Compute) => {
                    Arc::new(SdfComputeExecutor::new(shared.clone()))
                }
                Some(BuiltinRenderPassExecutor::Compose) => {
                    Arc::new(SdfComposeExecutor::new(shared.clone()))
                }
                Some(BuiltinRenderPassExecutor::UiComposite) => Arc::new(SdfUiCompositeExecutor),
                Some(other) => Arc::new(BuiltinDelegatingExecutor { builtin: other }),
                None => {
                    return Err(anyhow!(
                        "executor binding '{}' references unsupported builtin '{}'",
                        id,
                        builtin_label
                    ));
                }
            };
            registry.register_custom(id, executor);
            count = count.saturating_add(1);
        }
        Ok(count)
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
                    depends_on: vec![],
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
pub(crate) struct SdfExecutorBindingConfig {
    pub(crate) id: String,
    pub(crate) builtin: String,
}

impl Default for SdfExecutorBindingConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            builtin: String::new(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct BuiltinDelegatingExecutor {
    builtin: BuiltinRenderPassExecutor,
}

impl RenderPassExecutor for BuiltinDelegatingExecutor {
    fn encode(&self, ctx: &mut RenderPassEncodeContext<'_>) -> Result<()> {
        ctx.run_builtin(self.builtin)
    }
}

pub(crate) fn parse_builtin_executor(raw: &str) -> Option<BuiltinRenderPassExecutor> {
    BuiltinRenderPassExecutor::from_label(raw)
}
