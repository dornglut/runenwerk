mod template;
mod text;

pub use template::*;
pub use text::*;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
#[cfg(test)]
mod tests;

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiNode {
    pub visible: bool,
    pub z: i32,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiTransform {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiStyle {
    pub bg_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub radius: f32,
}

#[derive(Debug, Clone, ecs::Component)]
pub struct UiText {
    pub content: String,
    pub color: [f32; 4],
    pub size: f32,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiInputField {
    pub cursor: usize,
    pub focused: bool,
    pub submit_requested: bool,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiButton {
    pub enabled: bool,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiInteraction {
    pub hovered: bool,
    pub pressed: bool,
    pub clicked: bool,
    pub focused: bool,
}

#[derive(Debug, Copy, Clone, ecs::Component)]
pub struct UiDirty {
    pub layout: bool,
    pub style: bool,
    pub text: bool,
}

#[derive(Debug, Clone)]
pub struct UiSubmitEvent {
    pub line: String,
}

pub type UiEntity = ecs::Entity;
pub type ConsoleUiRuntimeState = ConsoleUiState<UiEntity>;
pub type UiButtonRuntimeClickEvent = UiButtonClickEvent<UiEntity>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UiButtonClickEvent<E = ecs::Entity> {
    pub entity: E,
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
        clip: Option<[f32; 4]>,
    },
}

#[derive(Debug, Default, Clone)]
pub struct UiDrawList {
    pub commands: Vec<UiDrawCmd>,
}

#[derive(Debug, Clone)]
pub struct UiRenderShaderConfig {
    pub rect_shader_asset_id: String,
}

impl Default for UiRenderShaderConfig {
    fn default() -> Self {
        Self {
            rect_shader_asset_id: "ui_rect".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiWorldHudStats {
    pub visible: bool,
    pub player_x: f32,
    pub player_y: f32,
    pub enemies_alive: usize,
    pub enemy_kills: u32,
    pub panel_title: String,
    pub lines: Vec<String>,
}

impl Default for UiWorldHudStats {
    fn default() -> Self {
        Self {
            visible: false,
            player_x: 0.0,
            player_y: 0.0,
            enemies_alive: 0,
            enemy_kills: 0,
            panel_title: "World Stats".to_string(),
            lines: Vec::new(),
        }
    }
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
        clip: Option<[f32; 4]>,
    },
}

#[derive(Debug, Default, Clone)]
pub struct UiBatchList {
    pub commands: Vec<UiBatchCmd>,
}

#[derive(Debug, Default, Clone)]
pub struct EditorBuffer {
    pub text: String,
    pub cursor_chars: usize,
    pub viewport_row: usize,
    pub preferred_caret_x: Option<f32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum UiEditorNode {
    Root,
    Scrollback,
    Input,
    ConfirmButton,
}

#[derive(Debug, Clone)]
pub struct UiEditorState {
    pub enabled: bool,
    pub selected: Option<UiEditorNode>,
    pub dragging: bool,
    pub drag_pointer_offset: (f32, f32),
    pub status: String,
}

impl Default for UiEditorState {
    fn default() -> Self {
        Self {
            enabled: false,
            selected: None,
            dragging: false,
            drag_pointer_offset: (0.0, 0.0),
            status: "editor: off (F1 to toggle)".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UiTextMetrics {
    pub glyphs: HashMap<char, GlyphMetrics>,
    pub base_size: f32,
    pub fallback_advance: f32,
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
    pub logs_width_ratio: f32,
    pub logs_height_ratio: f32,
    pub logs_min_width: f32,
    pub logs_min_height: f32,
    pub logs_margin: f32,
    pub show_scroll_indicators: bool,
    pub show_scroll_hints: bool,
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
            logs_width_ratio: 0.36,
            logs_height_ratio: 0.30,
            logs_min_width: 320.0,
            logs_min_height: 180.0,
            logs_margin: 18.0,
            show_scroll_indicators: true,
            show_scroll_hints: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum UiPresentationMode {
    #[default]
    Standard,
    CenteredDemo,
}

#[derive(Debug)]
pub struct ConsoleUiState<E = ecs::Entity> {
    pub root: E,
    pub scrollback: E,
    pub input: E,
    pub confirm_button: E,
    pub batches: UiBatchList,
    pub draw_list: UiDrawList,
    pub input_editor: EditorBuffer,
    pub text_metrics: UiTextMetrics,
    pub lines: Vec<String>,
    pub log_lines: Vec<String>,
    pub log_paused_lines: Vec<String>,
    pub logs_paused: bool,
    pub max_lines: usize,
    pub scroll_lines_from_bottom: usize,
    pub scroll_horizontal_chars: usize,
    pub log_scroll_lines_from_bottom: usize,
    pub log_scroll_horizontal_chars: usize,
    pub caret_blink_timer: f32,
    pub caret_visible: bool,
    pub layout: UiLayoutConfig,
    pub template_path: Option<PathBuf>,
    pub template_modified: Option<SystemTime>,
    pub template_node_hashes: HashMap<String, u64>,
    pub screen_size: (f32, f32),
    pub scale: f32,
    pub layout_dirty: bool,
    pub presentation_mode: UiPresentationMode,
    pub editor: UiEditorState,
}

pub fn initialize_console_ui(world: &mut ecs::World) -> ConsoleUiRuntimeState {
    let font_provider = FileFontProvider;
    let atlas = font_provider.load_default_font();
    let fallback_advance = atlas
        .glyphs
        .get(&' ')
        .map(|glyph| glyph.advance_px)
        .unwrap_or(8.0);
    let text_metrics = UiTextMetrics {
        glyphs: atlas.glyphs,
        base_size: atlas.base_size.max(1.0),
        fallback_advance,
    };

    let root = world.spawn((
        UiNode {
            visible: true,
            z: 0,
        },
        UiTransform {
            x: 32.0,
            y: 32.0,
            w: 640.0,
            h: 360.0,
        },
        UiStyle {
            bg_color: [0.08, 0.09, 0.11, 1.0],
            border_color: [0.20, 0.22, 0.28, 1.0],
            border_width: 1.0,
            radius: 6.0,
        },
        UiDirty {
            layout: true,
            style: true,
            text: true,
        },
    ));

    let scrollback = world.spawn((
        UiNode {
            visible: true,
            z: 1,
        },
        UiTransform {
            x: 44.0,
            y: 52.0,
            w: 616.0,
            h: 280.0,
        },
        UiText {
            content: "grotto> boot complete".to_string(),
            color: [0.78, 0.86, 0.94, 1.0],
            size: 14.0,
        },
        UiDirty {
            layout: true,
            style: false,
            text: true,
        },
    ));

    let input = world.spawn((
        UiNode {
            visible: true,
            z: 1,
        },
        UiTransform {
            x: 44.0,
            y: 338.0,
            w: 504.0,
            h: 24.0,
        },
        UiText {
            content: "grotto> ".to_string(),
            color: [0.95, 0.95, 0.95, 1.0],
            size: 14.0,
        },
        UiInputField {
            cursor: 0,
            focused: true,
            submit_requested: false,
        },
        UiInteraction {
            hovered: false,
            pressed: false,
            clicked: false,
            focused: true,
        },
        UiDirty {
            layout: true,
            style: false,
            text: true,
        },
    ));

    let confirm_button = world.spawn((
        UiNode {
            visible: true,
            z: 2,
        },
        UiTransform {
            x: 556.0,
            y: 338.0,
            w: 104.0,
            h: 24.0,
        },
        UiStyle {
            bg_color: [0.16, 0.30, 0.22, 1.0],
            border_color: [0.32, 0.56, 0.42, 1.0],
            border_width: 1.0,
            radius: 4.0,
        },
        UiText {
            content: "Confirm".to_string(),
            color: [0.95, 0.96, 0.95, 1.0],
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
    if let Ok(mut entity) = world.entity_mut(confirm_button) {
        let _ = entity.insert(UiDirty {
            layout: true,
            style: true,
            text: true,
        });
    }

    ConsoleUiState {
        root,
        scrollback,
        input,
        confirm_button,
        batches: UiBatchList::default(),
        draw_list: UiDrawList::default(),
        input_editor: EditorBuffer::default(),
        text_metrics,
        lines: vec!["grotto> boot complete".to_string()],
        log_lines: vec!["[world] log window ready".to_string()],
        log_paused_lines: Vec::new(),
        logs_paused: false,
        max_lines: 500,
        scroll_lines_from_bottom: 0,
        scroll_horizontal_chars: 0,
        log_scroll_lines_from_bottom: 0,
        log_scroll_horizontal_chars: 0,
        caret_blink_timer: 0.0,
        caret_visible: true,
        layout: UiLayoutConfig::default(),
        template_path: None,
        template_modified: None,
        template_node_hashes: HashMap::new(),
        screen_size: (1280.0, 720.0),
        scale: 1.0,
        layout_dirty: true,
        presentation_mode: UiPresentationMode::Standard,
        editor: UiEditorState::default(),
    }
}
