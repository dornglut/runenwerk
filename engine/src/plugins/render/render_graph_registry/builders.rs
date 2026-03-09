use super::*;

// Owner: Engine Render Graph Registry - Builder API
#[derive(Debug, Clone)]
pub struct RenderFeatureGraphSpecBuilder {
    feature: RenderFeatureId,
    resources: Vec<RenderResourceId>,
    pipelines: Vec<RenderPipelineSpec>,
    passes: Vec<RenderPassSpec>,
    errors: Vec<String>,
}

impl RenderFeatureGraphSpecBuilder {
    pub fn new(feature: impl Into<RenderFeatureId>) -> Self {
        Self {
            feature: feature.into(),
            resources: Vec::new(),
            pipelines: Vec::new(),
            passes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn resource(mut self, id: impl Into<RenderResourceId>) -> Self {
        self.resources.push(id.into());
        self
    }

    pub fn pipeline_compute(
        mut self,
        id: impl Into<RenderPipelineId>,
        shader_path: impl Into<String>,
    ) -> Self {
        let id = id.into();
        let shader_path = shader_path.into();
        self.pipelines.push(
            RenderPipelineSpec::new(id.clone(), id.as_str().to_string().into())
                .with_shader_path(shader_path),
        );
        self
    }

    pub fn pipeline_render_builtin(
        mut self,
        id: impl Into<RenderPipelineId>,
        _builtin: impl AsRef<str>,
    ) -> Self {
        let id = id.into();
        self.pipelines.push(RenderPipelineSpec::new(
            id.clone(),
            id.as_str().to_string().into(),
        ));
        self
    }

    pub fn pipeline_builtin(mut self, id: impl Into<RenderPipelineId>, key: PipelineKey) -> Self {
        self.pipelines.push(RenderPipelineSpec::new(id, key));
        self
    }

    pub fn compute_pass(self, id: impl Into<RenderPassId>) -> RenderPassSpecBuilder {
        RenderPassSpecBuilder::new(self, RenderPassSpec::compute(id))
    }

    pub fn render_pass(self, id: impl Into<RenderPassId>) -> RenderPassSpecBuilder {
        RenderPassSpecBuilder::new(self, RenderPassSpec::render(id))
    }

    pub fn build(self) -> Result<RenderFeatureGraphSpec> {
        if !self.errors.is_empty() {
            return Err(anyhow!(self.errors.join("; ")));
        }

        let spec = RenderFeatureGraphSpec {
            feature: self.feature,
            resources: self.resources,
            pipelines: self.pipelines,
            passes: self.passes,
        };
        spec.validate()?;
        Ok(spec)
    }
}

#[derive(Debug, Clone)]
pub struct RenderPassSpecBuilder {
    parent: RenderFeatureGraphSpecBuilder,
    pass: RenderPassSpec,
}

impl RenderPassSpecBuilder {
    fn new(parent: RenderFeatureGraphSpecBuilder, pass: RenderPassSpec) -> Self {
        Self { parent, pass }
    }

    pub fn pipeline(mut self, pipeline: impl Into<RenderPipelineId>) -> Self {
        self.pass.pipeline = Some(pipeline.into());
        self
    }

    pub fn executor(mut self, executor: impl Into<RenderPassExecutorId>) -> Self {
        self.pass.executor = Some(executor.into());
        self
    }

    pub fn executor_builtin_compute(mut self) -> Self {
        self.pass.executor = Some(RenderPassExecutorId::new("builtin_compute"));
        self
    }

    pub fn executor_builtin_compose(mut self) -> Self {
        self.pass.executor = Some(RenderPassExecutorId::new("builtin_compose"));
        self
    }

    pub fn executor_builtin_ui_composite(mut self) -> Self {
        self.pass.executor = Some(RenderPassExecutorId::new("builtin_ui_composite"));
        self
    }

    pub fn reads<I, S>(mut self, reads: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<RenderResourceId>,
    {
        self.pass.reads = reads.into_iter().map(Into::into).collect();
        self
    }

    pub fn writes<I, S>(mut self, writes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<RenderResourceId>,
    {
        self.pass.writes = writes.into_iter().map(Into::into).collect();
        self
    }

    pub fn depends_on<I, S>(mut self, depends_on: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<RenderPassId>,
    {
        self.pass.depends_on = depends_on.into_iter().map(Into::into).collect();
        self
    }

    pub fn finish(mut self) -> RenderFeatureGraphSpecBuilder {
        self.parent.passes.push(self.pass);
        self.parent
    }
}


