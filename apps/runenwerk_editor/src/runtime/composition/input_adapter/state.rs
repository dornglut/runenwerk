use std::collections::{BTreeMap, HashMap, HashSet};

use engine::runtime::NativeWindowId;
use runen_input::{
    ContactPhase, ContinuityLoss, DigitalState, InputContext, InputDeviceId, InputSourceId,
    PhysicalKeyIdentity,
};
use ui_input::{Modifiers, PointerDeviceId};
use ui_math::{UiPoint, UiVector};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ModifierControl {
    ShiftLeft,
    ShiftRight,
    ControlLeft,
    ControlRight,
    AltLeft,
    AltRight,
    MetaLeft,
    MetaRight,
}

#[derive(Debug, Default)]
struct ModifierState {
    held: HashSet<(InputContext, ModifierControl)>,
}

impl ModifierState {
    fn update(&mut self, context: InputContext, key: &PhysicalKeyIdentity, state: DigitalState) {
        let Some(control) = modifier_control(key) else {
            return;
        };
        let scoped = (context, control);
        match state {
            DigitalState::Pressed => {
                self.held.insert(scoped);
            }
            DigitalState::Released => {
                self.held.remove(&scoped);
            }
        }
    }

    fn invalidate_continuity(&mut self, context: InputContext, loss: ContinuityLoss) {
        self.held
            .retain(|(candidate, _)| !continuity_loss_contains_context(context, loss, *candidate));
    }

    fn snapshot(&self) -> Modifiers {
        Modifiers {
            shift: self.held.iter().any(|(_, control)| {
                matches!(
                    control,
                    ModifierControl::ShiftLeft | ModifierControl::ShiftRight
                )
            }),
            ctrl: self.held.iter().any(|(_, control)| {
                matches!(
                    control,
                    ModifierControl::ControlLeft | ModifierControl::ControlRight
                )
            }),
            alt: self.held.iter().any(|(_, control)| {
                matches!(
                    control,
                    ModifierControl::AltLeft | ModifierControl::AltRight
                )
            }),
            meta: self.held.iter().any(|(_, control)| {
                matches!(
                    control,
                    ModifierControl::MetaLeft | ModifierControl::MetaRight
                )
            }),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct TargetInputState {
    mouse_positions: HashMap<InputSourceId, UiPoint>,
    touch_positions: HashMap<(InputContext, u64), UiPoint>,
    modifiers: ModifierState,
}

impl TargetInputState {
    pub(super) fn mouse_position(&self, source: InputSourceId) -> UiPoint {
        self.mouse_positions
            .get(&source)
            .copied()
            .unwrap_or(UiPoint::ZERO)
    }

    pub(super) fn observe_mouse_position(
        &mut self,
        source: InputSourceId,
        next: UiPoint,
    ) -> UiVector {
        let previous = self.mouse_position(source);
        self.mouse_positions.insert(source, next);
        next - previous
    }

    pub(super) fn observe_touch(
        &mut self,
        context: InputContext,
        id: u64,
        phase: ContactPhase,
        next: UiPoint,
    ) -> UiVector {
        let key = (context, id);
        let previous = self.touch_positions.get(&key).copied().unwrap_or(next);
        match phase {
            ContactPhase::Begin | ContactPhase::Update => {
                self.touch_positions.insert(key, next);
            }
            ContactPhase::End | ContactPhase::Cancel => {
                self.touch_positions.remove(&key);
            }
        }
        next - previous
    }

    pub(super) fn update_modifiers(
        &mut self,
        context: InputContext,
        key: &PhysicalKeyIdentity,
        state: DigitalState,
    ) {
        self.modifiers.update(context, key, state);
    }

    pub(super) fn modifiers(&self) -> Modifiers {
        self.modifiers.snapshot()
    }

    fn invalidate_continuity(&mut self, context: InputContext, loss: ContinuityLoss) {
        if loss == ContinuityLoss::Source {
            self.mouse_positions.remove(&context.source);
        }
        self.touch_positions.retain(|(candidate, _), _| {
            !continuity_loss_contains_context(context, loss, *candidate)
        });
        self.modifiers.invalidate_continuity(context, loss);
    }
}

#[derive(Debug, Default, runen_ecs::Resource)]
pub struct EditorTargetInputRuntimeResource {
    by_window: BTreeMap<NativeWindowId, TargetInputState>,
    device_ids: HashMap<InputDeviceId, PointerDeviceId>,
    next_device_id: u64,
}

impl EditorTargetInputRuntimeResource {
    pub(crate) fn clear_window(&mut self, native_window_id: NativeWindowId) {
        self.by_window.remove(&native_window_id);
    }

    pub(super) fn invalidate_continuity(
        &mut self,
        native_window_id: NativeWindowId,
        context: InputContext,
        loss: ContinuityLoss,
    ) {
        if let Some(state) = self.by_window.get_mut(&native_window_id) {
            state.invalidate_continuity(context, loss);
        }
    }

    pub(super) fn device_id(&mut self, context: InputContext) -> Option<PointerDeviceId> {
        let device = context.device?;
        if let Some(id) = self.device_ids.get(&device) {
            return Some(*id);
        }
        self.next_device_id = self
            .next_device_id
            .checked_add(1)
            .expect("editor UI input device identity exhausted");
        let id = PointerDeviceId(self.next_device_id);
        self.device_ids.insert(device, id);
        Some(id)
    }

    pub(super) fn target_mut(&mut self, native_window_id: NativeWindowId) -> &mut TargetInputState {
        self.by_window.entry(native_window_id).or_default()
    }
}

fn continuity_loss_contains_context(
    context: InputContext,
    loss: ContinuityLoss,
    candidate: InputContext,
) -> bool {
    match loss {
        ContinuityLoss::Source => candidate.source == context.source,
        ContinuityLoss::Device => {
            candidate.source == context.source && candidate.device == context.device
        }
    }
}

fn modifier_control(key: &PhysicalKeyIdentity) -> Option<ModifierControl> {
    let PhysicalKeyIdentity::Code(code) = key else {
        return None;
    };
    Some(match code.as_str() {
        "ShiftLeft" => ModifierControl::ShiftLeft,
        "ShiftRight" => ModifierControl::ShiftRight,
        "ControlLeft" => ModifierControl::ControlLeft,
        "ControlRight" => ModifierControl::ControlRight,
        "AltLeft" => ModifierControl::AltLeft,
        "AltRight" => ModifierControl::AltRight,
        "SuperLeft" => ModifierControl::MetaLeft,
        "SuperRight" => ModifierControl::MetaRight,
        _ => return None,
    })
}
