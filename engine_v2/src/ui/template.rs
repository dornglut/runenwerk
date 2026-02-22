use super::{ConsoleUiState, UiStyle, UiText};
use anyhow::Context;
use ecs::{EntityHandle, World};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const DEFAULT_TEMPLATE_PATHS: &[&str] =
    &["assets/ui/console.ron", "engine_v2/assets/ui/console.ron"];

#[derive(Debug, Clone, Deserialize, Default)]
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

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Hash)]
pub enum UiNodeKind {
    Panel,
    Scrollback,
    Input,
    Button,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct UiNodeTemplate {
    pub id: String,
    pub kind: Option<UiNodeKind>,
    pub style: Option<UiStyleTemplate>,
    pub text: Option<UiTextTemplate>,
    pub children: Vec<UiNodeTemplate>,
}

#[derive(Debug, Copy, Clone, Deserialize, Default)]
#[serde(default)]
pub struct UiStyleTemplate {
    pub bg_color: Option<(f32, f32, f32, f32)>,
    pub border_color: Option<(f32, f32, f32, f32)>,
    pub border_width: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct UiTextTemplate {
    pub content: Option<String>,
    pub color: Option<(f32, f32, f32, f32)>,
    pub size: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct UiButtonTemplate {
    pub style: Option<UiStyleTemplate>,
    pub text: Option<UiTextTemplate>,
}

#[derive(Debug, Copy, Clone, Deserialize, Default)]
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

fn apply_layout(ui: &mut ConsoleUiState, patch: UiLayoutTemplate) {
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
}

fn entity_for_id(ui: &ConsoleUiState, id: &str) -> Option<EntityHandle> {
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

fn apply_single_node_template(world: &mut World, ui: &ConsoleUiState, node: &UiNodeTemplate) {
    let Some(entity) = entity_for_id(ui, &node.id) else {
        return;
    };
    if let (Some(expected), Some(kind)) = (expected_kind_for_id(&node.id), node.kind.as_ref()) {
        if kind != &expected {
            // Skip mismatched declarations to avoid applying a wrong template subtree.
            return;
        }
    }

    if let Some(style_patch) = node.style {
        if let Some(style) = world.get_component_mut::<UiStyle>(entity) {
            apply_style(style, style_patch);
        }
    }
    if let Some(text_patch) = node.text.clone() {
        if let Some(text) = world.get_component_mut::<UiText>(entity) {
            let apply_content = !matches!(node.id.as_str(), "input" | "scrollback");
            apply_text(text, text_patch, apply_content);
        }
    }
}

fn node_local_hash(node: &UiNodeTemplate) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    node.id.hash(&mut hasher);
    node.kind.hash(&mut hasher);
    format!("{:?}", node.style).hash(&mut hasher);
    format!("{:?}", node.text).hash(&mut hasher);
    hasher.finish()
}

fn apply_nodes_diff(
    world: &mut World,
    ui: &mut ConsoleUiState,
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

pub fn apply_console_template(world: &mut World, ui: &mut ConsoleUiState, tpl: ConsoleUiTemplate) {
    if let Some(v) = tpl.max_lines {
        ui.max_lines = v.max(1);
    }

    if let Some(patch) = tpl.layout {
        apply_layout(ui, patch);
    }

    if let Some(patch) = tpl.root_style {
        if let Some(style) = world.get_component_mut::<UiStyle>(ui.root) {
            apply_style(style, patch);
        }
    }

    if let Some(patch) = tpl.scrollback_text {
        if let Some(text) = world.get_component_mut::<UiText>(ui.scrollback) {
            // Keep runtime-generated content unless explicitly requested.
            apply_text(text, patch, false);
        }
    }

    if let Some(patch) = tpl.input_text {
        if let Some(text) = world.get_component_mut::<UiText>(ui.input) {
            // Preserve what the player has typed during hot reloads.
            apply_text(text, patch, false);
        }
    }

    if let Some(button) = tpl.confirm_button {
        if let Some(style_patch) = button.style {
            if let Some(style) = world.get_component_mut::<UiStyle>(ui.confirm_button) {
                apply_style(style, style_patch);
            }
        }
        if let Some(text_patch) = button.text {
            if let Some(text) = world.get_component_mut::<UiText>(ui.confirm_button) {
                apply_text(text, text_patch, true);
            }
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

pub fn initialize_template_tracking(ui: &mut ConsoleUiState) {
    if ui.template_path.is_none() {
        ui.template_path = discover_default_template_path();
    }
}

pub fn reload_console_template_if_changed(
    world: &mut World,
    ui: &mut ConsoleUiState,
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
            // Avoid spamming the same parse error every frame; retry after next file modification.
            ui.template_modified = modified;
            return Err(anyhow::anyhow!(err))
                .with_context(|| format!("failed parsing RON UI template at {}", path.display()));
        }
    };

    apply_console_template(world, ui, template);
    ui.template_modified = modified;
    Ok(true)
}
