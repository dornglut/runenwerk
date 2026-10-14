//! Native-window semantic input recipients. Presentation is never hit authority.
//! This registration is preparatory: the U9-C native event cutover remains gated.

use std::collections::BTreeMap;

use runenui_runtime::{InputArbitrationScope, InputOwnershipSnapshot, RuntimeStatus};

use crate::plugins::render::host::{RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};
use crate::runtime::{NativeWindowId, NativeWindowLifecycleState, WindowStateRegistryResource};

use super::{
    UiRuntimeDisplayedPresentation, UiRuntimePresentationAssociationsResource, UiRuntimeSlotId,
    UiRuntimeSlotOperationFailure, UiRuntimeSlotsResource,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeInputRecipient {
    LegacyNativeConsumer,
    RunenUiSlot(UiRuntimeSlotId),
}

#[derive(Clone, Debug)]
struct Binding {
    recipient: NativeInputRecipient,
    scope: Option<InputArbitrationScope>,
    /// Monotonic native recipient registration lifetime; no focus mirror.
    epoch: u64,
}

/// Explicit retirement is not absence. A formerly UI-owned native window must
/// not become an implicit game recipient when its final UI mount disappears.
#[derive(Clone, Debug)]
enum WindowRecipientBinding {
    Bound(Binding),
    Retired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum NativeInputRecipientError {
    #[error("native input window does not exist")]
    MissingWindow,
    #[error("native input window is not created")]
    InactiveWindow,
    #[error("a native window with mounted UI runtimes requires an explicit input recipient")]
    MissingRecipient,
    #[error("native window already has a different semantic input recipient")]
    ConflictingRecipient,
    #[error("native input recipient lifetime identities have been exhausted")]
    BindingEpochExhausted,
    #[error("RunenUI runtime slot is already assigned to another native window")]
    SlotAlreadyBound,
    #[error("RunenUI mount/scope changed since native input binding")]
    StaleSlot,
    #[error("RunenUI runtime is not running")]
    RuntimeNotRunning,
    #[error("selected UI slot has no completed Present for the current native target")]
    NoCompletedPresentation,
    #[error("fresh RunenUI M13 ownership observation unavailable: {0:?}")]
    OwnershipUnavailable(UiRuntimeSlotOperationFailure),
}

/// Exactly one explicit semantic recipient per native window. No substitute
/// RunenUI focus, presentation, physical input or game permission authority.
#[derive(Default, runen_ecs::Resource)]
pub struct UiNativeInputRecipientsResource {
    bindings: BTreeMap<NativeWindowId, WindowRecipientBinding>,
    next_epoch: u64,
    /// An explicit application selection, not visual z-order or a UI mount.
    /// Realized only once the native primary window is Created and UI is Running.
    requested_primary: Option<NativeInputRecipient>,
}

impl UiNativeInputRecipientsResource {
    /// Declare primary input ownership during App configuration, before the
    /// native window exists. A UI mount alone NEVER makes this choice.
    pub fn request_primary(
        &mut self,
        recipient: NativeInputRecipient,
    ) -> Result<(), NativeInputRecipientError> {
        if let Some(requested) = self.requested_primary {
            return if requested == recipient {
                Ok(())
            } else {
                Err(NativeInputRecipientError::ConflictingRecipient)
            };
        }
        if let Some(WindowRecipientBinding::Bound(existing)) =
            self.bindings.get(&NativeWindowId::primary())
            && existing.recipient != recipient
        {
            return Err(NativeInputRecipientError::ConflictingRecipient);
        }
        self.requested_primary = Some(recipient);
        Ok(())
    }

    /// Admit the declared selection against the real created primary window,
    /// actual mounted slot and current M13 Running scope. Idempotent if native
    /// startup is re-entered, and never silently chooses a visible slot.
    pub fn realize_requested_primary(
        &mut self,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
    ) -> Result<bool, NativeInputRecipientError> {
        let Some(recipient) = self.requested_primary else {
            return Ok(false);
        };
        self.bind(NativeWindowId::primary(), recipient, windows, slots)
    }

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
        if let Some(WindowRecipientBinding::Bound(existing)) = self.bindings.get(&window) {
            return if existing.recipient == recipient && existing.scope == scope {
                Ok(false)
            } else {
                Err(NativeInputRecipientError::ConflictingRecipient)
            };
        }
        if let NativeInputRecipient::RunenUiSlot(slot) = recipient
            && self.bindings.values().any(|binding| {
                matches!(
                    binding,
                    WindowRecipientBinding::Bound(Binding {
                        recipient: NativeInputRecipient::RunenUiSlot(bound),
                        ..
                    }) if *bound == slot
                )
            })
        {
            return Err(NativeInputRecipientError::SlotAlreadyBound);
        }
        let epoch = self
            .next_epoch
            .checked_add(1)
            .ok_or(NativeInputRecipientError::BindingEpochExhausted)?;
        self.bindings.insert(
            window,
            WindowRecipientBinding::Bound(Binding {
                recipient,
                scope,
                epoch,
            }),
        );
        self.next_epoch = epoch;
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
        let binding = match self.bindings.get(&window) {
            Some(WindowRecipientBinding::Bound(binding)) => binding,
            Some(WindowRecipientBinding::Retired) => {
                return Err(NativeInputRecipientError::MissingRecipient);
            }
            None if slots.is_empty() => return Ok(NativeInputRecipient::LegacyNativeConsumer),
            None => return Err(NativeInputRecipientError::MissingRecipient),
        };
        if let NativeInputRecipient::RunenUiSlot(slot) = binding.recipient {
            let ownership = Self::running_ownership(slots, slot)?;
            if binding.scope.as_ref() != Some(ownership.scope()) {
                return Err(NativeInputRecipientError::StaleSlot);
            }
        }
        Ok(binding.recipient)
    }

    /// Read after authoritative `resolve` in the same invoker-thread callback.
    /// Retire/rebind always allocates a new identity, even when window, slot and
    /// physical source happen to have the same nominal identity.
    #[must_use]
    pub fn binding_epoch_for_window(&self, window: NativeWindowId) -> Option<u64> {
        match self.bindings.get(&window) {
            Some(WindowRecipientBinding::Bound(binding)) => Some(binding.epoch),
            _ => None,
        }
    }

    /// Returns only the existing terminal completed-Present snapshot for this
    /// explicitly selected native recipient. Prepared paint and GPU admission
    /// are not display evidence, and this does not grant gameplay permission.
    pub fn displayed_pointer_input(
        &self,
        window: NativeWindowId,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
        presentations: &UiRuntimePresentationAssociationsResource,
        render_surfaces: &RenderSurfaceRegistryResource,
    ) -> Result<Option<UiRuntimeDisplayedPresentation>, NativeInputRecipientError> {
        match self.resolve(window, windows, slots)? {
            NativeInputRecipient::LegacyNativeConsumer => Ok(None),
            NativeInputRecipient::RunenUiSlot(slot) => {
                let displayed = presentations
                    .displayed_for_native_recipient(slot, window)
                    .ok_or(NativeInputRecipientError::NoCompletedPresentation)?;
                let live = windows
                    .record(window)
                    .ok_or(NativeInputRecipientError::MissingWindow)?;
                // Native resize/DPI callbacks update this record immediately.
                // RenderPrepare mapping invalidation may run only next frame.
                // Never route input against old pixels during that interval.
                if displayed.mapping.target_size_px() != live.size_px
                    || displayed.mapping.native_scale_factor() != live.scale_factor
                {
                    return Err(NativeInputRecipientError::NoCompletedPresentation);
                }
                // Render-surface retirement can precede the next RenderPrepare
                // refresh without retiring the native window itself.
                let attached = render_surfaces
                    .record(displayed.render_surface_id)
                    .ok_or(NativeInputRecipientError::NoCompletedPresentation)?;
                if attached.lifecycle_state != RenderSurfaceLifecycleState::Attached
                    || attached.native_window_id != window
                    || attached.target_size_px != displayed.mapping.target_size_px()
                    || render_surfaces.surface_for_native_window(window)
                        != Some(displayed.render_surface_id)
                {
                    return Err(NativeInputRecipientError::NoCompletedPresentation);
                }
                Ok(Some(displayed.clone()))
            }
        }
    }

    /// Retire on native window destruction, target change or source loss.
    /// The *existing* completed-Present ledger also loses that slot's mapping;
    /// rebind must wait for a fresh terminal display rather than reviving an
    /// obsolete context from the previous window/source lifetime.
    pub fn retire_window(
        &mut self,
        window: NativeWindowId,
        presentations: &mut UiRuntimePresentationAssociationsResource,
    ) -> bool {
        let Some(binding) = self.bindings.get_mut(&window) else {
            return false;
        };
        let WindowRecipientBinding::Bound(previous) = binding else {
            return false;
        };
        if let NativeInputRecipient::RunenUiSlot(slot) = previous.recipient {
            presentations.invalidate_mapping(slot);
        }
        *binding = WindowRecipientBinding::Retired;
        true
    }

    /// Retire a mounted runtime's recipient and its old displayed input.
    /// Preserve an explicit unbound marker until the surviving native window
    /// has a newly validated recipient or has itself been destroyed.
    pub fn retire_slot(
        &mut self,
        slot: UiRuntimeSlotId,
        presentations: &mut UiRuntimePresentationAssociationsResource,
    ) -> usize {
        let mut retired = 0;
        for binding in self.bindings.values_mut() {
            if matches!(
                binding,
                WindowRecipientBinding::Bound(Binding {
                    recipient: NativeInputRecipient::RunenUiSlot(bound),
                    ..
                }) if *bound == slot
            ) {
                *binding = WindowRecipientBinding::Retired;
                retired += 1;
            }
        }
        presentations.invalidate_mapping(slot);
        retired
    }

    /// Release a retired window's identity only after its native record is
    /// actually gone. This bounds registry state to real window lifetimes;
    /// retiring a still-live input source never silently grants gameplay.
    pub fn forget_destroyed_window(
        &mut self,
        window: NativeWindowId,
        windows: &WindowStateRegistryResource,
    ) -> Result<bool, NativeInputRecipientError> {
        if windows.record(window).is_some() {
            return Err(NativeInputRecipientError::InactiveWindow);
        }
        match self.bindings.get(&window) {
            Some(WindowRecipientBinding::Bound(_)) => {
                Err(NativeInputRecipientError::ConflictingRecipient)
            }
            Some(WindowRecipientBinding::Retired) => {
                self.bindings.remove(&window);
                Ok(true)
            }
            None => Ok(false),
        }
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
    use runenui_core::{LogicalLength, NoHostProtocol, StyleEnvironment, UiApp, View, text};
    use runenui_runtime::{LogicalSize, RasterScale, SurfaceBuildContext};

    use super::*;
    use crate::plugins::render::RenderFrameProducerId;
    use crate::plugins::render::host::RenderSurfaceId;
    use crate::plugins::ui::{
        UiRuntimeFontConfiguration, UiRuntimeNativeMapping, UiRuntimePresentationBinding,
        UiTypedScreenId,
    };

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
    fn declared_primary_native_recipient_requires_explicit_selection_and_created_window() {
        let (windows, primary, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let mut recipients = UiNativeInputRecipientsResource::default();
        assert_eq!(
            recipients
                .realize_requested_primary(&windows, &mut slots)
                .expect("no input binding is an intentional render-only state"),
            false
        );
        let slot = mount(&mut slots, 15);
        recipients
            .request_primary(NativeInputRecipient::RunenUiSlot(slot))
            .expect("author deliberately selected the primary keyboard recipient");
        assert_eq!(
            recipients.request_primary(NativeInputRecipient::LegacyNativeConsumer),
            Err(NativeInputRecipientError::ConflictingRecipient),
            "selection cannot be implicitly overwritten by another consumer"
        );
        assert_eq!(
            recipients.realize_requested_primary(&windows, &mut slots),
            Ok(true)
        );
        let epoch = recipients
            .binding_epoch_for_window(primary)
            .expect("one live exact binding");
        assert_eq!(
            recipients.realize_requested_primary(&windows, &mut slots),
            Ok(false),
            "repeat startup cannot advance binding identity or create a second owner"
        );
        assert_eq!(recipients.binding_epoch_for_window(primary), Some(epoch));
        assert_eq!(
            recipients.resolve(primary, &windows, &mut slots),
            Ok(NativeInputRecipient::RunenUiSlot(slot))
        );
    }

    #[test]
    fn binding_epoch_exhaustion_rejects_without_implicit_recipient_mutation() {
        let (windows, first, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let mut recipients = UiNativeInputRecipientsResource {
            next_epoch: u64::MAX,
            ..Default::default()
        };
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots,
            ),
            Err(NativeInputRecipientError::BindingEpochExhausted),
        );
        assert!(recipients.bindings.is_empty());
        assert_eq!(recipients.binding_epoch_for_window(first), None);
        assert_eq!(recipients.next_epoch, u64::MAX);
    }

    #[test]
    fn game_only_window_without_any_ui_runtime_retains_existing_route() {
        let (windows, first, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let recipients = UiNativeInputRecipientsResource::default();
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Ok(NativeInputRecipient::LegacyNativeConsumer)
        );
    }

    #[test]
    fn unbound_window_with_mounted_ui_runtime_never_defaults_to_game() {
        let (windows, first, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let _slot = mount(&mut slots, 1);
        let recipients = UiNativeInputRecipientsResource::default();
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingRecipient)
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
            Err(NativeInputRecipientError::MissingRecipient)
        );
        assert_eq!(
            recipients.bind(
                second,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots
            ),
            Ok(true)
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
                &mut slots,
            )
            .expect("bind first");
        slots
            .unmount_with_retirement(a)
            .expect("runtime retirement");
        assert!(matches!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::OwnershipUnavailable(
                UiRuntimeSlotOperationFailure::MissingSlot
            ))
        ));
        let mut presentations = UiRuntimePresentationAssociationsResource::default();
        assert_eq!(recipients.retire_slot(a, &mut presentations), 1);
        assert_eq!(recipients.retire_slot(a, &mut presentations), 0);
        windows.remove_window(second);
        assert_eq!(
            recipients.resolve(second, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingWindow)
        );
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingRecipient)
        );
        // A deliberate new semantic target can restore gameplay, but source
        // retirement and mount removal alone must never do so.
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots,
            ),
            Ok(true)
        );
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Ok(NativeInputRecipient::LegacyNativeConsumer)
        );
    }

    #[test]
    fn live_source_retirement_requires_rebind_and_destroyed_source_can_be_pruned() {
        let (mut windows, first, second) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = mount(&mut slots, 99);
        let mut recipients = UiNativeInputRecipientsResource::default();
        let mut presentations = UiRuntimePresentationAssociationsResource::default();

        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(slot),
                &windows,
                &mut slots,
            ),
            Ok(true)
        );
        assert!(recipients.retire_window(first, &mut presentations));
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingRecipient)
        );
        // Even with zero UI mounts, retirement cannot auto-promote game input.
        slots.unmount_with_retirement(slot).expect("retire slot");
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingRecipient)
        );
        assert_eq!(
            recipients.forget_destroyed_window(first, &windows),
            Err(NativeInputRecipientError::InactiveWindow)
        );
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots,
            ),
            Ok(true)
        );
        windows.remove_window(first);
        assert_eq!(
            recipients.forget_destroyed_window(first, &windows),
            Err(NativeInputRecipientError::ConflictingRecipient)
        );
        assert!(recipients.retire_window(first, &mut presentations));
        assert_eq!(
            recipients.forget_destroyed_window(first, &windows),
            Ok(true)
        );
        assert_eq!(
            recipients.forget_destroyed_window(first, &windows),
            Ok(false)
        );
        assert_eq!(
            recipients.resolve(second, &windows, &mut slots),
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
        assert!(recipients.retire_window(
            first,
            &mut UiRuntimePresentationAssociationsResource::default()
        ));
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

    #[test]
    fn displayed_pointer_input_requires_exact_terminal_present_and_current_native_mapping() {
        // Structural correlation proof only: the actual GPU completed-Present
        // producer/observer is independently exercised by the native GPU lane.
        let (mut windows, first, second) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = mount(&mut slots, 1);
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                first,
                NativeInputRecipient::RunenUiSlot(slot),
                &windows,
                &mut slots,
            )
            .expect("explicit native recipient");

        let mut presentations = UiRuntimePresentationAssociationsResource::default();
        let mut render_surfaces = RenderSurfaceRegistryResource::default();
        let render_surface = RenderSurfaceId::primary();
        render_surfaces
            .confirm_surface_attachment(render_surface, first, (640, 480))
            .expect("realized render surface");
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        assert_eq!(
            recipients
                .displayed_pointer_input(
                    second,
                    &windows,
                    &mut slots,
                    &presentations,
                    &render_surfaces,
                )
                .unwrap_err(),
            NativeInputRecipientError::MissingRecipient
        );

        let producer = RenderFrameProducerId::try_from_raw(71).expect("valid producer");
        let style = StyleEnvironment::default();
        let binding =
            UiRuntimePresentationBinding::new(slot, producer, render_surface, style.clone());
        let mapping = UiRuntimeNativeMapping::new(first, render_surface, (640, 480), 1.0)
            .expect("valid host mapping");
        presentations.bind(binding.clone()).expect("exact binding");
        assert!(presentations.update_mapping(slot, mapping));

        let context = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(LogicalLength::from(640_u16), LogicalLength::from(480_u16)),
        );
        let publication = slots
            .publish_surface_for_slot(slot, &context)
            .expect("real RunenUI runtime publication");
        let expected = publication.input_context().clone();
        let publication_id = presentations
            .stage(
                binding.clone(),
                mapping,
                slots.ingress_epoch(slot).expect("mounted epoch"),
                publication,
            )
            .expect("valid prepared publication");

        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        assert!(presentations.associate_accepted_submission(publication_id));
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        assert!(
            presentations
                .promote_completed_publication(publication_id)
                .is_some()
        );
        let displayed = recipients
            .displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            )
            .expect("live mounted recipient")
            .expect("only completed display is available");
        assert_eq!(displayed.input_context, expected);
        assert_eq!(displayed.mapping.native_window_id(), first);
        assert!(
            presentations
                .displayed_for_native_recipient(slot, second)
                .is_none()
        );

        // A real native callback changes the live host record before the
        // next RenderPrepare refreshes the association mapping. The lookup
        // itself must reject either changed dimension or DPI immediately.
        windows
            .record_mut(first)
            .expect("created window")
            .scale_factor = 2.0;
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        windows
            .record_mut(first)
            .expect("created window")
            .scale_factor = 1.0;
        windows.record_mut(first).expect("created window").size_px = (800, 600);
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        windows
            .record_mut(first)
            .expect("created window")
            .scale_factor = 2.0;
        assert!(render_surfaces.update_surface_extent_for_native_window(first, (800, 600)));

        let changed = UiRuntimeNativeMapping::new(first, render_surface, (800, 600), 2.0)
            .expect("valid changed host mapping");
        assert!(presentations.update_mapping(slot, changed));
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));

        // Establish a second genuine framework publication and structural
        // completion witness for the new mapping before retiring the native
        // source. This verifies that retirement invalidates a *live* display.
        // The 800x600 physical target at 2x raster scale must publish the
        // corresponding 400x300 logical surface, not reuse the old 1x input.
        let changed_context = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(LogicalLength::from(400_u16), LogicalLength::from(300_u16)),
        )
        .with_raster_scale(RasterScale::new(2.0).expect("valid native raster scale"));
        let next = slots
            .publish_surface_for_slot(slot, &changed_context)
            .expect("new RunenUI publication");
        let next_id = presentations
            .stage(
                binding,
                changed,
                slots.ingress_epoch(slot).expect("mounted epoch"),
                next,
            )
            .expect("stage new native mapping");
        assert!(presentations.associate_accepted_submission(next_id));
        assert!(
            presentations
                .promote_completed_publication(next_id)
                .is_some()
        );
        assert!(
            recipients
                .displayed_pointer_input(
                    first,
                    &windows,
                    &mut slots,
                    &presentations,
                    &render_surfaces,
                )
                .expect("valid recipient")
                .is_some()
        );

        // A renderer detach is immediately authoritative even before a UI
        // RenderPrepare withdrawal and while the native window still exists.
        assert_eq!(
            render_surfaces.retire_surface_for_native_window(first),
            Some(render_surface)
        );
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));

        assert!(recipients.retire_window(first, &mut presentations));
        assert!(
            presentations
                .displayed_for_native_recipient(slot, first)
                .is_none()
        );
        assert_eq!(
            recipients.bind(
                first,
                NativeInputRecipient::RunenUiSlot(slot),
                &windows,
                &mut slots,
            ),
            Ok(true)
        );
        assert!(matches!(
            recipients.displayed_pointer_input(
                first,
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
            ),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
    }
}
