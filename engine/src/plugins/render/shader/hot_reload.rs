use super::ShaderRegistryResource;

pub fn poll_shader_hot_reload(registry: &mut ShaderRegistryResource) -> Vec<String> {
    registry.poll_updates()
}
