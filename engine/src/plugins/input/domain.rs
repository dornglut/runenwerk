use std::collections::{HashMap, HashSet};
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

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

    fn matching_actions_for_key(&self, key: KeyCode, modifiers: ModifiersSnapshot) -> Vec<String> {
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

    fn action_down(
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

    fn action_ids(&self) -> impl Iterator<Item = &String> {
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

#[derive(Debug)]
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    bindings: InputBindings,
    actions_down: HashSet<String>,
    actions_pressed: HashSet<String>,
    pub typed_text: String,
    pub submitted: bool,
    pub insert_newline: bool,
    pub backspace: bool,
    pub delete: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub move_home: bool,
    pub move_end: bool,
    pub page_up: bool,
    pub page_down: bool,
    pub world_move_left: bool,
    pub world_move_right: bool,
    pub world_move_up: bool,
    pub world_move_down: bool,
    pub toggle_pause_menu: bool,
    pub toggle_ui_editor_mode: bool,
    pub save_ui_template: bool,
    pub editor_hide_selected: bool,
    pub editor_restore_all: bool,
    pub scene_next: bool,
    pub scene_prev: bool,
    pub scene_console: bool,
    pub scene_hud: bool,
    pub scene_overlay_push: bool,
    pub scene_overlay_pop: bool,
    pub overlay_consumed: bool,
    pub mouse_delta: (f32, f32),
    pub mouse_position: (f32, f32),
    pub scroll_delta: f32,
    mouse_buttons_down: HashSet<MouseButton>,
    left_mouse_pressed: bool,
    left_mouse_released: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            keys_down: HashSet::new(),
            bindings: InputBindings::with_default_bindings(),
            actions_down: HashSet::new(),
            actions_pressed: HashSet::new(),
            typed_text: String::new(),
            submitted: false,
            insert_newline: false,
            backspace: false,
            delete: false,
            move_left: false,
            move_right: false,
            move_up: false,
            move_down: false,
            move_home: false,
            move_end: false,
            page_up: false,
            page_down: false,
            world_move_left: false,
            world_move_right: false,
            world_move_up: false,
            world_move_down: false,
            toggle_pause_menu: false,
            toggle_ui_editor_mode: false,
            save_ui_template: false,
            editor_hide_selected: false,
            editor_restore_all: false,
            scene_next: false,
            scene_prev: false,
            scene_console: false,
            scene_hud: false,
            scene_overlay_push: false,
            scene_overlay_pop: false,
            overlay_consumed: false,
            mouse_delta: (0.0, 0.0),
            mouse_position: (0.0, 0.0),
            scroll_delta: 0.0,
            mouse_buttons_down: HashSet::new(),
            left_mouse_pressed: false,
            left_mouse_released: false,
        }
    }
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bindings(&self) -> &InputBindings {
        &self.bindings
    }

    pub fn set_bindings(&mut self, bindings: InputBindings) {
        self.bindings = bindings;
        self.actions_pressed.clear();
        self.refresh_action_state_from_bindings();
    }

    pub fn reset_default_bindings(&mut self) {
        self.set_bindings(InputBindings::with_default_bindings());
    }

    pub fn map_key(&mut self, action: impl Into<String>, key: KeyCode) {
        self.map_chord(action, KeyChord::new(key));
    }

    pub fn map_chord(&mut self, action: impl Into<String>, chord: KeyChord) {
        if self.bindings.map_chord(action, chord) {
            self.refresh_action_state_from_bindings();
        }
    }

    pub fn unmap_key(&mut self, action: &str, key: KeyCode) -> usize {
        let removed = self.bindings.unmap_key(action, key);
        if removed > 0 {
            self.refresh_action_state_from_bindings();
        }
        removed
    }

    pub fn unmap_chord(&mut self, action: &str, chord: KeyChord) -> bool {
        let removed = self.bindings.unmap_chord(action, chord);
        if removed {
            self.refresh_action_state_from_bindings();
        }
        removed
    }

    pub fn clear_action_bindings(&mut self, action: &str) -> bool {
        let removed = self.bindings.clear_action(action);
        if removed {
            self.refresh_action_state_from_bindings();
        }
        removed
    }

    pub fn apply_binding_change(&mut self, change: InputBindingChange) -> InputBindingChangeResult {
        if self.apply_binding_change_inner(change) {
            self.refresh_action_state_from_bindings();
            InputBindingChangeResult::Applied
        } else {
            InputBindingChangeResult::Noop
        }
    }

    pub fn apply_binding_changes<I>(&mut self, changes: I) -> usize
    where
        I: IntoIterator<Item = InputBindingChange>,
    {
        let mut applied = 0usize;
        for change in changes {
            if self.apply_binding_change_inner(change) {
                applied = applied.saturating_add(1);
            }
        }
        if applied > 0 {
            self.refresh_action_state_from_bindings();
        }
        applied
    }

    pub fn action_down(&self, action: &str) -> bool {
        self.actions_down.contains(action)
    }

    pub fn action_pressed(&self, action: &str) -> bool {
        self.actions_pressed.contains(action)
    }

    pub fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            let is_new_press = self.keys_down.insert(code);
                            self.recompute_action_down_states();
                            if is_new_press {
                                self.apply_action_press_for_key(code);
                            } else {
                                self.sync_legacy_flags();
                            }
                        }
                        ElementState::Released => {
                            self.keys_down.remove(&code);
                            self.recompute_action_down_states();
                            self.sync_legacy_flags();
                        }
                    }
                }

                if let Some(text) = &event.text {
                    for ch in text.chars() {
                        if !ch.is_control() {
                            self.typed_text.push(ch);
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(_, y) => self.scroll_delta += *y,
                MouseScrollDelta::PixelDelta(p) => self.scroll_delta += p.y as f32,
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    self.mouse_buttons_down.insert(*button);
                    if *button == MouseButton::Left {
                        self.left_mouse_pressed = true;
                    }
                }
                ElementState::Released => {
                    self.mouse_buttons_down.remove(button);
                    if *button == MouseButton::Left {
                        self.left_mouse_released = true;
                    }
                }
            },
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.mouse_delta.0 += delta.0 as f32;
            self.mouse_delta.1 += delta.1 as f32;
        }
    }

    pub fn clear_frame(&mut self) {
        self.typed_text.clear();
        self.actions_pressed.clear();
        self.overlay_consumed = false;
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = 0.0;
        self.left_mouse_pressed = false;
        self.left_mouse_released = false;
        self.refresh_action_state_from_bindings();
    }

    pub fn left_mouse_down(&self) -> bool {
        self.mouse_buttons_down.contains(&MouseButton::Left)
    }

    pub fn right_mouse_down(&self) -> bool {
        self.mouse_buttons_down.contains(&MouseButton::Right)
    }

    pub fn left_mouse_pressed(&self) -> bool {
        self.left_mouse_pressed
    }

    pub fn left_mouse_released(&self) -> bool {
        self.left_mouse_released
    }

    pub fn shift_down(&self) -> bool {
        self.keys_down.contains(&KeyCode::ShiftLeft)
            || self.keys_down.contains(&KeyCode::ShiftRight)
    }

    fn ctrl_down(&self) -> bool {
        self.keys_down.contains(&KeyCode::ControlLeft)
            || self.keys_down.contains(&KeyCode::ControlRight)
    }

    fn alt_down(&self) -> bool {
        self.keys_down.contains(&KeyCode::AltLeft) || self.keys_down.contains(&KeyCode::AltRight)
    }

    fn super_down(&self) -> bool {
        self.keys_down.contains(&KeyCode::SuperLeft)
            || self.keys_down.contains(&KeyCode::SuperRight)
    }

    fn modifiers_snapshot(&self) -> ModifiersSnapshot {
        ModifiersSnapshot {
            shift: self.shift_down(),
            ctrl: self.ctrl_down(),
            alt: self.alt_down(),
            super_key: self.super_down(),
        }
    }

    fn apply_action_press_for_key(&mut self, key: KeyCode) {
        let modifiers = self.modifiers_snapshot();
        let actions = self.bindings.matching_actions_for_key(key, modifiers);
        for action in actions {
            self.actions_pressed.insert(action.clone());
            self.actions_down.insert(action);
        }
        self.sync_legacy_flags();
    }

    fn apply_binding_change_inner(&mut self, change: InputBindingChange) -> bool {
        match change {
            InputBindingChange::MapKey { action, key } => self.bindings.map_key(action, key),
            InputBindingChange::MapChord { action, chord } => {
                self.bindings.map_chord(action, chord)
            }
            InputBindingChange::UnmapKey { action, key } => {
                self.bindings.unmap_key(&action, key) > 0
            }
            InputBindingChange::UnmapChord { action, chord } => {
                self.bindings.unmap_chord(&action, chord)
            }
            InputBindingChange::ClearAction { action } => self.bindings.clear_action(&action),
            InputBindingChange::ResetDefaults => {
                self.bindings = InputBindings::with_default_bindings();
                true
            }
        }
    }

    fn refresh_action_state_from_bindings(&mut self) {
        self.recompute_action_down_states();
        self.sync_legacy_flags();
    }

    fn recompute_action_down_states(&mut self) {
        self.actions_down.clear();
        let modifiers = self.modifiers_snapshot();
        for action in self.bindings.action_ids() {
            if self
                .bindings
                .action_down(action, &self.keys_down, modifiers)
            {
                self.actions_down.insert(action.clone());
            }
        }
    }

    fn sync_legacy_flags(&mut self) {
        self.submitted = self.action_pressed(action::UI_SUBMIT);
        self.insert_newline = self.action_pressed(action::UI_INSERT_NEWLINE);
        self.backspace = self.action_pressed(action::UI_BACKSPACE);
        self.delete = self.action_pressed(action::UI_DELETE);
        self.move_left = self.action_down(action::UI_MOVE_LEFT);
        self.move_right = self.action_down(action::UI_MOVE_RIGHT);
        self.move_up = self.action_down(action::UI_MOVE_UP);
        self.move_down = self.action_down(action::UI_MOVE_DOWN);
        self.move_home = self.action_down(action::UI_MOVE_HOME);
        self.move_end = self.action_down(action::UI_MOVE_END);
        self.page_up = self.action_down(action::UI_PAGE_UP);
        self.page_down = self.action_down(action::UI_PAGE_DOWN);
        self.world_move_left = self.action_down(action::WORLD_MOVE_LEFT);
        self.world_move_right = self.action_down(action::WORLD_MOVE_RIGHT);
        self.world_move_up = self.action_down(action::WORLD_MOVE_UP);
        self.world_move_down = self.action_down(action::WORLD_MOVE_DOWN);
        self.toggle_pause_menu = self.action_pressed(action::SYSTEM_TOGGLE_PAUSE_MENU);
        self.toggle_ui_editor_mode = self.action_pressed(action::UI_TOGGLE_EDITOR_MODE);
        self.save_ui_template = self.action_pressed(action::UI_SAVE_TEMPLATE);
        self.editor_hide_selected = self.action_pressed(action::UI_EDITOR_HIDE_SELECTED);
        self.editor_restore_all = self.action_pressed(action::UI_EDITOR_RESTORE_ALL);
        self.scene_next = self.action_pressed(action::SCENE_NEXT);
        self.scene_prev = self.action_pressed(action::SCENE_PREV);
        self.scene_console = self.action_pressed(action::SCENE_CONSOLE);
        self.scene_hud = self.action_pressed(action::SCENE_HUD);
        self.scene_overlay_push = self.action_pressed(action::SCENE_OVERLAY_PUSH);
        self.scene_overlay_pop = self.action_pressed(action::SCENE_OVERLAY_POP);
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct ModifiersSnapshot {
    shift: bool,
    ctrl: bool,
    alt: bool,
    super_key: bool,
}

#[cfg(test)]
mod tests {
    use super::{InputBindingChange, InputBindingChangeResult, InputState, KeyChord, action};
    use winit::keyboard::KeyCode;

    #[test]
    fn default_bindings_split_enter_by_shift() {
        let mut submit = InputState::new();
        submit.keys_down.insert(KeyCode::Enter);
        submit.apply_action_press_for_key(KeyCode::Enter);
        assert!(submit.submitted);
        assert!(!submit.insert_newline);

        let mut newline = InputState::new();
        newline.keys_down.insert(KeyCode::ShiftLeft);
        newline.keys_down.insert(KeyCode::Enter);
        newline.apply_action_press_for_key(KeyCode::Enter);
        assert!(newline.insert_newline);
        assert!(!newline.submitted);
    }

    #[test]
    fn default_bindings_split_scene_f2_by_shift() {
        let mut next = InputState::new();
        next.keys_down.insert(KeyCode::F2);
        next.apply_action_press_for_key(KeyCode::F2);
        assert!(next.scene_next);
        assert!(!next.scene_prev);

        let mut prev = InputState::new();
        prev.keys_down.insert(KeyCode::ShiftLeft);
        prev.keys_down.insert(KeyCode::F2);
        prev.apply_action_press_for_key(KeyCode::F2);
        assert!(prev.scene_prev);
        assert!(!prev.scene_next);
    }

    #[test]
    fn save_template_requires_ctrl_or_super() {
        let mut plain_s = InputState::new();
        plain_s.keys_down.insert(KeyCode::KeyS);
        plain_s.apply_action_press_for_key(KeyCode::KeyS);
        assert!(!plain_s.save_ui_template);

        let mut ctrl_s = InputState::new();
        ctrl_s.keys_down.insert(KeyCode::ControlLeft);
        ctrl_s.keys_down.insert(KeyCode::KeyS);
        ctrl_s.apply_action_press_for_key(KeyCode::KeyS);
        assert!(ctrl_s.save_ui_template);
    }

    #[test]
    fn runtime_map_key_rebinds_world_move_left() {
        let mut state = InputState::new();
        assert_eq!(state.unmap_key(action::WORLD_MOVE_LEFT, KeyCode::KeyA), 1);
        state.map_key(action::WORLD_MOVE_LEFT, KeyCode::KeyJ);

        state.keys_down.insert(KeyCode::KeyJ);
        state.apply_action_press_for_key(KeyCode::KeyJ);
        assert!(state.world_move_left);
        assert!(state.action_pressed(action::WORLD_MOVE_LEFT));

        state.clear_frame();
        assert!(state.world_move_left);
        assert!(!state.action_pressed(action::WORLD_MOVE_LEFT));

        state.keys_down.remove(&KeyCode::KeyJ);
        state.recompute_action_down_states();
        state.sync_legacy_flags();
        assert!(!state.world_move_left);
    }

    #[test]
    fn custom_action_is_runtime_queryable() {
        let mut state = InputState::new();
        state.map_chord(
            "debug.toggle_freecam",
            KeyChord::new(KeyCode::KeyP).with_shift_required(),
        );
        state.keys_down.insert(KeyCode::ShiftLeft);
        state.keys_down.insert(KeyCode::KeyP);
        state.apply_action_press_for_key(KeyCode::KeyP);
        assert!(state.action_pressed("debug.toggle_freecam"));
        assert!(state.action_down("debug.toggle_freecam"));
    }

    #[test]
    fn apply_binding_change_supports_event_style_updates() {
        let mut state = InputState::new();
        let result = state.apply_binding_change(InputBindingChange::UnmapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: KeyCode::KeyA,
        });
        assert_eq!(result, InputBindingChangeResult::Applied);
        state.apply_binding_change(InputBindingChange::MapKey {
            action: action::WORLD_MOVE_LEFT.to_string(),
            key: KeyCode::KeyJ,
        });

        state.keys_down.insert(KeyCode::KeyJ);
        state.recompute_action_down_states();
        state.apply_action_press_for_key(KeyCode::KeyJ);
        state.sync_legacy_flags();
        assert!(state.world_move_left);
    }

    #[test]
    fn apply_binding_changes_batches_operations() {
        let mut state = InputState::new();
        let applied = state.apply_binding_changes([
            InputBindingChange::UnmapKey {
                action: action::WORLD_MOVE_RIGHT.to_string(),
                key: KeyCode::KeyD,
            },
            InputBindingChange::MapChord {
                action: action::WORLD_MOVE_RIGHT.to_string(),
                chord: KeyChord::new(KeyCode::ArrowRight),
            },
        ]);
        assert_eq!(applied, 2);
        state.keys_down.insert(KeyCode::ArrowRight);
        state.recompute_action_down_states();
        state.sync_legacy_flags();
        assert!(state.world_move_right);
    }
}
