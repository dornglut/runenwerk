use engine::prelude::App;
use engine::runtime::NativeWindowHookRegistryResource;

#[test]
fn bare_apps_do_not_provision_native_window_hook_registry() {
    let headless = App::headless();
    assert!(
        headless
            .world()
            .resource::<NativeWindowHookRegistryResource>()
            .is_err(),
        "bare headless App should not provision native-window hook state"
    );

    let windowed = App::new();
    assert!(
        windowed
            .world()
            .resource::<NativeWindowHookRegistryResource>()
            .is_err(),
        "bare windowed App should not provision native-window hook state before adapter selection"
    );
}

#[test]
fn bounded_headless_execution_does_not_require_native_window_hook_registry() {
    let app = App::headless();
    assert!(
        app.world()
            .resource::<NativeWindowHookRegistryResource>()
            .is_err()
    );

    let app = app
        .run_for_frames(1)
        .expect("bounded headless execution should not require native-window hook state");

    assert!(
        app.world()
            .resource::<NativeWindowHookRegistryResource>()
            .is_err(),
        "bounded headless execution should not synthesize native-window hook state"
    );
}
