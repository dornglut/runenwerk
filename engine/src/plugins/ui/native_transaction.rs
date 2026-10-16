//! Atomic Engine World boundary for selected native RunenUI app mutations.
//!
//! The native binding and original Winit source remain the sole owners of
//! their identities. This boundary changes only Engine game permission, via
//! the existing ActionState, when a framework M13 transition or observation
//! uncertainty makes a previously projected key no longer trustworthy.

use anyhow::{Result, anyhow};
use runen_ecs::World;
use runenui_runtime::RuntimeStatus;

use crate::plugins::input::{ActionState, InputState};
use crate::runtime::winit_input::WinitInputAdapter;
use crate::runtime::{
    NativeWindowId, NativeWindowLifecycleState, WindowStateRegistryResource,
};

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
        Err(panic) => std::panic::resume_unwind(panic),
    };
    if changed {
        if let Some(source) = source {
            revoke_game_source(world, source)?;
        }
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
        return Err(anyhow!("native completed-Present associations are unavailable"));
    };
    let retired = recipients.retire_window(window, &mut presentations);
    world.insert_resource(presentations);
    world.insert_resource(recipients);
    if retired {
        if let Some(source) = source {
            revoke_game_source(world, source)?;
        }
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
    let valid_before = created
        && before.as_ref().is_ok_and(|snapshot| {
            snapshot.scope() == &scope && snapshot.status() == RuntimeStatus::Running
        });
    if !valid_before {
        if let Some(source) = source {
            revoke_game_source(world, source)?;
        }
        return Err(anyhow!(
            "native RunenUI app mutation denied: original window/scope/ownership not current (slot {slot:?}, epoch {epoch})"
        ));
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
            runenui_core::text("an ordinary native selected app")
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
    fn native_transaction_reports_failure_if_final_m13_scope_disappears() {
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

            // Exact selected-slot retirement is a real M13 scope loss, not
            // a fabricated success or synthetic keyboard release.
            slots.unmount_with_retirement(slot)?;
            Ok(())
        });
        assert!(
            result.is_err(),
            "a successful callback may not certify a missing final M13 scope"
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

        assert!(
            bind_native_input_recipient(
                &mut world,
                window,
                NativeInputRecipient::RunenUiSlot(slot)
            )
            .unwrap()
        );
        assert!(
            world
                .resource::<InputState>()
                .unwrap()
                .keyboard_down_generation_in(context, &PhysicalKeyIdentity::code("KeyW"))
                .is_some(),
            "new native binding must not synthesize original physical Up"
        );
        assert!(
            !world
                .resource::<ActionState>()
                .unwrap()
                .action_down(action::WORLD_MOVE_UP),
            "native source is revoked before the World binding call returns"
        );
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
}
