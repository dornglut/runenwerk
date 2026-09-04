use ecs::{
    BroadcastReader, BroadcastWriter, Component, Query, ResMut, Resource, Runtime,
    TickBufferProvenance, TickBufferReader, TickBufferWriter, WorkQueueReader, WorkQueueWriter,
    World,
};
use scheduler::ScheduleLabel;

#[derive(Debug, Copy, Clone, Component)]
struct A(i32);

#[derive(Debug, Copy, Clone, Component)]
struct B(i32);

#[derive(Debug, Copy, Clone, Resource)]
struct ResourceA(i32);

#[derive(Debug, Copy, Clone, Resource)]
struct ResourceB(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageA(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageB(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageC(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageD(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageE(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MessageF(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct WorkA(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct WorkB(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct WorkC(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickA(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickB(i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct TickC(i32);

#[derive(Copy, Clone)]
struct C3;

impl ScheduleLabel for C3 {
    fn name() -> &'static str {
        "C3Miri"
    }
}

fn world_with_two_entities() -> World {
    let mut world = World::new();
    world.spawn((A(1), B(10))).unwrap();
    world.spawn((A(2), B(20))).unwrap();
    world
}

#[test]
fn mutable_query_items_remain_unique_across_iterator_advancement() {
    let mut world = world_with_two_entities();
    let query = world.query_state::<&mut A, ()>();
    let mut iter = query.iter(&mut world);
    let first = iter.next().unwrap();
    let second = iter.next().unwrap();
    first.0 += 10;
    second.0 += 20;
    assert!(iter.next().is_none());
}

#[test]
fn mutable_tuple_items_remain_disjoint_across_iterator_advancement() {
    let mut world = world_with_two_entities();
    let query = world.query_state::<(&mut A, &mut B), ()>();
    let mut iter = query.iter(&mut world);
    let (first_a, first_b) = iter.next().unwrap();
    let (second_a, second_b) = iter.next().unwrap();
    first_a.0 += 1;
    first_b.0 += 2;
    second_a.0 += 3;
    second_b.0 += 4;
    assert!(iter.next().is_none());
}

#[test]
fn query_and_disjoint_query_can_retain_items_together() {
    fn system(mut query_a: Query<&mut A>, mut query_b: Query<&mut B>) {
        let mut a_iter = query_a.iter();
        let first_a = a_iter.next().unwrap();
        let mut b_iter = query_b.iter();
        let first_b = b_iter.next().unwrap();
        first_a.0 += 1;
        first_b.0 += 2;
    }

    let mut world = world_with_two_entities();
    let mut runtime = Runtime::new();
    runtime.add_systems::<C3, _, _>(&mut world, system);
    runtime.run_schedule::<C3>(&mut world).unwrap();
}

#[test]
fn query_item_and_resource_mutation_can_be_live_together() {
    fn system(mut query: Query<&mut A>, mut resource: ResMut<ResourceB>) {
        let mut iter = query.iter();
        let item = iter.next().unwrap();
        drop(iter);
        resource.0 += 1;
        item.0 += resource.0;
    }

    let mut world = world_with_two_entities();
    world.insert_resource(ResourceB(4));
    let mut runtime = Runtime::new();
    runtime.add_systems::<C3, _, _>(&mut world, system);
    runtime.run_schedule::<C3>(&mut world).unwrap();
}

#[test]
fn resource_payloads_survive_other_resource_mutation_bookkeeping() {
    fn system(mut first: ResMut<ResourceA>, mut second: ResMut<ResourceB>) {
        let first_value: &mut ResourceA = &mut first;
        let second_value: &mut ResourceB = &mut second;
        second_value.0 += 1;
        first_value.0 += second_value.0;
    }

    let mut world = World::new();
    world.insert_resource(ResourceA(1));
    world.insert_resource(ResourceB(2));
    let mut runtime = Runtime::new();
    runtime.add_systems::<C3, _, _>(&mut world, system);
    runtime.run_schedule::<C3>(&mut world).unwrap();
}

#[test]
fn broadcast_reader_reference_survives_other_stream_map_growth() {
    fn system(
        reader: BroadcastReader<MessageA>,
        mut writer_b: BroadcastWriter<MessageB>,
        mut writer_c: BroadcastWriter<MessageC>,
        mut writer_d: BroadcastWriter<MessageD>,
        mut writer_e: BroadcastWriter<MessageE>,
        mut writer_f: BroadcastWriter<MessageF>,
    ) {
        let retained = reader.iter_all().next().unwrap();
        writer_b.send(MessageB(retained.0));
        writer_c.send(MessageC(retained.0));
        writer_d.send(MessageD(retained.0));
        writer_e.send(MessageE(retained.0));
        writer_f.send(MessageF(retained.0));
        assert_eq!(retained.0, 7);
    }

    let mut world = World::new();
    world.publish_broadcast(MessageA(7));
    let mut runtime = Runtime::new();
    runtime.add_systems::<C3, _, _>(&mut world, system);
    runtime.run_schedule::<C3>(&mut world).unwrap();
}

#[test]
fn work_reader_reference_survives_other_queue_map_growth() {
    fn system(
        reader: WorkQueueReader<WorkA>,
        mut writer_b: WorkQueueWriter<WorkB>,
        mut writer_c: WorkQueueWriter<WorkC>,
    ) {
        let retained = reader.peek().unwrap();
        writer_b.enqueue(WorkB(retained.0)).unwrap();
        writer_c.enqueue(WorkC(retained.0)).unwrap();
        assert_eq!(retained.0, 8);
    }

    let mut world = World::new();
    world.work_queue_enqueue(WorkA(8)).unwrap();
    let mut runtime = Runtime::new();
    runtime.add_systems::<C3, _, _>(&mut world, system);
    runtime.run_schedule::<C3>(&mut world).unwrap();
}

#[test]
fn tick_reader_reference_survives_other_buffer_map_growth() {
    fn system(
        reader: TickBufferReader<TickA>,
        mut writer_b: TickBufferWriter<TickB>,
        mut writer_c: TickBufferWriter<TickC>,
    ) {
        let retained = reader.iter_current().next().unwrap();
        writer_b
            .push_current(TickB(retained.0))
            .expect("tick B should accept the message");
        writer_c
            .push_current(TickC(retained.0))
            .expect("tick C should accept the message");
        assert_eq!(retained.0, 9);
    }

    let mut world = World::new();
    world
        .push_buffer_message_for_current_tick(TickBufferProvenance::UNSPECIFIED, TickA(9))
        .unwrap();
    let mut runtime = Runtime::new();
    runtime.add_systems::<C3, _, _>(&mut world, system);
    runtime.run_schedule::<C3>(&mut world).unwrap();
}

#[test]
fn migrated_archetype_query_still_yields_unique_mutable_items() {
    let mut world = World::new();
    let first_entity = world.spawn(A(1)).unwrap();
    let second_entity = world.spawn(A(2)).unwrap();
    world.insert(first_entity, B(10)).unwrap();
    world.insert(second_entity, B(20)).unwrap();

    let query = world.query_state::<&mut A, ()>();
    let mut iter = query.iter(&mut world);
    let first = iter.next().unwrap();
    let second = iter.next().unwrap();
    first.0 += 10;
    second.0 += 20;
    assert!(iter.next().is_none());
}
