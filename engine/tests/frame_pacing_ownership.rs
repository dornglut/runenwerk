use engine::prelude::App;
use engine::runtime::{
    FramePacingMode, FramePacingPolicyResource, FramePacingRuntimeStateResource,
};

#[test]
fn bare_apps_do_not_provision_frame_pacing_resources() {
    let headless = App::headless();
    assert!(
        headless
            .world()
            .resource::<FramePacingPolicyResource>()
            .is_err(),
        "bare headless App should not provision native pacing policy"
    );
    assert!(
        headless
            .world()
            .resource::<FramePacingRuntimeStateResource>()
            .is_err(),
        "bare headless App should not provision native pacing observation state"
    );

    let windowed = App::new();
    assert!(
        windowed
            .world()
            .resource::<FramePacingPolicyResource>()
            .is_err(),
        "bare windowed App should not provision native pacing policy before Host realization"
    );
    assert!(
        windowed
            .world()
            .resource::<FramePacingRuntimeStateResource>()
            .is_err(),
        "bare windowed App should not provision native pacing observation state before Host realization"
    );
}

#[test]
fn explicit_frame_pacing_configures_only_policy_before_host_realization() {
    let mut app = App::new();
    app.with_frame_pacing(FramePacingPolicyResource::on_demand());

    assert_eq!(
        app.world()
            .resource::<FramePacingPolicyResource>()
            .expect("explicit frame pacing should install policy")
            .mode,
        FramePacingMode::OnDemand
    );
    assert!(
        app.world()
            .resource::<FramePacingRuntimeStateResource>()
            .is_err(),
        "explicit pre-run policy selection should not realize native Host observation state"
    );
}

#[test]
fn bounded_headless_execution_does_not_require_frame_pacing_resources() {
    let app = App::headless();
    let app = app
        .run_for_frames(1)
        .expect("bounded headless execution should not require native pacing state");

    assert!(
        app.world().resource::<FramePacingPolicyResource>().is_err(),
        "headless execution should not synthesize native pacing policy"
    );
    assert!(
        app.world()
            .resource::<FramePacingRuntimeStateResource>()
            .is_err(),
        "headless execution should not synthesize native pacing observation state"
    );
}
