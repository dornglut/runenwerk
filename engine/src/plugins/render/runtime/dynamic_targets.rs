use crate::plugins::render::{
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetDescriptorError,
    RenderDynamicTextureTargetKey,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderDynamicTextureTargetRequestDiagnostic {
    pub key: RenderDynamicTextureTargetKey,
    pub message: String,
}

#[derive(Debug, Default, Clone, ecs::Component, ecs::Resource)]
pub struct RenderDynamicTextureTargetRequestRegistryResource {
    requests: BTreeMap<RenderDynamicTextureTargetKey, RenderDynamicTextureTargetDescriptor>,
    diagnostics: Vec<RenderDynamicTextureTargetRequestDiagnostic>,
}

impl RenderDynamicTextureTargetRequestRegistryResource {
    pub fn clear(&mut self) {
        self.requests.clear();
        self.diagnostics.clear();
    }

    pub fn request(
        &mut self,
        descriptor: RenderDynamicTextureTargetDescriptor,
    ) -> Result<(), RenderDynamicTextureTargetDescriptorError> {
        if let Err(err) = descriptor.validate() {
            self.diagnostics
                .push(RenderDynamicTextureTargetRequestDiagnostic {
                    key: descriptor.key.clone(),
                    message: err.to_string(),
                });
            return Err(err);
        }
        self.requests.insert(descriptor.key.clone(), descriptor);
        Ok(())
    }

    pub fn replace_requests(
        &mut self,
        descriptors: impl IntoIterator<Item = RenderDynamicTextureTargetDescriptor>,
    ) -> Result<(), RenderDynamicTextureTargetDescriptorError> {
        self.clear();
        for descriptor in descriptors {
            self.request(descriptor)?;
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Vec<RenderDynamicTextureTargetDescriptor> {
        self.requests.values().cloned().collect()
    }

    pub fn diagnostics(&self) -> &[RenderDynamicTextureTargetRequestDiagnostic] {
        &self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}
