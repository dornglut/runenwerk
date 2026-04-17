use editor_core::{ComponentTypeId, EditorSession, EntityId, ResourceTypeId};
use editor_inspector::InspectTarget;
use editor_scene::SceneComponentDescriptor;

use crate::editor_runtime::{
    EditorRuntimeIdRegistry, HierarchySnapshot, OutlinerTree, RatifiedChangeLog,
    RunenwerkEditorInspectorBridge, RunenwerkEditorSceneRuntime, SceneCommandStore,
    SceneDocumentState, SceneEntityView, all_entity_views, build_hierarchy_snapshot,
    outliner_tree_from_hierarchy_snapshot, primary_selected_entity,
    resolve_primary_inspect_target_from_runtime, validate_reparent,
};

struct SceneRealityStore {
    authored: SceneDocumentState,
    instantiated: ecs::World,
    identities: EditorRuntimeIdRegistry,
}

impl SceneRealityStore {
    fn new() -> Self {
        Self {
            authored: SceneDocumentState::new(),
            instantiated: ecs::World::new(),
            identities: EditorRuntimeIdRegistry::new(),
        }
    }
}

pub struct RunenwerkEditorRuntime {
    session: EditorSession,
    scene_realities: SceneRealityStore,
    command_store: SceneCommandStore,
    ratified_changes: RatifiedChangeLog,
    session_changes: editor_core::SessionChangeLog,
    shared_changes: editor_core::SharedChangeOutbox,
    sharing_policy: editor_core::SharingPolicy,
    workflow: editor_core::WorkflowLog,
    next_command_id: u64,
    next_transaction_id: u64,
    next_ratification_id: u64,
    next_causality_id: u64,
    next_session_change_id: u64,
    next_shared_change_sequence: u64,
    next_workflow_event_id: u64,
    scene_reality_version: u64,
}

impl RunenwerkEditorRuntime {
    pub fn new() -> Self {
        Self {
            session: EditorSession::new(),
            scene_realities: SceneRealityStore::new(),
            command_store: SceneCommandStore::new(),
            ratified_changes: RatifiedChangeLog::new(),
            session_changes: editor_core::SessionChangeLog::new(),
            shared_changes: editor_core::SharedChangeOutbox::new(),
            sharing_policy: editor_core::SharingPolicy::LocalOnly,
            workflow: editor_core::WorkflowLog::new(),
            next_command_id: 1,
            next_transaction_id: 1,
            next_ratification_id: 1,
            next_causality_id: 1,
            next_session_change_id: 1,
            next_shared_change_sequence: 1,
            next_workflow_event_id: 1,
            scene_reality_version: 0,
        }
    }

    pub fn session(&self) -> &EditorSession {
        &self.session
    }

    pub(crate) fn session_mut(&mut self) -> &mut EditorSession {
        &mut self.session
    }

    pub fn world(&self) -> &ecs::World {
        &self.scene_realities.instantiated
    }

    pub(crate) fn world_mut(&mut self) -> &mut ecs::World {
        &mut self.scene_realities.instantiated
    }

    pub fn document(&self) -> &SceneDocumentState {
        &self.scene_realities.authored
    }

    pub fn ids(&self) -> &EditorRuntimeIdRegistry {
        &self.scene_realities.identities
    }

    pub fn command_store(&self) -> &SceneCommandStore {
        &self.command_store
    }

    pub(crate) fn command_store_mut(&mut self) -> &mut SceneCommandStore {
        &mut self.command_store
    }

    pub fn ratified_change_log(&self) -> &RatifiedChangeLog {
        &self.ratified_changes
    }

    pub fn session_change_log(&self) -> &editor_core::SessionChangeLog {
        &self.session_changes
    }

    pub fn workflow_log(&self) -> &editor_core::WorkflowLog {
        &self.workflow
    }

    pub fn sharing_policy(&self) -> editor_core::SharingPolicy {
        self.sharing_policy
    }

    pub fn set_sharing_policy(&mut self, policy: editor_core::SharingPolicy) {
        self.sharing_policy = policy;
    }

    pub fn queued_shared_change_count(&self) -> usize {
        self.shared_changes.len()
    }

    pub fn drain_shared_changes(&mut self) -> Vec<editor_core::SharedChangeEnvelope> {
        self.shared_changes.drain()
    }

    pub fn propagate_shared_changes<S>(
        &mut self,
        sink: &mut S,
    ) -> Result<usize, <S as editor_core::SharedChangePropagationSink>::Error>
    where
        S: editor_core::SharedChangePropagationSink,
    {
        let mut propagated = 0usize;
        while let Some(envelope) = self.shared_changes.pop_front() {
            if let Err(error) = sink.push_shared_change(envelope.clone()) {
                self.shared_changes.enqueue_front(envelope);
                return Err(error);
            }
            propagated += 1;
        }

        Ok(propagated)
    }

    pub fn reconcile_shared_change(
        &mut self,
        envelope: editor_core::SharedChangeEnvelope,
    ) -> editor_core::ReconciliationDecision {
        let current_version = self.current_scene_reality_version();
        let decision = editor_core::evaluate_reconciliation(&envelope.change, current_version);

        self.record_workflow_event(editor_core::WorkflowEventKind::SharedChangeReconciled {
            sequence: envelope.sequence,
            result: decision.result,
        });

        if decision.is_accepted() {
            self.ratified_changes.push(envelope.change.clone());
            self.adopt_scene_reality_version(envelope.change.result_version);
            self.record_workflow_event(editor_core::WorkflowEventKind::RatifiedChangeRecorded {
                ratification_id: envelope.change.ratification_id,
            });
        }

        decision
    }

    pub fn last_ratified_change(&self) -> Option<&editor_core::RatifiedChange> {
        self.ratified_changes.last()
    }

    pub(crate) fn record_ratified_change(&mut self, change: editor_core::RatifiedChange) {
        self.ratified_changes.push(change.clone());

        if change.propagation_structure == editor_core::PropagationStructure::SessionBroadcast {
            let sequence = editor_core::SharedChangeSequence(self.next_shared_change_sequence);
            self.next_shared_change_sequence += 1;
            self.shared_changes
                .enqueue(editor_core::SharedChangeEnvelope::new(
                    sequence,
                    change.clone(),
                ));
            self.record_workflow_event(editor_core::WorkflowEventKind::SharedChangeQueued {
                sequence,
            });
        }

        self.record_workflow_event(editor_core::WorkflowEventKind::RatifiedChangeRecorded {
            ratification_id: change.ratification_id,
        });
    }

    pub(crate) fn record_session_change(
        &mut self,
        origin: editor_core::ChangeOrigin,
        kind: editor_core::SessionChangeKind,
    ) {
        let id = editor_core::SessionChangeId(self.next_session_change_id);
        self.next_session_change_id += 1;
        self.session_changes
            .push(editor_core::SessionChange::new(id, origin, kind));
    }

    pub(crate) fn record_workflow_event(&mut self, kind: editor_core::WorkflowEventKind) {
        let id = editor_core::WorkflowEventId(self.next_workflow_event_id);
        self.next_workflow_event_id += 1;
        self.workflow
            .push(editor_core::WorkflowEvent::new(id, kind));
    }

    pub fn inspector_bridge(&self) -> RunenwerkEditorInspectorBridge<'_> {
        RunenwerkEditorInspectorBridge::new(&self.scene_realities.identities)
    }

    pub(crate) fn scene_runtime(&mut self) -> RunenwerkEditorSceneRuntime<'_> {
        RunenwerkEditorSceneRuntime::new(
            &mut self.scene_realities.authored,
            &mut self.scene_realities.instantiated,
            &mut self.scene_realities.identities,
        )
    }

    pub(crate) fn session_and_scene_runtime(
        &mut self,
    ) -> (&mut EditorSession, RunenwerkEditorSceneRuntime<'_>) {
        let session = &mut self.session;
        let scene_runtime = RunenwerkEditorSceneRuntime::new(
            &mut self.scene_realities.authored,
            &mut self.scene_realities.instantiated,
            &mut self.scene_realities.identities,
        );
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
        self.scene_realities
            .authored
            .register_entity(editor_id, display_name.clone(), parent)
            .expect("document entity registration failed");
        self.scene_realities.identities.register_entity(
            editor_id,
            ecs_entity,
            display_name,
            parent,
        );
    }

    pub fn register_component_type<T>(&mut self, editor_id: ComponentTypeId)
    where
        T: ecs::Component + ecs::Reflect + Default + 'static,
    {
        self.scene_realities
            .instantiated
            .register_component_type::<T>();
        self.scene_realities
            .identities
            .register_component_type::<T>(editor_id);
    }

    pub fn register_resource_type<T>(&mut self, editor_id: ResourceTypeId)
    where
        T: ecs::Resource + ecs::Reflect + 'static,
    {
        self.scene_realities
            .instantiated
            .register_resource_type::<T>();
        self.scene_realities
            .identities
            .register_resource_type::<T>(editor_id);
    }

    pub fn list_scene_entities(&self) -> Vec<SceneEntityView> {
        all_entity_views(&self.scene_realities.authored)
    }

    pub fn hierarchy_snapshot(&self) -> HierarchySnapshot {
        build_hierarchy_snapshot(&self.scene_realities.authored)
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
        validate_reparent(&self.scene_realities.authored, entity, new_parent)
    }

    pub fn entity_has_component(&self, entity: EntityId, component_type: ComponentTypeId) -> bool {
        let Some(ecs_entity) = self.scene_realities.identities.resolve_entity(entity) else {
            return false;
        };

        let Some(rust_type_id) = self
            .scene_realities
            .identities
            .resolve_component_rust_type_id(component_type)
        else {
            return false;
        };

        self.scene_realities
            .instantiated
            .reflected_component_value_ref(ecs_entity, rust_type_id)
            .is_some()
    }

    pub fn list_entity_components(&self, entity: EntityId) -> Vec<SceneComponentDescriptor> {
        let mut components = self
            .scene_realities
            .identities
            .component_type_ids()
            .filter(|component_type| self.entity_has_component(entity, *component_type))
            .filter_map(|component_type| {
                let display_name = self
                    .scene_realities
                    .identities
                    .component_display_name(component_type)?;
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

    pub(crate) fn allocate_command_id(&mut self) -> editor_core::CommandId {
        let id = editor_core::CommandId(self.next_command_id);
        self.next_command_id += 1;
        id
    }

    pub(crate) fn allocate_transaction_id(&mut self) -> editor_core::TransactionId {
        let id = editor_core::TransactionId(self.next_transaction_id);
        self.next_transaction_id += 1;
        id
    }

    pub(crate) fn allocate_ratification_id(&mut self) -> editor_core::RatificationId {
        let id = editor_core::RatificationId(self.next_ratification_id);
        self.next_ratification_id += 1;
        id
    }

    pub(crate) fn allocate_causality_id(&mut self) -> editor_core::CausalityId {
        let id = editor_core::CausalityId(self.next_causality_id);
        self.next_causality_id += 1;
        id
    }

    pub fn current_scene_reality_version(&self) -> editor_core::RealityVersion {
        editor_core::RealityVersion(self.scene_reality_version)
    }

    pub(crate) fn advance_scene_reality_version(&mut self) -> editor_core::RealityVersion {
        self.scene_reality_version += 1;
        editor_core::RealityVersion(self.scene_reality_version)
    }

    pub(crate) fn adopt_scene_reality_version(&mut self, version: editor_core::RealityVersion) {
        if version.0 > self.scene_reality_version {
            self.scene_reality_version = version.0;
        }
    }

    pub fn list_registered_component_types(&self) -> Vec<(ComponentTypeId, String)> {
        let mut component_types = self
            .scene_realities
            .identities
            .component_type_ids()
            .filter_map(|component_type| {
                let display_name = self
                    .scene_realities
                    .identities
                    .component_display_name(component_type)?;
                Some((component_type, display_name.to_string()))
            })
            .collect::<Vec<_>>();

        component_types
            .sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(&right.0)));

        component_types
    }
}
