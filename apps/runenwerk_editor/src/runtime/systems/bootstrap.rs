use std::path::PathBuf;

use engine::plugins::render::ShaderRegistryResource;
use engine::runtime::ResMut;

use crate::editor_runtime::{bootstrap_mvp_scene_if_empty, register_mvp_component_types};
use crate::runtime::resources::EditorHostResource;

pub fn bootstrap_editor_demo_system(
    mut host: ResMut<EditorHostResource>,
    mut shader_registry: ResMut<ShaderRegistryResource>,
) {
    initialize_editor_shader_root(&mut shader_registry);
    register_mvp_component_types(host.app.runtime_mut());
    if let Err(error) = bootstrap_mvp_scene_if_empty(host.app.runtime_mut()) {
        eprintln!("editor mvp bootstrap failed: {error}");
    }
}

fn initialize_editor_shader_root(shader_registry: &mut ShaderRegistryResource) {
    let workspace_shader_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/shaders");
    let shader_root = workspace_shader_root
        .canonicalize()
        .unwrap_or(workspace_shader_root);
    shader_registry.add_root(shader_root.to_string_lossy().to_string());
}
