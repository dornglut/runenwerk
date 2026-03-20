use crate::plugins::{
    InputBindingChange, InputBindingChangeResult, InputBindings, KeyChord, action,
};
use std::collections::HashSet;
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

// Owner: Engine Input Plugin - Input State and Event Processing
#[derive(Debug, ecs::Component, ecs::Resource)]
pub struct InputState {
    pub(crate) keys_down: HashSet<KeyCode>,
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
                    self.handle_keyboard_input(code, event.state, event.text.as_deref());
                }
            }
            WindowEvent::MouseWheel { delta, .. } => self.handle_mouse_wheel_delta(match delta {
                MouseScrollDelta::LineDelta(_, y) => *y,
                MouseScrollDelta::PixelDelta(p) => p.y as f32,
            }),
            WindowEvent::CursorMoved { position, .. } => {
                self.handle_cursor_moved(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.handle_mouse_input(*state, *button);
            }
            _ => {}
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.handle_mouse_motion(delta.0 as f32, delta.1 as f32);
        }
    }

    pub fn handle_keyboard_input(
        &mut self,
        code: KeyCode,
        state: ElementState,
        text: Option<&str>,
    ) {
        match state {
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

        if let Some(text) = text {
            for ch in text.chars() {
                if !ch.is_control() {
                    self.typed_text.push(ch);
                }
            }
        }
    }

    pub fn handle_mouse_wheel_delta(&mut self, delta: f32) {
        self.scroll_delta += delta;
    }

    pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
        self.mouse_position = (x, y);
    }

    pub fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => {
                self.mouse_buttons_down.insert(button);
                if button == MouseButton::Left {
                    self.left_mouse_pressed = true;
                }
            }
            ElementState::Released => {
                self.mouse_buttons_down.remove(&button);
                if button == MouseButton::Left {
                    self.left_mouse_released = true;
                }
            }
        }
    }

    pub fn handle_mouse_motion(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
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

    pub(crate) fn apply_action_press_for_key(&mut self, key: KeyCode) {
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

    pub(crate) fn recompute_action_down_states(&mut self) {
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

    pub(crate) fn sync_legacy_flags(&mut self) {
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
pub struct ModifiersSnapshot {
    pub(crate) shift: bool,
    pub(crate) ctrl: bool,
    pub(crate) alt: bool,
    pub(crate) super_key: bool,
}
