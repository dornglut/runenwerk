use super::{
    RenderFragmentPackageDescriptor, RenderFragmentRegistryResource, RenderFragmentReloadOutcome,
};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFragmentHotReloadRequest {
    pub package: RenderFragmentPackageDescriptor,
}

impl RenderFragmentHotReloadRequest {
    pub fn new(package: RenderFragmentPackageDescriptor) -> Self {
        Self { package }
    }
}

pub fn apply_render_fragment_hot_reload(
    registry: &mut RenderFragmentRegistryResource,
    request: RenderFragmentHotReloadRequest,
) -> RenderFragmentReloadOutcome {
    registry.apply_package(request.package)
}
