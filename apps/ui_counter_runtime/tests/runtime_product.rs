use ui_counter_runtime::{CounterRuntimeOptions, WINDOW_TITLE, build_counter_app};

#[test]
fn counter_app_prepares_visible_runtime_frame() {
    let app = build_counter_app(CounterRuntimeOptions::headless())
        .unwrap()
        .run_for_frames(1)
        .unwrap();
    let prepared = app
        .world()
        .resource::<engine::plugins::ui::UiRuntimePreparedFrameResource>()
        .unwrap()
        .latest_record()
        .unwrap();
    assert!(
        prepared
            .content_labels()
            .iter()
            .any(|label| label == WINDOW_TITLE)
    );
    assert!(
        prepared
            .content_labels()
            .iter()
            .any(|label| label == "Count: 0")
    );
    assert!(
        prepared
            .interactive_routes()
            .iter()
            .any(|route| route == "counter.increment")
    );
    let targets = app
        .world()
        .resource::<engine::plugins::ui::UiRuntimeHitTargetResource>()
        .unwrap();
    assert_eq!(targets.targets().len(), 3);
}
