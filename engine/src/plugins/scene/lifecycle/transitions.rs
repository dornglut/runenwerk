use super::super::runtime::{
    SceneTemplateAction, SceneTemplateFlowResource, flush_lifecycle_status,
    process_overlay_pointer_input, publish_scene_state, sync_overlay_viewport,
    sync_world_scene_context_from_input,
};
use crate::plugins::{InputState, SceneResource};
use crate::prelude::Time;
use crate::prelude::domain::{SceneCommand, SceneId};
use crate::runtime::{FixedTimeConfig, Res, WindowState, WorldMut};
use crate::{GameplayRuntimeConfig, SceneRuntimeState, UiOverlayState};
use anyhow::Result;

pub(crate) fn scene_transition_system(
    mut world: WorldMut,
    window: Res<WindowState>,
    time: Res<Time>,
    fixed_time: Res<FixedTimeConfig>,
) -> Result<()> {
    let mut input = world.remove_resource::<InputState>().unwrap_or_default();
    let mut scene_templates = world
        .remove_resource::<SceneTemplateFlowResource>()
        .unwrap_or_default();
    let mut scene_resource = world.remove_resource::<SceneResource>().unwrap_or_default();
    let mut scene_state = world
        .remove_resource::<SceneRuntimeState>()
        .unwrap_or_default();
    let mut gameplay = world
        .remove_resource::<GameplayRuntimeConfig>()
        .unwrap_or_default();
    let mut overlay = world
        .remove_resource::<UiOverlayState>()
        .unwrap_or_default();

    let result = (|| -> Result<()> {
        let Some(manager) = scene_resource.manager.as_mut() else {
            return Ok(());
        };

        sync_overlay_viewport(manager, &window);
        sync_world_scene_context_from_input(
            manager,
            &input,
            time.delta_seconds,
            fixed_time.step_seconds,
        );

        if scene_templates.has_scenes() {
            process_overlay_pointer_input(manager, &mut input, &mut scene_templates, &time)?;
            if input.toggle_pause_menu {
                match scene_templates.active_scene_id() {
                    Some("game_scene") => {
                        let action = SceneTemplateAction::GoTo("pause_menu".to_string());
                        scene_templates.apply_action(
                            &action,
                            manager,
                            "toggle_pause_menu",
                            Some("system"),
                        )?;
                    }
                    Some("pause_menu") => {
                        let action = SceneTemplateAction::GoTo("game_scene".to_string());
                        scene_templates.apply_action(
                            &action,
                            manager,
                            "toggle_pause_menu",
                            Some("system"),
                        )?;
                    }
                    _ => {}
                }
            }
        } else {
            if input.toggle_pause_menu {
                let show_overlay = !manager.overlay_visible();
                manager.set_active_overlay_visible(show_overlay);
                manager.queue(SceneCommand::PauseWorld(show_overlay));
                if show_overlay && manager.active_overlay() != SceneId::HudUi {
                    manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
                }
            }
            if input.scene_next {
                let next = manager.active_overlay().next_overlay();
                manager.queue(SceneCommand::ReplaceOverlay(next));
            }
            if input.scene_prev {
                let prev = manager.active_overlay().previous_overlay();
                manager.queue(SceneCommand::ReplaceOverlay(prev));
            }
            if input.scene_console {
                manager.queue(SceneCommand::ReplaceOverlay(SceneId::ConsoleUi));
            }
            if input.scene_hud {
                manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
            }
            if input.scene_overlay_push {
                let next = manager.active_overlay().next_overlay();
                manager.queue(SceneCommand::PushOverlay(next));
            }
            if input.scene_overlay_pop {
                manager.queue(SceneCommand::PopOverlay);
            }
        }

        let result = manager.apply_pending()?;
        if result.world_changed {
            manager.overlay_runtime.ui.editor.status = format!(
                "editor: world scene switched to {}",
                manager.world.active.label()
            );
        }
        if result.overlay_changed {
            let active = manager.active_overlay();
            let path = manager
                .registry
                .ui_template_path(active)
                .unwrap_or("<none>");
            manager.overlay_runtime.ui.editor.status = format!(
                "editor: overlay scene switched to {} ({}) [stack={}]",
                active.label(),
                path,
                manager.overlays.len()
            );
        }
        if result.world_pause_changed {
            manager.overlay_runtime.ui.editor.status = if manager.world.paused {
                "editor: world scene paused".to_string()
            } else {
                "editor: world scene resumed".to_string()
            };
        }

        flush_lifecycle_status(manager);
        publish_scene_state(manager, &mut scene_state, &mut gameplay, &mut overlay);
        Ok(())
    })();

    world.insert_resource(input);
    world.insert_resource(scene_templates);
    world.insert_resource(scene_resource);
    world.insert_resource(scene_state);
    world.insert_resource(gameplay);
    world.insert_resource(overlay);
    result
}
