use editor_core::{ComponentTypeId, EditorMutationError, EntityId, ResourceTypeId};
use editor_inspector::{
    set_component_field_value, set_resource_field_value, InspectorEditError, InspectorEditValue,
    InspectorPath, InspectorPathSegment,
};
use editor_scene::{SceneComponentSnapshot, SceneEntitySnapshot, SceneRuntime};

use crate::editor_runtime::{
    EditorRuntimeIdRegistry, RunenwerkEditorInspectorBridge, SceneDocumentState,
};

struct EmptyEntityBundle;

impl ecs::Bundle for EmptyEntityBundle {
    fn register(_world: &mut ecs::World) {}

    fn insert(self, _world: &mut ecs::World, _entity: ecs::Entity) -> Result<(), ecs::EntityError> {
        Ok(())
    }

    fn remove(_world: &mut ecs::World, _entity: ecs::Entity) -> Result<Self, ecs::EntityError> {
        Ok(Self)
    }
}

pub struct RunenwerkEditorSceneRuntime<'a> {
    document: &'a mut SceneDocumentState,
    world: &'a mut ecs::World,
    ids: &'a mut EditorRuntimeIdRegistry,
}

impl<'a> RunenwerkEditorSceneRuntime<'a> {
    pub fn new(
        document: &'a mut SceneDocumentState,
        world: &'a mut ecs::World,
        ids: &'a mut EditorRuntimeIdRegistry,
    ) -> Self {
        Self {
            document,
            world,
            ids,
        }
    }
}

impl<'a> SceneRuntime for RunenwerkEditorSceneRuntime<'a> {
    fn create_entity(
        &mut self,
        parent: Option<EntityId>,
        display_name: &str,
    ) -> Result<EntityId, EditorMutationError> {
        if let Some(parent) = parent {
            if !self.document.contains(parent) {
                return Err(EditorMutationError::runtime_rejected(
                    "new parent entity is not registered",
                ));
            }
        }

        let ecs_entity = self.world.spawn(EmptyEntityBundle);
        let editor_id = self.ids.allocate_entity_id();
        self.document
            .register_entity(editor_id, display_name.to_string(), parent)?;
        self.ids.register_entity(editor_id, ecs_entity);

        Ok(editor_id)
    }

    fn restore_entity(&mut self, snapshot: SceneEntitySnapshot) -> Result<(), EditorMutationError> {
        if let Some(parent) = snapshot.parent {
            if !self.document.contains(parent) {
                return Err(EditorMutationError::runtime_rejected(
                    "new parent entity is not registered",
                ));
            }
        }

        self.document.restore_entity(snapshot.clone())?;

        if self.ids.resolve_entity(snapshot.id).is_none() {
            let ecs_entity = self.world.spawn(EmptyEntityBundle);
            self.ids.register_entity(snapshot.id, ecs_entity);
        }

        Ok(())
    }

    fn delete_entity(
        &mut self,
        entity: EntityId,
    ) -> Result<SceneEntitySnapshot, EditorMutationError> {
        if self.document.has_children(entity) {
            return Err(EditorMutationError::runtime_rejected(
                "cannot delete entity while it still has children",
            ));
        }

        let snapshot =
            self.document
                .entity_snapshot(entity)
                .ok_or(EditorMutationError::runtime_rejected(
                    "editor entity is not registered",
                ))?;

        let ecs_entity =
            self.ids
                .resolve_entity(entity)
                .ok_or(EditorMutationError::runtime_rejected(
                    "editor entity is not registered",
                ))?;

        self.world
            .despawn(ecs_entity)
            .map_err(|_| EditorMutationError::runtime_rejected("failed to despawn ecs entity"))?;

        let _ = self.document.unregister_entity(entity);
        let _ = self.ids.unregister_entity(entity);

        Ok(snapshot)
    }

    fn reparent_entity(
        &mut self,
        entity: EntityId,
        new_parent: Option<EntityId>,
    ) -> Result<Option<EntityId>, EditorMutationError> {
        self.document.reparent_entity(entity, new_parent)
    }

    fn add_component(
        &mut self,
        entity: EntityId,
        component_type: ComponentTypeId,
    ) -> Result<(), EditorMutationError> {
        let ecs_entity =
            self.ids
                .resolve_entity(entity)
                .ok_or(EditorMutationError::runtime_rejected(
                    "editor entity is not registered",
                ))?;

        self.ids
            .add_default_component(self.world, ecs_entity, component_type)
    }

    fn remove_component(
        &mut self,
        entity: EntityId,
        component_type: ComponentTypeId,
    ) -> Result<SceneComponentSnapshot, EditorMutationError> {
        let ecs_entity =
            self.ids
                .resolve_entity(entity)
                .ok_or(EditorMutationError::runtime_rejected(
                    "editor entity is not registered",
                ))?;

        let display_name = self
            .ids
            .component_display_name(component_type)
            .ok_or(EditorMutationError::runtime_rejected(
                "component type is not registered in editor runtime",
            ))?
            .to_string();

        self.ids
            .remove_component_and_capture(self.world, entity, ecs_entity, component_type)?;

        Ok(SceneComponentSnapshot::new(
            entity,
            component_type,
            display_name,
        ))
    }

    fn restore_component(
        &mut self,
        snapshot: SceneComponentSnapshot,
    ) -> Result<(), EditorMutationError> {
        let ecs_entity = self.ids.resolve_entity(snapshot.entity).ok_or(
            EditorMutationError::runtime_rejected("editor entity is not registered"),
        )?;

        self.ids.restore_removed_component(
            self.world,
            snapshot.entity,
            ecs_entity,
            snapshot.component_type,
        )
    }

    fn read_component_field(
        &self,
        entity: EntityId,
        component_type: ComponentTypeId,
        path: &InspectorPath,
    ) -> Result<InspectorEditValue, InspectorEditError> {
        let ecs_entity = self
            .ids
            .resolve_entity(entity)
            .ok_or(InspectorEditError::TargetNotFound)?;

        let rust_type_id = self
            .ids
            .resolve_component_rust_type_id(component_type)
            .ok_or(InspectorEditError::TypeNotRegistered)?;

        let value = self
            .world
            .reflected_component_value_ref(ecs_entity, rust_type_id)
            .ok_or(InspectorEditError::ValueNotAvailable)?;

        read_edit_value_at_path(value, path)
    }

    fn write_component_field(
        &mut self,
        entity: EntityId,
        component_type: ComponentTypeId,
        path: &InspectorPath,
        value: InspectorEditValue,
    ) -> Result<(), InspectorEditError> {
        let ids = &*self.ids;
        let world = &mut *self.world;
        let bridge = RunenwerkEditorInspectorBridge::new(ids);

        set_component_field_value(world, &bridge, entity, component_type, path, value)
    }

    fn read_resource_field(
        &self,
        resource_type: ResourceTypeId,
        path: &InspectorPath,
    ) -> Result<InspectorEditValue, InspectorEditError> {
        let rust_type_id = self
            .ids
            .resolve_resource_rust_type_id(resource_type)
            .ok_or(InspectorEditError::TypeNotRegistered)?;

        let value = self
            .world
            .reflected_resource_value_ref(rust_type_id)
            .ok_or(InspectorEditError::ValueNotAvailable)?;

        read_edit_value_at_path(value, path)
    }

    fn write_resource_field(
        &mut self,
        resource_type: ResourceTypeId,
        path: &InspectorPath,
        value: InspectorEditValue,
    ) -> Result<(), InspectorEditError> {
        let ids = &*self.ids;
        let world = &mut *self.world;
        let bridge = RunenwerkEditorInspectorBridge::new(ids);

        set_resource_field_value(world, &bridge, resource_type, path, value)
    }

    fn rename_entity(
        &mut self,
        entity: EntityId,
        new_display_name: &str,
    ) -> Result<String, EditorMutationError> {
        self.document
            .rename_entity(entity, new_display_name.to_string())
    }
}

fn read_edit_value_at_path(
    current: ecs::reflect::ReflectValueRef<'_>,
    path: &InspectorPath,
) -> Result<InspectorEditValue, InspectorEditError> {
    let mut current = current;

    for segment in path.segments() {
        match segment {
            InspectorPathSegment::Field(name) => {
                let struct_ref = current
                    .struct_ref()
                    .ok_or(InspectorEditError::InvalidPath)?;
                current = struct_ref
                    .field(name)
                    .ok_or(InspectorEditError::InvalidPath)?;
            }
            InspectorPathSegment::Index(_) => {
                return Err(InspectorEditError::UnsupportedPathSegment);
            }
        }
    }

    if let Some(value) = current.downcast_ref::<bool>() {
        return Ok(InspectorEditValue::Bool(*value));
    }
    if let Some(value) = current.downcast_ref::<i8>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<i16>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<i32>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<i64>() {
        return Ok(InspectorEditValue::Integer(*value));
    }
    if let Some(value) = current.downcast_ref::<isize>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<u8>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<u16>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<u32>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<u64>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<usize>() {
        return Ok(InspectorEditValue::Integer(*value as i64));
    }
    if let Some(value) = current.downcast_ref::<f32>() {
        return Ok(InspectorEditValue::Float(*value as f64));
    }
    if let Some(value) = current.downcast_ref::<f64>() {
        return Ok(InspectorEditValue::Float(*value));
    }
    if let Some(value) = current.downcast_ref::<String>() {
        return Ok(InspectorEditValue::Text(value.clone()));
    }

    Err(InspectorEditError::UnsupportedValueType {
        actual_type: current.type_info().stable_name.to_string(),
    })
}
