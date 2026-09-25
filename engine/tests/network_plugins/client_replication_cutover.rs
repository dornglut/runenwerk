// Owner: Engine Networking Tests - RunenNet Client Replication Cutover

fn client_test_snapshot(label: &str) -> TestSnapshot {
    TestSnapshot {
        context: TestSnapshotContext {
            world_scene_label: label.to_string(),
        },
    }
}

fn client_full_message(cursor: u64, tick: u64, label: &str) -> ServerMessage {
    let snapshot = client_test_snapshot(label);
    ServerMessage::Snapshot(Snapshot {
        tick: SimulationTick(tick),
        cursor: SnapshotCursor(cursor),
        last_applied: SnapshotCursor::default(),
        payload: TestReplicationDriver::encode_snapshot(&snapshot)
            .expect("test full snapshot should encode"),
    })
}

fn client_delta_message(base: u64, cursor: u64, tick: u64, changed: bool) -> ServerMessage {
    ServerMessage::DeltaSnapshot(DeltaSnapshot {
        tick: SimulationTick(tick),
        base: SnapshotCursor(base),
        cursor: SnapshotCursor(cursor),
        payload: TestReplicationDriver::encode_delta(&TestDelta { changed })
            .expect("test delta should encode"),
    })
}

fn clear_client_outbound(world: &mut World) {
    world
        .resource_mut::<NetworkOutboundQueue>()
        .expect("client outbound queue should exist")
        .clear_client_messages();
}

fn outbound_ack(world: &World) -> Option<Ack> {
    world
        .resource::<NetworkOutboundQueue>()
        .expect("client outbound queue should exist")
        .client_messages()
        .iter()
        .find_map(|message| match message {
            ClientMessage::Ack(ack) => Some(ack.clone()),
            _ => None,
        })
}

fn active_client_snapshot(world: &World) -> Option<TestSnapshot> {
    let active = world
        .resource::<ActiveClientReplicatedStateProduct>()
        .ok()?
        .active()?;
    TestReplicationDriver::decode_snapshot(active.bytes()).ok()
}

fn client_app_with_policy(policy: ClientReplicationPolicy) -> App {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.init_resource::<PlayerCommandBuffer>();
    app.add_plugin(SimulationPlugin);
    app.add_plugin(
        NetPlugin::<TestReplicationDriver>::new(NetRole::Client).with_config(
            NetPluginConfig::default().with_client_replication_policy(policy),
        ),
    );
    app
}

#[test]
fn client_replication_requires_explicit_policy_for_snapshot_messages() {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.init_resource::<PlayerCommandBuffer>();
    app.add_plugin(SimulationPlugin);
    app.add_plugin(NetPlugin::<TestReplicationDriver>::new(NetRole::Client));

    enqueue_client_inbox(app.world_mut(), client_full_message(1, 1, "one"))
        .expect("snapshot should stage");
    let app = app
        .run_for_frames(1)
        .expect("missing replication policy should fail closed without breaking the frame");

    assert_eq!(client_replication_acknowledgement(app.world()), None);
    assert!(app.world().resource::<ActiveClientReplicatedStateProduct>().is_err());
    assert_eq!(outbound_ack(app.world()), None);
}

#[test]
fn client_replication_duplicate_reacks_but_stale_and_tick_regression_do_not_commit() {
    let mut app = client_app_with_policy(test_client_replication_policy());

    enqueue_client_inbox(app.world_mut(), client_full_message(2, 2, "two"))
        .expect("initial snapshot should stage");
    let mut app = app.run_for_frames(1).expect("initial snapshot should commit");
    assert_eq!(
        client_replication_acknowledgement(app.world()),
        Some((SnapshotCursor(2), SimulationTick(2)))
    );
    assert_eq!(
        active_client_snapshot(app.world())
            .expect("active product should decode")
            .context
            .world_scene_label,
        "two"
    );

    clear_client_outbound(app.world_mut());
    enqueue_client_inbox(app.world_mut(), client_full_message(2, 2, "ignored-duplicate"))
        .expect("duplicate snapshot should stage");
    app = app
        .run_for_frames(1)
        .expect("duplicate-current snapshot should re-realize");
    assert_eq!(
        outbound_ack(app.world()).map(|ack| ack.cursor),
        Some(SnapshotCursor(2)),
        "duplicate current may re-ACK the RunenNet committed cursor"
    );
    assert_eq!(
        active_client_snapshot(app.world())
            .expect("duplicate must keep committed product")
            .context
            .world_scene_label,
        "two",
        "duplicate payload must not replace the committed product"
    );

    clear_client_outbound(app.world_mut());
    enqueue_client_inbox(app.world_mut(), client_full_message(1, 3, "stale"))
        .expect("stale snapshot should stage");
    app = app.run_for_frames(1).expect("stale snapshot should classify");
    assert_eq!(outbound_ack(app.world()), None);
    assert_eq!(
        active_client_snapshot(app.world())
            .expect("stale target must preserve active product")
            .context
            .world_scene_label,
        "two"
    );

    clear_client_outbound(app.world_mut());
    enqueue_client_inbox(app.world_mut(), client_full_message(3, 1, "regressed"))
        .expect("tick-regressed snapshot should stage");
    app = app
        .run_for_frames(1)
        .expect("tick regression should classify");
    assert_eq!(outbound_ack(app.world()), None);
    assert_eq!(
        client_replication_acknowledgement(app.world()),
        Some((SnapshotCursor(2), SimulationTick(2)))
    );
}

#[test]
fn client_delta_reconstructs_from_exact_runennet_retained_declared_base() {
    let mut app = client_app_with_policy(test_client_replication_policy());

    for message in [
        client_full_message(1, 1, "base-one"),
        client_full_message(2, 2, "current-two"),
    ] {
        enqueue_client_inbox(app.world_mut(), message).expect("full snapshot should stage");
        app = app.run_for_frames(1).expect("full snapshot should commit");
        clear_client_outbound(app.world_mut());
    }

    enqueue_client_inbox(app.world_mut(), client_delta_message(1, 3, 3, false))
        .expect("historical-base delta should stage");
    let app = app
        .run_for_frames(1)
        .expect("historical-base delta should reconstruct");

    assert_eq!(
        client_replication_acknowledgement(app.world()),
        Some((SnapshotCursor(3), SimulationTick(3)))
    );
    assert_eq!(
        active_client_snapshot(app.world())
            .expect("reconstructed active product should decode")
            .context
            .world_scene_label,
        "base-one",
        "delta reconstruction must use the exact declared retained base, not the prior current product"
    );
}

#[test]
fn client_delta_missing_base_and_malformed_payload_enter_runennet_recovery_without_host_mutation() {
    let mut missing = client_app_with_policy(test_client_replication_policy());
    enqueue_client_inbox(missing.world_mut(), client_full_message(1, 1, "baseline"))
        .expect("baseline should stage");
    let mut missing = missing.run_for_frames(1).expect("baseline should commit");
    clear_client_outbound(missing.world_mut());
    let before = active_client_snapshot(missing.world()).expect("baseline should be active");

    enqueue_client_inbox(missing.world_mut(), client_delta_message(99, 100, 2, false))
        .expect("missing-base delta should stage");
    let missing = missing
        .run_for_frames(1)
        .expect("missing-base delta should classify");
    assert_eq!(outbound_ack(missing.world()), None);
    assert_eq!(active_client_snapshot(missing.world()), Some(before));
    assert_eq!(
        client_replication_state(missing.world()),
        Some(ClientReplicationState::FullSnapshotRequired(
            ClientRecoveryReason::MissingBase
        ))
    );

    let mut malformed = client_app_with_policy(test_client_replication_policy());
    enqueue_client_inbox(malformed.world_mut(), client_full_message(1, 1, "baseline"))
        .expect("baseline should stage");
    let mut malformed = malformed.run_for_frames(1).expect("baseline should commit");
    clear_client_outbound(malformed.world_mut());
    let before = active_client_snapshot(malformed.world()).expect("baseline should be active");
    enqueue_client_inbox(
        malformed.world_mut(),
        ServerMessage::DeltaSnapshot(DeltaSnapshot {
            tick: SimulationTick(2),
            base: SnapshotCursor(1),
            cursor: SnapshotCursor(2),
            payload: Vec::new(),
        }),
    )
    .expect("malformed delta should stage");
    let malformed = malformed
        .run_for_frames(1)
        .expect("malformed delta should classify");
    assert_eq!(outbound_ack(malformed.world()), None);
    assert_eq!(active_client_snapshot(malformed.world()), Some(before));
    assert_eq!(
        client_replication_state(malformed.world()),
        Some(ClientReplicationState::FullSnapshotRequired(
            ClientRecoveryReason::MalformedDelta
        ))
    );
}

#[test]
fn client_replication_resource_limit_rejection_preserves_empty_active_product() {
    let policy = test_client_replication_policy_with_state_limit(1);
    let mut app = client_app_with_policy(policy);
    enqueue_client_inbox(app.world_mut(), client_full_message(1, 1, "larger-than-one-byte"))
        .expect("oversized snapshot should stage");

    let app = app
        .run_for_frames(1)
        .expect("resource rejection should fail closed");

    assert_eq!(client_replication_acknowledgement(app.world()), None);
    assert!(
        app.world()
            .resource::<ActiveClientReplicatedStateProduct>()
            .expect("active product owner should exist")
            .active()
            .is_none()
    );
    assert_eq!(outbound_ack(app.world()), None);
}

#[test]
fn client_replication_connection_replacement_preserves_lineage_and_requires_full() {
    let mut app = client_app_with_policy(test_client_replication_policy());
    enqueue_client_inbox(app.world_mut(), client_full_message(1, 1, "baseline"))
        .expect("baseline should stage");
    let mut app = app.run_for_frames(1).expect("baseline should commit");

    let lineage = client_replication_lineage(app.world()).expect("configured lineage should exist");
    require_client_replication_connection_replacement(app.world_mut())
        .expect("replacement should enter RunenNet recovery");

    assert_eq!(client_replication_lineage(app.world()), Some(lineage));
    assert_eq!(
        client_replication_state(app.world()),
        Some(ClientReplicationState::FullSnapshotRequired(
            ClientRecoveryReason::ConnectionReplacement
        ))
    );
}

#[test]
fn client_realization_failure_keeps_runennet_commit_and_duplicate_retries_without_recommit() {
    let mut app = client_app_with_policy(test_client_replication_policy());
    app.world_mut().insert_resource(RejectSnapshotRealization(true));
    let message = client_full_message(1, 1, "committed-but-not-realized");
    enqueue_client_inbox(app.world_mut(), message.clone()).expect("snapshot should stage");

    let mut app = app
        .run_for_frames(1)
        .expect("realization failure should be contained by client receive");
    assert_eq!(
        client_replication_acknowledgement(app.world()),
        Some((SnapshotCursor(1), SimulationTick(1))),
        "RunenNet protocol commit must survive downstream realization failure"
    );
    assert_eq!(outbound_ack(app.world()), None, "failed realization must not ACK");

    app.world_mut()
        .resource_mut::<RejectSnapshotRealization>()
        .expect("realization switch should exist")
        .0 = false;
    clear_client_outbound(app.world_mut());
    enqueue_client_inbox(app.world_mut(), message).expect("duplicate should stage");
    let app = app
        .run_for_frames(1)
        .expect("duplicate-current should retry downstream realization");

    assert_eq!(
        outbound_ack(app.world()).map(|ack| ack.cursor),
        Some(SnapshotCursor(1))
    );
    assert_eq!(
        active_client_snapshot(app.world())
            .expect("committed product should remain active")
            .context
            .world_scene_label,
        "committed-but-not-realized"
    );
}


struct ReconstructionFailureDriver;

impl ReplicationDriver for ReconstructionFailureDriver {
    type Snapshot = TestSnapshot;
    type Delta = TestDelta;
    type Input = ClientCommandEnvelope;
    type Error = io::Error;

    fn capture_snapshot(_world: &World) -> Result<Option<Self::Snapshot>, Self::Error> {
        Ok(Some(TestSnapshot::default()))
    }

    fn build_delta(previous: &Self::Snapshot, current: &Self::Snapshot) -> Self::Delta {
        TestDelta {
            changed: previous != current,
        }
    }

    fn apply_delta_to_snapshot(base: &Self::Snapshot, delta: &Self::Delta) -> Self::Snapshot {
        if delta.changed {
            TestSnapshot::default()
        } else {
            base.clone()
        }
    }

    fn encode_snapshot(snapshot: &Self::Snapshot) -> Result<Vec<u8>, Self::Error> {
        if snapshot.context.world_scene_label == "gameplay_stub" {
            return Err(io::Error::other(
                "test reconstructed snapshot encoding rejected",
            ));
        }
        postcard::to_allocvec(snapshot).map_err(Self::map_codec_error)
    }

    fn map_codec_error(error: postcard::Error) -> Self::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}

impl SnapshotApplyDriver for ReconstructionFailureDriver {
    fn apply_snapshot(
        _world: &mut World,
        _tick: engine_sim::SimulationTick,
        _snapshot: Self::Snapshot,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

impl InputDriver for ReconstructionFailureDriver {
    fn receive_remote_input(
        _world: &mut World,
        _connection: ConnectionHandle,
        _tick: engine_sim::SimulationTick,
        _input: Vec<Self::Input>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn take_local_input(_world: &mut World) -> Result<Vec<Self::Input>, Self::Error> {
        Ok(Vec::new())
    }

    fn apply_input(
        _world: &mut World,
        _tick: engine_sim::SimulationTick,
        _input: &[Self::Input],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn client_app_with_reconstruction_failure_driver(policy: ClientReplicationPolicy) -> App {
    let mut app = App::headless();
    app.add_plugins(default_plugins());
    app.add_plugin(SimulationPlugin);
    app.add_plugin(
        NetPlugin::<ReconstructionFailureDriver>::new(NetRole::Client).with_config(
            NetPluginConfig::default().with_client_replication_policy(policy),
        ),
    );
    app
}

#[test]
fn malformed_full_snapshot_is_rejected_before_runennet_commit() {
    let mut app = client_app_with_policy(test_client_replication_policy());
    enqueue_client_inbox(
        app.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(1),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload: Vec::new(),
        }),
    )
    .expect("malformed full snapshot should stage");

    let app = app
        .run_for_frames(1)
        .expect("malformed full snapshot should fail closed");

    assert_eq!(client_replication_acknowledgement(app.world()), None);
    assert!(
        app.world()
            .resource::<ActiveClientReplicatedStateProduct>()
            .expect("active product owner should exist")
            .active()
            .is_none()
    );
    assert_eq!(outbound_ack(app.world()), None);
}

#[test]
fn client_delta_reconstruction_failure_enters_runennet_recovery_without_host_mutation() {
    let mut app =
        client_app_with_reconstruction_failure_driver(test_client_replication_policy());
    enqueue_client_inbox(app.world_mut(), client_full_message(1, 1, "baseline"))
        .expect("baseline should stage");
    let mut app = app.run_for_frames(1).expect("baseline should commit");
    clear_client_outbound(app.world_mut());
    let before = active_client_snapshot(app.world()).expect("baseline should be active");

    enqueue_client_inbox(app.world_mut(), client_delta_message(1, 2, 2, true))
        .expect("reconstruction-failure delta should stage");
    let app = app
        .run_for_frames(1)
        .expect("reconstruction failure should classify");

    assert_eq!(outbound_ack(app.world()), None);
    assert_eq!(active_client_snapshot(app.world()), Some(before));
    assert_eq!(
        client_replication_state(app.world()),
        Some(ClientReplicationState::FullSnapshotRequired(
            ClientRecoveryReason::ReconstructionFailure
        ))
    );
}

#[test]
fn client_replication_aggregate_retained_bytes_rejection_preserves_current_product() {
    let first = client_test_snapshot("aggregate-first");
    let second = client_test_snapshot("aggregate-second");
    let first_bytes = TestReplicationDriver::encode_snapshot(&first)
        .expect("first aggregate snapshot should encode")
        .len();
    let second_bytes = TestReplicationDriver::encode_snapshot(&second)
        .expect("second aggregate snapshot should encode")
        .len();
    let aggregate_budget = first_bytes
        .checked_add(second_bytes)
        .and_then(|total| total.checked_sub(1))
        .expect("aggregate test budget should be representable");

    let retention = ReplicationRetentionLimits::new(
        NonZeroUsize::new(64 * 1024).expect("state-image limit must be non-zero"),
        NonZeroUsize::new(256).expect("retained-image limit must be non-zero"),
        NonZeroUsize::new(16 * 1024 * 1024).expect("retained-byte limit must be non-zero"),
        NonZeroUsize::new(64 * 1024).expect("candidate limit must be non-zero"),
        NonZeroUsize::new(256).expect("emission-evidence limit must be non-zero"),
    )
    .expect("aggregate test retention limits must be valid");
    let aggregate = ClientAggregateLimits::new(
        NonZeroUsize::new(1).expect("lineage limit must be non-zero"),
        NonZeroUsize::new(256).expect("aggregate image limit must be non-zero"),
        NonZeroUsize::new(aggregate_budget).expect("aggregate byte budget must be non-zero"),
    );
    let policy = ClientReplicationPolicy::new(
        ReplicationLineageKey::new(SessionId::new(1), ParticipantId::new(1)),
        aggregate,
        retention,
    );

    let mut app = client_app_with_policy(policy);
    enqueue_client_inbox(
        app.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(1),
            cursor: SnapshotCursor(1),
            last_applied: SnapshotCursor::default(),
            payload: TestReplicationDriver::encode_snapshot(&first)
                .expect("first aggregate snapshot should encode"),
        }),
    )
    .expect("first aggregate snapshot should stage");
    let mut app = app
        .run_for_frames(1)
        .expect("first aggregate snapshot should commit");
    clear_client_outbound(app.world_mut());
    let before = active_client_snapshot(app.world()).expect("first product should be active");

    enqueue_client_inbox(
        app.world_mut(),
        ServerMessage::Snapshot(Snapshot {
            tick: SimulationTick(2),
            cursor: SnapshotCursor(2),
            last_applied: SnapshotCursor(1),
            payload: TestReplicationDriver::encode_snapshot(&second)
                .expect("second aggregate snapshot should encode"),
        }),
    )
    .expect("second aggregate snapshot should stage");
    let app = app
        .run_for_frames(1)
        .expect("aggregate rejection should fail closed");

    assert_eq!(
        client_replication_acknowledgement(app.world()),
        Some((SnapshotCursor(1), SimulationTick(1)))
    );
    assert_eq!(active_client_snapshot(app.world()), Some(before));
    assert_eq!(outbound_ack(app.world()), None);
}
