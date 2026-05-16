//! File: apps/runenwerk_editor/src/runtime/systems/viewport_lifecycle.rs
//! Purpose: Runtime viewport instance lifecycle synchronization.

use std::collections::BTreeSet;

use editor_shell::{ToolSurfaceInstanceId, ToolSurfaceKind, WorkspaceMutation};
use editor_viewport::{ViewportId, ViewportRuntimeSettings};
use engine::runtime::ResMut;

use crate::runtime::resources::EditorHostResource;
use crate::runtime::viewport::{
    MAIN_VIEWPORT_ID, ViewportInstanceRegistryResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportRenderStateResource,
    ViewportRuntimeSettingsHydrationKey, ViewportRuntimeSettingsHydrationResource,
    initial_presentation_state,
};

#[allow(clippy::too_many_arguments)]
pub fn sync_viewport_instances_system(
    mut host: ResMut<EditorHostResource>,
    mut viewport_instances: ResMut<ViewportInstanceRegistryResource>,
    mut viewport_render_states: ResMut<ViewportRenderStateResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    mut viewport_settings_hydration: ResMut<ViewportRuntimeSettingsHydrationResource>,
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
    viewport_settings_hydration
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
            if let Some(settings) = surface.viewport_settings {
                hydrate_presentation_from_viewport_settings(
                    record.tool_surface_id,
                    record.viewport_id,
                    settings,
                    &mut viewport_presentations,
                    &mut viewport_settings_hydration,
                );
            }
            let selected_primary_product_id = viewport_presentations
                .state_for(record.viewport_id)
                .map(|state| state.selected_primary_product_id)
                .or_else(|| {
                    surface
                        .viewport_settings
                        .and_then(|settings| settings.selected_primary_product_id)
                });
            let field_visualizer_settings = viewport_presentations
                .state_for(record.viewport_id)
                .map(|state| state.field_visualizer_settings)
                .or_else(|| {
                    surface
                        .viewport_settings
                        .map(|settings| settings.field_visualizer_settings)
                })
                .unwrap_or_default();
            let next = viewport_render_states
                .state_for(record.viewport_id)
                .map(|entry| {
                    entry
                        .render_state
                        .viewport_settings(selected_primary_product_id, field_visualizer_settings)
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

fn hydrate_presentation_from_viewport_settings(
    tool_surface_id: ToolSurfaceInstanceId,
    viewport_id: ViewportId,
    settings: ViewportRuntimeSettings,
    viewport_presentations: &mut ViewportPresentationStateResource,
    viewport_settings_hydration: &mut ViewportRuntimeSettingsHydrationResource,
) {
    let hydration_key = ViewportRuntimeSettingsHydrationKey::new(
        tool_surface_id,
        viewport_id,
        settings.selected_primary_product_id,
        settings.field_visualizer_settings,
    );
    if !viewport_settings_hydration.should_hydrate(hydration_key) {
        return;
    }

    let mut presentation = viewport_presentations
        .state_for(viewport_id)
        .cloned()
        .unwrap_or_else(|| initial_presentation_state(viewport_id));
    if let Some(product_id) = settings.selected_primary_product_id {
        presentation.select_primary_product(product_id);
    }
    presentation.set_field_visualizer_settings(settings.field_visualizer_settings);
    viewport_presentations.upsert_state(presentation);
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_viewport::{
        ExpressionProductId, ViewportFieldVisualizerColorRamp, ViewportFieldVisualizerComponent,
        ViewportFieldVisualizerDebugMode, ViewportFieldVisualizerSettings,
    };

    fn tool_surface_id() -> ToolSurfaceInstanceId {
        ToolSurfaceInstanceId::try_from_raw(1).unwrap()
    }

    fn non_default_field_settings() -> ViewportFieldVisualizerSettings {
        ViewportFieldVisualizerSettings::default()
            .with_component(ViewportFieldVisualizerComponent::Magnitude)
            .with_slice_index(3)
            .with_color_ramp(ViewportFieldVisualizerColorRamp::Heat)
            .with_debug_mode(ViewportFieldVisualizerDebugMode::Freshness)
    }

    #[test]
    fn lifecycle_hydration_restores_persisted_field_settings_over_existing_default_presentation() {
        let viewport_id = ViewportId(2);
        let selected_product_id = ExpressionProductId(6);
        let persisted_settings = ViewportRuntimeSettings {
            selected_primary_product_id: Some(selected_product_id),
            field_visualizer_settings: non_default_field_settings(),
            ..Default::default()
        };
        let mut presentations = ViewportPresentationStateResource::default();
        let mut hydration = ViewportRuntimeSettingsHydrationResource::default();
        presentations.upsert_state(initial_presentation_state(viewport_id));

        hydrate_presentation_from_viewport_settings(
            tool_surface_id(),
            viewport_id,
            persisted_settings,
            &mut presentations,
            &mut hydration,
        );

        let presentation = presentations
            .state_for(viewport_id)
            .expect("viewport presentation should be hydrated");
        assert_eq!(
            presentation.selected_primary_product_id,
            selected_product_id
        );
        assert_eq!(
            presentation.field_visualizer_settings,
            persisted_settings.field_visualizer_settings
        );
    }

    #[test]
    fn lifecycle_hydration_does_not_reapply_old_workspace_settings_after_user_change() {
        let viewport_id = ViewportId(2);
        let old_persisted_settings = ViewportRuntimeSettings {
            selected_primary_product_id: Some(ExpressionProductId(6)),
            field_visualizer_settings: non_default_field_settings(),
            ..Default::default()
        };
        let user_settings = ViewportFieldVisualizerSettings::default()
            .with_component(ViewportFieldVisualizerComponent::X)
            .with_slice_index(7)
            .with_color_ramp(ViewportFieldVisualizerColorRamp::DivergingSigned)
            .with_debug_mode(ViewportFieldVisualizerDebugMode::Availability);
        let mut presentations = ViewportPresentationStateResource::default();
        let mut hydration = ViewportRuntimeSettingsHydrationResource::default();

        hydrate_presentation_from_viewport_settings(
            tool_surface_id(),
            viewport_id,
            old_persisted_settings,
            &mut presentations,
            &mut hydration,
        );
        let mut presentation = presentations
            .state_for(viewport_id)
            .cloned()
            .expect("viewport presentation should be hydrated");
        presentation.set_field_visualizer_settings(user_settings);
        presentations.upsert_state(presentation);

        hydrate_presentation_from_viewport_settings(
            tool_surface_id(),
            viewport_id,
            old_persisted_settings,
            &mut presentations,
            &mut hydration,
        );

        assert_eq!(
            presentations
                .state_for(viewport_id)
                .map(|state| state.field_visualizer_settings),
            Some(user_settings)
        );
    }
}
