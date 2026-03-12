// Owner: SDF Renderer Example - Input Binding Runtime Helpers
use crate::*;

pub(crate) fn app_input_bindings(config: &SdfInputBindingsConfig) -> Vec<(&'static str, KeyCode)> {
    let mut bindings = Vec::<(&'static str, KeyCode)>::new();
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
        let Some(action) = known_action_name(action) else {
            tracing::error!(
                index,
                action,
                key = binding.key.as_str(),
                "unknown sdf input action; expected one of the built-in sdf action names"
            );
            continue;
        };
        let Some(key) = parse_key_code(binding.key.as_str()) else {
            tracing::error!(
                index,
                action,
                key = binding.key.as_str(),
                "invalid sdf input key code; skipping binding"
            );
            continue;
        };
        bindings.push((action, key));
    }
    bindings
}

fn known_action_name(action: &str) -> Option<&'static str> {
    match action {
        ACTION_UP => Some(ACTION_UP),
        ACTION_DOWN => Some(ACTION_DOWN),
        ACTION_DEBUG_NEXT => Some(ACTION_DEBUG_NEXT),
        ACTION_DEBUG_PREV => Some(ACTION_DEBUG_PREV),
        ACTION_SPEED_UP => Some(ACTION_SPEED_UP),
        ACTION_SPEED_DOWN => Some(ACTION_SPEED_DOWN),
        _ => None,
    }
}

pub(crate) fn parse_key_code(raw: &str) -> Option<KeyCode> {
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
