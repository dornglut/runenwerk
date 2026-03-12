use crate::plugins::render::{RenderResourceDescriptor, RenderResourceId};
use std::any::{TypeId, type_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcsResourceDeclaration {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

impl EcsResourceDeclaration {
    pub fn of<T: ecs::Component + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResourceGraph {
    pub ecs_resources: Vec<EcsResourceDeclaration>,
    pub resources: Vec<RenderResourceDescriptor>,
}

impl ResourceGraph {
    pub fn add_ecs_resource<T>(&mut self)
    where
        T: ecs::Component + 'static,
    {
        self.ecs_resources.push(EcsResourceDeclaration::of::<T>());
    }

    pub fn add_resource(&mut self, descriptor: RenderResourceDescriptor) {
        self.resources.push(descriptor);
    }

    pub fn uniform_buffer_ids_by_params_type(&self, type_id: TypeId) -> Vec<RenderResourceId> {
        self.resources
            .iter()
            .filter_map(|resource| match resource {
                RenderResourceDescriptor::UniformBuffer(value)
                    if value.params_type_id == type_id =>
                {
                    Some(value.id.clone())
                }
                _ => None,
            })
            .collect()
    }

    pub fn has_ecs_resource(&self, type_id: TypeId) -> bool {
        self.ecs_resources
            .iter()
            .any(|resource| resource.type_id == type_id)
    }
}
