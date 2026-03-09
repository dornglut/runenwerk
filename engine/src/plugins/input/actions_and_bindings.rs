use std::collections::{HashMap, HashSet};
use winit::keyboard::KeyCode;
use crate::plugins::ModifiersSnapshot;

// Owner: Engine Input Plugin - Action Bindings and Chords
pub mod action {
    pub const UI_SUBMIT: &str = "ui.submit";
    pub const UI_INSERT_NEWLINE: &str = "ui.insert_newline";
    pub const UI_BACKSPACE: &str = "ui.backspace";
    pub const UI_DELETE: &str = "ui.delete";
    pub const UI_MOVE_LEFT: &str = "ui.move_left";
    pub const UI_MOVE_RIGHT: &str = "ui.move_right";
    pub const UI_MOVE_UP: &str = "ui.move_up";
    pub const UI_MOVE_DOWN: &str = "ui.move_down";
    pub const UI_MOVE_HOME: &str = "ui.move_home";
    pub const UI_MOVE_END: &str = "ui.move_end";
    pub const UI_PAGE_UP: &str = "ui.page_up";
    pub const UI_PAGE_DOWN: &str = "ui.page_down";
    pub const WORLD_MOVE_LEFT: &str = "world.move_left";
    pub const WORLD_MOVE_RIGHT: &str = "world.move_right";
    pub const WORLD_MOVE_UP: &str = "world.move_up";
    pub const WORLD_MOVE_DOWN: &str = "world.move_down";
    pub const SYSTEM_TOGGLE_PAUSE_MENU: &str = "system.toggle_pause_menu";
    pub const UI_TOGGLE_EDITOR_MODE: &str = "ui.toggle_editor_mode";
    pub const UI_SAVE_TEMPLATE: &str = "ui.save_template";
    pub const UI_EDITOR_HIDE_SELECTED: &str = "ui.editor_hide_selected";
    pub const UI_EDITOR_RESTORE_ALL: &str = "ui.editor_restore_all";
    pub const SCENE_NEXT: &str = "scene.next";
    pub const SCENE_PREV: &str = "scene.prev";
    pub const SCENE_CONSOLE: &str = "scene.console";
    pub const SCENE_HUD: &str = "scene.hud";
    pub const SCENE_OVERLAY_PUSH: &str = "scene.overlay_push";
    pub const SCENE_OVERLAY_POP: &str = "scene.overlay_pop";
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ModifierRule {
    Ignore,
    Required,
    Forbidden,
}

impl ModifierRule {
    fn matches(self, is_down: bool) -> bool {
        match self {
            Self::Ignore => true,
            Self::Required => is_down,
            Self::Forbidden => !is_down,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct KeyChord {
    pub key: KeyCode,
    pub shift: ModifierRule,
    pub ctrl: ModifierRule,
    pub alt: ModifierRule,
    pub super_key: ModifierRule,
}

impl KeyChord {
    pub const fn new(key: KeyCode) -> Self {
        Self {
            key,
            shift: ModifierRule::Ignore,
            ctrl: ModifierRule::Ignore,
            alt: ModifierRule::Ignore,
            super_key: ModifierRule::Ignore,
        }
    }

    pub const fn with_shift_required(mut self) -> Self {
        self.shift = ModifierRule::Required;
        self
    }

    pub const fn with_shift_forbidden(mut self) -> Self {
        self.shift = ModifierRule::Forbidden;
        self
    }

    pub const fn with_ctrl_required(mut self) -> Self {
        self.ctrl = ModifierRule::Required;
        self
    }

    pub const fn with_super_required(mut self) -> Self {
        self.super_key = ModifierRule::Required;
        self
    }

    fn matches(&self, modifiers: ModifiersSnapshot) -> bool {
        self.shift.matches(modifiers.shift)
            && self.ctrl.matches(modifiers.ctrl)
            && self.alt.matches(modifiers.alt)
            && self.super_key.matches(modifiers.super_key)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InputBindings {
    by_action: HashMap<String, Vec<KeyChord>>,
}

impl InputBindings {
    pub fn with_default_bindings() -> Self {
        let mut bindings = Self::default();
        bindings.install_default_bindings();
        bindings
    }

    pub fn map_key(&mut self, action: impl Into<String>, key: KeyCode) -> bool {
        self.map_chord(action, KeyChord::new(key))
    }

    pub fn map_chord(&mut self, action: impl Into<String>, chord: KeyChord) -> bool {
        let action = action.into();
        let bindings = self.by_action.entry(action).or_default();
        if !bindings.contains(&chord) {
            bindings.push(chord);
            return true;
        }
        false
    }

    pub fn unmap_key(&mut self, action: &str, key: KeyCode) -> usize {
        let Some(bindings) = self.by_action.get_mut(action) else {
            return 0;
        };
        let before = bindings.len();
        bindings.retain(|chord| chord.key != key);
        let removed = before.saturating_sub(bindings.len());
        if bindings.is_empty() {
            self.by_action.remove(action);
        }
        removed
    }

    pub fn unmap_chord(&mut self, action: &str, chord: KeyChord) -> bool {
        let Some(bindings) = self.by_action.get_mut(action) else {
            return false;
        };
        let before = bindings.len();
        bindings.retain(|existing| *existing != chord);
        let removed = before != bindings.len();
        if bindings.is_empty() {
            self.by_action.remove(action);
        }
        removed
    }

    pub fn clear_action(&mut self, action: &str) -> bool {
        self.by_action.remove(action).is_some()
    }

    pub fn clear_all(&mut self) {
        self.by_action.clear();
    }

    pub fn chords_for_action(&self, action: &str) -> Option<&[KeyChord]> {
        self.by_action.get(action).map(Vec::as_slice)
    }

    pub fn actions(&self) -> impl Iterator<Item = &str> {
        self.by_action.keys().map(String::as_str)
    }

    pub(crate) fn matching_actions_for_key(&self, key: KeyCode, modifiers: ModifiersSnapshot) -> Vec<String> {
        let mut actions = Vec::new();
        for (action, chords) in &self.by_action {
            if chords
                .iter()
                .any(|chord| chord.key == key && chord.matches(modifiers))
            {
                actions.push(action.clone());
            }
        }
        actions
    }

    pub(crate) fn action_down(
        &self,
        action: &str,
        keys_down: &HashSet<KeyCode>,
        modifiers: ModifiersSnapshot,
    ) -> bool {
        self.by_action.get(action).is_some_and(|chords| {
            chords
                .iter()
                .any(|chord| keys_down.contains(&chord.key) && chord.matches(modifiers))
        })
    }

    pub(crate) fn action_ids(&self) -> impl Iterator<Item = &String> {
        self.by_action.keys()
    }

    fn install_default_bindings(&mut self) {
        self.map_chord(
            action::UI_SUBMIT,
            KeyChord::new(KeyCode::Enter).with_shift_forbidden(),
        );
        self.map_chord(
            action::UI_SUBMIT,
            KeyChord::new(KeyCode::NumpadEnter).with_shift_forbidden(),
        );
        self.map_chord(
            action::UI_INSERT_NEWLINE,
            KeyChord::new(KeyCode::Enter).with_shift_required(),
        );
        self.map_chord(
            action::UI_INSERT_NEWLINE,
            KeyChord::new(KeyCode::NumpadEnter).with_shift_required(),
        );
        self.map_key(action::UI_BACKSPACE, KeyCode::Backspace);
        self.map_key(action::UI_DELETE, KeyCode::Delete);
        self.map_key(action::UI_MOVE_LEFT, KeyCode::ArrowLeft);
        self.map_key(action::UI_MOVE_RIGHT, KeyCode::ArrowRight);
        self.map_key(action::UI_MOVE_UP, KeyCode::ArrowUp);
        self.map_key(action::UI_MOVE_DOWN, KeyCode::ArrowDown);
        self.map_key(action::UI_MOVE_HOME, KeyCode::Home);
        self.map_key(action::UI_MOVE_END, KeyCode::End);
        self.map_key(action::UI_PAGE_UP, KeyCode::PageUp);
        self.map_key(action::UI_PAGE_DOWN, KeyCode::PageDown);
        self.map_key(action::WORLD_MOVE_LEFT, KeyCode::KeyA);
        self.map_key(action::WORLD_MOVE_RIGHT, KeyCode::KeyD);
        self.map_key(action::WORLD_MOVE_UP, KeyCode::KeyW);
        self.map_key(action::WORLD_MOVE_DOWN, KeyCode::KeyS);
        self.map_key(action::SYSTEM_TOGGLE_PAUSE_MENU, KeyCode::Escape);
        self.map_key(action::UI_TOGGLE_EDITOR_MODE, KeyCode::F1);
        self.map_chord(
            action::UI_SAVE_TEMPLATE,
            KeyChord::new(KeyCode::KeyS).with_ctrl_required(),
        );
        self.map_chord(
            action::UI_SAVE_TEMPLATE,
            KeyChord::new(KeyCode::KeyS).with_super_required(),
        );
        self.map_key(action::UI_EDITOR_HIDE_SELECTED, KeyCode::KeyX);
        self.map_key(action::UI_EDITOR_RESTORE_ALL, KeyCode::KeyA);
        self.map_chord(
            action::SCENE_NEXT,
            KeyChord::new(KeyCode::F2).with_shift_forbidden(),
        );
        self.map_chord(
            action::SCENE_PREV,
            KeyChord::new(KeyCode::F2).with_shift_required(),
        );
        self.map_key(action::SCENE_CONSOLE, KeyCode::F3);
        self.map_key(action::SCENE_HUD, KeyCode::F4);
        self.map_chord(
            action::SCENE_OVERLAY_PUSH,
            KeyChord::new(KeyCode::F5).with_shift_forbidden(),
        );
        self.map_chord(
            action::SCENE_OVERLAY_POP,
            KeyChord::new(KeyCode::F5).with_shift_required(),
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InputBindingChange {
    MapKey { action: String, key: KeyCode },
    MapChord { action: String, chord: KeyChord },
    UnmapKey { action: String, key: KeyCode },
    UnmapChord { action: String, chord: KeyChord },
    ClearAction { action: String },
    ResetDefaults,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputBindingChangeResult {
    Applied,
    Noop,
}
