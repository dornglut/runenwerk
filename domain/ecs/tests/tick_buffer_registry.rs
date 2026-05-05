use ecs::{
    BroadcastKey, TickBufferConfig, TickBufferKey, TickBufferProvenance, TickBufferPushError,
    WorkQueueKey, World,
};

#[test]
fn tick_buffer_sequence_is_global_and_monotonic_across_ticks() {
    let mut world = World::new();

    let first = world
        .push_buffer_message_for_tick(5, TickBufferProvenance::UNSPECIFIED, 1_u32)
        .expect("first input should enqueue");
    let second = world
        .push_buffer_message_for_tick(5, TickBufferProvenance::UNSPECIFIED, 2_u32)
        .expect("second input should enqueue");
    let third = world
        .push_buffer_message_for_tick(6, TickBufferProvenance::UNSPECIFIED, 3_u32)
        .expect("third input should enqueue");

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    assert_eq!(third.sequence, 3);
    assert_eq!(first.buffer_key, second.buffer_key);
    assert_eq!(second.buffer_key, third.buffer_key);

    world.finalize_tick_boundary(5);
    let fourth = world
        .push_buffer_message_for_tick(6, TickBufferProvenance::UNSPECIFIED, 4_u32)
        .expect("sequence should continue after tick finalization");
    assert_eq!(fourth.sequence, 4);
}

#[test]
fn finalized_tick_late_writes_are_rejected_and_do_not_survive_repeat_finalization() {
    let mut world = World::new();

    world
        .push_buffer_message_for_tick(10, TickBufferProvenance::UNSPECIFIED, 1_u32)
        .expect("initial input should enqueue");
    world.finalize_tick_boundary(10);

    let late = world.push_buffer_message_for_tick(10, TickBufferProvenance::UNSPECIFIED, 2_u32);
    assert!(matches!(
        late,
        Err(TickBufferPushError::FinalizedTick {
            tick: 10,
            finalized_tick: 10,
            ..
        })
    ));

    world.finalize_tick_boundary(10);
    assert!(world.buffer_messages_at_tick::<u32>(10).is_empty());

    let stats = world
        .buffer_stats::<u32>()
        .expect("buffer stats should exist after rejected late write");
    assert_eq!(stats.pushed, 1);
    assert_eq!(stats.rejected, 1);
    assert_eq!(stats.pending_messages, 0);
}

#[test]
fn older_than_finalized_tick_is_rejected_and_future_tick_still_accepts_messages() {
    let mut world = World::new();

    world.finalize_tick_boundary(10);

    let older = world.push_buffer_message_for_tick(9, TickBufferProvenance::UNSPECIFIED, 1_u32);
    assert!(matches!(
        older,
        Err(TickBufferPushError::FinalizedTick {
            tick: 9,
            finalized_tick: 10,
            ..
        })
    ));

    let future = world
        .push_buffer_message_for_tick(11, TickBufferProvenance::UNSPECIFIED, 2_u32)
        .expect("future tick should stay open");
    assert_eq!(future.tick, 11);
    assert!(world.buffer_messages_at_tick::<u32>(9).is_empty());
    assert_eq!(world.buffer_messages_at_tick::<u32>(11), &[2]);
}

#[test]
fn tick_finalization_is_monotonic_and_gap_finalization_purges_all_closed_ticks() {
    let mut world = World::new();

    world
        .push_buffer_message_for_tick(3, TickBufferProvenance::UNSPECIFIED, 3_u32)
        .unwrap();
    world
        .push_buffer_message_for_tick(5, TickBufferProvenance::UNSPECIFIED, 5_u32)
        .unwrap();
    world
        .push_buffer_message_for_tick(7, TickBufferProvenance::UNSPECIFIED, 7_u32)
        .unwrap();

    world.finalize_tick_boundary(5);
    world.finalize_tick_boundary(5);
    world.finalize_tick_boundary(4);

    assert!(world.buffer_messages_at_tick::<u32>(3).is_empty());
    assert!(world.buffer_messages_at_tick::<u32>(5).is_empty());
    assert_eq!(world.buffer_messages_at_tick::<u32>(7), &[7]);
    assert_eq!(world.current_buffer_tick(), 5);
    assert_eq!(world.messaging_finalization_counters().tick_boundaries, 1);

    world.finalize_tick_boundary(7);
    assert!(world.buffer_messages_at_tick::<u32>(7).is_empty());
    assert_eq!(world.messaging_finalization_counters().tick_boundaries, 2);
}

#[test]
fn finalized_ticks_are_closed_even_when_retained_for_inspection() {
    let mut world = World::new();
    world.configure_tick_buffer::<u32>(TickBufferConfig {
        capacity: None,
        retain_finalized_ticks: true,
    });

    world
        .push_buffer_message_for_tick(4, TickBufferProvenance::UNSPECIFIED, 4_u32)
        .unwrap();
    world.finalize_tick_boundary(4);
    assert_eq!(world.buffer_messages_at_tick::<u32>(4), &[4]);

    let late = world.push_buffer_message_for_tick(4, TickBufferProvenance::UNSPECIFIED, 40_u32);
    assert!(matches!(
        late,
        Err(TickBufferPushError::FinalizedTick {
            tick: 4,
            finalized_tick: 4,
            ..
        })
    ));
    assert_eq!(world.buffer_messages_at_tick::<u32>(4), &[4]);
}

#[test]
fn tick_buffer_dedup_hook_drops_duplicates_with_diagnostics() {
    let mut world = World::new();
    world.set_tick_buffer_dedup_hook::<u32, _>(|left, right| left == right);

    world
        .push_buffer_message_for_tick(11, TickBufferProvenance::UNSPECIFIED, 7_u32)
        .expect("first value should enqueue");
    let duplicate =
        world.push_buffer_message_for_tick(11, TickBufferProvenance::UNSPECIFIED, 7_u32);

    assert!(matches!(
        duplicate,
        Err(TickBufferPushError::Deduplicated { .. })
    ));

    let stats = world
        .buffer_stats::<u32>()
        .expect("stream stats should exist");
    assert_eq!(stats.pushed, 1);
    assert_eq!(stats.dropped, 1);
}

#[test]
fn messaging_diagnostics_snapshot_exposes_stable_keys() {
    let mut world = World::new();

    world.ensure_work_queue::<u8>();
    world.ensure_broadcast_stream::<u16>();
    world
        .push_buffer_message_for_tick(1, TickBufferProvenance::UNSPECIFIED, 99_u32)
        .expect("input stream should accept message");

    let diagnostics = world.messaging_diagnostics_snapshot();

    assert!(
        diagnostics
            .work_queues
            .iter()
            .any(|queue| queue.key == WorkQueueKey(1) && queue.work_queue_type.ends_with("u8"))
    );
    assert!(diagnostics.broadcasts.iter().any(|broadcast| {
        broadcast.key == BroadcastKey(1) && broadcast.stream_type.ends_with("u16")
    }));
    assert!(
        diagnostics.tick_buffers.iter().any(|buffer| {
            buffer.key == TickBufferKey(1) && buffer.buffer_type.ends_with("u32")
        })
    );
}
