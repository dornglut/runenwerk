//! File: domain/editor/editor_scene/src/bridge/scene_runtime.rs
//! Purpose: Runtime bridge between editor_scene commands and the live scene/world.

use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_inspector::{InspectorEditError, InspectorEditValue, InspectorPath};

use crate::{SceneComponentSnapshot, SceneEntitySnapshot};

pub trait SceneRuntime {
	fn create_entity(
		&mut self,
		parent: Option<EntityId>,
		display_name: &str,
	) -> Result<EntityId, &'static str>;

	fn restore_entity(&mut self, snapshot: SceneEntitySnapshot) -> Result<(), &'static str>;

	fn delete_entity(&mut self, entity: EntityId) -> Result<SceneEntitySnapshot, &'static str>;

	fn reparent_entity(
		&mut self,
		entity: EntityId,
		new_parent: Option<EntityId>,
	) -> Result<Option<EntityId>, &'static str>;

	fn add_component(
		&mut self,
		entity: EntityId,
		component_type: ComponentTypeId,
	) -> Result<(), &'static str>;

	fn remove_component(
		&mut self,
		entity: EntityId,
		component_type: ComponentTypeId,
	) -> Result<SceneComponentSnapshot, &'static str>;

	fn restore_component(
		&mut self,
		snapshot: SceneComponentSnapshot,
	) -> Result<(), &'static str>;

	fn read_component_field(
		&self,
		entity: EntityId,
		component_type: ComponentTypeId,
		path: &InspectorPath,
	) -> Result<InspectorEditValue, InspectorEditError>;

	fn write_component_field(
		&mut self,
		entity: EntityId,
		component_type: ComponentTypeId,
		path: &InspectorPath,
		value: InspectorEditValue,
	) -> Result<(), InspectorEditError>;

	fn read_resource_field(
		&self,
		resource_type: ResourceTypeId,
		path: &InspectorPath,
	) -> Result<InspectorEditValue, InspectorEditError>;

	fn write_resource_field(
		&mut self,
		resource_type: ResourceTypeId,
		path: &InspectorPath,
		value: InspectorEditValue,
	) -> Result<(), InspectorEditError>;

	fn rename_entity(
		&mut self,
		entity: EntityId,
		new_display_name: &str,
	) -> Result<String, &'static str>;
}

pub struct SceneCommandContext<'a> {
	session: &'a mut editor_core::EditorSession,
	runtime: &'a mut dyn SceneRuntime,
}

impl<'a> SceneCommandContext<'a> {
	/// File: domain/editor/editor_scene/src/bridge/scene_runtime.rs
	/// Method: new
	pub fn new(
		session: &'a mut editor_core::EditorSession,
		runtime: &'a mut dyn SceneRuntime,
	) -> Self {
		Self { session, runtime }
	}

	/// File: domain/editor/editor_scene/src/bridge/scene_runtime.rs
	/// Method: session
	pub fn session(&self) -> &editor_core::EditorSession {
		self.session
	}

	/// File: domain/editor/editor_scene/src/bridge/scene_runtime.rs
	/// Method: session_mut
	pub fn session_mut(&mut self) -> &mut editor_core::EditorSession {
		self.session
	}

	/// File: domain/editor/editor_scene/src/bridge/scene_runtime.rs
	/// Method: runtime
	pub fn runtime(&self) -> &dyn SceneRuntime {
		self.runtime
	}

	/// File: domain/editor/editor_scene/src/bridge/scene_runtime.rs
	/// Method: runtime_mut
	pub fn runtime_mut(&mut self) -> &mut dyn SceneRuntime {
		self.runtime
	}
}

impl<'a> editor_core::CommandContext for SceneCommandContext<'a> {
	type Error = &'static str;

	fn mark_document_dirty(
		&mut self,
		document_id: editor_core::DocumentId,
		is_dirty: bool,
	) -> Result<(), Self::Error> {
		self.session.set_document_dirty(document_id, is_dirty)
	}
}