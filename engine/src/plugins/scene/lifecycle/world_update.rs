use super::super::runtime::publish_scene_state;
use crate::plugins::scene::{SceneOverlayViewportState, SceneResource, SceneRuntimeState};
use crate::runtime::{FixedTimeConfig, WorldMut};
use anyhow::Result;

pub(crate) fn world_scene_update_system(mut world: WorldMut) -> Result<()> {
    let step_seconds = world.resource::<FixedTimeConfig>()?.step_seconds;
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
        if !manager.world.visible || manager.world.paused {
            publish_scene_state(manager, &mut scene_state, &mut viewport);
            return Ok(());
        }

        manager.world_runtime.ctx.delta_seconds = step_seconds.clamp(1.0 / 240.0, 1.0 / 30.0);
        manager.world_runtime.ctx.fixed_step_seconds = manager.world_runtime.ctx.delta_seconds;
        manager.world_runtime.run()?;
        let outbound = std::mem::take(&mut manager.world_runtime.ctx.outbound_notifications);
        manager.channels.world_to_overlay.extend(outbound);
        publish_scene_state(manager, &mut scene_state, &mut viewport);
        Ok(())
    })();

    world.insert_resource(scene_resource);
    world.insert_resource(scene_state);
    world.insert_resource(viewport);
    result
}
