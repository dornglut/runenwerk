use ecs::{
    BroadcastKey, TickBufferKey, TickBufferProvenance, TickBufferPushError, WorkQueueKey, World,
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
