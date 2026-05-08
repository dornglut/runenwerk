//! File: apps/runenwerk_editor/src/runtime/systems/viewport_lifecycle.rs
//! Purpose: Runtime viewport instance lifecycle synchronization.

use std::collections::BTreeSet;

use editor_shell::{ToolSurfaceKind, WorkspaceMutation};
use engine::runtime::ResMut;

use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::{
    MAIN_VIEWPORT_ID, ViewportInstanceRegistryResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportRenderStateResource, initial_presentation_state,
};

#[allow(clippy::too_many_arguments)]
pub fn sync_viewport_instances_system(
    mut host: ResMut<EditorHostResource>,
    mut viewport_instances: ResMut<ViewportInstanceRegistryResource>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    mut viewport_picking_results: ResMut<ViewportPickingResultsResource>,
) {
    viewport_instances.sync_from_workspace_state(host.shell_state.workspace_state());
    let active_viewport_ids = viewport_instances
        .records()
        .map(|record| record.viewport_id)
        .chain(std::iter::once(MAIN_VIEWPORT_ID))
        .collect::<BTreeSet<_>>();
    viewport_render_states
        .retain_viewports(|viewport_id| active_viewport_ids.contains(&viewport_id));
    viewport_presentations
        .retain_viewports(|viewport_id| active_viewport_ids.contains(&viewport_id));
    viewport_picking_results
        .retain_viewports(|viewport_id| active_viewport_ids.contains(&viewport_id));

    let assignments = viewport_instances
        .records()
        .filter_map(|record| {
            let surface = host
                .shell_state
                .workspace_state()
                .tool_surface(record.tool_surface_id)?;
            if surface.tool_surface_kind == ToolSurfaceKind::Viewport
                && surface.viewport_instance_id != Some(record.viewport_id)
            {
                Some((record.tool_surface_id, record.viewport_id))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let settings_updates = viewport_instances
        .records()
        .filter_map(|record| {
            let surface = host
                .shell_state
                .workspace_state()
                .tool_surface(record.tool_surface_id)?;
            if surface.tool_surface_kind != ToolSurfaceKind::Viewport {
                return None;
            }
            if let Some(settings) = surface.viewport_settings
                && settings.selected_primary_product_id.is_some()
                && viewport_presentations
                    .state_for(record.viewport_id)
                    .is_none()
            {
                let mut presentation = initial_presentation_state(record.viewport_id);
                if let Some(product_id) = settings.selected_primary_product_id {
                    presentation.select_primary_product(product_id);
                }
                viewport_presentations.upsert_state(presentation);
            }
            let selected_primary_product_id = viewport_presentations
                .state_for(record.viewport_id)
                .map(|state| state.selected_primary_product_id)
                .or_else(|| {
                    surface
                        .viewport_settings
                        .and_then(|settings| settings.selected_primary_product_id)
                });
            let next = viewport_render_states
                .state_for(record.viewport_id)
                .map(|entry| {
                    entry
                        .render_state
                        .viewport_settings(selected_primary_product_id)
                })
                .or(surface.viewport_settings)?;
            (surface.viewport_settings != Some(next)).then_some((record.tool_surface_id, next))
        })
        .collect::<Vec<_>>();

    for (tool_surface_id, viewport_id) in assignments {
        if let Err(error) = host.shell_state.apply_workspace_mutation(
            WorkspaceMutation::SetToolSurfaceViewportInstanceId {
                tool_surface_id,
                viewport_instance_id: Some(viewport_id),
            },
        ) && host.app.debug_logs_enabled()
        {
            host.app.append_console_line(format!(
                "[viewport.lifecycle] failed to persist viewport instance for tool_surface={}: {}",
                tool_surface_id.raw(),
                error
            ));
        }
    }

    for (tool_surface_id, viewport_settings) in settings_updates {
        if let Err(error) = host.shell_state.apply_workspace_mutation(
            WorkspaceMutation::SetToolSurfaceViewportSettings {
                tool_surface_id,
                viewport_settings: Some(viewport_settings),
            },
        ) && host.app.debug_logs_enabled()
        {
            host.app.append_console_line(format!(
                "[viewport.lifecycle] failed to persist viewport settings for tool_surface={}: {}",
                tool_surface_id.raw(),
                error
            ));
        }
    }
}
