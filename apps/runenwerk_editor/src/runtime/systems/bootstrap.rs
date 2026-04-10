use engine::runtime::ResMut;

use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
use crate::runtime::resources::EditorHostResource;

pub fn bootstrap_editor_demo_system(mut host: ResMut<EditorHostResource>) {
    register_mvp_component_types(host.app.runtime_mut());
    if let Err(error) = bootstrap_mvp_scene_if_empty(host.app.runtime_mut()) {
        eprintln!("editor mvp bootstrap failed: {error}");
    }
}
