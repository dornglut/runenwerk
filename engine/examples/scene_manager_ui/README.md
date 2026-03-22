# SceneManager UI Example (Template-Driven)

This example should demonstrate a scene-manager workflow where `main.rs` stays minimal and scenes are declared from `.ron` UI templates in local example assets.

## Target `main.rs` API

The intended usage in `engine/examples/scene_manager_ui/main.rs` is:

```rust
const MAIN_MENU_SCENE: &str = ".../main_menu.ron";
const SETTINGS_MENU_SCENE: &str = ".../settings_menu.ron";
const PAUSE_MENU_SCENE: &str = ".../pause_menu.ron";
const GAME_SCENE: &str = ".../game_scene.ron";
const LOADING_SCENE: &str = ".../loading_scene.ron";

App::new()
    .add_render_flow(build_scene_manager_ui_render_flow())
    .add_scene(LOADING_SCENE)
    .add_scene(MAIN_MENU_SCENE)
    .add_scene(SETTINGS_MENU_SCENE)
    .add_scene(PAUSE_MENU_SCENE)
    .add_scene(GAME_SCENE)
    .run()?;
```

`add_scene(".../foo.ron")` derives scene ids from template filenames and maps them to runtime `SceneHandle`s internally. Scene template loading/hot-reload are handled by `ScenePlugin`. Rendering still requires a flow registration in this example (`build_scene_manager_ui_render_flow`) so the builtin UI composite pass is executed.

## Scene Flow

- `main_menu`: buttons to go to `game_scene` and `settings_menu`.
- `settings_menu`: button to go back to the previous scene plus a demo button that emits custom events on click/hold.
- `game_scene`: text panel that shows `"Gameplay Preview"`.
- `pause_menu`: triggered by `Esc` in `game_scene`, with `settings` and `main menu` buttons.
- `loading_scene`: shown at startup until render warmup stabilizes, then transitions to `main_menu`.

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
    loading_scene.ron
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

### Scene Schema (Buttons + Triggers)

Each scene file supports:

```ron
(
  body: "Title",
  panel_component: "../components/panel.ron",
  text_component: "../components/text_label.ron",
  primary_button: Some((
    label: "Start",
    component: "../components/button_primary.ron",
    on_click: Some(go_to("game_scene")),
    on_press_start: Some(emit("start.press_start")),
    on_press_end: Some(emit("start.press_end")),
    on_hold: Some((
      threshold_ms: Some(650),
      repeat_ms: Some(250),
      action: Some(emit("start.hold")),
    )),
  )),
)
```

Supported button trigger fields:

- `on_click`: fires on pointer click edge.
- `on_press_start`: fires once when press begins.
- `on_press_end`: fires once when press ends.
- `on_hold`: fires after `threshold_ms`, then repeats if `repeat_ms` is set.

Supported trigger actions:

- `go_to("<scene_id>")`
- `back`
- `main_menu`
- `emit("<event_name>")`

Backward compatibility:

- `action: <...>` is still accepted and treated as `on_click`.

### Custom Events

- `emit("...")` publishes a scene-template event in this demo flow.
- Emitted events are written to overlay lines as `[scene-event] <event_name>`.
- Emitted events also update the editor status line and are logged via `tracing`.
- Emitted events are also available as typed ECS events:
  - type: `engine::plugins::scene::domain::SceneTemplateUiEvent`
  - fields: `name`, `scene_id`, `button`, `trigger`

Example consumer system:

```rust
use engine::prelude::{ResMut, WorldMut};
use engine::plugins::scene::domain::SceneTemplateUiEvent;

pub fn consume_scene_template_events_system(mut world: WorldMut) -> anyhow::Result<()> {
    let events = world.drain_events::<SceneTemplateUiEvent>();
    for event in events {
        tracing::info!(
            name = %event.name,
            scene = %event.scene_id,
            button = ?event.button,
            trigger = event.trigger,
            "received scene template ui event"
        );
    }
    Ok(())
}
```

## Authoring Pipeline Direction

This example should follow the shared engine authoring pipeline described in [docs/authoring-layer.md](../../../docs/authoring-layer.md).

Target split:

- authored assets:
  - `scenes/*.ron`
  - `components/*.ron`
- compiled artifacts:
  - resolved scene template
  - resolved reusable component-template set
- live runtime state:
  - spawned ECS entities/components
  - scene transition hooks
  - emitted scene-template events

Reload rules for this example:

- scene assets and referenced component templates form one dependency graph
- changing a component template invalidates all scenes that reference it
- a reload only applies when the full affected set compiles successfully
- failed reloads keep the last-known-good scene/template generation active

Diagnostics should report:

- the root scene asset
- the referenced component-template path if applicable
- the include chain that led to the failure
- the exact field/action that failed validation

## Run

```bash
cargo run -p engine --example scene_manager_ui
```

## Controls

- Mouse click: trigger scene buttons.
- Mouse hold: trigger `on_hold` for buttons configured with hold actions.
- `Esc` in `game_scene`: open `pause_menu`.
- `Esc` in `pause_menu`: return to `game_scene`.

## Source

- Entry: `engine/examples/scene_manager_ui/main.rs`
- Render flow builder: `build_scene_manager_ui_render_flow()` in `engine/examples/scene_manager_ui/main.rs`
- Scene flow runtime: `engine/src/plugins/scene/mod.rs`
- App API: `engine::App`
- Scene registration helpers: `App::add_scene(...)`, `App::add_scene_template(...)`
- Assets: `engine/examples/scene_manager_ui/assets/`
