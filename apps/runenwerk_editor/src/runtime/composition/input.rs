use engine::runtime::platform::{PlatformEvent, PlatformWindowEventQueueResource};
use engine::runtime::{NativeWindowId, Res, ResMut, WindowStateRegistryResource};
use ui_math::UiRect;

use super::{EditorTargetInputRuntimeResource, translate_platform_event};
use crate::runtime::resources::{EditorHostResource, scaled_shell_theme};
use crate::runtime::viewport::{
    ToolSurfaceRuntimeBindingRegistryResource, ViewportArtifactObservationResource,
    ViewportInstanceRegistryResource, ViewportPresentationStateResource,
    ViewportRenderStateCommandQueueResource,
};

#[allow(clippy::too_many_arguments)]
pub fn dispatch_editor_target_input_system(
    mut host: ResMut<EditorHostResource>,
    mut runtime: ResMut<EditorTargetInputRuntimeResource>,
    mut events: ResMut<PlatformWindowEventQueueResource>,
    windows: Res<WindowStateRegistryResource>,
    mut viewport_presentations: ResMut<ViewportPresentationStateResource>,
    viewport_observations: Res<ViewportArtifactObservationResource>,
    tool_surface_bindings: Res<ToolSurfaceRuntimeBindingRegistryResource>,
    viewport_instances: Res<ViewportInstanceRegistryResource>,
    mut viewport_render_commands: ResMut<ViewportRenderStateCommandQueueResource>,
) {
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
            let EditorHostResource {
                app, shell_state, ..
            } = &mut *host;
            if let Err(error) = app.dispatch_shell_input_for_target(
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
                app.append_console_line(format!(
                    "[editor_composition.input.target_dispatch_failed] {error}"
                ));
            }
        }
    }
}
