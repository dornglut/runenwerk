//! File: apps/runenwerk_editor/src/shell/shortcut_resolution.rs
//! Purpose: App-owned resolution of authored editor shortcuts to engine input chords.

use editor_definition::{EditorShortcutDefinition, EditorShortcutSetDefinition};
use engine::plugins::{KeyChord, ModifierRule};
use ui_definition::UiDefinitionDiagnostic;
use winit::keyboard::KeyCode;

use crate::shell::{ActiveEditorDefinitionCatalogs, KnownEditorCommand};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedEditorShortcut {
    pub action_id: String,
    pub command: KnownEditorCommand,
    pub command_key: String,
    pub chord_text: String,
    pub chord: KeyChord,
}

pub fn validate_editor_shortcuts(
    shortcuts: &EditorShortcutSetDefinition,
) -> Vec<UiDefinitionDiagnostic> {
    let mut diagnostics = Vec::new();
    for shortcut in &shortcuts.shortcuts {
        if KnownEditorCommand::from_key(&shortcut.command).is_none() {
            diagnostics.push(UiDefinitionDiagnostic::error(
                "editor.definition.shortcut.command.unknown",
                format!(
                    "shortcut '{}' references unknown editor command '{}'",
                    shortcut.id, shortcut.command
                ),
            ));
        }
        if let Err(message) = parse_editor_shortcut_chord(&shortcut.chord) {
            diagnostics.push(UiDefinitionDiagnostic::error(
                "editor.definition.shortcut.chord.unsupported",
                format!(
                    "shortcut '{}' has unsupported chord: {message}",
                    shortcut.id
                ),
            ));
        }
    }
    diagnostics
}

pub fn resolve_active_editor_shortcuts(
    catalogs: &ActiveEditorDefinitionCatalogs,
) -> Result<Vec<ResolvedEditorShortcut>, Vec<UiDefinitionDiagnostic>> {
    let mut resolved = Vec::new();
    let mut diagnostics = Vec::new();
    for shortcut_set in catalogs.shortcuts().values() {
        for shortcut in &shortcut_set.shortcuts {
            match resolve_shortcut(shortcut_set, shortcut) {
                Ok(binding) => resolved.push(binding),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
    }
    if diagnostics.is_empty() {
        Ok(resolved)
    } else {
        Err(diagnostics)
    }
}

fn resolve_shortcut(
    shortcut_set: &EditorShortcutSetDefinition,
    shortcut: &EditorShortcutDefinition,
) -> Result<ResolvedEditorShortcut, UiDefinitionDiagnostic> {
    let Some(command) = KnownEditorCommand::from_key(&shortcut.command) else {
        return Err(UiDefinitionDiagnostic::error(
            "editor.definition.shortcut.command.unknown",
            format!(
                "shortcut '{}' references unknown editor command '{}'",
                shortcut.id, shortcut.command
            ),
        ));
    };
    let chord = parse_editor_shortcut_chord(&shortcut.chord).map_err(|message| {
        UiDefinitionDiagnostic::error(
            "editor.definition.shortcut.chord.unsupported",
            format!(
                "shortcut '{}' has unsupported chord: {message}",
                shortcut.id
            ),
        )
    })?;
    Ok(ResolvedEditorShortcut {
        action_id: active_shortcut_action_id(&shortcut_set.id, &shortcut.id),
        command,
        command_key: shortcut.command.clone(),
        chord_text: shortcut.chord.clone(),
        chord,
    })
}

pub fn active_shortcut_action_id(shortcut_set_id: &str, shortcut_id: &str) -> String {
    format!("editor.shortcut.{shortcut_set_id}.{shortcut_id}")
}

pub fn parse_editor_shortcut_chord(chord: &str) -> Result<KeyChord, String> {
    let mut key = None;
    let mut shift = ModifierRule::Forbidden;
    let mut ctrl = ModifierRule::Forbidden;
    let mut alt = ModifierRule::Forbidden;
    let mut super_key = ModifierRule::Forbidden;
    for raw_part in chord.split('+') {
        let part = raw_part.trim();
        if part.is_empty() {
            return Err(format!("'{chord}' contains an empty chord segment"));
        }
        match normalized_modifier(part) {
            Some("shift") => shift = ModifierRule::Required,
            Some("ctrl") => ctrl = ModifierRule::Required,
            Some("alt") => alt = ModifierRule::Required,
            Some("super") => super_key = ModifierRule::Required,
            Some(_) => unreachable!("normalized_modifier returns only known values"),
            None => {
                if key.is_some() {
                    return Err(format!("'{chord}' contains more than one key token"));
                }
                key =
                    Some(parse_key_code(part).ok_or_else(|| {
                        format!("'{part}' is not a supported editor shortcut key")
                    })?);
            }
        }
    }
    let Some(key) = key else {
        return Err(format!("'{chord}' does not contain a key token"));
    };
    Ok(KeyChord {
        key,
        shift,
        ctrl,
        alt,
        super_key,
    })
}

fn normalized_modifier(part: &str) -> Option<&'static str> {
    match part.to_ascii_lowercase().as_str() {
        "shift" => Some("shift"),
        "ctrl" | "control" => Some("ctrl"),
        "alt" | "option" => Some("alt"),
        "cmd" | "command" | "meta" | "super" => Some("super"),
        _ => None,
    }
}

fn parse_key_code(part: &str) -> Option<KeyCode> {
    let normalized = part.trim().to_ascii_lowercase();
    if normalized.len() == 1 {
        let byte = normalized.as_bytes()[0];
        if byte.is_ascii_lowercase() {
            return Some(match byte {
                b'a' => KeyCode::KeyA,
                b'b' => KeyCode::KeyB,
                b'c' => KeyCode::KeyC,
                b'd' => KeyCode::KeyD,
                b'e' => KeyCode::KeyE,
                b'f' => KeyCode::KeyF,
                b'g' => KeyCode::KeyG,
                b'h' => KeyCode::KeyH,
                b'i' => KeyCode::KeyI,
                b'j' => KeyCode::KeyJ,
                b'k' => KeyCode::KeyK,
                b'l' => KeyCode::KeyL,
                b'm' => KeyCode::KeyM,
                b'n' => KeyCode::KeyN,
                b'o' => KeyCode::KeyO,
                b'p' => KeyCode::KeyP,
                b'q' => KeyCode::KeyQ,
                b'r' => KeyCode::KeyR,
                b's' => KeyCode::KeyS,
                b't' => KeyCode::KeyT,
                b'u' => KeyCode::KeyU,
                b'v' => KeyCode::KeyV,
                b'w' => KeyCode::KeyW,
                b'x' => KeyCode::KeyX,
                b'y' => KeyCode::KeyY,
                b'z' => KeyCode::KeyZ,
                _ => return None,
            });
        }
        if byte.is_ascii_digit() {
            return Some(match byte {
                b'0' => KeyCode::Digit0,
                b'1' => KeyCode::Digit1,
                b'2' => KeyCode::Digit2,
                b'3' => KeyCode::Digit3,
                b'4' => KeyCode::Digit4,
                b'5' => KeyCode::Digit5,
                b'6' => KeyCode::Digit6,
                b'7' => KeyCode::Digit7,
                b'8' => KeyCode::Digit8,
                b'9' => KeyCode::Digit9,
                _ => return None,
            });
        }
    }
    match normalized.as_str() {
        "escape" | "esc" => Some(KeyCode::Escape),
        "tab" => Some(KeyCode::Tab),
        "enter" | "return" => Some(KeyCode::Enter),
        "backspace" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "space" => Some(KeyCode::Space),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "page_up" | "page-up" => Some(KeyCode::PageUp),
        "pagedown" | "page_down" | "page-down" => Some(KeyCode::PageDown),
        "arrowleft" | "arrow_left" | "arrow-left" | "left" => Some(KeyCode::ArrowLeft),
        "arrowright" | "arrow_right" | "arrow-right" | "right" => Some(KeyCode::ArrowRight),
        "arrowup" | "arrow_up" | "arrow-up" | "up" => Some(KeyCode::ArrowUp),
        "arrowdown" | "arrow_down" | "arrow-down" | "down" => Some(KeyCode::ArrowDown),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f4" => Some(KeyCode::F4),
        "f5" => Some(KeyCode::F5),
        "f6" => Some(KeyCode::F6),
        "f7" => Some(KeyCode::F7),
        "f8" => Some(KeyCode::F8),
        "f9" => Some(KeyCode::F9),
        "f10" => Some(KeyCode::F10),
        "f11" => Some(KeyCode::F11),
        "f12" => Some(KeyCode::F12),
        _ => None,
    }
}
