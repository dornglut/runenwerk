//! File: domain/editor/editor_scene/src/scene_command.rs
//! Purpose: Scene authoring command intents.

use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_inspector::{InspectorEditValue, InspectorPath};

use crate::{SceneTransform, SdfPrimitiveSpec};

#[derive(Debug, Clone, PartialEq)]
pub enum SceneCommandIntent {
    CreateEntity {
        parent: Option<EntityId>,
        display_name: String,
    },
    CreateChildEntity {
        parent: EntityId,
        display_name: String,
    },
    CreateSdfPrimitive {
        parent: Option<EntityId>,
        display_name: String,
        primitive: SdfPrimitiveSpec,
    },
    DeleteEntity {
        entity: EntityId,
    },
    DeleteEntities {
        entities: Vec<EntityId>,
    },
    DuplicateEntitySubtree {
        source: EntityId,
        new_parent: Option<EntityId>,
        name_suffix: String,
    },
    ReparentEntity {
        entity: EntityId,
        new_parent: Option<EntityId>,
    },
    AddComponent {
        entity: EntityId,
        component_type: ComponentTypeId,
    },
    RemoveComponent {
        entity: EntityId,
        component_type: ComponentTypeId,
    },
    EditComponentField {
        entity: EntityId,
        component_type: ComponentTypeId,
        path: InspectorPath,
        value: InspectorEditValue,
    },
    EditResourceField {
        resource_type: ResourceTypeId,
        path: InspectorPath,
        value: InspectorEditValue,
    },
    RenameEntity {
        entity: EntityId,
        new_display_name: String,
    },
    SetTransform {
        entity: EntityId,
        transform: SceneTransform,
    },
    ResetTransform {
        entity: EntityId,
    },
}
