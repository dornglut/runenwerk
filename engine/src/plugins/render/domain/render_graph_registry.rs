use super::PipelineKey;
use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderFeatureId(String);

impl RenderFeatureId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderFeatureId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderFeatureId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for RenderFeatureId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderResourceId(String);

impl RenderResourceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderResourceId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderResourceId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for RenderResourceId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPipelineId(String);

impl RenderPipelineId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderPipelineId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderPipelineId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for RenderPipelineId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPassId(String);

impl RenderPassId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderPassId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderPassId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for RenderPassId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RenderPassExecutorId(String);

impl RenderPassExecutorId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for RenderPassExecutorId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for RenderPassExecutorId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl AsRef<str> for RenderPassExecutorId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegisteredPassKind {
    Compute,
    Render,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisteredPipelineRef {
    Builtin(PipelineKey),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredPipelineDescriptor {
    pub id: String,
    pub key: PipelineKey,
}

impl RegisteredPipelineDescriptor {
    pub fn new(id: impl Into<String>, key: PipelineKey) -> Self {
        Self { id: id.into(), key }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredPassDescriptor {
    pub id: String,
    pub kind: RegisteredPassKind,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub depends_on: Vec<String>,
    pub pipeline: Option<RegisteredPipelineRef>,
    pub executor: Option<String>,
}

impl RegisteredPassDescriptor {
    pub fn compute(id: impl Into<String>) -> Self {
        Self::new(id, RegisteredPassKind::Compute)
    }

    pub fn render(id: impl Into<String>) -> Self {
        Self::new(id, RegisteredPassKind::Render)
    }

    pub fn new(id: impl Into<String>, kind: RegisteredPassKind) -> Self {
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

    pub fn with_reads<I, S>(mut self, reads: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.reads = reads.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_writes<I, S>(mut self, writes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.writes = writes.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_dependencies<I, S>(mut self, depends_on: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.depends_on = depends_on.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_pipeline(mut self, pipeline: RegisteredPipelineRef) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    pub fn with_executor(mut self, executor: impl Into<String>) -> Self {
        self.executor = Some(executor.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerRenderGraphRegistration {
    pub owner: String,
    pub pipelines: Vec<RegisteredPipelineDescriptor>,
    pub passes: Vec<RegisteredPassDescriptor>,
}

impl OwnerRenderGraphRegistration {
    pub fn new(owner: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            pipelines: Vec::new(),
            passes: Vec::new(),
        }
    }

    pub fn with_pipelines(mut self, pipelines: Vec<RegisteredPipelineDescriptor>) -> Self {
        self.pipelines = pipelines;
        self
    }

    pub fn with_passes(mut self, passes: Vec<RegisteredPassDescriptor>) -> Self {
        self.passes = passes;
        self
    }
}

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

pub struct RenderGraphRegistryResource {
    owners: BTreeMap<String, OwnerRenderGraphRegistration>,
    revision: u64,
}

impl std::fmt::Debug for RenderGraphRegistryResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderGraphRegistryResource")
            .field("owner_count", &self.owner_count())
            .field("revision", &self.revision())
            .finish()
    }
}

impl Default for RenderGraphRegistryResource {
    fn default() -> Self {
        Self {
            owners: BTreeMap::new(),
            revision: 0,
        }
    }
}

impl RenderGraphRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn owner_count(&self) -> usize {
        self.owners.len()
    }

    pub fn upsert_owner(&mut self, registration: OwnerRenderGraphRegistration) {
        self.owners.insert(registration.owner.clone(), registration);
        self.bump_revision();
    }

    pub fn register_feature_graph(&mut self, spec: RenderFeatureGraphSpec) {
        self.upsert_owner(spec.into_owner_registration());
    }

    pub fn replace_feature_graph(
        &mut self,
        feature: impl Into<RenderFeatureId>,
        spec: RenderFeatureGraphSpec,
    ) -> Result<()> {
        let expected = feature.into();
        if expected != spec.feature {
            bail!(
                "feature graph id mismatch: expected '{}', got '{}'",
                expected.as_str(),
                spec.feature.as_str()
            );
        }
        self.register_feature_graph(spec);
        Ok(())
    }

    pub fn clear_owner(&mut self, owner: &str) -> bool {
        let owner = owner.trim();
        if owner.is_empty() {
            return false;
        }
        let removed = self.owners.remove(owner).is_some();
        if removed {
            self.bump_revision();
        }
        removed
    }

    pub fn remove_feature_graph(&mut self, feature: impl AsRef<str>) -> bool {
        self.clear_owner(feature.as_ref())
    }

    pub fn clear(&mut self) {
        if self.owners.is_empty() {
            return;
        }
        self.owners.clear();
        self.bump_revision();
    }

    pub fn owners(&self) -> Vec<OwnerRenderGraphRegistration> {
        self.owners.values().cloned().collect()
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OwnerRenderGraphRegistration, RegisteredPassDescriptor, RegisteredPipelineDescriptor,
        RenderFeatureGraphSpec, RenderGraphRegistryResource,
    };

    #[test]
    fn upsert_owner_replaces_existing_owner_registration() {
        let mut registry = RenderGraphRegistryResource::default();
        registry.upsert_owner(
            OwnerRenderGraphRegistration::new("sdf").with_pipelines(vec![
                RegisteredPipelineDescriptor::new("sdf.compute", "world_compute_sdf_3d".into()),
            ]),
        );
        registry.upsert_owner(
            OwnerRenderGraphRegistration::new("sdf")
                .with_passes(vec![RegisteredPassDescriptor::compute("sdf_compute")]),
        );
        assert_eq!(registry.owner_count(), 1);
        let owners = registry.owners();
        let owner = &owners[0];
        assert!(owner.pipelines.is_empty());
        assert_eq!(owner.passes.len(), 1);
    }

    #[test]
    fn builder_builds_feature_graph_and_converts_to_owner_registration() {
        let spec = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.params")
            .resource("sdf.color")
            .resource("surface.color")
            .pipeline_compute(
                "sdf.compute.raymarch",
                "assets/shaders/sdf_compute_3d_example.wgsl",
            )
            .pipeline_render_builtin("sdf.compose.fullscreen", "compose.fullscreen")
            .compute_pass("sdf.compute")
            .pipeline("sdf.compute.raymarch")
            .executor_builtin_compute()
            .reads(["sdf.params"])
            .writes(["sdf.color"])
            .finish()
            .render_pass("sdf.compose")
            .pipeline("sdf.compose.fullscreen")
            .executor_builtin_compose()
            .reads(["sdf.color"])
            .writes(["surface.color"])
            .depends_on(["sdf.compute"])
            .finish()
            .build()
            .expect("feature graph should build");

        let owner = spec.clone().into_owner_registration();
        assert_eq!(owner.owner, "sdf_renderer");
        assert_eq!(owner.pipelines.len(), 2);
        assert_eq!(owner.passes.len(), 2);
        assert_eq!(owner.passes[0].id, "sdf.compute");
        assert_eq!(owner.passes[0].executor.as_deref(), Some("builtin_compute"));
    }

    #[test]
    fn builder_accepts_arbitrary_render_builtin_labels() {
        let spec = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("surface.color")
            .pipeline_render_builtin("sdf.compose.bad", "compose.unknown")
            .render_pass("sdf.compose")
            .pipeline("sdf.compose.bad")
            .executor_builtin_compose()
            .writes(["surface.color"])
            .finish()
            .build()
            .expect("arbitrary render builtin labels should be accepted");
        assert_eq!(spec.pipelines.len(), 1);
    }

    #[test]
    fn register_feature_graph_replaces_existing_registration() {
        let spec_a = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.params")
            .pipeline_compute(
                "sdf.compute.a",
                "assets/shaders/sdf_compute_3d_example.wgsl",
            )
            .compute_pass("sdf.compute")
            .pipeline("sdf.compute.a")
            .executor_builtin_compute()
            .reads(["sdf.params"])
            .finish()
            .build()
            .expect("spec_a should build");
        let spec_b = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.color")
            .pipeline_render_builtin("sdf.compose.fullscreen", "compose.fullscreen")
            .render_pass("sdf.compose")
            .pipeline("sdf.compose.fullscreen")
            .executor_builtin_compose()
            .reads(["sdf.color"])
            .finish()
            .build()
            .expect("spec_b should build");

        let mut registry = RenderGraphRegistryResource::default();
        registry.register_feature_graph(spec_a);
        registry.register_feature_graph(spec_b);

        assert_eq!(registry.owner_count(), 1);
        let owners = registry.owners();
        assert_eq!(owners[0].owner, "sdf_renderer");
        assert_eq!(owners[0].passes.len(), 1);
        assert_eq!(owners[0].passes[0].id, "sdf.compose");
    }

    #[test]
    fn replace_feature_graph_rejects_mismatched_feature_id() {
        let spec = RenderFeatureGraphSpec::builder("sdf_renderer")
            .resource("sdf.params")
            .pipeline_compute("sdf.compute", "assets/shaders/sdf_compute_3d_example.wgsl")
            .compute_pass("sdf.compute")
            .pipeline("sdf.compute")
            .executor_builtin_compute()
            .reads(["sdf.params"])
            .finish()
            .build()
            .expect("spec should build");

        let mut registry = RenderGraphRegistryResource::default();
        let err = registry
            .replace_feature_graph("different_feature", spec)
            .expect_err("replace should reject mismatched feature id");
        assert!(err.to_string().contains("feature graph id mismatch"));
    }
}
