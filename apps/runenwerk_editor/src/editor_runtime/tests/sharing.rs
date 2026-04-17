use editor_core::{
    ChangeOrigin, ReconciliationRejectReason, ReconciliationResult, SharedChangeEnvelope,
    SharedChangePropagationSink, SharedChangeSequence, SharingPolicy, WorkflowEventKind,
};
use editor_scene::{SceneCommandIntent, scene_intent_to_command};

use crate::editor_runtime::{
    RunenwerkEditorRuntime, execute_scene_command_and_push_history_with_origin,
};

#[test]
fn broadcast_policy_enqueues_shared_change_and_logs_workflow_event() {
    let mut runtime = RunenwerkEditorRuntime::new();
    runtime.set_sharing_policy(SharingPolicy::SessionBroadcast);

    let change = create_entity_change(&mut runtime, "Shared", ChangeOrigin::Runtime);

    assert_eq!(runtime.queued_shared_change_count(), 1);
    assert!(runtime.workflow_log().iter().any(|event| {
        matches!(
            event.kind,
            WorkflowEventKind::SharedChangeQueued {
                sequence: SharedChangeSequence(1)
            }
        )
    }));

    let queued = runtime.drain_shared_changes();
    assert_eq!(queued.len(), 1);
    assert_eq!(queued[0].change.ratification_id, change.ratification_id);
}

#[test]
fn propagate_shared_changes_preserves_queue_when_sink_fails() {
    let mut runtime = RunenwerkEditorRuntime::new();
    runtime.set_sharing_policy(SharingPolicy::SessionBroadcast);
    create_entity_change(&mut runtime, "Queued", ChangeOrigin::Runtime);

    let mut sink = AlwaysFailingSink;
    let result = runtime.propagate_shared_changes(&mut sink);
    assert_eq!(result, Err("transport unavailable"));
    assert_eq!(runtime.queued_shared_change_count(), 1);
}

#[test]
fn reconcile_shared_change_accepts_matching_base_version() {
    let mut producer = RunenwerkEditorRuntime::new();
    let produced = create_entity_change(&mut producer, "Producer", ChangeOrigin::Runtime);
    let envelope = SharedChangeEnvelope::new(SharedChangeSequence(1), produced.clone());

    let mut consumer = RunenwerkEditorRuntime::new();
    let decision = consumer.reconcile_shared_change(envelope);

    assert!(decision.is_accepted());
    assert_eq!(
        consumer.current_scene_reality_version(),
        produced.result_version
    );
    assert_eq!(consumer.ratified_change_log().len(), 1);
    assert!(consumer.workflow_log().iter().any(|event| {
        matches!(
            event.kind,
            WorkflowEventKind::SharedChangeReconciled {
                sequence: SharedChangeSequence(1),
                result: ReconciliationResult::Accepted,
            }
        )
    }));
}

#[test]
fn reconcile_shared_change_rejects_base_version_mismatch() {
    let mut local_runtime = RunenwerkEditorRuntime::new();
    let local_change = create_entity_change(&mut local_runtime, "Local", ChangeOrigin::Runtime);

    let mut remote_runtime = RunenwerkEditorRuntime::new();
    let remote_change = create_entity_change(&mut remote_runtime, "Remote", ChangeOrigin::Runtime);
    let decision = local_runtime.reconcile_shared_change(SharedChangeEnvelope::new(
        SharedChangeSequence(9),
        remote_change,
    ));

    assert!(matches!(
        decision.result,
        ReconciliationResult::Rejected(ReconciliationRejectReason::BaseVersionMismatch { .. })
    ));
    assert_eq!(
        local_runtime.current_scene_reality_version(),
        local_change.result_version
    );
    assert_eq!(local_runtime.ratified_change_log().len(), 1);
}

fn create_entity_change(
    runtime: &mut RunenwerkEditorRuntime,
    display_name: &str,
    origin: ChangeOrigin,
) -> editor_core::RatifiedChange {
    let command_id = runtime.allocate_command_id();
    let transaction_id = runtime.allocate_transaction_id();
    execute_scene_command_and_push_history_with_origin(
        runtime,
        scene_intent_to_command(
            command_id,
            SceneCommandIntent::CreateEntity {
                parent: None,
                display_name: display_name.to_string(),
            },
        ),
        format!("Create {display_name}"),
        transaction_id,
        origin,
    )
    .expect("scene command should execute")
    .expect("scene command should ratify a change")
}

struct AlwaysFailingSink;

impl SharedChangePropagationSink for AlwaysFailingSink {
    type Error = &'static str;

    fn push_shared_change(&mut self, _envelope: SharedChangeEnvelope) -> Result<(), Self::Error> {
        Err("transport unavailable")
    }
}
