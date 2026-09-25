use super::super::runtime::{
    SceneTemplateAction, SceneTemplateFlowResource, flush_lifecycle_status,
    process_overlay_pointer_input, publish_scene_state, sync_overlay_viewport,
    sync_world_scene_context_from_input,
};
use crate::plugins::input::domain::action;
use crate::plugins::scene::{SceneOverlayViewportState, SceneResource, SceneRuntimeState};
use crate::plugins::{ActionState, InputState};
use crate::prelude::Time;
use crate::prelude::domain::{SceneCommand, SceneId};
use crate::runtime::{FixedTimeConfig, PrimaryPresentationMetricsResource, WorldMut};
use anyhow::Result;

pub(crate) fn scene_transition_system(mut world: WorldMut) -> Result<()> {
    let presentation = *world.resource::<PrimaryPresentationMetricsResource>()?;
    let delta_seconds = world.resource::<Time>()?.delta_seconds;
    let fixed_step_seconds = world
        .resource::<FixedTimeConfig>()
        .ok()
        .map(|config| config.step_seconds);

    let input_resource = world.remove_resource::<InputState>();
    let action_resource = world.remove_resource::<ActionState>();
    let input_was_installed = input_resource.is_some();
    let actions_were_installed = action_resource.is_some();
    let mut input = input_resource.unwrap_or_default();
    let actions = action_resource.unwrap_or_default();
    let mut scene_templates = world
        .remove_resource::<SceneTemplateFlowResource>()
        .unwrap_or_default();
    let mut scene_resource = world.remove_resource::<SceneResource>().unwrap_or_default();
    let mut scene_state = world
        .remove_resource::<SceneRuntimeState>()
        .unwrap_or_default();
    let mut viewport = world
        .remove_resource::<SceneOverlayViewportState>()
        .unwrap_or_default();

    let result = (|| -> Result<()> {
        let Some(manager) = scene_resource.manager.as_mut() else {
            return Ok(());
        };

        sync_overlay_viewport(manager, &presentation);
        let fixed_step_seconds =
            fixed_step_seconds.unwrap_or(manager.world_runtime.ctx.fixed_step_seconds);
        sync_world_scene_context_from_input(
            manager,
            &input,
            &actions,
            delta_seconds,
            fixed_step_seconds,
        );

        if scene_templates.has_scenes() {
            process_overlay_pointer_input(
                manager,
                &mut input,
                &mut scene_templates,
                delta_seconds,
            )?;
            if actions.action_pressed(action::SYSTEM_TOGGLE_PAUSE_MENU) {
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
            if actions.action_pressed(action::SYSTEM_TOGGLE_PAUSE_MENU) {
                let show_overlay = !manager.overlay_visible();
                manager.set_active_overlay_visible(show_overlay);
                manager.queue(SceneCommand::PauseWorld(show_overlay));
                if show_overlay && manager.active_overlay() != SceneId::HudUi {
                    manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
                }
            }
            if actions.action_pressed(action::SCENE_NEXT) {
                let next = manager.active_overlay().next_overlay();
                manager.queue(SceneCommand::ReplaceOverlay(next));
            }
            if actions.action_pressed(action::SCENE_PREV) {
                let prev = manager.active_overlay().previous_overlay();
                manager.queue(SceneCommand::ReplaceOverlay(prev));
            }
            if actions.action_pressed(action::SCENE_CONSOLE) {
                manager.queue(SceneCommand::ReplaceOverlay(SceneId::ConsoleUi));
            }
            if actions.action_pressed(action::SCENE_HUD) {
                manager.queue(SceneCommand::ReplaceOverlay(SceneId::HudUi));
            }
            if actions.action_pressed(action::SCENE_OVERLAY_PUSH) {
                let next = manager.active_overlay().next_overlay();
                manager.queue(SceneCommand::PushOverlay(next));
            }
            if actions.action_pressed(action::SCENE_OVERLAY_POP) {
                manager.queue(SceneCommand::PopOverlay);
            }
        }

        let result = manager.apply_pending()?;
        if result.world_changed || result.overlay_changed || result.world_pause_changed {
            manager.overlay_runtime.ui.layout_dirty = true;
        }

        flush_lifecycle_status(manager);
        publish_scene_state(manager, &mut scene_state, &mut viewport);
        Ok(())
    })();

    if input_was_installed {
        world.insert_resource(input);
    }
    if actions_were_installed {
        world.insert_resource(actions);
    }
    world.insert_resource(scene_templates);
    world.insert_resource(scene_resource);
    world.insert_resource(scene_state);
    world.insert_resource(viewport);
    result
}
