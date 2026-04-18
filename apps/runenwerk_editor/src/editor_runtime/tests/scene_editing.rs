use editor_core::{
    Command, CommandExecutor, CommandId, ComponentTypeId, EntityId, SemanticOperation,
    TransactionId,
};
use editor_inspector::{InspectorEditValue, InspectorPath};
use editor_scene::{SceneCommandIntent, SceneEditorCommand, scene_intent_to_command};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, execute_scene_command, execute_scene_intent,
    ratify_scene_command_with_transaction_id, ratify_scene_redo, ratify_scene_undo,
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

    runtime
        .with_scene_command_context(|ctx| {
            CommandExecutor::execute_command(ctx, &mut remove_command)
        })
        .expect("remove component command should execute");

    assert!(runtime.world().get::<Position>(ecs_entity).is_none());

    runtime
        .with_scene_command_context(|ctx| remove_command.undo(ctx))
        .expect("undo remove component should restore prior value");

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

    ratify_scene_command_with_transaction_id(
        &mut runtime,
        "Create Entity",
        scene_intent_to_command(
            CommandId(10),
            SceneCommandIntent::CreateEntity {
                parent: None,
                display_name: "Player".to_string(),
            },
        ),
        TransactionId(100),
        editor_core::ChangeOrigin::Runtime,
    )
    .expect("create entity with history should succeed");

    let add_change = ratify_scene_command_with_transaction_id(
        &mut runtime,
        "Add Component",
        scene_intent_to_command(
            CommandId(11),
            SceneCommandIntent::AddComponent {
                entity: EntityId(1),
                component_type,
            },
        ),
        TransactionId(101),
        editor_core::ChangeOrigin::Runtime,
    )
    .expect("add component with history should succeed")
    .expect("add component should ratify");

    let ecs_entity = runtime
        .ids()
        .resolve_entity(EntityId(1))
        .expect("entity should exist");

    assert!(runtime.world().get::<Position>(ecs_entity).is_some());

    let undone = ratify_scene_undo(&mut runtime, editor_core::ChangeOrigin::Runtime)
        .expect("undo should succeed")
        .expect("undo should return history entry");
    assert_eq!(undone.transaction.id, TransactionId(101));
    assert_eq!(undone.causality_id, add_change.causality_id);
    assert_eq!(
        undone.semantic_operations,
        vec![SemanticOperation::SceneTransactionUndone]
    );
    assert!(runtime.world().get::<Position>(ecs_entity).is_none());

    let redone = ratify_scene_redo(&mut runtime, editor_core::ChangeOrigin::Runtime)
        .expect("redo should succeed")
        .expect("redo should return history entry");
    assert_eq!(redone.transaction.id, TransactionId(101));
    assert_eq!(redone.causality_id, add_change.causality_id);
    assert_eq!(
        redone.semantic_operations,
        vec![SemanticOperation::SceneTransactionRedone]
    );
    assert!(runtime.world().get::<Position>(ecs_entity).is_some());

    assert_eq!(runtime.ratified_change_log().len(), 4);
    assert_eq!(
        runtime
            .last_ratified_change()
            .expect("redo ratification should be retained")
            .semantic_operations,
        vec![SemanticOperation::SceneTransactionRedone]
    );
}
