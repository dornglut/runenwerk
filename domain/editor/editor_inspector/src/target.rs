//! File: domain/editor/editor_inspector/src/target.rs
//! Purpose: Inspector target surface for editor-owned and external domain state.

use editor_core::{
    AssetId, ComponentTypeId, DocumentId, EntityId, ResourceTypeId, SelectionTarget,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InspectTarget {
    Document(DocumentId),
    Entity(EntityId),
    Component {
        entity: EntityId,
        component_type: ComponentTypeId,
    },
    Resource(ResourceTypeId),
    Asset(AssetId),
    Query {
        stable_name: String,
    },
    Custom {
        domain: &'static str,
        id: u64,
    },
}

impl From<&SelectionTarget> for InspectTarget {
    fn from(value: &SelectionTarget) -> Self {
        match value {
            SelectionTarget::Document(id) => Self::Document(*id),
            SelectionTarget::Entity(id) => Self::Entity(*id),
            SelectionTarget::Component {
                entity,
                component_type,
            } => Self::Component {
                entity: *entity,
                component_type: *component_type,
            },
            SelectionTarget::Resource(id) => Self::Resource(*id),
            SelectionTarget::Asset(id) => Self::Asset(*id),
            SelectionTarget::Custom { domain, id } => Self::Custom { domain, id: *id },
        }
    }
}
