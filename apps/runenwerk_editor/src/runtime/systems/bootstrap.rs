use editor_core::CommandId;
use editor_scene::SceneCommandIntent;
use engine::runtime::ResMut;

use crate::editor_runtime::execute_scene_intent;
use crate::runtime::resources::EditorHostResource;

pub fn bootstrap_editor_demo_system(mut host: ResMut<EditorHostResource>) {
    if !host.app.outliner_state().rows.is_empty() {
        return;
    }

    let create_root_result = execute_scene_intent(
        host.app.runtime_mut(),
        CommandId(1),
        SceneCommandIntent::CreateEntity {
            parent: None,
            display_name: "Root".to_string(),
        },
    );

    if let Err(error) = create_root_result {
        eprintln!("editor demo bootstrap root creation failed: {error}");
        return;
    }

    if let Err(error) = execute_scene_intent(
        host.app.runtime_mut(),
        CommandId(2),
        SceneCommandIntent::CreateEntity {
            parent: Some(editor_core::EntityId(1)),
            display_name: "Child".to_string(),
        },
    ) {
        eprintln!("editor demo bootstrap child creation failed: {error}");
    }
}
