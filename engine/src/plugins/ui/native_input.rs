//! Native-window semantic input recipients. Presentation is never hit authority.
//! This registration is preparatory: the U9-C native event cutover remains gated.

use std::collections::BTreeMap;

use runenui_runtime::{InputArbitrationScope, InputOwnershipSnapshot, RuntimeStatus};

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
            && self
                .bindings
                .values()
                .any(|binding| binding.recipient == NativeInputRecipient::RunenUiSlot(slot))
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
            return if slots.is_empty() {
                Ok(NativeInputRecipient::LegacyNativeConsumer)
            } else {
                Err(NativeInputRecipientError::MissingRecipient)
            };
        };
        if let NativeInputRecipient::RunenUiSlot(slot) = binding.recipient {
            let ownership = Self::running_ownership(slots, slot)?;
            if binding.scope.as_ref() != Some(ownership.scope()) {
                return Err(NativeInputRecipientError::StaleSlot);
            }
        }
        Ok(binding.recipient)
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
    ) -> Result<Option<UiRuntimeDisplayedPresentation>, NativeInputRecipientError> {
        match self.resolve(window, windows, slots)? {
            NativeInputRecipient::LegacyNativeConsumer => Ok(None),
            NativeInputRecipient::RunenUiSlot(slot) => presentations
                .displayed_for_native_recipient(slot, window)
                .cloned()
                .map(Some)
                .ok_or(NativeInputRecipientError::NoCompletedPresentation),
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
        let Some(binding) = self.bindings.remove(&window) else {
            return false;
        };
        if let NativeInputRecipient::RunenUiSlot(slot) = binding.recipient {
            presentations.invalidate_mapping(slot);
        }
        true
    }

    /// Retire a mounted runtime's recipient and its old displayed input.
    /// No separate presentation state is maintained in this registry.
    pub fn retire_slot(
        &mut self,
        slot: UiRuntimeSlotId,
        presentations: &mut UiRuntimePresentationAssociationsResource,
    ) -> usize {
        let old_len = self.bindings.len();
        self.bindings
            .retain(|_, binding| binding.recipient != NativeInputRecipient::RunenUiSlot(slot));
        presentations.invalidate_mapping(slot);
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
        let (windows, first, second) = windows();
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
        assert!(matches!(
            recipients.displayed_pointer_input(first, &windows, &mut slots, &presentations),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        assert_eq!(
            recipients
                .displayed_pointer_input(second, &windows, &mut slots, &presentations)
                .unwrap_err(),
            NativeInputRecipientError::MissingRecipient
        );

        let render_surface = RenderSurfaceId::primary();
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
            LogicalSize::new(
                LogicalLength::from(640.0_f32),
                LogicalLength::from(480.0_f32),
            ),
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
            recipients.displayed_pointer_input(first, &windows, &mut slots, &presentations),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        assert!(presentations.associate_accepted_submission(publication_id));
        assert!(matches!(
            recipients.displayed_pointer_input(first, &windows, &mut slots, &presentations),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
        assert!(
            presentations
                .promote_completed_publication(publication_id)
                .is_some()
        );
        let displayed = recipients
            .displayed_pointer_input(first, &windows, &mut slots, &presentations)
            .expect("live mounted recipient")
            .expect("only completed display is available");
        assert_eq!(displayed.input_context, expected);
        assert_eq!(displayed.mapping.native_window_id(), first);
        assert!(
            presentations
                .displayed_for_native_recipient(slot, second)
                .is_none()
        );

        let changed = UiRuntimeNativeMapping::new(first, render_surface, (800, 600), 2.0)
            .expect("valid changed host mapping");
        assert!(presentations.update_mapping(slot, changed));
        assert!(matches!(
            recipients.displayed_pointer_input(first, &windows, &mut slots, &presentations),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));

        // Establish a second genuine framework publication and structural
        // completion witness for the new mapping before retiring the native
        // source. This verifies that retirement invalidates a *live* display.
        // The 800x600 physical target at 2x raster scale must publish the
        // corresponding 400x300 logical surface, not reuse the old 1x input.
        let changed_context = SurfaceBuildContext::tight(
            &style,
            LogicalSize::new(
                LogicalLength::from(400.0_f32),
                LogicalLength::from(300.0_f32),
            ),
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
                .displayed_pointer_input(first, &windows, &mut slots, &presentations)
                .expect("valid recipient")
                .is_some()
        );

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
            recipients.displayed_pointer_input(first, &windows, &mut slots, &presentations),
            Err(NativeInputRecipientError::NoCompletedPresentation)
        ));
    }
}
