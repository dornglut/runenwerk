use super::super::{
    SceneEntityDeltaV2, SceneEntitySnapshotV2, SceneSimulationDeltaV2, SceneSimulationSnapshotV2,
    SceneWorldContextDeltaV2, SceneWorldContextSnapshotV2,
};

pub(crate) fn build_scene_simulation_delta(
    base: &SceneSimulationSnapshotV2,
    current: &SceneSimulationSnapshotV2,
) -> SceneSimulationDeltaV2 {
    let base_ctx = &base.context;
    let current_ctx = &current.context;
    let base_entities = &base.entities;
    let current_entities = &current.entities;

    SceneSimulationDeltaV2 {
        context: SceneWorldContextDeltaV2 {
            world: (base_ctx.world != current_ctx.world).then_some(current_ctx.world),
            overlays: (base_ctx.overlays != current_ctx.overlays)
                .then_some(current_ctx.overlays.clone()),
            world_scene_label: (base_ctx.world_scene_label != current_ctx.world_scene_label)
                .then_some(current_ctx.world_scene_label.clone()),
            overlay_scene_label: (base_ctx.overlay_scene_label != current_ctx.overlay_scene_label)
                .then_some(current_ctx.overlay_scene_label.clone()),
            gameplay_config: (base_ctx.gameplay_config != current_ctx.gameplay_config)
                .then_some(current_ctx.gameplay_config.clone()),
            gameplay_config_modified_millis: (base_ctx.gameplay_config_modified_millis
                != current_ctx.gameplay_config_modified_millis)
                .then_some(current_ctx.gameplay_config_modified_millis),
            gameplay_config_revision: (base_ctx.gameplay_config_revision
                != current_ctx.gameplay_config_revision)
                .then_some(current_ctx.gameplay_config_revision),
            overlay_consumed: (base_ctx.overlay_consumed != current_ctx.overlay_consumed)
                .then_some(current_ctx.overlay_consumed),
            player_move_x: (base_ctx.player_move_x != current_ctx.player_move_x)
                .then_some(current_ctx.player_move_x),
            player_move_y: (base_ctx.player_move_y != current_ctx.player_move_y)
                .then_some(current_ctx.player_move_y),
            camera_yaw: (base_ctx.camera_yaw != current_ctx.camera_yaw)
                .then_some(current_ctx.camera_yaw),
            camera_pitch: (base_ctx.camera_pitch != current_ctx.camera_pitch)
                .then_some(current_ctx.camera_pitch),
            camera_distance: (base_ctx.camera_distance != current_ctx.camera_distance)
                .then_some(current_ctx.camera_distance),
            delta_seconds: (base_ctx.delta_seconds != current_ctx.delta_seconds)
                .then_some(current_ctx.delta_seconds),
            fixed_step_seconds: (base_ctx.fixed_step_seconds != current_ctx.fixed_step_seconds)
                .then_some(current_ctx.fixed_step_seconds),
            fixed_step_accumulator: (base_ctx.fixed_step_accumulator
                != current_ctx.fixed_step_accumulator)
                .then_some(current_ctx.fixed_step_accumulator),
            frame_count: (base_ctx.frame_count != current_ctx.frame_count)
                .then_some(current_ctx.frame_count),
            enemy_kills: (base_ctx.enemy_kills != current_ctx.enemy_kills)
                .then_some(current_ctx.enemy_kills),
        },
        entities: SceneEntityDeltaV2 {
            frame_counter: (base_entities.frame_counter != current_entities.frame_counter)
                .then_some(current_entities.frame_counter),
            debug_position: (base_entities.debug_position != current_entities.debug_position)
                .then_some(current_entities.debug_position),
            debug_velocity: (base_entities.debug_velocity != current_entities.debug_velocity)
                .then_some(current_entities.debug_velocity),
        },
    }
}

pub(crate) fn apply_scene_simulation_delta(
    base: &SceneSimulationSnapshotV2,
    delta: &SceneSimulationDeltaV2,
) -> SceneSimulationSnapshotV2 {
    SceneSimulationSnapshotV2 {
        context: SceneWorldContextSnapshotV2 {
            world: delta.context.world.unwrap_or(base.context.world),
            overlays: delta
                .context
                .overlays
                .clone()
                .unwrap_or_else(|| base.context.overlays.clone()),
            world_scene_label: delta
                .context
                .world_scene_label
                .clone()
                .unwrap_or_else(|| base.context.world_scene_label.clone()),
            overlay_scene_label: delta
                .context
                .overlay_scene_label
                .clone()
                .unwrap_or_else(|| base.context.overlay_scene_label.clone()),
            gameplay_config: delta
                .context
                .gameplay_config
                .clone()
                .unwrap_or_else(|| base.context.gameplay_config.clone()),
            gameplay_config_modified_millis: delta
                .context
                .gameplay_config_modified_millis
                .unwrap_or(base.context.gameplay_config_modified_millis),
            gameplay_config_revision: delta
                .context
                .gameplay_config_revision
                .unwrap_or(base.context.gameplay_config_revision),
            overlay_consumed: delta
                .context
                .overlay_consumed
                .unwrap_or(base.context.overlay_consumed),
            player_move_x: delta
                .context
                .player_move_x
                .unwrap_or(base.context.player_move_x),
            player_move_y: delta
                .context
                .player_move_y
                .unwrap_or(base.context.player_move_y),
            camera_yaw: delta.context.camera_yaw.unwrap_or(base.context.camera_yaw),
            camera_pitch: delta
                .context
                .camera_pitch
                .unwrap_or(base.context.camera_pitch),
            camera_distance: delta
                .context
                .camera_distance
                .unwrap_or(base.context.camera_distance),
            delta_seconds: delta
                .context
                .delta_seconds
                .unwrap_or(base.context.delta_seconds),
            fixed_step_seconds: delta
                .context
                .fixed_step_seconds
                .unwrap_or(base.context.fixed_step_seconds),
            fixed_step_accumulator: delta
                .context
                .fixed_step_accumulator
                .unwrap_or(base.context.fixed_step_accumulator),
            frame_count: delta
                .context
                .frame_count
                .unwrap_or(base.context.frame_count),
            enemy_kills: delta
                .context
                .enemy_kills
                .unwrap_or(base.context.enemy_kills),
        },
        entities: SceneEntitySnapshotV2 {
            frame_counter: delta
                .entities
                .frame_counter
                .unwrap_or(base.entities.frame_counter),
            debug_position: delta
                .entities
                .debug_position
                .unwrap_or(base.entities.debug_position),
            debug_velocity: delta
                .entities
                .debug_velocity
                .unwrap_or(base.entities.debug_velocity),
        },
    }
}
