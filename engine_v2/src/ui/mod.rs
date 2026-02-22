mod template;

pub use template::*;

use ecs::{EntityHandle, World, WorldBuilderExt};
use std::path::PathBuf;
use std::time::SystemTime;
#[cfg(test)]
mod tests;

#[derive(Debug, Copy, Clone)]
pub struct UiNode {
    pub visible: bool,
    pub z: i32,
}

#[derive(Debug, Copy, Clone)]
pub struct UiTransform {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct UiStyle {
    pub bg_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub struct UiText {
    pub content: String,
    pub color: [f32; 4],
    pub size: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct UiInputField {
    pub cursor: usize,
    pub focused: bool,
    pub submit_requested: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct UiButton {
    pub enabled: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct UiInteraction {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct UiDirty {
    pub layout: bool,
    pub style: bool,
    pub text: bool,
}

#[derive(Debug, Clone)]
pub struct UiSubmitEvent {
    pub line: String,
}

#[derive(Debug, Clone)]
pub enum UiDrawCmd {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        radius: f32,
    },
    Text {
        x: f32,
        y: f32,
        content: String,
        color: [f32; 4],
        size: f32,
    },
}

#[derive(Debug, Default, Clone)]
pub struct UiDrawList {
    pub commands: Vec<UiDrawCmd>,
}

#[derive(Debug, Clone)]
pub enum UiBatchCmd {
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
        radius: f32,
    },
    Text {
        x: f32,
        y: f32,
        content: String,
        color: [f32; 4],
        size: f32,
    },
}

#[derive(Debug, Default, Clone)]
pub struct UiBatchList {
    pub commands: Vec<UiBatchCmd>,
}

#[derive(Debug, Copy, Clone)]
pub struct UiLayoutConfig {
    pub panel_width_ratio: f32,
    pub panel_height_ratio: f32,
    pub panel_min_width: f32,
    pub panel_min_height: f32,
    pub outer_margin: f32,
    pub inner_padding: f32,
    pub footer_offset: f32,
    pub input_height: f32,
    pub button_width: f32,
    pub input_button_gap: f32,
}

impl Default for UiLayoutConfig {
    fn default() -> Self {
        Self {
            panel_width_ratio: 0.6,
            panel_height_ratio: 0.45,
            panel_min_width: 480.0,
            panel_min_height: 280.0,
            outer_margin: 24.0,
            inner_padding: 12.0,
            footer_offset: 40.0,
            input_height: 28.0,
            button_width: 100.0,
            input_button_gap: 8.0,
        }
    }
}

#[derive(Debug)]
pub struct ConsoleUiState {
    pub root: EntityHandle,
    pub scrollback: EntityHandle,
    pub input: EntityHandle,
    pub confirm_button: EntityHandle,
    pub batches: UiBatchList,
    pub draw_list: UiDrawList,
    pub lines: Vec<String>,
    pub max_lines: usize,
    pub scroll_lines_from_bottom: usize,
    pub caret_blink_timer: f32,
    pub caret_visible: bool,
    pub layout: UiLayoutConfig,
    pub template_path: Option<PathBuf>,
    pub template_modified: Option<SystemTime>,
    pub screen_size: (f32, f32),
    pub scale: f32,
    pub layout_dirty: bool,
}

pub fn initialize_console_ui(world: &mut World) -> ConsoleUiState {
    let root = world
        .entity()
        .with(UiNode { visible: true, z: 0 })
        .with(UiTransform {
            x: 32.0,
            y: 32.0,
            w: 640.0,
            h: 360.0,
        })
        .with(UiStyle {
            bg_color: [0.08, 0.09, 0.11, 1.0],
            border_color: [0.20, 0.22, 0.28, 1.0],
            border_width: 1.0,
            radius: 6.0,
        })
        .with(UiDirty {
            layout: true,
            style: true,
            text: true,
        })
        .build();

    let scrollback = world
        .entity()
        .with(UiNode { visible: true, z: 1 })
        .with(UiTransform {
            x: 44.0,
            y: 52.0,
            w: 616.0,
            h: 280.0,
        })
        .with(UiText {
            content: "grotto> boot complete".to_string(),
            color: [0.78, 0.86, 0.94, 1.0],
            size: 14.0,
        })
        .with(UiDirty {
            layout: true,
            style: false,
            text: true,
        })
        .build();

    let input = world
        .entity()
        .with(UiNode { visible: true, z: 1 })
        .with(UiTransform {
            x: 44.0,
            y: 338.0,
            w: 504.0,
            h: 24.0,
        })
        .with(UiText {
            content: "grotto> ".to_string(),
            color: [0.95, 0.95, 0.95, 1.0],
            size: 14.0,
        })
        .with(UiInputField {
            cursor: 0,
            focused: true,
            submit_requested: false,
        })
        .with(UiInteraction {
            hovered: false,
            pressed: false,
            clicked: false,
            focused: true,
        })
        .with(UiDirty {
            layout: true,
            style: false,
            text: true,
        })
        .build();

    let confirm_button = world
        .entity()
        .with(UiNode { visible: true, z: 2 })
        .with(UiTransform {
            x: 556.0,
            y: 338.0,
            w: 104.0,
            h: 24.0,
        })
        .with(UiStyle {
            bg_color: [0.16, 0.30, 0.22, 1.0],
            border_color: [0.32, 0.56, 0.42, 1.0],
            border_width: 1.0,
            radius: 4.0,
        })
        .with(UiText {
            content: "Confirm".to_string(),
            color: [0.95, 0.96, 0.95, 1.0],
            size: 13.0,
        })
        .with(UiButton { enabled: true })
        .with(UiInteraction {
            hovered: false,
            pressed: false,
            clicked: false,
            focused: false,
        })
        .with(UiDirty {
            layout: true,
            style: true,
            text: true,
        })
        .build();

    ConsoleUiState {
        root,
        scrollback,
        input,
        confirm_button,
        batches: UiBatchList::default(),
        draw_list: UiDrawList::default(),
        lines: vec!["grotto> boot complete".to_string()],
        max_lines: 500,
        scroll_lines_from_bottom: 0,
        caret_blink_timer: 0.0,
        caret_visible: true,
        layout: UiLayoutConfig::default(),
        template_path: None,
        template_modified: None,
        screen_size: (1280.0, 720.0),
        scale: 1.0,
        layout_dirty: true,
    }
}
