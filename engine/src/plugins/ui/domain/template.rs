use super::{ConsoleUiRuntimeState, ConsoleUiState, UiNode, UiStyle, UiText};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DEFAULT_TEMPLATE_PATHS: &[&str] = &["assets/ui/console.ron", "engine/assets/ui/console.ron"];

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ConsoleUiTemplate {
    pub nodes: Option<Vec<UiNodeTemplate>>,
    pub max_lines: Option<usize>,
    pub root_style: Option<UiStyleTemplate>,
    pub scrollback_text: Option<UiTextTemplate>,
    pub input_text: Option<UiTextTemplate>,
    pub confirm_button: Option<UiButtonTemplate>,
    pub layout: Option<UiLayoutTemplate>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum UiNodeKind {
    Panel,
    Scrollback,
    Input,
    Button,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UiNodeTemplate {
    pub id: String,
    pub kind: Option<UiNodeKind>,
    pub visible: Option<bool>,
    pub style: Option<UiStyleTemplate>,
    pub text: Option<UiTextTemplate>,
    pub children: Vec<UiNodeTemplate>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UiStyleTemplate {
    pub bg_color: Option<(f32, f32, f32, f32)>,
    pub border_color: Option<(f32, f32, f32, f32)>,
    pub border_width: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UiTextTemplate {
    pub content: Option<String>,
    pub color: Option<(f32, f32, f32, f32)>,
    pub size: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UiButtonTemplate {
    pub style: Option<UiStyleTemplate>,
    pub text: Option<UiTextTemplate>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct UiLayoutTemplate {
    pub panel_width_ratio: Option<f32>,
    pub panel_height_ratio: Option<f32>,
    pub panel_min_width: Option<f32>,
    pub panel_min_height: Option<f32>,
    pub outer_margin: Option<f32>,
    pub inner_padding: Option<f32>,
    pub footer_offset: Option<f32>,
    pub input_height: Option<f32>,
    pub button_width: Option<f32>,
    pub input_button_gap: Option<f32>,
    pub logs_width_ratio: Option<f32>,
    pub logs_height_ratio: Option<f32>,
    pub logs_min_width: Option<f32>,
    pub logs_min_height: Option<f32>,
    pub logs_margin: Option<f32>,
    pub show_scroll_indicators: Option<bool>,
    pub show_scroll_hints: Option<bool>,
}

fn canonical_existing_path(path: &str) -> Option<PathBuf> {
    let p = Path::new(path);
    if !p.exists() {
        return None;
    }
    fs::canonicalize(p).ok().or_else(|| Some(p.to_path_buf()))
}

pub fn discover_default_template_path() -> Option<PathBuf> {
    for candidate in DEFAULT_TEMPLATE_PATHS {
        if let Some(path) = canonical_existing_path(candidate) {
            return Some(path);
        }
    }
    None
}

fn file_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn apply_style(style: &mut UiStyle, patch: UiStyleTemplate) {
    if let Some(v) = patch.bg_color {
        style.bg_color = [v.0, v.1, v.2, v.3];
    }
    if let Some(v) = patch.border_color {
        style.border_color = [v.0, v.1, v.2, v.3];
    }
    if let Some(v) = patch.border_width {
        style.border_width = v.max(0.0);
    }
    if let Some(v) = patch.radius {
        style.radius = v.max(0.0);
    }
}

fn apply_text(text: &mut UiText, patch: UiTextTemplate, apply_content: bool) {
    if apply_content {
        if let Some(v) = patch.content {
            text.content = v;
        }
    }
    if let Some(v) = patch.color {
        text.color = [v.0, v.1, v.2, v.3];
    }
    if let Some(v) = patch.size {
        text.size = v.max(1.0);
    }
}

fn apply_layout<E>(ui: &mut ConsoleUiState<E>, patch: UiLayoutTemplate) {
    if let Some(v) = patch.panel_width_ratio {
        ui.layout.panel_width_ratio = v.clamp(0.1, 1.0);
    }
    if let Some(v) = patch.panel_height_ratio {
        ui.layout.panel_height_ratio = v.clamp(0.1, 1.0);
    }
    if let Some(v) = patch.panel_min_width {
        ui.layout.panel_min_width = v.max(1.0);
    }
    if let Some(v) = patch.panel_min_height {
        ui.layout.panel_min_height = v.max(1.0);
    }
    if let Some(v) = patch.outer_margin {
        ui.layout.outer_margin = v.max(0.0);
    }
    if let Some(v) = patch.inner_padding {
        ui.layout.inner_padding = v.max(0.0);
    }
    if let Some(v) = patch.footer_offset {
        ui.layout.footer_offset = v.max(0.0);
    }
    if let Some(v) = patch.input_height {
        ui.layout.input_height = v.max(1.0);
    }
    if let Some(v) = patch.button_width {
        ui.layout.button_width = v.max(1.0);
    }
    if let Some(v) = patch.input_button_gap {
        ui.layout.input_button_gap = v.max(0.0);
    }
    if let Some(v) = patch.logs_width_ratio {
        ui.layout.logs_width_ratio = v.clamp(0.1, 1.0);
    }
    if let Some(v) = patch.logs_height_ratio {
        ui.layout.logs_height_ratio = v.clamp(0.1, 1.0);
    }
    if let Some(v) = patch.logs_min_width {
        ui.layout.logs_min_width = v.max(1.0);
    }
    if let Some(v) = patch.logs_min_height {
        ui.layout.logs_min_height = v.max(1.0);
    }
    if let Some(v) = patch.logs_margin {
        ui.layout.logs_margin = v.max(0.0);
    }
    if let Some(v) = patch.show_scroll_indicators {
        ui.layout.show_scroll_indicators = v;
    }
    if let Some(v) = patch.show_scroll_hints {
        ui.layout.show_scroll_hints = v;
    }
}

fn entity_for_id<E: Copy>(ui: &ConsoleUiState<E>, id: &str) -> Option<E> {
    match id {
        "root" => Some(ui.root),
        "scrollback" => Some(ui.scrollback),
        "input" => Some(ui.input),
        "confirm_button" | "confirm" => Some(ui.confirm_button),
        _ => None,
    }
}

fn expected_kind_for_id(id: &str) -> Option<UiNodeKind> {
    match id {
        "root" => Some(UiNodeKind::Panel),
        "scrollback" => Some(UiNodeKind::Scrollback),
        "input" => Some(UiNodeKind::Input),
        "confirm_button" | "confirm" => Some(UiNodeKind::Button),
        _ => None,
    }
}

fn apply_single_node_template(
    world: &mut ecs::World,
    ui: &ConsoleUiRuntimeState,
    node: &UiNodeTemplate,
) {
    let Some(entity) = entity_for_id(ui, &node.id) else {
        return;
    };
    if let (Some(expected), Some(kind)) = (expected_kind_for_id(&node.id), node.kind.as_ref()) {
        if kind != &expected {
            return;
        }
    }
    if let Some(visible) = node.visible
        && let Ok(mut entity_ref) = world.entity_mut(entity)
        && let Some(mut ui_node) = entity_ref.get_mut::<UiNode>()
    {
        ui_node.visible = visible;
    }

    if let Some(style_patch) = node.style
        && let Ok(mut entity_ref) = world.entity_mut(entity)
        && let Some(mut style) = entity_ref.get_mut::<UiStyle>()
    {
        apply_style(&mut style, style_patch);
    }
    if let Some(text_patch) = node.text.clone()
        && let Ok(mut entity_ref) = world.entity_mut(entity)
        && let Some(mut text) = entity_ref.get_mut::<UiText>()
    {
        let apply_content = !matches!(node.id.as_str(), "input" | "scrollback");
        apply_text(&mut text, text_patch, apply_content);
    }
}

fn node_local_hash(node: &UiNodeTemplate) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    node.id.hash(&mut hasher);
    node.kind.hash(&mut hasher);
    node.visible.hash(&mut hasher);
    format!("{:?}", node.style).hash(&mut hasher);
    format!("{:?}", node.text).hash(&mut hasher);
    hasher.finish()
}

fn apply_nodes_diff(
    world: &mut ecs::World,
    ui: &mut ConsoleUiRuntimeState,
    previous: &HashMap<String, u64>,
    next: &mut HashMap<String, u64>,
    node: &UiNodeTemplate,
) {
    let hash = node_local_hash(node);
    let changed = previous.get(&node.id).copied() != Some(hash);
    next.insert(node.id.clone(), hash);

    if changed {
        apply_single_node_template(world, ui, node);
    }

    for child in &node.children {
        apply_nodes_diff(world, ui, previous, next, child);
    }
}

pub fn apply_console_template(
    world: &mut ecs::World,
    ui: &mut ConsoleUiRuntimeState,
    tpl: ConsoleUiTemplate,
) {
    if let Some(v) = tpl.max_lines {
        ui.max_lines = v.max(1);
    }

    if let Some(patch) = tpl.layout {
        apply_layout(ui, patch);
    }

    if let Some(patch) = tpl.root_style
        && let Ok(mut entity_ref) = world.entity_mut(ui.root)
        && let Some(mut style) = entity_ref.get_mut::<UiStyle>()
    {
        apply_style(&mut style, patch);
    }

    if let Some(patch) = tpl.scrollback_text
        && let Ok(mut entity_ref) = world.entity_mut(ui.scrollback)
        && let Some(mut text) = entity_ref.get_mut::<UiText>()
    {
        apply_text(&mut text, patch, false);
    }

    if let Some(patch) = tpl.input_text
        && let Ok(mut entity_ref) = world.entity_mut(ui.input)
        && let Some(mut text) = entity_ref.get_mut::<UiText>()
    {
        apply_text(&mut text, patch, false);
    }

    if let Some(button) = tpl.confirm_button {
        if let Some(style_patch) = button.style
            && let Ok(mut entity_ref) = world.entity_mut(ui.confirm_button)
            && let Some(mut style) = entity_ref.get_mut::<UiStyle>()
        {
            apply_style(&mut style, style_patch);
        }
        if let Some(text_patch) = button.text
            && let Ok(mut entity_ref) = world.entity_mut(ui.confirm_button)
            && let Some(mut text) = entity_ref.get_mut::<UiText>()
        {
            apply_text(&mut text, text_patch, true);
        }
    }

    if let Some(nodes) = tpl.nodes {
        let previous = ui.template_node_hashes.clone();
        let mut next = HashMap::new();
        for node in &nodes {
            apply_nodes_diff(world, ui, &previous, &mut next, node);
        }
        ui.template_node_hashes = next;
    } else {
        ui.template_node_hashes.clear();
    }

    ui.layout_dirty = true;
}

pub fn initialize_template_tracking<E>(ui: &mut ConsoleUiState<E>) {
    if ui.template_path.is_none() {
        ui.template_path = discover_default_template_path();
    }
}

pub fn reload_console_template_if_changed(
    world: &mut ecs::World,
    ui: &mut ConsoleUiRuntimeState,
    force: bool,
) -> anyhow::Result<bool> {
    initialize_template_tracking(ui);
    let Some(path) = ui.template_path.clone() else {
        return Ok(false);
    };

    let modified = file_modified(&path);
    if !force {
        if let (Some(prev), Some(now)) = (ui.template_modified, modified) {
            if now <= prev {
                return Ok(false);
            }
        } else if ui.template_modified.is_some() && modified.is_none() {
            return Ok(false);
        }
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed reading UI template at {}", path.display()))?;
    let template: ConsoleUiTemplate = match ron::from_str(&raw) {
        Ok(template) => template,
        Err(err) => {
            ui.template_modified = modified;
            return Err(anyhow::anyhow!(err))
                .with_context(|| format!("failed parsing RON UI template at {}", path.display()));
        }
    };

    apply_console_template(world, ui, template);
    ui.template_modified = modified;
    Ok(true)
}

pub fn load_console_template(
    world: &mut ecs::World,
    ui: &mut ConsoleUiRuntimeState,
    path: &Path,
) -> anyhow::Result<()> {
    ui.template_path = Some(path.to_path_buf());
    ui.template_modified = None;
    let _ = reload_console_template_if_changed(world, ui, true)?;
    Ok(())
}

fn serialize_style(style: &UiStyle) -> UiStyleTemplate {
    UiStyleTemplate {
        bg_color: Some((
            style.bg_color[0],
            style.bg_color[1],
            style.bg_color[2],
            style.bg_color[3],
        )),
        border_color: Some((
            style.border_color[0],
            style.border_color[1],
            style.border_color[2],
            style.border_color[3],
        )),
        border_width: Some(style.border_width),
        radius: Some(style.radius),
    }
}

fn serialize_text(text: &UiText, include_content: bool) -> UiTextTemplate {
    UiTextTemplate {
        content: include_content.then(|| text.content.clone()),
        color: Some((text.color[0], text.color[1], text.color[2], text.color[3])),
        size: Some(text.size),
    }
}

pub fn export_console_template(
    world: &ecs::World,
    ui: &ConsoleUiRuntimeState,
) -> ConsoleUiTemplate {
    let root_style = world
        .get::<UiStyle>(ui.root)
        .map(serialize_style)
        .unwrap_or_default();
    let scroll_text = world
        .get::<UiText>(ui.scrollback)
        .map(|t| serialize_text(t, false))
        .unwrap_or_default();
    let input_text = world
        .get::<UiText>(ui.input)
        .map(|t| serialize_text(t, false))
        .unwrap_or_default();
    let button_style = world
        .get::<UiStyle>(ui.confirm_button)
        .map(serialize_style)
        .unwrap_or_default();
    let button_text = world
        .get::<UiText>(ui.confirm_button)
        .map(|t| serialize_text(t, true))
        .unwrap_or_default();

    let layout = UiLayoutTemplate {
        panel_width_ratio: Some(ui.layout.panel_width_ratio),
        panel_height_ratio: Some(ui.layout.panel_height_ratio),
        panel_min_width: Some(ui.layout.panel_min_width),
        panel_min_height: Some(ui.layout.panel_min_height),
        outer_margin: Some(ui.layout.outer_margin),
        inner_padding: Some(ui.layout.inner_padding),
        footer_offset: Some(ui.layout.footer_offset),
        input_height: Some(ui.layout.input_height),
        button_width: Some(ui.layout.button_width),
        input_button_gap: Some(ui.layout.input_button_gap),
        logs_width_ratio: Some(ui.layout.logs_width_ratio),
        logs_height_ratio: Some(ui.layout.logs_height_ratio),
        logs_min_width: Some(ui.layout.logs_min_width),
        logs_min_height: Some(ui.layout.logs_min_height),
        logs_margin: Some(ui.layout.logs_margin),
        show_scroll_indicators: Some(ui.layout.show_scroll_indicators),
        show_scroll_hints: Some(ui.layout.show_scroll_hints),
    };

    ConsoleUiTemplate {
        max_lines: Some(ui.max_lines),
        layout: Some(layout),
        root_style: Some(root_style),
        scrollback_text: Some(scroll_text),
        input_text: Some(input_text),
        confirm_button: Some(UiButtonTemplate {
            style: Some(button_style),
            text: Some(button_text),
        }),
        nodes: Some(vec![UiNodeTemplate {
            id: "root".to_string(),
            kind: Some(UiNodeKind::Panel),
            visible: world.get::<UiNode>(ui.root).map(|n| n.visible),
            style: None,
            text: None,
            children: vec![
                UiNodeTemplate {
                    id: "scrollback".to_string(),
                    kind: Some(UiNodeKind::Scrollback),
                    visible: world.get::<UiNode>(ui.scrollback).map(|n| n.visible),
                    style: None,
                    text: None,
                    children: Vec::new(),
                },
                UiNodeTemplate {
                    id: "input".to_string(),
                    kind: Some(UiNodeKind::Input),
                    visible: world.get::<UiNode>(ui.input).map(|n| n.visible),
                    style: None,
                    text: None,
                    children: Vec::new(),
                },
                UiNodeTemplate {
                    id: "confirm_button".to_string(),
                    kind: Some(UiNodeKind::Button),
                    visible: world.get::<UiNode>(ui.confirm_button).map(|n| n.visible),
                    style: None,
                    text: None,
                    children: Vec::new(),
                },
            ],
        }]),
    }
}

pub fn save_console_template_to_disk(
    world: &ecs::World,
    ui: &mut ConsoleUiRuntimeState,
) -> anyhow::Result<PathBuf> {
    initialize_template_tracking(ui);
    let path = ui
        .template_path
        .clone()
        .or_else(discover_default_template_path)
        .unwrap_or_else(|| PathBuf::from("assets/ui/console.ron"));

    let template = export_console_template(world, ui);
    let pretty = ron::ser::PrettyConfig::default();
    let raw =
        ron::ser::to_string_pretty(&template, pretty).context("failed serializing UI template")?;
    fs::write(&path, raw)
        .with_context(|| format!("failed writing UI template at {}", path.display()))?;
    ui.template_path = Some(path.clone());
    ui.template_modified = file_modified(&path);
    Ok(path)
}
