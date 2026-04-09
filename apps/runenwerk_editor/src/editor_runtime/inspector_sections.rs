use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_inspector::{
    EcsInspectorAdapter, InspectTarget, InspectorAdapter, InspectorAdapterError, InspectorSection,
};

use crate::editor_runtime::RunenwerkEditorRuntime;

pub fn build_component_inspector_sections(
    runtime: &RunenwerkEditorRuntime,
    entity: EntityId,
    component_type: ComponentTypeId,
) -> Result<Vec<InspectorSection>, InspectorAdapterError> {
    let bridge = runtime.inspector_bridge();
    let adapter = EcsInspectorAdapter::new(runtime.world(), &bridge);

    adapter.build_sections(&InspectTarget::Component {
        entity,
        component_type,
    })
}

pub fn build_resource_inspector_sections(
    runtime: &RunenwerkEditorRuntime,
    resource_type: ResourceTypeId,
) -> Result<Vec<InspectorSection>, InspectorAdapterError> {
    let bridge = runtime.inspector_bridge();
    let adapter = EcsInspectorAdapter::new(runtime.world(), &bridge);

    adapter.build_sections(&InspectTarget::Resource(resource_type))
}
