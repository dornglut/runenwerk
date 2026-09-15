//! File: domain/editor/editor_inspector/src/target.rs
//! Purpose: Inspector target surface for editor-owned and external domain state.

use editor_core::{AssetId, ComponentTypeId, DocumentId, EntityId, ResourceTypeId};

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
