use super::*;

// Owner: Engine Render Graph Registry - Feature Graph Spec and Validation
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RenderPassKind {
    Compute,
    Render,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipelineSpec {
    pub id: RenderPipelineId,
    pub key: PipelineKey,
    pub shader_path: Option<String>,
}

impl RenderPipelineSpec {
    pub fn new(id: impl Into<RenderPipelineId>, key: PipelineKey) -> Self {
        Self {
            id: id.into(),
            key,
            shader_path: None,
        }
    }

    pub fn with_shader_path(mut self, shader_path: impl Into<String>) -> Self {
        self.shader_path = Some(shader_path.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPassSpec {
    pub id: RenderPassId,
    pub kind: RenderPassKind,
    pub reads: Vec<RenderResourceId>,
    pub writes: Vec<RenderResourceId>,
    pub depends_on: Vec<RenderPassId>,
    pub pipeline: Option<RenderPipelineId>,
    pub executor: Option<RenderPassExecutorId>,
}

impl RenderPassSpec {
    pub fn compute(id: impl Into<RenderPassId>) -> Self {
        Self::new(id, RenderPassKind::Compute)
    }

    pub fn render(id: impl Into<RenderPassId>) -> Self {
        Self::new(id, RenderPassKind::Render)
    }

    pub fn new(id: impl Into<RenderPassId>, kind: RenderPassKind) -> Self {
        Self {
            id: id.into(),
            kind,
            reads: Vec::new(),
            writes: Vec::new(),
            depends_on: Vec::new(),
            pipeline: None,
            executor: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFeatureGraphSpec {
    pub feature: RenderFeatureId,
    pub resources: Vec<RenderResourceId>,
    pub pipelines: Vec<RenderPipelineSpec>,
    pub passes: Vec<RenderPassSpec>,
}

impl RenderFeatureGraphSpec {
    pub fn builder(feature: impl Into<RenderFeatureId>) -> RenderFeatureGraphSpecBuilder {
        RenderFeatureGraphSpecBuilder::new(feature)
    }

    pub fn validate(&self) -> Result<()> {
        let mut errors = Vec::<String>::new();
        let feature_id = self.feature.as_str().trim();
        if feature_id.is_empty() {
            errors.push("feature id must not be empty".to_string());
        }

        let mut resource_ids = BTreeSet::<String>::new();
        for resource in &self.resources {
            let id = resource.as_str().trim();
            if id.is_empty() {
                errors.push("resource id must not be empty".to_string());
                continue;
            }
            if !resource_ids.insert(id.to_string()) {
                errors.push(format!("duplicate resource id '{id}'"));
            }
        }

        let mut pipeline_ids = BTreeSet::<String>::new();
        for pipeline in &self.pipelines {
            let id = pipeline.id.as_str().trim();
            if id.is_empty() {
                errors.push("pipeline id must not be empty".to_string());
                continue;
            }
            if !pipeline_ids.insert(id.to_string()) {
                errors.push(format!("duplicate pipeline id '{id}'"));
            }
        }

        let mut pass_ids = BTreeSet::<String>::new();
        for pass in &self.passes {
            let id = pass.id.as_str().trim();
            if id.is_empty() {
                errors.push("pass id must not be empty".to_string());
                continue;
            }
            if !pass_ids.insert(id.to_string()) {
                errors.push(format!("duplicate pass id '{id}'"));
            }
        }
        if self.passes.is_empty() {
            errors.push("feature graph must declare at least one pass".to_string());
        }

        for pass in &self.passes {
            let pass_id = pass.id.as_str().trim();
            if let Some(pipeline_id) = &pass.pipeline {
                let pipeline_id = pipeline_id.as_str().trim();
                if !pipeline_ids.contains(pipeline_id) {
                    errors.push(format!(
                        "pass '{pass_id}' references unknown pipeline '{pipeline_id}'"
                    ));
                }
            }
            let executor = pass
                .executor
                .as_ref()
                .map(RenderPassExecutorId::as_str)
                .unwrap_or("");
            if executor.trim().is_empty() {
                errors.push(format!(
                    "pass '{pass_id}' must define an executor id (use .executor(...) or a builtin executor helper)"
                ));
            }
            for dep in &pass.depends_on {
                let dep_id = dep.as_str().trim();
                if dep_id == pass_id {
                    errors.push(format!("pass '{pass_id}' cannot depend on itself"));
                } else if !pass_ids.contains(dep_id) {
                    errors.push(format!(
                        "pass '{pass_id}' depends on unknown pass '{dep_id}'"
                    ));
                }
            }

            if !resource_ids.is_empty() {
                for read in &pass.reads {
                    let resource = read.as_str().trim();
                    if !resource_ids.contains(resource) {
                        errors.push(format!(
                            "pass '{pass_id}' reads unknown resource '{resource}'"
                        ));
                    }
                }
                for write in &pass.writes {
                    let resource = write.as_str().trim();
                    if !resource_ids.contains(resource) {
                        errors.push(format!(
                            "pass '{pass_id}' writes unknown resource '{resource}'"
                        ));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(errors.join("; ")))
        }
    }

    pub fn into_owner_registration(self) -> OwnerRenderGraphRegistration {
        let pipelines = self
            .pipelines
            .into_iter()
            .map(|pipeline| RegisteredPipelineDescriptor {
                id: pipeline.id.as_str().to_string(),
                key: pipeline.key,
            })
            .collect();

        let passes = self
            .passes
            .into_iter()
            .map(|pass| RegisteredPassDescriptor {
                id: pass.id.as_str().to_string(),
                kind: match pass.kind {
                    RenderPassKind::Compute => RegisteredPassKind::Compute,
                    RenderPassKind::Render => RegisteredPassKind::Render,
                },
                reads: pass
                    .reads
                    .into_iter()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                writes: pass
                    .writes
                    .into_iter()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                depends_on: pass
                    .depends_on
                    .into_iter()
                    .map(|id| id.as_str().to_string())
                    .collect(),
                pipeline: pass
                    .pipeline
                    .map(|pipeline| RegisteredPipelineRef::Named(pipeline.as_str().to_string())),
                executor: pass.executor.map(|executor| executor.as_str().to_string()),
            })
            .collect();

        OwnerRenderGraphRegistration {
            owner: self.feature.as_str().to_string(),
            pipelines,
            passes,
        }
    }
}


