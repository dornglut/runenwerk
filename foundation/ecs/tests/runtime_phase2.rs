use ecs::prelude::*;
use ecs::{QueryAccess, SystemParam, SystemParamError};
use scheduler::ScheduleLabel;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy, Clone)]
struct Update;

impl ScheduleLabel for Update {
    fn name() -> &'static str {
        "Update"
    }
}

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Position(f32);

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Velocity(f32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct Frame(u64);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct Score(u64);

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct DeltaTime(f32);

#[derive(Debug, Copy, Clone, PartialEq, ecs::Component)]
struct Bonus(f32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct Marker;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct SeenCount(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, ecs::Component)]
struct MissingRes;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct DamageEvent(u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct SpawnEvent;

#[test]
fn runtime_executes_1_2_and_8_param_systems() {
    fn bump_frame(mut frame: ResMut<Frame>) {
        frame.0 += 1;
    }

    fn integrate_positions(mut query: Query<&mut Position>, dt: Res<DeltaTime>) {
        for position in query.iter() {
            position.0 += dt.0;
        }
    }

    fn full_tick(
        mut query: Query<(&mut Position, Option<&Velocity>)>,
        dt: Res<DeltaTime>,
        mut frame: ResMut<Frame>,
        mut score: ResMut<Score>,
        mut commands: Commands,
        reader: EventReader<DamageEvent>,
        mut writer: EventWriter<SpawnEvent>,
        bonus: Res<Bonus>,
    ) {
        for (position, velocity) in query.iter() {
            position.0 += dt.0 + bonus.0 + velocity.map_or(0.0, |v| v.0);
        }
        frame.0 += 10;
        score.0 += reader.iter().map(|event| event.0 as u64).sum::<u64>();
        writer.send(SpawnEvent);
        commands.spawn(Marker);
    }

    let mut world = World::new();
    world.spawn((Position(1.0), Velocity(2.0)));
    world.spawn(Position(2.0));
    world.insert_resource(Frame(0));
    world.insert_resource(Score(0));
    world.insert_resource(DeltaTime(0.5));
    world.insert_resource(Bonus(1.0));
    world.emit_event(DamageEvent(3));
    world.emit_event(DamageEvent(4));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, (bump_frame, integrate_positions, full_tick));
    runtime.run_schedule::<Update>(&mut world).unwrap();

    let positions: Vec<_> = world
        .query_state::<&Position, ()>()
        .iter(&world)
        .map(|position| position.0)
        .collect();
    assert_eq!(positions, vec![5.0, 4.0]);
    assert_eq!(world.resource::<Frame>().unwrap().0, 11);
    assert_eq!(world.resource::<Score>().unwrap().0, 7);
    assert_eq!(world.event_count::<SpawnEvent>(), 1);
    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 1);
}

static INIT_STATE_CALLS: AtomicUsize = AtomicUsize::new(0);
static EXTRACT_CALLS: AtomicUsize = AtomicUsize::new(0);

struct CachedCounter(usize);

impl<'w> SystemParam<'w> for CachedCounter {
    type State = usize;

    fn init_state(_world: &mut World) -> Result<Self::State, SystemParamError> {
        INIT_STATE_CALLS.fetch_add(1, Ordering::SeqCst);
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
        *state += 1;
        EXTRACT_CALLS.fetch_add(1, Ordering::SeqCst);
        Ok(CachedCounter(*state))
    }
}

#[test]
fn runtime_caches_system_param_state_across_runs() {
    fn cached(counter: CachedCounter, mut frame: ResMut<Frame>) {
        frame.0 += counter.0 as u64;
    }

    let init_before = INIT_STATE_CALLS.load(Ordering::SeqCst);
    let extract_before = EXTRACT_CALLS.load(Ordering::SeqCst);

    let mut world = World::new();
    world.insert_resource(Frame(0));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, cached);
    runtime.run_schedule::<Update>(&mut world).unwrap();
    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(
        INIT_STATE_CALLS.load(Ordering::SeqCst) - init_before,
        1,
        "state should initialize exactly once per registered system",
    );
    assert_eq!(
        EXTRACT_CALLS.load(Ordering::SeqCst) - extract_before,
        2,
        "state should extract once per run",
    );
    assert_eq!(world.resource::<Frame>().unwrap().0, 3);
}

#[test]
fn runtime_reports_extraction_errors_cleanly() {
    fn requires_missing_resource(_missing: Res<MissingRes>) {}

    let mut world = World::new();
    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, requires_missing_resource);

    let err = runtime.run_schedule::<Update>(&mut world).unwrap_err();
    let message = format!("{err:#}");
    assert!(message.contains("runtime setup failed"), "{message}");
    assert!(message.contains("does not exist"), "{message}");
}

#[test]
fn scheduler_conflict_model_respects_reads_writes_and_events() {
    fn read_frame_a(_frame: Res<Frame>) {}
    fn read_frame_b(_frame: Res<Frame>) {}
    fn write_frame(_frame: ResMut<Frame>) {}
    fn read_events_a(_events: EventReader<DamageEvent>) {}
    fn read_events_b(_events: EventReader<DamageEvent>) {}
    fn write_events(_events: EventWriter<DamageEvent>) {}

    let mut world = World::new();
    world.insert_resource(Frame(0));

    let mut read_runtime = Runtime::new();
    read_runtime.add_systems::<Update, _, _>(&mut world, (read_frame_a, read_frame_b));
    let read_plan = read_runtime.plan_for::<Update>().unwrap().clone();
    assert!(read_plan.conflicts.is_empty());
    assert_eq!(read_plan.stages.len(), 1);
    assert_eq!(read_plan.stages[0].system_indices.len(), 2);

    let mut read_write_runtime = Runtime::new();
    read_write_runtime.add_systems::<Update, _, _>(&mut world, (read_frame_a, write_frame));
    let read_write_plan = read_write_runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(read_write_plan.conflicts.len(), 1);
    assert!(read_write_plan.stages.len() >= 2);

    let mut event_read_runtime = Runtime::new();
    event_read_runtime.add_systems::<Update, _, _>(&mut world, (read_events_a, read_events_b));
    let event_read_plan = event_read_runtime.plan_for::<Update>().unwrap().clone();
    assert!(event_read_plan.conflicts.is_empty());
    assert_eq!(event_read_plan.stages.len(), 1);
    assert_eq!(event_read_plan.stages[0].system_indices.len(), 2);

    let mut event_write_runtime = Runtime::new();
    event_write_runtime.add_systems::<Update, _, _>(&mut world, (read_events_a, write_events));
    let event_write_plan = event_write_runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(event_write_plan.conflicts.len(), 1);
}

#[test]
fn commands_flush_at_stage_end_not_between_systems_in_same_stage() {
    fn enqueue_spawn(mut commands: Commands) {
        commands.spawn(Marker);
    }

    fn observe_marker_count(mut seen: ResMut<SeenCount>, mut query: Query<&Marker>) {
        seen.0 = query.iter().count() as u32;
    }

    let mut world = World::new();
    world.insert_resource(SeenCount(99));

    let mut runtime = Runtime::new();
    runtime.add_systems::<Update, _, _>(&mut world, (enqueue_spawn, observe_marker_count));
    let plan = runtime.plan_for::<Update>().unwrap().clone();
    assert_eq!(plan.stages.len(), 1);
    assert_eq!(plan.stages[0].system_indices.len(), 2);

    runtime.run_schedule::<Update>(&mut world).unwrap();

    assert_eq!(world.resource::<SeenCount>().unwrap().0, 0);
    assert_eq!(world.query_state::<&Marker, ()>().iter(&world).count(), 1);
}
