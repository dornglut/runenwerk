use crate::plugins::SceneManager;
use crate::prelude::domain::{SceneId, SceneSlot, build_overlay_runtime};
use crate::{GameplayRuntimeConfig, SceneRuntimeState, UiOverlayState};
use anyhow::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Owner: Engine Scene Plugin - Runtime Helpers
pub(crate) fn rebuild_overlay_stack(manager: &mut SceneManager, slots: &[SceneSlot]) -> Result<()> {
    let slots = if slots.is_empty() {
        vec![SceneSlot {
            active: SceneId::ConsoleUi,
            paused: false,
            visible: false,
        }]
    } else {
        slots.to_vec()
    };
    let screen_size = manager.overlay_runtime.ui.screen_size;
    let scale = manager.overlay_runtime.ui.scale;
    let last_index = slots.len().saturating_sub(1);
    let mut stack = Vec::new();
    for slot in slots.iter().take(last_index) {
        stack.push((
            *slot,
            build_overlay_runtime(slot.active, screen_size, scale, &manager.registry)?,
        ));
    }
    let active_slot = slots[last_index];
    manager.overlay_runtime =
        build_overlay_runtime(active_slot.active, screen_size, scale, &manager.registry)?;
    manager.overlay_back_stack = stack;
    manager.overlays = slots;
    Ok(())
}

pub(crate) fn snapshot_public_scene_state(
    manager: &SceneManager,
) -> (
    SceneRuntimeState,
    GameplayRuntimeConfig,
    UiOverlayState,
) {
    let gameplay = GameplayRuntimeConfig {
        chunk_size: manager.world_runtime.ctx.gameplay_config.chunk_size,
        chunk_load_radius: manager.world_runtime.ctx.gameplay_config.chunk_load_radius,
        infinite_world: manager.world_runtime.ctx.gameplay_config.infinite_world,
    };
    let scene_state = SceneRuntimeState {
        world_scene_label: manager.world.active.label().to_string(),
        overlay_scene_label: manager.active_overlay().label().to_string(),
        overlay_visible: manager.overlay_visible(),
        world_paused: manager.world.paused,
        enemy_kills: manager.world_runtime.ctx.enemy_kills,
        gameplay,
    };
    let overlay = UiOverlayState {
        screen_size: manager.overlay_runtime.ui.screen_size,
        scale: manager.overlay_runtime.ui.scale,
        ..UiOverlayState::default()
    };
    (scene_state, gameplay, overlay)
}

pub(crate) fn system_time_to_millis(value: Option<SystemTime>) -> Option<u64> {
    value.and_then(|time| {
        time.duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| u64::try_from(duration.as_millis()).ok())
    })
}

pub(crate) fn millis_to_system_time(value: Option<u64>) -> Option<SystemTime> {
    value.map(|millis| UNIX_EPOCH + Duration::from_millis(millis))
}
