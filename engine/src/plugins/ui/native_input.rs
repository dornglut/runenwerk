//! Native-window semantic input recipients. Presentation is never hit authority.
//! This registration is preparatory: the U9-C native event cutover remains gated.

use std::collections::BTreeMap;

use runenui_core::{InputDeviceId, PointerEvent};
use runenui_runtime::{
    InputArbitrationScope, InputOwnershipSnapshot, PointerSubmission, RuntimeStatus,
};

use crate::plugins::render::host::{RenderSurfaceLifecycleState, RenderSurfaceRegistryResource};
use crate::runtime::{NativeWindowId, NativeWindowLifecycleState, WindowStateRegistryResource};

use super::{
    UiRuntimeDisplayedPresentation, UiRuntimeNativePointerAdmissionError,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotId, UiRuntimeSlotOperationFailure,
    UiRuntimeSlotsResource,
};

/// Original native pointer window/device origin, not derived from a displayed
/// UI surface or current redraw target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePointerOrigin {
    window: NativeWindowId,
    device: InputDeviceId,
}

impl NativePointerOrigin {
    #[must_use]
    pub const fn new(window: NativeWindowId, device: InputDeviceId) -> Self {
        Self { window, device }
    }

    #[must_use]
    pub const fn window(self) -> NativeWindowId {
        self.window
    }

    #[must_use]
    pub const fn device(self) -> InputDeviceId {
        self.device
    }
}

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
/// Borrow the true native-window, terminal UI presentation and attached
/// Render authorities inside the Engine UI integration boundary. The Winit
/// driver never names a renderer implementation type or owns hit authority.
pub(crate) struct UiNativePointerHostRead<'a> {
    windows: &'a WindowStateRegistryResource,
    presentations: &'a UiRuntimePresentationAssociationsResource,
    surfaces: &'a RenderSurfaceRegistryResource,
}

impl<'a> UiNativePointerHostRead<'a> {
    pub(crate) fn from_world(world: &'a runen_ecs::World) -> anyhow::Result<Self> {
        Ok(Self {
            windows: world.resource::<WindowStateRegistryResource>()?,
            presentations: world.resource::<UiRuntimePresentationAssociationsResource>()?,
            surfaces: world.resource::<RenderSurfaceRegistryResource>()?,
        })
    }

    pub(crate) fn displayed_input(
        &self,
        recipients: &UiNativeInputRecipientsResource,
        window: NativeWindowId,
        slots: &mut UiRuntimeSlotsResource,
    ) -> Result<Option<UiRuntimeDisplayedPresentation>, NativeInputRecipientError> {
        recipients.displayed_pointer_input(
            window,
            self.windows,
            slots,
            self.presentations,
            self.surfaces,
        )
    }

    pub(crate) fn submit(
        &self,
        recipients: &UiNativeInputRecipientsResource,
        origin: NativePointerOrigin,
        slots: &mut UiRuntimeSlotsResource,
        event: PointerEvent,
    ) -> Result<(UiRuntimeDisplayedPresentation, PointerSubmission), NativePointerIngressError>
    {
        recipients.submit_displayed_native_pointer(
            origin,
            self.windows,
            slots,
            self.presentations,
            self.surfaces,
            event,
        )
    }
}

/// Original-event rejection stays explicit; no legacy gameplay fallback is
/// manufactured from a missing terminal display or unrelated pointer device.
#[derive(Debug)]
pub enum NativePointerIngressError {
    LegacyRecipient(Box<PointerEvent>),
    MissingDisplay {
        reason: NativeInputRecipientError,
        event: Box<PointerEvent>,
    },
    MismatchedDisplay(Box<PointerEvent>),
    MismatchedDevice(Box<PointerEvent>),
    Admission(UiRuntimeNativePointerAdmissionError),
}

impl NativePointerIngressError {
    /// Recover the unaccepted original event from any rejection stage.
    #[must_use]
    pub fn into_event(self) -> PointerEvent {
        match self {
            Self::LegacyRecipient(event)
            | Self::MismatchedDisplay(event)
            | Self::MismatchedDevice(event)
            | Self::MissingDisplay { event, .. } => *event,
            Self::Admission(UiRuntimeNativePointerAdmissionError::Slot { event, .. }) => *event,
            Self::Admission(UiRuntimeNativePointerAdmissionError::Framework(error)) => {
                error.into_event()
            }
        }
    }
}

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
    pub(crate) fn realize_requested_primary(
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
    pub(crate) fn bind(
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
        if let NativeInputRecipient::RunenUiSlot(slot) = recipient {
            slots.enroll_native_input(slot);
        }
        Ok(true)
    }

    /// Raw winit DeviceEvent motion has no native-window identity. Certify
    /// the old source-erased game/camera projection using only the actual
    /// application-owned recipient declarations, never a focus or M13 hit
    /// inference. A pending UI selection, retired binding or unbound window
    /// while UI slots exist cannot silently become legacy game permission.
    ///
    /// Deliberately a read-only binding query: high-rate raw input must not
    /// repeatedly pump or observe a UI runtime merely to reject attribution.
    pub(crate) fn raw_motion_legacy_projection_allowed(
        &self,
        windows: &WindowStateRegistryResource,
        allow_implicit_legacy: bool,
    ) -> bool {
        if matches!(
            self.requested_primary,
            Some(NativeInputRecipient::RunenUiSlot(_))
        ) {
            return false;
        }

        let mut has_created_window = false;
        for window in windows.records() {
            match window.lifecycle_state {
                NativeWindowLifecycleState::Created => has_created_window = true,
                // A failed creation never had an input-capable native handle.
                NativeWindowLifecycleState::CreationFailed => continue,
                // An unknown/pending/closing lifetime may still have active
                // raw device events. Raw Winit motion carries no window ID;
                // skipping that window would falsely certify legacy gameplay.
                NativeWindowLifecycleState::Requested
                | NativeWindowLifecycleState::CloseIntentPending
                | NativeWindowLifecycleState::CloseApproved => return false,
            }
            match self.bindings.get(&window.native_window_id) {
                Some(WindowRecipientBinding::Bound(binding))
                    if binding.recipient == NativeInputRecipient::LegacyNativeConsumer => {}
                None if allow_implicit_legacy => {}
                _ => return false,
            }
        }
        has_created_window
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
        self.resolve_with_close_policy(window, windows, slots, false)
    }

    /// Cancellation during an approved close retains the original binding,
    /// epoch and RunenUI scope without reopening ordinary input admission.
    pub(crate) fn resolve_for_cancellation(
        &self,
        window: NativeWindowId,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
    ) -> Result<NativeInputRecipient, NativeInputRecipientError> {
        self.resolve_with_close_policy(window, windows, slots, true)
    }

    fn resolve_with_close_policy(
        &self,
        window: NativeWindowId,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
        allow_approved_close: bool,
    ) -> Result<NativeInputRecipient, NativeInputRecipientError> {
        let record = windows
            .record(window)
            .ok_or(NativeInputRecipientError::MissingWindow)?;
        if record.lifecycle_state != NativeWindowLifecycleState::Created
            && !(allow_approved_close
                && record.close_requested
                && record.lifecycle_state == NativeWindowLifecycleState::CloseApproved)
        {
            return Err(NativeInputRecipientError::InactiveWindow);
        }
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

    /// Query only the already accepted live native UI binding; this is the
    /// source/scope/epoch witness for an Engine World mutation transaction.
    /// Never infer input ownership from paint, mount order or window focus.
    pub(crate) fn selected_slot_binding(
        &self,
        slot: UiRuntimeSlotId,
    ) -> Option<(NativeWindowId, u64, InputArbitrationScope)> {
        self.bindings
            .iter()
            .find_map(|(window, binding)| match binding {
                WindowRecipientBinding::Bound(Binding {
                    recipient: NativeInputRecipient::RunenUiSlot(id),
                    epoch,
                    scope: Some(scope),
                }) if *id == slot => Some((*window, *epoch, scope.clone())),
                _ => None,
            })
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

    /// Physically identified native pointer admission MUST begin with the
    /// existing terminal completed-Present association for this exact native
    /// recipient. Merely published or GPU-submitted paint cannot supply the
    /// input context. The canonical pinned runtime still owns actual routing.
    pub(crate) fn submit_displayed_native_pointer(
        &self,
        origin: NativePointerOrigin,
        windows: &WindowStateRegistryResource,
        slots: &mut UiRuntimeSlotsResource,
        presentations: &UiRuntimePresentationAssociationsResource,
        render_surfaces: &RenderSurfaceRegistryResource,
        event: PointerEvent,
    ) -> Result<(UiRuntimeDisplayedPresentation, PointerSubmission), NativePointerIngressError>
    {
        let displayed = match self.displayed_pointer_input(
            origin.window(),
            windows,
            slots,
            presentations,
            render_surfaces,
        ) {
            Ok(Some(displayed)) => displayed,
            Ok(None) => return Err(NativePointerIngressError::LegacyRecipient(Box::new(event))),
            Err(reason) => {
                return Err(NativePointerIngressError::MissingDisplay {
                    reason,
                    event: Box::new(event),
                });
            }
        };
        if event.surface_context() != &displayed.input_context {
            return Err(NativePointerIngressError::MismatchedDisplay(Box::new(
                event,
            )));
        }
        if event.device_id() != Some(origin.device()) {
            return Err(NativePointerIngressError::MismatchedDevice(Box::new(event)));
        }
        let submission = slots
            .submit_native_pointer(displayed.slot_id, event)
            .map_err(NativePointerIngressError::Admission)?;
        Ok((displayed, submission))
    }

    /// Retire on native window destruction, target change or source loss.
    /// The *existing* completed-Present ledger also loses that slot's mapping;
    /// rebind must wait for a fresh terminal display rather than reviving an
    /// obsolete context from the previous window/source lifetime.
    pub(crate) fn retire_window(
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
        if window == NativeWindowId::primary() {
            self.requested_primary = None;
        }
        true
    }

    /// Retire a mounted runtime's recipient and its old displayed input.
    /// Preserve an explicit unbound marker until the surviving native window
    /// has a newly validated recipient or has itself been destroyed.
    pub(crate) fn retire_slot(
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
        if self.requested_primary == Some(NativeInputRecipient::RunenUiSlot(slot)) {
            self.requested_primary = None;
        }
        retired
    }

    /// Release a retired window's identity only after its native record is
    /// actually gone. This bounds registry state to real window lifetimes;
    /// retiring a still-live input source never silently grants gameplay.
    pub(crate) fn forget_destroyed_window(
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
                if window == NativeWindowId::primary() {
                    self.requested_primary = None;
                }
                Ok(true)
            }
            None => {
                if window == NativeWindowId::primary() {
                    self.requested_primary = None;
                }
                Ok(false)
            }
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
    use runenui_core::{
        LogicalLength, LogicalPoint, NoHostProtocol, PointerDeviceKind, PointerId, PointerPhase,
        StyleEnvironment, UiApp, View, text,
    };
    use runenui_runtime::{LogicalSize, RasterScale, SurfaceBuildContext};

    use super::*;
    use crate::plugins::render::RenderFrameProducerId;
    use crate::plugins::render::host::RenderSurfaceId;
    use crate::plugins::ui::{
        UiRuntimeFontConfiguration, UiRuntimeNativeMapping, UiRuntimePresentationBinding,
        UiTypedScreenId, native_pointer_finality,
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
    fn unattributed_raw_motion_cannot_treat_closing_ui_as_absent() {
        let (mut windows, legacy, ui_window) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = mount(&mut slots, 301);
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                legacy,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots,
            )
            .expect("ordinary legacy window");
        recipients
            .bind(
                ui_window,
                NativeInputRecipient::RunenUiSlot(slot),
                &windows,
                &mut slots,
            )
            .expect("independent UI window");
        assert!(!recipients.raw_motion_legacy_projection_allowed(&windows, false));

        windows
            .record_mut(ui_window)
            .expect("original UI window")
            .receive_close_intent();
        assert!(
            !recipients.raw_motion_legacy_projection_allowed(&windows, false),
            "closing UI window is not proof raw motion belongs to legacy game"
        );
        windows
            .record_mut(ui_window)
            .expect("original UI window")
            .approve_close();
        assert!(!recipients.raw_motion_legacy_projection_allowed(&windows, false));
    }

    #[test]
    fn approved_close_allows_only_original_scope_cancellation() {
        let (mut windows, primary, secondary) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = mount(&mut slots, 205);
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                primary,
                NativeInputRecipient::RunenUiSlot(slot),
                &windows,
                &mut slots,
            )
            .expect("original live M13 binding");
        let original_epoch = recipients
            .binding_epoch_for_window(primary)
            .expect("original binding epoch");
        windows
            .record_mut(primary)
            .expect("created window")
            .request_close();
        assert_eq!(
            recipients.resolve(primary, &windows, &mut slots),
            Err(NativeInputRecipientError::InactiveWindow),
            "approved close must deny normal input"
        );
        assert_eq!(
            recipients.resolve_for_cancellation(primary, &windows, &mut slots),
            Ok(NativeInputRecipient::RunenUiSlot(slot)),
            "original M13 scope remains available for terminal cancellation"
        );
        assert_eq!(
            recipients.binding_epoch_for_window(primary),
            Some(original_epoch)
        );
        assert_eq!(
            recipients.resolve_for_cancellation(secondary, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingRecipient),
            "a foreign window cannot borrow the closing recipient"
        );
        windows
            .record_mut(primary)
            .expect("closing window")
            .receive_close_intent();
        assert_eq!(
            recipients.resolve_for_cancellation(primary, &windows, &mut slots),
            Err(NativeInputRecipientError::InactiveWindow),
            "unapproved close intent cannot widen cancellation"
        );
        windows
            .record_mut(primary)
            .expect("pending window")
            .approve_close();
        crate::plugins::ui::native_transaction::test_selected_slot_operation(
            &mut slots,
            slot,
            |slots| slots.unmount_with_retirement(slot),
        )
        .expect("retire original framework scope");
        assert!(matches!(
            recipients.resolve_for_cancellation(primary, &windows, &mut slots),
            Err(NativeInputRecipientError::OwnershipUnavailable(
                UiRuntimeSlotOperationFailure::MissingSlot
            ))
        ));
    }

    #[test]
    fn declared_primary_native_recipient_requires_explicit_selection_and_created_window() {
        let (windows, primary, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let mut recipients = UiNativeInputRecipientsResource::default();
        assert!(
            !recipients
                .realize_requested_primary(&windows, &mut slots)
                .expect("no input binding is an intentional render-only state")
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
    fn retired_primary_input_cannot_resurrect_stale_application_selection() {
        let (windows, primary, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = mount(&mut slots, 72);
        let mut recipients = UiNativeInputRecipientsResource::default();
        let mut presentations = UiRuntimePresentationAssociationsResource::default();
        recipients
            .request_primary(NativeInputRecipient::RunenUiSlot(slot))
            .expect("initial explicit input authoring");
        assert_eq!(
            recipients.realize_requested_primary(&windows, &mut slots),
            Ok(true)
        );
        assert!(recipients.retire_window(primary, &mut presentations));
        assert_eq!(recipients.requested_primary, None);
        assert_eq!(
            recipients.realize_requested_primary(&windows, &mut slots),
            Ok(false),
            "retired input may not silently rebind from an old authoring request"
        );
        assert_eq!(
            recipients.resolve(primary, &windows, &mut slots),
            Err(NativeInputRecipientError::MissingRecipient)
        );
        recipients
            .request_primary(NativeInputRecipient::LegacyNativeConsumer)
            .expect("new explicit native target after retirement");
        assert_eq!(
            recipients.realize_requested_primary(&windows, &mut slots),
            Ok(true)
        );
        assert_eq!(
            recipients.resolve(primary, &windows, &mut slots),
            Ok(NativeInputRecipient::LegacyNativeConsumer)
        );
    }

    #[test]
    fn retiring_an_unrealized_slot_clears_its_pending_primary_request() {
        let (windows, _, _) = windows();
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = mount(&mut slots, 73);
        let mut recipients = UiNativeInputRecipientsResource::default();
        let mut presentations = UiRuntimePresentationAssociationsResource::default();
        recipients
            .request_primary(NativeInputRecipient::RunenUiSlot(slot))
            .expect("pre-window explicit authoring");
        assert_eq!(recipients.retire_slot(slot, &mut presentations), 0);
        assert_eq!(recipients.requested_primary, None);
        assert_eq!(
            recipients.realize_requested_primary(&windows, &mut slots),
            Ok(false)
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
        crate::plugins::ui::native_transaction::test_selected_slot_operation(
            &mut slots,
            a,
            |slots| slots.unmount_with_retirement(a),
        )
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
        crate::plugins::ui::native_transaction::test_selected_slot_operation(
            &mut slots,
            slot,
            |slots| slots.unmount_with_retirement(slot),
        )
        .expect("retire slot");
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
        let publication = crate::plugins::ui::native_transaction::test_selected_slot_operation(
            &mut slots,
            slot,
            |slots| slots.publish_surface_for_slot(slot, &context),
        )
        .expect("real RunenUI runtime publication");
        let expected = publication.input_context().clone();
        let device = InputDeviceId::new(7).expect("actual checked native device");
        let other_device = InputDeviceId::new(8).expect("different physical device");
        let pointer = PointerId::new(1).expect("native mouse pointer identity");
        let native_pointer = |device_id| {
            PointerEvent::new(
                pointer,
                PointerDeviceKind::Mouse,
                PointerPhase::Move,
                LogicalPoint::new(12.0, 8.0).expect("finite native logical point"),
                expected.clone(),
            )
            .with_device_id(device_id)
        };
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
        assert!(
            matches!(
                recipients.submit_displayed_native_pointer(
                    NativePointerOrigin::new(first, device),
                    &windows,
                    &mut slots,
                    &presentations,
                    &render_surfaces,
                    native_pointer(device),
                ),
                Err(NativePointerIngressError::MissingDisplay {
                    reason: NativeInputRecipientError::NoCompletedPresentation,
                    ..
                })
            ),
            "submitted paint is not an input context until completed Present"
        );
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
        let wrong = recipients
            .submit_displayed_native_pointer(
                NativePointerOrigin::new(first, device),
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
                native_pointer(other_device),
            )
            .expect_err("a foreign original device cannot impersonate the native cursor");
        assert!(matches!(
            wrong,
            NativePointerIngressError::MismatchedDevice(_)
        ));
        assert_eq!(
            wrong.into_event().device_id(),
            Some(other_device),
            "even rejected device identity is preserved"
        );
        let ownership = slots
            .input_ownership(slot)
            .expect("fresh M13 scope must precede pointer ingress");
        let (checked_display, submission) = recipients
            .submit_displayed_native_pointer(
                NativePointerOrigin::new(first, device),
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
                native_pointer(device),
            )
            .expect("only exact completed-native-display evidence can admit pointer input");
        assert_eq!(checked_display.input_context, expected);
        assert_eq!(
            recipients.resolve(first, &windows, &mut slots),
            Err(NativeInputRecipientError::OwnershipUnavailable(
                UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce
            )),
            "a genuinely admitted, not-yet-pumped M13 event must be visible as unsettled to the native recipient resolver"
        );
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .expect("one actual pinned ordered M13 pointer result");
        assert_eq!(status, Ok(()));
        assert!(
            native_pointer_finality(
                ownership.scope(),
                device,
                pointer,
                expected.surface_id(),
                submission,
                &batch,
                &status,
            )
            .is_ok(),
            "original device/stream/display and M13 sequence must all agree"
        );
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
        let next = crate::plugins::ui::native_transaction::test_selected_slot_operation(
            &mut slots,
            slot,
            |slots| slots.publish_surface_for_slot(slot, &changed_context),
        )
        .expect("new RunenUI publication");
        let new_context = next.input_context().clone();
        assert_ne!(
            new_context, expected,
            "DPI/extent remap must create a different surface input context"
        );
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

        let stale = recipients
            .submit_displayed_native_pointer(
                NativePointerOrigin::new(first, device),
                &windows,
                &mut slots,
                &presentations,
                &render_surfaces,
                native_pointer(device),
            )
            .expect_err("old completed-Present context must not reach the next hit scene");
        assert!(matches!(
            stale,
            NativePointerIngressError::MismatchedDisplay(_)
        ));
        assert_eq!(
            stale.into_event().surface_context(),
            &expected,
            "rejection retains the exact old original displayed input context"
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
