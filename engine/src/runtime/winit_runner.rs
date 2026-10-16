use crate::app::WindowedAppState;
use crate::plugins::render::host::native as render_native_host;
use crate::plugins::ui::{
    NativeInputRecipient, NativeInputRecipientError, NativePointerOrigin,
    UiNativeInputRecipientsResource, UiNativePointerHostRead, UiRuntimeDisplayedPresentation,
    UiRuntimeNativeCompositionAdmissionError, UiRuntimeNativeTextAdmissionError,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotId, UiRuntimeSlotsResource,
    native_composition_finality, native_keyboard_finality,
    native_no_focused_keyboard_claim_witness, native_original_keyboard_release_nonclaim,
    native_pointer_finality, native_text_finality,
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
    NativeWindowCreationRequest, NativeWindowId, NativeWindowLifecycleState,
    WindowStateRegistryResource,
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
    CommittedTextEvent, InputDeviceId as UiInputDeviceId, KeyModifiers, KeyboardCompositionState,
    LogicalDelta, LogicalPoint,
};
use runenui_runtime::{UiInputConflict, UiInputFinality};
use runenui_winit::keyboard_input::{
    KeyboardInputOutcome, KeyboardInputState, NativeKeyTransition,
};
use runenui_winit::mouse_input::{MouseButtonOutcome, MouseInputState, TranslatedPointerPoint};
use runenui_winit::touch_input::TouchInputState;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{
    DeviceEvent, ElementState, Ime, MouseButton, MouseScrollDelta, Touch, WindowEvent,
};
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

/// Translate the actual M13 composition lifetime into the pinned native
/// keyboard adapter's classification. The Engine does not own another IME
/// generation and must not fabricate Inactive while RunenUI is composing.
fn native_keyboard_composition(
    ownership: &runenui_runtime::InputOwnershipSnapshot,
) -> KeyboardCompositionState {
    if ownership.keyboard().composition_generation().is_some() {
        KeyboardCompositionState::Active
    } else {
        KeyboardCompositionState::Inactive
    }
}

/// A derived OS permission, never a second focus or text-input authority.
/// The selected native recipient and actual focused window are checked by
/// WinitRunner before consulting this canonical M13 ownership snapshot.
fn native_ime_allowed_for_m13(
    native_window_focused: bool,
    ownership: &runenui_runtime::InputOwnershipSnapshot,
) -> bool {
    let keyboard = ownership.keyboard();
    let capability = keyboard.text_input_capability();
    native_window_focused
        && ownership.status() == runenui_runtime::RuntimeStatus::Running
        && keyboard.focused_node().is_some()
        && (capability.accepts_committed_text() || capability.accepts_composition())
}

/// A pinned winit translator must NEVER cross its native window, physical
/// source or selected M13 recipient lifetime. No UI focus or gameplay owner
/// is replicated here; only the adapter's transient pressed-key state.
struct NativeUiKeyboardTranslator {
    recipient: UiRuntimeSlotId,
    source: InputSourceId,
    /// Observed native hardware identities only; pressed-key state remains
    /// exclusively inside the device-qualified RunenUI keyboard translator.
    devices: HashSet<UiInputDeviceId>,
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

/// Unlike mouse, native touch permits simultaneous device/contact streams on
/// the same host window. Identity and canonical contact state stay inside the
/// pinned winit adapter; this wrapper only scopes its lifetime to native owners.
struct NativeUiTouchTranslator {
    recipient: UiRuntimeSlotId,
    source: InputSourceId,
    binding_epoch: u64,
    scope: runenui_runtime::InputArbitrationScope,
    backend_devices: HashSet<winit::event::DeviceId>,
    touch: TouchInputState,
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

/// Explicit Runenwerk native wheel policy. This is a host UX metric,
/// not a RunenUI protocol or a platform-dependent guessed pixel ratio.
const NATIVE_UI_LINE_SCROLL_LOGICAL_UNITS: f64 = 60.0;

/// Phase-only zero-displacement wheel events are inert. For real movement,
/// native physical-pixel wheel values use the exact displayed DPI mapping.
#[allow(
    clippy::cast_possible_truncation,
    reason = "native wheel deltas are finite and f32-range-checked before conversion to UI logical coordinates"
)]
fn native_ui_logical_wheel_delta(
    delta: &MouseScrollDelta,
    displayed_scale: f64,
) -> Result<Option<LogicalDelta>> {
    let (x, y) = match delta {
        MouseScrollDelta::LineDelta(x, y) => (
            f64::from(*x) * NATIVE_UI_LINE_SCROLL_LOGICAL_UNITS,
            f64::from(*y) * NATIVE_UI_LINE_SCROLL_LOGICAL_UNITS,
        ),
        MouseScrollDelta::PixelDelta(delta) => {
            if delta.x == 0.0 && delta.y == 0.0 {
                return Ok(None);
            }
            anyhow::ensure!(
                displayed_scale.is_finite() && displayed_scale > 0.0,
                "native wheel lacks a valid terminal displayed scale"
            );
            (delta.x / displayed_scale, delta.y / displayed_scale)
        }
    };
    if x == 0.0 && y == 0.0 {
        return Ok(None);
    }
    anyhow::ensure!(
        x.is_finite()
            && y.is_finite()
            && x >= f64::from(f32::MIN)
            && x <= f64::from(f32::MAX)
            && y >= f64::from(f32::MIN)
            && y <= f64::from(f32::MAX),
        "native wheel delta out of finite RunenUI logical range"
    );
    Ok(Some(
        LogicalDelta::new(x as f32, y as f32).context("checked native wheel delta rejected")?,
    ))
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
        native_ui_touches: HashMap::new(),
        native_ui_modifiers: HashMap::new(),
        native_ime_applied: HashSet::new(),
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
    native_ui_touches: HashMap<NativeWindowId, NativeUiTouchTranslator>,
    native_ui_modifiers: HashMap<NativeWindowId, KeyModifiers>,
    /// Applied native OS effect only, not a focus/IME generation authority.
    /// Winit offers no getter for this permission; avoid repeatedly toggling
    /// OS IME on each mouse/wake event, and use M13 for every recalculation.
    native_ime_applied: HashSet<NativeWindowId>,
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
        // A required original source-loss journal entry MUST have its full
        // bounded admission (and normalized reducer) proven before any M13
        // Cancel mutates the separately owned framework pressed streams.
        // Otherwise journal exhaustion could leave the original source live
        // after RunenUI has irreversibly retired the same hardware lifetime.
        let original_source = self.input_adapter.window_source_context(native_window_id);
        if let Some(context) = original_source {
            self.state
                .world
                .resource_mut::<PlatformWindowEventQueueResource>()
                .context("native input journal missing before source retirement")?
                .reserve_native_group(&[PlatformEvent::InputContinuityLost {
                    context,
                    loss: ContinuityLoss::Source,
                }])?;
            self.state
                .world
                .resource::<InputState>()
                .context("native physical state missing before source retirement")?;
        }
        // Source retirement is a mandatory native pointer-stream boundary,
        // even when an original window callback is no longer forthcoming.
        let mouse_cancel = self.cancel_native_ui_mouse(native_window_id);
        let touch_cancel = self.cancel_native_ui_touch(native_window_id);
        let keyboard_cancel = match self
            .native_ui_keyboards
            .get(&native_window_id)
            .map(|lifetime| lifetime.recipient)
        {
            Some(slot) => self.cancel_native_ui_keyboard(native_window_id, slot),
            None => Ok(()),
        };
        let cancellation_failure = mouse_cancel
            .err()
            .or(touch_cancel.err())
            .or(keyboard_cancel.err());
        let Some(context) = original_source else {
            return match cancellation_failure {
                Some(error) => Err(error),
                None => Ok(false),
            };
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
        if let Some(error) = cancellation_failure {
            return Err(error);
        }
        match selection {
            Ok(_) => Ok(true),
            Err(error)
                if error.downcast_ref::<NativeInputRecipientError>()
                    == Some(&NativeInputRecipientError::MissingRecipient) =>
            {
                Ok(true)
            }
            Err(error)
                if error.downcast_ref::<NativeInputRecipientError>()
                    == Some(&NativeInputRecipientError::InactiveWindow)
                    && self
                        .state
                        .world
                        .resource::<WindowStateRegistryResource>()
                        .ok()
                        .and_then(|windows| windows.record(native_window_id))
                        .is_some_and(|record| {
                            record.close_requested
                                && record.lifecycle_state
                                    == NativeWindowLifecycleState::CloseApproved
                        }) =>
            {
                // An already-approved native close is no longer Created.
                // The original source/ActionState are now retired; this is
                // not a gameplay recipient fallback or an M13 Cancel claim.
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
        let mouse_cancel = self.cancel_native_ui_mouse(window);
        let touch_cancel = self.cancel_native_ui_touch(window);
        let mut cancellation_failure = mouse_cancel.err().or(touch_cancel.err());
        // Retire the original framework-held keyboard stream while its
        // source/recipient binding is still valid. After retirement, a
        // synthetic Cancel cannot safely cross into a replacement M13 scope.
        if let Some(slot) = self
            .native_ui_keyboards
            .get(&window)
            .map(|state| state.recipient)
            && let Err(error) = self.cancel_native_ui_keyboard(window, slot)
        {
            // Invalid framework state cannot prevent mandatory native target
            // retirement or manufacture gameplay input permission.
            tracing::warn!(
                target: "engine.runtime.input",
                ?window,
                ?error,
                "native keyboard cancellation failed during recipient retirement"
            );
            if cancellation_failure.is_none() {
                cancellation_failure = Some(error);
            }
        }
        let world = &mut self.state.world;
        if !world.has_resource::<UiNativeInputRecipientsResource>() {
            return match cancellation_failure {
                Some(error) => Err(error),
                None => Ok(false),
            };
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
        if let Some(error) = cancellation_failure {
            return Err(error);
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
                    || existing.binding_epoch != binding_epoch
            });
        if replaced {
            // No key/modifier stream from the previous binding may become
            // a pressed continuation under the replacement physical source.
            self.native_ui_keyboards.remove(&window);
            self.native_ui_modifiers.remove(&window);
        }
        let lifetime =
            self.native_ui_keyboards
                .entry(window)
                .or_insert_with(|| NativeUiKeyboardTranslator {
                    recipient: slot,
                    source,
                    devices: HashSet::new(),
                    binding_epoch,
                    keyboard: KeyboardInputState::default(),
                });
        lifetime.devices.insert(device);
        &mut lifetime.keyboard
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
        // A keyboard transition must be translated against the SAME fresh
        // M13 owner-authored pre-input snapshot used for its finality proof.
        // In particular, composition-active shortcuts must not be decoded
        // using an invented Inactive state. Observe before mutating the pinned
        // pressed-key translator or submitting any framework work.
        let observed = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("selected native UI runtime vanished before input observation")?
            .input_ownership(slot);
        let before = match observed {
            Ok(snapshot) => snapshot,
            Err(error) => {
                if matches!(original.state, winit::event::ElementState::Pressed) {
                    self.deny_native_keyboard_lifetime(context, &physical_key)?;
                }
                tracing::trace!(
                    target: "engine.runtime.input",
                    ?window,
                    ?error,
                    "no M13 pre-input ownership: original keyboard remains neutral only"
                );
                return Err(anyhow!(
                    "native keyboard M13 pre-input ownership unavailable: {error:?}"
                ));
            }
        };
        let composition = native_keyboard_composition(&before);
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
            .key_input(device, &transition, modifiers, composition);

        // Once the real M13 queue has admitted a keyboard occurrence,
        // missing finality is not merely a gameplay nonclaim: continuing the
        // host could preserve an unresolved framework keyboard lifetime.
        let mut m13_failure: Option<anyhow::Error> = None;
        let permit = match translated {
            KeyboardInputOutcome::Submit(event) => {
                let slots = self
                    .state
                    .world
                    .resource_mut::<UiRuntimeSlotsResource>()
                    .context("selected UI runtime vanished from native host")?;
                match slots.submit_native_keyboard(slot, event) {
                    Ok(submission) => match slots.pump_native_input_records(slot) {
                        Ok((batch, status)) => {
                            let evidence = native_keyboard_finality(
                                before.scope(),
                                device,
                                submission,
                                &batch,
                                &status,
                            );
                            match &evidence {
                                Ok(UiInputFinality::Committed(facts)) => matches!(
                                    facts.conflict(),
                                    UiInputConflict::Unclaimed
                                        | UiInputConflict::ObservedNonexclusive
                                ),
                                _ => {
                                    m13_failure = Some(anyhow!(
                                        "native keyboard lacked exact committed M13 settlement: {evidence:?}"
                                    ));
                                    false
                                }
                            }
                        }
                        Err(error) => {
                            m13_failure = Some(anyhow!(
                                "native keyboard admitted M13 work but its pump failed: {error:?}"
                            ));
                            false
                        }
                    },
                    Err(error) => match slots.input_ownership(slot) {
                        Ok(after)
                            if native_no_focused_keyboard_claim_witness(
                                device, &before, &after, &error,
                            ) =>
                        {
                            true
                        }
                        Ok(after)
                            if !is_synthetic
                                && matches!(
                                    original.state,
                                    winit::event::ElementState::Released
                                )
                                && native_original_keyboard_release_nonclaim(
                                    device, &before, &after, &error,
                                ) =>
                        {
                            // Genuine physical Up remains neutral-only. A
                            // release never supplies gameplay press authority.
                            false
                        }
                        Ok(_) => {
                            m13_failure = Some(anyhow!(
                                "native keyboard M13 admission rejected without exact original no-focus evidence: {error:?}"
                            ));
                            false
                        }
                        Err(observation) => {
                            m13_failure = Some(anyhow!(
                                "native keyboard M13 admission rejected and post-state observation failed: {error:?}; {observation:?}"
                            ));
                            false
                        }
                    },
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
        // The genuine physical Down remains denied before returning the
        // terminal framework error; an ambiguous pump may not grant gameplay.
        if let Some(error) = m13_failure {
            return Err(error);
        }
        Ok(())
    }

    /// Original Winit preedit has no trustworthy DeviceId. Composition
    /// generation, focus, scope and routing remain the pinned RunenUI owner's
    /// facts; Engine keeps no second IME generation or pending preedit queue.
    fn route_ui_ime_preedit(
        &mut self,
        slot: UiRuntimeSlotId,
        preedit: String,
        range: Option<(usize, usize)>,
    ) -> Result<()> {
        let selected_range = match range
            .map(|(from, to)| runenui_core::CompositionRange::new(&preedit, from, to))
            .transpose()
        {
            Ok(range) => range,
            Err(error) => {
                tracing::trace!(
                    target: "engine.runtime.input",
                    ?error,
                    "invalid OS preedit UTF-8 range rejected without changing M13"
                );
                return Ok(());
            }
        };
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("selected native IME preedit lacks pinned UI slot")?;
        let mut before = slots
            .input_ownership(slot)
            .map_err(|error| anyhow!("native preedit has no settled M13 owner: {error:?}"))?;
        let generation = if let Some(generation) = before.keyboard().composition_generation() {
            generation.clone()
        } else {
            if preedit.is_empty() {
                return Ok(());
            }
            let start = match slots.start_native_composition(slot) {
                Ok(start) => start,
                Err(UiRuntimeNativeCompositionAdmissionError::Start(error))
                    if matches!(
                        error.kind(),
                        runenui_runtime::SubmitCompositionErrorKind::NoFocusedTarget
                            | runenui_runtime::SubmitCompositionErrorKind::FocusedTargetNotCompositionCapable
                    ) => return Ok(()),
                Err(UiRuntimeNativeCompositionAdmissionError::Slot(failure)) => {
                    return Err(anyhow!("native composition slot unavailable: {failure:?}"));
                }
                Err(error) => return Err(anyhow!("native composition start rejected: {error:?}")),
            };
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .map_err(|e| anyhow!("native composition start pump rejected: {e:?}"))?;
            let finality =
                native_composition_finality(before.scope(), start.sequence(), &batch, &status);
            anyhow::ensure!(
                matches!(finality, Ok(UiInputFinality::Committed(_))),
                "native composition start lacks exact M13 settlement: {finality:?}"
            );
            before = slots
                .input_ownership(slot)
                .map_err(|e| anyhow!("composition start left M13 ownership unsettled: {e:?}"))?;
            anyhow::ensure!(
                before.keyboard().composition_generation() == Some(start.generation()),
                "composition generation changed before original preedit update"
            );
            start.generation().clone()
        };
        let submitted =
            match slots.update_native_composition(slot, generation, preedit, selected_range) {
                Ok(receipt) => receipt,
                Err(UiRuntimeNativeCompositionAdmissionError::Event(error))
                    if matches!(
                        error.kind(),
                        runenui_runtime::SubmitCompositionErrorKind::MissingGeneration
                            | runenui_runtime::SubmitCompositionErrorKind::StaleGeneration
                    ) =>
                {
                    return Ok(());
                }
                Err(UiRuntimeNativeCompositionAdmissionError::Slot(failure)) => {
                    return Err(anyhow!("native preedit slot invalid: {failure:?}"));
                }
                Err(error) => {
                    return Err(anyhow!("native preedit M13 ingress rejected: {error:?}"));
                }
            };
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .map_err(|e| anyhow!("native preedit M13 pump rejected: {e:?}"))?;
        let finality =
            native_composition_finality(before.scope(), submitted.sequence(), &batch, &status);
        anyhow::ensure!(
            matches!(finality, Ok(UiInputFinality::Committed(_))),
            "native preedit lacked exact M13 composition settlement: {finality:?}"
        );
        Ok(())
    }

    /// An original IME commit ends the framework's active generation before
    /// submitting durable text. Disabled/focus loss cancels that generation.
    /// Neither can synthesize physical keyboard transitions.
    fn route_ui_ime_finish(&mut self, slot: UiRuntimeSlotId, cancel: bool) -> Result<()> {
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("native IME finish lost pinned UI slot")?;
        let before = slots
            .input_ownership(slot)
            .map_err(|error| anyhow!("native IME finish has no M13 owner: {error:?}"))?;
        let Some(generation) = before.keyboard().composition_generation().cloned() else {
            return Ok(());
        };
        let submission = if cancel {
            slots.cancel_native_composition(slot, generation)
        } else {
            slots.end_native_composition(slot, generation)
        };
        let submission = match submission {
            Ok(receipt) => receipt,
            Err(UiRuntimeNativeCompositionAdmissionError::Event(error))
                if matches!(
                    error.kind(),
                    runenui_runtime::SubmitCompositionErrorKind::MissingGeneration
                        | runenui_runtime::SubmitCompositionErrorKind::StaleGeneration
                ) =>
            {
                return Ok(());
            }
            Err(UiRuntimeNativeCompositionAdmissionError::Slot(failure)) => {
                return Err(anyhow!("native IME finish slot invalid: {failure:?}"));
            }
            Err(error) => return Err(anyhow!("native IME finish rejected: {error:?}")),
        };
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .map_err(|e| anyhow!("native IME finish pump rejected: {e:?}"))?;
        let finality =
            native_composition_finality(before.scope(), submission.sequence(), &batch, &status);
        anyhow::ensure!(
            matches!(finality, Ok(UiInputFinality::Committed(_))),
            "native IME finish lacked exact M13 settlement: {finality:?}"
        );
        Ok(())
    }

    /// Route one original OS-committed IME text occurrence to the selected
    /// M13 text owner. IME carries no winit device ID; do not invent one from
    /// the last key or guess the keyboard's physical lifetime.
    fn route_ui_ime_commit(&mut self, slot: UiRuntimeSlotId, text: String) -> Result<()> {
        self.route_ui_ime_finish(slot, false)?;
        let event = CommittedTextEvent::new(text, None)
            .context("empty original committed text must be filtered at native ingress")?;
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("native IME recipient has no pinned runtime slot")?;
        let before = slots
            .input_ownership(slot)
            .map_err(|error| anyhow!("native IME M13 text ownership unavailable: {error:?}"))?;
        let submission = match slots.submit_native_text(slot, event) {
            Ok(receipt) => receipt,
            Err(UiRuntimeNativeTextAdmissionError::Slot { failure, event }) => {
                tracing::warn!(
                    target: "engine.runtime.input",
                    ?failure,
                    bytes = event.text().len(),
                    "native IME text rejected by missing/invalid slot authority"
                );
                return Err(anyhow!("native IME text target unavailable: {failure:?}"));
            }
            Err(UiRuntimeNativeTextAdmissionError::Framework(error)) => {
                let kind = error.kind();
                tracing::trace!(
                    target: "engine.runtime.input",
                    ?kind,
                    bytes = error.event().text().len(),
                    "native IME committed text rejected by pinned M13 ownership"
                );
                return if matches!(
                    kind,
                    runenui_runtime::SubmitTextErrorKind::NoFocusedTarget
                        | runenui_runtime::SubmitTextErrorKind::FocusedTargetNotTextCapable
                ) {
                    Ok(())
                } else {
                    Err(anyhow!("native IME text submission failed: {kind:?}"))
                };
            }
        };
        let (batch, status) = slots
            .pump_native_input_records(slot)
            .map_err(|error| anyhow!("native IME M13 pump unavailable: {error:?}"))?;
        let evidence = native_text_finality(before.scope(), None, submission, &batch, &status);
        anyhow::ensure!(
            matches!(evidence, Ok(UiInputFinality::Committed(_))),
            "native IME text lacked exact committed M13 settlement: {evidence:?}"
        );
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
        if let Some(mut actions) = self.state.world.remove_resource::<ActionState>() {
            let result = self.state.world.resource::<InputState>().map(|input| {
                if let Some(generation) = generation {
                    actions.revoke_keyboard_lifetime(input, context, physical_key, generation);
                } else {
                    actions.revoke_generationless_keyboard_lifetime(input, context, physical_key);
                }
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
        let before = slots.input_ownership(slot).map_err(|reason| {
            anyhow!("native keyboard cancellation lacked M13 pre-state: {reason:?}")
        })?;
        for event in lifetime
            .keyboard
            .cancel_all(KeyModifiers::NONE, native_keyboard_composition(&before))
        {
            let device = event
                .device_id()
                .context("native keyboard Cancel lost its original device identity")?;
            let submission = slots.submit_native_keyboard(slot, event).map_err(|error| {
                anyhow!("native keyboard Cancel M13 admission rejected: {error:?}")
            })?;
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .map_err(|error| anyhow!("native keyboard Cancel M13 pump failed: {error:?}"))?;
            let evidence =
                native_keyboard_finality(before.scope(), device, submission, &batch, &status);
            anyhow::ensure!(
                matches!(evidence, Ok(UiInputFinality::Committed(_))),
                "native keyboard Cancel lacked committed M13 settlement: {evidence:?}"
            );
        }
        Ok(())
    }

    /// Retire the actually disconnected keyboard device without destroying
    /// other keys in the same window/slot/source lifetime. The translator owns
    /// its pressed keys; this host tracks only observed hardware membership.
    fn cancel_native_ui_keyboard_device(
        &mut self,
        window: NativeWindowId,
        device: UiInputDeviceId,
    ) -> Result<()> {
        let Some((recipient, epoch)) = self.native_ui_keyboards.get(&window).and_then(|lifetime| {
            lifetime
                .devices
                .contains(&device)
                .then_some((lifetime.recipient, lifetime.binding_epoch))
        }) else {
            return Ok(());
        };
        if !matches!(
            self.native_input_recipient(window),
            Ok((NativeInputRecipient::RunenUiSlot(slot), Some(current)))
                if slot == recipient && current == epoch
        ) {
            // Stale target cannot receive a synthetic event in a new scope.
            self.native_ui_keyboards.remove(&window);
            self.native_ui_modifiers.remove(&window);
            return Ok(());
        }
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("device-specific keyboard cancellation lost its pinned slot")?;
        let before = slots.input_ownership(recipient).map_err(|error| {
            anyhow!("native keyboard device-loss M13 ownership unavailable: {error:?}")
        })?;
        let events = {
            let lifetime = self
                .native_ui_keyboards
                .get_mut(&window)
                .context("selected original native keyboard lifetime vanished")?;
            lifetime.devices.remove(&device);
            lifetime
                .keyboard
                .cancel_device(device, modifiers, native_keyboard_composition(&before))
        };
        // The winit modifier callback is window-wide; no original hardware
        // identity proves whether Shift/Ctrl belonged to removed A or live B.
        // Fail closed on every removal (not just the last keyboard) until
        // another authentic window modifier observation arrives.
        self.native_ui_modifiers.remove(&window);
        for event in events {
            let submission = slots
                .submit_native_keyboard(recipient, event)
                .map_err(|error| {
                    anyhow!("native keyboard device-loss M13 submission rejected: {error:?}")
                })?;
            let (batch, status) = slots
                .pump_native_input_records(recipient)
                .map_err(|error| {
                    anyhow!("native keyboard device-loss M13 pump failed: {error:?}")
                })?;
            let evidence =
                native_keyboard_finality(before.scope(), device, submission, &batch, &status);
            anyhow::ensure!(
                matches!(evidence, Ok(UiInputFinality::Committed(_))),
                "native keyboard device-loss Cancel lacked exact M13 settlement: {evidence:?}"
            );
        }
        Ok(())
    }

    /// Retire one original touch hardware device only. Keep the same adapter
    /// alive so foreign device B retains contact and pointer ID continuity.
    fn cancel_native_ui_touch_device(
        &mut self,
        window: NativeWindowId,
        backend_device: winit::event::DeviceId,
        device: UiInputDeviceId,
    ) -> Result<()> {
        let Some((recipient, epoch, scope)) =
            self.native_ui_touches.get(&window).and_then(|lifetime| {
                lifetime.backend_devices.contains(&backend_device).then(|| {
                    (
                        lifetime.recipient,
                        lifetime.binding_epoch,
                        lifetime.scope.clone(),
                    )
                })
            })
        else {
            return Ok(());
        };
        if !matches!(
            self.native_input_recipient(window),
            Ok((NativeInputRecipient::RunenUiSlot(slot), Some(current)))
                if slot == recipient && current == epoch
        ) {
            self.native_ui_touches.remove(&window);
            return Ok(());
        }
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("device-specific touch cancellation lost its pinned slot")?;
        let before = slots.input_ownership(recipient).map_err(|error| {
            anyhow!("native touch device-loss M13 ownership unavailable: {error:?}")
        })?;
        anyhow::ensure!(
            before.scope() == &scope,
            "native touch device-loss Cancel crossed its original M13 scope"
        );
        let events = {
            let lifetime = self
                .native_ui_touches
                .get_mut(&window)
                .context("original native touch lifetime vanished")?;
            lifetime.backend_devices.remove(&backend_device);
            lifetime.touch.cancel_device(device)
        };
        for event in events {
            let pointer = event.pointer_id();
            // Retain the original displayed surface identity after the
            // framework takes ownership of the cancellation event.
            let surface = event.surface_context().surface_id().clone();
            let submission = slots
                .submit_native_pointer(recipient, event)
                .map_err(|error| {
                    anyhow!("native touch device-loss M13 submission rejected: {error:?}")
                })?;
            let (batch, status) = slots
                .pump_native_input_records(recipient)
                .map_err(|error| anyhow!("native touch device-loss M13 pump failed: {error:?}"))?;
            let evidence = native_pointer_finality(
                before.scope(),
                device,
                pointer,
                &surface,
                submission,
                &batch,
                &status,
            );
            anyhow::ensure!(
                matches!(evidence, Ok(UiInputFinality::Committed(_))),
                "native touch device-loss Cancel lacked exact M13 settlement: {evidence:?}"
            );
        }
        Ok(())
    }

    fn apply_raw_device_removal(&mut self, device_id: winit::event::DeviceId) -> Result<bool> {
        // Winit window events and DeviceEvents have separately allocated
        // RunenInput identities. They share only the original backend DeviceId.
        // Preflight both identity namespaces and retire them atomically BEFORE
        // mutating a UI translator. A fallible allocation or missing raw source
        // must not leave RunenUI cancelled while neutral held keys remain live.
        let (window_contexts, raw_context) = self.input_adapter.retire_backend_device(device_id)?;
        // UI Cancels still use the original pinned M13 recipient and scope;
        // the removed physical IDs remain preserved in window_contexts.
        let windows: Vec<_> = self
            .native_ui_mice
            .iter()
            .filter_map(|(window, lifetime)| {
                (lifetime.backend_device == device_id).then_some(*window)
            })
            .collect();
        let mut cancellation_failure: Option<anyhow::Error> = None;
        for window in windows.iter().copied() {
            if let Err(error) = self.cancel_native_ui_mouse(window)
                && cancellation_failure.is_none()
            {
                cancellation_failure = Some(error);
            }
        }
        let touch_windows: Vec<_> = self
            .native_ui_touches
            .iter()
            .filter_map(|(window, lifetime)| {
                lifetime
                    .backend_devices
                    .contains(&device_id)
                    .then_some(*window)
            })
            .collect();
        // Exactly one RunenInput window-device identity exists for this backend
        // device across its window sources. Raw identity is a separate namespace.
        if let Some(retired_device) = window_contexts.first().and_then(|context| context.device) {
            let ui_device = UiInputDeviceId::new(retired_device.raw())
                .context("retired keyboard/touch device identity is zero")?;
            for window in touch_windows.iter().copied() {
                if let Err(error) = self.cancel_native_ui_touch_device(window, device_id, ui_device)
                {
                    tracing::warn!(
                        target: "engine.runtime.input",
                        ?window,
                        ?error,
                        "native touch device cancellation failed during device removal"
                    );
                    if cancellation_failure.is_none() {
                        cancellation_failure = Some(error);
                    }
                }
            }
            let keyboard_windows: Vec<_> = self
                .native_ui_keyboards
                .iter()
                .filter_map(|(window, lifetime)| {
                    lifetime.devices.contains(&ui_device).then_some(*window)
                })
                .collect();
            for window in keyboard_windows {
                if let Err(error) = self.cancel_native_ui_keyboard_device(window, ui_device) {
                    // Neutral physical loss and ActionState revocation remain
                    // mandatory even when stale M13 cancellation cannot settle.
                    tracing::warn!(
                        target: "engine.runtime.input",
                        ?window,
                        ?error,
                        "native keyboard device cancellation failed during device removal"
                    );
                    if cancellation_failure.is_none() {
                        cancellation_failure = Some(error);
                    }
                }
            }
        } else if !touch_windows.is_empty() {
            // A tracked original touch stream cannot outlive its checked
            // window-device identity. Missing that identity makes an exact
            // M13 device Cancel impossible: success would strand contacts.
            cancellation_failure = Some(anyhow!(
                "native touch device loss lacks its original window-device identity"
            ));
        }
        if window_contexts.is_empty() && raw_context.is_none() {
            if let Some(error) = cancellation_failure {
                return Err(error);
            }
            return Ok(!windows.is_empty() || !touch_windows.is_empty());
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
        if let Some(error) = cancellation_failure {
            return Err(error);
        }
        Ok(true)
    }

    /// Retire only the pinned M13 mouse stream in its original live scope.
    /// A cancellation is never interpreted as an original physical Up.
    fn cancel_native_ui_mouse(&mut self, window: NativeWindowId) -> Result<()> {
        let Some(mut lifetime) = self.native_ui_mice.remove(&window) else {
            return Ok(());
        };
        // A remount/rebind cannot receive a Cancel from the previous native
        // window binding, even when the same runtime scope remains mounted.
        if !matches!(
            self.native_input_recipient(window),
            Ok((NativeInputRecipient::RunenUiSlot(slot), Some(epoch)))
                if slot == lifetime.recipient && epoch == lifetime.binding_epoch
        ) {
            return Ok(());
        }
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let Some(cancel) = lifetime.mouse.invalidate_point_authority(modifiers) else {
            return Ok(());
        };
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("native mouse Cancel lost its pinned framework runtime")?;
        let before = slots
            .input_ownership(lifetime.recipient)
            .map_err(|error| anyhow!("native mouse Cancel M13 pre-state unavailable: {error:?}"))?;
        if before.scope() != &lifetime.scope {
            // A stale stream must not be submitted to a replacement scope.
            return Ok(());
        }
        let pointer = cancel.pointer_id();
        let surface = cancel.surface_context().surface_id().clone();
        let submission = slots
            .submit_native_pointer(lifetime.recipient, cancel)
            .map_err(|error| anyhow!("native mouse Cancel M13 admission rejected: {error:?}"))?;
        let (batch, status) = slots
            .pump_native_input_records(lifetime.recipient)
            .map_err(|error| anyhow!("native mouse Cancel M13 pump failed: {error:?}"))?;
        let evidence = native_pointer_finality(
            before.scope(),
            lifetime.device,
            pointer,
            &surface,
            submission,
            &batch,
            &status,
        );
        anyhow::ensure!(
            matches!(evidence, Ok(UiInputFinality::Committed(_))),
            "native mouse Cancel lacked committed M13 settlement: {evidence:?}"
        );
        Ok(())
    }

    /// Retire every pinned touch contact from its exact original M13 scope.
    /// These are framework Cancels, never fabricated physical releases or
    /// positive gameplay permissions. Do not transplant them to a new scope.
    fn cancel_native_ui_touch(&mut self, window: NativeWindowId) -> Result<()> {
        let Some(mut lifetime) = self.native_ui_touches.remove(&window) else {
            return Ok(());
        };
        if !matches!(
            self.native_input_recipient(window),
            Ok((NativeInputRecipient::RunenUiSlot(slot), Some(epoch)))
                if slot == lifetime.recipient && epoch == lifetime.binding_epoch
        ) {
            return Ok(());
        }
        let events = lifetime.touch.cancel_all();
        if events.is_empty() {
            return Ok(());
        }
        let slots = self
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .context("native touch Cancel lost its pinned framework runtime")?;
        let before = slots
            .input_ownership(lifetime.recipient)
            .map_err(|error| anyhow!("native touch Cancel M13 pre-state unavailable: {error:?}"))?;
        if before.scope() != &lifetime.scope {
            // A retired generation must not cancel a newly mounted UI owner.
            return Ok(());
        }
        for event in events {
            let device = event
                .device_id()
                .context("native touch Cancel lost its original hardware device identity")?;
            let pointer = event.pointer_id();
            let surface = event.surface_context().surface_id().clone();
            let submission = slots
                .submit_native_pointer(lifetime.recipient, event)
                .map_err(|error| {
                    anyhow!("native touch Cancel M13 admission rejected: {error:?}")
                })?;
            let (batch, status) = slots
                .pump_native_input_records(lifetime.recipient)
                .map_err(|error| anyhow!("native touch Cancel M13 pump failed: {error:?}"))?;
            let evidence = native_pointer_finality(
                before.scope(),
                device,
                pointer,
                &surface,
                submission,
                &batch,
                &status,
            );
            anyhow::ensure!(
                matches!(evidence, Ok(UiInputFinality::Committed(_))),
                "native touch Cancel lacked committed M13 settlement: {evidence:?}"
            );
        }
        Ok(())
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
            self.cancel_native_ui_mouse(window)?;
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
        // Before framework ingress a rejected observation is neutral-only;
        // after an actual M13 attempt, missing finality is a host error.
        let mut m13_submission_attempted = false;
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
            m13_submission_attempted = true;
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
                matches!(finality, Ok(UiInputFinality::Committed(_))),
                "native UI cursor lacked exact M13 pointer settlement: {finality:?}"
            );
            Ok(())
        })();
        world.insert_resource(slots);
        if let Err(error) = operation {
            self.cancel_native_ui_mouse(window)?;
            tracing::trace!(
                target: "engine.runtime.input",
                ?window,
                ?error,
                "native UI cursor withheld without gameplay grant"
            );
            if m13_submission_attempted {
                return Err(error.context("native UI cursor M13 submission or settlement failed"));
            }
        }
        Ok(())
    }

    /// Admit one original native mouse button against the EXISTING neutral
    /// cursor stream, the exact current completed Present and a reached M13
    /// pointer receipt. No cursor point is invented if the device has not
    /// supplied one; a suppressed press/release never grants gameplay.
    fn route_ui_mouse_button(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        context: InputContext,
        state: ElementState,
        button: MouseButton,
    ) -> Result<()> {
        let device = Self::native_ui_device(context)?;
        let Some(mut lifetime) = self.native_ui_mice.remove(&window) else {
            // No authentic previous cursor position/stream. The pinned mouse
            // adapter would suppress the press; do not invent UI point truth.
            return Ok(());
        };
        if lifetime.recipient != slot
            || lifetime.binding_epoch != binding_epoch
            || lifetime.source != context.source
            || lifetime.device != device
        {
            self.native_ui_mice.insert(window, lifetime);
            self.cancel_native_ui_mouse(window)?;
            return Ok(());
        }
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let world = &mut self.state.world;
        let mut slots = world
            .remove_resource::<UiRuntimeSlotsResource>()
            .context("selected native UI mouse button lacks slot authority")?;
        // Before framework ingress a rejected observation is neutral-only;
        // after an actual M13 attempt, missing finality is a host error.
        let mut m13_submission_attempted = false;
        let operation = (|| -> Result<()> {
            let before = slots
                .input_ownership(slot)
                .map_err(|error| anyhow!("native mouse button pre-state unavailable: {error:?}"))?;
            anyhow::ensure!(
                before.scope() == &lifetime.scope,
                "native mouse button stream belongs to a retired M13 scope"
            );
            let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
            let host = UiNativePointerHostRead::from_world(world)?;
            let displayed = host
                .displayed_input(recipients, window, &mut slots)?
                .context("native mouse button has no completed displayed hit context")?;
            let point = lifetime
                .mouse
                .last_native_position()
                .map(|position| displayed_native_mouse_point(&displayed, position, modifiers))
                .transpose()?;
            let event = lifetime
                .mouse
                .button_input(device, state, button, point)
                .map_err(|error| {
                    anyhow!("native UI button translator rejected input: {error:?}")
                })?;
            let MouseButtonOutcome::Submit(event) = event else {
                // The pinned adapter tracks suppressed Down/Up pairs; they
                // carry neither UI claims nor gameplay permission.
                return Ok(());
            };
            let pointer = event.pointer_id();
            m13_submission_attempted = true;
            let (_, submission) = host
                .submit(
                    recipients,
                    NativePointerOrigin::new(window, device),
                    &mut slots,
                    event,
                )
                .map_err(|error| anyhow!("native button displayed admission failed: {error:?}"))?;
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .map_err(|error| anyhow!("native button M13 pump unavailable: {error:?}"))?;
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
                matches!(finality, Ok(UiInputFinality::Committed(_))),
                "native button lacks exact M13 pointer settlement: {finality:?}"
            );
            Ok(())
        })();
        world.insert_resource(slots);
        self.native_ui_mice.insert(window, lifetime);
        if let Err(error) = operation {
            self.cancel_native_ui_mouse(window)?;
            tracing::trace!(
                target: "engine.runtime.input",
                ?window,
                ?error,
                "native UI button withheld without gameplay grant"
            );
            if m13_submission_attempted {
                return Err(error.context("native UI button M13 submission or settlement failed"));
            }
        }
        Ok(())
    }

    /// Wheel is a transition on an authentic existing mouse stream. No
    /// cursor hit point, device identity or capture is fabricated from a
    /// native wheel event that arrived before a matching cursor observation.
    fn route_ui_mouse_wheel(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        backend_device: winit::event::DeviceId,
        context: InputContext,
        delta: &MouseScrollDelta,
    ) -> Result<()> {
        let zero = match delta {
            MouseScrollDelta::LineDelta(x, y) => *x == 0.0 && *y == 0.0,
            MouseScrollDelta::PixelDelta(point) => point.x == 0.0 && point.y == 0.0,
        };
        if zero {
            // Winit gesture-phase notifications with no movement have no
            // RunenUI wheel event even without a displayed frame or cursor.
            return Ok(());
        }
        let device = Self::native_ui_device(context)?;
        let Some(mut lifetime) = self.native_ui_mice.remove(&window) else {
            return Ok(());
        };
        if lifetime.recipient != slot
            || lifetime.binding_epoch != binding_epoch
            || lifetime.source != context.source
            || lifetime.backend_device != backend_device
            || lifetime.device != device
        {
            self.native_ui_mice.insert(window, lifetime);
            self.cancel_native_ui_mouse(window)?;
            return Ok(());
        }
        let modifiers = self
            .native_ui_modifiers
            .get(&window)
            .copied()
            .unwrap_or(KeyModifiers::NONE);
        let world = &mut self.state.world;
        let mut slots = match world.remove_resource::<UiRuntimeSlotsResource>() {
            Some(slots) => slots,
            None => {
                self.native_ui_mice.insert(window, lifetime);
                return Err(anyhow!("native UI wheel has no pinned slot authority"));
            }
        };
        // Before framework ingress a rejected observation is neutral-only;
        // after an actual M13 attempt, missing finality is a host error.
        let mut m13_submission_attempted = false;
        let operation = (|| -> Result<()> {
            let before = slots
                .input_ownership(slot)
                .map_err(|error| anyhow!("native wheel pre-state unavailable: {error:?}"))?;
            anyhow::ensure!(
                before.scope() == &lifetime.scope,
                "wheel stream crossed the original M13 scope"
            );
            let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
            let host = UiNativePointerHostRead::from_world(world)?;
            let displayed = host
                .displayed_input(recipients, window, &mut slots)?
                .context("native wheel has no terminal completed-Present context")?;
            let position = lifetime
                .mouse
                .last_native_position()
                .context("native wheel has no original cursor point")?;
            let point = displayed_native_mouse_point(&displayed, position, modifiers)?;
            let Some(logical_delta) =
                native_ui_logical_wheel_delta(delta, displayed.mapping.native_scale_factor())?
            else {
                return Ok(());
            };
            let event = lifetime
                .mouse
                .wheel(device, point, logical_delta)
                .map_err(|error| anyhow!("native wheel translator rejected input: {error:?}"))?;
            let pointer = event.pointer_id();
            m13_submission_attempted = true;
            let (_, submission) = host
                .submit(
                    recipients,
                    NativePointerOrigin::new(window, device),
                    &mut slots,
                    event,
                )
                .map_err(|error| anyhow!("native wheel admission rejected: {error:?}"))?;
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .map_err(|error| anyhow!("native wheel M13 pump unavailable: {error:?}"))?;
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
                matches!(finality, Ok(UiInputFinality::Committed(_))),
                "native UI wheel had no exact M13 pointer settlement: {finality:?}"
            );
            Ok(())
        })();
        world.insert_resource(slots);
        self.native_ui_mice.insert(window, lifetime);
        if let Err(error) = operation {
            self.cancel_native_ui_mouse(window)?;
            tracing::trace!(
                target: "engine.runtime.input",
                ?window,
                ?error,
                "native wheel withheld without gameplay grant"
            );
            if m13_submission_attempted {
                return Err(error.context("native UI wheel M13 submission or settlement failed"));
            }
        }
        Ok(())
    }

    /// Translate one original native touch callback through the one pinned
    /// contact normalizer and terminal displayed M13 pointer ledger. Multiple
    /// simultaneous devices/contacts remain distinct in TouchInputState.
    fn route_ui_touch(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        context: InputContext,
        original: Touch,
    ) -> Result<()> {
        let device = Self::native_ui_device(context)?;
        if self.native_ui_touches.get(&window).is_some_and(|old| {
            old.recipient != slot
                || old.binding_epoch != binding_epoch
                || old.source != context.source
        }) {
            self.cancel_native_ui_touch(window)?;
        }
        let world = &mut self.state.world;
        let mut slots = world
            .remove_resource::<UiRuntimeSlotsResource>()
            .context("selected native touch has no pinned slot authority")?;
        // Before framework ingress a rejected observation is neutral-only;
        // after an actual M13 attempt, missing finality is a host error.
        let mut m13_submission_attempted = false;
        let operation = (|| -> Result<()> {
            let before = slots
                .input_ownership(slot)
                .map_err(|error| anyhow!("native touch pre-state unavailable: {error:?}"))?;
            let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
            let host = UiNativePointerHostRead::from_world(world)?;
            let displayed = host
                .displayed_input(recipients, window, &mut slots)?
                .context("native touch has no terminal completed-Present context")?;
            let point = native_logical_mouse_point(
                original.location,
                displayed.mapping.native_scale_factor(),
            )?;
            let touches = &mut self.native_ui_touches;
            if touches
                .get(&window)
                .is_some_and(|old| old.scope != *before.scope())
            {
                touches.remove(&window);
            }
            let lifetime = touches
                .entry(window)
                .or_insert_with(|| NativeUiTouchTranslator {
                    recipient: slot,
                    source: context.source,
                    binding_epoch,
                    scope: before.scope().clone(),
                    backend_devices: HashSet::new(),
                    touch: TouchInputState::default(),
                });
            lifetime.backend_devices.insert(original.device_id);
            let event = lifetime
                .touch
                .transition(
                    device,
                    original.id,
                    original.phase,
                    point,
                    displayed.input_context.clone(),
                )
                .map_err(|error| anyhow!("native contact translation rejected: {error:?}"))?;
            if lifetime.touch.active_contact_count() == 0 {
                lifetime.backend_devices.clear();
            }
            let pointer = event.pointer_id();
            m13_submission_attempted = true;
            let (_, submission) = host
                .submit(
                    recipients,
                    NativePointerOrigin::new(window, device),
                    &mut slots,
                    event,
                )
                .map_err(|error| anyhow!("native touch admission rejected: {error:?}"))?;
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .map_err(|error| anyhow!("native touch M13 pump unavailable: {error:?}"))?;
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
                matches!(finality, Ok(UiInputFinality::Committed(_))),
                "native touch lacked exact M13 pointer settlement: {finality:?}"
            );
            Ok(())
        })();
        world.insert_resource(slots);
        if let Err(error) = operation {
            self.cancel_native_ui_touch(window)?;
            tracing::trace!(
                target: "engine.runtime.input",
                ?window,
                ?error,
                "native touch withheld without gameplay permission"
            );
            if m13_submission_attempted {
                return Err(error.context("native UI touch M13 submission or settlement failed"));
            }
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
        let mut cancellation_failure: Option<anyhow::Error> = None;
        let mouse_windows: Vec<_> = self.native_ui_mice.keys().copied().collect();
        for window in mouse_windows {
            if let Err(error) = self.cancel_native_ui_mouse(window)
                && cancellation_failure.is_none()
            {
                cancellation_failure = Some(error);
            }
        }
        let touch_windows: Vec<_> = self.native_ui_touches.keys().copied().collect();
        for window in touch_windows {
            if let Err(error) = self.cancel_native_ui_touch(window)
                && cancellation_failure.is_none()
            {
                cancellation_failure = Some(error);
            }
        }
        let cleanup = shutdown_native_ui(&mut self.state.world);
        self.shutdown_done = cleanup.is_ok();
        cleanup?;
        if let Some(error) = cancellation_failure {
            return Err(error);
        }
        Ok(())
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
                // A failed framework Cancel may stop this host, but must not
                // prevent the original neutral source and gameplay contributors
                // from retiring. Invoke both existing owners before propagating
                // either failure; never promote cleanup to an input authority.
                let recipient_retirement = self.retire_native_input_recipient(native_window_id);
                let source_retirement = self.retire_window_input_source(native_window_id);
                if native_window_id == NativeWindowId::primary() {
                    // Windows winit must not enter a blocking wait after exit.
                    event_loop.set_control_flow(ControlFlow::Poll);
                    event_loop.exit();
                    recipient_retirement?;
                    source_retirement?;
                    return Ok(());
                }
                recipient_retirement?;
                source_retirement?;
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
                self.native_ime_applied.remove(&native_window_id);
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

    /// Project the selected native slot's *current* M13 text capability to
    /// Winit's per-window OS IME switch. Absent/legacy/unfocused recipients
    /// are disabled; a temporarily unsettled framework is never granted.
    /// This runs after original input callbacks and UI frame settlement.
    fn project_native_ime(&mut self) -> Result<()> {
        for (_window_id, native_window_id, window) in self.window_realizer.realized_windows() {
            let enabled = if !window.has_focus() {
                false
            } else {
                match self.native_input_recipient(native_window_id) {
                    Ok((NativeInputRecipient::RunenUiSlot(slot), Some(_epoch))) => {
                        let slots = self
                            .state
                            .world
                            .resource_mut::<UiRuntimeSlotsResource>()
                            .context("native IME recipient lost its canonical RunenUI runtime")?;
                        match slots.input_ownership(slot) {
                            Ok(snapshot) => native_ime_allowed_for_m13(true, &snapshot),
                            Err(crate::plugins::ui::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce) => {
                                // Re-evaluate on the runtime's next genuine
                                // wake/pump, never reuse an earlier focus.
                                false
                            }
                            Err(error) => {
                                return Err(anyhow!(
                                    "native OS IME capability ownership unavailable: {error:?}"
                                ));
                            }
                        }
                    }
                    Ok((NativeInputRecipient::LegacyNativeConsumer, _)) => false,
                    Err(error)
                        if matches!(
                            error.downcast_ref::<NativeInputRecipientError>(),
                            Some(
                                NativeInputRecipientError::MissingRecipient
                                    | NativeInputRecipientError::InactiveWindow
                            )
                        ) =>
                    {
                        false
                    }
                    Err(error) => return Err(error),
                    Ok((NativeInputRecipient::RunenUiSlot(_), None)) => {
                        return Err(anyhow!("native IME slot selected without binding epoch"));
                    }
                }
            };
            if enabled != self.native_ime_applied.contains(&native_window_id) {
                window.set_ime_allowed(enabled);
                if enabled {
                    self.native_ime_applied.insert(native_window_id);
                } else {
                    self.native_ime_applied.remove(&native_window_id);
                }
            }
        }
        Ok(())
    }

    fn pace_or_exit(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(error) = self
            .project_native_ime()
            .and_then(|_| self.apply_frame_pacing(event_loop))
        {
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
                // Attempt framework cancellation before the original focus
                // transition, but report failure only after its mandatory
                // physical continuity and game-action revocation complete.
                let pointer_cancellation = if !focused {
                    let mouse = self.cancel_native_ui_mouse(native_window_id);
                    let touch = self.cancel_native_ui_touch(native_window_id);
                    mouse.and(touch)
                } else {
                    Ok(())
                };
                let cancellation = if !focused {
                    match self.native_input_recipient(native_window_id) {
                        Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                            // Preserve pinned reference ordering: cancel
                            // native keyboard contributors while composition
                            // is still active, then cancel that generation.
                            let keys = self.cancel_native_ui_keyboard(native_window_id, slot);
                            let composition = self.route_ui_ime_finish(slot, true);
                            keys.and(composition)
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
                pointer_cancellation.and(cancellation)
            }
            WindowEvent::Resized(size) => {
                let mouse = self.cancel_native_ui_mouse(native_window_id);
                let touch = self.cancel_native_ui_touch(native_window_id);
                self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::Resized {
                        width: size.width,
                        height: size.height,
                    },
                )?;
                mouse.and(touch)
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let mouse = self.cancel_native_ui_mouse(native_window_id);
                let touch = self.cancel_native_ui_touch(native_window_id);
                let size = window.inner_size();
                self.apply_event_for_native_window(
                    native_window_id,
                    PlatformEvent::ScaleFactorChanged {
                        scale_factor,
                        width: size.width,
                        height: size.height,
                    },
                )?;
                mouse.and(touch)
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
            WindowEvent::Ime(Ime::Enabled) => Ok(()),
            WindowEvent::Ime(Ime::Preedit(preedit, range)) => {
                match self.native_input_recipient(native_window_id) {
                    Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                        self.route_ui_ime_preedit(slot, preedit, range)
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
            }
            WindowEvent::Ime(Ime::Disabled) => {
                match self.native_input_recipient(native_window_id) {
                    Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                        self.route_ui_ime_finish(slot, true)
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
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                // Empty OS Commit still ends a live preedit generation, but
                // must never manufacture empty durable text.
                if text.is_empty() {
                    return match self.native_input_recipient(native_window_id) {
                        Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                            self.route_ui_ime_finish(slot, false)
                        }
                        Ok((NativeInputRecipient::LegacyNativeConsumer, _)) => Ok(()),
                        Err(error)
                            if error.downcast_ref::<NativeInputRecipientError>()
                                == Some(&NativeInputRecipientError::MissingRecipient) =>
                        {
                            Ok(())
                        }
                        Err(error) => Err(error),
                    };
                }
                let selected = self.native_input_recipient(native_window_id);
                match selected {
                    Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                        // This journal entry is original native text evidence,
                        // never a second legacy Editor delivery.
                        self.apply_event_group_for_recipient(
                            native_window_id,
                            [PlatformEvent::TextInput { text: text.clone() }],
                            Some(NativeInputRecipient::RunenUiSlot(slot)),
                        )?;
                        self.route_ui_ime_commit(slot, text)
                    }
                    Ok((NativeInputRecipient::LegacyNativeConsumer, _)) => {
                        // The accepted legacy Editor path derives text from its
                        // original keyboard observations. Do not double-commit
                        // it through an additional IME event at this boundary.
                        Ok(())
                    }
                    Err(error) => {
                        self.apply_event_group_for_recipient(
                            native_window_id,
                            [PlatformEvent::TextInput { text }],
                            None,
                        )?;
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
                let original_delta = delta;
                let input = scroll_input(delta, phase);
                self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::MouseWheel { context, input },
                )?;
                if let Ok((NativeInputRecipient::RunenUiSlot(slot), Some(epoch))) =
                    self.native_input_recipient(native_window_id)
                {
                    self.route_ui_mouse_wheel(
                        native_window_id,
                        slot,
                        epoch,
                        device_id,
                        context,
                        &original_delta,
                    )?;
                }
                self.request_redraw_for_native_window(native_window_id);
                Ok(())
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
            WindowEvent::CursorLeft { .. } => self.cancel_native_ui_mouse(native_window_id),
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, device_id)?;
                let input = pointer_button_input(state, button);
                self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::MouseInput { context, input },
                )?;
                if let Ok((NativeInputRecipient::RunenUiSlot(slot), Some(epoch))) =
                    self.native_input_recipient(native_window_id)
                {
                    self.route_ui_mouse_button(
                        native_window_id,
                        slot,
                        epoch,
                        context,
                        state,
                        button,
                    )?;
                }
                self.request_redraw_for_native_window(native_window_id);
                Ok(())
            }
            WindowEvent::Touch(touch) => {
                let context = self
                    .input_adapter
                    .window_context(native_window_id, touch.device_id)?;
                let original_touch = touch;
                let input = contact_input(touch);
                self.apply_semantic_native_input(
                    native_window_id,
                    PlatformEvent::Touch { context, input },
                )?;
                if let Ok((NativeInputRecipient::RunenUiSlot(slot), Some(epoch))) =
                    self.native_input_recipient(native_window_id)
                {
                    self.route_ui_touch(native_window_id, slot, epoch, context, original_touch)?;
                }
                self.request_redraw_for_native_window(native_window_id);
                Ok(())
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

    #[test]
    fn native_wheel_respects_displayed_scale_explicit_line_policy_and_inert_zero() {
        let lines = native_ui_logical_wheel_delta(&MouseScrollDelta::LineDelta(1.5, -2.0), 3.0)
            .expect("finite line wheel")
            .expect("nonzero displacement");
        assert_eq!(lines.x(), 90.0);
        assert_eq!(lines.y(), -120.0);
        let pixels = native_ui_logical_wheel_delta(
            &MouseScrollDelta::PixelDelta(PhysicalPosition::new(120.0, -40.0)),
            2.0,
        )
        .expect("display-scaled native wheel")
        .expect("nonzero displacement");
        assert_eq!(pixels.x(), 60.0);
        assert_eq!(pixels.y(), -20.0);
        assert!(
            native_ui_logical_wheel_delta(&MouseScrollDelta::LineDelta(0.0, -0.0), 0.0,)
                .expect("phase-only zero is inert without scale")
                .is_none()
        );
        assert!(
            native_ui_logical_wheel_delta(
                &MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, 0.0)),
                0.0,
            )
            .expect("phase-only zero physical wheel is inert without display")
            .is_none()
        );
        assert!(
            native_ui_logical_wheel_delta(
                &MouseScrollDelta::PixelDelta(PhysicalPosition::new(5.0, 0.0)),
                0.0,
            )
            .is_err(),
            "nonzero physical pixels cannot invent a missing display scale"
        );
        assert!(
            native_ui_logical_wheel_delta(
                &MouseScrollDelta::PixelDelta(PhysicalPosition::new(f64::NAN, 0.0)),
                2.0,
            )
            .is_err(),
            "nonfinite wheel must not enter M13"
        );
    }

    /// Exercise the same pinned typed RunenUI ownership as the native host:
    /// a genuine composition-capable focus is required before Active may be
    /// given to the pinned winit keyboard translator.
    #[test]
    fn native_keyboard_composition_tracks_real_m13_generation_and_cancellation() {
        use runenui_core::{
            CommandOrigin, Effects, IntoUpdateOutput, NoHostProtocol, SemanticCommand,
            SemanticEditableMode, TextAffinity, TextDocumentId, TextDocumentRevision,
            TextDocumentSnapshot, TextPosition, TextSelection, UiApp, UpdateOutput, View,
        };
        use runenui_runtime::{AppRuntime, PumpBudget};

        struct CompositionProbe;
        impl UiApp for CompositionProbe {
            type State = ();
            type Action = ();
            type HostProtocol = NoHostProtocol;

            fn root(_: &Self::State) -> impl View<Self::Action> {
                let document = TextDocumentSnapshot::new(
                    TextDocumentId::new(81),
                    TextDocumentRevision::new(0),
                );
                let position = TextPosition::new(document, "", 0, TextAffinity::Downstream)
                    .expect("empty text position is valid");
                runenui_core::text_field(
                    document,
                    String::new(),
                    TextSelection::collapsed(position),
                    SemanticEditableMode::SingleLine,
                    |_| (),
                )
                .expect("valid composition-capable editor")
                .id("native.composition.probe")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl IntoUpdateOutput<Self::Action, Self::HostProtocol> {
                UpdateOutput::effects(Effects::none())
            }
        }

        let budget = PumpBudget::new(usize::MAX, usize::MAX, usize::MAX, usize::MAX);
        let mut runtime = AppRuntime::<CompositionProbe>::mount(());
        runtime
            .pump(budget)
            .expect("initial mounted reconciliation");
        let initially_unfocused = runtime
            .input_ownership()
            .expect("initial RunenUI ownership");
        assert!(
            !native_ime_allowed_for_m13(true, &initially_unfocused),
            "a mounted text field without semantic focus cannot enable native OS IME"
        );
        let field = runtime
            .index()
            .nodes()
            .first()
            .expect("one text-capable mounted field")
            .id()
            .clone();
        runtime
            .submit_command(
                field,
                SemanticCommand::RequestFocus,
                CommandOrigin::programmatic(),
            )
            .expect("mounted text field accepts focus");
        runtime
            .pump(budget)
            .expect("focus has canonical settlement");
        let before = runtime.input_ownership().expect("real M13 focus snapshot");
        assert!(
            native_ime_allowed_for_m13(true, &before),
            "a text-capable focused M13 owner enables its selected native window's OS IME"
        );
        assert!(
            !native_ime_allowed_for_m13(false, &before),
            "losing native OS focus must disable IME despite retained semantic text focus"
        );
        assert_eq!(
            native_keyboard_composition(&before),
            KeyboardCompositionState::Inactive
        );
        let composition = runtime
            .start_composition(None)
            .expect("focused field accepts a genuine composition generation");
        runtime.pump(budget).expect("composition start settled");
        let composing = runtime
            .input_ownership()
            .expect("owner-authored active composition snapshot");
        assert_eq!(
            native_keyboard_composition(&composing),
            KeyboardCompositionState::Active,
            "native shortcut translation MUST NOT treat actual IME as inactive"
        );
        runtime
            .cancel_composition(composition.generation().clone())
            .expect("framework generation owner accepts cancellation");
        runtime
            .pump(budget)
            .expect("composition cancellation settled");
        let after = runtime
            .input_ownership()
            .expect("composition cancellation ownership observed");
        assert_eq!(
            native_keyboard_composition(&after),
            KeyboardCompositionState::Inactive,
            "only the framework's retired generation ends composition"
        );
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
            native_ui_touches: HashMap::new(),
            native_ui_modifiers: HashMap::new(),
            native_ime_applied: HashSet::new(),
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
            native_ui_touches: HashMap::new(),
            native_ui_modifiers: HashMap::new(),
            native_ime_applied: HashSet::new(),
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
    fn close_approved_window_retires_original_source_without_false_inactive_route_failure() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        // A passive UI host has an input-recipient registry but no selected
        // game/UI fallback. The original native close approval itself moves
        // the window out of the Created state.
        runner
            .state
            .world
            .insert_resource(UiRuntimeSlotsResource::default());
        runner
            .state
            .world
            .insert_resource(UiNativeInputRecipientsResource::default());
        let secondary = NativeWindowId::try_from_raw(2).expect("secondary window identity");
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native window authority")
            .register_created_window(secondary, "secondary", (800, 600), 1.0, false);
        let context = runner
            .input_adapter
            .window_context(secondary, winit::event::DeviceId::dummy())
            .expect("original native source");
        runner
            .apply_event_for_native_window(secondary, normalized_shift_press(context))
            .expect("physical Down reached the neutral reducer");
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("neutral state")
                .shift_down()
        );
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("native window authority")
            .record_mut(secondary)
            .expect("created secondary")
            .request_close();
        assert!(
            runner
                .retire_window_input_source(secondary)
                .expect("approved close must retire original source without requiring Created"),
        );
        assert_eq!(runner.input_adapter.window_source_context(secondary), None);
        assert!(
            !runner
                .state
                .world
                .resource::<InputState>()
                .expect("physical state")
                .shift_down(),
            "physical key must be retired before the approved close completes"
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
    fn saturated_original_journal_cannot_cancel_ui_before_physical_source_retirement() {
        use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};

        struct Probe;
        impl runenui_core::UiApp for Probe {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("original source lifetime probe")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let secondary = NativeWindowId::try_from_raw(2).expect("secondary native window");
        let context = runner
            .input_adapter
            .window_context(secondary, winit::event::DeviceId::dummy())
            .expect("original checked source and device");
        runner
            .apply_event_for_native_window(secondary, normalized_shift_press(context))
            .expect("neutral keyboard Down precedes the saturated journal");
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<Probe>(
                UiTypedScreenId::new("native.source.retirement.preflight"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("real typed framework runtime");
        runner.state.world.insert_resource(slots);
        let device = WinitRunner::native_ui_device(context).expect("original checked device");
        runner.native_ui_keyboard_translator(secondary, slot, context.source, device, 1);
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("one physical input reducer")
                .shift_down()
        );
        // Exactly one original keyboard Down already occupies the journal.
        // Force the existing finite 4096-event frame admission boundary.
        let journal = runner
            .state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .expect("one Engine native journal");
        for _ in 0..4095 {
            journal
                .publish(PlatformWindowEvent::new(
                    secondary,
                    PlatformEvent::RedrawRequested,
                ))
                .expect("remaining bounded journal slots");
        }
        assert_eq!(journal.events().len(), 4096);
        assert!(
            runner.retire_window_input_source(secondary).is_err(),
            "journal exhaustion must fail before framework cancellation or neutral retirement"
        );
        assert_eq!(
            runner.input_adapter.window_source_context(secondary),
            Some(InputContext::new(context.source, None)),
            "a failed continuity admission must retain the original window source, not invent a window device identity"
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("original physical truth")
                .shift_down(),
            "the failed source-loss journal must not silently erase the physical Down"
        );
        assert!(
            runner
                .native_ui_keyboards
                .get(&secondary)
                .is_some_and(|translator| translator.devices.contains(&device)),
            "pre-admission failure must not discard the original RunenUI translator lifetime"
        );
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
        // A native button with no prior authentic cursor position cannot
        // manufacture a logical hit point or a new RunenUI pressed stream.
        let unknown_position_context = runner
            .input_adapter
            .window_context(secondary, winit::event::DeviceId::dummy())
            .expect("original normalized window-event input context");
        runner
            .route_ui_mouse_button(
                secondary,
                slot,
                bound_epoch,
                unknown_position_context,
                winit::event::ElementState::Pressed,
                winit::event::MouseButton::Left,
            )
            .expect("missing native point is a deliberate UI nonclaim");
        assert!(
            !runner.native_ui_mice.contains_key(&secondary),
            "a button without original cursor stream may not synthesize a pointer lifetime"
        );
        runner
            .route_ui_mouse_wheel(
                secondary,
                slot,
                bound_epoch,
                winit::event::DeviceId::dummy(),
                unknown_position_context,
                &MouseScrollDelta::LineDelta(1.0, 0.0),
            )
            .expect("wheel without authenticated mouse cursor must be a nonclaim");
        assert!(
            !runner.native_ui_mice.contains_key(&secondary),
            "wheel cannot synthesize an original cursor stream"
        );
        runner
            .route_ui_mouse_wheel(
                secondary,
                slot,
                bound_epoch,
                winit::event::DeviceId::dummy(),
                unknown_position_context,
                &MouseScrollDelta::LineDelta(0.0, 0.0),
            )
            .expect("phase-only wheel must be inert before display checks");
        runner
            .route_ui_touch(
                secondary,
                slot,
                bound_epoch,
                unknown_position_context,
                Touch {
                    device_id: winit::event::DeviceId::dummy(),
                    phase: winit::event::TouchPhase::Started,
                    location: PhysicalPosition::new(18.0, 22.0),
                    force: None,
                    id: 42,
                },
            )
            .expect("unpresented native touch must remain an explicit nonclaim");
        assert!(
            !runner.native_ui_touches.contains_key(&secondary),
            "a touch cannot allocate a live input stream without terminal Present"
        );
        // A removed modifier-carrying keyboard must not lend its old
        // window-global Shift/Ctrl snapshot to device B. B's translator
        // membership remains live and the next authentic modifier callback
        // may reestablish the aggregate state.
        let device_a = UiInputDeviceId::new(31).expect("device A identity");
        let device_b = UiInputDeviceId::new(32).expect("device B identity");
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            unknown_position_context.source,
            device_a,
            bound_epoch,
        );
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            unknown_position_context.source,
            device_b,
            bound_epoch,
        );
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        runner
            .cancel_native_ui_keyboard_device(secondary, device_a)
            .expect("device A empty-key stream retires in its original M13 scope");
        assert!(
            runner
                .native_ui_keyboards
                .get(&secondary)
                .is_some_and(
                    |state| state.devices.contains(&device_b) && !state.devices.contains(&device_a)
                ),
            "device B's original translator state survives A removal"
        );
        assert!(
            !runner.native_ui_modifiers.contains_key(&secondary),
            "A removal invalidates the unattributed window modifier snapshot"
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
        // Device B must not replace device A's translator and silently
        // discard its framework-owned pressed-key state or host modifiers.
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            source_a,
            UiInputDeviceId::new(8).expect("distinct checked test device"),
            bound_epoch,
        );
        let original = runner
            .native_ui_keyboards
            .get(&secondary)
            .expect("same-window multi-device lifetime");
        assert_eq!(original.source, source_a);
        assert_eq!(original.devices.len(), 2);
        assert!(
            original
                .devices
                .contains(&UiInputDeviceId::new(7).expect("device A"))
        );
        assert!(
            original
                .devices
                .contains(&UiInputDeviceId::new(8).expect("device B"))
        );
        assert_eq!(
            runner.native_ui_modifiers.get(&secondary),
            Some(&KeyModifiers::SHIFT),
            "a second native keyboard must not reset the first stream's modifiers"
        );
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
        // A second original keyboard sharing the same window, source, slot
        // and binding epoch is NOT a replacement lifetime. It cannot evict
        // device A's pressed state or reset the live window modifiers.
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        let second_device = UiInputDeviceId::new(8).expect("checked second device");
        runner.native_ui_keyboard_translator(secondary, slot, source_b, second_device, new_epoch);
        assert_eq!(
            runner.native_ui_modifiers.get(&secondary),
            Some(&KeyModifiers::SHIFT),
            "simultaneous keyboard B must not clear the existing modifier observation"
        );
        let lifetime = runner
            .native_ui_keyboards
            .get(&secondary)
            .expect("shared native keyboard lifetime remains available");
        assert_eq!(lifetime.devices.len(), 2);
        assert!(
            lifetime
                .devices
                .contains(&UiInputDeviceId::new(7).expect("device A remains live"))
        );
        assert!(lifetime.devices.contains(&second_device));

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
            runner
                .native_ui_keyboards
                .get(&secondary)
                .is_some_and(|lifetime| lifetime.devices.is_empty()),
            "device removal retires only its membership, not the reusable window translator"
        );
        assert!(
            !runner.native_ui_modifiers.contains_key(&secondary),
            "last device removal must clear stale physical modifier evidence"
        );
        let successor = runner
            .input_adapter
            .window_context(secondary, backend)
            .expect("device can reappear with another checked generation");
        assert_eq!(successor.source, original.source);
        assert_ne!(successor.device, original.device);
        let scope = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("mounted framework is retained")
            .input_ownership(slot)
            .expect("live UI M13 scope")
            .scope()
            .clone();
        let actual_context = runner
            .input_adapter
            .window_context(secondary, winit::event::DeviceId::dummy())
            .expect("fresh device generation");
        runner.native_ui_mice.insert(
            secondary,
            NativeUiMouseTranslator {
                recipient: slot,
                source: actual_context.source,
                backend_device: winit::event::DeviceId::dummy(),
                device: WinitRunner::native_ui_device(actual_context)
                    .expect("checked native UI device"),
                binding_epoch: new_epoch,
                scope,
                mouse: MouseInputState::default(),
            },
        );
        assert!(
            runner
                .retire_native_input_recipient(secondary)
                .expect("explicit UI input recipient retirement")
        );
        assert!(
            !runner.native_ui_mice.contains_key(&secondary),
            "no previous mouse stream can survive semantic recipient retirement"
        );
        assert!(
            runner.state.world.has_resource::<UiRuntimeSlotsResource>(),
            "retirement rejection cannot discard the native UI runtime"
        );
        // A retained UI touch backend with no original window-device mapping
        // must never be silently reported as a successfully retired contact.
        let stale_backend = winit::event::DeviceId::dummy();
        let retired = runner
            .input_adapter
            .retire_window_device(stale_backend)
            .expect("checked physical identity retires");
        assert!(!retired.is_empty());
        let stale_scope = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("mounted UI source remains")
            .input_ownership(slot)
            .expect("original framework scope")
            .scope()
            .clone();
        runner.native_ui_touches.insert(
            secondary,
            NativeUiTouchTranslator {
                recipient: slot,
                source: actual_context.source,
                binding_epoch: new_epoch,
                scope: stale_scope,
                backend_devices: HashSet::from([stale_backend]),
                touch: TouchInputState::default(),
            },
        );
        let missing = runner
            .apply_raw_device_removal(stale_backend)
            .expect_err("missing original touch device ID must fail closed");
        assert!(
            format!("{missing:#}").contains("lacks its original window-device identity"),
            "diagnostic retains the missing original identity: {missing:#}"
        );
        assert!(
            runner
                .native_ui_touches
                .get(&secondary)
                .is_some_and(|state| state.backend_devices.contains(&stale_backend)),
            "a missing identity must not be reported as a settled M13 Cancel"
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
