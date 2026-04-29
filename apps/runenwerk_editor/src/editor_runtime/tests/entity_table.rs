use editor_core::{ChangeOrigin, CommandId, EntityId, SelectionTarget, SessionChangeKind};
use editor_scene::SceneCommandIntent;
use editor_shell::EntityTableSortKey;

use crate::editor_app::RunenwerkEditorApp;
use crate::editor_panels::EntityTablePanelCommand;
use crate::editor_runtime::{
    EDITOR_PRIMITIVE_COMPONENT_TYPE_ID, LOCAL_TRANSFORM_COMPONENT_TYPE_ID, execute_scene_intent,
    register_mvp_component_types,
};

#[test]
fn entity_table_panel_filters_sorts_and_counts_components() {
    let mut app = RunenwerkEditorApp::new();
    seed_entity_table_scene(&mut app);

    let initial = app.entity_table_state();
    assert_eq!(
        initial
            .rows
            .iter()
            .map(|row| row.display_name.as_str())
            .collect::<Vec<_>>(),
        vec!["Camera", "Child Light", "Player Root"],
    );

    let child = initial
        .rows
        .iter()
        .find(|row| row.entity == EntityId(2))
        .expect("child row should exist");
    assert_eq!(child.parent, Some(EntityId(1)));
    assert_eq!(child.component_count, 2);
    assert!(!child.is_selected);

    app.dispatch_entity_table_command(EntityTablePanelCommand::AppendSearchText {
        text: "root".to_string(),
    })
    .expect("search update should succeed");
    let filtered = app.entity_table_state();
    assert_eq!(filtered.search_query, "root");
    assert_eq!(filtered.rows.len(), 1);
    assert_eq!(filtered.rows[0].entity, EntityId(1));

    app.dispatch_entity_table_command(EntityTablePanelCommand::BackspaceSearchQuery)
        .expect("search backspace should succeed");
    let narrowed = app.entity_table_state();
    assert_eq!(narrowed.search_query, "roo");
    assert_eq!(narrowed.rows.len(), 1);
}

#[test]
fn entity_table_panel_sort_and_select_use_entity_table_origin() {
    let mut app = RunenwerkEditorApp::new();
    seed_entity_table_scene(&mut app);

    let sorted = app
        .dispatch_entity_table_command(EntityTablePanelCommand::ToggleSort {
            sort_key: EntityTableSortKey::ComponentCount,
        })
        .expect("sort update should succeed")
        .state;
    assert_eq!(
        sorted
            .rows
            .iter()
            .map(|row| (row.entity, row.component_count))
            .collect::<Vec<_>>(),
        vec![(EntityId(3), 0), (EntityId(1), 1), (EntityId(2), 2)],
    );

    app.dispatch_entity_table_command(EntityTablePanelCommand::SelectEntity {
        entity: EntityId(2),
    })
    .expect("entity table select should succeed");

    assert_eq!(
        app.runtime().session().selection().primary(),
        Some(&SelectionTarget::Entity(EntityId(2)))
    );
    assert!(matches!(
        app.runtime()
            .session_change_log()
            .last()
            .map(|change| (change.origin, change.kind.clone())),
        Some((
            ChangeOrigin::EntityTablePanel,
            SessionChangeKind::SelectionSetSingle {
                target: SelectionTarget::Entity(EntityId(2))
            }
        ))
    ));
    assert!(
        app.entity_table_state()
            .rows
            .iter()
            .any(|row| row.entity == EntityId(2) && row.is_selected)
    );
}

fn seed_entity_table_scene(app: &mut RunenwerkEditorApp) {
    register_mvp_component_types(app.runtime_mut());
    execute_scene_intent(
        app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Player Root".to_string(),
        },
    )
    .expect("root create should succeed");
    execute_scene_intent(
        app.runtime_mut(),
        CommandId(2),
        SceneCommandIntent::CreateEntity {
            parent: Some(EntityId(1)),
            display_name: "Child Light".to_string(),
        },
    )
    .expect("child create should succeed");
    execute_scene_intent(
        app.runtime_mut(),
        CommandId(3),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Camera".to_string(),
        },
    )
    .expect("camera create should succeed");
    execute_scene_intent(
        app.runtime_mut(),
        CommandId(4),
        SceneCommandIntent::AddComponent {
            entity: EntityId(1),
            component_type: LOCAL_TRANSFORM_COMPONENT_TYPE_ID,
        },
    )
    .expect("root component add should succeed");
    execute_scene_intent(
        app.runtime_mut(),
        CommandId(5),
        SceneCommandIntent::AddComponent {
            entity: EntityId(2),
            component_type: LOCAL_TRANSFORM_COMPONENT_TYPE_ID,
        },
    )
    .expect("child transform add should succeed");
    execute_scene_intent(
        app.runtime_mut(),
        CommandId(6),
        SceneCommandIntent::AddComponent {
            entity: EntityId(2),
            component_type: EDITOR_PRIMITIVE_COMPONENT_TYPE_ID,
        },
    )
    .expect("child primitive add should succeed");
}
