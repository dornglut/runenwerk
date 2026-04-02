use ecs::prelude::*;
use ecs::{
    EventChannelConfig, EventLifetime, EventTracingPolicy, OverflowPolicy, QueryAccess,
    SystemParam, SystemParamError,
};
use scheduler::ScheduleLabel;
use scheduler::access::ConflictKind;
use scheduler::label::SystemSet;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

#[derive(Copy, Clone)]
struct Update;

impl ScheduleLabel for Update {
    fn name() -> &'static str {
        "Update"
    }
}

#[derive(Copy, Clone)]
struct GameplaySet;

impl SystemSet for GameplaySet {
    fn name() -> &'static str {
        "GameplaySet"
    }
}

#[derive(Copy, Clone)]
struct PostGameplaySet;

impl SystemSet for PostGameplaySet {
    fn name() -> &'static str {
        "PostGameplaySet"
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct Marker(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct Extra(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct Toggle;

#[derive(Debug, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct IndexedName(String);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct SeenCount(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DamageEvent(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct TargetEntity(Entity);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct Step(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct SpawnGate(bool);

#[derive(Debug, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct CountHistory(Vec<usize>);

#[derive(Debug, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct AddedChangedHistory(Vec<(usize, usize)>);

#[derive(Debug, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct EventHistory(Vec<usize>);

#[derive(Debug, PartialEq, Eq, ecs::Component, ecs::Resource)]
struct PresenceHistory(Vec<usize>);

#[derive(ecs::Resource)]
struct EscapedCommands(Option<Commands>);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SpawnMarkerDeferred(u32);

impl DeferredCommand<()> for SpawnMarkerDeferred {
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), ecs::CommandError> {
        world.spawn(Marker(self.0));
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct InsertExtraDeferred {
    entity: Entity,
    value: i32,
}

impl DeferredCommand<()> for InsertExtraDeferred {
    fn apply(self: Box<Self>, world: &mut World) -> Result<(), ecs::CommandError> {
        world.insert(self.entity, Extra(self.value))?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct LateObserveSet;

impl SystemSet for LateObserveSet {
    fn name() -> &'static str {
        "LateObserveSet"
    }
}

fn run_order_log() -> &'static Mutex<Vec<&'static str>> {
    static LOG: OnceLock<Mutex<Vec<&'static str>>> = OnceLock::new();
    LOG.get_or_init(|| Mutex::new(Vec::new()))
}

fn push_run_order(label: &'static str) {
    run_order_log().lock().unwrap().push(label);
}

fn clear_run_order() {
    run_order_log().lock().unwrap().clear();
}

fn snapshot_run_order() -> Vec<&'static str> {
    run_order_log().lock().unwrap().clone()
}

#[test]
fn runtime_honors_in_set_before_and_after_ordering() {
    fn run_before_set() {
        push_run_order("before");
    }

    fn run_in_set() {
        push_run_order("in_set");
    }

    fn run_after_set() {
        push_run_order("after");
    }

    clear_run_order();
    let mut world = World::new();
    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, run_in_set.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(&mut world, run_before_set.before(GameplaySet));
    runtime.add_systems::<Update, _, _>(&mut world, run_after_set.after(GameplaySet));

    let plan = runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(plan.stages.len(), 3);
    assert_eq!(plan.stages[0].system_indices.len(), 1);
    assert_eq!(plan.stages[1].system_indices.len(), 1);
    assert_eq!(plan.stages[2].system_indices.len(), 1);

    runtime.run_schedule::<Update>(&mut world).unwrap();
    assert_eq!(snapshot_run_order(), vec!["before", "in_set", "after"]);
}

#[test]
fn scheduler_event_conflict_matrix_includes_write_write() {
    fn read_a(_events: EventReader<DamageEvent>) {}
    fn read_b(_events: EventReader<DamageEvent>) {}
    fn write_a(_events: EventWriter<DamageEvent>) {}
    fn write_b(_events: EventWriter<DamageEvent>) {}

    let mut world = World::new();

    let mut read_runtime = Runtime::new();
    read_runtime.add_systems::<Update, _, _>(&mut world, (read_a, read_b));
    let read_plan = read_runtime.plan_for::<Update>().unwrap().clone();
    assert!(read_plan.conflicts.is_empty());
    assert_eq!(read_plan.stages.len(), 1);
    assert_eq!(read_plan.stages[0].system_indices.len(), 2);

    let mut read_write_runtime = Runtime::new();
    read_write_runtime.add_systems::<Update, _, _>(&mut world, (read_a, write_a));
    let read_write_plan = read_write_runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(read_write_plan.conflicts.len(), 1);
    assert_eq!(
        read_write_plan.conflicts[0].conflict.kind,
        ConflictKind::ReadWrite
    );
    assert!(read_write_plan.stages.len() >= 2);

    let mut write_write_runtime = Runtime::new();
    write_write_runtime.add_systems::<Update, _, _>(&mut world, (write_a, write_b));
    let write_write_plan = write_write_runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(write_write_plan.conflicts.len(), 1);
    assert_eq!(
        write_write_plan.conflicts[0].conflict.kind,
        ConflictKind::WriteWrite
    );
    assert!(write_write_plan.stages.len() >= 2);
}

#[test]
fn structural_command_systems_share_stage_and_merge_deterministically() {
    fn enqueue_first(mut commands: Commands) {
        commands.spawn(Marker(1));
    }

    fn enqueue_second(mut commands: Commands) {
        commands.spawn(Marker(2));
    }

    fn observe_stage_visibility(mut seen: ResMut<SeenCount>, mut query: Query<&Marker>) {
        seen.0 = query.iter().count() as u32;
    }

    let mut world = World::new();
    world.insert_resource(SeenCount(99));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(
        &mut world,
        (enqueue_first, enqueue_second, observe_stage_visibility),
    );

    let plan = runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(plan.conflicts.len(), 0);
    assert_eq!(plan.stages.len(), 1);
    assert_eq!(plan.stages[0].system_indices.len(), 3);

    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<SeenCount>().unwrap().0, 0);
    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![1, 2]);
}

#[test]
fn command_flush_occurs_at_stage_boundary() {
    fn enqueue_stage(mut commands: Commands) {
        commands.spawn(Marker(7));
    }

    fn observe_followup_stage(mut seen: ResMut<SeenCount>, mut query: Query<&Marker>) {
        seen.0 = query.iter().count() as u32;
    }

    let mut world = World::new();
    world.insert_resource(SeenCount(0));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, enqueue_stage.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        observe_followup_stage
            .in_set(PostGameplaySet)
            .after(GameplaySet),
    );

    let plan = runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(plan.stages.len(), 2);

    runtime.run_schedule::<Update>(&mut world).unwrap();
    assert_eq!(world.resource::<SeenCount>().unwrap().0, 1);
}

#[test]
fn closure_commands_queue_api_remains_functional() {
    let mut world = World::new();
    let mut commands = world.commands();
    commands.queue(|world| {
        world.spawn(Marker(33));
        Ok(())
    });

    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 0);
    commands.apply(&mut world).unwrap();

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![33]);
}

#[test]
fn typed_deferred_commands_apply_correctly() {
    let mut world = World::new();
    let mut commands = world.commands();
    commands.defer(SpawnMarkerDeferred(77));

    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 0);
    commands.apply(&mut world).unwrap();

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![77]);
}

#[test]
fn mixed_legacy_and_typed_commands_apply_in_deterministic_order() {
    let mut world = World::new();
    let mut commands = world.commands();
    commands.spawn(Marker(1));
    commands.defer(SpawnMarkerDeferred(2));
    commands.queue(|world| {
        world.spawn(Marker(3));
        Ok(())
    });
    commands.defer(SpawnMarkerDeferred(4));

    commands.apply(&mut world).unwrap();

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![1, 2, 3, 4]);
}

#[test]
fn batch_commands_apply_in_deterministic_insertion_order() {
    let mut world = World::new();
    let mut commands = world.commands();
    commands.batch(|batch| {
        batch.spawn(Marker(1));
        batch.defer(SpawnMarkerDeferred(2));
        batch.queue(|world| {
            world.spawn(Marker(3));
            Ok(())
        });
    });

    commands.apply(&mut world).unwrap();

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn batch_commands_do_not_mutate_before_stage_flush() {
    fn enqueue_batch(mut commands: Commands) {
        commands.batch(|batch| {
            batch.spawn(Marker(9));
        });
    }

    fn observe_same_stage(mut seen: ResMut<SeenCount>, mut query: Query<&Marker>) {
        seen.0 = query.iter().count() as u32;
    }

    let mut world = World::new();
    world.insert_resource(SeenCount(99));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, (enqueue_batch, observe_same_stage));

    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<SeenCount>().unwrap().0, 0);
    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 1);
}

#[test]
fn batch_and_non_batch_commands_share_queue_order_deterministically() {
    let mut world = World::new();
    let mut commands = world.commands();
    commands.spawn(Marker(1));
    commands.batch(|batch| {
        batch.spawn(Marker(2));
    });
    commands.queue(|world| {
        world.spawn(Marker(3));
        Ok(())
    });
    commands.batch(|batch| {
        batch.defer(SpawnMarkerDeferred(4));
    });

    commands.apply(&mut world).unwrap();

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![1, 2, 3, 4]);
}

#[test]
fn batch_supports_mixed_command_kinds() {
    let mut world = World::new();
    let entity = world.spawn(Marker(1));
    let mut commands = world.commands();
    commands.batch(|batch| {
        batch.queue(move |world| {
            world.insert(entity, Extra(5))?;
            Ok(())
        });
        batch.defer(InsertExtraDeferred { entity, value: 6 });
        batch.remove::<Extra>(entity);
    });

    commands.apply(&mut world).unwrap();
    assert!(world.get::<Extra>(entity).is_none());
}

#[test]
fn batch_stops_on_first_error_and_keeps_earlier_mutations() {
    let mut world = World::new();
    let target = world.spawn(Marker(0));
    let mut commands = world.commands();
    commands.batch(|batch| {
        batch.spawn(Marker(10));
        batch.remove::<Extra>(target);
        batch.spawn(Marker(11));
    });

    let result = commands.apply(&mut world);
    assert!(matches!(
        result,
        Err(ecs::CommandError::Entity(
            ecs::EntityError::MissingComponent { .. }
        ))
    ));

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![0, 10]);
}

#[test]
fn multiple_batches_in_one_stage_keep_deterministic_system_order() {
    fn enqueue_batch_a(mut commands: Commands) {
        commands.batch(|batch| {
            batch.spawn(Marker(1));
            batch.spawn(Marker(2));
        });
    }

    fn enqueue_batch_b(mut commands: Commands) {
        commands.batch(|batch| {
            batch.spawn(Marker(3));
        });
    }

    let mut world = World::new();
    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, (enqueue_batch_a, enqueue_batch_b));

    runtime.run_schedule::<Update>(&mut world).unwrap();

    let values: Vec<_> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn typed_commands_do_not_mutate_before_stage_flush() {
    fn enqueue_typed(mut commands: Commands) {
        commands.defer(SpawnMarkerDeferred(9));
    }

    fn observe_same_stage(mut seen: ResMut<SeenCount>, mut query: Query<&Marker>) {
        seen.0 = query.iter().count() as u32;
    }

    let mut world = World::new();
    world.insert_resource(SeenCount(99));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, (enqueue_typed, observe_same_stage));

    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<SeenCount>().unwrap().0, 0);
    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 1);
}

#[test]
fn typed_commands_follow_stage_boundary_visibility_contract() {
    fn enqueue_stage_typed(target: Res<TargetEntity>, mut commands: Commands) {
        commands.defer(InsertExtraDeferred {
            entity: target.0,
            value: 17,
        });
    }

    fn observe_followup_stage(mut seen: ResMut<SeenCount>, mut query: Query<&Extra>) {
        seen.0 = query.iter().count() as u32;
    }

    let mut world = World::new();
    let target = world.spawn(Marker(1));
    world.insert_resource(TargetEntity(target));
    world.insert_resource(SeenCount(0));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, enqueue_stage_typed.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        observe_followup_stage
            .in_set(PostGameplaySet)
            .after(GameplaySet),
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<SeenCount>().unwrap().0, 1);
    assert_eq!(world.require::<Extra>(target).unwrap().0, 17);
}

static NEXT_MARKER_ID: AtomicU32 = AtomicU32::new(0);

#[test]
fn borrowed_command_owner_is_stable_across_repeated_runs() {
    fn enqueue_a(mut commands: Commands) {
        let id = NEXT_MARKER_ID.fetch_add(1, Ordering::SeqCst);
        commands.spawn(Marker(id));
    }

    fn enqueue_b(mut commands: Commands) {
        let id = NEXT_MARKER_ID.fetch_add(1, Ordering::SeqCst);
        commands.spawn(Marker(id));
    }

    NEXT_MARKER_ID.store(0, Ordering::SeqCst);

    let mut world = World::new();
    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, (enqueue_a, enqueue_b));

    for _ in 0..20 {
        runtime.run_schedule::<Update>(&mut world).unwrap();
    }

    let mut ids: Vec<u32> = world
        .query_state::<&Marker, ()>()
        .iter(&world)
        .map(|marker| marker.0)
        .collect();
    ids.sort_unstable();
    assert_eq!(ids.len(), 40);
    assert_eq!(ids, (0..40).collect::<Vec<_>>());
}

#[test]
fn failed_schedule_drops_stage_deferred_commands_instead_of_replaying_next_run() {
    fn enqueue_then_fail_once(
        mut gate: ResMut<SpawnGate>,
        mut commands: Commands,
    ) -> anyhow::Result<()> {
        if gate.0 {
            return Ok(());
        }
        commands.spawn(Marker(99));
        gate.0 = true;
        anyhow::bail!("intentional failure");
    }

    let mut world = World::new();
    world.insert_resource(SpawnGate(false));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, enqueue_then_fail_once);

    assert!(runtime.run_schedule::<Update>(&mut world).is_err());
    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 0);

    runtime.run_schedule::<Update>(&mut world).unwrap();
    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 0);
}

#[test]
#[should_panic(expected = "commands param escaped its system execution scope")]
fn escaped_commands_panics_after_system_scope() {
    fn stash_commands_once(mut escaped: ResMut<EscapedCommands>, commands: Commands) {
        if escaped.0.is_none() {
            escaped.0 = Some(commands);
        }
    }

    let mut world = World::new();
    world.insert_resource(EscapedCommands(None));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, stash_commands_once);
    runtime.run_schedule::<Update>(&mut world).unwrap();

    let mut escaped = world.remove_resource::<EscapedCommands>().unwrap();
    let mut commands = escaped.0.take().expect("commands should have been stashed");
    commands.spawn(Marker(1));
}

static PARAM_INIT_CALLS: AtomicUsize = AtomicUsize::new(0);
static PARAM_EXTRACT_CALLS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct StatefulParam(u32);

impl<'w> SystemParam<'w> for StatefulParam {
    type State = u32;

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        PARAM_INIT_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(0)
    }

    fn access(_state: &Self::State) -> QueryAccess {
        QueryAccess::default()
    }

    unsafe fn extract(
        state: &'w mut Self::State,
        _world: *mut World,
        _commands: *mut Commands,
    ) -> Result<Self, SystemParamError> {
        *state = state.saturating_add(1);
        PARAM_EXTRACT_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(StatefulParam(*state))
    }
}

#[test]
fn cached_system_param_state_reuse_is_stable_over_many_runs() {
    fn accumulate_state(counter: StatefulParam, mut seen: ResMut<SeenCount>) {
        seen.0 = seen.0.saturating_add(counter.0);
    }

    let init_before = PARAM_INIT_CALLS.load(Ordering::SeqCst);
    let extract_before = PARAM_EXTRACT_CALLS.load(Ordering::SeqCst);

    let mut world = World::new();
    world.insert_resource(SeenCount(0));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, accumulate_state);

    for _ in 0..5 {
        runtime.run_schedule::<Update>(&mut world).unwrap();
    }

    assert_eq!(PARAM_INIT_CALLS.load(Ordering::SeqCst) - init_before, 1);
    assert_eq!(
        PARAM_EXTRACT_CALLS.load(Ordering::SeqCst) - extract_before,
        5
    );
    assert_eq!(world.resource::<SeenCount>().unwrap().0, 15);
}

#[test]
fn flush_stage_structural_migration_is_visible_in_followup_stage() {
    fn queue_migration(mut step: ResMut<Step>, target: Res<TargetEntity>, mut commands: Commands) {
        match step.0 {
            0 => commands.insert(target.0, Extra(7)),
            1 => commands.remove::<Extra>(target.0),
            2 => commands.insert(target.0, Extra(11)),
            _ => {}
        }
        step.0 = step.0.saturating_add(1);
    }

    fn observe_marker_extra(
        mut history: ResMut<CountHistory>,
        mut query: Query<(&Marker, &Extra)>,
    ) {
        history.0.push(query.iter().count());
    }

    let mut world = World::new();
    let target = world.spawn(Marker(1));
    world.insert_resource(TargetEntity(target));
    world.insert_resource(Step(0));
    world.insert_resource(CountHistory(Vec::new()));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, queue_migration.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        observe_marker_extra
            .in_set(PostGameplaySet)
            .after(GameplaySet),
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<CountHistory>().unwrap().0, vec![1, 0, 1]);
    assert_eq!(world.require::<Extra>(target).unwrap().0, 11);
}

#[test]
fn system_order_controls_added_and_changed_visibility() {
    fn queue_spawn_once(mut gate: ResMut<SpawnGate>, mut commands: Commands) {
        if gate.0 {
            return;
        }
        commands.spawn(Marker(5));
        gate.0 = true;
    }

    fn mutate_markers(mut query: Query<&mut Marker>) {
        for marker in query.iter() {
            marker.0 = marker.0.saturating_add(1);
        }
    }

    fn observe_added_changed(
        mut added: Query<&Marker, Added<Marker>>,
        mut changed: Query<&Marker, Changed<Marker>>,
        mut history: ResMut<AddedChangedHistory>,
    ) {
        history
            .0
            .push((added.iter().count(), changed.iter().count()));
    }

    let mut world = World::new();
    world.insert_resource(SpawnGate(false));
    world.insert_resource(AddedChangedHistory(Vec::new()));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, queue_spawn_once.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        mutate_markers.in_set(PostGameplaySet).after(GameplaySet),
    );
    runtime.add_systems::<Update, _, _>(
        &mut world,
        observe_added_changed
            .in_set(LateObserveSet)
            .after(PostGameplaySet),
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(
        world.resource::<AddedChangedHistory>().unwrap().0,
        vec![(1, 1), (0, 1)]
    );
}

#[test]
fn event_heavy_mixed_workload_with_structural_churn_remains_stable() {
    fn churn_and_emit(
        mut step: ResMut<Step>,
        target: Res<TargetEntity>,
        mut commands: Commands,
        mut writer: EventWriter<DamageEvent>,
    ) {
        step.0 = step.0.saturating_add(1);
        for offset in 0..4 {
            writer.send(DamageEvent(
                step.0.saturating_mul(10).saturating_add(offset),
            ));
        }
        if step.0 % 2 == 1 {
            commands.remove::<Toggle>(target.0);
        } else {
            commands.insert(target.0, Toggle);
        }
    }

    fn observe_events_and_presence(
        reader: EventReader<DamageEvent>,
        mut query: Query<&Toggle>,
        mut events: ResMut<EventHistory>,
        mut presence: ResMut<PresenceHistory>,
    ) {
        events.0.push(reader.iter().count());
        presence.0.push(query.iter().count());
    }

    let mut world = World::new();
    world.configure_event_channel::<DamageEvent>(EventChannelConfig {
        capacity: None,
        overflow: OverflowPolicy::DropOldest,
        lifetime: EventLifetime::FrameTransient,
        tracing: EventTracingPolicy::Disabled,
    });
    let target = world.spawn((Marker(0), Toggle));
    world.insert_resource(TargetEntity(target));
    world.insert_resource(Step(0));
    world.insert_resource(EventHistory(Vec::new()));
    world.insert_resource(PresenceHistory(Vec::new()));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, churn_and_emit.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        observe_events_and_presence
            .in_set(PostGameplaySet)
            .after(GameplaySet),
    );

    for _ in 0..4 {
        runtime.run_schedule::<Update>(&mut world).unwrap();
        world.finish_event_frame();
    }

    assert_eq!(
        world.resource::<EventHistory>().unwrap().0,
        vec![4, 4, 4, 4]
    );
    assert_eq!(
        world.resource::<PresenceHistory>().unwrap().0,
        vec![0, 1, 0, 1]
    );
}

#[test]
fn deferred_commands_keep_secondary_indexes_correct_after_apply() {
    fn queue_index_updates(
        mut step: ResMut<Step>,
        target: Res<TargetEntity>,
        mut commands: Commands,
    ) {
        match step.0 {
            0 => commands.insert(target.0, IndexedName("renamed".to_string())),
            1 => commands.remove::<IndexedName>(target.0),
            2 => commands.insert(target.0, IndexedName("restored".to_string())),
            _ => {}
        }
        step.0 = step.0.saturating_add(1);
    }

    let mut world = World::new();
    world.ensure_component_index::<IndexedName, String>(|name| name.0.clone());
    let target = world.spawn(IndexedName("initial".to_string()));
    let other = world.spawn(IndexedName("other".to_string()));
    world.insert_resource(TargetEntity(target));
    world.insert_resource(Step(0));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, queue_index_updates);

    runtime.run_schedule::<Update>(&mut world).unwrap();
    assert_eq!(
        world.find_entity_by_index::<IndexedName, String>(&"renamed".to_string()),
        Some(target)
    );
    assert_eq!(
        world.find_entity_by_index::<IndexedName, String>(&"initial".to_string()),
        None
    );
    assert_eq!(
        world.find_entity_by_index::<IndexedName, String>(&"other".to_string()),
        Some(other)
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();
    assert_eq!(
        world.find_entity_by_index::<IndexedName, String>(&"renamed".to_string()),
        None
    );
    assert_eq!(
        world.find_entity_by_index::<IndexedName, String>(&"other".to_string()),
        Some(other)
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();
    assert_eq!(
        world.find_entity_by_index::<IndexedName, String>(&"restored".to_string()),
        Some(target)
    );
}

#[test]
fn event_channel_iter_new_reads_only_unseen_events_across_runs() {
    fn produce_once(mut step: ResMut<Step>, mut writer: EventWriter<DamageEvent>) {
        if step.0 == 0 {
            writer.send(DamageEvent(7));
        }
        step.0 = step.0.saturating_add(1);
    }

    fn consume_unread(mut channel: EventChannel<DamageEvent>, mut history: ResMut<EventHistory>) {
        history.0.push(channel.iter_new().count());
    }

    let mut world = World::new();
    world.insert_resource(Step(0));
    world.insert_resource(EventHistory(Vec::new()));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, produce_once.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        consume_unread.in_set(PostGameplaySet).after(GameplaySet),
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<EventHistory>().unwrap().0, vec![1, 0]);
    assert_eq!(world.event_count::<DamageEvent>(), 1);
}

#[test]
fn event_channel_iter_new_survives_drop_oldest_overflow() {
    fn emit_each_run(mut step: ResMut<Step>, mut writer: EventWriter<DamageEvent>) {
        step.0 = step.0.saturating_add(1);
        writer.send(DamageEvent(step.0));
    }

    fn consume_unread(mut channel: EventChannel<DamageEvent>, mut history: ResMut<EventHistory>) {
        history.0.push(channel.iter_new().count());
    }

    let mut world = World::new();
    world.configure_event_channel::<DamageEvent>(EventChannelConfig {
        capacity: Some(1),
        overflow: OverflowPolicy::DropOldest,
        lifetime: EventLifetime::Manual,
        tracing: EventTracingPolicy::Disabled,
    });
    world.insert_resource(Step(0));
    world.insert_resource(EventHistory(Vec::new()));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, emit_each_run.in_set(GameplaySet));
    runtime.add_systems::<Update, _, _>(
        &mut world,
        consume_unread.in_set(PostGameplaySet).after(GameplaySet),
    );

    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<EventHistory>().unwrap().0, vec![1, 1, 1]);
    assert_eq!(world.event_count::<DamageEvent>(), 1);
    assert_eq!(world.read_events::<DamageEvent>(), &[DamageEvent(3)]);
}
