use super::state::{InputState, KeyboardPressSample, ModifiersSnapshot};
use runen_input::{InputContext, InputSourceId, PhysicalKeyIdentity};
use std::collections::{HashMap, HashSet};
use winit::keyboard::KeyCode;

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
    /// A denied physical modifier cannot satisfy Required, but it remains
    /// physically held and therefore cannot fabricate a Forbidden match.
    fn matches_pressed(self, physical: bool, eligible: bool) -> bool {
        match self {
            Self::Ignore => true,
            Self::Required => eligible,
            Self::Forbidden => !physical,
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

    fn matches_pressed(&self, physical: ModifiersSnapshot, eligible: ModifiersSnapshot) -> bool {
        self.shift.matches_pressed(physical.shift, eligible.shift)
            && self.ctrl.matches_pressed(physical.ctrl, eligible.ctrl)
            && self.alt.matches_pressed(physical.alt, eligible.alt)
            && self
                .super_key
                .matches_pressed(physical.super_key, eligible.super_key)
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

    fn matching_pressed_chords(
        &self,
        key: &PhysicalKeyIdentity,
        physical: ModifiersSnapshot,
        eligible: ModifiersSnapshot,
    ) -> Vec<(String, KeyChord)> {
        let mut matches = Vec::new();
        for (action, chords) in &self.by_action {
            for chord in chords {
                if chord.key == *key && chord.matches_pressed(physical, eligible) {
                    // Keep EACH actual accepted chord: if one modifier
                    // lifetime is denied later, another independent chord
                    // may still support the same action.
                    matches.push((action.clone(), chord.clone()));
                }
            }
        }
        matches
    }

    fn action_down<F>(
        &self,
        action: &str,
        mut key_is_down: F,
        physical: ModifiersSnapshot,
        eligible: ModifiersSnapshot,
    ) -> bool
    where
        F: FnMut(&PhysicalKeyIdentity) -> bool,
    {
        self.by_action.get(action).is_some_and(|chords| {
            chords
                .iter()
                .any(|chord| key_is_down(&chord.key) && chord.matches_pressed(physical, eligible))
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

/// Engine-owned lifetime denial; removing UI focus never grants this
/// physical Down back to gameplay.
#[derive(Debug, Clone, Eq, PartialEq)]
struct RevokedKeyboardLifetime {
    context: InputContext,
    key: PhysicalKeyIdentity,
    generation: u64,
}

/// The exact press contributor accepted under the bindings that existed when
/// it was projected. Later binding edits cannot resurrect discarded evidence.
#[derive(Debug, Clone, Eq, PartialEq)]
struct PressedActionContributor {
    action: String,
    context: InputContext,
    key: PhysicalKeyIdentity,
    generation: u64,
    sample_index: usize,
    /// The exact binding accepted at press time, never reinterpreted under
    /// later mapping changes when an M13 modifier receipt settles.
    chord: KeyChord,
}

/// A revoked held physical key remains blocked through synthetic cancellation,
/// repeat, focus restoration and binding mutation until a genuine source Up.
#[derive(Debug, Clone, Eq, PartialEq)]
struct BlockedHeldKey {
    context: InputContext,
    key: PhysicalKeyIdentity,
    /// Exact genuine Down which the UI claimed; generation ordering is
    /// monotonic across contexts, even if synthetic reconciliation hides it.
    generation: u64,
    /// Index into this frame's genuine Up observations at denial time.
    releases_seen: usize,
}

#[derive(Debug, Clone, runen_ecs::Component, runen_ecs::Resource)]
pub struct ActionState {
    bindings: InputBindings,
    actions_down: HashSet<String>,
    actions_pressed: HashSet<String>,
    processed_keyboard_presses: usize,
    pressed_contributors: Vec<PressedActionContributor>,
    /// Terminal denial for each source-local Down, including held keys and
    /// any same-frame press that may already have reached action projection.
    revoked_keyboard_lifetimes: Vec<RevokedKeyboardLifetime>,
    blocked_held_keys: Vec<BlockedHeldKey>,
}

impl Default for ActionState {
    fn default() -> Self {
        Self {
            bindings: InputBindings::with_default_bindings(),
            actions_down: HashSet::new(),
            actions_pressed: HashSet::new(),
            processed_keyboard_presses: 0,
            pressed_contributors: Vec::new(),
            revoked_keyboard_lifetimes: Vec::new(),
            blocked_held_keys: Vec::new(),
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
        self.pressed_contributors.clear();
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

    /// Permanently revoke an exact source/device/key Down lifetime. This is
    /// immediate even if PreUpdate already projected its pressed/held action.
    /// A UI ownership revision cannot restore the same generation.
    ///
    /// The native owner MUST supply the correlated real physical generation,
    /// not a keyboard key guessed from RunenUI's current focus.
    pub fn revoke_keyboard_lifetime(
        &mut self,
        input: &InputState,
        context: InputContext,
        key: &PhysicalKeyIdentity,
        generation: u64,
    ) {
        if !self.revoked_keyboard_lifetimes.iter().any(|revoked| {
            revoked.context == context && revoked.key == *key && revoked.generation == generation
        }) {
            self.revoked_keyboard_lifetimes
                .push(RevokedKeyboardLifetime {
                    context,
                    key: key.clone(),
                    generation,
                });
        }
        // If the physical key is still held, deny the whole lifetime even
        // through synthetic cancellation; only an authentic Up can retire it.
        if input.physical_key_down_in(context, key)
            && input
                .keyboard_down_generation_in(context, key)
                .is_none_or(|held_generation| held_generation == generation)
            && !self
                .blocked_held_keys
                .iter()
                .any(|blocked| blocked.context == context && blocked.key == *key)
        {
            self.blocked_held_keys.push(BlockedHeldKey {
                context,
                key: key.clone(),
                generation,
                releases_seen: input.keyboard_real_release_samples().len(),
            });
        }
        // The exact denied generation may also have been a modifier for
        // other physical keys pressed later. Withdraw those chord claims too
        // without replaying historic press evidence under current bindings.
        self.retract_ineligible_pressed_contributors(input);
        self.recompute_action_down_states(input);
    }

    fn keyboard_generation_eligible(
        &self,
        context: InputContext,
        key: &PhysicalKeyIdentity,
        generation: u64,
    ) -> bool {
        !self.revoked_keyboard_lifetimes.iter().any(|revoked| {
            revoked.context == context && revoked.key == *key && revoked.generation == generation
        })
    }

    fn blocked_held_key(&self, context: InputContext, key: &PhysicalKeyIdentity) -> bool {
        self.blocked_held_keys
            .iter()
            .any(|blocked| blocked.context == context && blocked.key == *key)
    }

    fn retire_genuinely_released_denials(&mut self, input: &InputState) {
        let releases = input.keyboard_real_release_samples();
        self.blocked_held_keys.retain(|blocked| {
            !releases.iter().skip(blocked.releases_seen).any(|release| {
                release.context == blocked.context && release.physical_key == blocked.key
            })
        });
    }

    /// Only the source-local modifiers recorded at the moment of this
    /// physical Down may satisfy a press chord. Current global/held modifiers
    /// cannot retroactively create a pressed action or mask a denied one.
    fn eligible_pressed_modifiers(&self, press: &KeyboardPressSample) -> ModifiersSnapshot {
        let down = |code: KeyCode| {
            let key = PhysicalKeyIdentity::code(format!("{code:?}"));
            press.modifier_lifetimes.iter().any(|modifier| {
                modifier.physical_key == key
                    && modifier.generation.map_or_else(
                        || !self.blocked_held_key(press.context, &key),
                        |generation| {
                            self.keyboard_generation_eligible(press.context, &key, generation)
                        },
                    )
            })
        };
        ModifiersSnapshot {
            shift: down(KeyCode::ShiftLeft) || down(KeyCode::ShiftRight),
            ctrl: down(KeyCode::ControlLeft) || down(KeyCode::ControlRight),
            alt: down(KeyCode::AltLeft) || down(KeyCode::AltRight),
            super_key: down(KeyCode::SuperLeft) || down(KeyCode::SuperRight),
        }
    }

    /// Synthetic cancellation is not authentic release proof. When a
    /// claimed modifier's genuine Down predates this press, its physical
    /// absence in a reconciliation snapshot is unknown, not an affirmative
    /// absence satisfying a Shift/Ctrl/Alt/Super Forbidden chord.
    fn possibly_present_pressed_modifiers(&self, press: &KeyboardPressSample) -> ModifiersSnapshot {
        let uncertain = |code: KeyCode| {
            let key = PhysicalKeyIdentity::code(format!("{code:?}"));
            self.blocked_held_keys.iter().any(|blocked| {
                blocked.context == press.context
                    && blocked.key == key
                    && blocked.generation < press.generation
            })
        };
        ModifiersSnapshot {
            shift: press.modifiers.shift
                || uncertain(KeyCode::ShiftLeft)
                || uncertain(KeyCode::ShiftRight),
            ctrl: press.modifiers.ctrl
                || uncertain(KeyCode::ControlLeft)
                || uncertain(KeyCode::ControlRight),
            alt: press.modifiers.alt || uncertain(KeyCode::AltLeft) || uncertain(KeyCode::AltRight),
            super_key: press.modifiers.super_key
                || uncertain(KeyCode::SuperLeft)
                || uncertain(KeyCode::SuperRight),
        }
    }

    fn pressed_contributor_eligible(
        &self,
        input: &InputState,
        contribution: &PressedActionContributor,
    ) -> bool {
        let Some(press) = input
            .keyboard_press_samples()
            .get(contribution.sample_index)
        else {
            return false;
        };
        !press.retired_by_continuity
            && press.context == contribution.context
            && press.physical_key == contribution.key
            && press.generation == contribution.generation
            && self.keyboard_generation_eligible(
                press.context,
                &press.physical_key,
                press.generation,
            )
            && contribution.chord.matches_pressed(
                self.possibly_present_pressed_modifiers(press),
                self.eligible_pressed_modifiers(press),
            )
    }

    /// Retire invalidated press/held-modifier evidence immediately, including
    /// a focus-loss callback or M13 receipt arriving after PreUpdate.
    fn retract_ineligible_pressed_contributors(&mut self, input: &InputState) {
        // A read-only eligibility pass before mutable retention avoids a
        // competing borrower and never reinterprets the current binding map.
        let retain: Vec<bool> = self
            .pressed_contributors
            .iter()
            .map(|contributor| self.pressed_contributor_eligible(input, contributor))
            .collect();
        let mut index = 0;
        self.pressed_contributors.retain(|_| {
            let accepted = retain[index];
            index += 1;
            accepted
        });
        self.actions_pressed = self
            .pressed_contributors
            .iter()
            .map(|pressed| pressed.action.clone())
            .collect();
    }

    /// Only real native source retirement releases denied lifetime storage.
    /// Ordinary focus loss is NOT source retirement: it must retain denial
    /// through synthetic key restoration until the original physical Up.
    pub(crate) fn retire_native_keyboard_source(
        &mut self,
        input: &InputState,
        source: InputSourceId,
    ) {
        self.revoked_keyboard_lifetimes
            .retain(|entry| entry.context.source != source);
        self.blocked_held_keys
            .retain(|entry| entry.context.source != source);
        self.pressed_contributors
            .retain(|entry| entry.context.source != source);
        self.actions_pressed = self
            .pressed_contributors
            .iter()
            .map(|entry| entry.action.clone())
            .collect();
        self.recompute_action_down_states(input);
    }

    fn project_presses(&mut self, input: &InputState, start: usize, end: usize) {
        let presses = input.keyboard_press_samples();
        for (sample_index, press) in presses.iter().enumerate().take(end).skip(start) {
            if press.retired_by_continuity
                || self.blocked_held_key(press.context, &press.physical_key)
                || !self.keyboard_generation_eligible(
                    press.context,
                    &press.physical_key,
                    press.generation,
                )
            {
                continue;
            }
            let eligible = self.eligible_pressed_modifiers(press);
            for (action, chord) in self.bindings.matching_pressed_chords(
                &press.physical_key,
                self.possibly_present_pressed_modifiers(press),
                eligible,
            ) {
                self.actions_pressed.insert(action.clone());
                self.pressed_contributors.push(PressedActionContributor {
                    action,
                    context: press.context,
                    key: press.physical_key.clone(),
                    generation: press.generation,
                    sample_index,
                    chord,
                });
            }
        }
    }

    pub fn project(&mut self, input: &InputState) {
        self.retire_genuinely_released_denials(input);
        self.retract_ineligible_pressed_contributors(input);
        self.recompute_action_down_states(input);
        let end = input.keyboard_press_samples().len();
        self.project_presses(input, self.processed_keyboard_presses, end);
        self.processed_keyboard_presses = end;
    }

    pub(crate) fn clear_frame(&mut self, input: &InputState) {
        self.retire_genuinely_released_denials(input);
        self.actions_pressed.clear();
        self.pressed_contributors.clear();
        self.processed_keyboard_presses = 0;
        // Press evidence is frame-local. Denied held lifetimes instead persist
        // until authentic Up, including after synthetic cancellation.
        self.revoked_keyboard_lifetimes.clear();
        for blocked in &mut self.blocked_held_keys {
            blocked.releases_seen = 0;
        }
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

    fn eligible_held_key(
        &self,
        input: &InputState,
        context: InputContext,
        key: &PhysicalKeyIdentity,
    ) -> bool {
        input.physical_key_down_in(context, key)
            && !self.blocked_held_key(context, key)
            && input
                .keyboard_down_generation_in(context, key)
                .is_none_or(|generation| {
                    self.keyboard_generation_eligible(context, key, generation)
                })
    }

    /// Modifiers are physical keys in the SAME source/device context, subject
    /// to the SAME revoked-lifetime rule as ordinary action contributors.
    /// UI-owned Shift cannot silently help a game chord remain held.
    fn eligible_held_modifiers(
        &self,
        input: &InputState,
        context: InputContext,
    ) -> ModifiersSnapshot {
        let down = |code: KeyCode| {
            self.eligible_held_key(
                input,
                context,
                &PhysicalKeyIdentity::code(format!("{code:?}")),
            )
        };
        ModifiersSnapshot {
            shift: down(KeyCode::ShiftLeft) || down(KeyCode::ShiftRight),
            ctrl: down(KeyCode::ControlLeft) || down(KeyCode::ControlRight),
            alt: down(KeyCode::AltLeft) || down(KeyCode::AltRight),
            super_key: down(KeyCode::SuperLeft) || down(KeyCode::SuperRight),
        }
    }

    /// The physical snapshot is intentionally independent of gameplay
    /// permission. A physically down UI-owned modifier is NOT equivalent to
    /// a physically absent modifier for a Forbidden binding.
    fn physical_held_modifiers(
        &self,
        input: &InputState,
        context: InputContext,
    ) -> ModifiersSnapshot {
        let down = |code: KeyCode| {
            let key = PhysicalKeyIdentity::code(format!("{code:?}"));
            input.physical_key_down_in(context, &key) || self.blocked_held_key(context, &key)
        };
        ModifiersSnapshot {
            shift: down(KeyCode::ShiftLeft) || down(KeyCode::ShiftRight),
            ctrl: down(KeyCode::ControlLeft) || down(KeyCode::ControlRight),
            alt: down(KeyCode::AltLeft) || down(KeyCode::AltRight),
            super_key: down(KeyCode::SuperLeft) || down(KeyCode::SuperRight),
        }
    }

    fn recompute_action_down_states(&mut self, input: &InputState) {
        let mut actions_down = HashSet::new();
        for action in self.bindings.action_ids() {
            // One eligible chord must be physically held within one exact
            // source/device context. Keys from different windows or devices
            // must not synthesize a combined held chord.
            if input.keyboard_held_contexts().any(|context| {
                self.bindings.action_down(
                    action,
                    |key| self.eligible_held_key(input, context, key),
                    self.physical_held_modifiers(input, context),
                    self.eligible_held_modifiers(input, context),
                )
            }) {
                actions_down.insert(action.clone());
            }
        }
        self.actions_down = actions_down;
    }
}
