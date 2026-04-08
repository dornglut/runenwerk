use editor_core::{Command, CommandExecutor, CommandId, ComponentTypeId, EntityId, TransactionId};
use editor_inspector::{InspectorEditValue, InspectorPath};
use editor_scene::{
    SceneCommandContext, SceneCommandIntent, SceneEditorCommand, scene_intent_to_command,
};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, execute_scene_command, execute_scene_command_and_push_history,
    execute_scene_intent, redo_last_scene_transaction, undo_last_scene_transaction,
};

use super::shared::Position;

#[test]
fn scene_editing_vertical_slice_create_add_edit_remove_and_undo_remove() {
    let mut runtime = RunenwerkEditorRuntime::new();
    let component_type = ComponentTypeId(10);

    runtime.register_component_type::<Position>(component_type);

    execute_scene_intent(
        &mut runtime,
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Player".to_string(),
        },
    )
    .expect("create entity command should succeed");

    let editor_entity = EntityId(1);
    let ecs_entity = runtime
        .ids()
        .resolve_entity(editor_entity)
        .expect("editor entity should be mapped to an ecs entity");

    execute_scene_intent(
        &mut runtime,
        CommandId(2),
        SceneCommandIntent::AddComponent {
            entity: editor_entity,
            component_type,
        },
    )
    .expect("add component command should succeed");

    let created_component = runtime
        .world()
        .get::<Position>(ecs_entity)
        .expect("position component should exist after add");

    assert_eq!(created_component.speed, 0.0);
    assert_eq!(created_component.label, "");
    assert_eq!(created_component.value.x, 0.0);
    assert_eq!(created_component.value.y, 0.0);

    execute_scene_command(
        &mut runtime,
        SceneEditorCommand::new_edit_component_field(
            CommandId(3),
            "Edit Position Speed",
            editor_entity,
            component_type,
            InspectorPath::root().child_field("speed"),
            InspectorEditValue::Float(7.0),
        ),
    )
    .expect("edit speed command should succeed");

    execute_scene_command(
        &mut runtime,
        SceneEditorCommand::new_edit_component_field(
            CommandId(4),
            "Edit Position Label",
            editor_entity,
            component_type,
            InspectorPath::root().child_field("label"),
            InspectorEditValue::Text("Hero".to_string()),
        ),
    )
    .expect("edit label command should succeed");

    execute_scene_command(
        &mut runtime,
        SceneEditorCommand::new_edit_component_field(
            CommandId(5),
            "Edit Position X",
            editor_entity,
            component_type,
            InspectorPath::root().child_field("value").child_field("x"),
            InspectorEditValue::Float(3.5),
        ),
    )
    .expect("edit nested field command should succeed");

    {
        let edited_component = runtime
            .world()
            .get::<Position>(ecs_entity)
            .expect("position component should still exist after edits");

        assert_eq!(edited_component.speed, 7.0);
        assert_eq!(edited_component.label, "Hero");
        assert_eq!(edited_component.value.x, 3.5);
        assert_eq!(edited_component.value.y, 0.0);
    }

    let mut remove_command = scene_intent_to_command(
        CommandId(6),
        SceneCommandIntent::RemoveComponent {
            entity: editor_entity,
            component_type,
        },
    );

    {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        CommandExecutor::execute_command(&mut ctx, &mut remove_command)
            .expect("remove component command should execute");
    }

    assert!(runtime.world().get::<Position>(ecs_entity).is_none());

    {
        let (session, mut scene_runtime) = runtime.session_and_scene_runtime();
        let mut ctx = SceneCommandContext::new(session, &mut scene_runtime);

        remove_command
            .undo(&mut ctx)
            .expect("undo remove component should restore prior value");
    }

    let restored_component = runtime
        .world()
        .get::<Position>(ecs_entity)
        .expect("position component should be restored after undo");

    assert_eq!(restored_component.speed, 7.0);
    assert_eq!(restored_component.label, "Hero");
    assert_eq!(restored_component.value.x, 3.5);
    assert_eq!(restored_component.value.y, 0.0);
}

#[test]
fn undo_redo_replays_stored_scene_transaction() {
    let mut runtime = RunenwerkEditorRuntime::new();
    let component_type = ComponentTypeId(20);

    runtime.register_component_type::<Position>(component_type);

    execute_scene_command_and_push_history(
        &mut runtime,
        scene_intent_to_command(
            CommandId(10),
            SceneCommandIntent::CreateEntity {
                parent: None,
                display_name: "Player".to_string(),
            },
        ),
        "Create Entity",
        TransactionId(100),
    )
    .expect("create entity with history should succeed");

    execute_scene_command_and_push_history(
        &mut runtime,
        scene_intent_to_command(
            CommandId(11),
            SceneCommandIntent::AddComponent {
                entity: EntityId(1),
                component_type,
            },
        ),
        "Add Component",
        TransactionId(101),
    )
    .expect("add component with history should succeed");

    let ecs_entity = runtime
        .ids()
        .resolve_entity(EntityId(1))
        .expect("entity should exist");

    assert!(runtime.world().get::<Position>(ecs_entity).is_some());

    let undone = undo_last_scene_transaction(&mut runtime)
        .expect("undo should succeed")
        .expect("undo should return history entry");
    assert_eq!(undone.transaction.id, TransactionId(101));
    assert!(runtime.world().get::<Position>(ecs_entity).is_none());

    let redone = redo_last_scene_transaction(&mut runtime)
        .expect("redo should succeed")
        .expect("redo should return history entry");
    assert_eq!(redone.transaction.id, TransactionId(101));
    assert!(runtime.world().get::<Position>(ecs_entity).is_some());
}
