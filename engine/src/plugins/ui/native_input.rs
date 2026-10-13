//! Native-window semantic input recipients. Presentation is never hit authority.
//! This registration is preparatory: the U9-C native event cutover remains gated.

use std::collections::BTreeMap;

use runenui_runtime::{InputArbitrationScope, InputOwnershipSnapshot, RuntimeStatus};

use crate::runtime::{NativeWindowId, NativeWindowLifecycleState, WindowStateRegistryResource};

use super::{UiRuntimeSlotId, UiRuntimeSlotOperationFailure, UiRuntimeSlotsResource};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeInputRecipient {
    LegacyNativeConsumer,
    RunenUiSlot(UiRuntimeSlotId),
}

#[derive(Clone, Debug)]
struct Binding {
    recipient: NativeInputRecipient,
    scope: Option<InputArbitrationScope>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NativeInputRecipientError {
    #[error("native input window does not exist")]
    MissingWindow,
    #[error("native input window is not created")]
    InactiveWindow,
    #[error("native window already has a different semantic input recipient")]
    ConflictingRecipient,
    #[error("RunenUI runtime slot is already assigned to another native window")]
    SlotAlreadyBound,
    #[error("RunenUI mount/scope changed since native input binding")]
    StaleSlot,
    #[error("RunenUI runtime is not running")]
    RuntimeNotRunning,
    #[error("fresh RunenUI M13 ownership observation unavailable: {0:?}")]
    OwnershipUnavailable(UiRuntimeSlotOperationFailure),
}

/// Exactly one explicit semantic recipient per native window. No substitute
/// RunenUI focus, presentation, physical input or game permission authority.
#[derive(Default, runen_ecs::Resource)]
pub struct UiNativeInputRecipientsResource {
    bindings: BTreeMap<NativeWindowId, Binding>,
}

impl UiNativeInputRecipientsResource {
    /// An identical live binding is idempotent; changing target requires retire.
    /// All UI registrations obtain the actual runtime scope synchronously.
    pub fn bind(
        &mut self,
        window: NativeWindowId,
        recipient: NativeInputRecipient,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
    ) -> Result<bool, NativeInputRecipientError> {
        Self::require_created(window, windows)?;
        let scope = match recipient {
            NativeInputRecipient::LegacyNativeConsumer => None,
            NativeInputRecipient::RunenUiSlot(slot) => {
                Some(Self::running_ownership(slots, slot)?.scope().clone())
            }
        };
        if let Some(existing) = self.bindings.get(&window) {
            return if existing.recipient == recipient && existing.scope == scope {
                Ok(false)
            } else {
                Err(NativeInputRecipientError::ConflictingRecipient)
            };
        }
        if let NativeInputRecipient::RunenUiSlot(slot) = recipient
            && self.bindings.values().any(|binding| {
                binding.recipient == NativeInputRecipient::RunenUiSlot(slot)
            })
        {
            return Err(NativeInputRecipientError::SlotAlreadyBound);
        }
        self.bindings.insert(window, Binding { recipient, scope });
        Ok(true)
    }

    /// Revalidate target and current scope on every native callback. Unknown,
    /// stopped, stale and capacity-paused UI runtime state never grants legacy
    /// gameplay fallback. An unbound ordinary game window retains its old route.
    pub fn resolve(
        &self,
        window: NativeWindowId,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
    ) -> Result<NativeInputRecipient, NativeInputRecipientError> {
        Self::require_created(window, windows)?;
        let Some(binding) = self.bindings.get(&window) else {
            return Ok(NativeInputRecipient::LegacyNativeConsumer);
        };
        if let NativeInputRecipient::RunenUiSlot(slot) = binding.recipient {
            let ownership = Self::running_ownership(slots, slot)?;
            if binding.scope.as_ref() != Some(ownership.scope()) {
                return Err(NativeInputRecipientError::StaleSlot);
            }
        }
        Ok(binding.recipient)
    }

    /// Retire on window destruction, explicit rebind and native source loss.
    pub fn retire_window(&mut self, window: NativeWindowId) -> bool {
        self.bindings.remove(&window).is_some()
    }

    /// Retire any consumer of a RunenUI mount before releasing that mount.
    pub fn retire_slot(&mut self, slot: UiRuntimeSlotId) -> usize {
        let old_len = self.bindings.len();
        self.bindings.retain(|_, binding| {
            binding.recipient != NativeInputRecipient::RunenUiSlot(slot)
        });
        old_len - self.bindings.len()
    }

    fn require_created(
        window: NativeWindowId,
        windows: &WindowStateRegistryResource,
    ) -> Result<(), NativeInputRecipientError> {
        let record = windows
            .record(window)
            .ok_or(NativeInputRecipientError::MissingWindow)?;
        if record.lifecycle_state != NativeWindowLifecycleState::Created {
            return Err(NativeInputRecipientError::InactiveWindow);
        }
        Ok(())
    }

    fn running_ownership(
        slots: &mut UiRuntimeSlotsResource,
        slot: UiRuntimeSlotId,
    ) -> Result<InputOwnershipSnapshot, NativeInputRecipientError> {
        let ownership = slots
            .input_ownership(slot)
            .map_err(NativeInputRecipientError::OwnershipUnavailable)?;
        if ownership.status() != RuntimeStatus::Running {
            return Err(NativeInputRecipientError::RuntimeNotRunning);
        }
        Ok(ownership)
    }
}

#[cfg(test)]
mod tests {
    use runenui_core::{NoHostProtocol, UiApp, View, text};

    use super::*;
    use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};

    struct TestApp;

    impl UiApp for TestApp {
        type State = ();
        type Action = ();
        type HostProtocol = NoHostProtocol;

        fn root(_: &Self::State) -> impl View<Self::Action> {
            text("native recipient test")
        }

        fn update(
            _: &mut Self::State,
            _: Self::Action,
        ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        }
    }

    fn mount(slots: &mut UiRuntimeSlotsResource, number: u64) -> UiRuntimeSlotId {
        slots
            .mount_app::<TestApp>(
                UiTypedScreenId::new(format!("native.recipient.{number}")),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("fixture mount succeeds")
    }

    fn windows() -> (WindowStateRegistryResource, NativeWindowId, NativeWindowId) {
        let mut windows = WindowStateRegistryResource::default();
        let first = windows.register_primary_window("first", (640, 480), 1.0, true);
        let second = NativeWindowId::try_from_raw(2).expect("second window ID");
        windows.register_created_window(second, "second", (320, 240), 1.0, false);
        (windows, first, second)
    }

    #[test]
    fn unbound_game_window_does_not_infer_ui_from_a_mounted_slot() {
        let (windows, first, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let _slot = mount(&mut slots, 1);
        let recipients = UiNativeInputRecipientsResource::default();
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Ok(NativeInputRecipient::LegacyNativeConsumer)
        );
    }

    #[test]
    fn a_window_has_one_recipient_and_a_slot_cannot_bind_two_windows() {
        let (windows, first, second) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let a = mount(&mut slots, 1);
        let b = mount(&mut slots, 2);
        let mut recipients = UiNativeInputRecipientsResource::default();
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(a),
                &windows,
                &mut slots
            ),
            Ok(true)
        );
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(a),
                &windows,
                &mut slots
            ),
            Ok(false)
        );
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(b),
                &windows,
                &mut slots
            ),
            Err(NativeInputRecipientError::ConflictingRecipient)
        );
        assert_eq!(
            recipients.bind(
                second,
                NativeInputRecipient::RunenUiSlot(a),
                &windows,
                &mut slots
            ),
            Err(NativeInputRecipientError::SlotAlreadyBound)
        );
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Ok(NativeInputRecipient::RunenUiSlot(a))
        );
        assert_eq!(
            recipients.resolve(second, &windows, &mut slots),
            Ok(NativeInputRecipient::LegacyNativeConsumer)
        );
    }

    #[test]
    fn stale_slot_fails_closed_and_does_not_erase_other_window_input() {
        let (mut windows, first, second) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let a = mount(&mut slots, 1);
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                first,
                NativeInputRecipient::RunenUiSlot(a),
                &windows,
                &mut slots
            )
            .expect("bind first");
        slots.unmount_with_retirement(a).expect("runtime retirement");
        assert!(matches!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::OwnershipUnavailable(
                UiRuntimeSlotOperationFailure::MissingSlot
            ))
        ));
        assert_eq!(recipients.retire_slot(a), 1);
        assert_eq!(recipients.retire_slot(a), 0);
        windows.remove_window(second);
        assert_eq!(
            recipients.resolve(second, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingWindow)
        );
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Ok(NativeInputRecipient::LegacyNativeConsumer)
        );
    }

    #[test]
    fn explicit_legacy_ownership_blocks_implicit_ui_takeover() {
        let (windows, first, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let a = mount(&mut slots, 1);
        let mut recipients = UiNativeInputRecipientsResource::default();
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots
            ),
            Ok(true)
        );
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(a),
                &windows,
                &mut slots
            ),
            Err(NativeInputRecipientError::ConflictingRecipient)
        );
        assert!(recipients.retire_window(first));
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(a),
                &windows,
                &mut slots
            ),
            Ok(true)
        );
    }
}
