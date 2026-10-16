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
use crate::runtime::{NativeWindowLifecycleState, WindowStateRegistryResource};

use super::{
    UiNativeInputRecipientsResource, UiRuntimeSlotId, UiRuntimeSlotOperationFailure,
    UiRuntimeSlotsResource,
};

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
            .transact_native_app(slot, operation)
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
    if !valid_after || changed || !matches!(&outcome, Ok(Ok(_))) {
        if let Some(source) = source {
            revoke_game_source(world, source)?;
        }
    }
    match outcome {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::action;
    use crate::plugins::ui::{NativeInputRecipient, UiRuntimeFontConfiguration, UiTypedScreenId};
    use crate::runtime::{NativeWindowId, SystemConfigExt, SystemMobilityExt, Update, WorldMut};
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
