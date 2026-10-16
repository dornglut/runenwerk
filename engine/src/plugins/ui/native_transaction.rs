//! Atomic Engine World boundary for selected native RunenUI app mutations.
//!
//! The native binding and original Winit source remain the sole owners of
//! their identities. This boundary changes only Engine game permission, via
//! the existing ActionState, when a framework M13 transition or observation
//! uncertainty makes a previously projected key no longer trustworthy.

use anyhow::{Result, anyhow};
use runen_ecs::World;
use runenui_runtime::{InputShutdownBatch, RuntimeStatus};

use crate::plugins::input::{ActionState, InputState};
use crate::runtime::winit_input::WinitInputAdapter;
use crate::runtime::{NativeWindowId, NativeWindowLifecycleState, WindowStateRegistryResource};

use super::{
    NativeInputRecipient, NativeInputRecipientError, UiNativeInputRecipientsResource,
    UiRuntimePresentationAssociationsResource, UiRuntimeSlotId, UiRuntimeSlotOperationFailure,
    UiRuntimeSlotsResource,
};

/// This construction right is confined to the one World mutation module.
/// Merely holding a mutable `UiRuntimeSlotsResource` cannot manufacture it.
pub(crate) struct NativeAppMutationPermit {
    _sealed: (),
}

#[cfg(test)]
pub(crate) fn test_retired_slot_teardown(
    slots: &mut UiRuntimeSlotsResource,
    slot: UiRuntimeSlotId,
) -> std::result::Result<Option<InputShutdownBatch>, UiRuntimeSlotOperationFailure> {
    // Recipient tests must first verify the live binding/intent is retired.
    // Only the production World boundary may normally grant this permit.
    slots.unmount_retired_native(&NativeAppMutationPermit { _sealed: () }, slot)
}

#[cfg(test)]
pub(crate) fn test_selected_slot_operation<T>(
    slots: &mut UiRuntimeSlotsResource,
    slot: UiRuntimeSlotId,
    operation: impl FnOnce(
        &mut UiRuntimeSlotsResource,
    ) -> std::result::Result<T, UiRuntimeSlotOperationFailure>,
) -> std::result::Result<T, UiRuntimeSlotOperationFailure> {
    // Lower-level recipient/Present unit tests explicitly model an already
    // validated native World transaction; this path exists only in tests.
    slots.transact_native_app(&NativeAppMutationPermit { _sealed: () }, slot, operation)
}

/// Perform a native recipient selection inside the same exclusive World
/// boundary as the existing original source and gameplay ActionState owners.
/// The registry's lower-level methods are crate-private to prevent an ECS
/// caller from bypassing revocation during the same Update.
fn change_native_recipient(
    world: &mut World,
    window: NativeWindowId,
    operation: impl FnOnce(
        &mut UiNativeInputRecipientsResource,
        &WindowStateRegistryResource,
        &mut UiRuntimeSlotsResource,
    ) -> std::result::Result<bool, NativeInputRecipientError>,
) -> Result<bool> {
    // Reject inconsistent gameplay resources before the binding can commit.
    // With no original source yet, no prior Winit lifetime can be attributed
    // to this window; the adapter remains the sole source allocator.
    let source = world
        .resource::<WinitInputAdapter>()
        .ok()
        .and_then(|adapter| adapter.window_source_context(window))
        .map(|context| context.source);
    if source.is_some()
        && world.resource::<ActionState>().is_ok()
        && world.resource::<InputState>().is_err()
    {
        return Err(anyhow!(
            "cannot change native input recipient without its physical InputState"
        ));
    }
    world.resource::<WindowStateRegistryResource>()?;
    let mut recipients = world
        .remove_resource::<UiNativeInputRecipientsResource>()
        .ok_or_else(|| anyhow!("native recipient registry is unavailable"))?;
    let Some(mut slots) = world.remove_resource::<UiRuntimeSlotsResource>() else {
        world.insert_resource(recipients);
        return Err(anyhow!("native RunenUI slot registry is unavailable"));
    };
    // The exclusive World has already proved the window resource exists.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        operation(
            &mut recipients,
            world
                .resource::<WindowStateRegistryResource>()
                .expect("checked native window registry"),
            &mut slots,
        )
    }));
    world.insert_resource(slots);
    world.insert_resource(recipients);
    let changed = match outcome {
        Ok(result) => result?,
        Err(panic) => {
            // The bind callback may have committed before unwinding. Restore
            // the exact original sources first, then deny any previously
            // projected gameplay key before propagating the panic.
            if let Some(source) = source {
                let _ = revoke_game_source(world, source);
            }
            std::panic::resume_unwind(panic)
        }
    };
    if changed && let Some(source) = source {
        revoke_game_source(world, source)?;
    }
    Ok(changed)
}

/// Bind an explicit native input recipient, synchronously retiring prior
/// gameplay permission from the original window source on a new binding.
/// A failed or idempotent binding changes no game permission.
///
/// This World-level API is required for ECS callers. The registry cannot be
/// mutated directly by external ECS systems.
pub fn bind_native_input_recipient(
    world: &mut World,
    window: NativeWindowId,
    recipient: NativeInputRecipient,
) -> Result<bool> {
    change_native_recipient(world, window, |recipients, windows, slots| {
        recipients.bind(window, recipient, windows, slots)
    })
}

/// Realize a prior explicit primary recipient request without bypassing the
/// same-schedule game-input revocation boundary.
pub fn realize_requested_native_primary(world: &mut World) -> Result<bool> {
    change_native_recipient(
        world,
        NativeWindowId::primary(),
        |recipients, windows, slots| recipients.realize_requested_primary(windows, slots),
    )
}

/// Retire an original native window's selected recipient and its completed
/// display mapping, then synchronously deny old gameplay held-key generations.
/// This does not synthesize any RunenInput physical release.
pub fn retire_native_input_recipient(world: &mut World, window: NativeWindowId) -> Result<bool> {
    let source = world
        .resource::<WinitInputAdapter>()
        .ok()
        .and_then(|adapter| adapter.window_source_context(window))
        .map(|context| context.source);
    if source.is_some()
        && world.resource::<ActionState>().is_ok()
        && world.resource::<InputState>().is_err()
    {
        return Err(anyhow!(
            "cannot retire native input recipient without its physical InputState"
        ));
    }
    let mut recipients = world
        .remove_resource::<UiNativeInputRecipientsResource>()
        .ok_or_else(|| anyhow!("native recipient registry is unavailable"))?;
    let Some(mut presentations) =
        world.remove_resource::<UiRuntimePresentationAssociationsResource>()
    else {
        world.insert_resource(recipients);
        return Err(anyhow!(
            "native completed-Present associations are unavailable"
        ));
    };
    let retired = recipients.retire_window(window, &mut presentations);
    world.insert_resource(presentations);
    world.insert_resource(recipients);
    if retired && let Some(source) = source {
        revoke_game_source(world, source)?;
    }
    Ok(retired)
}

fn revoke_game_source(world: &mut World, source: runen_input::InputSourceId) -> Result<()> {
    if let Some(mut actions) = world.remove_resource::<ActionState>() {
        let result = world.resource::<InputState>().map(|input| {
            actions.revoke_source_keyboard_for_ui_ownership(input, source);
        });
        world.insert_resource(actions);
        result?;
    }
    Ok(())
}

/// Shut down one previously native-enrolled slot only after retiring its
/// original recipient. OS-owned streams must first be cancelled against the
/// existing M13 scope; unmount cannot perform those native cancellations.
pub fn unmount_retired_native_ui_slot(
    world: &mut World,
    slot: UiRuntimeSlotId,
) -> Result<Option<InputShutdownBatch>> {
    let selected = world
        .resource::<UiRuntimeSlotsResource>()?
        .was_native_input_selected(slot);
    let Some(selected) = selected else {
        return Ok(None);
    };
    if !selected {
        return Err(anyhow!(
            "slot {slot:?} is not native-enrolled; its ordinary lifecycle owns unmount"
        ));
    }
    if world
        .resource::<UiNativeInputRecipientsResource>()?
        .has_native_slot_intent(slot)
    {
        return Err(anyhow!(
            "slot {slot:?} still has a native input recipient or pending primary intent; retire it before unmount"
        ));
    }
    // Check the display authority before irreversible framework shutdown.
    world.resource::<UiRuntimePresentationAssociationsResource>()?;
    let retirement = world
        .resource_mut::<UiRuntimeSlotsResource>()?
        .unmount_retired_native(&NativeAppMutationPermit { _sealed: () }, slot)
        .map_err(|error| anyhow!("retired native slot {slot:?} shutdown failed: {error:?}"))?;
    if retirement.is_some() {
        // Unbind also schedules exactly the existing renderer paint withdrawal.
        world
            .resource_mut::<UiRuntimePresentationAssociationsResource>()?
            .unbind(slot);
    }
    Ok(retirement)
}

/// Execute a selected-native RunenUI app mutation and withdraw incompatible
/// game input *before the next ECS system can read its cached ActionState*.
///
/// An unselected/headless slot retains its ordinary direct mutation API. A
/// selected-native slot's bare mutators reject with
/// `NativeInputTransactionRequired`; only this boundary receives the
/// temporary mutation capability, after validating the existing binding.
///
/// A revision change is evidence of a missing interval, not a claimed UI
/// key settlement. Fail closed on errors and incomplete/terminal observation,
/// without erasing RunenInput's original physical Down.
pub fn transact_ui_slot<T>(
    world: &mut World,
    slot: UiRuntimeSlotId,
    operation: impl FnOnce(
        &mut UiRuntimeSlotsResource,
    ) -> std::result::Result<T, UiRuntimeSlotOperationFailure>,
) -> Result<T> {
    transact_ui_slot_inner(world, slot, false, operation)
}

/// Mount a new typed RunenUI app while a native World is running.
///
/// Merely adding the first mounted UI runtime changes unbound native windows
/// from implicit game-only recipients into unresolved recipients. Therefore
/// the canonical original input sources and the one Engine ActionState must
/// be reconciled before this operation returns to another ECS system.
/// Headless/configuration-time mounts retain the ordinary mounting API.
pub fn mount_native_ui_app<A>(
    world: &mut World,
    screen_id: super::UiTypedScreenId,
    state: A::State,
    fonts: &super::UiRuntimeFontConfiguration,
) -> Result<super::UiRuntimeSlotMountReport>
where
    A: runenui_core::UiApp + 'static,
{
    transact_native_ui_mount(world, |slots, permit| {
        slots.mount_native_app::<A>(permit, screen_id, state, fonts)
    })
}

/// Mount a legacy UiProgram through the same native World transaction as
/// typed UiApp mounting. Selection remains independent from presentation.
pub fn mount_native_ui_program(
    world: &mut World,
    source: super::UiTypedSource,
    registry: &ui_controls::ControlPackageRegistrySnapshot,
    fonts: &super::UiRuntimeFontConfiguration,
) -> Result<super::UiRuntimeSlotMountReport> {
    transact_native_ui_mount(world, |slots, permit| {
        slots.mount_native_program(permit, source, registry, fonts)
    })
}

fn transact_native_ui_mount(
    world: &mut World,
    operation: impl FnOnce(
        &mut UiRuntimeSlotsResource,
        &NativeAppMutationPermit,
    ) -> super::UiRuntimeSlotMountReport,
) -> Result<super::UiRuntimeSlotMountReport> {
    let sources = world
        .resource::<WinitInputAdapter>()?
        .window_source_ids()
        .collect::<Vec<_>>();
    world.resource::<WindowStateRegistryResource>()?;
    world.resource::<UiNativeInputRecipientsResource>()?;
    world.resource::<UiRuntimeSlotsResource>()?;
    if !sources.is_empty()
        && world.resource::<ActionState>().is_ok()
        && world.resource::<InputState>().is_err()
    {
        return Err(anyhow!(
            "cannot mount native RunenUI while physical InputState is unavailable"
        ));
    }

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        operation(
            world
                .resource_mut::<UiRuntimeSlotsResource>()
                .expect("native slot registry verified before mutable mounting"),
            &NativeAppMutationPermit { _sealed: () },
        )
    }));
    // The caller may catch a panic from user-authored UiApp mount code.
    // Reconcile even then: partial native runtime changes cannot preserve
    // previously projected gameplay eligibility on an unknown route.
    if !matches!(&outcome, Ok(report) if !report.accepted_mount())
        && let Err(error) = reconcile_game_after_native_mount(world, &sources)
    {
        for (_, source) in sources {
            let _ = revoke_game_source(world, source);
        }
        match outcome {
            Ok(_) => return Err(error),
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }
    match outcome {
        Ok(report) => Ok(report),
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn reconcile_game_after_native_mount(
    world: &mut World,
    sources: &[(NativeWindowId, runen_input::InputSourceId)],
) -> Result<()> {
    let mut slots = world
        .remove_resource::<UiRuntimeSlotsResource>()
        .ok_or_else(|| anyhow!("native UI slots missing during mount reconciliation"))?;
    let decisions = (|| -> Result<Vec<_>> {
        let windows = world.resource::<WindowStateRegistryResource>()?;
        let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
        Ok(sources
            .iter()
            .filter_map(|(window, source)| {
                (!matches!(
                    recipients.resolve(*window, windows, &mut slots),
                    Ok(NativeInputRecipient::LegacyNativeConsumer)
                ))
                .then_some(*source)
            })
            .collect())
    })();
    world.insert_resource(slots);
    for source in decisions? {
        revoke_game_source(world, source)?;
    }
    Ok(())
}

/// The only native-selected recovery lane for an incomplete RunenUI pump.
/// A pending M13 snapshot cannot certify old gameplay permission: withdraw
/// the exact source *before* attempting bounded framework-owned settlement,
/// then validate the current binding/scope again before success.
pub fn settle_native_ui_slot(world: &mut World, slot: UiRuntimeSlotId) -> Result<()> {
    transact_ui_slot_inner(world, slot, true, |slots| slots.settle_slot(slot))
}

fn transact_ui_slot_inner<T>(
    world: &mut World,
    slot: UiRuntimeSlotId,
    allow_pending_settlement: bool,
    operation: impl FnOnce(
        &mut UiRuntimeSlotsResource,
    ) -> std::result::Result<T, UiRuntimeSlotOperationFailure>,
) -> Result<T> {
    let selected = world
        .resource::<UiNativeInputRecipientsResource>()
        .ok()
        .and_then(|recipients| recipients.selected_slot_binding(slot));
    let Some((window, epoch, scope)) = selected else {
        // A previously selected/retired slot is still latched: the bare
        // operation fails rather than gaining a capability from missing
        // recipient state. Ordinary headless slots are unchanged.
        return operation(world.resource_mut::<UiRuntimeSlotsResource>()?)
            .map_err(|err| anyhow!("RunenUI ordinary slot mutation failed: {err:?}"));
    };
    let source = world
        .resource::<WinitInputAdapter>()
        .ok()
        .and_then(|adapter| adapter.window_source_context(window))
        .map(|context| context.source);
    let created = world
        .resource::<WindowStateRegistryResource>()
        .ok()
        .and_then(|windows| windows.record(window))
        .is_some_and(|record| record.lifecycle_state == NativeWindowLifecycleState::Created);
    let before = world
        .resource_mut::<UiRuntimeSlotsResource>()?
        .input_ownership(slot);
    let pending = matches!(
        &before,
        Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
    );
    let valid_before = created
        && (before.as_ref().is_ok_and(|snapshot| {
            snapshot.scope() == &scope && snapshot.status() == RuntimeStatus::Running
        }) || (allow_pending_settlement && pending));
    if !valid_before {
        if let Some(source) = source {
            revoke_game_source(world, source)?;
        }
        return Err(anyhow!(
            "native RunenUI app mutation denied: original window/scope/ownership not current (slot {slot:?}, epoch {epoch})"
        ));
    }
    if pending && allow_pending_settlement {
        // The last RunenUI snapshot was not observable. Revoke before any
        // partial pump could run and before this ECS invocation returns.
        if let Some(source) = source {
            revoke_game_source(world, source)?;
        }
    }
    // A panic in user-authored UiApp code must not leave a mutation permit
    // installed. The slot owner resets the permit on unwind; this World
    // boundary also withdraws old gameplay permission before propagating.
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world
            .resource_mut::<UiRuntimeSlotsResource>()
            .map_err(|err| anyhow!("native RunenUI slots unavailable: {err}"))?
            .transact_native_app(&NativeAppMutationPermit { _sealed: () }, slot, operation)
            .map_err(|err| anyhow!("native RunenUI app mutation failed: {err:?}"))
    }));
    let after = world
        .resource_mut::<UiRuntimeSlotsResource>()
        .map_err(|err| anyhow!("native RunenUI slots unavailable: {err}"))
        .and_then(|slots| {
            slots
                .input_ownership(slot)
                .map_err(|err| anyhow!("native RunenUI ownership observation failed: {err:?}"))
        });
    let same_binding = world
        .resource::<UiNativeInputRecipientsResource>()
        .ok()
        .and_then(|recipients| recipients.selected_slot_binding(slot))
        .is_some_and(|(current_window, current_epoch, current_scope)| {
            current_window == window && current_epoch == epoch && current_scope == scope
        });
    let valid_after = same_binding
        && after.as_ref().is_ok_and(|current| {
            current.scope() == &scope && current.status() == RuntimeStatus::Running
        });
    let changed = match (&before, &after) {
        (Ok(previous), Ok(current)) => previous.revision() != current.revision(),
        _ => true,
    };
    if let Some(source) = source
        && (!valid_after || changed || !matches!(&outcome, Ok(Ok(_))))
    {
        revoke_game_source(world, source)?;
    }
    match outcome {
        Ok(Ok(value)) if valid_after => Ok(value),
        Ok(Ok(_)) => Err(anyhow!(
            "native RunenUI app mutation lost its binding or final M13 ownership observation"
        )),
        Ok(Err(error)) => Err(error),
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::action;
    use crate::plugins::ui::{NativeInputRecipient, UiRuntimeFontConfiguration, UiTypedScreenId};
    use crate::runtime::{SystemConfigExt, SystemMobilityExt, Update, WorldMut};
    use runen_ecs::{Runtime, SystemSet};
    use runen_input::{
        DigitalState, InputContext, KeyLocation, KeyboardInput, LogicalKey, NativeLogicalKey,
        ObservationOrigin, PhysicalKeyIdentity,
    };
    use runenui_core::{NoHostProtocol, UiApp, View};

    struct ProbeApp;
    impl UiApp for ProbeApp {
        type State = ();
        type Action = ();
        type HostProtocol = NoHostProtocol;

        fn root(_: &Self::State) -> impl View<Self::Action> {
            runenui_core::button("native focus probe").on_activate(|| ())
        }

        fn update(
            _: &mut Self::State,
            _: Self::Action,
        ) -> impl runenui_core::IntoUpdateOutput<Self::Action, Self::HostProtocol> {
        }
    }

    #[derive(runen_ecs::Resource)]
    struct Probe {
        slot: UiRuntimeSlotId,
        ui: InputContext,
        game: InputContext,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
    struct NativeMutation;
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
    struct ImmediateReader;

    fn key(state: DigitalState) -> KeyboardInput {
        KeyboardInput {
            physical_key: PhysicalKeyIdentity::code("KeyW"),
            logical_key: LogicalKey::Native(NativeLogicalKey::Unidentified),
            location: KeyLocation::Standard,
            state,
            repeat: false,
            origin: ObservationOrigin::SourceReport,
        }
    }

    fn setup() -> World {
        let mut world = World::new();
        let mut windows = WindowStateRegistryResource::default();
        let ui_window = windows.register_primary_window("UI", (800, 600), 1.0, true);
        let game_window = crate::runtime::NativeWindowId::try_from_raw(2).unwrap();
        windows.register_created_window(game_window, "game", (640, 480), 1.0, false);

        let mut adapter = WinitInputAdapter::default();
        let ui = adapter
            .window_context(ui_window, winit::event::DeviceId::dummy())
            .unwrap();
        let game = adapter
            .window_context(game_window, winit::event::DeviceId::dummy())
            .unwrap();
        assert_ne!(ui.source, game.source);

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<ProbeApp>(
                UiTypedScreenId::new("native.world.mutation"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        let mut recipients = UiNativeInputRecipientsResource::default();
        recipients
            .bind(
                ui_window,
                NativeInputRecipient::RunenUiSlot(slot),
                &windows,
                &mut slots,
            )
            .unwrap();
        recipients
            .bind(
                game_window,
                NativeInputRecipient::LegacyNativeConsumer,
                &windows,
                &mut slots,
            )
            .unwrap();

        let mut input = InputState::new();
        input.handle_normalized_keyboard(ui, &key(DigitalState::Pressed));
        input.handle_normalized_keyboard(game, &key(DigitalState::Pressed));
        let mut actions = ActionState::new();
        actions.project(&input);
        assert!(actions.action_down(action::WORLD_MOVE_UP));

        world.insert_resource(windows);
        world.insert_resource(adapter);
        world.insert_resource(slots);
        world.insert_resource(recipients);
        world.insert_resource(input);
        world.insert_resource(actions);
        world.insert_resource(Probe { slot, ui, game });
        world
    }

    fn mutate(mut world: WorldMut) -> Result<()> {
        let slot = world.resource::<Probe>()?.slot;
        // Direct ECS resource mutation is a demonstrated forbidden bypass.
        assert!(matches!(
            world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .submit_app_action::<ProbeApp>(slot, ()),
            Err(UiRuntimeSlotOperationFailure::NativeInputTransactionRequired)
        ));
        // An actually admitted RunenUI application action followed by a
        // partial/fallible host outcome must synchronously withdraw the exact
        // source's old game eligibility before this system returns.
        let outcome = transact_ui_slot(&mut world, slot, |slots| {
            slots.submit_app_action::<ProbeApp>(slot, ())?;
            Err::<(), _>(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        });
        assert!(outcome.is_err());
        assert!(
            world
                .resource::<ActionState>()?
                .action_down(action::WORLD_MOVE_UP),
            "unrelated live game window still holds a genuine W"
        );
        Ok(())
    }

    fn mutate_real_m13_focus(mut world: WorldMut) -> Result<()> {
        let slot = world.resource::<Probe>()?.slot;
        let before = world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .input_ownership(slot)
            .map_err(|error| anyhow!("before M13 observation failed: {error:?}"))?;
        assert!(before.keyboard().focused_node().is_none());
        transact_ui_slot(&mut world, slot, |slots| {
            slots.test_request_first_typed_focus::<ProbeApp>(slot)
        })?;
        let after = world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .input_ownership(slot)
            .map_err(|error| anyhow!("after M13 observation failed: {error:?}"))?;
        assert_eq!(before.scope(), after.scope());
        assert_ne!(
            before.revision(),
            after.revision(),
            "real RunenUI semantic focus must advance M13 ownership"
        );
        assert!(
            after.keyboard().focused_node().is_some(),
            "framework M13 must have actually accepted focus"
        );
        assert!(
            world
                .resource::<ActionState>()?
                .action_down(action::WORLD_MOVE_UP),
            "the independent legacy physical source retains its held W"
        );
        Ok(())
    }

    fn observe_immediately(mut world: WorldMut) -> Result<()> {
        let (ui, game) = {
            let probe = world.resource::<Probe>()?;
            (probe.ui, probe.game)
        };
        world
            .resource_mut::<InputState>()?
            .handle_normalized_keyboard(game, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>()?);
        world.insert_resource(actions);
        assert!(
            world
                .resource::<InputState>()?
                .keyboard_down_generation_in(ui, &PhysicalKeyIdentity::code("KeyW"),)
                .is_some(),
            "RunenInput alone still owns the real UI-window physical W"
        );
        assert!(
            !world
                .resource::<ActionState>()?
                .action_down(action::WORLD_MOVE_UP),
            "next system in same Update cannot consume stale UI-window game W"
        );
        Ok(())
    }

    #[test]
    fn native_app_action_failure_revokes_before_next_ordered_ecs_update_reader() {
        let mut world = setup();
        let mut runtime = Runtime::new();
        runtime
            .add_systems(Update, mutate.on_invoker_thread().in_set(NativeMutation))
            .unwrap();
        runtime
            .add_systems(
                Update,
                observe_immediately
                    .on_invoker_thread()
                    .in_set(ImmediateReader)
                    .after(NativeMutation),
            )
            .unwrap();
        runtime.run_schedule::<Update>(&mut world).unwrap();
    }

    #[test]
    fn real_m13_focus_mutation_revokes_before_next_ecs_update_reader() {
        let mut world = setup();
        let mut runtime = Runtime::new();
        runtime
            .add_systems(
                Update,
                mutate_real_m13_focus
                    .on_invoker_thread()
                    .in_set(NativeMutation),
            )
            .unwrap();
        runtime
            .add_systems(
                Update,
                observe_immediately
                    .on_invoker_thread()
                    .in_set(ImmediateReader)
                    .after(NativeMutation),
            )
            .unwrap();
        runtime.run_schedule::<Update>(&mut world).unwrap();
    }

    #[test]
    fn selected_native_incomplete_prestate_has_one_fail_closed_settlement_lane() {
        let mut world = setup();
        let (slot, ui, game) = {
            let probe = world.resource::<Probe>().unwrap();
            (probe.slot, probe.ui, probe.game)
        };
        world
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap()
            .test_mark_native_pending(slot)
            .unwrap();
        assert!(matches!(
            world
                .resource_mut::<UiRuntimeSlotsResource>()
                .unwrap()
                .input_ownership(slot),
            Err(UiRuntimeSlotOperationFailure::RuntimeDidNotQuiesce)
        ));

        let mut called = false;
        let rejected = transact_ui_slot(&mut world, slot, |_slots| {
            called = true;
            Ok(())
        });
        assert!(rejected.is_err());
        assert!(
            !called,
            "ordinary mutation may not bypass missing M13 prestate"
        );

        // Unlike normal ingress, a bounded real RunenUI settlement must
        // remain possible; it revokes the exact physical game lifetime
        // BEFORE running or observing the incomplete native slot.
        settle_native_ui_slot(&mut world, slot).unwrap();
        let final_ownership = world
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap()
            .input_ownership(slot)
            .unwrap();
        assert_eq!(final_ownership.status(), RuntimeStatus::Running);
        assert!(
            world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "independent game source must remain eligible"
        );
        assert!(
            world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(ui, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "no fake original source Up during settlement"
        );
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(game, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>().unwrap());
        world.insert_resource(actions);
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "missing prestate revoked the original UI-bound game generation"
        );
    }

    #[test]
    fn native_transaction_rejects_active_scope_destruction() {
        let mut world = setup();
        let (slot, ui, game) = {
            let probe = world.resource::<Probe>().unwrap();
            (probe.slot, probe.ui, probe.game)
        };
        let result = transact_ui_slot(&mut world, slot, |slots| {
            // A transaction for one native-selected slot may not pump or
            // shut down ALL mounted runtimes while sibling sources have
            // already projected game input in the same ECS schedule.
            assert!(slots.settle_native_slots().is_err());
            assert!(slots.shutdown_native_slots().is_err());
            assert!(slots.contains(slot), "rejected global shutdown was inert");

            // Ordinary app mutation may not secretly shut down the active
            // native M13 scope, even with its transient mutation permit.
            slots.unmount_with_retirement(slot)?;
            Ok(())
        });
        assert!(
            result.is_err(),
            "the active selected native scope cannot be removed by app mutation"
        );
        assert!(
            world
                .resource::<UiRuntimeSlotsResource>()
                .unwrap()
                .contains(slot),
            "rejected shutdown cannot destroy the selected runtime"
        );
        assert!(
            world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(ui, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "framework shutdown must not manufacture original physical key Up"
        );
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(game, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>().unwrap());
        world.insert_resource(actions);
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "invalid final M13 scope must revoke old UI-source game permission"
        );
    }

    #[derive(runen_ecs::Resource)]
    struct PendingNativeBinding {
        window: NativeWindowId,
        slot: UiRuntimeSlotId,
        original: InputContext,
    }

    fn bind_native_during_update(mut world: WorldMut) -> Result<()> {
        let (window, slot) = {
            let pending = world.resource::<PendingNativeBinding>()?;
            (pending.window, pending.slot)
        };
        assert!(bind_native_input_recipient(
            &mut world,
            window,
            NativeInputRecipient::RunenUiSlot(slot)
        )?);
        Ok(())
    }

    fn read_game_after_native_binding(world: WorldMut) -> Result<()> {
        let original = world.resource::<PendingNativeBinding>()?.original;
        assert!(
            world
                .resource::<InputState>()?
                .keyboard_down_generation_in(original, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "new binding must preserve original physical Down"
        );
        assert!(
            !world
                .resource::<ActionState>()?
                .action_down(action::WORLD_MOVE_UP),
            "following ECS Update system cannot observe the previous game hold"
        );
        Ok(())
    }

    #[test]
    fn world_native_binding_revokes_prior_game_hold_without_physical_release() {
        let mut world = World::new();
        let mut windows = WindowStateRegistryResource::default();
        let window = windows.register_primary_window("new UI", (640, 480), 1.0, true);
        let mut adapter = WinitInputAdapter::default();
        let context = adapter
            .window_context(window, winit::event::DeviceId::dummy())
            .unwrap();
        let mut input = InputState::new();
        input.handle_normalized_keyboard(context, &key(DigitalState::Pressed));
        let mut actions = ActionState::new();
        actions.project(&input);
        assert!(actions.action_down(action::WORLD_MOVE_UP));

        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<ProbeApp>(
                UiTypedScreenId::new("native.world.binding"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        world.insert_resource(windows);
        world.insert_resource(adapter);
        world.insert_resource(slots);
        world.insert_resource(UiNativeInputRecipientsResource::default());
        world.insert_resource(input);
        world.insert_resource(actions);

        world.insert_resource(PendingNativeBinding {
            window,
            slot,
            original: context,
        });
        let mut runtime = Runtime::new();
        runtime
            .add_systems(
                Update,
                bind_native_during_update
                    .on_invoker_thread()
                    .in_set(NativeMutation),
            )
            .unwrap();
        runtime
            .add_systems(
                Update,
                read_game_after_native_binding
                    .on_invoker_thread()
                    .in_set(ImmediateReader)
                    .after(NativeMutation),
            )
            .unwrap();
        runtime.run_schedule::<Update>(&mut world).unwrap();
        assert!(matches!(
            world
                .resource_mut::<UiRuntimeSlotsResource>()
                .unwrap()
                .submit_app_action::<ProbeApp>(slot, ()),
            Err(UiRuntimeSlotOperationFailure::NativeInputTransactionRequired)
        ));
    }

    #[test]
    fn world_native_retirement_revokes_old_source_in_one_transaction() {
        let mut world = setup();
        let source = world.resource::<Probe>().unwrap().ui;
        let window = NativeWindowId::primary();
        world.insert_resource(UiRuntimePresentationAssociationsResource::default());
        assert!(
            world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "an already projected original W must be invalidated"
        );
        assert!(retire_native_input_recipient(&mut world, window).unwrap());
        assert!(
            world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(source, &PhysicalKeyIdentity::code("KeyW"))
                .is_some()
        );
        assert!(
            world
                .resource::<UiNativeInputRecipientsResource>()
                .unwrap()
                .selected_slot_binding(world.resource::<Probe>().unwrap().slot)
                .is_none(),
            "the exact retired UI scope is no longer an active native binding"
        );
        // The independent other window can still contribute its own held W.
        assert!(
            world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP)
        );
        let other = world.resource::<Probe>().unwrap().game;
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(other, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>().unwrap());
        world.insert_resource(actions);
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "retired native UI source cannot keep game W held"
        );
    }

    fn retire_then_unmount_in_update(mut world: WorldMut) -> Result<()> {
        let (slot, ui) = {
            let probe = world.resource::<Probe>()?;
            (probe.slot, probe.ui)
        };
        assert!(
            unmount_retired_native_ui_slot(&mut world, slot).is_err(),
            "an active original recipient must never be removed before cancellation"
        );
        assert!(world.resource::<UiRuntimeSlotsResource>()?.contains(slot));
        let original_scope = world
            .resource_mut::<UiRuntimeSlotsResource>()?
            .input_ownership(slot)
            .map_err(|error| anyhow!("original native M13 scope unavailable: {error:?}"))?
            .scope()
            .clone();
        assert!(retire_native_input_recipient(
            &mut world,
            NativeWindowId::primary()
        )?);
        assert!(matches!(
            world
                .resource_mut::<UiRuntimeSlotsResource>()?
                .unmount_with_retirement(slot),
            Err(UiRuntimeSlotOperationFailure::NativeInputTransactionRequired)
        ));
        let shutdown = unmount_retired_native_ui_slot(&mut world, slot)?
            .expect("retired M13 scope must shut down through the World");
        assert_eq!(shutdown.final_ownership().scope(), &original_scope);
        assert_eq!(shutdown.final_ownership().status(), RuntimeStatus::Closed);
        assert!(
            shutdown.ordered_records().iter().any(|record| {
                matches!(
                    record,
                    runenui_runtime::InputArbitrationRecord::ScopeRetired(retirement)
                        if retirement.scope() == &original_scope
                )
            }),
            "native teardown must return the actual original framework scope retirement"
        );
        assert!(!world.resource::<UiRuntimeSlotsResource>()?.contains(slot));
        assert!(
            world
                .resource::<InputState>()?
                .keyboard_down_generation_in(ui, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "semantic retirement must not synthesize a physical key release"
        );
        Ok(())
    }

    #[test]
    fn retired_native_slot_teardown_is_ordered_and_cannot_restore_game_input() -> Result<()> {
        let mut world = setup();
        let slot = world.resource::<Probe>().unwrap().slot;
        let producer = crate::plugins::render::RenderFrameProducerId::try_from_raw(71).unwrap();
        let surface = crate::plugins::render::host::RenderSurfaceId::primary();
        let binding = super::super::UiRuntimePresentationBinding::new(
            slot,
            producer,
            surface,
            runenui_core::StyleEnvironment::default(),
        );
        let mut presentation = UiRuntimePresentationAssociationsResource::default();
        presentation.bind(binding).unwrap();
        world.insert_resource(presentation);
        let mut runtime = Runtime::new();
        runtime
            .add_systems(
                Update,
                retire_then_unmount_in_update
                    .on_invoker_thread()
                    .in_set(NativeMutation),
            )
            .unwrap();
        runtime
            .add_systems(
                Update,
                observe_immediately
                    .on_invoker_thread()
                    .in_set(ImmediateReader)
                    .after(NativeMutation),
            )
            .unwrap();
        runtime.run_schedule::<Update>(&mut world).unwrap();
        let presentations = world
            .resource_mut::<UiRuntimePresentationAssociationsResource>()
            .unwrap();
        assert!(
            presentations.binding(slot).is_none(),
            "retired runtime must not retain stale displayed-input binding"
        );
        assert_eq!(
            presentations.take_withdrawals(),
            vec![(producer, surface)],
            "retired native presentation must withdraw its exact renderer paint"
        );
        assert!(
            unmount_retired_native_ui_slot(&mut world, slot)?.is_none(),
            "a retired slot cannot be shut down twice"
        );
        Ok(())
    }

    #[test]
    fn retired_native_slot_missing_display_authority_preserves_runtime_for_recovery() {
        let mut world = setup();
        let slot = world.resource::<Probe>().unwrap().slot;
        world.insert_resource(UiRuntimePresentationAssociationsResource::default());
        let retired = retire_native_input_recipient(&mut world, NativeWindowId::primary()).unwrap();
        assert!(retired);
        let presentation = world
            .remove_resource::<UiRuntimePresentationAssociationsResource>()
            .unwrap();
        assert!(
            unmount_retired_native_ui_slot(&mut world, slot).is_err(),
            "missing native presentation owner must reject before framework shutdown"
        );
        assert!(
            world
                .resource::<UiRuntimeSlotsResource>()
                .unwrap()
                .contains(slot),
            "failure must preserve the original framework scope for recovery"
        );
        world.insert_resource(presentation);
        assert!(
            unmount_retired_native_ui_slot(&mut world, slot)
                .unwrap()
                .is_some(),
            "restored authoritative presentation enables exactly one real shutdown"
        );
    }

    #[test]
    fn native_binding_panic_restores_world_owners_and_denies_old_source() {
        let mut world = setup();
        let original = world.resource::<Probe>().unwrap().ui;
        let game = world.resource::<Probe>().unwrap().game;
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = change_native_recipient(
                &mut world,
                NativeWindowId::primary(),
                |_recipients,
                 _windows,
                 _slots|
                 -> std::result::Result<bool, NativeInputRecipientError> {
                    panic!("injected native recipient callback failure");
                },
            );
        }));
        assert!(caught.is_err());
        assert!(world.resource::<UiNativeInputRecipientsResource>().is_ok());
        assert!(world.resource::<UiRuntimeSlotsResource>().is_ok());
        assert!(
            world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(original, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "panic recovery must not fabricate original hardware Up"
        );
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(game, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>().unwrap());
        world.insert_resource(actions);
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "a failed native binding cannot leave the old UI-source game W eligible"
        );
    }

    #[test]
    fn non_native_slot_preserves_ordinary_direct_app_mutations() {
        let mut slots = UiRuntimeSlotsResource::default();
        let slot = slots
            .mount_app::<ProbeApp>(
                UiTypedScreenId::new("headless.direct.action"),
                (),
                &UiRuntimeFontConfiguration::default(),
            )
            .slot_id()
            .unwrap();
        slots.submit_app_action::<ProbeApp>(slot, ()).unwrap();
    }

    #[derive(runen_ecs::Resource)]
    struct FirstMountInput(InputContext);

    fn mount_first_ui_in_update(mut world: WorldMut) -> Result<()> {
        assert!(
            world
                .resource::<ActionState>()?
                .action_down(action::WORLD_MOVE_UP)
        );
        let report = mount_native_ui_app::<ProbeApp>(
            &mut world,
            UiTypedScreenId::new("native.dynamic.first"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )?;
        assert!(report.accepted_mount());
        Ok(())
    }

    fn read_after_first_ui_mount(mut world: WorldMut) -> Result<()> {
        let original = world.resource::<FirstMountInput>()?.0;
        assert!(
            world
                .resource::<InputState>()?
                .keyboard_down_generation_in(original, &PhysicalKeyIdentity::code("KeyW"))
                .is_some()
        );
        assert!(
            !world
                .resource::<ActionState>()?
                .action_down(action::WORLD_MOVE_UP),
            "first UI mount invalidates an old implicit-legacy game hold in the SAME Update"
        );
        let mut slots = world.remove_resource::<UiRuntimeSlotsResource>().unwrap();
        let resolution = {
            let recipients = world.resource::<UiNativeInputRecipientsResource>()?;
            let windows = world.resource::<WindowStateRegistryResource>()?;
            recipients.resolve(NativeWindowId::primary(), windows, &mut slots)
        };
        world.insert_resource(slots);
        assert!(resolution.is_err());
        Ok(())
    }

    #[test]
    fn first_native_ui_mount_revokes_implicit_game_hold_before_next_ecs_reader() {
        let mut world = World::new();
        let mut windows = WindowStateRegistryResource::default();
        let window = windows.register_primary_window("game", (640, 480), 1.0, true);
        let mut adapter = WinitInputAdapter::default();
        let context = adapter
            .window_context(window, winit::event::DeviceId::dummy())
            .unwrap();
        let mut input = InputState::new();
        input.handle_normalized_keyboard(context, &key(DigitalState::Pressed));
        let mut actions = ActionState::new();
        actions.project(&input);
        assert!(actions.action_down(action::WORLD_MOVE_UP));
        world.insert_resource(windows);
        world.insert_resource(adapter);
        world.insert_resource(UiNativeInputRecipientsResource::default());
        world.insert_resource(UiRuntimeSlotsResource::default());
        world.insert_resource(input);
        world.insert_resource(actions);
        world.insert_resource(FirstMountInput(context));
        let mut runtime = Runtime::new();
        runtime
            .add_systems(
                Update,
                mount_first_ui_in_update
                    .on_invoker_thread()
                    .in_set(NativeMutation),
            )
            .unwrap();
        runtime
            .add_systems(
                Update,
                read_after_first_ui_mount
                    .on_invoker_thread()
                    .in_set(ImmediateReader)
                    .after(NativeMutation),
            )
            .unwrap();
        runtime.run_schedule::<Update>(&mut world).unwrap();
    }

    #[test]
    fn direct_selected_native_mount_is_denied_but_world_mount_keeps_legacy_source() {
        let mut world = setup();
        let rejected = world
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap()
            .mount_app::<ProbeApp>(
                UiTypedScreenId::new("native.direct.denied"),
                (),
                &UiRuntimeFontConfiguration::default(),
            );
        assert_eq!(
            rejected.failure(),
            Some(super::super::UiRuntimeSlotMountFailure::NativeInputTransactionRequired)
        );
        let accepted = mount_native_ui_app::<ProbeApp>(
            &mut world,
            UiTypedScreenId::new("native.world.allowed"),
            (),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap();
        assert!(accepted.accepted_mount());
        assert!(
            world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "world transaction must preserve independent explicit legacy source"
        );
        let game = world.resource::<Probe>().unwrap().game;
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(game, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>().unwrap());
        world.insert_resource(actions);
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "selected UI source must remain denied after independent legacy Up"
        );
    }
    fn program_source() -> super::super::UiTypedSource {
        super::super::UiTypedSource::new(
            UiTypedScreenId::new("native.program"),
            ui_program::UiProgramSourceId::new("native.program.source"),
            ui_definition::UiNodeDefinition::Label {
                id: ui_definition::AuthoredId::new("native.program.label"),
                label: ui_definition::UiValueBinding::static_text("Native program"),
                availability: None,
            },
        )
    }

    fn program_registry() -> ui_controls::ControlPackageRegistrySnapshot {
        ui_controls::ControlPackageRegistry::new()
            .with_package(ui_controls::runenwerk_control_package())
            .unwrap()
            .snapshot()
    }

    #[test]
    fn selected_native_program_mount_cannot_bypass_world_input_consistency() {
        let mut world = setup();
        let rejected = world
            .resource_mut::<UiRuntimeSlotsResource>()
            .unwrap()
            .mount_with_fonts(
                program_source(),
                &program_registry(),
                &UiRuntimeFontConfiguration::default(),
            );
        assert_eq!(
            rejected.failure(),
            Some(super::super::UiRuntimeSlotMountFailure::NativeInputTransactionRequired),
            "the program/Counter mount cannot mutate native routing through a bare slot resource"
        );
    }

    #[test]
    fn world_native_program_mount_preserves_independent_legacy_input() {
        let mut world = setup();
        let result = mount_native_ui_program(
            &mut world,
            program_source(),
            &program_registry(),
            &UiRuntimeFontConfiguration::default(),
        )
        .unwrap();
        assert!(
            result.accepted_mount(),
            "a valid UI program must mount through the World transaction"
        );
        assert!(
            world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "a new visual-only program cannot revoke an explicitly legacy second window"
        );
        let game = world.resource::<Probe>().unwrap().game;
        world
            .resource_mut::<InputState>()
            .unwrap()
            .handle_normalized_keyboard(game, &key(DigitalState::Released));
        let mut actions = world.remove_resource::<ActionState>().unwrap();
        actions.project(world.resource::<InputState>().unwrap());
        world.insert_resource(actions);
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "the old native UI source may not regain game permission after legacy Up"
        );
    }
}
