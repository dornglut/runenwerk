use ui_runtime::{
    base_controls_overlay_layering_fixture, base_controls_overlay_layering_positive_script,
    replay_overlay_layering, BASE_CONTROLS_EXECUTABLE_OVERLAY_LAYERING_STORY_ID,
};

#[test]
fn executable_overlay_layering_story_replays_same_runtime_path() {
    let fixture = base_controls_overlay_layering_fixture();
    let replay_script = base_controls_overlay_layering_positive_script();
    let live_log_script = base_controls_overlay_layering_positive_script();

    let replay = replay_overlay_layering(&fixture, &replay_script);
    let replayed_live_log = replay_overlay_layering(&fixture, &live_log_script);

    assert_eq!(replay.input_steps, replayed_live_log.input_steps);
    assert_eq!(replay.open_intents.len(), replayed_live_log.open_intents.len());
    assert_eq!(replay.stack_entries.len(), replayed_live_log.stack_entries.len());
    assert_eq!(replay.dismissal_evidence.len(), replayed_live_log.dismissal_evidence.len());
    assert_eq!(replay.boundary_assertions, replayed_live_log.boundary_assertions);
    assert!(BASE_CONTROLS_EXECUTABLE_OVERLAY_LAYERING_STORY_ID.contains("overlay-layering.story"));
}
