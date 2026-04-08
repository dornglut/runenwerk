//! File: domain/editor/editor_scene/src/command.rs
//! Purpose: Executable scene command layer backed by the scene runtime bridge.

use editor_core::{Command, CommandId, CommandMetadata, CommandOutcome};
use editor_inspector::{InspectorEditValue, InspectorPath};

use crate::{
    AddComponentCommand, CreateEntityCommand, DeleteEntityCommand, EditComponentFieldCommand,
    EditResourceFieldCommand, RemoveComponentCommand, RenameEntityCommand, ReparentEntityCommand,
    SceneCommandContext, SceneCommandIntent,
};

#[derive(Debug, Clone, PartialEq)]
enum SceneCommandKind {
    CreateEntity(CreateEntityCommand),
    DeleteEntity(DeleteEntityCommand),
    AddComponent(AddComponentCommand),
    RemoveComponent(RemoveComponentCommand),
    ReparentEntity(ReparentEntityCommand),
    EditComponentField(EditComponentFieldCommand),
    EditResourceField(EditResourceFieldCommand),
    RenameEntity(RenameEntityCommand),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneEditorCommand {
    metadata: CommandMetadata,
    kind: SceneCommandKind,
}

impl SceneEditorCommand {
    /// File: domain/editor/editor_scene/src/command.rs
    /// Method: from_intent
    pub fn from_intent(
        id: CommandId,
        label: impl Into<String>,
        intent: SceneCommandIntent,
    ) -> Self {
        let metadata = CommandMetadata::new(id, label);

        let kind = match intent {
            SceneCommandIntent::CreateEntity {
                parent,
                display_name,
            } => SceneCommandKind::CreateEntity(CreateEntityCommand::new(parent, display_name)),
            SceneCommandIntent::DeleteEntity { entity } => {
                SceneCommandKind::DeleteEntity(DeleteEntityCommand::new(entity))
            }
            SceneCommandIntent::AddComponent {
                entity,
                component_type,
            } => SceneCommandKind::AddComponent(AddComponentCommand::new(entity, component_type)),
            SceneCommandIntent::RemoveComponent {
                entity,
                component_type,
            } => SceneCommandKind::RemoveComponent(RemoveComponentCommand::new(
                entity,
                component_type,
            )),
            SceneCommandIntent::ReparentEntity { entity, new_parent } => {
                SceneCommandKind::ReparentEntity(ReparentEntityCommand::new(entity, new_parent))
            }
            SceneCommandIntent::EditComponentField {
                entity,
                component_type,
                path,
                value,
            } => SceneCommandKind::EditComponentField(EditComponentFieldCommand::new(
                entity,
                component_type,
                path,
                value,
            )),
            SceneCommandIntent::EditResourceField {
                resource_type,
                path,
                value,
            } => SceneCommandKind::EditResourceField(EditResourceFieldCommand::new(
                resource_type,
                path,
                value,
            )),
            SceneCommandIntent::RenameEntity {
                entity,
                new_display_name,
            } => SceneCommandKind::RenameEntity(RenameEntityCommand::new(entity, new_display_name)),
        };

        Self { metadata, kind }
    }

    /// File: domain/editor/editor_scene/src/command.rs
    /// Method: new_edit_component_field
    pub fn new_edit_component_field(
        id: CommandId,
        label: impl Into<String>,
        entity: editor_core::EntityId,
        component_type: editor_core::ComponentTypeId,
        path: InspectorPath,
        value: InspectorEditValue,
    ) -> Self {
        Self {
            metadata: CommandMetadata::new(id, label),
            kind: SceneCommandKind::EditComponentField(EditComponentFieldCommand::new(
                entity,
                component_type,
                path,
                value,
            )),
        }
    }
}

impl Command for SceneEditorCommand {
    type Error = &'static str;
    type Context<'a> = SceneCommandContext<'a>;

    fn metadata(&self) -> &CommandMetadata {
        &self.metadata
    }

    fn apply<'a>(&mut self, ctx: &mut Self::Context<'a>) -> Result<CommandOutcome, Self::Error> {
        match &mut self.kind {
            SceneCommandKind::CreateEntity(command) => command.apply(ctx)?,
            SceneCommandKind::DeleteEntity(command) => command.apply(ctx)?,
            SceneCommandKind::AddComponent(command) => command.apply(ctx)?,
            SceneCommandKind::RemoveComponent(command) => command.apply(ctx)?,
            SceneCommandKind::ReparentEntity(command) => command.apply(ctx)?,
            SceneCommandKind::EditComponentField(command) => command.apply(ctx)?,
            SceneCommandKind::EditResourceField(command) => command.apply(ctx)?,
            SceneCommandKind::RenameEntity(command) => command.apply(ctx)?,
        }

        Ok(CommandOutcome::Applied)
    }

    fn undo<'a>(&mut self, ctx: &mut Self::Context<'a>) -> Result<CommandOutcome, Self::Error> {
        match &mut self.kind {
            SceneCommandKind::CreateEntity(command) => command.undo(ctx)?,
            SceneCommandKind::DeleteEntity(command) => command.undo(ctx)?,
            SceneCommandKind::AddComponent(command) => command.undo(ctx)?,
            SceneCommandKind::RemoveComponent(command) => command.undo(ctx)?,
            SceneCommandKind::ReparentEntity(command) => command.undo(ctx)?,
            SceneCommandKind::EditComponentField(command) => command.undo(ctx)?,
            SceneCommandKind::EditResourceField(command) => command.undo(ctx)?,
            SceneCommandKind::RenameEntity(command) => command.undo(ctx)?,
        }

        Ok(CommandOutcome::Applied)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use editor_core::{Command, ComponentTypeId, EntityId, ResourceTypeId};
    use editor_inspector::{InspectorEditError, InspectorEditValue, InspectorPath};

    use super::*;
    use crate::{SceneEntitySnapshot, SceneRuntime};

    #[derive(Default)]
    struct MockSceneRuntime {
        next_entity_id: u64,
        entities: BTreeMap<EntityId, SceneEntitySnapshot>,
        component_fields: BTreeMap<(EntityId, ComponentTypeId, String), InspectorEditValue>,
        resource_fields: BTreeMap<(ResourceTypeId, String), InspectorEditValue>,
    }

    impl MockSceneRuntime {
        fn key(path: &InspectorPath) -> String {
            path.stable_key()
        }
    }

    impl SceneRuntime for MockSceneRuntime {
        fn create_entity(
            &mut self,
            parent: Option<EntityId>,
            display_name: &str,
        ) -> Result<EntityId, &'static str> {
            self.next_entity_id += 1;
            let id = EntityId(self.next_entity_id);
            self.entities.insert(
                id,
                SceneEntitySnapshot::new(id, display_name.to_string(), parent),
            );
            Ok(id)
        }

        fn restore_entity(&mut self, snapshot: SceneEntitySnapshot) -> Result<(), &'static str> {
            self.entities.insert(snapshot.id, snapshot);
            Ok(())
        }

        fn delete_entity(&mut self, entity: EntityId) -> Result<SceneEntitySnapshot, &'static str> {
            self.entities.remove(&entity).ok_or("entity not found")
        }

        fn reparent_entity(
            &mut self,
            entity: EntityId,
            new_parent: Option<EntityId>,
        ) -> Result<Option<EntityId>, &'static str> {
            let entry = self.entities.get_mut(&entity).ok_or("entity not found")?;
            let old_parent = entry.parent;
            entry.parent = new_parent;
            Ok(old_parent)
        }

        fn add_component(
            &mut self,
            entity: EntityId,
            component_type: ComponentTypeId,
        ) -> Result<(), &'static str> {
            self.component_fields.insert(
                (entity, component_type, "root".to_string()),
                InspectorEditValue::Text("component".to_string()),
            );
            Ok(())
        }

        fn remove_component(
            &mut self,
            entity: EntityId,
            component_type: ComponentTypeId,
        ) -> Result<crate::SceneComponentSnapshot, &'static str> {
            self.component_fields
                .retain(|(stored_entity, stored_type, _), _| {
                    *stored_entity != entity || *stored_type != component_type
                });

            Ok(crate::SceneComponentSnapshot::new(
                entity,
                component_type,
                "component",
            ))
        }

        fn restore_component(
            &mut self,
            snapshot: crate::SceneComponentSnapshot,
        ) -> Result<(), &'static str> {
            self.component_fields.insert(
                (snapshot.entity, snapshot.component_type, "root".to_string()),
                InspectorEditValue::Text(snapshot.display_name),
            );
            Ok(())
        }

        fn read_component_field(
            &self,
            entity: EntityId,
            component_type: ComponentTypeId,
            path: &InspectorPath,
        ) -> Result<InspectorEditValue, InspectorEditError> {
            self.component_fields
                .get(&(entity, component_type, Self::key(path)))
                .cloned()
                .ok_or(InspectorEditError::InvalidPath)
        }

        fn write_component_field(
            &mut self,
            entity: EntityId,
            component_type: ComponentTypeId,
            path: &InspectorPath,
            value: InspectorEditValue,
        ) -> Result<(), InspectorEditError> {
            self.component_fields
                .insert((entity, component_type, Self::key(path)), value);
            Ok(())
        }

        fn read_resource_field(
            &self,
            resource_type: ResourceTypeId,
            path: &InspectorPath,
        ) -> Result<InspectorEditValue, InspectorEditError> {
            self.resource_fields
                .get(&(resource_type, Self::key(path)))
                .cloned()
                .ok_or(InspectorEditError::InvalidPath)
        }

        fn write_resource_field(
            &mut self,
            resource_type: ResourceTypeId,
            path: &InspectorPath,
            value: InspectorEditValue,
        ) -> Result<(), InspectorEditError> {
            self.resource_fields
                .insert((resource_type, Self::key(path)), value);
            Ok(())
        }

        fn rename_entity(
            &mut self,
            entity: EntityId,
            new_display_name: &str,
        ) -> Result<String, &'static str> {
            let entry = self.entities.get_mut(&entity).ok_or("entity not found")?;
            let previous = std::mem::replace(&mut entry.display_name, new_display_name.to_string());
            Ok(previous)
        }
    }

    #[test]
    fn create_entity_command_applies_and_undoes() {
        let mut session = editor_core::EditorSession::new();
        let mut runtime = MockSceneRuntime::default();
        let mut ctx = SceneCommandContext::new(&mut session, &mut runtime);

        let mut command = SceneEditorCommand::from_intent(
            editor_core::CommandId(1),
            "Create Entity",
            crate::SceneCommandIntent::CreateEntity {
                parent: None,
                display_name: "Player".to_string(),
            },
        );

        command.apply(&mut ctx).expect("apply should succeed");
        assert_eq!(
            ctx.runtime()
                .read_resource_field(ResourceTypeId(999), &InspectorPath::root())
                .err(),
            Some(InspectorEditError::InvalidPath)
        );

        assert!(ctx.runtime_mut().delete_entity(EntityId(1)).is_ok());
        ctx.runtime_mut()
            .restore_entity(SceneEntitySnapshot::new(EntityId(1), "Player", None))
            .expect("restore should succeed");

        command.undo(&mut ctx).expect("undo should succeed");
        assert!(ctx.runtime_mut().delete_entity(EntityId(1)).is_err());
    }

    #[test]
    fn edit_component_field_command_round_trips() {
        let mut session = editor_core::EditorSession::new();
        let mut runtime = MockSceneRuntime::default();
        let entity = runtime
            .create_entity(None, "Player")
            .expect("entity create should succeed");
        let component_type = ComponentTypeId(10);
        let path = InspectorPath::root().child_field("speed");

        runtime
            .write_component_field(
                entity,
                component_type,
                &path,
                InspectorEditValue::Float(3.5),
            )
            .expect("seed value should be written");

        let mut ctx = SceneCommandContext::new(&mut session, &mut runtime);

        let mut command = SceneEditorCommand::from_intent(
            editor_core::CommandId(2),
            "Edit Component Field",
            crate::SceneCommandIntent::EditComponentField {
                entity,
                component_type,
                path: path.clone(),
                value: InspectorEditValue::Float(7.0),
            },
        );

        command.apply(&mut ctx).expect("apply should succeed");

        let edited = ctx
            .runtime()
            .read_component_field(entity, component_type, &path)
            .expect("edited value should exist");
        assert_eq!(edited, InspectorEditValue::Float(7.0));

        command.undo(&mut ctx).expect("undo should succeed");

        let restored = ctx
            .runtime()
            .read_component_field(entity, component_type, &path)
            .expect("restored value should exist");
        assert_eq!(restored, InspectorEditValue::Float(3.5));
    }

    #[test]
    fn rename_entity_command_round_trips() {
        let mut session = editor_core::EditorSession::new();
        let mut runtime = MockSceneRuntime::default();
        let entity = runtime
            .create_entity(None, "Player")
            .expect("entity create should succeed");

        let mut ctx = SceneCommandContext::new(&mut session, &mut runtime);

        let mut command = SceneEditorCommand::from_intent(
            editor_core::CommandId(3),
            "Rename Entity",
            crate::SceneCommandIntent::RenameEntity {
                entity,
                new_display_name: "Hero".to_string(),
            },
        );

        command.apply(&mut ctx).expect("rename should succeed");
        assert_eq!(
            ctx.runtime_mut()
                .delete_entity(entity)
                .expect("entity should exist")
                .display_name,
            "Hero"
        );

        ctx.runtime_mut()
            .restore_entity(SceneEntitySnapshot::new(entity, "Hero", None))
            .expect("restore should succeed");

        command.undo(&mut ctx).expect("undo rename should succeed");
        assert_eq!(
            ctx.runtime_mut()
                .delete_entity(entity)
                .expect("entity should exist after undo")
                .display_name,
            "Player"
        );
    }
}
