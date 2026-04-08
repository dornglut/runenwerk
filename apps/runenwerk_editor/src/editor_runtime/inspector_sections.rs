use editor_core::{ComponentTypeId, EntityId, ResourceTypeId};
use editor_inspector::{
    EcsInspectorAdapter, InspectTarget, InspectorAdapter, InspectorAdapterError, InspectorSection,
};

use crate::editor_runtime::RunenwerkEditorRuntime;

/// File: apps/runenwerk_editor/src/editor_runtime/inspector_sections.rs
/// Method: build_component_inspector_sections
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

/// File: apps/runenwerk_editor/src/editor_runtime/inspector_sections.rs
/// Method: build_resource_inspector_sections
pub fn build_resource_inspector_sections(
    runtime: &RunenwerkEditorRuntime,
    resource_type: ResourceTypeId,
) -> Result<Vec<InspectorSection>, InspectorAdapterError> {
    let bridge = runtime.inspector_bridge();
    let adapter = EcsInspectorAdapter::new(runtime.world(), &bridge);

    adapter.build_sections(&InspectTarget::Resource(resource_type))
}
