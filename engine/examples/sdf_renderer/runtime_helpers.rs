// Owner: SDF Renderer Example - Runtime Helpers and Parsers
use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(super) struct SdfComputePipelineConfig {
    pub(super) id: String,
    pub(super) shader: String,
}

impl Default for SdfComputePipelineConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            shader: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(super) struct SdfRenderBuiltinPipelineConfig {
    pub(super) id: String,
    pub(super) builtin: String,
}

impl Default for SdfRenderBuiltinPipelineConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            builtin: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub(super) struct SdfRenderPassConfig {
    pub(super) id: String,
    pub(super) kind: SdfPassKindConfig,
    pub(super) pipeline: String,
    pub(super) executor: String,
    pub(super) reads: Vec<String>,
    pub(super) writes: Vec<String>,
    pub(super) depends_on: Vec<String>,
}

pub(super) fn apply_sdf_params(state: &mut SdfWorldState, params: &SdfParamsConfig) {
    let _ = (
        params.world_scene_label.as_str(),
        params.overlay_scene_label.as_str(),
        params.render_mesh_overlay,
    );
    state.world_bounds = params.world_bounds;
    state.camera_target = params.camera.target;
    state.camera_yaw = params.camera.yaw;
    state.camera_pitch = params.camera.pitch;
    state.camera_distance = params.camera.distance;
    state.camera_pitch_min = params.camera.pitch_min;
    state.camera_pitch_max = params.camera.pitch_max;
    state.camera_distance_min = params.camera.distance_min;
    state.camera_distance_max = params.camera.distance_max;
    state.camera_fov_y = params.camera.fov_y_radians;
    state.world_paused = params.world_paused;
    state.debug_view_mode = params.debug_view_mode;
    state.elapsed_time_seconds = 0.0;
    state.agents.clear();
}

pub(super) fn apply_input_bindings(input: &mut InputState, config: &SdfInputBindingsConfig) -> usize {
    let mut applied = 0usize;
    for (index, binding) in config.bindings.iter().enumerate() {
        let action = binding.action.trim();
        if action.is_empty() {
            tracing::error!(
                index,
                key = binding.key.as_str(),
                "sdf input binding has empty action; skipping"
            );
            continue;
        }
        let Some(key) = parse_key_code(binding.key.as_str()) else {
            tracing::error!(
                index,
                action,
                key = binding.key.as_str(),
                "invalid sdf input key code; skipping binding"
            );
            continue;
        };
        input.map_key(action.to_string(), key);
        applied = applied.saturating_add(1);
    }
    applied
}

pub(super) fn parse_key_code(raw: &str) -> Option<KeyCode> {
    let token = raw.trim();
    if token.is_empty() {
        return None;
    }
    let normalized = token.to_ascii_lowercase();
    match normalized.as_str() {
        "arrowleft" | "left" => Some(KeyCode::ArrowLeft),
        "arrowright" | "right" => Some(KeyCode::ArrowRight),
        "arrowup" | "up" => Some(KeyCode::ArrowUp),
        "arrowdown" | "down" => Some(KeyCode::ArrowDown),
        "tab" => Some(KeyCode::Tab),
        "backquote" | "backtick" | "`" => Some(KeyCode::Backquote),
        "escape" | "esc" => Some(KeyCode::Escape),
        "space" => Some(KeyCode::Space),
        "enter" => Some(KeyCode::Enter),
        "numpadenter" => Some(KeyCode::NumpadEnter),
        "shiftleft" => Some(KeyCode::ShiftLeft),
        "shiftright" => Some(KeyCode::ShiftRight),
        "controlleft" => Some(KeyCode::ControlLeft),
        "controlright" => Some(KeyCode::ControlRight),
        "altleft" => Some(KeyCode::AltLeft),
        "altright" => Some(KeyCode::AltRight),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "delete" => Some(KeyCode::Delete),
        "backspace" => Some(KeyCode::Backspace),
        _ => parse_key_code_compact(token),
    }
}

fn parse_key_code_compact(token: &str) -> Option<KeyCode> {
    if let Some(rest) = token
        .strip_prefix("Key")
        .or_else(|| token.strip_prefix("key"))
        && rest.len() == 1
    {
        return parse_letter_key(rest.chars().next().expect("checked len"));
    }
    if let Some(rest) = token
        .strip_prefix("Digit")
        .or_else(|| token.strip_prefix("digit"))
        && rest.len() == 1
    {
        return parse_digit_key(rest.chars().next().expect("checked len"));
    }
    if token.len() == 1 {
        let ch = token.chars().next().expect("checked len");
        return parse_letter_key(ch).or_else(|| parse_digit_key(ch));
    }
    if let Some(rest) = token.strip_prefix('F').or_else(|| token.strip_prefix('f')) {
        return parse_function_key(rest);
    }
    None
}

fn parse_letter_key(ch: char) -> Option<KeyCode> {
    match ch.to_ascii_uppercase() {
        'A' => Some(KeyCode::KeyA),
        'B' => Some(KeyCode::KeyB),
        'C' => Some(KeyCode::KeyC),
        'D' => Some(KeyCode::KeyD),
        'E' => Some(KeyCode::KeyE),
        'F' => Some(KeyCode::KeyF),
        'G' => Some(KeyCode::KeyG),
        'H' => Some(KeyCode::KeyH),
        'I' => Some(KeyCode::KeyI),
        'J' => Some(KeyCode::KeyJ),
        'K' => Some(KeyCode::KeyK),
        'L' => Some(KeyCode::KeyL),
        'M' => Some(KeyCode::KeyM),
        'N' => Some(KeyCode::KeyN),
        'O' => Some(KeyCode::KeyO),
        'P' => Some(KeyCode::KeyP),
        'Q' => Some(KeyCode::KeyQ),
        'R' => Some(KeyCode::KeyR),
        'S' => Some(KeyCode::KeyS),
        'T' => Some(KeyCode::KeyT),
        'U' => Some(KeyCode::KeyU),
        'V' => Some(KeyCode::KeyV),
        'W' => Some(KeyCode::KeyW),
        'X' => Some(KeyCode::KeyX),
        'Y' => Some(KeyCode::KeyY),
        'Z' => Some(KeyCode::KeyZ),
        _ => None,
    }
}

fn parse_digit_key(ch: char) -> Option<KeyCode> {
    match ch {
        '0' => Some(KeyCode::Digit0),
        '1' => Some(KeyCode::Digit1),
        '2' => Some(KeyCode::Digit2),
        '3' => Some(KeyCode::Digit3),
        '4' => Some(KeyCode::Digit4),
        '5' => Some(KeyCode::Digit5),
        '6' => Some(KeyCode::Digit6),
        '7' => Some(KeyCode::Digit7),
        '8' => Some(KeyCode::Digit8),
        '9' => Some(KeyCode::Digit9),
        _ => None,
    }
}

fn parse_function_key(suffix: &str) -> Option<KeyCode> {
    match suffix {
        "1" => Some(KeyCode::F1),
        "2" => Some(KeyCode::F2),
        "3" => Some(KeyCode::F3),
        "4" => Some(KeyCode::F4),
        "5" => Some(KeyCode::F5),
        "6" => Some(KeyCode::F6),
        "7" => Some(KeyCode::F7),
        "8" => Some(KeyCode::F8),
        "9" => Some(KeyCode::F9),
        "10" => Some(KeyCode::F10),
        "11" => Some(KeyCode::F11),
        "12" => Some(KeyCode::F12),
        _ => None,
    }
}

pub(super) fn parse_builtin_executor(raw: &str) -> Option<BuiltinRenderPassExecutor> {
    BuiltinRenderPassExecutor::from_label(raw)
}

pub(super) fn load_config_with_default<T>(file_name: &str) -> T
where
    T: DeserializeOwned + Default,
{
    let config_path = find_config_path(file_name);
    let raw = match fs::read_to_string(&config_path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                config = file_name,
                path = config_path.display().to_string(),
                "sdf config file missing; using built-in defaults"
            );
            return T::default();
        }
        Err(err) => {
            tracing::error!(
                config = file_name,
                path = config_path.display().to_string(),
                ?err,
                "sdf config file read failed; using built-in defaults"
            );
            return T::default();
        }
    };

    match ron::from_str::<T>(&raw) {
        Ok(parsed) => parsed,
        Err(err) => {
            tracing::error!(
                config = file_name,
                path = config_path.display().to_string(),
                ?err,
                "sdf config parse failed; using built-in defaults"
            );
            T::default()
        }
    }
}

pub(super) fn find_config_path(file_name: &str) -> PathBuf {
    let primary = Path::new(SDF_ASSETS_DIR_PRIMARY).join(file_name);
    if primary.exists() {
        return primary;
    }
    let fallback = Path::new(SDF_ASSETS_DIR_FALLBACK).join(file_name);
    if fallback.exists() {
        return fallback;
    }
    primary
}
