use crate::app::WindowedAppState;
use crate::plugins::render::host::native as render_native_host;
use crate::plugins::ui::{
    NativeInputRecipient, NativeInputRecipientError, NativePointerOrigin,
    UiNativeInputRecipientsResource, UiNativePointerHostRead, UiRuntimeDisplayedPresentation,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotId, UiRuntimeSlotsResource,
    native_keyboard_finality, native_no_focused_keyboard_claim_witness, native_pointer_finality,
};
use crate::plugins::{ActionState, InputState};
use crate::runtime::PrimaryPresentationMetricsResource;
use crate::runtime::frame_lifecycle::{run_frame as run_runtime_frame, run_startup_if_needed};
use crate::runtime::frame_pacing::{
    FramePacingPolicyResource, FramePacingRuntimeStateResource, FramePacingSchedule,
};
use crate::runtime::native_window_hooks::with_native_window_hooks;
use crate::runtime::platform::{
    PlatformEvent, PlatformWindowEvent, PlatformWindowEventQueueResource,
    apply_native_window_event, apply_platform_input_event,
};
use crate::runtime::presentation::ensure_primary_presentation_metrics;
use crate::runtime::window::{
    NativeWindowCreationRequest, NativeWindowId, WindowStateRegistryResource,
};
use crate::runtime::winit_input::{
    WinitInputAdapter, contact_input, cursor_position, keyboard_input, pointer_button_input,
    scroll_input, text_input,
};
use crate::runtime::winit_window_realizer::WinitWindowRealizer;
use crate::runtime::{NativeHostEvent, NativeWakeTransport};
use anyhow::{Context, Result, anyhow};
use runen_input::{ContinuityLoss, InputContext, InputSourceId, PhysicalKeyIdentity};
use runenui_core::{
    InputDeviceId as UiInputDeviceId, KeyModifiers, KeyboardCompositionState, LogicalPoint,
};
use runenui_runtime::{UiInputConflict, UiInputFinality};
use runenui_winit::keyboard_input::{
    KeyboardInputOutcome, KeyboardInputState, NativeKeyTransition,
};
use runenui_winit::mouse_input::{MouseInputState, TranslatedPointerPoint};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Use the same native winit modifier normalization as the pinned adapter's
/// reference host; the modifier callback does not synthesize a physical Down.
fn translate_native_ui_modifiers(state: winit::keyboard::ModifiersState) -> KeyModifiers {
    let mut result = KeyModifiers::NONE;
    if state.shift_key() {
        result = result.with_shift();
    }
    if state.control_key() {
        result = result.with_control();
    }
    if state.alt_key() {
        result = result.with_alt();
    }
    if state.super_key() {
        result = result.with_meta();
    }
    result
}

/// A pinned winit translator must NEVER cross its native window, physical
/// source or selected M13 recipient lifetime. No UI focus or gameplay owner
/// is replicated here; only the adapter's transient pressed-key state.
struct NativeUiKeyboardTranslator {
    recipient: UiRuntimeSlotId,
    source: InputSourceId,
    device: UiInputDeviceId,
    binding_epoch: u64,
    keyboard: KeyboardInputState,
}

/// Native mouse translation is transient. It never owns pointer gameplay
/// permission; native window, physical source, binding epoch and M13 scope
/// must agree before its active pointer stream may be reused.
struct NativeUiMouseTranslator {
    recipient: UiRuntimeSlotId,
    source: InputSourceId,
    backend_device: winit::event::DeviceId,
    device: UiInputDeviceId,
    binding_epoch: u64,
    scope: runenui_runtime::InputArbitrationScope,
    mouse: MouseInputState,
}

/// Translate with the mapping proven by the terminal completed Present, not
/// the newest UI publication or the requested framebuffer dimensions.
#[allow(
    clippy::cast_possible_truncation,
    reason = "native f64 cursor positions are finite and checked against the f32 range before conversion"
)]
fn native_logical_mouse_point(position: PhysicalPosition<f64>, scale: f64) -> Result<LogicalPoint> {
    anyhow::ensure!(scale.is_finite() && scale > 0.0, "invalid native scale");
    let x = position.x / scale;
    let y = position.y / scale;
    anyhow::ensure!(
        x.is_finite()
            && y.is_finite()
            && x >= f64::from(f32::MIN)
            && x <= f64::from(f32::MAX)
            && y >= f64::from(f32::MIN)
            && y <= f64::from(f32::MAX),
        "native cursor cannot be represented as a RunenUI logical point"
    );
    LogicalPoint::new(x as f32, y as f32)
        .context("checked native logical pointer position rejected")
}

fn displayed_native_mouse_point(
    displayed: &UiRuntimeDisplayedPresentation,
    position: PhysicalPosition<f64>,
    modifiers: KeyModifiers,
) -> Result<TranslatedPointerPoint> {
    Ok(TranslatedPointerPoint {
        position: native_logical_mouse_point(position, displayed.mapping.native_scale_factor())?,
        input_context: displayed.input_context.clone(),
        modifiers,
    })
}

pub(crate) fn run(mut state: WindowedAppState) -> Result<()> {
    install_native_window_provider_resources(&mut state.world);

    let mut event_loop_builder = EventLoop::<NativeHostEvent>::with_user_event();
    with_native_window_hooks(&mut state.world, |registry, _world| {
        registry.configure_event_loop(&mut event_loop_builder);
    });
    let event_loop = match event_loop_builder.build() {
        Ok(event_loop) => event_loop,
        Err(error) => {
            let cleanup = shutdown_native_ui(&mut state.world);
            return combine_execution_and_cleanup(Err(error.into()), cleanup);
        }
    };
    let transport = NativeWakeTransport::new(event_loop.create_proxy());
    if state.world.has_resource::<UiRuntimeSlotsResource>() {
        state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .attach_native_transport(transport.clone());
    }
    state.world.insert_resource(transport);
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut runner = WinitRunner {
        state,
        window_realizer: WinitWindowRealizer::default(),
        input_adapter: WinitInputAdapter::default(),
        native_ui_keyboards: HashMap::new(),
        native_ui_mice: HashMap::new(),
        native_ui_modifiers: HashMap::new(),
        last_primary_redraw_at: None,
        frame_pacing_schedule: FramePacingSchedule::default(),
        fatal_error: None,
        frame_requested: false,
        owned_redraws: HashSet::new(),
        shutdown_done: false,
    };
    let execution = event_loop.run_app(&mut runner).map_err(anyhow::Error::from);
    let execution = match (runner.fatal_error.take(), execution) {
        (Some(error), Err(loop_error)) => {
            Err(error.context(format!("native event loop also failed: {loop_error:#}")))
        }
        (Some(error), Ok(())) => Err(error),
        (None, execution) => execution,
    };
    let cleanup = runner.shutdown_native();
    combine_execution_and_cleanup(execution, cleanup)
}

fn combine_execution_and_cleanup(execution: Result<()>, cleanup: Result<()>) -> Result<()> {
    match (execution, cleanup) {
        (Err(error), Err(cleanup)) => {
            Err(error.context(format!("native cleanup also failed: {cleanup:#}")))
        }
        (Err(error), _) | (_, Err(error)) => Err(error),
        _ => Ok(()),
    }
}

fn shutdown_native_ui(world: &mut runen_ecs::World) -> Result<()> {
    if world.has_resource::<UiRuntimeSlotsResource>() {
        world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .shutdown_native_slots()?;
    }
    Ok(())
}

fn install_native_window_provider_resources(world: &mut runen_ecs::World) {
    ensure_primary_presentation_metrics(world);
    if !world.has_resource::<WindowStateRegistryResource>() {
        world.insert_resource(WindowStateRegistryResource::default());
    }
    if !world.has_resource::<PlatformWindowEventQueueResource>() {
        world.insert_resource(PlatformWindowEventQueueResource::default());
    }
}

struct WinitRunner {
    state: WindowedAppState,
    window_realizer: WinitWindowRealizer,
    input_adapter: WinitInputAdapter,
    /// Pinned native translator state; input ownership still lives in RunenUI.
    native_ui_keyboards: HashMap<NativeWindowId, NativeUiKeyboardTranslator>,
    native_ui_mice: HashMap<NativeWindowId, NativeUiMouseTranslator>,
    native_ui_modifiers: HashMap<NativeWindowId, KeyModifiers>,
    last_primary_redraw_at: Option<Instant>,
    frame_pacing_schedule: FramePacingSchedule,
    fatal_error: Option<anyhow::Error>,
    frame_requested: bool,
    /// Acknowledges host-requested redraws without advancing the frame twice.
    owned_redraws: HashSet<NativeWindowId>,
    shutdown_done: bool,
}

impl WinitRunner {
    fn sync_window_state(&mut self, window: &Window) -> Result<()> {
        let (size_px, scale_factor) = {
            let registry = self
                .state
                .world
                .resource_mut::<WindowStateRegistryResource>()
                .context("native Host window registry is unavailable")?;
            self.window_realizer
                .project_primary_window(registry, window)
        };
        self.sync_primary_presentation_and_surface_extent(size_px, scale_factor)
    }

    fn apply_event(&mut self, event: PlatformEvent) -> Result<()> {
        self.apply_event_for_native_window(NativeWindowId::primary(), event)
    }

    fn apply_window_focus_event(
        &mut self,
        native_window_id: NativeWindowId,
        focused: bool,
    ) -> Result<()> {
        if !focused
            && let Some(context) = self.input_adapter.window_source_context(native_window_id)
        {
            // The focus transition is window lifecycle truth; the associated
            // continuity loss is source-specific semantic input and retains
            // the original recipient snapshot before the canonical group is
            // applied. It cannot be reinterpreted by Editor after rebinding.
            let selection = self.native_input_recipient(native_window_id);
            let recipient = selection.as_ref().ok().map(|(recipient, _)| *recipient);
            // Focus/physical continuity is NOT contingent on a healthy UI
            // recipient. Journal admission remains atomic; a stale target
            // still fails, but only AFTER the original loss retracted input.
            self.apply_event_group_for_recipient(
                native_window_id,
                [
                    PlatformEvent::Focused { focused },
                    PlatformEvent::InputContinuityLost {
                        context,
                        loss: ContinuityLoss::Source,
                    },
                ],
                recipient,
            )?;
            return match selection {
                Ok(_) => Ok(()),
                Err(error)
                    if error.downcast_ref::<NativeInputRecipientError>()
                        == Some(&NativeInputRecipientError::MissingRecipient) =>
                {
                    Ok(())
                }
                Err(error) => Err(error),
            };
        }
        self.apply_event_for_native_window(native_window_id, PlatformEvent::Focused { focused })
    }

    fn retire_window_input_source(&mut self, native_window_id: NativeWindowId) -> Result<bool> {
        // Source retirement is a mandatory native pointer-stream boundary,
        // even when an original window callback is no longer forthcoming.
        self.cancel_native_ui_mouse(native_window_id);
        let Some(context) = self.input_adapter.window_source_context(native_window_id) else {
            return Ok(false);
        };
        let selection = self.native_input_recipient(native_window_id);
        let recipient = selection.as_ref().ok().map(|(recipient, _)| *recipient);
        // A stale UI recipient must not strand the old physical source in
        // the host allocator or leave its held gameplay contributors live.
        // Journal admission failure still aborts BEFORE retiring the source.
        self.apply_event_group_for_recipient(
            native_window_id,
            [PlatformEvent::InputContinuityLost {
                context,
                loss: ContinuityLoss::Source,
            }],
            recipient,
        )?;
        let retired = self
            .input_adapter
            .retire_window_source(native_window_id)
            .expect("known window input source must remain mapped until retirement");
        debug_assert_eq!(retired, context);
        self.native_ui_keyboards.remove(&native_window_id);
        self.native_ui_modifiers.remove(&native_window_id);
        if let Some(mut actions) = self.state.world.remove_resource::<ActionState>() {
            let result = self
                .state
                .world
                .resource::<InputState>()
                .context("missing neutral input after native source retirement")
                .map(|input| actions.retire_native_keyboard_source(input, context.source));
            self.state.world.insert_resource(actions);
            result?;
        }
        match selection {
            Ok(_) => Ok(true),
            Err(error)
                if error.downcast_ref::<NativeInputRecipientError>()
                    == Some(&NativeInputRecipientError::MissingRecipient) =>
            {
                Ok(true)
            }
            Err(error) => Err(error),
        }
    }

    /// Retire the semantic input recipient and its completed-display mapping
    /// BEFORE native/renderer destruction. The Engine's existing UI resources
    /// remain the sole authorities; no parallel host recipient cache exists.
    fn retire_native_input_recipient(&mut self, window: NativeWindowId) -> Result<bool> {
        // Cancel against the previous pinned M13 scope before removing the
        // recipient and its terminal displayed input authority.
        self.cancel_native_ui_mouse(window);
        let world = &mut self.state.world;
        if !world.has_resource::<UiNativeInputRecipientsResource>() {
            return Ok(false);
        }
        if !world.has_resource::<UiRuntimePresentationAssociationsResource>() {
            return Err(anyhow!(
                "native input recipients exist without the UI presentation authority"
            ));
        }
        // World resource access is exclusive on the invoker thread. Move one
        // registry temporarily rather than bypassing typed resource borrowing
        // or cloning either authority.
        let mut recipients = world
            .remove_resource::<UiNativeInputRecipientsResource>()
            .context("native input recipient resource disappeared")?;
        let retired = world
            .resource_mut::<UiRuntimePresentationAssociationsResource>()
            .map(|presentations| recipients.retire_window(window, presentations));
        world.insert_resource(recipients);
        let retired = retired.context("native UI presentation resource disappeared")?;
        if retired {
            self.native_ui_keyboards.remove(&window);
            self.native_ui_modifiers.remove(&window);
        }
        Ok(retired)
    }

    /// Forget only after the native window record is actually destroyed.
    /// A still-bound recipient is rejected by the owning registry; retirement
    /// and display invalidation may never be skipped just to reclaim storage.
    fn forget_destroyed_native_input_recipient(&mut self, window: NativeWindowId) -> Result<bool> {
        let world = &mut self.state.world;
        let Some(mut recipients) = world.remove_resource::<UiNativeInputRecipientsResource>()
        else {
            return Ok(false);
        };
        let result = world
            .resource::<WindowStateRegistryResource>()
            .context("native Host window registry unavailable during recipient teardown")
            .and_then(|windows| {
                recipients
                    .forget_destroyed_window(window, windows)
                    .map_err(anyhow::Error::from)
            });
        world.insert_resource(recipients);
        result
    }

    /// Resolve the original native window's one semantic input recipient before
    /// routing. Absence is legacy only when NO UI runtime is mounted; a stale
    /// or retired UI binding never becomes a game input fallback.
    /// Materialize only the author-declared primary input target once the
    /// window is real and startup has installed all mounted UI resources.
    /// No presentation order, UI mount count or implicit z-order is consulted.
    fn realize_declared_primary_input(&mut self) -> Result<()> {
        let world = &mut self.state.world;
        let Some(mut recipients) = world.remove_resource::<UiNativeInputRecipientsResource>()
        else {
            return Ok(());
        };
        let result = (|| -> Result<()> {
            let mut slots = world
                .remove_resource::<UiRuntimeSlotsResource>()
                .context("declared native UI recipient has no slot authority")?;
            let operation = (|| -> Result<()> {
                let windows = world.resource::<WindowStateRegistryResource>()?;
                recipients.realize_requested_primary(windows, &mut slots)?;
                Ok(())
            })();
            world.insert_resource(slots);
            operation
        })();
        world.insert_resource(recipients);
        result
    }

    fn native_input_recipient(
        &mut self,
        window: NativeWindowId,
    ) -> Result<(NativeInputRecipient, Option<u64>)> {
        let world = &mut self.state.world;
        let Some(mut slots) = world.remove_resource::<UiRuntimeSlotsResource>() else {
            if world.has_resource::<UiNativeInputRecipientsResource>() {
                return Err(anyhow!("native recipients exist without UI slot authority"));
            }
            return Ok((NativeInputRecipient::LegacyNativeConsumer, None));
        };
        // Closure scope guarantees slot restoration on *every* selection
        // failure, including missing window authority and stale M13 scope.
        let selected = (|| -> Result<(NativeInputRecipient, Option<u64>)> {
            if world.has_resource::<UiNativeInputRecipientsResource>() {
                let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
                let windows = world
                    .resource::<WindowStateRegistryResource>()
                    .context("missing native window registry")?;
                let recipient = recipients.resolve(window, windows, &mut slots)?;
                let epoch = recipients.binding_epoch_for_window(window);
                if matches!(recipient, NativeInputRecipient::RunenUiSlot(_)) && epoch.is_none() {
                    return Err(anyhow!(
                        "resolved native UI recipient has no binding lifetime"
                    ));
                }
                Ok((recipient, epoch))
            } else if slots.is_empty() {
                Ok((NativeInputRecipient::LegacyNativeConsumer, None))
            } else {
                Err(NativeInputRecipientError::MissingRecipient.into())
            }
        })();
        world.insert_resource(slots);
        selected
    }

    fn native_ui_keyboard_translator(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        source: InputSourceId,
        device: UiInputDeviceId,
        binding_epoch: u64,
    ) -> &mut KeyboardInputState {
        let replaced = self
            .native_ui_keyboards
            .get(&window)
            .is_some_and(|existing| {
                existing.recipient != slot
                    || existing.source != source
                    || existing.device != device
                    || existing.binding_epoch != binding_epoch
            });
        if replaced {
            // No key/modifier stream from the previous binding may become
            // a pressed continuation under the replacement physical source.
            self.native_ui_keyboards.remove(&window);
            self.native_ui_modifiers.remove(&window);
        }
        &mut self
            .native_ui_keyboards
            .entry(window)
            .or_insert_with(|| NativeUiKeyboardTranslator {
                recipient: slot,
                source,
                device,
                binding_epoch,
                keyboard: KeyboardInputState::default(),
            })
            .keyboard
    }

    fn native_ui_device(context: InputContext) -> Result<UiInputDeviceId> {
        let device = context
            .device
            .context("original native keyboard event lacks device identity")?;
        UiInputDeviceId::new(device.raw()).context("native keyboard device identity is zero")
    }

    /// An original keyboard observation is FIRST admitted to the one neutral
    /// physical reducer. The explicitly bound UI host then invokes the pinned
    /// translation and synchronously correlates the real M13 input settlement.
    /// No separate input dispatcher, focus cache or mutable UI permission store.
    fn route_ui_keyboard(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        context: InputContext,
        original: &winit::event::KeyEvent,
        is_synthetic: bool,
    ) -> Result<()> {
        // Same original event and normalized source; no alternate input owner.
        let physical_key = keyboard_input(original, is_synthetic).physical_key;
        let device = Self::native_ui_device(context)?;
        // Revalidate the translator's actual recipient and physical-source
        // generation BEFORE reusing cached modifier or pressed-key evidence.
        self.native_ui_keyboard_translator(window, slot, context.source, device, binding_epoch);
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let transition = NativeKeyTransition::from_event(original, is_synthetic);
        let translated = self
            .native_ui_keyboard_translator(window, slot, context.source, device, binding_epoch)
            .key_input(
                device,
                &transition,
                modifiers,
                KeyboardCompositionState::Inactive,
            );

        let permit = match translated {
            KeyboardInputOutcome::Submit(event) => {
                let slots = self
                    .state
                    .world
                    .resource_mut::<UiRuntimeSlotsResource>()
                    .context("selected UI runtime vanished from native host")?;
                match slots.input_ownership(slot) {
                    Ok(before) => match slots.submit_native_keyboard(slot, event) {
                        Ok(submission) => match slots.pump_native_input_records(slot) {
                            Ok((batch, status)) => matches!(
                                native_keyboard_finality(
                                    before.scope(),
                                    device,
                                    submission,
                                    &batch,
                                    &status,
                                ),
                                Ok(UiInputFinality::Committed(ref facts))
                                    if matches!(
                                        facts.conflict(),
                                        UiInputConflict::Unclaimed
                                            | UiInputConflict::ObservedNonexclusive
                                    )
                            ),
                            Err(_) => false,
                        },
                        Err(error) => {
                            let after = slots.input_ownership(slot);
                            after.is_ok_and(|after| {
                                native_no_focused_keyboard_claim_witness(
                                    device, &before, &after, &error,
                                )
                            })
                        }
                    },
                    // No owner-authored pre-state means no input mutation and
                    // no game grant. A later observation cannot be fabricated
                    // from a successful framework queue submission.
                    Err(_) => false,
                }
            }
            KeyboardInputOutcome::Suppressed(_) => false,
        };

        // Missing or ambiguous M13 evidence, runtime failure, UI exclusivity,
        // and translator suppression NEVER produce positive gameplay evidence.
        // Revocation happens before PreUpdate/CoreSet::Input.
        if !permit && matches!(original.state, winit::event::ElementState::Pressed) {
            self.deny_native_keyboard_lifetime(context, &physical_key)?;
        }
        Ok(())
    }

    /// Deny only the actual admitted physical Down. A failed recipient lookup
    /// must not erase RunenInput truth or create an unverified action grant.
    fn deny_native_keyboard_lifetime(
        &mut self,
        context: InputContext,
        physical_key: &PhysicalKeyIdentity,
    ) -> Result<()> {
        let generation = self
            .state
            .world
            .resource::<InputState>()?
            .keyboard_down_generation_in(context, physical_key);
        if let Some(generation) = generation
            && let Some(mut actions) = self.state.world.remove_resource::<ActionState>()
        {
            let result = self.state.world.resource::<InputState>().map(|input| {
                actions.revoke_keyboard_lifetime(input, context, physical_key, generation);
            });
            self.state.world.insert_resource(actions);
            result?;
        }
        Ok(())
    }

    /// Focus loss is a lifetime boundary for native translator pressed state.
    /// Deliver framework-owned cancellation; synthetic cancellation is NOT an
    /// authentic neutral physical Up and never resurrects game permissions.
    fn cancel_native_ui_keyboard(
        &mut self,
        window: NativeWindowId,
        slot: crate::plugins::ui::UiRuntimeSlotId,
    ) -> Result<()> {
        self.native_ui_modifiers.remove(&window);
        let Some(mut lifetime) = self.native_ui_keyboards.remove(&window) else {
            return Ok(());
        };
        if lifetime.recipient != slot {
            return Ok(());
        }
        // A synthetic cancellation may mutate only the original still-bound
        // recipient lifetime. Rebind/unmount/scope changes must not receive
        // held keys from a previous native binding.
        if !matches!(
            self.native_input_recipient(window),
            Ok((NativeInputRecipient::RunenUiSlot(active), Some(epoch)))
                if active == slot && epoch == lifetime.binding_epoch
        ) {
            return Ok(());
        }
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("UI slot authority missing during native keyboard cancellation")?;
        for event in lifetime
            .keyboard
            .cancel_all(KeyModifiers::NONE, KeyboardCompositionState::Inactive)
        {
            if slots.submit_native_keyboard(slot, event).is_ok() {
                // Cancellation does not grant gameplay, even if settlement is
                // paused, rejected, or unavailable.
                let _ = slots.pump_native_input_records(slot);
            }
        }
        Ok(())
    }

    fn apply_raw_device_removal(&mut self, device_id: winit::event::DeviceId) -> Result<bool> {
        // Winit window events and DeviceEvents have separately allocated
        // RunenInput identities. They share only the original backend DeviceId.
        // Cancel the live UI pointer while its original M13 scope still exists.
        let windows: Vec<_> = self
            .native_ui_mice
            .iter()
            .filter_map(|(window, lifetime)| {
                (lifetime.backend_device == device_id).then_some(*window)
            })
            .collect();
        for window in windows.iter().copied() {
            self.cancel_native_ui_mouse(window);
        }
        let window_contexts = self.input_adapter.retire_window_device(device_id)?;
        let raw_context = self.input_adapter.retire_raw_device(device_id)?;
        // Device removal also retires the pinned keyboard translator lifetime.
        // The native window/source can remain alive; a new hardware generation
        // must never reuse cached pressed keys, modifiers or M13 keyboard state.
        if let Some(retired_device) = window_contexts.first().and_then(|context| context.device) {
            let keyboard_targets: Vec<_> = self
                .native_ui_keyboards
                .iter()
                .filter_map(|(window, lifetime)| {
                    (lifetime.device.get() == retired_device.raw())
                        .then_some((*window, lifetime.recipient))
                })
                .collect();
            for (window, recipient) in keyboard_targets {
                if let Err(error) = self.cancel_native_ui_keyboard(window, recipient) {
                    // Even a stale UI slot must not stop the original neutral
                    // device loss and ActionState retirement below.
                    tracing::warn!(
                        target: "engine.runtime.input",
                        ?window,
                        ?error,
                        "native keyboard cancellation failed during device removal"
                    );
                }
            }
        }
        if window_contexts.is_empty() && raw_context.is_none() {
            return Ok(!windows.is_empty());
        }
        {
            let input = self
                .state
                .world
                .resource_mut::<InputState>()
                .context("missing InputState resource")?;
            for context in window_contexts {
                input.handle_continuity_loss(context, ContinuityLoss::Device);
            }
            if let Some(context) = raw_context {
                input.handle_continuity_loss(context, ContinuityLoss::Source);
            }
        }
        // Original device removal is an immediate action-lifetime boundary,
        // not a permission change deferred to the next input system frame.
        self.project_actions_after_native_continuity()?;
        Ok(true)
    }

    /// Retire only the pinned M13 mouse stream in its original live scope.
    /// A cancellation is never interpreted as an original physical Up.
    fn cancel_native_ui_mouse(&mut self, window: NativeWindowId) {
        let Some(mut lifetime) = self.native_ui_mice.remove(&window) else {
            return;
        };
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let Some(cancel) = lifetime.mouse.invalidate_point_authority(modifiers) else {
            return;
        };
        let Ok(slots) = self.state.world.resource_mut::<UiRuntimeSlotsResource>() else {
            return;
        };
        if !slots
            .input_ownership(lifetime.recipient)
            .is_ok_and(|observed| observed.scope() == &lifetime.scope)
        {
            return;
        }
        if slots
            .submit_native_pointer(lifetime.recipient, cancel)
            .is_ok()
        {
            let _ = slots.pump_native_input_records(lifetime.recipient);
        }
    }

    /// Only original neutral-admitted cursor movement and a current terminal
    /// displayed pointer context may advance the pinned RunenUI pointer stream.
    fn route_ui_cursor_moved(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        backend_device: winit::event::DeviceId,
        context: InputContext,
        position: PhysicalPosition<f64>,
    ) -> Result<()> {
        let device = Self::native_ui_device(context)?;
        if self.native_ui_mice.get(&window).is_some_and(|old| {
            old.recipient != slot
                || old.binding_epoch != binding_epoch
                || old.source != context.source
                || old.backend_device != backend_device
                || old.device != device
        }) {
            self.cancel_native_ui_mouse(window);
        }
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let world = &mut self.state.world;
        let mut slots = world
            .remove_resource::<UiRuntimeSlotsResource>()
            .context("selected UI pointer has no slot authority")?;
        let operation = (|| -> Result<()> {
            let before = slots
                .input_ownership(slot)
                .map_err(|error| anyhow!("native UI pointer pre-state unavailable: {error:?}"))?;
            let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
            let host = UiNativePointerHostRead::from_world(world)?;
            let displayed = host
                .displayed_input(recipients, window, &mut slots)?
                .context("no completed-Present UI pointer input context")?;
            let point = displayed_native_mouse_point(&displayed, position, modifiers)?;
            let mice = &mut self.native_ui_mice;
            if mice
                .get(&window)
                .is_some_and(|old| &old.scope != before.scope())
            {
                mice.remove(&window);
            }
            let lifetime = mice
                .entry(window)
                .or_insert_with(|| NativeUiMouseTranslator {
                    recipient: slot,
                    source: context.source,
                    backend_device,
                    device,
                    binding_epoch,
                    scope: before.scope().clone(),
                    mouse: MouseInputState::default(),
                });
            lifetime.mouse.note_cursor_position(position);
            let event = lifetime
                .mouse
                .cursor_moved(device, point)
                .map_err(|error| anyhow!("native UI mouse translation failed: {error:?}"))?;
            let pointer = event.pointer_id();
            let (_, submission) = host
                .submit(
                    recipients,
                    NativePointerOrigin::new(window, device),
                    &mut slots,
                    event,
                )
                .map_err(|error| anyhow!("native displayed pointer admission failed: {error:?}"))?;
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .map_err(|error| anyhow!("native UI pointer M13 pump unavailable: {error:?}"))?;
            let finality = native_pointer_finality(
                before.scope(),
                device,
                pointer,
                displayed.input_context.surface_id(),
                submission,
                &batch,
                &status,
            );
            anyhow::ensure!(
                finality.is_ok(),
                "native UI cursor lacked exact M13 pointer settlement: {finality:?}"
            );
            Ok(())
        })();
        world.insert_resource(slots);
        if let Err(error) = operation {
            self.cancel_native_ui_mouse(window);
            tracing::trace!(
                target: "engine.runtime.input",
                ?window,
                ?error,
                "native UI cursor withheld without gameplay grant"
            );
        }
        Ok(())
    }

    /// Preserve one original recipient for native pointer, wheel and touch
    /// callbacks too. Until pinned M13 pointer delivery is integrated, UI-owned
    /// input stays neutral physical evidence; it is NEVER dispatched to Editor
    /// merely because no translated UI pointer settlement exists yet.
    fn apply_semantic_native_input(
        &mut self,
        window: NativeWindowId,
        event: PlatformEvent,
    ) -> Result<()> {
        let selection = self.native_input_recipient(window);
        match selection {
            Ok((recipient, _)) => {
                self.apply_event_group_for_recipient(window, [event], Some(recipient))
            }
            Err(error) => {
                self.apply_event_group_for_recipient(window, [event], None)?;
                if error.downcast_ref::<NativeInputRecipientError>()
                    == Some(&NativeInputRecipientError::MissingRecipient)
                {
                    Ok(())
                } else {
                    Err(error)
                }
            }
        }
    }

    fn apply_event_for_native_window(
        &mut self,
        native_window_id: NativeWindowId,
        event: PlatformEvent,
    ) -> Result<()> {
        self.apply_event_group(native_window_id, [event])
    }

    fn apply_event_group<const N: usize>(
        &mut self,
        native_window_id: NativeWindowId,
        events: [PlatformEvent; N],
    ) -> Result<()> {
        self.apply_event_group_for_recipient(
            native_window_id,
            events,
            Some(NativeInputRecipient::LegacyNativeConsumer),
        )
    }

    fn apply_event_group_for_recipient<const N: usize>(
        &mut self,
        native_window_id: NativeWindowId,
        events: [PlatformEvent; N],
        recipient: Option<NativeInputRecipient>,
    ) -> Result<()> {
        // Journal capacity is checked for the entire observed callback before
        // any physical state or native record receives its members.
        self.state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()?
            .reserve_native_group(&events)?;
        let is_input = |event: &PlatformEvent| {
            matches!(
                event,
                PlatformEvent::KeyboardInput { .. }
                    | PlatformEvent::InputContinuityLost { .. }
                    | PlatformEvent::TextInput { .. }
                    | PlatformEvent::MouseWheel { .. }
                    | PlatformEvent::CursorMoved { .. }
                    | PlatformEvent::MouseInput { .. }
                    | PlatformEvent::Touch { .. }
            )
        };
        if events.iter().any(is_input) {
            self.state
                .world
                .resource::<InputState>()
                .context("missing InputState resource")?;
        }
        if events.iter().any(|event| !is_input(event)) {
            self.state
                .world
                .resource::<WindowStateRegistryResource>()?
                .record(native_window_id)
                .context("native Host event targets an unknown window")?;
        }
        if native_window_id == NativeWindowId::primary()
            && events.iter().any(|event| {
                matches!(
                    event,
                    PlatformEvent::Resized { .. } | PlatformEvent::ScaleFactorChanged { .. }
                )
            })
        {
            self.state
                .world
                .resource::<PrimaryPresentationMetricsResource>()?;
        }
        // Move owned payloads once. Admission does not require infallible String
        // clones after the explicit allocation/capacity check.
        let journal = events.map(|event| {
            PlatformWindowEvent::new(native_window_id, event).with_native_input_recipient(recipient)
        });
        for entry in &journal {
            self.apply_admitted_event(native_window_id, &entry.event)?;
        }
        self.state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .context("native Host platform-window event queue is unavailable")?
            .publish_group(journal)?;
        Ok(())
    }

    fn apply_admitted_event(
        &mut self,
        native_window_id: NativeWindowId,
        event: &PlatformEvent,
    ) -> Result<()> {
        match event {
            PlatformEvent::KeyboardInput { .. }
            | PlatformEvent::InputContinuityLost { .. }
            | PlatformEvent::TextInput { .. }
            | PlatformEvent::MouseWheel { .. }
            | PlatformEvent::CursorMoved { .. }
            | PlatformEvent::MouseInput { .. }
            | PlatformEvent::Touch { .. } => {
                {
                    let input = self
                        .state
                        .world
                        .resource_mut::<InputState>()
                        .context("missing InputState resource")?;
                    apply_platform_input_event(input, event);
                }
                if matches!(event, PlatformEvent::InputContinuityLost { .. }) {
                    // Focus loss/source invalidation can arrive after the ECS
                    // Input set projected actions. Retract those invalidated
                    // presses and held keys in this SAME native callback, not
                    // at some later PreUpdate or newly invented input queue.
                    self.project_actions_after_native_continuity()?;
                }
            }
            PlatformEvent::Resumed
            | PlatformEvent::CloseRequested
            | PlatformEvent::Focused { .. }
            | PlatformEvent::Resized { .. }
            | PlatformEvent::ScaleFactorChanged { .. }
            | PlatformEvent::RedrawRequested => {
                let projected_primary_metrics = {
                    let registry = self
                        .state
                        .world
                        .resource_mut::<WindowStateRegistryResource>()
                        .context("native Host window registry is unavailable")?;
                    let record = registry
                        .record_mut(native_window_id)
                        .context("native Host event targets an unknown window")?;
                    apply_native_window_event(record, event);
                    if native_window_id == NativeWindowId::primary()
                        && matches!(
                            event,
                            PlatformEvent::Resized { .. }
                                | PlatformEvent::ScaleFactorChanged { .. }
                        )
                    {
                        Some((record.size_px, record.scale_factor))
                    } else {
                        None
                    }
                };
                if let Some((size_px, scale_factor)) = projected_primary_metrics {
                    self.sync_primary_presentation_and_surface_extent(size_px, scale_factor)?;
                }
            }
        }
        Ok(())
    }

    fn project_actions_after_native_continuity(&mut self) -> Result<()> {
        let world = &mut self.state.world;
        let Some(mut actions) = world.remove_resource::<ActionState>() else {
            // Non-game UI/Editor consumers may not install ActionState.
            return Ok(());
        };
        let projection = world
            .resource::<InputState>()
            .context("missing normalized InputState for continuity projection")
            .map(|input| actions.project(input));
        world.insert_resource(actions);
        projection
    }

    fn apply_raw_mouse_motion(&mut self, context: InputContext, dx: f32, dy: f32) -> Result<()> {
        let input = self
            .state
            .world
            .resource_mut::<InputState>()
            .context("missing InputState resource")?;
        input.handle_relative_motion(context, dx, dy);
        Ok(())
    }

    fn sync_primary_presentation_and_surface_extent(
        &mut self,
        size_px: (u32, u32),
        scale_factor: f64,
    ) -> Result<()> {
        self.state
            .world
            .resource_mut::<PrimaryPresentationMetricsResource>()
            .context("missing primary presentation metrics")?
            .update(size_px, scale_factor);
        render_native_host::project_primary_extent(&mut self.state.world, size_px);
        Ok(())
    }

    fn run_startup_if_needed(&mut self) -> Result<()> {
        run_startup_if_needed(
            &mut self.state.world,
            &mut self.state.scheduler,
            &mut self.state.startup_ran,
        )
    }

    fn run_frame(&mut self) -> Result<()> {
        if let Some(window) = self.window_realizer.primary_window() {
            with_native_window_hooks(&mut self.state.world, |registry, world| {
                registry.dispatch_frame(&window, world);
            });
        }
        if self.state.world.has_resource::<UiRuntimeSlotsResource>() {
            self.state
                .world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .settle_native_slots()?;
        }
        run_runtime_frame(&mut self.state.world, &mut self.state.scheduler)
    }

    fn shutdown_native(&mut self) -> Result<()> {
        if self.shutdown_done {
            return Ok(());
        }
        if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>() {
            transport.close();
        }
        let mouse_windows: Vec<_> = self.native_ui_mice.keys().copied().collect();
        for window in mouse_windows {
            self.cancel_native_ui_mouse(window);
        }
        let cleanup = shutdown_native_ui(&mut self.state.world);
        self.shutdown_done = cleanup.is_ok();
        cleanup
    }

    fn request_native_redraw(&mut self, id: NativeWindowId, window: &Window) {
        if self.owned_redraws.insert(id) {
            WinitWindowRealizer::request_redraw(window);
        }
    }

    fn acknowledge_native_redraw(&mut self, id: NativeWindowId) {
        if !self.owned_redraws.remove(&id) {
            self.frame_requested = true;
        }
    }

    fn attach_native_window_hooks(&mut self, window: &Window) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.attach_hooks(window, world);
        });
    }

    fn drain_pending_window_requests(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        let requests = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .ok()
            .map(|registry| registry.take_pending_creation_requests())
            .unwrap_or_default();
        for request in requests {
            self.create_window_for_request(event_loop, request)?;
        }
        Ok(())
    }

    fn create_window_for_request(
        &mut self,
        event_loop: &ActiveEventLoop,
        request: NativeWindowCreationRequest,
    ) -> Result<()> {
        let window = match self.window_realizer.realize_secondary(event_loop, &request) {
            Ok(window) => window,
            Err(err) => {
                self.mark_window_creation_failed(request.native_window_id, format!("{err:#}"));
                return Ok(());
            }
        };

        let realized_size = window.inner_size();
        let realized_size_px = (realized_size.width.max(1), realized_size.height.max(1));
        if let Err(err) = render_native_host::attach_secondary(
            &mut self.state.world,
            request.native_window_id,
            window.clone(),
            request.size_px,
            realized_size_px,
        ) {
            self.mark_window_creation_failed(request.native_window_id, format!("{err:#}"));
            return Ok(());
        }

        let logical_commit = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .context("native Host window registry is unavailable")
            .map(|registry| {
                self.window_realizer.project_created_window(
                    registry,
                    request.native_window_id,
                    &window,
                );
            });
        if let Err(err) = logical_commit {
            render_native_host::rollback_secondary(&mut self.state.world, request.native_window_id);
            return Err(err);
        }

        self.attach_native_window_hooks(&window);
        self.request_native_redraw(request.native_window_id, &window);
        self.window_realizer
            .publish_window(request.native_window_id, window);
        Ok(())
    }

    fn mark_window_creation_failed(
        &mut self,
        native_window_id: NativeWindowId,
        reason: impl Into<String>,
    ) {
        render_native_host::rollback_secondary(&mut self.state.world, native_window_id);
        if let Ok(registry) = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
        {
            self.window_realizer
                .mark_creation_failed(registry, native_window_id, reason);
        }
    }

    fn dispatch_native_window_event(&mut self, window: &Window, event: &WindowEvent) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.dispatch_window_event(window, event, world);
        });
    }

    fn dispatch_native_device_event(&mut self, event: &DeviceEvent) {
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.dispatch_device_event(event, world);
        });
    }

    fn apply_window_effects(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        self.drain_pending_window_requests(event_loop)?;

        let windows = self.window_realizer.realized_windows();
        for (window_id, native_window_id, window) in windows {
            let record = self
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .ok()
                .and_then(|registry| registry.record(native_window_id).cloned());
            let Some(record) = record else {
                continue;
            };

            WinitWindowRealizer::apply_physical_record(&window, &record);

            if record.close_requested {
                if native_window_id == NativeWindowId::primary() {
                    self.retire_native_input_recipient(native_window_id)?;
                    // Windows winit must not enter a blocking wait after exit.
                    event_loop.set_control_flow(ControlFlow::Poll);
                    event_loop.exit();
                    return Ok(());
                }
                self.retire_native_input_recipient(native_window_id)?;
                self.retire_window_input_source(native_window_id)?;
                render_native_host::retire_secondary(&mut self.state.world, native_window_id);
                if let Ok(window_registry) = self
                    .state
                    .world
                    .resource_mut::<WindowStateRegistryResource>()
                {
                    window_registry.remove_window(native_window_id);
                }
                self.forget_destroyed_native_input_recipient(native_window_id)?;
                self.window_realizer.remove_window(window_id);
                self.owned_redraws.remove(&native_window_id);
                continue;
            }

            if record.redraw_requested {
                let request_now = native_window_id != NativeWindowId::primary()
                    || self.should_request_primary_redraw_now(Instant::now());
                if request_now {
                    self.request_native_redraw(native_window_id, &window);
                    if let Ok(registry) = self
                        .state
                        .world
                        .resource_mut::<WindowStateRegistryResource>()
                        && let Some(record) = registry.record_mut(native_window_id)
                    {
                        record.redraw_requested = false;
                    }
                }
            }
        }

        Ok(())
    }

    fn should_request_primary_redraw_now(&mut self, now: Instant) -> bool {
        let policy = self.frame_pacing_policy();
        let decision = self.frame_pacing_schedule.decide(policy, now);
        policy.target_frame_interval().is_none() || decision.request_redraw
    }

    fn frame_pacing_policy(&self) -> FramePacingPolicyResource {
        self.state
            .world
            .resource::<FramePacingPolicyResource>()
            .ok()
            .copied()
            .unwrap_or_default()
    }

    fn observe_frame_pacing_decision(
        &mut self,
        policy: FramePacingPolicyResource,
        now: Instant,
        next_deadline: Option<Instant>,
        redraw_requested: bool,
    ) {
        if let Ok(state) = self
            .state
            .world
            .resource_mut::<FramePacingRuntimeStateResource>()
        {
            state.observe_policy(policy);
            state.observe_next_deadline(now, next_deadline);
            state.observe_redraw_requested(redraw_requested);
        }
    }

    fn request_redraw_for_native_window(&mut self, native_window_id: NativeWindowId) {
        self.frame_requested = true;
        if let Ok(registry) = self
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            && let Some(record) = registry.record_mut(native_window_id)
        {
            record.request_redraw();
        }
        if let Ok(state) = self
            .state
            .world
            .resource_mut::<FramePacingRuntimeStateResource>()
        {
            state.observe_redraw_requested(true);
        }
    }

    fn observe_primary_frame_started(&mut self) {
        let now = Instant::now();
        if let Some(previous) = self.last_primary_redraw_at
            && let Ok(state) = self
                .state
                .world
                .resource_mut::<FramePacingRuntimeStateResource>()
        {
            state.observe_frame_interval(now.saturating_duration_since(previous));
        }
        self.last_primary_redraw_at = Some(now);
    }

    fn frame_is_ready(&self, now: Instant) -> bool {
        self.last_primary_redraw_at
            .zip(self.frame_pacing_policy().target_frame_interval())
            .is_none_or(|(previous, interval)| now.saturating_duration_since(previous) >= interval)
    }

    fn ui_readiness(&self) -> Result<(bool, Option<Instant>)> {
        if self.state.world.has_resource::<UiRuntimeSlotsResource>() {
            self.state
                .world
                .resource::<UiRuntimeSlotsResource>()?
                .native_readiness()
        } else {
            Ok((false, None))
        }
    }

    fn apply_frame_pacing(&mut self, event_loop: &ActiveEventLoop) -> Result<()> {
        if event_loop.exiting() {
            return Ok(());
        }
        if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>() {
            transport.ensure_delivery()?;
        }
        let now = Instant::now();
        let policy = self.frame_pacing_policy();
        let decision = self.frame_pacing_schedule.decide(policy, now);
        let (unfinished, ui_deadline) = self.ui_readiness()?;
        if unfinished
            || ui_deadline.is_some_and(|deadline| deadline <= now)
            || decision.request_redraw
        {
            self.frame_requested = true;
        }
        if decision.request_redraw
            && let Some(window) = self.window_realizer.primary_window()
        {
            self.request_native_redraw(NativeWindowId::primary(), &window);
        }
        let capped_deadline = if self.frame_requested && !self.frame_is_ready(now) {
            self.last_primary_redraw_at
                .zip(policy.target_frame_interval())
                .and_then(|(last, interval)| last.checked_add(interval))
        } else {
            None
        };
        // Due work is represented by the pending frame bit. Never schedule a
        // past WaitUntil that would spin while waiting for a continuous cap.
        let next_deadline = [
            decision.next_deadline,
            ui_deadline.filter(|deadline| *deadline > now),
            capped_deadline,
        ]
        .into_iter()
        .flatten()
        .min();
        match next_deadline {
            Some(deadline) => event_loop.set_control_flow(ControlFlow::WaitUntil(deadline)),
            None => event_loop.set_control_flow(ControlFlow::Wait),
        }
        self.observe_frame_pacing_decision(policy, now, next_deadline, decision.request_redraw);
        Ok(())
    }

    fn pace_or_exit(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self.apply_frame_pacing(event_loop) {
            self.exit_with_error(event_loop, error);
        }
    }

    fn exit_with_error(&mut self, event_loop: &ActiveEventLoop, err: anyhow::Error) {
        tracing::error!(error = %format!("{err:#}"), "runtime windowed execution failed");
        self.fatal_error = Some(err);
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.exit();
    }
}

impl Drop for WinitRunner {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown_native() {
            tracing::error!(%error, "native UI destructor cleanup failed");
        }
        with_native_window_hooks(&mut self.state.world, |registry, world| {
            registry.detach_hooks(world);
        });
    }
}

impl ApplicationHandler<NativeHostEvent> for WinitRunner {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, _event: NativeHostEvent) {
        if event_loop.exiting() {
            return;
        }
        if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>() {
            transport.acknowledge();
        }
        self.frame_requested = true;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_realizer.has_primary_window() {
            return;
        }

        let window = match self
            .window_realizer
            .realize_primary(event_loop, &self.state)
        {
            Ok(window) => window,
            Err(err) => {
                self.exit_with_error(event_loop, err);
                return;
            }
        };

        if let Err(err) = self.sync_window_state(&window) {
            self.exit_with_error(
                event_loop,
                anyhow!("failed to sync initial window state: {err:#}"),
            );
            return;
        }

        let size = window.inner_size();
        if let Err(err) = render_native_host::initialize_primary(
            &mut self.state.world,
            window.clone(),
            (size.width, size.height),
        ) {
            self.exit_with_error(event_loop, err);
            return;
        }

        if let Err(err) = self.apply_event(PlatformEvent::Resumed) {
            self.exit_with_error(event_loop, anyhow!("failed to apply resume event: {err:#}"));
            return;
        }

        self.attach_native_window_hooks(&window);

        if let Err(err) = self.run_startup_if_needed() {
            self.exit_with_error(event_loop, anyhow!("runtime startup failed: {err:#}"));
            return;
        }
        if let Err(err) = self.realize_declared_primary_input() {
            self.exit_with_error(
                event_loop,
                anyhow!("declared native primary input target invalid: {err:#}"),
            );
            return;
        }

        self.frame_requested = true;
        self.request_native_redraw(NativeWindowId::primary(), &window);
        self.window_realizer
            .publish_window(NativeWindowId::primary(), window);
        self.pace_or_exit(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if event_loop.exiting() {
            return;
        }
        let Some(window) = self.window_realizer.window(window_id) else {
            tracing::debug!(
                target = "engine.runtime.window",
                ?window_id,
                "ignoring event for unknown native window"
            );
            return;
        };
        let Some(native_window_id) = self.window_realizer.native_window_id(window_id) else {
            tracing::debug!(
                target = "engine.runtime.window",
                ?window_id,
                "ignoring event for unregistered native window"
            );
            return;
        };

        self.dispatch_native_window_event(&window, &event);
        if !matches!(event, WindowEvent::RedrawRequested) {
            self.frame_requested = true;
        }

        let result: Result<()> = (|| match event {
            WindowEvent::CloseRequested => {
                self.apply_event_for_native_window(native_window_id, PlatformEvent::CloseRequested)
            }
            WindowEvent::Focused(focused) => {
                // UI cancellation is best-effort only for ordering: even a
                // terminal/stale M13 target cannot intercept the operating
                // system's original focus-loss and physical-key revocation.
                if !focused {
                    self.cancel_native_ui_mouse(native_window_id);
                }
                let cancellation = if !focused {
                    match self.native_input_recipient(native_window_id) {
                        Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                            self.cancel_native_ui_keyboard(native_window_id, slot)
                        }
                        Ok((NativeInputRecipient::LegacyNativeConsumer, _)) => Ok(()),
                        Err(error)
                            if error.downcast_ref::<NativeInputRecipientError>()
                                == Some(&NativeInputRecipientError::MissingRecipient) =>
                        {
                            Ok(())
                        }
                        Err(error) => Err(error),
                    }
                } else {
                    Ok(())
                };
                self.apply_window_focus_event(native_window_id, focused)?;
                cancellation
            }
            WindowEvent::Resized(size) => {
                self.cancel_native_ui_mouse(native_window_id);
                self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::Resized {
                        width: size.width,
                        height: size.height,
                    },
                )
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.cancel_native_ui_mouse(native_window_id);
                let size = window.inner_size();
                self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::ScaleFactorChanged {
                        scale_factor,
                        width: size.width,
                        height: size.height,
                    },
                )
            }
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                // Even a stale/missing recipient must not suppress original
                // physical truth. Admit it before reporting the UI route error,
                // and fail gameplay closed for that exact lifetime.
                let recipient = self.native_input_recipient(native_window_id);
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let input = keyboard_input(&event, is_synthetic);
                let physical_key = input.physical_key.clone();
                let result = match recipient {
                    Ok((NativeInputRecipient::LegacyNativeConsumer, _)) => {
                        match text_input(&event, is_synthetic) {
                            Some(text) => self.apply_event_group(
                                native_window_id,
                                [
                                    PlatformEvent::KeyboardInput { context, input },
                                    PlatformEvent::TextInput { text },
                                ],
                            ),
                            None => self.apply_event_for_native_window(
                                native_window_id,
                                PlatformEvent::KeyboardInput { context, input },
                            ),
                        }
                    }
                    Ok((NativeInputRecipient::RunenUiSlot(slot), epoch)) => {
                        self.apply_event_group_for_recipient(
                            native_window_id,
                            [PlatformEvent::KeyboardInput { context, input }],
                            Some(NativeInputRecipient::RunenUiSlot(slot)),
                        )?;
                        self.route_ui_keyboard(
                            native_window_id,
                            slot,
                            epoch.context("selected native UI slot has no binding epoch")?,
                            context,
                            &event,
                            is_synthetic,
                        )
                    }
                    Err(error) => {
                        // Preserve neutral physical truth but select NO semantic
                        // recipient. No implicit Editor/game fallback is possible.
                        self.apply_event_group_for_recipient(
                            native_window_id,
                            [PlatformEvent::KeyboardInput { context, input }],
                            None,
                        )?;
                        if matches!(event.state, winit::event::ElementState::Pressed) {
                            self.deny_native_keyboard_lifetime(context, &physical_key)?;
                        }
                        if error.downcast_ref::<NativeInputRecipientError>()
                            == Some(&NativeInputRecipientError::MissingRecipient)
                        {
                            // A passive UI runtime may coexist with rendering
                            // without an input target. Do not kill its window.
                            tracing::trace!(
                                target: "engine.runtime.input",
                                ?native_window_id,
                                "unassigned native UI input retained as physical truth"
                            );
                            Ok(())
                        } else {
                            Err(error)
                        }
                    }
                };
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.native_ui_modifiers.insert(
                    native_window_id,
                    translate_native_ui_modifiers(modifiers.state()),
                );
                Ok(())
            }
            WindowEvent::MouseWheel {
                device_id,
                delta,
                phase,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let input = scroll_input(delta, phase);
                let result = self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::MouseWheel { context, input },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let original_position = position;
                let neutral_point = cursor_position(position);
                self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::CursorMoved {
                        context,
                        position: neutral_point,
                    },
                )?;
                if let Ok((NativeInputRecipient::RunenUiSlot(slot), Some(epoch))) =
                    self.native_input_recipient(native_window_id)
                {
                    self.route_ui_cursor_moved(
                        native_window_id,
                        slot,
                        epoch,
                        device_id,
                        context,
                        original_position,
                    )?;
                }
                self.request_redraw_for_native_window(native_window_id);
                Ok(())
            }
            WindowEvent::CursorLeft { .. } => {
                self.cancel_native_ui_mouse(native_window_id);
                Ok(())
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let input = pointer_button_input(state, button);
                let result = self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::MouseInput { context, input },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::Touch(touch) => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, touch.device_id)?;
                let input = contact_input(touch);
                let result = self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::Touch { context, input },
                );
                if result.is_ok() {
                    self.request_redraw_for_native_window(native_window_id);
                }
                result
            }
            WindowEvent::RedrawRequested => {
                self.acknowledge_native_redraw(native_window_id);
                self.apply_event_for_native_window(native_window_id, PlatformEvent::RedrawRequested)
            }
            _ => Ok(()),
        })();

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime window event failed: {err:#}"));
            return;
        }
        self.pace_or_exit(event_loop);
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.dispatch_native_device_event(&event);

        let result: Result<()> = (|| match event {
            DeviceEvent::MouseMotion { delta } => {
                let context = self.input_adapter.raw_device_context(device_id)?;
                let result = self.apply_raw_mouse_motion(context, delta.0 as f32, delta.1 as f32);
                if result.is_ok() {
                    self.request_redraw_for_native_window(NativeWindowId::primary());
                }
                result
            }
            DeviceEvent::Removed => {
                let result = self.apply_raw_device_removal(device_id);
                if result.as_ref().is_ok_and(|removed| *removed) {
                    self.request_redraw_for_native_window(NativeWindowId::primary());
                }
                result.map(|_| ())
            }
            _ => Ok(()),
        })();

        if let Err(err) = result.and_then(|_| self.apply_window_effects(event_loop)) {
            self.exit_with_error(event_loop, anyhow!("runtime device event failed: {err:#}"));
            return;
        }
        self.pace_or_exit(event_loop);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if event_loop.exiting() {
            return;
        }
        self.pace_or_exit(event_loop);
        if event_loop.exiting() {
            return;
        }
        if self.frame_requested
            && self.window_realizer.has_primary_window()
            && self.frame_is_ready(Instant::now())
        {
            self.frame_requested = false;
            self.observe_primary_frame_started();
            if let Err(error) = self
                .run_frame()
                .and_then(|_| self.apply_window_effects(event_loop))
            {
                self.exit_with_error(event_loop, anyhow!("runtime frame failed: {error:#}"));
                return;
            }
            self.pace_or_exit(event_loop);
            if !event_loop.exiting() && self.frame_requested && self.frame_is_ready(Instant::now())
            {
                // A deadline or bounded continuation can become due during the
                // frame. Supply another native turn rather than strand it in Wait.
                if let Ok(transport) = self.state.world.resource::<NativeWakeTransport>()
                    && let Err(error) = transport.notify()
                {
                    self.exit_with_error(event_loop, error.into());
                }
            }
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Err(error) = self.shutdown_native() {
            self.fatal_error = Some(match self.fatal_error.take() {
                Some(previous) => {
                    previous.context(format!("native cleanup also failed: {error:#}"))
                }
                None => error,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, AppNativeHostExt};
    use crate::plugins::{FixedStepPlugin, SimulationPlugin, TimePlugin};
    use crate::runtime::fixed_time::{
        CatchupBudget, FixedTimeConfig, FixedTimeState, SimulationTick,
    };
    use crate::runtime::schedules::{FixedUpdate, PreUpdate};
    use crate::runtime::{CoreSet, Res, ResMut, SystemConfigExt};

    #[derive(Debug, Default, runen_ecs::Component, runen_ecs::Resource)]
    struct FixedTickLog(Vec<u64>);

    fn configure_probe(app: &mut App) {
        app.add_plugins((TimePlugin, FixedStepPlugin, SimulationPlugin));
        app.init_resource::<FixedTickLog>();
        app.insert_resource(FixedTimeConfig {
            step_seconds: 1.0 / 60.0,
        });
        app.insert_resource(CatchupBudget {
            max_steps_per_frame: 4,
        });
        app.add_systems(PreUpdate, set_frame_delta.after(CoreSet::Time));
        app.add_systems(FixedUpdate, log_tick);
    }

    fn runner_with_frame_pacing(policy: FramePacingPolicyResource) -> WinitRunner {
        let mut app = App::new();
        app.with_frame_pacing(policy);
        install_native_window_provider_resources(app.world_mut());
        app.world_mut()
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native Host test fixture should install window registry")
            .register_primary_window("test primary", (1280, 720), 1.0, true);
        WinitRunner {
            state: app.into_windowed_state(),
            window_realizer: WinitWindowRealizer::default(),
            input_adapter: WinitInputAdapter::default(),
            native_ui_keyboards: HashMap::new(),
            native_ui_mice: HashMap::new(),
            native_ui_modifiers: HashMap::new(),
            last_primary_redraw_at: None,
            frame_pacing_schedule: FramePacingSchedule::default(),
            fatal_error: None,
            frame_requested: false,
            owned_redraws: HashSet::new(),
            shutdown_done: false,
        }
    }

    fn set_frame_delta(mut time: ResMut<crate::plugins::time::domain::Time>) {
        time.delta_seconds = 0.05;
    }

    fn log_tick(tick: Res<SimulationTick>, mut log: ResMut<FixedTickLog>) {
        log.0.push(tick.0);
    }

    #[test]
    fn native_mouse_coordinate_normalization_requires_displayed_scale_and_finite_range() {
        let physical = PhysicalPosition::new(240.0, 120.0);
        let logical = native_logical_mouse_point(physical, 2.0)
            .expect("native HiDPI mapping is representable");
        assert_eq!(logical.x(), 120.0);
        assert_eq!(logical.y(), 60.0);
        assert!(native_logical_mouse_point(physical, 0.0).is_err());
        assert!(native_logical_mouse_point(physical, -1.0).is_err());
        assert!(native_logical_mouse_point(physical, f64::INFINITY).is_err());
        assert!(native_logical_mouse_point(PhysicalPosition::new(f64::NAN, 0.0), 1.0).is_err());
        assert!(native_logical_mouse_point(PhysicalPosition::new(f64::MAX, 0.0), 1.0).is_err());
    }

    #[test]
    fn headless_and_windowed_paths_share_fixed_step_semantics() {
        let mut headless = App::headless();
        configure_probe(&mut headless);
        headless
            .prepare_for_run()
            .expect("headless startup should run");
        headless.run_frame().expect("headless frame should run");

        let headless_log = headless
            .world()
            .resource::<FixedTickLog>()
            .expect("headless log resource should exist")
            .0
            .clone();
        let headless_tick = headless
            .world()
            .resource::<SimulationTick>()
            .expect("headless tick should exist")
            .0;
        let headless_fixed = *headless
            .world()
            .resource::<FixedTimeState>()
            .expect("headless fixed state should exist");

        let mut windowed = App::new();
        configure_probe(&mut windowed);
        let mut runner = WinitRunner {
            state: windowed.into_windowed_state(),
            window_realizer: WinitWindowRealizer::default(),
            input_adapter: WinitInputAdapter::default(),
            native_ui_keyboards: HashMap::new(),
            native_ui_mice: HashMap::new(),
            native_ui_modifiers: HashMap::new(),
            last_primary_redraw_at: None,
            frame_pacing_schedule: FramePacingSchedule::default(),
            fatal_error: None,
            frame_requested: false,
            owned_redraws: HashSet::new(),
            shutdown_done: false,
        };
        runner
            .run_startup_if_needed()
            .expect("windowed startup should run");
        runner.run_frame().expect("windowed frame should run");

        let windowed_log = runner
            .state
            .world
            .resource::<FixedTickLog>()
            .expect("windowed log resource should exist")
            .0
            .clone();
        let windowed_tick = runner
            .state
            .world
            .resource::<SimulationTick>()
            .expect("windowed tick should exist")
            .0;
        let windowed_fixed = *runner
            .state
            .world
            .resource::<FixedTimeState>()
            .expect("windowed fixed state should exist");

        assert_eq!(headless_log, vec![1, 2, 3]);
        assert_eq!(windowed_log, vec![1, 2, 3]);
        assert_eq!(headless_log, windowed_log);
        assert_eq!(headless_tick, windowed_tick);
        assert_eq!(headless_fixed.steps_ran_last_frame, 3);
        assert_eq!(windowed_fixed.steps_ran_last_frame, 3);
        assert_eq!(headless_fixed.saturated_frames, 0);
        assert_eq!(windowed_fixed.saturated_frames, 0);
    }

    fn normalized_shift_press(context: InputContext) -> PlatformEvent {
        PlatformEvent::KeyboardInput {
            context,
            input: runen_input::KeyboardInput {
                physical_key: runen_input::PhysicalKeyIdentity::code("ShiftLeft"),
                logical_key: runen_input::LogicalKey::Native(
                    runen_input::NativeLogicalKey::Unidentified,
                ),
                location: runen_input::KeyLocation::Left,
                state: runen_input::DigitalState::Pressed,
                repeat: false,
                origin: runen_input::ObservationOrigin::SourceReport,
            },
        }
    }

    #[test]
    fn focus_loss_invalidates_existing_window_source_without_retiring_its_identity() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let backend_device = winit::event::DeviceId::dummy();
        let context = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), backend_device)
            .unwrap();
        runner
            .apply_event(normalized_shift_press(context))
            .expect("normalized key press should apply");
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );

        runner
            .apply_window_focus_event(NativeWindowId::primary(), false)
            .expect("focus loss should apply");

        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );
        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary())
                .map(|value| value.source),
            Some(context.source)
        );
    }

    #[test]
    fn stale_native_recipient_cannot_prevent_original_focus_and_source_retirement() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let primary = NativeWindowId::primary();
        let context = runner
            .input_adapter
            .window_context(primary, winit::event::DeviceId::dummy())
            .expect("one original window input source");
        runner
            .apply_event(normalized_shift_press(context))
            .expect("original physical Down admitted");
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );

        // A recipient registry without mounted UI slots is inconsistent and
        // must be reported, but never prevents physical focus loss.
        runner
            .state
            .world
            .insert_resource(UiNativeInputRecipientsResource::default());
        assert!(
            runner.apply_window_focus_event(primary, false).is_err(),
            "stale semantic route remains observable as an error"
        );
        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .expect("physical reducer")
                .shift_down(),
            "original focus loss must retract held physical input"
        );
        assert!(
            !runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .expect("native windows")
                .record(primary)
                .expect("primary native window")
                .focused,
            "native window focus truth is still updated"
        );
        let journal = runner
            .state
            .world
            .resource::<PlatformWindowEventQueueResource>()
            .expect("one original journal");
        assert!(
            !journal
                .events()
                .last()
                .expect("source continuity observation retained")
                .authorized_for_legacy_editor(),
            "stale-target continuity cannot be dispatched as Editor-owned input"
        );

        let mut teardown = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        teardown.state.world.insert_resource(InputState::new());
        let context = teardown
            .input_adapter
            .window_context(primary, winit::event::DeviceId::dummy())
            .expect("native teardown source");
        teardown
            .apply_event(normalized_shift_press(context))
            .expect("teardown original Down");
        teardown
            .state
            .world
            .insert_resource(UiNativeInputRecipientsResource::default());
        assert!(
            teardown.retire_window_input_source(primary).is_err(),
            "unresolvable recipient is still reported"
        );
        assert!(
            teardown
                .input_adapter
                .window_source_context(primary)
                .is_none(),
            "original physical source identity must be retired despite stale UI route"
        );
        assert!(
            !teardown
                .state
                .world
                .resource::<InputState>()
                .expect("neutral input")
                .shift_down(),
            "source retirement must retract the held key"
        );
    }

    #[test]
    fn focus_loss_does_not_allocate_an_unused_window_source() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());

        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary()),
            None
        );
        runner
            .apply_window_focus_event(NativeWindowId::primary(), false)
            .expect("focus loss should apply");
        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary()),
            None
        );
    }

    #[test]
    fn explicit_window_source_retirement_invalidates_state_and_refreshes_source_identity() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let secondary =
            NativeWindowId::try_from_raw(2).expect("secondary native window id should be valid");
        let backend_device = winit::event::DeviceId::dummy();
        let context = runner
            .input_adapter
            .window_context(secondary, backend_device)
            .unwrap();
        runner
            .apply_event_for_native_window(secondary, normalized_shift_press(context))
            .expect("secondary key press should apply");

        assert!(
            runner
                .retire_window_input_source(secondary)
                .expect("window source retirement should apply")
        );
        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .expect("input state")
                .shift_down()
        );
        assert_eq!(runner.input_adapter.window_source_context(secondary), None);

        let replacement = runner
            .input_adapter
            .window_context(secondary, backend_device)
            .unwrap();
        assert_ne!(replacement.source, context.source);
        assert_eq!(replacement.device, context.device);
    }

    #[test]
    fn declared_native_primary_binding_uses_created_window_and_restores_authorities() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner
            .realize_declared_primary_input()
            .expect("ordinary game-only host has no implicit UI binding");

        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .request_primary(NativeInputRecipient::LegacyNativeConsumer)
            .expect("explicit application owner declaration");
        runner.state.world.insert_resource(recipients);
        assert!(
            runner.realize_declared_primary_input().is_err(),
            "declared recipient cannot silently fabricate missing slot authority"
        );
        assert!(
            runner
                .state
                .world
                .has_resource::<UiNativeInputRecipientsResource>(),
            "failed native bootstrap must restore its exact recipient registry"
        );

        runner
            .state
            .world
            .insert_resource(UiRuntimeSlotsResource::default());
        runner
            .realize_declared_primary_input()
            .expect("created primary window can realize the original owner");
        let (selected, epoch) = runner
            .native_input_recipient(NativeWindowId::primary())
            .expect("realized original native owner");
        assert_eq!(selected, NativeInputRecipient::LegacyNativeConsumer);
        let epoch = epoch.expect("explicit native recipient has a lifetime identity");
        runner
            .realize_declared_primary_input()
            .expect("repeated native bootstrap is idempotent");
        assert_eq!(
            runner
                .native_input_recipient(NativeWindowId::primary())
                .expect("same original native owner")
                .1,
            Some(epoch),
        );
    }

    #[test]
    fn native_callback_selects_exact_recipient_without_implicit_game_fallback() {
        use crate::plugins::ui::{
            NativeInputRecipient, UiRuntimeFontConfiguration, UiRuntimeSlotsResource,
            UiTypedScreenId,
        };

        struct NativeInputApp;
        impl runenui_core::UiApp for NativeInputApp {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("native input routing fixture")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let secondary = NativeWindowId::try_from_raw(2).expect("secondary identity");
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native window authority")
            .register_created_window(secondary, "secondary", (800, 600), 1.0, false);

        assert_eq!(
            runner
                .native_input_recipient(secondary)
                .expect("game-only baseline"),
            (NativeInputRecipient::LegacyNativeConsumer, None),
        );

        // The original native journal records the actual semantic recipient
        // for pointer callbacks before Editor drains the same authority.
        fn cursor_observation() -> PlatformEvent {
            PlatformEvent::CursorMoved {
                context: InputContext::new(
                    InputSourceId::new(91),
                    Some(runen_input::InputDeviceId::new(5)),
                ),
                position: runen_input::Point2::new(
                    24.0,
                    12.0,
                    runen_input::CoordinateSpace::WindowPhysicalPixels,
                ),
            }
        }
        runner.state.world.insert_resource(InputState::new());
        runner
            .apply_semantic_native_input(secondary, cursor_observation())
            .expect("legacy input routed normally");
        assert!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .expect("original journal")
                .events()
                .last()
                .expect("admitted legacy cursor")
                .authorized_for_legacy_editor()
        );

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<NativeInputApp>(
                UiTypedScreenId::new("native.input.exact"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("real framework slot");
        runner.state.world.insert_resource(slots);
        assert!(
            runner.native_input_recipient(secondary).is_err(),
            "a mounted UI slot without explicit native binding cannot fall back to gameplay"
        );
        runner
            .apply_semantic_native_input(secondary, cursor_observation())
            .expect("passive UI leaves the native event loop running");
        assert!(
            !runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .expect("original journal")
                .events()
                .last()
                .expect("admitted unassigned cursor")
                .authorized_for_legacy_editor(),
            "unassigned pointer input is physical truth, not Editor input"
        );
        assert!(
            runner.state.world.has_resource::<UiRuntimeSlotsResource>(),
            "failed recipient selection must restore the exact live slot owner"
        );

        let mut slots = runner
            .state
            .world
            .remove_resource::<UiRuntimeSlotsResource>()
            .expect("same slot owner is available");
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                secondary,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .expect("created native window"),
                &mut slots,
            )
            .expect("exact active M13 recipient binding");
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        runner
            .state
            .world
            .insert_resource(UiRuntimePresentationAssociationsResource::default());
        let (selected, bound_epoch) = runner
            .native_input_recipient(secondary)
            .expect("selected UI recipient");
        assert_eq!(selected, NativeInputRecipient::RunenUiSlot(slot));
        let bound_epoch = bound_epoch.expect("selected UI recipient has exact binding lifetime");
        runner
            .apply_semantic_native_input(secondary, cursor_observation())
            .expect("bound UI pointer observed without legacy dispatch");
        assert!(
            !runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .expect("original journal")
                .events()
                .last()
                .expect("admitted UI-targeted cursor")
                .authorized_for_legacy_editor(),
            "selected RunenUI input cannot be double-dispatched to Editor"
        );
        // A device-free namespace change is still a hard translator
        // lifetime boundary; no previous UI pressed keys or modifiers survive.
        let source_a = InputSourceId::new(91);
        let source_b = InputSourceId::new(92);
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            source_a,
            UiInputDeviceId::new(7).expect("checked test device"),
            bound_epoch,
        );
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            source_b,
            UiInputDeviceId::new(7).expect("checked test device"),
            bound_epoch,
        );
        assert_eq!(
            runner
                .native_ui_keyboards
                .get(&secondary)
                .map(|state| state.source),
            Some(source_b)
        );
        assert!(
            !runner.native_ui_modifiers.contains_key(&secondary),
            "a replacement input source must not reuse old modifier evidence"
        );
        assert!(
            runner
                .retire_native_input_recipient(secondary)
                .expect("source retirement")
        );
        assert!(
            runner.native_input_recipient(secondary).is_err(),
            "retired UI scope cannot resurrect implicit game eligibility"
        );
        // Retire/rebind of the SAME window/slot/source starts a new exact
        // recipient lifetime. Its old keyboard translator must not survive.
        let mut slots = runner
            .state
            .world
            .remove_resource::<UiRuntimeSlotsResource>()
            .expect("original UI slot remains mounted");
        let mut recipients = runner
            .state
            .world
            .remove_resource::<UiNativeInputRecipientsResource>()
            .expect("original recipient registry");
        recipients
            .bind(
                secondary,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .expect("window remains created"),
                &mut slots,
            )
            .expect("explicit recipient rebind");
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        let (rebinding, new_epoch) = runner
            .native_input_recipient(secondary)
            .expect("same native window rebound");
        assert_eq!(rebinding, NativeInputRecipient::RunenUiSlot(slot));
        let new_epoch = new_epoch.expect("new exact binding lifetime");
        assert!(new_epoch > bound_epoch, "binding generations never alias");
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            source_b,
            UiInputDeviceId::new(7).expect("checked test device"),
            bound_epoch,
        );
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            source_b,
            UiInputDeviceId::new(7).expect("checked test device"),
            new_epoch,
        );
        assert!(
            !runner.native_ui_modifiers.contains_key(&secondary),
            "new recipient epoch cannot inherit prior physical modifier state"
        );
        // Nominal window/slot/source/epoch identity is not sufficient: a
        // reappearing hardware device has a distinct checked generation.
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        let replacement_device = UiInputDeviceId::new(8).expect("checked replacement device");
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            source_b,
            replacement_device,
            new_epoch,
        );
        assert!(
            !runner.native_ui_modifiers.contains_key(&secondary),
            "replaced hardware may not inherit modifiers from old device lifetime"
        );
        assert_eq!(
            runner
                .native_ui_keyboards
                .get(&secondary)
                .map(|state| state.device),
            Some(replacement_device)
        );

        // Backend device removal retracts that framework lifetime BEFORE
        // its neutral physical device generation is reallocated, without
        // destroying the native window's logical source.
        let backend = winit::event::DeviceId::dummy();
        let original = runner
            .input_adapter
            .window_context(secondary, backend)
            .expect("one checked backend device");
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            original.source,
            WinitRunner::native_ui_device(original).expect("checked UI device"),
            new_epoch,
        );
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        assert!(
            runner
                .apply_raw_device_removal(backend)
                .expect("original native backend device loss")
        );
        assert!(
            !runner.native_ui_keyboards.contains_key(&secondary),
            "removed device must cancel its M13 keyboard translator"
        );
        assert!(
            !runner.native_ui_modifiers.contains_key(&secondary),
            "removed device cannot leave pressed modifier evidence"
        );
        let successor = runner
            .input_adapter
            .window_context(secondary, backend)
            .expect("device can reappear with another checked generation");
        assert_eq!(successor.source, original.source);
        assert_ne!(successor.device, original.device);
        assert!(
            runner.state.world.has_resource::<UiRuntimeSlotsResource>(),
            "retirement rejection cannot discard the native UI runtime"
        );
    }

    #[test]
    fn native_window_destruction_retires_ui_recipient_before_forgetting_identity() {
        use crate::plugins::ui::{
            NativeInputRecipient, NativeInputRecipientError, UiRuntimeFontConfiguration,
            UiRuntimeSlotsResource, UiTypedScreenId,
        };

        struct NativeLifetimeApp;
        impl runenui_core::UiApp for NativeLifetimeApp {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("native recipient lifetime")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let secondary = NativeWindowId::try_from_raw(2).expect("secondary identity");
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native windows")
            .register_created_window(secondary, "secondary", (800, 600), 1.0, false);

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<NativeLifetimeApp>(
                UiTypedScreenId::new("native.recipient.teardown"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("real pinned RunenUI mount");
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                secondary,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .expect("native windows"),
                &mut slots,
            )
            .expect("live native binding");
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        runner
            .state
            .world
            .insert_resource(UiRuntimePresentationAssociationsResource::default());

        assert_eq!(
            runner
                .forget_destroyed_native_input_recipient(secondary)
                .expect_err("still-live native identity cannot be pruned")
                .to_string(),
            NativeInputRecipientError::InactiveWindow.to_string()
        );
        assert!(
            runner
                .retire_native_input_recipient(secondary)
                .expect("native retirement")
        );
        assert!(
            !runner
                .retire_native_input_recipient(secondary)
                .expect("idempotent retirement")
        );

        let mut slots = runner
            .state
            .world
            .remove_resource::<UiRuntimeSlotsResource>()
            .expect("mounted slot registry");
        assert_eq!(
            runner
                .state
                .world
                .resource::<UiNativeInputRecipientsResource>()
                .expect("recipient registry remains present")
                .resolve(
                    secondary,
                    runner
                        .state
                        .world
                        .resource::<WindowStateRegistryResource>()
                        .expect("native window still exists"),
                    &mut slots,
                ),
            Err(NativeInputRecipientError::MissingRecipient),
            "a retired live window cannot silently revert to gameplay"
        );
        runner.state.world.insert_resource(slots);

        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native windows")
            .remove_window(secondary);
        assert!(
            runner
                .forget_destroyed_native_input_recipient(secondary)
                .expect("retired destroyed window can be pruned")
        );
        assert!(
            !runner
                .forget_destroyed_native_input_recipient(secondary)
                .expect("pruning is idempotent")
        );
        assert!(
            runner
                .state
                .world
                .has_resource::<UiNativeInputRecipientsResource>()
        );
        assert!(
            runner
                .state
                .world
                .has_resource::<UiRuntimePresentationAssociationsResource>()
        );
    }

    #[test]
    fn native_focus_loss_retracts_game_actions_in_the_same_event_turn() {
        use crate::plugins::{ActionState, action};

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let source = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), winit::event::DeviceId::dummy())
            .expect("actual native window source");
        runner
            .apply_event(PlatformEvent::KeyboardInput {
                context: source,
                input: runen_input::KeyboardInput {
                    physical_key: runen_input::PhysicalKeyIdentity::code("KeyW"),
                    logical_key: runen_input::LogicalKey::Native(
                        runen_input::NativeLogicalKey::Unidentified,
                    ),
                    location: runen_input::KeyLocation::Standard,
                    state: runen_input::DigitalState::Pressed,
                    repeat: false,
                    origin: runen_input::ObservationOrigin::SourceReport,
                },
            })
            .expect("source Down admitted");
        let mut actions = runner
            .state
            .world
            .remove_resource::<ActionState>()
            .expect("ActionState");
        let input = runner
            .state
            .world
            .resource::<InputState>()
            .expect("neutral input");
        actions.project(input);
        assert!(actions.action_pressed(action::WORLD_MOVE_UP));
        assert!(actions.action_down(action::WORLD_MOVE_UP));
        runner.state.world.insert_resource(actions);

        runner
            .apply_window_focus_event(NativeWindowId::primary(), false)
            .expect("source continuity admitted");
        let actions = runner
            .state
            .world
            .resource::<ActionState>()
            .expect("same retained ActionState");
        assert!(
            !actions.action_pressed(action::WORLD_MOVE_UP)
                && !actions.action_down(action::WORLD_MOVE_UP),
            "no post-PreUpdate stale gameplay action survives native focus loss"
        );
        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary())
                .map(|context| context.source),
            Some(source.source),
            "focus loss is not physical source destruction"
        );
    }

    #[test]
    fn native_source_retirement_retracts_projected_game_action_evidence() {
        use crate::plugins::{ActionState, action};
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let source = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), winit::event::DeviceId::dummy())
            .expect("monotonic source and device identities");
        let key_press = PlatformEvent::KeyboardInput {
            context: source,
            input: runen_input::KeyboardInput {
                physical_key: runen_input::PhysicalKeyIdentity::code("KeyW"),
                logical_key: runen_input::LogicalKey::Native(
                    runen_input::NativeLogicalKey::Unidentified,
                ),
                location: runen_input::KeyLocation::Standard,
                state: runen_input::DigitalState::Pressed,
                repeat: false,
                origin: runen_input::ObservationOrigin::SourceReport,
            },
        };
        runner
            .apply_event(key_press)
            .expect("physical window Down admitted");
        let mut actions = runner
            .state
            .world
            .remove_resource::<ActionState>()
            .expect("game ActionState");
        let input = runner
            .state
            .world
            .resource::<InputState>()
            .expect("neutral input");
        actions.project(input);
        assert!(actions.action_pressed(action::WORLD_MOVE_UP));
        assert!(actions.action_down(action::WORLD_MOVE_UP));
        runner.state.world.insert_resource(actions);

        let retired = runner
            .retire_window_input_source(NativeWindowId::primary())
            .expect("real window source retirement");
        assert!(retired);
        let actions = runner
            .state
            .world
            .resource::<ActionState>()
            .expect("retained ActionState resource");
        assert!(!actions.action_pressed(action::WORLD_MOVE_UP));
        assert!(!actions.action_down(action::WORLD_MOVE_UP));
        assert_eq!(
            runner
                .input_adapter
                .window_source_context(NativeWindowId::primary()),
            None
        );
    }

    #[test]
    fn original_device_removal_retires_window_and_raw_devices_without_destroying_window_source() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let backend_device = winit::event::DeviceId::dummy();
        let window = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), backend_device)
            .unwrap();
        let raw = runner
            .input_adapter
            .raw_device_context(backend_device)
            .unwrap();

        assert!(
            runner
                .apply_raw_device_removal(backend_device)
                .expect("raw device removal should apply")
        );
        let replacement = runner
            .input_adapter
            .raw_device_context(backend_device)
            .unwrap();
        assert_ne!(replacement.source, raw.source);
        assert_ne!(replacement.device, raw.device);
        let replacement_window = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), backend_device)
            .expect("a new window-device lifetime on observed hardware reappearance");
        assert_eq!(
            replacement_window.source, window.source,
            "device loss is not destruction of the native window's input source"
        );
        assert_ne!(
            replacement_window.device, window.device,
            "removed hardware must not reuse the old window-event device identity"
        );
        assert_ne!(
            replacement_window.device, replacement.device,
            "raw-device observations use a separate identity from window events"
        );
    }

    #[test]
    fn native_host_provider_setup_installs_empty_window_registry_and_event_queue() {
        let mut app = App::new();
        assert!(
            app.world()
                .resource::<WindowStateRegistryResource>()
                .is_err()
        );
        assert!(
            app.world()
                .resource::<PlatformWindowEventQueueResource>()
                .is_err()
        );

        install_native_window_provider_resources(app.world_mut());

        let windows = app
            .world()
            .resource::<WindowStateRegistryResource>()
            .expect("native Host should install the native window registry");
        assert_eq!(windows.records().count(), 0);
        assert!(
            app.world()
                .resource::<PlatformWindowEventQueueResource>()
                .is_ok()
        );
    }

    #[test]
    fn primary_window_events_project_primary_presentation_metrics() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());

        runner
            .apply_event(PlatformEvent::Resized {
                width: 1600,
                height: 900,
            })
            .expect("primary resize should apply");
        runner
            .apply_event(PlatformEvent::ScaleFactorChanged {
                scale_factor: 2.0,
                width: 1600,
                height: 900,
            })
            .expect("primary scale change should apply");

        let presentation = runner
            .state
            .world
            .resource::<PrimaryPresentationMetricsResource>()
            .expect("primary presentation metrics should exist");
        assert_eq!(presentation.size_px(), (1600, 900));
        assert_eq!(presentation.scale_factor(), 2.0);
    }

    #[test]
    fn secondary_window_geometry_does_not_replace_primary_presentation_metrics() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let secondary = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .request_window("Secondary", (640, 480))
            .native_window_id;

        runner
            .apply_event_for_native_window(
                secondary,
                PlatformEvent::ScaleFactorChanged {
                    scale_factor: 1.75,
                    width: 900,
                    height: 600,
                },
            )
            .expect("secondary scale event should apply");

        let presentation = runner
            .state
            .world
            .resource::<PrimaryPresentationMetricsResource>()
            .expect("primary presentation metrics should exist");
        assert_eq!(presentation, &PrimaryPresentationMetricsResource::default());
        let secondary_record = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .record(secondary)
            .expect("secondary record should exist");
        assert_eq!(secondary_record.size_px, (900, 600));
        assert_eq!(secondary_record.scale_factor, 1.75);
    }

    #[test]
    fn secondary_creation_failure_marks_only_the_requested_logical_window() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let request = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry should exist")
            .request_window("Secondary", (900, 600));

        runner.mark_window_creation_failed(request.native_window_id, "test realization failure");

        let windows = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window registry should exist");
        assert_eq!(
            windows
                .record(request.native_window_id)
                .map(|record| record.lifecycle_state),
            Some(crate::runtime::NativeWindowLifecycleState::CreationFailed)
        );
        assert!(
            runner.window_realizer.realized_windows().is_empty(),
            "failed realization must not publish physical correlation"
        );
    }

    #[test]
    fn explicit_primary_redraw_request_wakes_on_demand_pacing() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());

        runner.request_redraw_for_native_window(NativeWindowId::primary());

        let registry = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .expect("window registry should exist");
        assert!(
            registry
                .record(NativeWindowId::primary())
                .expect("primary should exist")
                .redraw_requested
        );
        let pacing = runner
            .state
            .world
            .resource::<FramePacingRuntimeStateResource>()
            .expect("pacing state should exist");
        assert!(pacing.redraw_requested);
    }

    #[test]
    fn primary_input_redraws_wait_for_the_continuous_deadline() {
        let policy = FramePacingPolicyResource::continuous_capped(60);
        let interval = policy.target_frame_interval().expect("frame interval");
        let mut runner = runner_with_frame_pacing(policy);
        let start = Instant::now();

        assert!(runner.should_request_primary_redraw_now(start));
        assert!(!runner.should_request_primary_redraw_now(start + interval / 2));
        assert!(runner.should_request_primary_redraw_now(start + interval));
    }

    #[test]
    fn primary_input_redraws_stay_immediate_on_demand() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let now = Instant::now();

        assert!(runner.should_request_primary_redraw_now(now));
        assert!(runner.should_request_primary_redraw_now(now + std::time::Duration::from_secs(1)));
    }

    #[test]
    fn any_window_readiness_respects_continuous_cap_and_owned_redraw_does_not_duplicate() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::continuous_capped(60));
        let start = Instant::now();
        runner.last_primary_redraw_at = Some(start);
        let secondary = NativeWindowId::try_from_raw(2).unwrap();
        runner.request_redraw_for_native_window(secondary);
        assert!(runner.frame_requested);
        assert!(!runner.frame_is_ready(start));
        assert!(runner.frame_is_ready(start + std::time::Duration::from_millis(17)));
        runner.frame_requested = false;
        runner.owned_redraws.insert(secondary);
        runner.acknowledge_native_redraw(secondary);
        assert!(!runner.frame_requested);
        runner.acknowledge_native_redraw(secondary);
        assert!(
            runner.frame_requested,
            "an independent native redraw requests an Engine turn"
        );
    }

    #[test]
    fn native_group_overflow_changes_neither_physical_state_nor_journal() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let context = runner
            .input_adapter
            .window_context(NativeWindowId::primary(), winit::event::DeviceId::dummy())
            .unwrap();
        for _ in 0..4095 {
            runner
                .state
                .world
                .resource_mut::<PlatformWindowEventQueueResource>()
                .unwrap()
                .publish(PlatformWindowEvent::new(
                    NativeWindowId::primary(),
                    PlatformEvent::RedrawRequested,
                ))
                .unwrap();
        }
        assert!(
            runner
                .apply_event_group(
                    NativeWindowId::primary(),
                    [
                        normalized_shift_press(context),
                        PlatformEvent::TextInput { text: "x".into() },
                    ]
                )
                .is_err()
        );
        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .shift_down()
        );
        assert_eq!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .unwrap()
                .events()
                .len(),
            4095
        );
    }

    #[test]
    fn missing_physical_owner_rejects_focus_group_before_window_or_journal_mutation() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.remove_resource::<InputState>();
        let primary = NativeWindowId::primary();
        runner
            .input_adapter
            .window_context(primary, winit::event::DeviceId::dummy())
            .unwrap();
        let before = runner
            .state
            .world
            .resource::<WindowStateRegistryResource>()
            .unwrap()
            .record(primary)
            .unwrap()
            .focused;
        assert!(runner.apply_window_focus_event(primary, false).is_err());
        assert_eq!(
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .unwrap()
                .record(primary)
                .unwrap()
                .focused,
            before
        );
        assert!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .unwrap()
                .events()
                .is_empty()
        );
    }
}
