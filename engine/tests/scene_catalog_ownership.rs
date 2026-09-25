use engine::plugins::ScenePlugin;
use engine::prelude::{App, AppSceneExt, SceneCatalog};

const SCENE_TEMPLATE: &str = "engine/tests/fixtures/scene_templates/main_menu.ron";

#[test]
fn bare_app_does_not_install_scene_catalog() {
    let app = App::headless();

    assert!(app.world().resource::<SceneCatalog>().is_err());
}

#[test]
fn explicit_scene_registration_materializes_scene_catalog() {
    let mut app = App::headless();
    app.add_scene(SCENE_TEMPLATE);

    assert_eq!(app.registered_scene_count(), 1);
    assert!(app.world().resource::<SceneCatalog>().is_ok());
}

#[test]
fn scene_plugin_installs_empty_scene_catalog() {
    let mut app = App::headless();
    app.add_plugin(ScenePlugin);

    let catalog = app.world().resource::<SceneCatalog>().unwrap();
    assert!(catalog.is_empty());
}

#[test]
fn scene_plugin_preserves_explicit_scene_registrations() {
    let mut app = App::headless();
    app.add_scene(SCENE_TEMPLATE);

    app.add_plugin(ScenePlugin);

    let catalog = app.world().resource::<SceneCatalog>().unwrap();
    assert_eq!(catalog.len(), 1);
    assert!(catalog.handle("main_menu").is_some());
}

#[test]
fn late_scene_composition_is_rejected_without_mutating_catalog() {
    let mut app = App::headless();
    app.add_scene(SCENE_TEMPLATE);
    let mut app = app
        .run_for_frames(0)
        .expect("initial Scene composition should admit");
    let before = app.registered_scene_count();

    app.add_scene_template(SCENE_TEMPLATE);

    assert_eq!(app.registered_scene_count(), before);
    let error = app
        .run_for_frames(0)
        .err()
        .expect("late Scene composition should reject the next execution");
    let message = format!("{error:#}");
    assert!(message.contains("App composition admission rejected"));
    assert!(message.contains("add_scene_template"));
}
