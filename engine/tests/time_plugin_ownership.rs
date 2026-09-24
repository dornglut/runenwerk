use engine::plugins::TimePlugin;
use engine::prelude::{App, Time};

#[test]
fn bare_app_does_not_install_time() {
    let app = App::headless();

    assert!(app.world().resource::<Time>().is_err());
}

#[test]
fn time_plugin_installs_time() {
    let mut app = App::headless();
    app.add_plugin(TimePlugin);

    assert!(app.world().resource::<Time>().is_ok());
}

#[test]
fn time_plugin_preserves_preinserted_time_configuration() {
    let mut app = App::headless();
    let mut time = Time::new();
    time.delta_seconds = 0.125;
    app.insert_resource(time);

    app.add_plugin(TimePlugin);

    assert_eq!(app.world().resource::<Time>().unwrap().delta_seconds, 0.125);
}

#[test]
fn time_plugin_advances_owned_time_state() {
    let mut app = App::headless();
    let mut time = Time::new();
    time.delta_seconds = -1.0;
    app.insert_resource(time);
    app.add_plugin(TimePlugin);

    let app = app
        .run_for_frames(1)
        .expect("TimePlugin should advance a headless frame");
    let delta_seconds = app.world().resource::<Time>().unwrap().delta_seconds;

    assert!(delta_seconds >= 0.0);
    assert!(delta_seconds <= 0.25);
}
