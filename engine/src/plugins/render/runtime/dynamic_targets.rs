use crate::plugins::render::{
    RenderDynamicTextureTargetDescriptor, RenderDynamicTextureTargetDescriptorError,
    RenderDynamicTextureTargetKey, RenderFrameProducerId, RenderFrameSurfaceScope,
};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderDynamicTextureTargetRequestDiagnostic {
    pub producer_id: RenderFrameProducerId,
    pub key: RenderDynamicTextureTargetKey,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderDynamicTextureTargetRequestRegistryError {
    InvalidDescriptor(RenderDynamicTextureTargetDescriptorError),
    DuplicateTargetWithinProducer {
        producer_id: RenderFrameProducerId,
        key: RenderDynamicTextureTargetKey,
    },
    DuplicateTargetAcrossProducers {
        existing_producer_id: RenderFrameProducerId,
        replacement_producer_id: RenderFrameProducerId,
        key: RenderDynamicTextureTargetKey,
    },
}

impl fmt::Display for RenderDynamicTextureTargetRequestRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDescriptor(err) => write!(f, "{err}"),
            Self::DuplicateTargetWithinProducer { producer_id, key } => write!(
                f,
                "render target producer '{producer_id:?}' requested duplicate dynamic target '{key}'"
            ),
            Self::DuplicateTargetAcrossProducers {
                existing_producer_id,
                replacement_producer_id,
                key,
            } => write!(
                f,
                "render target producer '{replacement_producer_id:?}' requested dynamic target '{key}' already owned by producer '{existing_producer_id:?}'"
            ),
        }
    }
}

impl std::error::Error for RenderDynamicTextureTargetRequestRegistryError {}

impl From<RenderDynamicTextureTargetDescriptorError>
    for RenderDynamicTextureTargetRequestRegistryError
{
    fn from(value: RenderDynamicTextureTargetDescriptorError) -> Self {
        Self::InvalidDescriptor(value)
    }
}

#[derive(Debug, Clone, Default)]
struct RenderDynamicTextureTargetRequestContribution {
    scope: RenderFrameSurfaceScope,
    descriptors: BTreeMap<RenderDynamicTextureTargetKey, RenderDynamicTextureTargetDescriptor>,
}

#[derive(Debug, Default, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct RenderDynamicTextureTargetRequestRegistryResource {
    contributions: BTreeMap<RenderFrameProducerId, RenderDynamicTextureTargetRequestContribution>,
    diagnostics: Vec<RenderDynamicTextureTargetRequestDiagnostic>,
}

impl RenderDynamicTextureTargetRequestRegistryResource {
    pub fn clear(&mut self) {
        self.contributions.clear();
        self.diagnostics.clear();
    }

    pub fn remove_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
    ) -> Option<BTreeMap<RenderDynamicTextureTargetKey, RenderDynamicTextureTargetDescriptor>> {
        self.contributions
            .remove(&producer_id.into())
            .map(|contribution| contribution.descriptors)
    }

    pub fn replace_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
        descriptors: impl IntoIterator<Item = RenderDynamicTextureTargetDescriptor>,
    ) -> Result<(), RenderDynamicTextureTargetRequestRegistryError> {
        self.replace_scoped_contribution(
            producer_id.into(),
            RenderFrameSurfaceScope::AllSurfaces,
            descriptors,
        )
    }

    pub fn replace_surface_contribution(
        &mut self,
        producer_id: impl Into<RenderFrameProducerId>,
        render_surface_id: crate::plugins::render::backend::RenderSurfaceId,
        descriptors: impl IntoIterator<Item = RenderDynamicTextureTargetDescriptor>,
    ) -> Result<(), RenderDynamicTextureTargetRequestRegistryError> {
        self.replace_scoped_contribution(
            producer_id.into(),
            RenderFrameSurfaceScope::Surface(render_surface_id),
            descriptors,
        )
    }

    fn replace_scoped_contribution(
        &mut self,
        producer_id: RenderFrameProducerId,
        scope: RenderFrameSurfaceScope,
        descriptors: impl IntoIterator<Item = RenderDynamicTextureTargetDescriptor>,
    ) -> Result<(), RenderDynamicTextureTargetRequestRegistryError> {
        self.diagnostics
            .retain(|diagnostic| diagnostic.producer_id != producer_id);
        let mut descriptors_by_key =
            BTreeMap::<RenderDynamicTextureTargetKey, RenderDynamicTextureTargetDescriptor>::new();

        for descriptor in descriptors {
            if let Err(err) = descriptor.validate() {
                self.diagnostics
                    .push(RenderDynamicTextureTargetRequestDiagnostic {
                        producer_id,
                        key: descriptor.key.clone(),
                        message: err.to_string(),
                    });
                return Err(err.into());
            }
            let key = descriptor.key.clone();
            if descriptors_by_key.insert(key.clone(), descriptor).is_some() {
                let err =
                    RenderDynamicTextureTargetRequestRegistryError::DuplicateTargetWithinProducer {
                        producer_id,
                        key: key.clone(),
                    };
                self.diagnostics
                    .push(RenderDynamicTextureTargetRequestDiagnostic {
                        producer_id,
                        key,
                        message: err.to_string(),
                    });
                return Err(err);
            }
        }

        for (existing_producer_id, existing_contribution) in &self.contributions {
            if *existing_producer_id == producer_id || !scope.overlaps(existing_contribution.scope)
            {
                continue;
            }
            for key in descriptors_by_key.keys() {
                if existing_contribution.descriptors.contains_key(key) {
                    let err =
                        RenderDynamicTextureTargetRequestRegistryError::DuplicateTargetAcrossProducers {
                            existing_producer_id: *existing_producer_id,
                            replacement_producer_id: producer_id,
                            key: key.clone(),
                        };
                    self.diagnostics
                        .push(RenderDynamicTextureTargetRequestDiagnostic {
                            producer_id,
                            key: key.clone(),
                            message: err.to_string(),
                        });
                    return Err(err);
                }
            }
        }
        self.contributions.insert(
            producer_id,
            RenderDynamicTextureTargetRequestContribution {
                scope,
                descriptors: descriptors_by_key,
            },
        );
        Ok(())
    }

    pub fn snapshot(&self) -> Vec<RenderDynamicTextureTargetDescriptor> {
        self.contributions
            .values()
            .flat_map(|contribution| contribution.descriptors.values().cloned())
            .collect()
    }

    pub fn snapshot_for_surface(
        &self,
        render_surface_id: crate::plugins::render::backend::RenderSurfaceId,
    ) -> Vec<RenderDynamicTextureTargetDescriptor> {
        self.contributions
            .values()
            .filter(|contribution| contribution.scope.applies_to(render_surface_id))
            .flat_map(|contribution| contribution.descriptors.values().cloned())
            .collect()
    }

    pub fn diagnostics(&self) -> &[RenderDynamicTextureTargetRequestDiagnostic] {
        &self.diagnostics
    }

    pub fn is_empty(&self) -> bool {
        self.contributions
            .values()
            .all(|contribution| contribution.descriptors.is_empty())
    }
}
