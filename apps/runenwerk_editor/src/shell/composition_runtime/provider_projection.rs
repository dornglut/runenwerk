use editor_shell::{
    SurfaceDocumentContext, SurfaceProviderRequest, ToolSurfaceInstanceId, ToolSurfaceRegistry,
    ToolSurfaceStableKey, tool_surface_capabilities_from_registry_or_legacy,
    tool_surface_definition_id, tool_surface_kind_for_stable_key,
};

use crate::shell::RunenwerkEditorShellState;

pub(crate) fn composition_surface_provider_requests(
    shell_state: &RunenwerkEditorShellState,
    document_context: SurfaceDocumentContext,
    tool_surface_registry: Option<&ToolSurfaceRegistry>,
) -> Vec<SurfaceProviderRequest> {
    let runtime = shell_state.composition_runtime();
    let definition = runtime.composition().definition();
    let mut requests = Vec::with_capacity(definition.mounted_units().len());

    for mounted_unit in definition.mounted_units() {
        let Some(extension) = runtime.extension().mounted_unit(mounted_unit.id) else {
            continue;
        };
        let Some(tab_stack_id) = definition.regions().iter().find_map(|region| {
            let ui_composition::RegionKind::Stack { ordered_units, .. } = &region.kind else {
                return None;
            };
            if !ordered_units.contains(&mounted_unit.id) {
                return None;
            }
            runtime
                .extension()
                .region(region.id)
                .and_then(|record| record.tab_stack_raw)
                .and_then(|raw| editor_shell::TabStackId::try_from_raw(raw).ok())
        }) else {
            continue;
        };
        let Ok(panel_instance_id) =
            editor_shell::PanelInstanceId::try_from_raw(extension.panel_instance_raw)
        else {
            continue;
        };
        let Ok(tool_surface_instance_id) =
            ToolSurfaceInstanceId::try_from_raw(extension.compatibility_surface_raw)
        else {
            continue;
        };
        let Ok(stable_surface_key) =
            ToolSurfaceStableKey::new(extension.stable_content_key.clone())
        else {
            continue;
        };
        let registered_surface =
            tool_surface_registry.and_then(|registry| registry.get(&stable_surface_key));
        let stable_key_kind = tool_surface_kind_for_stable_key(&stable_surface_key);
        let surface_definition_id = stable_key_kind
            .map(tool_surface_definition_id)
            .unwrap_or(editor_shell::PLACEHOLDER_SURFACE_DEFINITION_ID);
        let capabilities = stable_key_kind
            .map(|kind| {
                tool_surface_capabilities_from_registry_or_legacy(
                    kind,
                    Some(&stable_surface_key),
                    tool_surface_registry,
                )
            })
            .or_else(|| registered_surface.map(|definition| definition.capabilities))
            .unwrap_or_default();

        requests.push(SurfaceProviderRequest {
            mounted_unit_id: mounted_unit.id,
            unavailable_content_policy: mounted_unit.unavailable_policy(),
            workspace_profile_id: shell_state.active_workspace_profile_id(),
            document_context: document_context.clone(),
            panel_instance_id,
            tab_stack_id,
            tool_surface_instance_id,
            stable_surface_key,
            provider_family_id: registered_surface
                .map(|definition| definition.provider_family.clone()),
            surface_route: registered_surface.map(|definition| definition.route),
            surface_definition_id,
            capabilities,
        });
    }

    requests.sort_by_key(|request| request.mounted_unit_id);
    requests
}
