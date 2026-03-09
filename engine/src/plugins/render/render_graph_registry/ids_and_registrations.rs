use super::*;

// Owner: Engine Render Graph Registry - Identifiers and Registrations
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


