use engine::runtime::platform::{PlatformEvent, PlatformWindowEventQueueResource};
use engine::runtime::{NativeWindowId, Res, ResMut, WindowStateRegistryResource};
use ui_math::UiRect;

use super::{EditorTargetInputRuntimeResource, translate_platform_event};
use crate::runtime::resources::{
    EditorHostResource, EditorInputBridgeState, scaled_shell_theme,
};
use crate::runtime::systems::input_bridge::{
    clear_editor_viewport_interaction_for_target, dispatch_editor_viewport_input_for_target,
};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPickingResultsResource,
    ViewportPresentationStateResource, ViewportRenderStateCommandQueueResource,
    ViewportRenderStateResource,
};

#[allow(clippy::too_many_arguments)]
pub fn dispatch_editor_target_input_system(
    mut host: ResMut<EditorHostResource>,
    mut bridge: ResMut<EditorInputBridgeState>,
    mut runtime: ResMut<EditorTargetInputRuntimeResource>,
    mut events: ResMut<PlatformWindowEventQueueResource>,
    windows: Res<WindowStateRegistryResource>,
    mut picking_results: ResMut<ViewportPickingResultsResource>,
    viewport_render_states: Res<ViewportRenderStateResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    mut viewport_render_commands: ResMut<ViewportRenderStateCommandQueueResource>,
) {
    let live_targets = host
        .shell_state
        .composition_runtime()
        .composition()
        .definition()
        .targets()
        .iter()
        .map(|target| target.id)
        .collect::<Vec<_>>();
    let retired_targets = bridge
        .interaction_targets()
        .filter(|target_id| !live_targets.contains(target_id))
        .collect::<Vec<_>>();
    for target_id in retired_targets {
        clear_editor_viewport_interaction_for_target(
            target_id,
            &mut bridge,
            &mut picking_results,
            &tool_surface_bindings,
        );
    }

    for window_event in events.drain() {
        let native_window_id = window_event.native_window_id;
        if native_window_id == NativeWindowId::primary() {
            continue;
        }
        let Some(target_id) = host
            .shell_state
            .composition_target_bindings()
            .find(|entry| entry.binding.native_window_id == native_window_id)
            .map(|entry| entry.target_id)
        else {
            continue;
        };
        let Some(window) = windows.record(native_window_id) else {
            runtime.clear_window(native_window_id);
            clear_editor_viewport_interaction_for_target(
                target_id,
                &mut bridge,
                &mut picking_results,
                &tool_surface_bindings,
            );
            continue;
        };
        if matches!(
            window_event.event,
            PlatformEvent::Focused { focused: false }
        ) {
            runtime.clear_window(native_window_id);
            host.shell_state
                .runtime_for_target_mut(target_id)
                .set_focused_widget(None);
            host.shell_state.clear_tab_drag_for_target(target_id);
            clear_editor_viewport_interaction_for_target(
                target_id,
                &mut bridge,
                &mut picking_results,
                &tool_surface_bindings,
            );
            continue;
        }
        let ui_events =
            translate_platform_event(&mut runtime, native_window_id, window_event.event);
        if ui_events.is_empty() {
            continue;
        }
        let bounds = UiRect::new(
            0.0,
            0.0,
            window.size_px.0.max(1) as f32,
            window.size_px.1.max(1) as f32,
        );
        let theme = scaled_shell_theme(&host.theme, window.scale_factor);
        for event in ui_events {
            let outcome = {
                let EditorHostResource {
                    app, shell_state, ..
                } = &mut *host;
                match app.dispatch_shell_input_for_target(
                    shell_state,
                    target_id,
                    bounds,
                    &theme,
                    &event,
                    Some(&mut *viewport_presentations),
                    Some(&viewport_observations),
                    Some(&tool_surface_bindings),
                    Some(&viewport_instances),
                    Some(&mut *viewport_render_commands),
                ) {
                    Ok(outcome) => Some(outcome),
                    Err(error) => {
                        app.append_console_line(format!(
                            "[editor_composition.input.target_dispatch_failed] {error}"
                        ));
                        None
                    }
                }
            };
            let ui_consumed = outcome.as_ref().is_some_and(|value| {
                value.dispatch.response.propagation == ui_input::EventPropagation::Stop
            });
            dispatch_editor_viewport_input_for_target(
                target_id,
                &event,
                outcome.as_ref().map(|value| &value.dispatch),
                ui_consumed,
                &mut host,
                &mut bridge,
                &mut picking_results,
                &viewport_render_states,
                &tool_surface_bindings,
                &mut viewport_render_commands,
            );
        }
    }
}
