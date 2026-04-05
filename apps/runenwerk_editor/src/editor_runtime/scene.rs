use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_inspector::{
	InspectorEditError, InspectorEditValue, InspectorPath, InspectorPathSegment,
	set_component_field_value, set_resource_field_value,
};
use editor_scene::{SceneComponentSnapshot, SceneEntitySnapshot, SceneRuntime};

use crate::editor_runtime::{
	EditorRuntimeIdRegistry, RunenwerkEditorInspectorBridge, validate_reparent,
};

struct EmptyEntityBundle;

impl ecs::Bundle for EmptyEntityBundle {
	fn register(_world: &mut ecs::World) {}

	fn insert(
		self,
		_world: &mut ecs::World,
		_entity: ecs::Entity,
	) -> Result<(), ecs::EntityError> {
		Ok(())
	}

	fn remove(
		_world: &mut ecs::World,
		_entity: ecs::Entity,
	) -> Result<Self, ecs::EntityError> {
		Ok(Self)
	}
}

pub struct RunenwerkEditorSceneRuntime<'a> {
	world: &'a mut ecs::World,
	ids: &'a mut EditorRuntimeIdRegistry,
}

impl<'a> RunenwerkEditorSceneRuntime<'a> {
	/// File: apps/runenwerk_editor/src/editor_runtime/scene.rs
	/// Method: new
	pub fn new(world: &'a mut ecs::World, ids: &'a mut EditorRuntimeIdRegistry) -> Self {
		Self { world, ids }
	}
}

impl<'a> SceneRuntime for RunenwerkEditorSceneRuntime<'a> {
	fn create_entity(
		&mut self,
		parent: Option<EntityId>,
		display_name: &str,
	) -> Result<EntityId, &'static str> {
		if let Some(parent) = parent {
			if self.ids.resolve_entity(parent).is_none() {
				return Err("new parent entity is not registered");
			}
		}

		let ecs_entity = self.world.spawn(EmptyEntityBundle);
		let editor_id = self.ids.allocate_entity_id();
		self.ids
			.register_entity(editor_id, ecs_entity, display_name.to_string(), parent);

		Ok(editor_id)
	}

	fn restore_entity(&mut self, snapshot: SceneEntitySnapshot) -> Result<(), &'static str> {
		if let Some(parent) = snapshot.parent {
			if self.ids.resolve_entity(parent).is_none() {
				return Err("new parent entity is not registered");
			}
		}

		let ecs_entity = self.world.spawn(EmptyEntityBundle);
		self.ids.register_entity(
			snapshot.id,
			ecs_entity,
			snapshot.display_name,
			snapshot.parent,
		);
		Ok(())
	}

	fn delete_entity(&mut self, entity: EntityId) -> Result<SceneEntitySnapshot, &'static str> {
		if self.ids.has_children(entity) {
			return Err("cannot delete entity while it still has children");
		}

		let snapshot = self
			.ids
			.entity_snapshot(entity)
			.ok_or("editor entity is not registered")?;

		let ecs_entity = self
			.ids
			.resolve_entity(entity)
			.ok_or("editor entity is not registered")?;

		self.world
			.despawn(ecs_entity)
			.map_err(|_| "failed to despawn ecs entity")?;

		let _ = self.ids.unregister_entity(entity);

		Ok(snapshot)
	}

	fn reparent_entity(
		&mut self,
		entity: EntityId,
		new_parent: Option<EntityId>,
	) -> Result<Option<EntityId>, &'static str> {
		validate_reparent(self.ids, entity, new_parent)?;
		self.ids.reparent_entity(entity, new_parent)
	}

	fn add_component(
		&mut self,
		entity: EntityId,
		component_type: ComponentTypeId,
	) -> Result<(), &'static str> {
		let ecs_entity = self
			.ids
			.resolve_entity(entity)
			.ok_or("editor entity is not registered")?;

		self.ids
			.add_default_component(self.world, ecs_entity, component_type)
	}

	fn remove_component(
		&mut self,
		entity: EntityId,
		component_type: ComponentTypeId,
	) -> Result<SceneComponentSnapshot, &'static str> {
		let ecs_entity = self
			.ids
			.resolve_entity(entity)
			.ok_or("editor entity is not registered")?;

		let display_name = self
			.ids
			.component_display_name(component_type)
			.ok_or("component type is not registered in editor runtime")?
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
	) -> Result<(), &'static str> {
		let ecs_entity = self
			.ids
			.resolve_entity(snapshot.entity)
			.ok_or("editor entity is not registered")?;

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
	) -> Result<String, &'static str> {
		self.ids.rename_entity(entity, new_display_name.to_string())
	}
}

/// File: apps/runenwerk_editor/src/editor_runtime/scene.rs
/// Method: read_edit_value_at_path
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