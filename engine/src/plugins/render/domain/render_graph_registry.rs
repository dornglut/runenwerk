use super::{PassSlot, PipelineKey};

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
    pub slot: Option<PassSlot>,
}

impl RegisteredPipelineDescriptor {
    pub fn new(id: impl Into<String>, key: PipelineKey) -> Self {
        Self {
            id: id.into(),
            key,
            slot: None,
        }
    }

    pub fn with_slot(mut self, slot: PassSlot) -> Self {
        self.slot = Some(slot);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredPassDescriptor {
    pub id: String,
    pub kind: RegisteredPassKind,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
    pub depends_on: Vec<String>,
    pub slot: Option<PassSlot>,
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
            slot: None,
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

    pub fn with_slot(mut self, slot: PassSlot) -> Self {
        self.slot = Some(slot);
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

#[derive(Debug, Clone, Default)]
pub struct RenderGraphRegistryResource {
    owners: Vec<OwnerRenderGraphRegistration>,
    revision: u64,
}

impl RenderGraphRegistryResource {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn owner_count(&self) -> usize {
        self.owners.len()
    }

    pub fn upsert_owner(&mut self, registration: OwnerRenderGraphRegistration) {
        if let Some(existing) = self
            .owners
            .iter_mut()
            .find(|entry| entry.owner == registration.owner)
        {
            *existing = registration;
        } else {
            self.owners.push(registration);
        }
        self.revision = self.revision.saturating_add(1);
    }

    pub fn clear_owner(&mut self, owner: &str) -> bool {
        let before = self.owners.len();
        self.owners.retain(|entry| entry.owner != owner);
        let removed = before != self.owners.len();
        if removed {
            self.revision = self.revision.saturating_add(1);
        }
        removed
    }

    pub fn clear(&mut self) {
        if self.owners.is_empty() {
            return;
        }
        self.owners.clear();
        self.revision = self.revision.saturating_add(1);
    }

    pub fn owners(&self) -> &[OwnerRenderGraphRegistration] {
        &self.owners
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OwnerRenderGraphRegistration, RegisteredPassDescriptor, RegisteredPipelineDescriptor,
        RenderGraphRegistryResource,
    };
    use crate::plugins::render::domain::PipelineKey;

    #[test]
    fn upsert_owner_replaces_existing_owner_registration() {
        let mut registry = RenderGraphRegistryResource::default();
        registry.upsert_owner(
            OwnerRenderGraphRegistration::new("sdf").with_pipelines(vec![
                RegisteredPipelineDescriptor::new("sdf.compute", PipelineKey::WorldComputeSdf3d),
            ]),
        );
        registry.upsert_owner(
            OwnerRenderGraphRegistration::new("sdf")
                .with_passes(vec![RegisteredPassDescriptor::compute("sdf_compute")]),
        );
        assert_eq!(registry.owner_count(), 1);
        let owner = &registry.owners()[0];
        assert!(owner.pipelines.is_empty());
        assert_eq!(owner.passes.len(), 1);
    }
}
