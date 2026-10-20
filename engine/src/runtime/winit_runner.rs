use crate::app::WindowedAppState;
use crate::plugins::render::host::native as render_native_host;
use crate::plugins::ui::{
    NativeInputRecipient, NativeInputRecipientError, NativePointerOrigin,
    UiNativeInputRecipientsResource, UiNativePointerHostRead, UiRuntimeDisplayedPresentation,
    UiRuntimeNativeCompositionAdmissionError, UiRuntimeNativeTextAdmissionError,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotId, UiRuntimeSlotsResource,
    native_composition_finality, native_keyboard_finality,
    native_no_focused_keyboard_claim_witness, native_original_keyboard_release_nonclaim,
    native_original_keyboard_repeat_nonclaim, native_pointer_finality, native_text_finality,
    native_unfocused_game_keyboard_snapshot,
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
    apply_native_recipient_input_event, apply_native_window_event,
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
    KeyboardDownAdmission, KeyboardDownDecision, KeyboardIngressDiagnostic, KeyboardInputOutcome,
    KeyboardInputState, NativeKeyTransition,
};
use runenui_winit::mouse_input::{MouseButtonOutcome, MouseInputState, TranslatedPointerPoint};
use runenui_winit::text_input::keyboard_committed_text_candidate;
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

/// Original native IME text requires both live window focus and the host's
/// applied OS IME permission. A delayed OS payload after either lifetime
/// retires is not authority to submit new RunenUI text or composition work.
fn native_ime_payload_allowed_for_window(focused: bool, ime_permission_applied: bool) -> bool {
    focused && ime_permission_applied
}

/// Original KeyEvent.text belongs to the focus lifetime observed before the
/// corresponding native keyboard event, not a successor selected by that
/// keyboard's own M13 processing. The public revision catches A-B-A changes
/// even when the same node has focus again; Engine stores no focus session.
fn native_keyboard_text_owner_unchanged(
    before: &runenui_runtime::InputOwnershipSnapshot,
    after: &runenui_runtime::InputOwnershipSnapshot,
) -> bool {
    before.status() == runenui_runtime::RuntimeStatus::Running
        && after.status() == runenui_runtime::RuntimeStatus::Running
        && before.scope() == after.scope()
        && before.revision() == after.revision()
        && before.keyboard().focused_node().is_some()
        && before.keyboard().focused_node() == after.keyboard().focused_node()
        && after
            .keyboard()
            .text_input_capability()
            .accepts_committed_text()
        && after.keyboard().composition_generation().is_none()
}

fn native_ui_blocks_preframe_keyboard_gameplay(
    ownership: &runenui_runtime::InputOwnershipSnapshot,
) -> bool {
    let keyboard = ownership.keyboard();
    let text = keyboard.text_input_capability();
    ownership.status() != runenui_runtime::RuntimeStatus::Running
        || (keyboard.focused_node().is_some()
            && (text.accepts_committed_text() || text.accepts_composition()))
        || keyboard.composition_generation().is_some()
        || ownership
            .surfaces()
            .iter()
            .any(|surface| surface.modal_blocker().is_some())
}

/// A passive OS-effect projection may observe M13 while work is pending.
/// The real recipient resolver reports this before any secondary ownership
/// lookup can run. Deny OS IME temporarily; ordinary input admission still
/// fails closed on an unsettled runtime.
fn native_ime_waiting_for_m13_settlement(error: &anyhow::Error) -> bool {
    matches!(
        error.downcast_ref::<NativeInputRecipientError>(),
        Some(NativeInputRecipientError::OwnershipUnavailable(
            crate::plugins::ui::UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce
        ))
    )
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
    /// Last actual M13 revision observed for this exact native UI keyboard
    /// lifetime. This is only a continuity witness, never copied focus,
    /// permission, or a second held-key store.
    last_observed_revision: Option<u64>,
    keyboard: KeyboardInputState,
}

/// One original Winit keyboard callback and its pre-admission neutral Down
/// generation. This is ephemeral source evidence, not an input queue or a
/// second physical-key, framework-focus, or gameplay-permission authority.
struct NativeUiKeyboardOriginal<'a> {
    event: &'a winit::event::KeyEvent,
    is_synthetic: bool,
    previous_down_generation: Option<u64>,
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

impl NativeUiTouchTranslator {
    /// A rejected native transition does not mutate the canonical contact
    /// table or the backend membership index. One device's duplicate/orphan
    /// event must not retire another device's valid contact.
    fn transition(
        &mut self,
        original: &Touch,
        device: UiInputDeviceId,
        point: LogicalPoint,
        input_context: runenui_core::SurfaceInputContext,
    ) -> Result<runenui_core::PointerEvent, runenui_winit::touch_input::TouchIngressDiagnostic>
    {
        let event =
            self.touch
                .transition(device, original.id, original.phase, point, input_context)?;
        self.backend_devices.insert(original.device_id);
        if self.touch.active_contact_count() == 0 {
            self.backend_devices.clear();
        }
        Ok(event)
    }
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
    // The ECS World owns the *one* real winit source allocator. A native
    // mutation can now resolve its original source without a mirrored map
    // or an unverifiable primary-window assumption.
    if !world.has_resource::<WinitInputAdapter>() {
        world.insert_resource(WinitInputAdapter::default());
    }
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
    /// The original winit identity allocator is directly World-owned, not
    /// cloned into a second native/UI source authority.
    fn native_input(&self) -> &WinitInputAdapter {
        self.state
            .world
            .resource::<WinitInputAdapter>()
            .expect("native host must install the original input allocator")
    }

    fn native_input_mut(&mut self) -> &mut WinitInputAdapter {
        self.state
            .world
            .resource_mut::<WinitInputAdapter>()
            .expect("native host must install the original input allocator")
    }

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

    /// Reserve the complete original focus/continuity occurrence before any
    /// derived M13 cancellation can mutate a framework-owned input lifetime.
    /// The native callback is synchronous: no other journal producer may
    /// interleave between this preflight and the final original group.
    fn preflight_native_focus_loss(&mut self, window: NativeWindowId) -> Result<()> {
        let original_source = self.native_input().window_source_context(window);
        self.state
            .world
            .resource::<WindowStateRegistryResource>()?
            .record(window)
            .context("native focus-loss preflight targets an unknown window")?;
        if original_source.is_some() {
            self.state
                .world
                .resource::<InputState>()
                .context("native focus-loss preflight has no physical input owner")?;
        }
        let journal = self
            .state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .context("native focus-loss preflight has no original journal")?;
        if let Some(context) = original_source {
            journal.reserve_native_group(&[
                PlatformEvent::Focused { focused: false },
                PlatformEvent::InputContinuityLost {
                    context,
                    loss: ContinuityLoss::Source,
                },
            ])?;
        } else {
            journal.reserve_native_group(&[PlatformEvent::Focused { focused: false }])?;
        }
        Ok(())
    }

    fn apply_native_focus_change(
        &mut self,
        window: NativeWindowId,
        focused: bool,
    ) -> Result<()> {
        if !focused {
            // Without this reservation, a full journal can reject physical
            // focus loss after UI translators already issued their Cancels.
            self.preflight_native_focus_loss(window)?;
        }
        // Preserve the accepted framework order: pointer cancellation first,
        // then keyboard cancellation while composition is active, then IME.
        let pointer_cancellation = if !focused {
            let mouse = self.cancel_native_ui_mouse(window);
            let touch = self.cancel_native_ui_touch(window);
            mouse.and(touch)
        } else {
            Ok(())
        };
        let cancellation = if !focused {
            match self.native_input_recipient(window) {
                Ok((NativeInputRecipient::RunenUiSlot(slot), _)) => {
                    let keys = self.cancel_native_ui_keyboard(window, slot);
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
        // Once the original group is admitted, any framework failure remains
        // fatal, but may not suppress physical continuity or game revocation.
        self.apply_window_focus_event(window, focused)?;
        pointer_cancellation.and(cancellation)
    }

    fn apply_window_focus_event(
        &mut self,
        native_window_id: NativeWindowId,
        focused: bool,
    ) -> Result<()> {
        if !focused
            && let Some(context) = self.native_input().window_source_context(native_window_id)
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
        let original_source = self.native_input().window_source_context(native_window_id);
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
            .native_input_mut()
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
            // Recipient retirement is itself an input-permission boundary.
            // The original window source may remain physically held while a
            // new UI recipient is bound. No old game-held generation may be
            // carried into that replacement merely because M13 is unfocused.
            if let Some(context) = self.native_input().window_source_context(window) {
                self.revoke_native_game_keyboard_source(context.source)?;
            }
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
        if self
            .state
            .world
            .has_resource::<UiNativeInputRecipientsResource>()
        {
            // Native startup is not a special authority. Use the same
            // source-correlated World transaction as a later ECS rebind so a
            // preexisting held game key cannot survive this first selection.
            crate::plugins::ui::realize_requested_native_primary(&mut self.state.world)?;
        }
        Ok(())
    }

    fn native_input_recipient(
        &mut self,
        window: NativeWindowId,
    ) -> Result<(NativeInputRecipient, Option<u64>)> {
        self.native_input_recipient_with_close_policy(window, false)
    }

    fn native_input_recipient_for_cancellation(
        &mut self,
        window: NativeWindowId,
    ) -> Result<(NativeInputRecipient, Option<u64>)> {
        self.native_input_recipient_with_close_policy(window, true)
    }

    fn native_input_recipient_with_close_policy(
        &mut self,
        window: NativeWindowId,
        allow_approved_close: bool,
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
                let recipient = if allow_approved_close {
                    recipients.resolve_for_cancellation(window, windows, &mut slots)?
                } else {
                    recipients.resolve(window, windows, &mut slots)?
                };
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
                    last_observed_revision: None,
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
    /// Admit one original normalized keyboard observation only after checking
    /// any PREVIOUS M13 revision gap. This is the same boundary the actual
    /// Winit callback calls; tests must not simulate a reordered manual pair.
    fn admit_native_ui_keyboard_for_recipient(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        context: InputContext,
        input: runen_input::KeyboardInput,
        native_text: Option<String>,
    ) -> Result<()> {
        let physical_key = input.physical_key.clone();
        let pressed = input.state == runen_input::DigitalState::Pressed;
        let preflight = self.reconcile_native_ui_keyboard_before_ingress(
            window,
            slot,
            binding_epoch,
            context.source,
        );
        // Original KeyEvent.text belongs to this SAME Winit callback as its
        // physical keyboard observation, even though M13 may later reject
        // text for focus/capability/composition reasons. Reserve the full
        // original group before admitting either neutral observation.
        // UI-directed text is never dispatched to the legacy Editor.
        let recipient = Some(NativeInputRecipient::RunenUiSlot(slot));
        match native_text {
            Some(text) => self.apply_event_group_for_recipient(
                window,
                [
                    PlatformEvent::KeyboardInput { context, input },
                    PlatformEvent::TextInput { text },
                ],
                recipient,
            )?,
            None => self.apply_event_group_for_recipient(
                window,
                [PlatformEvent::KeyboardInput { context, input }],
                recipient,
            )?,
        }
        if let Err(error) = preflight {
            // Failed pre-observation cannot suppress the original neutral
            // Down or permit it as a game-held action.
            if pressed {
                self.deny_native_keyboard_lifetime(context, &physical_key)?;
            }
            return Err(error);
        }
        Ok(())
    }

    /// Reconcile an older M13 ownership interval before admitting the next
    /// genuine physical Down. Otherwise a source-wide revision-gap withdrawal
    /// would also revoke that newly admitted, independently settled generation.
    /// This is a witness over RunenUI's real ownership, not an Engine focus cache.
    fn reconcile_native_ui_keyboard_before_ingress(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        source: InputSourceId,
    ) -> Result<()> {
        let observed = match self.state.world.resource_mut::<UiRuntimeSlotsResource>() {
            Ok(slots) => slots.input_ownership(slot).map_err(|error| {
                anyhow!("native pre-ingress M13 ownership unavailable: {error:?}")
            }),
            Err(error) => Err(anyhow!(
                "selected native UI runtime vanished before physical admission: {error:?}"
            )),
        };
        let revision = match observed {
            Ok(snapshot) => snapshot.revision().get(),
            Err(error) => {
                // Unavailable ownership cannot leave previously projected
                // held game actions readable while a new original Down arrives.
                self.revoke_native_game_keyboard_source(source)?;
                return Err(error);
            }
        };
        self.reconcile_native_ui_keyboard_revision(window, slot, binding_epoch, source, revision)
    }

    /// Only the matching translator's framework revision is a continuity
    /// witness. Do not substitute current focus for missing intermediate M13
    /// ownership history, and do not touch another source or recipient.
    fn reconcile_native_ui_keyboard_revision(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        source: InputSourceId,
        current_revision: u64,
    ) -> Result<()> {
        let previous = self.native_ui_keyboards.get(&window).and_then(|lifetime| {
            (lifetime.recipient == slot
                && lifetime.source == source
                && lifetime.binding_epoch == binding_epoch)
                .then_some(lifetime.last_observed_revision)
                .flatten()
        });
        if previous.is_some_and(|revision| revision != current_revision) {
            self.revoke_native_game_keyboard_source(source)?;
        }
        if let Some(lifetime) = self.native_ui_keyboards.get_mut(&window)
            && lifetime.recipient == slot
            && lifetime.source == source
            && lifetime.binding_epoch == binding_epoch
        {
            lifetime.last_observed_revision = Some(current_revision);
        }
        Ok(())
    }

    /// Resolve the one adapter-owned pending original Down with the actual
    /// M13 queue-admission outcome, not its later settlement or game decision.
    /// This is a same-turn acknowledgment, never a second keyboard ledger.
    fn resolve_native_ui_down_admission(
        &mut self,
        window: NativeWindowId,
        admission: KeyboardDownAdmission,
        decision: KeyboardDownDecision,
    ) -> Result<()> {
        self.native_ui_keyboards
            .get_mut(&window)
            .context(
                "original native keyboard translator disappeared before admission acknowledgment",
            )?
            .keyboard
            .resolve_down_admission(admission, decision)
            .map_err(|error| {
                anyhow!("original native keyboard admission acknowledgment failed: {error:?}")
            })
    }

    fn route_ui_keyboard(
        &mut self,
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        binding_epoch: u64,
        context: InputContext,
        original: NativeUiKeyboardOriginal<'_>,
    ) -> Result<()> {
        let NativeUiKeyboardOriginal {
            event: original,
            is_synthetic,
            previous_down_generation,
        } = original;
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
        // A semantic/host mutation since this original keyboard lifetime's
        // last observed M13 revision may have crossed an exclusive owner and
        // returned to nonexclusive before this native callback. Without all
        // intervening ordered M13 records it is not a gameplay nonclaim.
        self.reconcile_native_ui_keyboard_revision(
            window,
            slot,
            binding_epoch,
            context.source,
            before.revision().get(),
        )?;
        // Revalidate the translator's actual recipient and physical-source
        // generation BEFORE reusing cached modifier or pressed-key evidence.
        self.native_ui_keyboard_translator(window, slot, context.source, device, binding_epoch);
        self.native_ui_keyboards
            .get_mut(&window)
            .expect("pinned native keyboard translator was just initialized")
            .last_observed_revision = Some(before.revision().get());
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
        // The same original Winit key may carry ordinary committed text.
        // Only a successful exact M13 keyboard settlement may authorize
        // a separate source-correlated text submission in that same turn.
        let mut committed_keyboard_text = None;
        // A genuine repeat retains its prior physical lifetime. This never
        // grants a new game Down, restores a revoked held action, or manufactures
        // an M13 sequence; ActionState remains the only gameplay permission owner.
        let mut preserve_prior_held_permission = false;
        let permit = match translated {
            KeyboardInputOutcome::Submit {
                event,
                down_admission,
            } => {
                let text_candidate = keyboard_committed_text_candidate(
                    original.state,
                    is_synthetic,
                    self.native_ime_applied.contains(&window)
                        && before.keyboard().focused_node().is_some()
                        && before
                            .keyboard()
                            .text_input_capability()
                            .accepts_committed_text(),
                    composition,
                    event.logical_key(),
                    original.text.as_deref(),
                )
                .map(str::to_owned);
                let submission = self
                    .state
                    .world
                    .resource_mut::<UiRuntimeSlotsResource>()
                    .context("selected UI runtime vanished from native host")?
                    .submit_native_keyboard(slot, event);
                match submission {
                    Ok(submission) => {
                        // Runtime queue admission is distinct from M13 settlement.
                        // A queued original Down must be acknowledged BEFORE
                        // pumping so a later failure cannot mislabel it rejected.
                        let acknowledgment = match down_admission {
                            Some(token) => self.resolve_native_ui_down_admission(
                                window,
                                token,
                                KeyboardDownDecision::Queued,
                            ),
                            None => Ok(()),
                        };
                        match acknowledgment {
                            Err(error) => {
                                m13_failure = Some(error);
                                false
                            }
                            Ok(()) => {
                                let slots = self
                                    .state
                                    .world
                                    .resource_mut::<UiRuntimeSlotsResource>()
                                    .context(
                                        "selected UI runtime vanished during original input pump",
                                    )?;
                                match slots.pump_native_input_records(slot) {
                                    Ok((batch, status)) => {
                                        let evidence = native_keyboard_finality(
                                            before.scope(),
                                            device,
                                            submission,
                                            &batch,
                                            &status,
                                        );
                                        match &evidence {
                                            Ok(UiInputFinality::Committed(facts)) => {
                                                committed_keyboard_text = text_candidate;
                                                matches!(
                                                    facts.conflict(),
                                                    UiInputConflict::Unclaimed
                                                        | UiInputConflict::ObservedNonexclusive
                                                )
                                            }
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
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let after = self
                            .state
                            .world
                            .resource_mut::<UiRuntimeSlotsResource>()
                            .context(
                                "selected UI runtime vanished before rejected-input observation",
                            )?
                            .input_ownership(slot);
                        match after {
                            Ok(after)
                                if !is_synthetic
                                    && !original.repeat
                                    && matches!(
                                        original.state,
                                        winit::event::ElementState::Pressed
                                    )
                                    && native_no_focused_keyboard_claim_witness(
                                        device, &before, &after, &error,
                                    ) =>
                            {
                                // Only this exact definitive pre-queue rejection
                                // can leave a physical tombstone without a
                                // fabricated M13 Cancel on future source loss.
                                let acknowledged = down_admission
                                    .context("original no-focus Down lacked admission identity")
                                    .and_then(|token| {
                                        self.resolve_native_ui_down_admission(
                                            window,
                                            token,
                                            KeyboardDownDecision::RejectedBeforeQueue,
                                        )
                                    });
                                match acknowledged {
                                    Ok(()) => true,
                                    Err(reason) => {
                                        m13_failure = Some(reason);
                                        false
                                    }
                                }
                            }
                            Ok(after)
                                if !is_synthetic
                                    && original.repeat
                                    && matches!(
                                        original.state,
                                        winit::event::ElementState::Pressed
                                    )
                                    && native_original_keyboard_repeat_nonclaim(
                                        device, &before, &after, &error,
                                    ) =>
                            {
                                // A legitimate original repeat can retain only
                                // independently established prior ActionState
                                // eligibility. A repeat never grants a Down.
                                preserve_prior_held_permission = true;
                                false
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
                                // A physical Up is neutral truth, not a new
                                // game press or a synthetic M13 settlement.
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
                        }
                    }
                }
            }
            KeyboardInputOutcome::Suppressed(diagnostic)
                if !is_synthetic
                    && matches!(original.state, winit::event::ElementState::Pressed)
                    && matches!(
                        (diagnostic, original.repeat),
                        (KeyboardIngressDiagnostic::DuplicatePress, false)
                            | (KeyboardIngressDiagnostic::RepeatOfRejectedDown, true)
                    )
                    && native_unfocused_game_keyboard_snapshot(&before) =>
            {
                // Same pinned translator, matching original device and binding
                // lifetime; a duplicate physical Down has no new M13 sequence.
                // The immutable pre-admission generation prevents cross-press
                // resurrection after an original Up followed by a new Down.
                preserve_prior_held_permission = match (
                    self.state.world.resource::<InputState>(),
                    self.state.world.resource::<ActionState>(),
                ) {
                    (Ok(input), Ok(actions)) => actions
                        .original_native_held_generation_remains_eligible(
                            input,
                            context,
                            &physical_key,
                            previous_down_generation,
                        ),
                    _ => false,
                };
                false
            }
            KeyboardInputOutcome::Suppressed(_) => false,
            KeyboardInputOutcome::Failed(failure) => {
                m13_failure = Some(anyhow!(
                    "native keyboard translator failed closed before M13 admission: {failure:?}"
                ));
                false
            }
        };
        if preserve_prior_held_permission {
            // A suppressed repeat (even with an exact framework nonclaim)
            // can preserve only previously eligible, source-qualified physical
            // Down. Reuse the same one ActionState predicate as duplicate Down.
            preserve_prior_held_permission = match (
                self.state.world.resource::<InputState>(),
                self.state.world.resource::<ActionState>(),
            ) {
                (Ok(input), Ok(actions)) => actions
                    .original_native_held_generation_remains_eligible(
                        input,
                        context,
                        &physical_key,
                        previous_down_generation,
                    ),
                _ => false,
            };
        }

        if m13_failure.is_none()
            && let Some(text) = committed_keyboard_text
        {
            // M13 keyboard callbacks can change the focused recipient before
            // this original key's committed text is enqueued. Never retarget
            // that text to a successor, including A-B-A within one pump.
            let latest = self
                .state
                .world
                .resource_mut::<UiRuntimeSlotsResource>()
                .context("original keyboard text lost its canonical UI slot")?
                .input_ownership(slot);
            match latest {
                Ok(after) if native_keyboard_text_owner_unchanged(&before, &after) => {
                    if let Err(error) = self.route_ui_committed_text(slot, text, Some(device)) {
                        m13_failure = Some(error);
                    }
                }
                Ok(_) => {
                    tracing::trace!(
                        target: "engine.runtime.input",
                        ?window,
                        "original keyboard text skipped after canonical focus or composition transition"
                    );
                }
                Err(error) => {
                    m13_failure = Some(anyhow!(
                        "original keyboard text could not verify M13 focus lifetime: {error:?}"
                    ));
                }
            }
        }
        // Missing or ambiguous M13 evidence, runtime failure, UI exclusivity,
        // and translator suppression NEVER produce positive gameplay evidence.
        // Revocation happens before PreUpdate/CoreSet::Input.
        if (!permit || m13_failure.is_some())
            && !preserve_prior_held_permission
            && matches!(original.state, winit::event::ElementState::Pressed)
        {
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
        self.route_ui_committed_text(slot, text, None)
    }

    /// One existing M13 text owner handles either original OS IME Commit
    /// (device-free) or native KeyEvent.text (exact keyboard device). Never
    /// infer the latter from the last key while handling an IME event.
    fn route_ui_committed_text(
        &mut self,
        slot: UiRuntimeSlotId,
        text: String,
        device: Option<UiInputDeviceId>,
    ) -> Result<()> {
        let event = CommittedTextEvent::new(text, device)
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
        let evidence = native_text_finality(before.scope(), device, submission, &batch, &status);
        anyhow::ensure!(
            matches!(evidence, Ok(UiInputFinality::Committed(_))),
            "native IME text lacked exact committed M13 settlement: {evidence:?}"
        );
        Ok(())
    }

    /// Withdraw precisely one native source's game eligibility when an actual
    /// M13 ownership revision gap cannot exclude an intermediate UI claim.
    fn revoke_native_game_keyboard_source(&mut self, source: InputSourceId) -> Result<()> {
        let world = &mut self.state.world;
        if let Some(mut actions) = world.remove_resource::<ActionState>() {
            let result = world.resource::<InputState>().map(|input| {
                actions.revoke_source_keyboard_for_ui_ownership(input, source);
            });
            world.insert_resource(actions);
            result?;
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
            self.native_input_recipient_for_cancellation(window),
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
        let cancelled = lifetime
            .keyboard
            .cancel_all(KeyModifiers::NONE, native_keyboard_composition(&before))
            .map_err(|error| {
                anyhow!(
                    "native keyboard cancellation has unresolved original Down admission: {error:?}"
                )
            })?;
        for event in cancelled {
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
    #[cfg(test)]
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
            self.native_input_recipient_for_cancellation(window),
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
            let events = lifetime
                .keyboard
                .cancel_device(device, modifiers, native_keyboard_composition(&before))
                .map_err(|error| anyhow!(
                    "native keyboard device-loss cancellation has unresolved original Down: {error:?}"
                ))?;
            lifetime.devices.remove(&device);
            events
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

    fn apply_raw_device_removal(&mut self, device_id: winit::event::DeviceId) -> Result<bool> {
        // A missing neutral reducer must not consume a raw device lifetime
        // before its original source-loss event can be applied.
        self.state
            .world
            .resource::<InputState>()
            .context("native raw-device removal lost its neutral input owner")?;
        // Winit raw DeviceEvent and window-local keyboard/touch/pointer
        // DeviceIds are not guaranteed to name the same OS device. In
        // particular a colliding raw Removed is NOT proof of a window event
        // source's loss. Retire only the raw observation's own generation.
        // Verified window focus/recipient/source retirement separately owns
        // its window keyboard, touch, mouse, M13 and ActionState cleanup.
        let Some(context) = self.native_input_mut().retire_raw_device(device_id)? else {
            return Ok(false);
        };
        self.state
            .world
            .resource_mut::<InputState>()
            .context("native raw-device removal lost its neutral input owner")?
            .handle_continuity_loss(context, ContinuityLoss::Source);
        self.project_actions_after_native_continuity()?;
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
            self.native_input_recipient_for_cancellation(window),
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
            self.native_input_recipient_for_cancellation(window),
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
        // All rejected transitions leave the pinned translator unchanged.
        // They cannot justify cancelling foreign active contacts in this window.
        let mut transition_rejected = false;
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
            let event = lifetime
                .transition(&original, device, point, displayed.input_context.clone())
                .map_err(|error| {
                    transition_rejected = true;
                    anyhow!("native contact translation rejected: {error:?}")
                })?;
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
            // Duplicate/orphan/invalid transitions did not touch the canonical
            // contact table; preserve B instead of globally cancelling it.
            // Genuine failures elsewhere still retire the whole stream.
            if !transition_rejected {
                self.cancel_native_ui_touch(window)?;
            }
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
            self.apply_admitted_event(native_window_id, &entry.event, recipient)?;
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
        recipient: Option<NativeInputRecipient>,
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
                    apply_native_recipient_input_event(input, event, recipient);
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

    /// The raw device event has no window, focus or capture association.
    /// Consult only the existing recipient declarations, rather than querying
    /// live M13 ownership on each high-frequency device-motion callback.
    /// All active native windows must be legacy eligible; unknown UI routing
    /// is never a fallback to a global camera/gameplay delta.
    fn raw_motion_has_legacy_route(&self) -> bool {
        let world = &self.state.world;
        let Ok(windows) = world.resource::<WindowStateRegistryResource>() else {
            return false;
        };
        let has_created_window = || {
            windows
                .records()
                .any(|record| record.lifecycle_state == NativeWindowLifecycleState::Created)
        };
        match (
            world.resource::<UiRuntimeSlotsResource>(),
            world.resource::<UiNativeInputRecipientsResource>(),
        ) {
            (Ok(slots), Ok(recipients)) => {
                recipients.raw_motion_legacy_projection_allowed(windows, slots.is_empty())
            }
            (Ok(slots), Err(_)) => slots.is_empty() && has_created_window(),
            (Err(_), Err(_)) => has_created_window(),
            (Err(_), Ok(_)) => false,
        }
    }

    fn apply_raw_mouse_motion(&mut self, context: InputContext, dx: f32, dy: f32) -> Result<()> {
        let project_legacy = self.raw_motion_has_legacy_route();
        let input = self
            .state
            .world
            .resource_mut::<InputState>()
            .context("missing InputState resource")?;
        input.handle_unattributed_relative_motion(context, dx, dy, project_legacy);
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

    /// Verify the original native source before the next PreUpdate. This
    /// cannot replace the synchronous same-Update mutation transaction.
    fn reconcile_native_ui_keyboard_before_frame(&mut self) -> Result<()> {
        if self.state.world.resource::<ActionState>().is_err() {
            return Ok(());
        }
        // The original Winit source allocator may outlive a window record.
        // Use its source identities to ensure orphaned physical holds are not
        // silently skipped at the frame's input permission boundary.
        let sources = self.native_input().window_source_ids().collect::<Vec<_>>();
        for (window, source) in sources {
            let created = self
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .ok()
                .and_then(|windows| windows.record(window))
                .is_some_and(|record| {
                    record.lifecycle_state == NativeWindowLifecycleState::Created
                });
            let incompatible = if !created {
                // Native close/retirement may precede physical-source cleanup.
                // An old held game action cannot be authorized in that gap.
                true
            } else {
                match self.native_input_recipient(window) {
                    Ok((NativeInputRecipient::LegacyNativeConsumer, _)) => false,
                    Ok((NativeInputRecipient::RunenUiSlot(slot), epoch)) => {
                        let observed = self
                            .state
                            .world
                            .resource_mut::<UiRuntimeSlotsResource>()?
                            .input_ownership(slot);
                        match observed {
                            Ok(snapshot) => {
                                let previous =
                                    self.native_ui_keyboards.get(&window).and_then(|lifetime| {
                                        (lifetime.recipient == slot
                                            && lifetime.source == source
                                            && Some(lifetime.binding_epoch) == epoch)
                                            .then_some(lifetime.last_observed_revision)
                                            .flatten()
                                    });
                                // No matching pinned translator is also a continuity
                                // gap: a game-held key can predate this UI binding.
                                // Only a genuinely correlated native UI keyboard
                                // occurrence can establish the initial witness.
                                let gap = previous
                                    .is_none_or(|revision| revision != snapshot.revision().get());
                                if let Some(lifetime) = self.native_ui_keyboards.get_mut(&window)
                                    && lifetime.recipient == slot
                                    && lifetime.source == source
                                    && Some(lifetime.binding_epoch) == epoch
                                {
                                    lifetime.last_observed_revision =
                                        Some(snapshot.revision().get());
                                }
                                gap || native_ui_blocks_preframe_keyboard_gameplay(&snapshot)
                            }
                            Err(_) => true,
                        }
                    }
                    Err(_) => true,
                }
            };
            if incompatible {
                let world = &mut self.state.world;
                let mut actions = world
                    .remove_resource::<ActionState>()
                    .context("gameplay actions vanished during native M13 reconciliation")?;
                let update = world
                    .resource::<InputState>()
                    .map(|input| {
                        actions.revoke_source_keyboard_for_ui_ownership(input, source);
                    })
                    .context("native M13 reconciliation missing physical InputState");
                world.insert_resource(actions);
                update?;
            }
        }
        Ok(())
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
        self.reconcile_native_ui_keyboard_before_frame()?;
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
                    Err(error) if native_ime_waiting_for_m13_settlement(&error) => false,
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
                self.apply_native_focus_change(native_window_id, focused)
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
                    .native_input_mut()
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
                        let epoch =
                            epoch.context("selected native UI slot has no binding epoch")?;
                        let previous_down_generation = self
                            .state
                            .world
                            .resource::<InputState>()
                            .context("native UI keyboard lacks original physical input owner")?
                            .keyboard_down_generation_in(context, &physical_key);
                        // The exact same admission order is exercised by
                        // the source-level regression; M13 finality follows.
                        self.admit_native_ui_keyboard_for_recipient(
                            native_window_id,
                            slot,
                            epoch,
                            context,
                            input,
                            text_input(&event, is_synthetic),
                        )?;
                        self.route_ui_keyboard(
                            native_window_id,
                            slot,
                            epoch,
                            context,
                            NativeUiKeyboardOriginal {
                                event: &event,
                                is_synthetic,
                                previous_down_generation,
                            },
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
                if !native_ime_payload_allowed_for_window(
                    window.has_focus(),
                    self.native_ime_applied.contains(&native_window_id),
                ) {
                    return Ok(());
                }
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
                if !native_ime_payload_allowed_for_window(
                    window.has_focus(),
                    self.native_ime_applied.contains(&native_window_id),
                ) {
                    return Ok(());
                }
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
                    .native_input_mut()
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
                    .native_input_mut()
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
                    .native_input_mut()
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
                    .native_input_mut()
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
                let context = self.native_input_mut().raw_device_context(device_id)?;
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

    #[test]
    fn native_ime_payload_needs_focused_window_and_applied_os_permission() {
        assert!(native_ime_payload_allowed_for_window(true, true));
        for (focused, applied) in [(false, true), (true, false), (false, false)] {
            assert!(
                !native_ime_payload_allowed_for_window(focused, applied),
                "stale IME payload must not enter RunenUI after native lifetime loss"
            );
        }
    }

    #[test]
    fn native_ime_projection_denies_only_transient_m13_work() {
        use crate::plugins::ui::UiRuntimeSlotOperationFailure;

        assert!(native_ime_waiting_for_m13_settlement(&anyhow!(
            NativeInputRecipientError::OwnershipUnavailable(
                UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce
            )
        )));
        for error in [
            NativeInputRecipientError::StaleSlot,
            NativeInputRecipientError::RuntimeNotRunning,
            NativeInputRecipientError::MissingRecipient,
        ] {
            assert!(
                !native_ime_waiting_for_m13_settlement(&anyhow!(error)),
                "terminal, stale or absent recipients must not be mistaken for normal pending work"
            );
        }
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
        assert!(
            !native_ui_blocks_preframe_keyboard_gameplay(&initially_unfocused),
            "unfocused RunenUI is not a blanket game-keyboard veto"
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
            native_keyboard_text_owner_unchanged(&before, &before),
            "unchanged focused typed text ownership permits original KeyEvent.text"
        );
        assert!(
            native_ime_allowed_for_m13(true, &before),
            "a text-capable focused M13 owner enables its selected native window's OS IME"
        );
        assert!(
            native_ui_blocks_preframe_keyboard_gameplay(&before),
            "real M13 text focus must revoke old source-local gameplay holds"
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
        assert!(
            !native_keyboard_text_owner_unchanged(&before, &composing),
            "an intervening real M13 composition transition invalidates prior keyboard text"
        );
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
        assert!(
            !native_keyboard_text_owner_unchanged(&before, &after),
            "A-to-composition-to-A cannot reuse the earlier original keyboard text lifetime"
        );
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

    #[test]
    fn unattributed_raw_motion_preserves_physical_truth_without_ui_gameplay_fallback() {
        use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};
        use runen_input::{InputDeviceId, InputObservation};

        struct RawUi;
        impl runenui_core::UiApp for RawUi {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("raw input owner")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let raw = InputContext::new(InputSourceId::new(729), Some(InputDeviceId::new(41)));

        // With only a genuine game/legacy host, retain its existing raw camera
        // projection. Physical input remains independently observable.
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .start_admitted_input_capture();
        runner.apply_raw_mouse_motion(raw, 3.0, -2.0).unwrap();
        assert_eq!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .mouse_delta,
            (3.0, -2.0)
        );

        // Mounting UI with no selected input recipient is NOT game permission.
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<RawUi>(
                UiTypedScreenId::new("raw.motion.owner"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("real M13 slot");
        runner.state.world.insert_resource(slots);
        runner
            .state
            .world
            .insert_resource(UiNativeInputRecipientsResource::default());
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .clear_frame();
        runner.apply_raw_mouse_motion(raw, 5.0, 7.0).unwrap();
        assert_eq!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .mouse_delta,
            (0.0, 0.0),
            "an unresolved UI recipient must not move the gameplay camera"
        );

        // An explicitly mounted and bound UI window is equally not a window
        // attribution witness for raw DeviceEvent::MouseMotion.
        let secondary = NativeWindowId::try_from_raw(2).expect("distinct native window");
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .register_created_window(secondary, "legacy second window", (800, 600), 1.0, false);
        let mut slots = runner
            .state
            .world
            .remove_resource::<UiRuntimeSlotsResource>()
            .unwrap();
        let mut recipients = runner
            .state
            .world
            .remove_resource::<UiNativeInputRecipientsResource>()
            .unwrap();
        recipients
            .bind(
                secondary,
                NativeInputRecipient::LegacyNativeConsumer,
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .expect("independent secondary legacy recipient");
        recipients
            .bind(
                NativeWindowId::primary(),
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .expect("one explicit active UI recipient");
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        runner.apply_raw_mouse_motion(raw, 11.0, 13.0).unwrap();
        assert_eq!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .mouse_delta,
            (0.0, 0.0)
        );

        let captured = runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .stop_admitted_input_capture();
        assert_eq!(
            captured.len(),
            3,
            "all genuine raw observations are neutral"
        );
        assert!(captured.iter().all(|group| group.context == raw
            && matches!(
                group.observations.as_slice(),
                [InputObservation::RelativeMotion { .. }]
            )));
    }

    #[test]
    fn closing_native_window_retracts_previous_game_hold_at_frame_entry() {
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin,
        };

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let window = NativeWindowId::primary();
        let context = runner
            .native_input_mut()
            .window_context(window, winit::event::DeviceId::dummy())
            .unwrap();
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(
                context,
                &KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyW"),
                    logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
            );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP)
        );

        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .record_mut(window)
            .unwrap()
            .receive_close_intent();
        runner.reconcile_native_ui_keyboard_before_frame().unwrap();
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "closing native recipient cannot leave old gameplay input held"
        );
    }

    #[test]
    fn native_original_source_identity_is_world_owned_not_a_parallel_runner_map() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let window = NativeWindowId::primary();
        let device = winit::event::DeviceId::dummy();
        let identity = runner
            .native_input_mut()
            .window_context(window, device)
            .expect("original native identity must be allocatable");
        let world_identity = runner
            .state
            .world
            .resource::<WinitInputAdapter>()
            .expect("the original input allocator is a World resource")
            .window_source_context(window)
            .expect("same original source must remain registered");
        assert_eq!(identity.source, world_identity.source);
        let retired = runner
            .state
            .world
            .resource_mut::<WinitInputAdapter>()
            .unwrap()
            .retire_window_source(window)
            .expect("one original source authority owns retirement");
        assert_eq!(retired.source, identity.source);
        assert!(
            runner
                .native_input()
                .window_source_context(window)
                .is_none(),
            "the runner must observe World retirement without copying a map"
        );
        let next = runner
            .native_input_mut()
            .window_context(window, device)
            .unwrap();
        assert_ne!(
            next.source, identity.source,
            "retired identity cannot be reused"
        );
    }

    #[test]
    fn orphaned_native_window_source_is_denied_without_affecting_other_window() {
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin,
        };

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let primary = NativeWindowId::primary();
        let secondary = NativeWindowId::try_from_raw(2).unwrap();
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .register_created_window(secondary, "secondary", (640, 480), 1.0, false);
        let first = runner
            .native_input_mut()
            .window_context(primary, winit::event::DeviceId::dummy())
            .unwrap();
        let other = runner
            .native_input_mut()
            .window_context(secondary, winit::event::DeviceId::dummy())
            .unwrap();
        assert_ne!(first.source, other.source);

        let down = KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyW"),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: DigitalState::Pressed,
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        };
        {
            let input = runner.state.world.resource_mut::<InputState>().unwrap();
            input.handle_normalized_keyboard(first, &down);
            input.handle_normalized_keyboard(other, &down);
        }
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP)
        );

        // Simulate a lost window registry record while the Winit source
        // allocator and RunenInput still retain their original physical key.
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .remove_window(secondary)
            .expect("original secondary was created");
        assert!(
            runner
                .native_input()
                .window_source_context(secondary)
                .is_some(),
            "source retirement has not yet happened"
        );
        runner.reconcile_native_ui_keyboard_before_frame().unwrap();
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "the original primary game window must stay eligible"
        );

        // The orphan's real W Down remains in neutral physical truth but may
        // no longer keep the aggregate gameplay action held after primary Up.
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(
                first,
                &KeyboardInput {
                    state: DigitalState::Released,
                    ..down
                },
            );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(other, &PhysicalKeyIdentity::code("KeyW"))
                .is_some()
        );
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "orphaned W must not resurrect source-erased gameplay permission"
        );
    }

    #[test]
    fn nonexclusive_m13_revision_gap_revokes_only_its_original_game_source() {
        use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin,
        };

        struct RevisionProbe;
        impl runenui_core::UiApp for RevisionProbe {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::button("revision focus probe").on_activate(|| ())
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let primary = NativeWindowId::primary();
        let legacy = NativeWindowId::try_from_raw(2).unwrap();
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .register_created_window(legacy, "legacy", (640, 480), 1.0, false);
        let first = runner
            .native_input_mut()
            .window_context(primary, winit::event::DeviceId::dummy())
            .unwrap();
        let other = runner
            .native_input_mut()
            .window_context(legacy, winit::event::DeviceId::dummy())
            .unwrap();

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<RevisionProbe>(
                UiTypedScreenId::new("native.revision.gap"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("genuine mounted M13 runtime");
        let current = slots.input_ownership(slot).expect("current M13 snapshot");
        assert!(
            !native_ui_blocks_preframe_keyboard_gameplay(&current),
            "genuine unfocused nonexclusive UI may not veto gameplay by focus alone"
        );
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                primary,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .unwrap();
        recipients
            .bind(
                legacy,
                NativeInputRecipient::LegacyNativeConsumer,
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .unwrap();
        let epoch = recipients.binding_epoch_for_window(primary).unwrap();
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);

        let down = KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyW"),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: DigitalState::Pressed,
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        };
        let input = runner.state.world.resource_mut::<InputState>().unwrap();
        input.handle_normalized_keyboard(first, &down);
        input.handle_normalized_keyboard(other, &down);
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);

        // The host missed an intermediate meaningful ownership revision.
        // M13 is now truly nonexclusive, but that final state alone cannot
        // certify that the old physical W never belonged to UI in between.
        runner.native_ui_keyboards.insert(
            primary,
            NativeUiKeyboardTranslator {
                recipient: slot,
                source: first.source,
                devices: HashSet::new(),
                binding_epoch: epoch,
                last_observed_revision: Some(u64::MAX),
                keyboard: KeyboardInputState::default(),
            },
        );
        runner.reconcile_native_ui_keyboard_before_frame().unwrap();
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "independent real legacy source continues to drive the same action"
        );
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(
                other,
                &KeyboardInput {
                    state: DigitalState::Released,
                    ..down
                },
            );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "missed UI revision may not leave a ghost held gameplay action"
        );
        assert_eq!(
            runner
                .native_ui_keyboards
                .get(&primary)
                .unwrap()
                .last_observed_revision,
            Some(current.revision().get()),
            "witness reconciles to the actual framework revision"
        );

        // A real RunenUI semantic focus mutation now advances M13 ownership
        // after the native translator's last witness. The already-mounted
        // button is not a text/composition/modal blocker: focus by itself
        // must NOT grant or withdraw an independently settled game key.
        // The test-only selected permit models the framework mutation without
        // an Engine World transaction so the native next-callback gap is real.
        let focused = {
            let slots = runner
                .state
                .world
                .resource_mut::<UiRuntimeSlotsResource>()
                .unwrap();
            crate::plugins::ui::native_transaction::test_selected_slot_operation(
                slots,
                slot,
                |slots| {
                    slots.test_request_first_typed_focus::<RevisionProbe>(slot)?;
                    slots.settle_slot(slot)
                },
            )
            .expect("actual RunenUI semantic focus action settles");
            slots
                .input_ownership(slot)
                .expect("fresh framework M13 focus")
        };
        assert_ne!(
            focused.revision(),
            current.revision(),
            "real semantic focus, not a manufactured revision number, creates the gap"
        );
        assert!(
            focused.keyboard().focused_node().is_some()
                && !native_ui_blocks_preframe_keyboard_gameplay(&focused),
            "non-text native focus alone does not claim every game key"
        );

        // The new authentic KeyA is admitted against the already-reconciled
        // previous M13 revision, not swept up in its old source denial.
        runner
            .admit_native_ui_keyboard_for_recipient(
                primary,
                slot,
                epoch,
                first,
                KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyA"),
                    logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
                None,
            )
            .unwrap();
        // Settle the independent new KeyA against the pinned framework's
        // actual ordered M13 keyboard record. This doesn't invent a second
        // input owner or claim a physical Winit KeyEvent on the CI worker.
        let device = WinitRunner::native_ui_device(first).unwrap();
        let (submission, batch, status) = {
            let slots = runner
                .state
                .world
                .resource_mut::<UiRuntimeSlotsResource>()
                .unwrap();
            let before = slots.input_ownership(slot).unwrap();
            assert_eq!(before.revision(), focused.revision());
            let submission = slots
                .submit_native_keyboard(
                    slot,
                    runenui_core::KeyboardEvent::new(
                        runenui_core::KeyboardPhase::Down,
                        runenui_core::PhysicalKey::Code("KeyA".into()),
                        runenui_core::LogicalKey::Character("a".into()),
                        KeyModifiers::NONE,
                        false,
                        runenui_core::KeyLocation::Standard,
                        native_keyboard_composition(&before),
                        Some(device),
                    ),
                )
                .expect("real M13 keyboard admission for new KeyA");
            let (batch, status) = slots
                .pump_native_input_records(slot)
                .expect("real M13 keyboard settlement");
            (submission, batch, status)
        };
        let settlement =
            native_keyboard_finality(focused.scope(), device, submission, &batch, &status);
        assert!(
            matches!(
                settlement,
                Ok(UiInputFinality::Committed(ref facts))
                    if matches!(
                        facts.conflict(),
                        UiInputConflict::Unclaimed | UiInputConflict::ObservedNonexclusive
                    )
            ),
            "fresh physical A must have independently nonexclusive M13 finality: {settlement:?}"
        );
        // Executable counterexample for the superseded order: invoking
        // source-wide revocation AFTER the fresh physical admission erases
        // KeyA despite independently committed nonexclusive M13 finality.
        let mut old_order = ActionState::new();
        let physical = runner.state.world.resource::<InputState>().unwrap();
        old_order.project(physical);
        assert!(
            old_order.action_down(crate::plugins::action::WORLD_MOVE_LEFT),
            "the original physical A Down has a genuine game action before gap revocation"
        );
        old_order.revoke_source_keyboard_for_ui_ownership(physical, first.source);
        assert!(
            !old_order.action_down(crate::plugins::action::WORLD_MOVE_LEFT),
            "post-ingress gap reconciliation demonstrably kills the fresh A lifetime"
        );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        let actions = runner.state.world.resource::<ActionState>().unwrap();
        assert!(
            actions.action_down(crate::plugins::action::WORLD_MOVE_LEFT),
            "an older M13 revision gap cannot revoke a later authentic, source-local A Down"
        );
        assert!(
            !actions.action_down(crate::plugins::action::WORLD_MOVE_UP),
            "the older W generation stays revoked after the fresh A Down"
        );
        assert_eq!(
            runner
                .native_ui_keyboards
                .get(&primary)
                .unwrap()
                .last_observed_revision,
            Some(focused.revision().get()),
            "the actual ownership snapshot remains the only revision witness"
        );

        // If the selected framework owner disappears, all previous source
        // permission is withdrawn BEFORE the next original Down, which is
        // nevertheless kept in neutral physical truth and denied in ActionState.
        runner
            .state
            .world
            .remove_resource::<UiRuntimeSlotsResource>();
        assert!(
            runner
                .admit_native_ui_keyboard_for_recipient(
                    primary,
                    slot,
                    epoch,
                    first,
                    KeyboardInput {
                        physical_key: PhysicalKeyIdentity::code("KeyD"),
                        logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                        location: KeyLocation::Standard,
                        state: DigitalState::Pressed,
                        repeat: false,
                        origin: ObservationOrigin::SourceReport,
                    },
                    None,
                )
                .is_err(),
            "an absent M13 ownership snapshot is never a game nonclaim"
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(first, &PhysicalKeyIdentity::code("KeyD"))
                .is_some(),
            "failed UI preflight cannot erase original physical Down"
        );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        assert!(
            !actions.action_down(crate::plugins::action::WORLD_MOVE_LEFT)
                && !actions.action_down(crate::plugins::action::WORLD_MOVE_RIGHT),
            "unknown M13 state must revoke old A and deny the newly admitted D"
        );
        runner.state.world.insert_resource(actions);
    }

    #[test]
    fn unobserved_ui_binding_cannot_inherit_previously_held_game_key() {
        use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin,
        };

        struct UnobservedRecipient;
        impl runenui_core::UiApp for UnobservedRecipient {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("unfocused recipient")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let ui_window = NativeWindowId::primary();
        let game_window = NativeWindowId::try_from_raw(2).unwrap();
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .register_created_window(game_window, "independent game", (640, 480), 1.0, false);
        let first = runner
            .native_input_mut()
            .window_context(ui_window, winit::event::DeviceId::dummy())
            .unwrap();
        let other = runner
            .native_input_mut()
            .window_context(game_window, winit::event::DeviceId::dummy())
            .unwrap();
        let down = KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyW"),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: DigitalState::Pressed,
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        };
        {
            let input = runner.state.world.resource_mut::<InputState>().unwrap();
            input.handle_normalized_keyboard(first, &down);
            input.handle_normalized_keyboard(other, &down);
        }
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);

        // Application-side UI selection happened after the game's original W,
        // with no genuine native keyboard translation in the new UI scope.
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<UnobservedRecipient>(
                UiTypedScreenId::new("native.unobserved.binding"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let before = slots.input_ownership(slot).unwrap();
        assert!(!native_ui_blocks_preframe_keyboard_gameplay(&before));
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                ui_window,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .unwrap();
        recipients
            .bind(
                game_window,
                NativeInputRecipient::LegacyNativeConsumer,
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .unwrap();
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        assert!(
            !runner.native_ui_keyboards.contains_key(&ui_window),
            "an unrelated UI mount must not invent a pinned keyboard translator"
        );

        runner.reconcile_native_ui_keyboard_before_frame().unwrap();
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "independent legacy game source is still eligible"
        );
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(
                other,
                &KeyboardInput {
                    state: DigitalState::Released,
                    ..down
                },
            );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "unobserved selected UI binding cannot inherit the old W hold"
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(first, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "the UI source's original physical truth remains intact"
        );
    }

    #[test]
    fn retiring_legacy_recipient_retracts_old_game_hold_before_ui_rebind() {
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin,
        };

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let window = NativeWindowId::primary();
        let mut slots = UiRuntimeSlotsResource::default();
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                window,
                NativeInputRecipient::LegacyNativeConsumer,
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .expect("real legacy host recipient");
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        runner
            .state
            .world
            .insert_resource(UiRuntimePresentationAssociationsResource::default());

        let source = runner
            .native_input_mut()
            .window_context(window, winit::event::DeviceId::dummy())
            .unwrap();
        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(
                source,
                &KeyboardInput {
                    physical_key: PhysicalKeyIdentity::code("KeyW"),
                    logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
                    location: KeyLocation::Standard,
                    state: DigitalState::Pressed,
                    repeat: false,
                    origin: ObservationOrigin::SourceReport,
                },
            );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP)
        );

        assert!(
            runner
                .retire_native_input_recipient(window)
                .expect("recipient retirement must preserve a real source")
        );
        assert!(
            runner
                .native_input()
                .window_source_context(window)
                .is_some(),
            "recipient retirement must not counterfeit physical device loss"
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(source, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "canonical physical W remains genuinely held"
        );
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "former legacy recipient cannot carry its old W into new UI scope"
        );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "later projection cannot revive a retired original Down generation"
        );
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
        windowed
            .world_mut()
            .insert_resource(WinitInputAdapter::default());
        let mut runner = WinitRunner {
            state: windowed.into_windowed_state(),
            window_realizer: WinitWindowRealizer::default(),
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
            .native_input_mut()
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
            .apply_native_focus_change(NativeWindowId::primary(), false)
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
                .native_input()
                .window_source_context(NativeWindowId::primary())
                .map(|value| value.source),
            Some(context.source)
        );
    }

    #[test]
    fn saturated_focus_loss_does_not_partially_cancel_a_selected_ui_keyboard() {
        use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};

        struct FocusProbe;
        impl runenui_core::UiApp for FocusProbe {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("focus-loss journal preflight")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let window = NativeWindowId::primary();
        let context = runner
            .native_input_mut()
            .window_context(window, winit::event::DeviceId::dummy())
            .expect("checked native physical keyboard identity");
        runner
            .apply_event(normalized_shift_press(context))
            .expect("original Down enters physical state and journal");
        let device = WinitRunner::native_ui_device(context).expect("checked framework device");
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<FocusProbe>(
                UiTypedScreenId::new("native.focus.journal.preflight"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .expect("real RunenUI runtime");
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                window,
                NativeInputRecipient::RunenUiSlot(slot),
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .expect("original native window"),
                &mut slots,
            )
            .expect("explicit selected native UI recipient");
        let epoch = recipients
            .binding_epoch_for_window(window)
            .expect("checked native binding epoch");
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);
        runner.native_ui_keyboard_translator(window, slot, context.source, device, epoch);

        // One slot remains, but a focus loss with an original source needs
        // TWO adjacent records (Focused + physical ContinuityLoss).
        let journal = runner
            .state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .expect("bounded native journal");
        for _ in 0..4094 {
            journal
                .publish(PlatformWindowEvent::new(window, PlatformEvent::RedrawRequested))
                .expect("fill journal without overflowing it");
        }
        assert_eq!(journal.events().len(), 4095);
        assert!(
            runner.apply_native_focus_change(window, false).is_err(),
            "insufficient original journal capacity must reject before M13 cancellation"
        );
        assert_eq!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .expect("original journal")
                .events()
                .len(),
            4095,
            "no partial original focus record may be published"
        );
        assert!(
            runner
                .native_ui_keyboards
                .get(&window)
                .is_some_and(|translator| translator.devices.contains(&device)),
            "a rejected original focus group cannot prematurely discard the UI keyboard lifetime"
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("physical reducer")
                .shift_down(),
            "a rejected original group cannot mutate neutral Down truth"
        );
        assert!(
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .expect("window owner")
                .record(window)
                .expect("created window")
                .focused,
            "a rejected original group cannot partially update window focus"
        );
    }

    #[test]
    fn stale_native_recipient_cannot_prevent_original_focus_and_source_retirement() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let primary = NativeWindowId::primary();
        let context = runner
            .native_input_mut()
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
            .native_input_mut()
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
                .native_input()
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
                .native_input()
                .window_source_context(NativeWindowId::primary()),
            None
        );
        runner
            .apply_window_focus_event(NativeWindowId::primary(), false)
            .expect("focus loss should apply");
        assert_eq!(
            runner
                .native_input()
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
            .native_input_mut()
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
        assert_eq!(runner.native_input().window_source_context(secondary), None);
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
            .native_input_mut()
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
        assert_eq!(runner.native_input().window_source_context(secondary), None);

        let replacement = runner
            .native_input_mut()
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
            .native_input_mut()
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
            runner.native_input().window_source_context(secondary),
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
    fn native_startup_selection_retracts_preexisting_source_game_hold() {
        use crate::plugins::ui::{UiRuntimeFontConfiguration, UiTypedScreenId};
        use runen_input::{
            DigitalState, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
            ObservationOrigin, PhysicalKeyIdentity,
        };

        struct StartupUi;
        impl runenui_core::UiApp for StartupUi {
            type State = ();
            type Action = ();
            type HostProtocol = runenui_core::NoHostProtocol;

            fn root(_: &Self::State) -> impl runenui_core::View<Self::Action> {
                runenui_core::text("native startup permission")
            }

            fn update(
                _: &mut Self::State,
                _: Self::Action,
            ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
            }
        }

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let primary = NativeWindowId::primary();
        let secondary = NativeWindowId::try_from_raw(2).unwrap();
        runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .unwrap()
            .register_created_window(secondary, "independent game", (640, 480), 1.0, false);
        let original = runner
            .native_input_mut()
            .window_context(primary, winit::event::DeviceId::dummy())
            .unwrap();
        let other = runner
            .native_input_mut()
            .window_context(secondary, winit::event::DeviceId::dummy())
            .unwrap();
        assert_ne!(original.source, other.source);
        let press = KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyW"),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state: DigitalState::Pressed,
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        };
        let mut input = InputState::new();
        input.handle_normalized_keyboard(original, &press);
        input.handle_normalized_keyboard(other, &press);
        let mut actions = ActionState::new();
        actions.project(&input);
        assert!(actions.action_down(crate::plugins::action::WORLD_MOVE_UP));
        runner.state.world.insert_resource(input);
        runner.state.world.insert_resource(actions);

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<StartupUi>(
                UiTypedScreenId::new("native.startup.selected"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .request_primary(NativeInputRecipient::RunenUiSlot(slot))
            .unwrap();
        recipients
            .bind(
                secondary,
                NativeInputRecipient::LegacyNativeConsumer,
                runner
                    .state
                    .world
                    .resource::<WindowStateRegistryResource>()
                    .unwrap(),
                &mut slots,
            )
            .unwrap();
        runner.state.world.insert_resource(slots);
        runner.state.world.insert_resource(recipients);

        runner
            .realize_declared_primary_input()
            .expect("startup must use the source-correlated World boundary");
        let (recipient, first_epoch) = runner.native_input_recipient(primary).unwrap();
        assert_eq!(recipient, NativeInputRecipient::RunenUiSlot(slot));
        assert!(first_epoch.is_some());
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(original, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "startup selection cannot synthesize a neutral physical Up"
        );
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "independent game source must retain its own W"
        );

        runner
            .state
            .world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(
                other,
                &KeyboardInput {
                    state: DigitalState::Released,
                    ..press
                },
            );
        let mut actions = runner.state.world.remove_resource::<ActionState>().unwrap();
        actions.project(runner.state.world.resource::<InputState>().unwrap());
        runner.state.world.insert_resource(actions);
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP),
            "already held UI-source W cannot resurrect after other source releases"
        );
        runner
            .realize_declared_primary_input()
            .expect("idempotent startup must keep the first recipient epoch");
        assert_eq!(
            runner.native_input_recipient(primary).unwrap().1,
            first_epoch
        );
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .unwrap()
                .action_down(crate::plugins::action::WORLD_MOVE_UP)
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
        // Original window-keyboard text is reserved and admitted in the
        // same callback group as the physical Down. A later text settlement
        // cannot retrospectively add the observation to the neutral journal.
        let original_context = runner
            .native_input_mut()
            .window_context(secondary, winit::event::DeviceId::dummy())
            .expect("actual window-scoped keyboard source");
        let original_key = runen_input::KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyE"),
            logical_key: runen_input::LogicalKey::Character(String::from("é")),
            location: runen_input::KeyLocation::Standard,
            state: runen_input::DigitalState::Pressed,
            repeat: false,
            origin: runen_input::ObservationOrigin::SourceReport,
        };
        let prior_len = runner
            .state
            .world
            .resource::<PlatformWindowEventQueueResource>()
            .expect("original journal")
            .events()
            .len();
        runner
            .admit_native_ui_keyboard_for_recipient(
                secondary,
                slot,
                bound_epoch,
                original_context,
                original_key,
                Some(String::from("é")),
            )
            .expect("original UI keyboard and text group admitted atomically");
        let journal = runner
            .state
            .world
            .resource::<PlatformWindowEventQueueResource>()
            .expect("original journal");
        assert_eq!(journal.events().len(), prior_len + 2);
        let pair = &journal.events()[prior_len..];
        assert!(matches!(
            &pair[0].event,
            PlatformEvent::KeyboardInput { .. }
        ));
        assert!(matches!(&pair[1].event, PlatformEvent::TextInput { text } if text == "é"));
        assert!(
            pair.iter().all(|entry| {
                entry.native_window_id == secondary && !entry.authorized_for_legacy_editor()
            }),
            "original UI keyboard/text may not leak to Editor"
        );

        // Leave room for exactly one record: neither the original Down nor
        // its text can enter a bounded journal until BOTH fit. This failure
        // must leave physical Down generation and journal unchanged.
        let journal = runner
            .state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .expect("bounded original journal");
        journal.retire_frame();
        let empty_text = || {
            PlatformWindowEvent::new(
                secondary,
                PlatformEvent::TextInput {
                    text: String::new(),
                },
            )
        };
        while journal.events().len() < 4095 {
            journal
                .publish(empty_text())
                .expect("test fills journal up to one slot");
        }
        let reject_context = original_context;
        let rejected_key = runen_input::KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyZ"),
            logical_key: runen_input::LogicalKey::Character(String::from("z")),
            location: runen_input::KeyLocation::Standard,
            state: runen_input::DigitalState::Pressed,
            repeat: false,
            origin: runen_input::ObservationOrigin::SourceReport,
        };
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("neutral input")
                .keyboard_down_generation_in(reject_context, &PhysicalKeyIdentity::code("KeyZ"),)
                .is_none()
        );
        assert!(
            runner
                .admit_native_ui_keyboard_for_recipient(
                    secondary,
                    slot,
                    bound_epoch,
                    reject_context,
                    rejected_key,
                    Some(String::from("z")),
                )
                .is_err(),
            "capacity for one must reject the entire original keyboard+text group"
        );
        assert_eq!(
            runner
                .state
                .world
                .resource::<PlatformWindowEventQueueResource>()
                .expect("bounded journal")
                .events()
                .len(),
            4095
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("neutral input")
                .keyboard_down_generation_in(reject_context, &PhysicalKeyIdentity::code("KeyZ"),)
                .is_none(),
            "failed grouped journal preflight cannot partially admit a neutral keyboard Down"
        );
        runner
            .state
            .world
            .resource_mut::<PlatformWindowEventQueueResource>()
            .expect("restore journal capacity")
            .retire_frame();
        // A native button with no prior authentic cursor position cannot
        // manufacture a logical hit point or a new RunenUI pressed stream.
        let unknown_position_context = runner
            .native_input_mut()
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
        // RunenUI's one contact translator owns both devices. A duplicate
        // or orphan native A observation must leave B's real contact intact;
        // no global M13 Cancel may be derived from a rejected transition.
        let mut touch_runtime = runenui_runtime::AppRuntime::<NativeInputApp>::mount(());
        let touch_styles = runenui_core::StyleEnvironment::default();
        let published_touch = touch_runtime
            .publish_surface(&runenui_runtime::SurfaceBuildContext::new(
                &touch_styles,
                runenui_runtime::LayoutConstraints::unbounded(),
            ))
            .expect("real RunenUI contact surface");
        let touch_scope = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("mounted M13 owner")
            .input_ownership(slot)
            .expect("canonical original scope")
            .scope()
            .clone();
        let mut contacts = NativeUiTouchTranslator {
            recipient: slot,
            source: unknown_position_context.source,
            binding_epoch: bound_epoch,
            scope: touch_scope,
            backend_devices: HashSet::new(),
            touch: TouchInputState::default(),
        };
        let backend = winit::event::DeviceId::dummy();
        let observed = |id, phase| Touch {
            device_id: backend,
            phase,
            location: PhysicalPosition::new(18.0, 22.0),
            force: None,
            id,
        };
        let a_device = UiInputDeviceId::new(91).expect("original device A");
        let b_device = UiInputDeviceId::new(92).expect("independent device B");
        let point = LogicalPoint::new(18.0, 22.0).expect("finite native point");
        let a = contacts
            .transition(
                &observed(7, winit::event::TouchPhase::Started),
                a_device,
                point,
                published_touch.input_context().clone(),
            )
            .expect("original A Down");
        let b = contacts
            .transition(
                &observed(7, winit::event::TouchPhase::Started),
                b_device,
                point,
                published_touch.input_context().clone(),
            )
            .expect("original B Down with overlapping native contact ID");
        assert_ne!(a.pointer_id(), b.pointer_id());
        let before_membership = contacts.backend_devices.clone();
        assert_eq!(
            contacts.transition(
                &observed(7, winit::event::TouchPhase::Started),
                a_device,
                point,
                published_touch.input_context().clone(),
            ),
            Err(runenui_winit::touch_input::TouchIngressDiagnostic::DuplicateStart)
        );
        assert_eq!(
            contacts.transition(
                &observed(999, winit::event::TouchPhase::Moved),
                a_device,
                point,
                published_touch.input_context().clone(),
            ),
            Err(runenui_winit::touch_input::TouchIngressDiagnostic::UnknownContact)
        );
        assert_eq!(contacts.backend_devices, before_membership);
        assert_eq!(contacts.touch.active_contact_count(), 2);
        let b_up = contacts
            .transition(
                &observed(7, winit::event::TouchPhase::Ended),
                b_device,
                point,
                published_touch.input_context().clone(),
            )
            .expect("unrelated B Up remains admitted");
        assert_eq!(b_up.pointer_id(), b.pointer_id());
        assert_eq!(contacts.touch.active_contact_count(), 1);
        assert_eq!(contacts.touch.cancel_device(a_device).len(), 1);
        assert_eq!(contacts.touch.active_contact_count(), 0);

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

        // A colliding raw Removed is not evidence of this window keyboard
        // device's disappearance. It must preserve the original M13
        // translator pressed-device membership and modifiers.
        let backend = winit::event::DeviceId::dummy();
        let original = runner
            .native_input_mut()
            .window_context(secondary, backend)
            .expect("window-observed device");
        let ui_device = WinitRunner::native_ui_device(original).expect("original device");
        runner.native_ui_keyboard_translator(
            secondary,
            slot,
            original.source,
            ui_device,
            new_epoch,
        );
        runner
            .native_ui_modifiers
            .insert(secondary, KeyModifiers::SHIFT);
        let raw = runner
            .native_input_mut()
            .raw_device_context(backend)
            .expect("independent raw device");
        assert!(
            runner
                .apply_raw_device_removal(backend)
                .expect("raw removal")
        );
        let next_raw = runner
            .native_input_mut()
            .raw_device_context(backend)
            .expect("raw ID reincarnation");
        assert_ne!(next_raw.source, raw.source);
        assert_ne!(next_raw.device, raw.device);
        assert_eq!(
            runner.native_input_mut().window_context(secondary, backend),
            Ok(original),
            "raw removal cannot reincarnate a live window keyboard device"
        );
        assert!(
            runner
                .native_ui_keyboards
                .get(&secondary)
                .is_some_and(|lifetime| lifetime.devices.contains(&ui_device)),
            "raw-device name collision cannot synthesize a UI Cancel"
        );
        assert_eq!(
            runner.native_ui_modifiers.get(&secondary),
            Some(&KeyModifiers::SHIFT),
            "raw Removed carries no original window modifier authority"
        );

        let scope = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("mounted RunenUI")
            .input_ownership(slot)
            .expect("live M13 scope")
            .scope()
            .clone();
        runner.native_ui_mice.insert(
            secondary,
            NativeUiMouseTranslator {
                recipient: slot,
                source: original.source,
                backend_device: backend,
                device: ui_device,
                binding_epoch: new_epoch,
                scope,
                mouse: MouseInputState::default(),
            },
        );
        assert!(
            runner
                .retire_native_input_recipient(secondary)
                .expect("actual verified UI recipient retirement")
        );
        assert!(
            !runner.native_ui_mice.contains_key(&secondary),
            "verified recipient retirement still retires the native pointer"
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
            crate::plugins::ui::unmount_retired_native_ui_slot(&mut runner.state.world, slot)
                .is_err(),
            "an original live native recipient must block premature M13 destruction"
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
        // The native runner has retired the only recipient and destroyed its
        // original window identity. Now prove that the World teardown really
        // consumes this same mounted framework scope, without interpreting
        // recipient retirement as an implicit M13 shutdown.
        let original_scope = runner
            .state
            .world
            .resource_mut::<UiRuntimeSlotsResource>()
            .expect("mounted native UI slot")
            .input_ownership(slot)
            .expect("real framework M13 scope still running")
            .scope()
            .clone();
        let retirement =
            crate::plugins::ui::unmount_retired_native_ui_slot(&mut runner.state.world, slot)
                .expect("retired native slot may shut down through the World")
                .expect("exactly one runtime was still mounted");
        assert_eq!(retirement.final_ownership().scope(), &original_scope);
        assert_eq!(
            retirement.final_ownership().status(),
            runenui_runtime::RuntimeStatus::Closed
        );
        assert!(
            retirement.ordered_records().iter().any(|record| {
                matches!(
                    record,
                    runenui_runtime::InputArbitrationRecord::ScopeRetired(actual)
                        if actual.scope() == &original_scope
                )
            }),
            "actual original M13 scope retirement must survive runner teardown"
        );
        assert!(
            crate::plugins::ui::unmount_retired_native_ui_slot(&mut runner.state.world, slot)
                .expect("idempotent retired slot teardown")
                .is_none()
        );
    }

    #[test]
    fn native_focus_loss_retracts_game_actions_in_the_same_event_turn() {
        use crate::plugins::{ActionState, action};

        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let source = runner
            .native_input_mut()
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
                .native_input()
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
            .native_input_mut()
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
                .native_input()
                .window_source_context(NativeWindowId::primary()),
            None
        );
    }

    #[test]
    fn raw_removal_does_not_revoke_original_window_keyboard_or_game_held_press() {
        use crate::plugins::{ActionState, action};
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        runner.state.world.insert_resource(ActionState::new());
        let backend = winit::event::DeviceId::dummy();
        let original = runner
            .native_input_mut()
            .window_context(NativeWindowId::primary(), backend)
            .expect("original window-keyboard identity");
        let raw = runner
            .native_input_mut()
            .raw_device_context(backend)
            .expect("independent raw-device identity");
        let key = runen_input::PhysicalKeyIdentity::code("KeyW");
        let keyboard_event = |state| PlatformEvent::KeyboardInput {
            context: original,
            input: runen_input::KeyboardInput {
                physical_key: key.clone(),
                logical_key: runen_input::LogicalKey::Native(
                    runen_input::NativeLogicalKey::Unidentified,
                ),
                location: runen_input::KeyLocation::Standard,
                state,
                repeat: false,
                origin: runen_input::ObservationOrigin::SourceReport,
            },
        };
        runner
            .apply_event(keyboard_event(runen_input::DigitalState::Pressed))
            .expect("original window-keyboard Down enters neutral input");
        {
            let mut actions = runner
                .state
                .world
                .remove_resource::<ActionState>()
                .expect("game action owner");
            actions.project(
                runner
                    .state
                    .world
                    .resource::<InputState>()
                    .expect("neutral input"),
            );
            runner.state.world.insert_resource(actions);
        }
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .expect("game actions")
                .action_down(action::WORLD_MOVE_UP),
            "legitimate game-held key is live before unrelated raw removal"
        );
        assert!(
            runner
                .apply_raw_device_removal(backend)
                .expect("raw Removed"),
            "original raw source must retire"
        );
        assert_eq!(
            runner
                .native_input_mut()
                .window_context(NativeWindowId::primary(), backend),
            Ok(original),
            "a colliding raw ID must not reincarnate a window-keyboard identity"
        );
        assert_eq!(
            runner.native_input_mut().retire_raw_device(backend),
            Ok(None),
            "raw source is retired exactly once"
        );
        assert!(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("neutral input")
                .keyboard_down_generation_in(original, &key)
                .is_some(),
            "physical window key must survive raw source retirement"
        );
        assert!(
            runner
                .state
                .world
                .resource::<ActionState>()
                .expect("game actions")
                .action_down(action::WORLD_MOVE_UP),
            "raw removal must not revoke an independent held gameplay action"
        );
        let reincarnated = runner
            .native_input_mut()
            .raw_device_context(backend)
            .expect("new raw generation");
        assert_ne!(reincarnated.source, raw.source);
        assert_ne!(reincarnated.device, raw.device);
        runner
            .apply_event(keyboard_event(runen_input::DigitalState::Released))
            .expect("genuine original window-keyboard Up");
        let mut actions = runner
            .state
            .world
            .remove_resource::<ActionState>()
            .expect("game action owner");
        actions.project(
            runner
                .state
                .world
                .resource::<InputState>()
                .expect("neutral input"),
        );
        runner.state.world.insert_resource(actions);
        assert!(
            !runner
                .state
                .world
                .resource::<ActionState>()
                .expect("game actions")
                .action_down(action::WORLD_MOVE_UP),
            "only an actual original keyboard release ends that held key"
        );
    }

    #[test]
    fn raw_removal_failure_preserves_original_raw_device_identity() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        let backend = winit::event::DeviceId::dummy();
        let raw = runner
            .native_input_mut()
            .raw_device_context(backend)
            .expect("allocated original raw device");
        assert!(
            runner.apply_raw_device_removal(backend).is_err(),
            "missing neutral input owner must reject before raw identity retirement"
        );
        assert_eq!(
            runner.native_input_mut().raw_device_context(backend),
            Ok(raw),
            "failure before source-loss delivery cannot reincarnate hardware"
        );
    }

    #[test]
    fn raw_device_removal_retires_only_raw_source_and_preserves_window_device() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::on_demand());
        runner.state.world.insert_resource(InputState::new());
        let backend_device = winit::event::DeviceId::dummy();
        let window = runner
            .native_input_mut()
            .window_context(NativeWindowId::primary(), backend_device)
            .unwrap();
        let raw = runner
            .native_input_mut()
            .raw_device_context(backend_device)
            .unwrap();

        assert!(
            runner
                .apply_raw_device_removal(backend_device)
                .expect("raw device removal should apply")
        );
        let replacement = runner
            .native_input_mut()
            .raw_device_context(backend_device)
            .unwrap();
        assert_ne!(replacement.source, raw.source);
        assert_ne!(replacement.device, raw.device);
        let replacement_window = runner
            .native_input_mut()
            .window_context(NativeWindowId::primary(), backend_device)
            .expect("a new window-device lifetime on observed hardware reappearance");
        assert_eq!(
            replacement_window.source, window.source,
            "device loss is not destruction of the native window's input source"
        );
        assert_eq!(
            replacement_window.device, window.device,
            "a raw Removed cannot certify or reincarnate a window-event device"
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
    fn continuous_uncapped_keeps_host_turns_ready_without_a_timer() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::continuous_uncapped());
        let start = Instant::now();
        runner.last_primary_redraw_at = Some(start);
        assert!(runner.frame_is_ready(start));
        assert!(runner.should_request_primary_redraw_now(start));
        assert!(
            runner.should_request_primary_redraw_now(start + std::time::Duration::from_nanos(1))
        );
        let policy = runner.frame_pacing_policy();
        let next = runner.frame_pacing_schedule.decide(policy, start);
        assert!(next.request_redraw);
        assert!(
            next.next_deadline.is_none(),
            "uncapped Host never creates a timer deadline"
        );
    }

    #[test]
    fn live_uncapped_mode_switches_preserve_explicit_redraw_and_capped_readiness() {
        let mut runner = runner_with_frame_pacing(FramePacingPolicyResource::continuous_uncapped());
        let now = Instant::now();
        runner.last_primary_redraw_at = Some(now);
        assert!(runner.frame_is_ready(now));
        let policy = runner.frame_pacing_policy();
        let uncapped = runner.frame_pacing_schedule.decide(policy, now);
        assert!(uncapped.request_redraw);
        assert_eq!(uncapped.next_deadline, None);

        // A secondary logical window's explicit redraw intent belongs to the
        // window registry, not to the primary continuous pacing policy.
        let secondary = runner
            .state
            .world
            .resource_mut::<WindowStateRegistryResource>()
            .expect("window registry")
            .request_window("secondary", (640, 480))
            .native_window_id;
        runner.request_redraw_for_native_window(secondary);
        assert!(runner.frame_requested);

        *runner
            .state
            .world
            .resource_mut::<FramePacingPolicyResource>()
            .expect("frame pacing policy") = FramePacingPolicyResource::on_demand();
        let policy = runner.frame_pacing_policy();
        let on_demand = runner.frame_pacing_schedule.decide(policy, now);
        assert!(!on_demand.request_redraw);
        assert_eq!(on_demand.next_deadline, None);
        assert!(runner.frame_is_ready(now));
        assert!(runner.should_request_primary_redraw_now(now));
        assert!(
            runner
                .state
                .world
                .resource::<WindowStateRegistryResource>()
                .expect("window registry")
                .record(secondary)
                .expect("secondary record")
                .redraw_requested,
            "switching to on-demand must preserve an explicit secondary redraw"
        );
        runner.frame_requested = false;
        assert!(
            !runner
                .frame_pacing_schedule
                .decide(policy, now)
                .request_redraw,
            "on-demand must not keep the former uncapped automatic requests alive"
        );

        let capped = FramePacingPolicyResource::continuous_capped(30);
        *runner
            .state
            .world
            .resource_mut::<FramePacingPolicyResource>()
            .expect("frame pacing policy") = capped;
        let interval = capped.target_frame_interval().expect("30 FPS interval");
        assert!(!runner.frame_is_ready(now));
        let resumed = runner.frame_pacing_schedule.decide(capped, now);
        assert!(resumed.request_redraw);
        assert_eq!(resumed.next_deadline, Some(now + interval));
        assert!(
            !runner
                .frame_pacing_schedule
                .decide(capped, now + interval / 2)
                .request_redraw
        );
        assert!(runner.frame_is_ready(now + interval));
        assert!(
            runner
                .frame_pacing_schedule
                .decide(capped, now + interval)
                .request_redraw
        );
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
            .native_input_mut()
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
            .native_input_mut()
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
