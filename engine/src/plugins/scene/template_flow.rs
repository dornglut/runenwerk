use super::domain::SceneCommand;
use crate::plugins::input::domain::action as input_action;
use crate::plugins::ui::domain::{
    UiBatchCmd, UiButton, UiButtonClickEvent, UiButtonTemplate, UiInteraction, UiNode,
    UiPresentationMode, UiStyle, UiStyleTemplate, UiText, UiTextTemplate, UiTransform,
};
use crate::runtime::{EngineData, SceneCatalog, SceneHandle};
use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const MAIN_MENU_ID: &str = "main_menu";
const SETTINGS_MENU_ID: &str = "settings_menu";
const PAUSE_MENU_ID: &str = "pause_menu";
const GAME_SCENE_ID: &str = "game_scene";

pub fn setup_template_flow(data: &mut EngineData) -> Result<()> {
    if data.scene_catalog.is_empty() {
        return Ok(());
    }
    if data
        .scene
        .overlay_runtime
        .world
        .has_resource::<SceneManagerUiResource>()
    {
        return Ok(());
    }

    let handles = resolve_handles(&data.scene_catalog)?;
    let (scenes, watched_files) = load_scene_catalog(&data.scene_catalog)?;
    if !scenes.contains_key(&handles.main) {
        bail!("scene handle {} was not loaded", handles.main.index());
    }

    data.scene.overlay_runtime.ui.presentation_mode = UiPresentationMode::CenteredDemo;
    data.scene.overlay_runtime.ui.layout.panel_width_ratio = 0.48;
    data.scene.overlay_runtime.ui.layout.panel_height_ratio = 0.52;
    data.scene.overlay_runtime.ui.layout.panel_min_width = 540.0;
    data.scene.overlay_runtime.ui.layout.panel_min_height = 340.0;
    data.scene.overlay_runtime.ui.layout.inner_padding = 18.0;
    data.scene.overlay_runtime.ui.layout.footer_offset = 72.0;
    data.scene.overlay_runtime.ui.layout.input_height = 36.0;
    data.scene.overlay_runtime.ui.layout.button_width = 154.0;
    data.scene.overlay_runtime.ui.layout.input_button_gap = 12.0;
    data.scene.overlay_runtime.ui.layout.show_scroll_indicators = false;
    data.scene.overlay_runtime.ui.layout.show_scroll_hints = false;
    data.scene.overlay_runtime.ui.layout_dirty = true;

    let secondary_button = data.scene.overlay_runtime.world.spawn_bundle((
        UiNode {
            visible: false,
            z: 2,
        },
        UiTransform {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 28.0,
        },
        UiStyle {
            bg_color: [0.16, 0.26, 0.20, 1.0],
            border_color: [0.34, 0.54, 0.46, 1.0],
            border_width: 1.0,
            radius: 4.0,
        },
        UiText {
            content: "Button".to_string(),
            color: [0.94, 0.96, 0.94, 1.0],
            size: 13.0,
        },
        UiButton { enabled: true },
        UiInteraction {
            hovered: false,
            pressed: false,
            clicked: false,
            focused: false,
        },
    ));

    data.scene
        .overlay_runtime
        .world
        .insert_resource(SceneManagerUiResource {
            handles,
            active_scene: handles.main,
            previous_scene: None,
            applied_scene: None,
            scenes,
            watched_files,
            revision: 0,
            watch_enabled: true,
            secondary_button,
            pause_toggle_key_down: false,
        });

    data.scene.set_active_overlay_visible(true);
    data.scene.queue(SceneCommand::PauseWorld(true));
    data.scene.apply_pending()?;
    apply_active_scene_if_needed(data)?;

    Ok(())
}

#[derive(Debug, Copy, Clone)]
struct SceneFlowHandles {
    main: SceneHandle,
    settings: Option<SceneHandle>,
    pause: Option<SceneHandle>,
    game: Option<SceneHandle>,
}

#[derive(Debug)]
struct SceneManagerUiResource {
    handles: SceneFlowHandles,
    active_scene: SceneHandle,
    previous_scene: Option<SceneHandle>,
    applied_scene: Option<SceneHandle>,
    scenes: HashMap<SceneHandle, LoadedScene>,
    watched_files: HashMap<PathBuf, Option<SystemTime>>,
    revision: u64,
    watch_enabled: bool,
    secondary_button: ecs::EntityHandle,
    pause_toggle_key_down: bool,
}

#[derive(Debug, Clone)]
struct LoadedScene {
    id: String,
    body: String,
    panel_style: UiStyleTemplate,
    text_style: UiTextTemplate,
    primary_button: Option<LoadedButton>,
    secondary_button: Option<LoadedButton>,
}

#[derive(Debug, Clone)]
struct LoadedButton {
    label: String,
    action: SceneAction,
    template: UiButtonTemplate,
}

#[derive(Debug, Clone)]
enum SceneAction {
    GoTo(SceneHandle),
    Back,
    MainMenu,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SceneActionRaw {
    GoTo(String),
    Back,
    MainMenu,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct SceneFileTemplate {
    body: String,
    panel_component: String,
    text_component: String,
    primary_button: Option<SceneFileButton>,
    secondary_button: Option<SceneFileButton>,
}

#[derive(Debug, Clone, Deserialize)]
struct SceneFileButton {
    label: String,
    component: String,
    action: SceneActionRaw,
}

pub fn template_flow_enabled(data: &EngineData) -> bool {
    data.scene
        .overlay_runtime
        .world
        .has_resource::<SceneManagerUiResource>()
}

pub fn set_template_scene_by_id(data: &mut EngineData, scene_id: &str) -> Result<bool> {
    if !template_flow_enabled(data) {
        return Ok(false);
    }
    let Some(handle) = data.scene_catalog.handle(scene_id) else {
        return Ok(false);
    };

    {
        let state = flow_resource_mut(data)?;
        if !state.scenes.contains_key(&handle) {
            return Ok(false);
        }
        if state.active_scene != handle {
            apply_scene_action(state, SceneAction::GoTo(handle));
        }
        state.applied_scene = None;
    }

    apply_active_scene_if_needed(data)?;
    Ok(true)
}

pub fn scene_template_flow_system(data: &mut EngineData) -> Result<()> {
    if !template_flow_enabled(data) {
        return Ok(());
    }
    maybe_reload_templates(data)?;

    let secondary_button = flow_resource(data)?.secondary_button;
    update_secondary_button_layout(data, secondary_button);
    let click_events = data
        .scene
        .overlay_runtime
        .world
        .drain_events::<UiButtonClickEvent>();
    let primary_button = data.scene.overlay_runtime.ui.confirm_button;
    let primary_clicked_from_events = click_events
        .iter()
        .any(|event| event.entity == primary_button);
    let secondary_clicked_from_events = click_events
        .iter()
        .any(|event| event.entity == secondary_button);
    let primary_clicked = primary_clicked_from_events
        || data
            .scene
            .overlay_runtime
            .world
            .get_component::<UiInteraction>(primary_button)
            .map(|interaction| interaction.clicked)
            .unwrap_or(false);
    let secondary_clicked = secondary_clicked_from_events
        || data
            .scene
            .overlay_runtime
            .world
            .get_component::<UiInteraction>(secondary_button)
            .map(|interaction| interaction.clicked)
            .unwrap_or(false);
    let pause_toggle_pressed = data.input.toggle_pause_menu;
    let pause_toggle_key_down = data
        .input
        .action_down(input_action::SYSTEM_TOGGLE_PAUSE_MENU);

    let mut queued_action: Option<SceneAction> = None;
    {
        let state = flow_resource_mut(data)?;
        let toggle_pause_menu =
            pause_toggle_pressed || (pause_toggle_key_down && !state.pause_toggle_key_down);
        state.pause_toggle_key_down = pause_toggle_key_down;
        if toggle_pause_menu {
            if let (Some(game), Some(pause)) = (state.handles.game, state.handles.pause) {
                if state.active_scene == game {
                    state.previous_scene = Some(game);
                    state.active_scene = pause;
                } else if state.active_scene == pause {
                    state.active_scene = game;
                }
            }
            if state.handles.settings == Some(state.active_scene) {
                state.active_scene = state.previous_scene.take().unwrap_or(state.handles.main);
            }
        }

        if let Some(scene) = state.scenes.get(&state.active_scene) {
            if primary_clicked && let Some(button) = &scene.primary_button {
                queued_action = Some(button.action.clone());
            }
            if queued_action.is_none()
                && secondary_clicked
                && let Some(button) = &scene.secondary_button
            {
                queued_action = Some(button.action.clone());
            }
        }

        if let Some(action) = queued_action {
            apply_scene_action(state, action);
        }
    }

    apply_active_scene_if_needed(data)
}

pub fn scene_template_secondary_button_batch_system(data: &mut EngineData) -> Result<()> {
    if !template_flow_enabled(data) {
        return Ok(());
    }
    let secondary_button = flow_resource(data)?.secondary_button;
    let Some(node) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiNode>(secondary_button)
    else {
        return Ok(());
    };
    if !node.visible {
        return Ok(());
    }

    let Some(transform) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiTransform>(secondary_button)
    else {
        return Ok(());
    };
    let Some(style) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiStyle>(secondary_button)
    else {
        return Ok(());
    };
    let Some(button) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiButton>(secondary_button)
    else {
        return Ok(());
    };
    let Some(interaction) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiInteraction>(secondary_button)
    else {
        return Ok(());
    };
    let Some(text) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiText>(secondary_button)
    else {
        return Ok(());
    };

    let color = if !button.enabled {
        tint_color(style.bg_color, 0.6)
    } else if interaction.pressed {
        tint_color(style.bg_color, 0.82)
    } else if interaction.hovered {
        tint_color(style.bg_color, 1.18)
    } else {
        style.bg_color
    };

    let ui_scale = data.scene.overlay_runtime.ui.scale.max(1.0);
    data.scene
        .overlay_runtime
        .ui
        .batches
        .commands
        .push(UiBatchCmd::Rect {
            x: transform.x,
            y: transform.y,
            w: transform.w,
            h: transform.h,
            color,
            radius: style.radius * ui_scale,
        });

    let text_size = text.size * ui_scale;
    let text_w = estimate_text_width(&text.content, text_size);
    let text_x = transform.x + ((transform.w - text_w) * 0.5).max(0.0);
    let text_y = transform.y + ((transform.h - text_size) * 0.5).max(0.0);
    data.scene
        .overlay_runtime
        .ui
        .batches
        .commands
        .push(UiBatchCmd::Text {
            x: text_x,
            y: text_y,
            content: text.content.clone(),
            color: text.color,
            size: text_size,
            clip: Some([transform.x, transform.y, transform.w, transform.h]),
        });

    Ok(())
}

fn resolve_handles(scene_catalog: &SceneCatalog) -> Result<SceneFlowHandles> {
    let main = scene_catalog
        .handle(MAIN_MENU_ID)
        .or_else(|| scene_catalog.iter().next().map(|scene| scene.handle))
        .ok_or_else(|| anyhow!("missing any scene registrations"))?;
    let settings = scene_catalog.handle(SETTINGS_MENU_ID);
    let pause = scene_catalog.handle(PAUSE_MENU_ID);
    let game = scene_catalog.handle(GAME_SCENE_ID);
    Ok(SceneFlowHandles {
        main,
        settings,
        pause,
        game,
    })
}

fn apply_scene_action(state: &mut SceneManagerUiResource, action: SceneAction) {
    match action {
        SceneAction::GoTo(target) => {
            if Some(target) == state.handles.settings {
                state.previous_scene = Some(state.active_scene);
            } else if target == state.handles.main || Some(target) == state.handles.game {
                state.previous_scene = None;
            }
            state.active_scene = target;
        }
        SceneAction::Back => {
            state.active_scene = state.previous_scene.take().unwrap_or(state.handles.main);
        }
        SceneAction::MainMenu => {
            state.previous_scene = None;
            state.active_scene = state.handles.main;
        }
    }
}

fn flow_resource(data: &EngineData) -> Result<&SceneManagerUiResource> {
    data.scene
        .overlay_runtime
        .world
        .get_resource::<SceneManagerUiResource>()
        .ok_or_else(|| anyhow!("missing scene manager flow resource"))
}

fn flow_resource_mut(data: &mut EngineData) -> Result<&mut SceneManagerUiResource> {
    data.scene
        .overlay_runtime
        .world
        .get_resource_mut::<SceneManagerUiResource>()
        .ok_or_else(|| anyhow!("missing scene manager flow resource"))
}

fn maybe_reload_templates(data: &mut EngineData) -> Result<()> {
    let watch_enabled = flow_resource(data)?.watch_enabled;
    if !watch_enabled {
        return Ok(());
    }

    let changed = {
        let state = flow_resource_mut(data)?;
        let mut changed = false;
        for (path, previous_modified) in &mut state.watched_files {
            let current_modified = file_modified(path);
            if *previous_modified != current_modified {
                *previous_modified = current_modified;
                changed = true;
            }
        }
        changed
    };

    if !changed {
        return Ok(());
    }

    let (scenes, watched_files) = load_scene_catalog(&data.scene_catalog)?;
    let revision = {
        let state = flow_resource_mut(data)?;
        state.scenes = scenes;
        state.watched_files = watched_files;
        state.revision = state.revision.saturating_add(1);
        state.applied_scene = None;
        if !state.scenes.contains_key(&state.active_scene) {
            state.active_scene = state.handles.main;
        }
        state.revision
    };
    data.scene.overlay_runtime.ui.log_lines.push(format!(
        "[world] scene templates hot-reloaded (rev={revision})"
    ));

    Ok(())
}

fn apply_active_scene_if_needed(data: &mut EngineData) -> Result<()> {
    let (scene, secondary_button, revision, world_should_pause) = {
        let state = flow_resource_mut(data)?;
        if state.applied_scene == Some(state.active_scene) {
            return Ok(());
        }
        let scene = state
            .scenes
            .get(&state.active_scene)
            .cloned()
            .with_context(|| format!("missing scene handle {}", state.active_scene.index()))?;
        state.applied_scene = Some(state.active_scene);
        (
            scene,
            state.secondary_button,
            state.revision,
            state.handles.game != Some(state.active_scene),
        )
    };

    data.scene.set_active_overlay_visible(true);
    data.scene
        .queue(SceneCommand::PauseWorld(world_should_pause));
    data.scene.apply_pending()?;
    data.scene.overlay_runtime.ui.lines = if scene.body.is_empty() {
        Vec::new()
    } else {
        vec![scene.body]
    };
    data.scene.overlay_runtime.ui.scroll_lines_from_bottom = 0;
    data.scene.overlay_runtime.ui.scroll_horizontal_chars = 0;
    data.scene.overlay_runtime.ui.log_lines.clear();
    data.scene.overlay_runtime.ui.log_paused_lines.clear();
    data.scene.overlay_runtime.ui.log_scroll_lines_from_bottom = 0;
    data.scene.overlay_runtime.ui.log_scroll_horizontal_chars = 0;
    data.scene.overlay_runtime.ui.input_editor.text.clear();
    data.scene.overlay_runtime.ui.input_editor.cursor_chars = 0;
    data.scene.overlay_runtime.ui.input_editor.viewport_row = 0;
    data.scene.overlay_runtime.ui.layout_dirty = true;
    data.scene.overlay_runtime.ui.editor.status = format!("scene: {} (rev={revision})", scene.id);

    if let Some(root_style) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiStyle>(data.scene.overlay_runtime.ui.root)
    {
        apply_style_template(root_style, &scene.panel_style);
    }

    if let Some(scrollback_text) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiText>(data.scene.overlay_runtime.ui.scrollback)
    {
        apply_text_template(scrollback_text, &scene.text_style, false);
    }

    if let Some(input_node) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiNode>(data.scene.overlay_runtime.ui.input)
    {
        input_node.visible = false;
    }

    if let Some(primary) = scene.primary_button {
        if let Some(node) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiNode>(data.scene.overlay_runtime.ui.confirm_button)
        {
            node.visible = true;
        }
        if let Some(style) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiStyle>(data.scene.overlay_runtime.ui.confirm_button)
            && let Some(patch) = primary.template.style.as_ref()
        {
            apply_style_template(style, patch);
        }
        if let Some(text) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiText>(data.scene.overlay_runtime.ui.confirm_button)
        {
            text.content = primary.label;
            if let Some(patch) = primary.template.text.as_ref() {
                apply_text_template(text, patch, false);
            }
        }
        if let Some(button) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiButton>(data.scene.overlay_runtime.ui.confirm_button)
        {
            button.enabled = true;
        }
    } else if let Some(node) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiNode>(data.scene.overlay_runtime.ui.confirm_button)
    {
        node.visible = false;
    }

    if let Some(secondary) = scene.secondary_button {
        if let Some(node) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiNode>(secondary_button)
        {
            node.visible = true;
        }
        if let Some(style) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiStyle>(secondary_button)
            && let Some(patch) = secondary.template.style.as_ref()
        {
            apply_style_template(style, patch);
        }
        if let Some(text) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiText>(secondary_button)
        {
            text.content = secondary.label;
            if let Some(patch) = secondary.template.text.as_ref() {
                apply_text_template(text, patch, false);
            }
        }
        if let Some(button) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiButton>(secondary_button)
        {
            button.enabled = true;
        }
    } else if let Some(node) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiNode>(secondary_button)
    {
        node.visible = false;
    }

    Ok(())
}

fn update_secondary_button_layout(data: &mut EngineData, secondary_button: ecs::EntityHandle) {
    let visible = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiNode>(secondary_button)
        .map(|node| node.visible)
        .unwrap_or(false);

    if !visible {
        if let Some(interaction) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiInteraction>(secondary_button)
        {
            interaction.hovered = false;
            interaction.pressed = false;
            interaction.clicked = false;
        }
        return;
    }

    let centered_demo = matches!(
        data.scene.overlay_runtime.ui.presentation_mode,
        UiPresentationMode::CenteredDemo
    );

    if let Some(confirm) = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiTransform>(data.scene.overlay_runtime.ui.confirm_button)
        .copied()
        && let Some(secondary) = data
            .scene
            .overlay_runtime
            .world
            .get_component_mut::<UiTransform>(secondary_button)
    {
        let gap =
            if centered_demo { 12.0 } else { 8.0 } * data.scene.overlay_runtime.ui.scale.max(1.0);
        secondary.w = confirm.w;
        secondary.h = confirm.h;
        secondary.x = confirm.x - confirm.w - gap;
        secondary.y = confirm.y;
    }

    let secondary_rect = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiTransform>(secondary_button)
        .copied();
    let hovered = secondary_rect
        .as_ref()
        .map(|rect| point_in_rect(data.input.mouse_position, rect))
        .unwrap_or(false);
    let button_enabled = data
        .scene
        .overlay_runtime
        .world
        .get_component::<UiButton>(secondary_button)
        .map(|button| button.enabled)
        .unwrap_or(false);
    let clicked = hovered && button_enabled && data.input.left_mouse_pressed();
    let pressed = hovered && button_enabled && data.input.left_mouse_down();
    if clicked {
        data.scene
            .overlay_runtime
            .world
            .emit_event(UiButtonClickEvent {
                entity: secondary_button,
            });
    }
    if let Some(interaction) = data
        .scene
        .overlay_runtime
        .world
        .get_component_mut::<UiInteraction>(secondary_button)
    {
        interaction.hovered = hovered;
        interaction.clicked = clicked;
        interaction.pressed = pressed;
    }
}

fn point_in_rect(point: (f32, f32), rect: &UiTransform) -> bool {
    point.0 >= rect.x
        && point.0 <= rect.x + rect.w
        && point.1 >= rect.y
        && point.1 <= rect.y + rect.h
}

fn tint_color(color: [f32; 4], factor: f32) -> [f32; 4] {
    [
        (color[0] * factor).clamp(0.0, 1.0),
        (color[1] * factor).clamp(0.0, 1.0),
        (color[2] * factor).clamp(0.0, 1.0),
        color[3],
    ]
}

fn estimate_text_width(text: &str, size: f32) -> f32 {
    text.chars().count() as f32 * size * 0.58
}

fn apply_style_template(style: &mut UiStyle, patch: &UiStyleTemplate) {
    if let Some(value) = patch.bg_color {
        style.bg_color = [value.0, value.1, value.2, value.3];
    }
    if let Some(value) = patch.border_color {
        style.border_color = [value.0, value.1, value.2, value.3];
    }
    if let Some(value) = patch.border_width {
        style.border_width = value.max(0.0);
    }
    if let Some(value) = patch.radius {
        style.radius = value.max(0.0);
    }
}

fn apply_text_template(text: &mut UiText, patch: &UiTextTemplate, apply_content: bool) {
    if apply_content && let Some(value) = patch.content.as_ref() {
        text.content = value.clone();
    }
    if let Some(value) = patch.color {
        text.color = [value.0, value.1, value.2, value.3];
    }
    if let Some(value) = patch.size {
        text.size = value.max(1.0);
    }
}

fn load_scene_catalog(
    scene_catalog: &SceneCatalog,
) -> Result<(
    HashMap<SceneHandle, LoadedScene>,
    HashMap<PathBuf, Option<SystemTime>>,
)> {
    let mut scenes = HashMap::new();
    let mut watched_files = HashMap::new();

    for registration in scene_catalog.iter() {
        let scene_path = PathBuf::from(&registration.template_path);
        let scene_file: SceneFileTemplate = load_ron_file(&scene_path).with_context(|| {
            format!(
                "failed loading scene template '{}' from {}",
                registration.id,
                scene_path.display()
            )
        })?;
        watched_files.insert(scene_path.clone(), file_modified(&scene_path));

        let root = scene_path.parent().unwrap_or(Path::new("."));
        let panel_component_path =
            resolve_component_path(root, &scene_file.panel_component, "panel_component")?;
        let text_component_path =
            resolve_component_path(root, &scene_file.text_component, "text_component")?;
        watched_files.insert(
            panel_component_path.clone(),
            file_modified(&panel_component_path),
        );
        watched_files.insert(
            text_component_path.clone(),
            file_modified(&text_component_path),
        );

        let panel_style: UiStyleTemplate =
            load_ron_file(&panel_component_path).with_context(|| {
                format!(
                    "failed loading panel component {}",
                    panel_component_path.display()
                )
            })?;
        let text_style: UiTextTemplate =
            load_ron_file(&text_component_path).with_context(|| {
                format!(
                    "failed loading text component {}",
                    text_component_path.display()
                )
            })?;

        let primary_button = match scene_file.primary_button {
            Some(button) => Some(load_button(
                root,
                button,
                scene_catalog,
                &mut watched_files,
            )?),
            None => None,
        };
        let secondary_button = match scene_file.secondary_button {
            Some(button) => Some(load_button(
                root,
                button,
                scene_catalog,
                &mut watched_files,
            )?),
            None => None,
        };

        scenes.insert(
            registration.handle,
            LoadedScene {
                id: registration.id.clone(),
                body: scene_file.body,
                panel_style,
                text_style,
                primary_button,
                secondary_button,
            },
        );
    }

    Ok((scenes, watched_files))
}

fn load_button(
    root: &Path,
    button: SceneFileButton,
    scene_catalog: &SceneCatalog,
    watched_files: &mut HashMap<PathBuf, Option<SystemTime>>,
) -> Result<LoadedButton> {
    let component_path = resolve_component_path(root, &button.component, "button component")?;
    watched_files.insert(component_path.clone(), file_modified(&component_path));
    let template: UiButtonTemplate = load_ron_file(&component_path).with_context(|| {
        format!(
            "failed loading button component template {}",
            component_path.display()
        )
    })?;

    Ok(LoadedButton {
        label: button.label,
        action: resolve_action(button.action, scene_catalog)?,
        template,
    })
}

fn resolve_action(raw: SceneActionRaw, scene_catalog: &SceneCatalog) -> Result<SceneAction> {
    match raw {
        SceneActionRaw::GoTo(label) => scene_catalog
            .handle(&label)
            .map(SceneAction::GoTo)
            .ok_or_else(|| anyhow!("unknown target scene '{label}'")),
        SceneActionRaw::Back => Ok(SceneAction::Back),
        SceneActionRaw::MainMenu => Ok(SceneAction::MainMenu),
    }
}

fn resolve_component_path(root: &Path, raw: &str, field: &str) -> Result<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        bail!("scene field '{field}' cannot be empty");
    }
    let path = PathBuf::from(trimmed);
    Ok(if path.is_absolute() {
        path
    } else {
        root.join(path)
    })
}

fn load_ron_file<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed reading RON file {}", path.display()))?;
    ron::from_str::<T>(&raw).with_context(|| format!("failed parsing RON file {}", path.display()))
}

fn file_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
