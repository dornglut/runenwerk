use crate::plugins::InputState;
use crate::plugins::ui::domain::UiDrawCmd;
// Owner: Engine Scene Plugin - Tests
use super::super::domain::{QuestState, WorldToOverlayMessage};
use super::super::{
    ScenePlugin, SceneResource, apply_scene_simulation_delta, build_scene_simulation_delta,
    capture_scene_simulation_snapshot, format_world_message, switch_scene_by_id,
};
use crate::prelude::*;

#[test]
fn format_world_message_renders_all_variants() {
    let tick = format_world_message(WorldToOverlayMessage::Tick {
        tick: 60,
        overlay: "console_ui".to_string(),
    });
    assert!(tick.contains("tick=60"));

    let combat = format_world_message(WorldToOverlayMessage::Combat {
        source: "Scout".to_string(),
        target: "Bat".to_string(),
        damage: 9,
        critical: true,
    });
    assert!(combat.contains("crits"));

    let loot = format_world_message(WorldToOverlayMessage::Loot {
        item: "Glowshard".to_string(),
        amount: 2,
        rarity: "rare".to_string(),
    });
    assert!(loot.contains("[loot]"));

    let quest = format_world_message(WorldToOverlayMessage::Quest {
        quest: "Map".to_string(),
        state: QuestState::Progress {
            current: 2,
            goal: 3,
        },
    });
    assert!(quest.contains("2/3"));
}

#[test]
fn scene_plugin_toggles_pause_overlay_and_updates_public_state() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);
    app.world_mut()
        .resource_mut::<InputState>()
        .expect("input state should exist")
        .toggle_pause_menu = true;

    let app = app.run_for_frames(1).expect("scene plugin should run");
    let scene = app
        .world()
        .resource::<SceneRuntimeState>()
        .expect("scene state should exist");
    assert_eq!(scene.overlay_scene_label, "hud_ui");
    assert!(scene.overlay_visible);
    assert!(scene.world_paused);
}

#[test]
fn scene_helper_switches_world_scene_by_label() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);
    switch_scene_by_id(app.world_mut(), "hub").expect("scene switch should queue");

    let app = app.run_for_frames(1).expect("scene plugin should run");
    let scene = app
        .world()
        .resource::<SceneRuntimeState>()
        .expect("scene state should exist");
    assert_eq!(scene.world_scene_label, "hub_stub");
    assert!(!scene.world_paused);
}

#[test]
fn scene_plugin_routes_world_tick_messages_into_overlay_log() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);

    let app = app.run_for_ticks(60).expect("scene plugin should run");
    let scene = app
        .world()
        .resource::<SceneResource>()
        .expect("scene resource should exist");
    let manager = scene
        .manager
        .as_ref()
        .expect("scene manager should be initialized");
    assert!(
        manager
            .overlay_runtime
            .ui
            .log_lines
            .iter()
            .any(|line| line.contains("tick=60"))
    );
}

#[test]
fn scene_simulation_delta_round_trips_back_to_the_current_snapshot() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);
    let mut app = app
        .run_for_frames(1)
        .expect("scene plugin should initialize");

    let base_snapshot = {
        let scene = app
            .world()
            .resource::<SceneResource>()
            .expect("scene resource should exist");
        let manager = scene
            .manager
            .as_ref()
            .expect("scene manager should be initialized");
        capture_scene_simulation_snapshot(manager).expect("base snapshot should capture")
    };

    {
        let mut scene = app
            .world_mut()
            .resource_mut::<SceneResource>()
            .expect("scene resource should exist");
        let manager = scene
            .manager
            .as_mut()
            .expect("scene manager should be initialized");
        manager.world_runtime.ctx.player_move_x = -0.25;
        manager.world_runtime.ctx.player_move_y = 0.75;
        manager.world_runtime.ctx.frame_count = 7;
        if let Ok(mut entity) = manager
            .world_runtime
            .ctx
            .world
            .entity_mut(manager.world_runtime.ctx.debug_entity)
            && let Some(mut position) = entity.get_mut::<super::super::domain::WorldDebugPosition>()
        {
            position.x = 4.0;
            position.y = -2.0;
        }
    }

    let current_snapshot = {
        let scene = app
            .world()
            .resource::<SceneResource>()
            .expect("scene resource should exist");
        let manager = scene
            .manager
            .as_ref()
            .expect("scene manager should be initialized");
        capture_scene_simulation_snapshot(manager).expect("current snapshot should capture")
    };

    let delta = build_scene_simulation_delta(&base_snapshot, &current_snapshot);
    let rebuilt_snapshot = apply_scene_simulation_delta(&base_snapshot, &delta);

    assert_eq!(rebuilt_snapshot, current_snapshot);
    assert_eq!(delta.context.player_move_x, Some(-0.25));
    assert_eq!(delta.context.player_move_y, Some(0.75));
    assert_eq!(delta.context.frame_count, Some(7));
    assert_eq!(
        delta.entities.debug_position,
        Some(current_snapshot.entities.debug_position)
    );
    assert_eq!(delta.context.world_scene_label, None);
}

#[test]
fn scene_registered_apps_publish_overlay_draw_list_with_buttons() {
    let mut app = App::headless();
    app.add_scene("engine/examples/scene_manager_ui/assets/scenes/main_menu.ron");
    app.add_plugin(ScenePlugin);

    let app = app.run_for_frames(1).expect("scene plugin should run");
    let scene = app
        .world()
        .resource::<SceneResource>()
        .expect("scene resource should exist");
    let manager = scene
        .manager
        .as_ref()
        .expect("scene manager should be initialized");

    assert!(
        manager.overlay_visible(),
        "overlay should auto-show for scene catalog apps"
    );
    let commands = &manager.overlay_runtime.ui.draw_list.commands;
    assert!(
        !commands.is_empty(),
        "overlay draw list should not be empty"
    );
    assert!(
        commands
            .iter()
            .any(|cmd| matches!(cmd, UiDrawCmd::Rect { .. })),
        "overlay draw list should include rect commands"
    );
    assert!(
        commands
            .iter()
            .any(|cmd| matches!(cmd, UiDrawCmd::Text { .. })),
        "overlay draw list should include text commands"
    );
    assert!(
        commands.iter().any(|cmd| {
            matches!(
                cmd,
                UiDrawCmd::Text { content, .. } if content.contains("Confirm")
            )
        }),
        "overlay draw list should include the confirm button label"
    );
}
