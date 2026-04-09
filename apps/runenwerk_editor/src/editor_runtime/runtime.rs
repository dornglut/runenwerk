use editor_core::{ComponentTypeId, EditorSession, EntityId, ResourceTypeId};
use editor_inspector::InspectTarget;
use editor_scene::SceneComponentDescriptor;

use crate::editor_runtime::{
    EditorRuntimeIdRegistry, HierarchySnapshot, OutlinerTree, RunenwerkEditorInspectorBridge,
    RunenwerkEditorSceneRuntime, SceneCommandStore, SceneDocumentState, SceneEntityView,
    all_entity_views, build_hierarchy_snapshot, outliner_tree_from_hierarchy_snapshot,
    primary_selected_entity, resolve_primary_inspect_target_from_runtime, validate_reparent,
};

pub struct RunenwerkEditorRuntime {
    session: EditorSession,
    document: SceneDocumentState,
    world: ecs::World,
    ids: EditorRuntimeIdRegistry,
    command_store: SceneCommandStore,
    next_command_id: u64,
    next_transaction_id: u64,
}

impl RunenwerkEditorRuntime {
    pub fn new() -> Self {
        Self {
            session: EditorSession::new(),
            document: SceneDocumentState::new(),
            world: ecs::World::new(),
            ids: EditorRuntimeIdRegistry::new(),
            command_store: SceneCommandStore::new(),
            next_command_id: 1,
            next_transaction_id: 1,
        }
    }

    pub fn session(&self) -> &EditorSession {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut EditorSession {
        &mut self.session
    }

    pub fn world(&self) -> &ecs::World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut ecs::World {
        &mut self.world
    }

    pub fn document(&self) -> &SceneDocumentState {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut SceneDocumentState {
        &mut self.document
    }

    pub fn ids(&self) -> &EditorRuntimeIdRegistry {
        &self.ids
    }

    pub fn ids_mut(&mut self) -> &mut EditorRuntimeIdRegistry {
        &mut self.ids
    }

    pub fn command_store(&self) -> &SceneCommandStore {
        &self.command_store
    }

    pub fn command_store_mut(&mut self) -> &mut SceneCommandStore {
        &mut self.command_store
    }

    pub fn inspector_bridge(&self) -> RunenwerkEditorInspectorBridge<'_> {
        RunenwerkEditorInspectorBridge::new(&self.ids)
    }

    pub fn scene_runtime(&mut self) -> RunenwerkEditorSceneRuntime<'_> {
        RunenwerkEditorSceneRuntime::new(&mut self.document, &mut self.world, &mut self.ids)
    }

    pub fn session_and_scene_runtime(
        &mut self,
    ) -> (&mut EditorSession, RunenwerkEditorSceneRuntime<'_>) {
        let session = &mut self.session;
        let scene_runtime =
            RunenwerkEditorSceneRuntime::new(&mut self.document, &mut self.world, &mut self.ids);
        (session, scene_runtime)
    }

    pub fn register_entity(
        &mut self,
        editor_id: EntityId,
        ecs_entity: ecs::Entity,
        display_name: impl Into<String>,
        parent: Option<EntityId>,
    ) {
        let display_name = display_name.into();
        self.document
            .register_entity(editor_id, display_name.clone(), parent)
            .expect("document entity registration failed");
        self.ids
            .register_entity(editor_id, ecs_entity, display_name, parent);
    }

    pub fn register_component_type<T>(&mut self, editor_id: ComponentTypeId)
    where
        T: ecs::Component + ecs::Reflect + Default + 'static,
    {
        self.world.register_component_type::<T>();
        self.ids.register_component_type::<T>(editor_id);
    }

    pub fn register_resource_type<T>(&mut self, editor_id: ResourceTypeId)
    where
        T: ecs::Resource + ecs::Reflect + 'static,
    {
        self.world.register_resource_type::<T>();
        self.ids.register_resource_type::<T>(editor_id);
    }

    pub fn list_scene_entities(&self) -> Vec<SceneEntityView> {
        all_entity_views(&self.document)
    }

    pub fn hierarchy_snapshot(&self) -> HierarchySnapshot {
        build_hierarchy_snapshot(&self.document)
    }

    pub fn outliner_tree(&self) -> OutlinerTree {
        outliner_tree_from_hierarchy_snapshot(&self.hierarchy_snapshot())
    }

    pub fn selected_entity(&self) -> Option<EntityId> {
        primary_selected_entity(self)
    }

    pub fn primary_inspect_target(&self) -> Option<InspectTarget> {
        resolve_primary_inspect_target_from_runtime(self)
    }

    pub fn validate_reparent(
        &self,
        entity: EntityId,
        new_parent: Option<EntityId>,
    ) -> Result<(), &'static str> {
        validate_reparent(&self.document, entity, new_parent)
    }

    pub fn entity_has_component(&self, entity: EntityId, component_type: ComponentTypeId) -> bool {
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

    pub fn list_entity_components(&self, entity: EntityId) -> Vec<SceneComponentDescriptor> {
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

    pub fn allocate_command_id(&mut self) -> editor_core::CommandId {
        let id = editor_core::CommandId(self.next_command_id);
        self.next_command_id += 1;
        id
    }

    pub fn allocate_transaction_id(&mut self) -> editor_core::TransactionId {
        let id = editor_core::TransactionId(self.next_transaction_id);
        self.next_transaction_id += 1;
        id
    }

    pub fn list_registered_component_types(&self) -> Vec<(ComponentTypeId, String)> {
        let mut component_types = self
            .ids
            .component_type_ids()
            .filter_map(|component_type| {
                let display_name = self.ids.component_display_name(component_type)?;
                Some((component_type, display_name.to_string()))
            })
            .collect::<Vec<_>>();

        component_types
            .sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));

        component_types
    }
}
