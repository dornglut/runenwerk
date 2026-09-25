//! File: apps/runenwerk_editor/src/shell/shortcut_resolution.rs
//! Purpose: App-owned resolution of authored editor shortcuts to engine input chords.

use editor_definition::{EditorShortcutDefinition, EditorShortcutSetDefinition};
use engine::plugins::{KeyChord, ModifierRule};
use runen_input::PhysicalKeyIdentity;
use ui_definition::UiDefinitionDiagnostic;

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
                    Some(parse_physical_key(part).ok_or_else(|| {
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

fn parse_physical_key(part: &str) -> Option<PhysicalKeyIdentity> {
    let normalized = part.trim().to_ascii_lowercase();
    let code = if normalized.len() == 1 {
        let byte = normalized.as_bytes()[0];
        if byte.is_ascii_lowercase() {
            Some(format!("Key{}", (byte as char).to_ascii_uppercase()))
        } else if byte.is_ascii_digit() {
            Some(format!("Digit{}", byte as char))
        } else {
            None
        }
    } else {
        match normalized.as_str() {
            "escape" | "esc" => Some("Escape".to_string()),
            "tab" => Some("Tab".to_string()),
            "enter" | "return" => Some("Enter".to_string()),
            "backspace" => Some("Backspace".to_string()),
            "delete" | "del" => Some("Delete".to_string()),
            "space" => Some("Space".to_string()),
            "home" => Some("Home".to_string()),
            "end" => Some("End".to_string()),
            "pageup" | "page_up" | "page-up" => Some("PageUp".to_string()),
            "pagedown" | "page_down" | "page-down" => Some("PageDown".to_string()),
            "arrowleft" | "arrow_left" | "arrow-left" | "left" => Some("ArrowLeft".to_string()),
            "arrowright" | "arrow_right" | "arrow-right" | "right" => {
                Some("ArrowRight".to_string())
            }
            "arrowup" | "arrow_up" | "arrow-up" | "up" => Some("ArrowUp".to_string()),
            "arrowdown" | "arrow_down" | "arrow-down" | "down" => Some("ArrowDown".to_string()),
            "f1" => Some("F1".to_string()),
            "f2" => Some("F2".to_string()),
            "f3" => Some("F3".to_string()),
            "f4" => Some("F4".to_string()),
            "f5" => Some("F5".to_string()),
            "f6" => Some("F6".to_string()),
            "f7" => Some("F7".to_string()),
            "f8" => Some("F8".to_string()),
            "f9" => Some("F9".to_string()),
            "f10" => Some("F10".to_string()),
            "f11" => Some("F11".to_string()),
            "f12" => Some("F12".to_string()),
            _ => None,
        }
    }?;
    Some(PhysicalKeyIdentity::code(code))
}
