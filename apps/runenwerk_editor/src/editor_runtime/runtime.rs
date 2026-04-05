use editor_core::{ComponentTypeId, EditorSession, EntityId, ResourceTypeId};
use editor_inspector::InspectTarget;
use editor_scene::SceneComponentDescriptor;

use crate::editor_runtime::{
	all_entity_views, build_hierarchy_snapshot, primary_selected_entity,
	resolve_primary_inspect_target_from_runtime, validate_reparent, EditorRuntimeIdRegistry,
	HierarchySnapshot, OutlinerTree, RunenwerkEditorInspectorBridge,
	RunenwerkEditorSceneRuntime, SceneCommandStore, SceneEntityView,
	outliner_tree_from_hierarchy_snapshot,
};

pub struct RunenwerkEditorRuntime {
	session: EditorSession,
	world: ecs::World,
	ids: EditorRuntimeIdRegistry,
	command_store: SceneCommandStore,
	next_command_id: u64,
	next_transaction_id: u64,
}

impl RunenwerkEditorRuntime {
	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: new
	pub fn new() -> Self {
		Self {
			session: EditorSession::new(),
			world: ecs::World::new(),
			ids: EditorRuntimeIdRegistry::new(),
			command_store: SceneCommandStore::new(),
			next_command_id: 1,
			next_transaction_id: 1,
		}
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: session
	pub fn session(&self) -> &EditorSession {
		&self.session
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: session_mut
	pub fn session_mut(&mut self) -> &mut EditorSession {
		&mut self.session
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: world
	pub fn world(&self) -> &ecs::World {
		&self.world
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: world_mut
	pub fn world_mut(&mut self) -> &mut ecs::World {
		&mut self.world
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: ids
	pub fn ids(&self) -> &EditorRuntimeIdRegistry {
		&self.ids
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: ids_mut
	pub fn ids_mut(&mut self) -> &mut EditorRuntimeIdRegistry {
		&mut self.ids
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: command_store
	pub fn command_store(&self) -> &SceneCommandStore {
		&self.command_store
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: command_store_mut
	pub fn command_store_mut(&mut self) -> &mut SceneCommandStore {
		&mut self.command_store
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: inspector_bridge
	pub fn inspector_bridge(&self) -> RunenwerkEditorInspectorBridge<'_> {
		RunenwerkEditorInspectorBridge::new(&self.ids)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: scene_runtime
	pub fn scene_runtime(&mut self) -> RunenwerkEditorSceneRuntime<'_> {
		RunenwerkEditorSceneRuntime::new(&mut self.world, &mut self.ids)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: session_and_scene_runtime
	pub fn session_and_scene_runtime(
		&mut self,
	) -> (&mut EditorSession, RunenwerkEditorSceneRuntime<'_>) {
		let session = &mut self.session;
		let scene_runtime = RunenwerkEditorSceneRuntime::new(&mut self.world, &mut self.ids);
		(session, scene_runtime)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: register_entity
	pub fn register_entity(
		&mut self,
		editor_id: EntityId,
		ecs_entity: ecs::Entity,
		display_name: impl Into<String>,
		parent: Option<EntityId>,
	) {
		self.ids
			.register_entity(editor_id, ecs_entity, display_name, parent);
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: register_component_type
	pub fn register_component_type<T>(&mut self, editor_id: ComponentTypeId)
	where
		T: ecs::Component + ecs::Reflect + Default + 'static,
	{
		self.world.register_component_type::<T>();
		self.ids.register_component_type::<T>(editor_id);
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: register_resource_type
	pub fn register_resource_type<T>(&mut self, editor_id: ResourceTypeId)
	where
		T: ecs::Resource + ecs::Reflect + 'static,
	{
		self.world.register_resource_type::<T>();
		self.ids.register_resource_type::<T>(editor_id);
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: list_scene_entities
	pub fn list_scene_entities(&self) -> Vec<SceneEntityView> {
		all_entity_views(&self.ids)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: hierarchy_snapshot
	pub fn hierarchy_snapshot(&self) -> HierarchySnapshot {
		build_hierarchy_snapshot(&self.ids)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: outliner_tree
	pub fn outliner_tree(&self) -> OutlinerTree {
		outliner_tree_from_hierarchy_snapshot(&self.hierarchy_snapshot())
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: selected_entity
	pub fn selected_entity(&self) -> Option<EntityId> {
		primary_selected_entity(self)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: primary_inspect_target
	pub fn primary_inspect_target(&self) -> Option<InspectTarget> {
		resolve_primary_inspect_target_from_runtime(self)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: validate_reparent
	pub fn validate_reparent(
		&self,
		entity: EntityId,
		new_parent: Option<EntityId>,
	) -> Result<(), &'static str> {
		validate_reparent(&self.ids, entity, new_parent)
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: entity_has_component
	pub fn entity_has_component(
		&self,
		entity: EntityId,
		component_type: ComponentTypeId,
	) -> bool {
		let Some(ecs_entity) = self.ids.resolve_entity(entity) else {
			return false;
		};

		let Some(rust_type_id) = self.ids.resolve_component_rust_type_id(component_type) else {
			return false;
		};

		self.world
			.reflected_component_value_ref(ecs_entity, rust_type_id)
			.is_some()
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: list_entity_components
	pub fn list_entity_components(
		&self,
		entity: EntityId,
	) -> Vec<SceneComponentDescriptor> {
		let mut components = self
			.ids
			.component_type_ids()
			.filter(|component_type| self.entity_has_component(entity, *component_type))
			.filter_map(|component_type| {
				let display_name = self.ids.component_display_name(component_type)?;
				Some(SceneComponentDescriptor::new(
					entity,
					component_type,
					display_name.to_string(),
				))
			})
			.collect::<Vec<_>>();

		components.sort_by(|left, right| {
			left.display_name
				.cmp(&right.display_name)
				.then_with(|| left.component_type.cmp(&right.component_type))
		});

		components
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: allocate_command_id
	pub fn allocate_command_id(&mut self) -> editor_core::CommandId {
		let id = editor_core::CommandId(self.next_command_id);
		self.next_command_id += 1;
		id
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: allocate_transaction_id
	pub fn allocate_transaction_id(&mut self) -> editor_core::TransactionId {
		let id = editor_core::TransactionId(self.next_transaction_id);
		self.next_transaction_id += 1;
		id
	}

	/// File: apps/runenwerk_editor/src/editor_runtime/runtime.rs
	/// Method: list_registered_component_types
	pub fn list_registered_component_types(
		&self,
	) -> Vec<(ComponentTypeId, String)> {
		let mut component_types = self
			.ids
			.component_type_ids()
			.filter_map(|component_type| {
				let display_name = self.ids.component_display_name(component_type)?;
				Some((component_type, display_name.to_string()))
			})
			.collect::<Vec<_>>();

		component_types.sort_by(|left, right| {
			left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0))
		});

		component_types
	}
}