use runen_input::PhysicalKeyIdentity;
use super::state::{InputState, ModifiersSnapshot};
use std::collections::{HashMap, HashSet};

// Owner: Engine Input Plugin - Runenwerk Product Action Bindings and Projection
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyChord {
    pub key: PhysicalKeyIdentity,
    pub shift: ModifierRule,
    pub ctrl: ModifierRule,
    pub alt: ModifierRule,
    pub super_key: ModifierRule,
}

impl KeyChord {
    pub fn new(key: PhysicalKeyIdentity) -> Self {
        Self {
            key,
            shift: ModifierRule::Ignore,
            ctrl: ModifierRule::Ignore,
            alt: ModifierRule::Ignore,
            super_key: ModifierRule::Ignore,
        }
    }

    pub fn code(code: impl Into<String>) -> Self {
        Self::new(PhysicalKeyIdentity::code(code))
    }

    pub fn with_shift_required(mut self) -> Self {
        self.shift = ModifierRule::Required;
        self
    }

    pub fn with_shift_forbidden(mut self) -> Self {
        self.shift = ModifierRule::Forbidden;
        self
    }

    pub fn with_ctrl_required(mut self) -> Self {
        self.ctrl = ModifierRule::Required;
        self
    }

    pub fn with_super_required(mut self) -> Self {
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

    pub fn map_key(&mut self, action: impl Into<String>, key: PhysicalKeyIdentity) -> bool {
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

    pub fn unmap_key(&mut self, action: &str, key: &PhysicalKeyIdentity) -> usize {
        let Some(bindings) = self.by_action.get_mut(action) else {
            return 0;
        };
        let before = bindings.len();
        bindings.retain(|chord| &chord.key != key);
        let removed = before.saturating_sub(bindings.len());
        if bindings.is_empty() {
            self.by_action.remove(action);
        }
        removed
    }

    pub fn unmap_chord(&mut self, action: &str, chord: &KeyChord) -> bool {
        let Some(bindings) = self.by_action.get_mut(action) else {
            return false;
        };
        let before = bindings.len();
        bindings.retain(|existing| existing != chord);
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

    fn matching_actions(
        &self,
        key: &PhysicalKeyIdentity,
        modifiers: ModifiersSnapshot,
    ) -> Vec<String> {
        let mut actions = Vec::new();
        for (action, chords) in &self.by_action {
            if chords
                .iter()
                .any(|chord| chord.key == *key && chord.matches(modifiers))
            {
                actions.push(action.clone());
            }
        }
        actions
    }

    fn action_down<F>(&self, action: &str, mut key_is_down: F, modifiers: ModifiersSnapshot) -> bool
    where
        F: FnMut(&PhysicalKeyIdentity) -> bool,
    {
        self.by_action.get(action).is_some_and(|chords| {
            chords
                .iter()
                .any(|chord| key_is_down(&chord.key) && chord.matches(modifiers))
        })
    }

    fn action_ids(&self) -> impl Iterator<Item = &String> {
        self.by_action.keys()
    }

    fn install_default_bindings(&mut self) {
        self.map_chord(
            action::UI_SUBMIT,
            KeyChord::code("Enter").with_shift_forbidden(),
        );
        self.map_chord(
            action::UI_SUBMIT,
            KeyChord::code("NumpadEnter").with_shift_forbidden(),
        );
        self.map_chord(
            action::UI_INSERT_NEWLINE,
            KeyChord::code("Enter").with_shift_required(),
        );
        self.map_chord(
            action::UI_INSERT_NEWLINE,
            KeyChord::code("NumpadEnter").with_shift_required(),
        );
        self.map_key(action::UI_BACKSPACE, PhysicalKeyIdentity::code("Backspace"));
        self.map_key(action::UI_DELETE, PhysicalKeyIdentity::code("Delete"));
        self.map_key(action::UI_MOVE_LEFT, PhysicalKeyIdentity::code("ArrowLeft"));
        self.map_key(
            action::UI_MOVE_RIGHT,
            PhysicalKeyIdentity::code("ArrowRight"),
        );
        self.map_key(action::UI_MOVE_UP, PhysicalKeyIdentity::code("ArrowUp"));
        self.map_key(action::UI_MOVE_DOWN, PhysicalKeyIdentity::code("ArrowDown"));
        self.map_key(action::UI_MOVE_HOME, PhysicalKeyIdentity::code("Home"));
        self.map_key(action::UI_MOVE_END, PhysicalKeyIdentity::code("End"));
        self.map_key(action::UI_PAGE_UP, PhysicalKeyIdentity::code("PageUp"));
        self.map_key(action::UI_PAGE_DOWN, PhysicalKeyIdentity::code("PageDown"));
        self.map_key(action::WORLD_MOVE_LEFT, PhysicalKeyIdentity::code("KeyA"));
        self.map_key(action::WORLD_MOVE_RIGHT, PhysicalKeyIdentity::code("KeyD"));
        self.map_key(action::WORLD_MOVE_UP, PhysicalKeyIdentity::code("KeyW"));
        self.map_key(action::WORLD_MOVE_DOWN, PhysicalKeyIdentity::code("KeyS"));
        self.map_key(
            action::SYSTEM_TOGGLE_PAUSE_MENU,
            PhysicalKeyIdentity::code("Escape"),
        );
        self.map_key(
            action::UI_TOGGLE_EDITOR_MODE,
            PhysicalKeyIdentity::code("F1"),
        );
        self.map_chord(
            action::UI_SAVE_TEMPLATE,
            KeyChord::code("KeyS").with_ctrl_required(),
        );
        self.map_chord(
            action::UI_SAVE_TEMPLATE,
            KeyChord::code("KeyS").with_super_required(),
        );
        self.map_key(
            action::UI_EDITOR_HIDE_SELECTED,
            PhysicalKeyIdentity::code("KeyX"),
        );
        self.map_key(
            action::UI_EDITOR_RESTORE_ALL,
            PhysicalKeyIdentity::code("KeyA"),
        );
        self.map_chord(
            action::SCENE_NEXT,
            KeyChord::code("F2").with_shift_forbidden(),
        );
        self.map_chord(
            action::SCENE_PREV,
            KeyChord::code("F2").with_shift_required(),
        );
        self.map_key(action::SCENE_CONSOLE, PhysicalKeyIdentity::code("F3"));
        self.map_key(action::SCENE_HUD, PhysicalKeyIdentity::code("F4"));
        self.map_chord(
            action::SCENE_OVERLAY_PUSH,
            KeyChord::code("F5").with_shift_forbidden(),
        );
        self.map_chord(
            action::SCENE_OVERLAY_POP,
            KeyChord::code("F5").with_shift_required(),
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InputBindingChange {
    MapKey {
        action: String,
        key: PhysicalKeyIdentity,
    },
    MapChord {
        action: String,
        chord: KeyChord,
    },
    UnmapKey {
        action: String,
        key: PhysicalKeyIdentity,
    },
    UnmapChord {
        action: String,
        chord: KeyChord,
    },
    ClearAction {
        action: String,
    },
    ResetDefaults,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputBindingChangeResult {
    Applied,
    Noop,
}

#[derive(Debug, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct ActionState {
    bindings: InputBindings,
    actions_down: HashSet<String>,
    actions_pressed: HashSet<String>,
    processed_keyboard_presses: usize,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            bindings: InputBindings::with_default_bindings(),
            actions_down: HashSet::new(),
            actions_pressed: HashSet::new(),
            processed_keyboard_presses: 0,
        }
    }
}

impl ActionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bindings(&self) -> &InputBindings {
        &self.bindings
    }

    pub fn set_bindings(&mut self, input: &InputState, bindings: InputBindings) {
        self.project(input);
        self.bindings = bindings;
        self.actions_pressed.clear();
        self.recompute_action_down_states(input);
    }

    pub fn reset_default_bindings(&mut self, input: &InputState) {
        self.set_bindings(input, InputBindings::with_default_bindings());
    }

    pub fn map_key(
        &mut self,
        input: &InputState,
        action: impl Into<String>,
        key: PhysicalKeyIdentity,
    ) {
        self.project(input);
        if self.bindings.map_key(action, key) {
            self.recompute_action_down_states(input);
        }
    }

    pub fn map_chord(&mut self, input: &InputState, action: impl Into<String>, chord: KeyChord) {
        self.project(input);
        if self.bindings.map_chord(action, chord) {
            self.recompute_action_down_states(input);
        }
    }

    pub fn unmap_key(
        &mut self,
        input: &InputState,
        action: &str,
        key: &PhysicalKeyIdentity,
    ) -> usize {
        self.project(input);
        let removed = self.bindings.unmap_key(action, key);
        if removed > 0 {
            self.recompute_action_down_states(input);
        }
        removed
    }

    pub fn unmap_chord(&mut self, input: &InputState, action: &str, chord: &KeyChord) -> bool {
        self.project(input);
        let removed = self.bindings.unmap_chord(action, chord);
        if removed {
            self.recompute_action_down_states(input);
        }
        removed
    }

    pub fn clear_action_bindings(&mut self, input: &InputState, action: &str) -> bool {
        self.project(input);
        let removed = self.bindings.clear_action(action);
        if removed {
            self.recompute_action_down_states(input);
        }
        removed
    }

    pub fn apply_binding_change(
        &mut self,
        input: &InputState,
        change: InputBindingChange,
    ) -> InputBindingChangeResult {
        self.project(input);
        if self.apply_binding_change_inner(change) {
            self.recompute_action_down_states(input);
            InputBindingChangeResult::Applied
        } else {
            InputBindingChangeResult::Noop
        }
    }

    pub fn apply_binding_changes<I>(&mut self, input: &InputState, changes: I) -> usize
    where
        I: IntoIterator<Item = InputBindingChange>,
    {
        self.project(input);
        let mut applied = 0usize;
        for change in changes {
            if self.apply_binding_change_inner(change) {
                applied = applied.saturating_add(1);
            }
        }
        if applied > 0 {
            self.recompute_action_down_states(input);
        }
        applied
    }

    pub fn action_down(&self, action: &str) -> bool {
        self.actions_down.contains(action)
    }

    pub fn action_pressed(&self, action: &str) -> bool {
        self.actions_pressed.contains(action)
    }

    pub fn project(&mut self, input: &InputState) {
        self.recompute_action_down_states(input);
        let presses = input.keyboard_press_samples();
        for press in presses.iter().skip(self.processed_keyboard_presses) {
            for action in self
                .bindings
                .matching_actions(&press.physical_key, press.modifiers)
            {
                self.actions_pressed.insert(action);
            }
        }
        self.processed_keyboard_presses = presses.len();
    }

    pub(crate) fn clear_frame(&mut self, input: &InputState) {
        self.actions_pressed.clear();
        self.processed_keyboard_presses = 0;
        self.recompute_action_down_states(input);
    }

    fn apply_binding_change_inner(&mut self, change: InputBindingChange) -> bool {
        match change {
            InputBindingChange::MapKey { action, key } => self.bindings.map_key(action, key),
            InputBindingChange::MapChord { action, chord } => {
                self.bindings.map_chord(action, chord)
            }
            InputBindingChange::UnmapKey { action, key } => {
                self.bindings.unmap_key(&action, &key) > 0
            }
            InputBindingChange::UnmapChord { action, chord } => {
                self.bindings.unmap_chord(&action, &chord)
            }
            InputBindingChange::ClearAction { action } => self.bindings.clear_action(&action),
            InputBindingChange::ResetDefaults => {
                self.bindings = InputBindings::with_default_bindings();
                true
            }
        }
    }

    fn recompute_action_down_states(&mut self, input: &InputState) {
        let modifiers = input.modifiers_snapshot();
        let mut actions_down = HashSet::new();
        for action in self.bindings.action_ids() {
            if self
                .bindings
                .action_down(action, |key| input.physical_key_down(key), modifiers)
            {
                actions_down.insert(action.clone());
            }
        }
        self.actions_down = actions_down;
    }
}
