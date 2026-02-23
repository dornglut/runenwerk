# SceneManager UI Example (Template-Driven)

This example should demonstrate a scene-manager workflow where `main.rs` stays minimal and scenes are declared from `.ron` UI templates in local example assets.

## Target `main.rs` API

The intended usage in `engine/examples/scene_manager_ui/main.rs` is:

```rust
const MAIN_MENU_SCENE: &str = ".../main_menu.ron";
const SETTINGS_MENU_SCENE: &str = ".../settings_menu.ron";
const PAUSE_MENU_SCENE: &str = ".../pause_menu.ron";
const GAME_SCENE: &str = ".../game_scene.ron";

App::new()
    .add_scene(MAIN_MENU_SCENE)
    .add_scene(SETTINGS_MENU_SCENE)
    .add_scene(PAUSE_MENU_SCENE)
    .add_scene(GAME_SCENE)
    .run()?;
```

`App::new()` now provides the default engine plugin stack automatically. `add_scene(".../foo.ron")` derives scene ids from template filenames and maps them to runtime `SceneHandle`s internally. Scene flow and template hot-reload are handled by the built-in `ScenePlugin`.

## Scene Flow

- `main_menu`: buttons to go to `game_scene` and `settings_menu`.
- `settings_menu`: button to go back to the previous scene.
- `game_scene`: text panel that shows `"Gameplay Preview"`.
- `pause_menu`: triggered by `Esc` in `game_scene`, with `settings` and `main menu` buttons.

## Demo Presentation

- Scene-template demo mode uses centered panel/content layout.
- Default world logs and diagnostics overlays are suppressed for this example.
- The default console input field is hidden for this flow.

## Assets Layout

Use assets under:

`engine/examples/scene_manager_ui/assets/`

Recommended structure:

```text
engine/examples/scene_manager_ui/assets/
  scenes/
    main_menu.ron
    settings_menu.ron
    game_scene.ron
    pause_menu.ron
  components/
    button_primary.ron
    panel.ron
    text_label.ron
```

## Templates

- Scene `.ron` files should reference component templates from `components/`.
- Component templates should define reusable UI building blocks such as buttons, panels, and labels.
- The runtime scene builder should construct each scene by loading these `.ron` templates instead of hardcoding UI layout in Rust.

## Run

```bash
cargo run -p engine --example scene_manager_ui
```

## Controls

- Mouse click: trigger scene buttons.
- `Esc` in `game_scene`: open `pause_menu`.
- `Esc` in `pause_menu`: return to `game_scene`.

## Source

- Entry: `engine/examples/scene_manager_ui/main.rs`
- Scene flow runtime: `engine/src/plugins/scene/template_flow.rs`
- App API: `engine::platform::App`
- Scene registration type: `engine::runtime::SceneRegistration`
- Scene handle type: `engine::runtime::SceneHandle`
- Assets: `engine/examples/scene_manager_ui/assets/`
