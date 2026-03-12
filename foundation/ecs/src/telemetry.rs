use scheduler::telemetry::SchedulerTelemetrySnapshot;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EcsTelemetrySnapshot {
    pub query_matching_calls: u64,
    pub query_matching_nanos: u64,
    pub query_matching_candidates: u64,
    pub query_matching_matches: u64,
    pub query_iter_calls: u64,
    pub query_iter_nanos: u64,
    pub query_get_calls: u64,
    pub query_get_nanos: u64,
    pub query_single_calls: u64,
    pub query_single_nanos: u64,
    pub changed_check_calls: u64,
    pub changed_check_nanos: u64,
    pub added_check_calls: u64,
    pub added_check_nanos: u64,
    pub runtime_plan_calls: u64,
    pub runtime_plan_nanos: u64,
    pub runtime_stage_calls: u64,
    pub runtime_stage_nanos: u64,
    pub runtime_flush_calls: u64,
    pub runtime_flush_nanos: u64,
    pub runtime_flush_command_queues: u64,
    pub event_reader_calls: u64,
    pub event_reader_nanos: u64,
    pub events_read: u64,
    pub event_writer_calls: u64,
    pub event_writer_nanos: u64,
    pub events_written: u64,
    pub scheduler: SchedulerTelemetrySnapshot,
}

#[cfg(feature = "telemetry")]
mod imp {
    use super::EcsTelemetrySnapshot;
    use scheduler::telemetry;
    use std::sync::atomic::{AtomicU64, Ordering};

    static QUERY_MATCHING_CALLS: AtomicU64 = AtomicU64::new(0);
    static QUERY_MATCHING_NANOS: AtomicU64 = AtomicU64::new(0);
    static QUERY_MATCHING_CANDIDATES: AtomicU64 = AtomicU64::new(0);
    static QUERY_MATCHING_MATCHES: AtomicU64 = AtomicU64::new(0);
    static QUERY_ITER_CALLS: AtomicU64 = AtomicU64::new(0);
    static QUERY_ITER_NANOS: AtomicU64 = AtomicU64::new(0);
    static QUERY_GET_CALLS: AtomicU64 = AtomicU64::new(0);
    static QUERY_GET_NANOS: AtomicU64 = AtomicU64::new(0);
    static QUERY_SINGLE_CALLS: AtomicU64 = AtomicU64::new(0);
    static QUERY_SINGLE_NANOS: AtomicU64 = AtomicU64::new(0);
    static CHANGED_CHECK_CALLS: AtomicU64 = AtomicU64::new(0);
    static CHANGED_CHECK_NANOS: AtomicU64 = AtomicU64::new(0);
    static ADDED_CHECK_CALLS: AtomicU64 = AtomicU64::new(0);
    static ADDED_CHECK_NANOS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_PLAN_CALLS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_PLAN_NANOS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_STAGE_CALLS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_STAGE_NANOS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_FLUSH_CALLS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_FLUSH_NANOS: AtomicU64 = AtomicU64::new(0);
    static RUNTIME_FLUSH_COMMAND_QUEUES: AtomicU64 = AtomicU64::new(0);
    static EVENT_READER_CALLS: AtomicU64 = AtomicU64::new(0);
    static EVENT_READER_NANOS: AtomicU64 = AtomicU64::new(0);
    static EVENTS_READ: AtomicU64 = AtomicU64::new(0);
    static EVENT_WRITER_CALLS: AtomicU64 = AtomicU64::new(0);
    static EVENT_WRITER_NANOS: AtomicU64 = AtomicU64::new(0);
    static EVENTS_WRITTEN: AtomicU64 = AtomicU64::new(0);

    pub fn reset() {
        QUERY_MATCHING_CALLS.store(0, Ordering::Relaxed);
        QUERY_MATCHING_NANOS.store(0, Ordering::Relaxed);
        QUERY_MATCHING_CANDIDATES.store(0, Ordering::Relaxed);
        QUERY_MATCHING_MATCHES.store(0, Ordering::Relaxed);
        QUERY_ITER_CALLS.store(0, Ordering::Relaxed);
        QUERY_ITER_NANOS.store(0, Ordering::Relaxed);
        QUERY_GET_CALLS.store(0, Ordering::Relaxed);
        QUERY_GET_NANOS.store(0, Ordering::Relaxed);
        QUERY_SINGLE_CALLS.store(0, Ordering::Relaxed);
        QUERY_SINGLE_NANOS.store(0, Ordering::Relaxed);
        CHANGED_CHECK_CALLS.store(0, Ordering::Relaxed);
        CHANGED_CHECK_NANOS.store(0, Ordering::Relaxed);
        ADDED_CHECK_CALLS.store(0, Ordering::Relaxed);
        ADDED_CHECK_NANOS.store(0, Ordering::Relaxed);
        RUNTIME_PLAN_CALLS.store(0, Ordering::Relaxed);
        RUNTIME_PLAN_NANOS.store(0, Ordering::Relaxed);
        RUNTIME_STAGE_CALLS.store(0, Ordering::Relaxed);
        RUNTIME_STAGE_NANOS.store(0, Ordering::Relaxed);
        RUNTIME_FLUSH_CALLS.store(0, Ordering::Relaxed);
        RUNTIME_FLUSH_NANOS.store(0, Ordering::Relaxed);
        RUNTIME_FLUSH_COMMAND_QUEUES.store(0, Ordering::Relaxed);
        EVENT_READER_CALLS.store(0, Ordering::Relaxed);
        EVENT_READER_NANOS.store(0, Ordering::Relaxed);
        EVENTS_READ.store(0, Ordering::Relaxed);
        EVENT_WRITER_CALLS.store(0, Ordering::Relaxed);
        EVENT_WRITER_NANOS.store(0, Ordering::Relaxed);
        EVENTS_WRITTEN.store(0, Ordering::Relaxed);
        telemetry::reset();
    }

    pub fn snapshot() -> EcsTelemetrySnapshot {
        EcsTelemetrySnapshot {
            query_matching_calls: QUERY_MATCHING_CALLS.load(Ordering::Relaxed),
            query_matching_nanos: QUERY_MATCHING_NANOS.load(Ordering::Relaxed),
            query_matching_candidates: QUERY_MATCHING_CANDIDATES.load(Ordering::Relaxed),
            query_matching_matches: QUERY_MATCHING_MATCHES.load(Ordering::Relaxed),
            query_iter_calls: QUERY_ITER_CALLS.load(Ordering::Relaxed),
            query_iter_nanos: QUERY_ITER_NANOS.load(Ordering::Relaxed),
            query_get_calls: QUERY_GET_CALLS.load(Ordering::Relaxed),
            query_get_nanos: QUERY_GET_NANOS.load(Ordering::Relaxed),
            query_single_calls: QUERY_SINGLE_CALLS.load(Ordering::Relaxed),
            query_single_nanos: QUERY_SINGLE_NANOS.load(Ordering::Relaxed),
            changed_check_calls: CHANGED_CHECK_CALLS.load(Ordering::Relaxed),
            changed_check_nanos: CHANGED_CHECK_NANOS.load(Ordering::Relaxed),
            added_check_calls: ADDED_CHECK_CALLS.load(Ordering::Relaxed),
            added_check_nanos: ADDED_CHECK_NANOS.load(Ordering::Relaxed),
            runtime_plan_calls: RUNTIME_PLAN_CALLS.load(Ordering::Relaxed),
            runtime_plan_nanos: RUNTIME_PLAN_NANOS.load(Ordering::Relaxed),
            runtime_stage_calls: RUNTIME_STAGE_CALLS.load(Ordering::Relaxed),
            runtime_stage_nanos: RUNTIME_STAGE_NANOS.load(Ordering::Relaxed),
            runtime_flush_calls: RUNTIME_FLUSH_CALLS.load(Ordering::Relaxed),
            runtime_flush_nanos: RUNTIME_FLUSH_NANOS.load(Ordering::Relaxed),
            runtime_flush_command_queues: RUNTIME_FLUSH_COMMAND_QUEUES.load(Ordering::Relaxed),
            event_reader_calls: EVENT_READER_CALLS.load(Ordering::Relaxed),
            event_reader_nanos: EVENT_READER_NANOS.load(Ordering::Relaxed),
            events_read: EVENTS_READ.load(Ordering::Relaxed),
            event_writer_calls: EVENT_WRITER_CALLS.load(Ordering::Relaxed),
            event_writer_nanos: EVENT_WRITER_NANOS.load(Ordering::Relaxed),
            events_written: EVENTS_WRITTEN.load(Ordering::Relaxed),
            scheduler: telemetry::snapshot(),
        }
    }

    pub fn record_query_matching(duration_nanos: u64, candidates: u64, matches: u64) {
        QUERY_MATCHING_CALLS.fetch_add(1, Ordering::Relaxed);
        QUERY_MATCHING_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
        QUERY_MATCHING_CANDIDATES.fetch_add(candidates, Ordering::Relaxed);
        QUERY_MATCHING_MATCHES.fetch_add(matches, Ordering::Relaxed);
    }

    pub fn record_query_iter(duration_nanos: u64) {
        QUERY_ITER_CALLS.fetch_add(1, Ordering::Relaxed);
        QUERY_ITER_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_query_get(duration_nanos: u64) {
        QUERY_GET_CALLS.fetch_add(1, Ordering::Relaxed);
        QUERY_GET_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_query_single(duration_nanos: u64) {
        QUERY_SINGLE_CALLS.fetch_add(1, Ordering::Relaxed);
        QUERY_SINGLE_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_changed_check(duration_nanos: u64) {
        CHANGED_CHECK_CALLS.fetch_add(1, Ordering::Relaxed);
        CHANGED_CHECK_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_added_check(duration_nanos: u64) {
        ADDED_CHECK_CALLS.fetch_add(1, Ordering::Relaxed);
        ADDED_CHECK_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_runtime_plan(duration_nanos: u64) {
        RUNTIME_PLAN_CALLS.fetch_add(1, Ordering::Relaxed);
        RUNTIME_PLAN_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_runtime_stage(duration_nanos: u64) {
        RUNTIME_STAGE_CALLS.fetch_add(1, Ordering::Relaxed);
        RUNTIME_STAGE_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
    }

    pub fn record_runtime_flush(duration_nanos: u64, command_queues: u64) {
        RUNTIME_FLUSH_CALLS.fetch_add(1, Ordering::Relaxed);
        RUNTIME_FLUSH_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
        RUNTIME_FLUSH_COMMAND_QUEUES.fetch_add(command_queues, Ordering::Relaxed);
    }

    pub fn record_event_reader(duration_nanos: u64, events_read: u64) {
        EVENT_READER_CALLS.fetch_add(1, Ordering::Relaxed);
        EVENT_READER_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
        EVENTS_READ.fetch_add(events_read, Ordering::Relaxed);
    }

    pub fn record_event_writer(duration_nanos: u64, events_written: u64) {
        EVENT_WRITER_CALLS.fetch_add(1, Ordering::Relaxed);
        EVENT_WRITER_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
        EVENTS_WRITTEN.fetch_add(events_written, Ordering::Relaxed);
    }
}

#[cfg(not(feature = "telemetry"))]
mod imp {
    use super::EcsTelemetrySnapshot;

    pub fn reset() {}

    pub fn snapshot() -> EcsTelemetrySnapshot {
        EcsTelemetrySnapshot::default()
    }

    pub fn record_query_matching(_duration_nanos: u64, _candidates: u64, _matches: u64) {}
    pub fn record_query_iter(_duration_nanos: u64) {}
    pub fn record_query_get(_duration_nanos: u64) {}
    pub fn record_query_single(_duration_nanos: u64) {}
    pub fn record_changed_check(_duration_nanos: u64) {}
    pub fn record_added_check(_duration_nanos: u64) {}
    pub fn record_runtime_plan(_duration_nanos: u64) {}
    pub fn record_runtime_stage(_duration_nanos: u64) {}
    pub fn record_runtime_flush(_duration_nanos: u64, _command_queues: u64) {}
    pub fn record_event_reader(_duration_nanos: u64, _events_read: u64) {}
    pub fn record_event_writer(_duration_nanos: u64, _events_written: u64) {}
}

pub fn reset() {
    imp::reset();
}

pub fn snapshot() -> EcsTelemetrySnapshot {
    imp::snapshot()
}

pub fn snapshot_delta(
    before: &EcsTelemetrySnapshot,
    after: &EcsTelemetrySnapshot,
) -> EcsTelemetrySnapshot {
    EcsTelemetrySnapshot {
        query_matching_calls: after
            .query_matching_calls
            .saturating_sub(before.query_matching_calls),
        query_matching_nanos: after
            .query_matching_nanos
            .saturating_sub(before.query_matching_nanos),
        query_matching_candidates: after
            .query_matching_candidates
            .saturating_sub(before.query_matching_candidates),
        query_matching_matches: after
            .query_matching_matches
            .saturating_sub(before.query_matching_matches),
        query_iter_calls: after
            .query_iter_calls
            .saturating_sub(before.query_iter_calls),
        query_iter_nanos: after
            .query_iter_nanos
            .saturating_sub(before.query_iter_nanos),
        query_get_calls: after.query_get_calls.saturating_sub(before.query_get_calls),
        query_get_nanos: after.query_get_nanos.saturating_sub(before.query_get_nanos),
        query_single_calls: after
            .query_single_calls
            .saturating_sub(before.query_single_calls),
        query_single_nanos: after
            .query_single_nanos
            .saturating_sub(before.query_single_nanos),
        changed_check_calls: after
            .changed_check_calls
            .saturating_sub(before.changed_check_calls),
        changed_check_nanos: after
            .changed_check_nanos
            .saturating_sub(before.changed_check_nanos),
        added_check_calls: after
            .added_check_calls
            .saturating_sub(before.added_check_calls),
        added_check_nanos: after
            .added_check_nanos
            .saturating_sub(before.added_check_nanos),
        runtime_plan_calls: after
            .runtime_plan_calls
            .saturating_sub(before.runtime_plan_calls),
        runtime_plan_nanos: after
            .runtime_plan_nanos
            .saturating_sub(before.runtime_plan_nanos),
        runtime_stage_calls: after
            .runtime_stage_calls
            .saturating_sub(before.runtime_stage_calls),
        runtime_stage_nanos: after
            .runtime_stage_nanos
            .saturating_sub(before.runtime_stage_nanos),
        runtime_flush_calls: after
            .runtime_flush_calls
            .saturating_sub(before.runtime_flush_calls),
        runtime_flush_nanos: after
            .runtime_flush_nanos
            .saturating_sub(before.runtime_flush_nanos),
        runtime_flush_command_queues: after
            .runtime_flush_command_queues
            .saturating_sub(before.runtime_flush_command_queues),
        event_reader_calls: after
            .event_reader_calls
            .saturating_sub(before.event_reader_calls),
        event_reader_nanos: after
            .event_reader_nanos
            .saturating_sub(before.event_reader_nanos),
        events_read: after.events_read.saturating_sub(before.events_read),
        event_writer_calls: after
            .event_writer_calls
            .saturating_sub(before.event_writer_calls),
        event_writer_nanos: after
            .event_writer_nanos
            .saturating_sub(before.event_writer_nanos),
        events_written: after.events_written.saturating_sub(before.events_written),
        scheduler: scheduler::telemetry::SchedulerTelemetrySnapshot {
            plan_build_calls: after
                .scheduler
                .plan_build_calls
                .saturating_sub(before.scheduler.plan_build_calls),
            plan_build_nanos: after
                .scheduler
                .plan_build_nanos
                .saturating_sub(before.scheduler.plan_build_nanos),
            plan_conflict_checks: after
                .scheduler
                .plan_conflict_checks
                .saturating_sub(before.scheduler.plan_conflict_checks),
            plan_stage_count: after
                .scheduler
                .plan_stage_count
                .saturating_sub(before.scheduler.plan_stage_count),
        },
    }
}

pub(crate) fn record_query_matching(duration_nanos: u64, candidates: u64, matches: u64) {
    imp::record_query_matching(duration_nanos, candidates, matches);
}

pub(crate) fn record_query_iter(duration_nanos: u64) {
    imp::record_query_iter(duration_nanos);
}

pub(crate) fn record_query_get(duration_nanos: u64) {
    imp::record_query_get(duration_nanos);
}

pub(crate) fn record_query_single(duration_nanos: u64) {
    imp::record_query_single(duration_nanos);
}

pub(crate) fn record_changed_check(duration_nanos: u64) {
    imp::record_changed_check(duration_nanos);
}

pub(crate) fn record_added_check(duration_nanos: u64) {
    imp::record_added_check(duration_nanos);
}

pub(crate) fn record_runtime_plan(duration_nanos: u64) {
    imp::record_runtime_plan(duration_nanos);
}

pub(crate) fn record_runtime_stage(duration_nanos: u64) {
    imp::record_runtime_stage(duration_nanos);
}

pub(crate) fn record_runtime_flush(duration_nanos: u64, command_queues: u64) {
    imp::record_runtime_flush(duration_nanos, command_queues);
}

pub(crate) fn record_event_reader(duration_nanos: u64, events_read: u64) {
    imp::record_event_reader(duration_nanos, events_read);
}

pub(crate) fn record_event_writer(duration_nanos: u64, events_written: u64) {
    imp::record_event_writer(duration_nanos, events_written);
}
