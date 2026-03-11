#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchedulerTelemetrySnapshot {
    pub plan_build_calls: u64,
    pub plan_build_nanos: u64,
    pub plan_conflict_checks: u64,
    pub plan_stage_count: u64,
}

#[cfg(feature = "telemetry")]
mod imp {
    use super::SchedulerTelemetrySnapshot;
    use std::sync::atomic::{AtomicU64, Ordering};

    static PLAN_BUILD_CALLS: AtomicU64 = AtomicU64::new(0);
    static PLAN_BUILD_NANOS: AtomicU64 = AtomicU64::new(0);
    static PLAN_CONFLICT_CHECKS: AtomicU64 = AtomicU64::new(0);
    static PLAN_STAGE_COUNT: AtomicU64 = AtomicU64::new(0);

    pub fn reset() {
        PLAN_BUILD_CALLS.store(0, Ordering::Relaxed);
        PLAN_BUILD_NANOS.store(0, Ordering::Relaxed);
        PLAN_CONFLICT_CHECKS.store(0, Ordering::Relaxed);
        PLAN_STAGE_COUNT.store(0, Ordering::Relaxed);
    }

    pub fn snapshot() -> SchedulerTelemetrySnapshot {
        SchedulerTelemetrySnapshot {
            plan_build_calls: PLAN_BUILD_CALLS.load(Ordering::Relaxed),
            plan_build_nanos: PLAN_BUILD_NANOS.load(Ordering::Relaxed),
            plan_conflict_checks: PLAN_CONFLICT_CHECKS.load(Ordering::Relaxed),
            plan_stage_count: PLAN_STAGE_COUNT.load(Ordering::Relaxed),
        }
    }

    pub fn record_plan_build(duration_nanos: u64, conflict_checks: u64, stage_count: u64) {
        PLAN_BUILD_CALLS.fetch_add(1, Ordering::Relaxed);
        PLAN_BUILD_NANOS.fetch_add(duration_nanos, Ordering::Relaxed);
        PLAN_CONFLICT_CHECKS.fetch_add(conflict_checks, Ordering::Relaxed);
        PLAN_STAGE_COUNT.fetch_add(stage_count, Ordering::Relaxed);
    }
}

#[cfg(not(feature = "telemetry"))]
mod imp {
    use super::SchedulerTelemetrySnapshot;

    pub fn reset() {}

    pub fn snapshot() -> SchedulerTelemetrySnapshot {
        SchedulerTelemetrySnapshot::default()
    }

    pub fn record_plan_build(_duration_nanos: u64, _conflict_checks: u64, _stage_count: u64) {}
}

pub fn reset() {
    imp::reset();
}

pub fn snapshot() -> SchedulerTelemetrySnapshot {
    imp::snapshot()
}

pub(crate) fn record_plan_build(duration_nanos: u64, conflict_checks: u64, stage_count: u64) {
    imp::record_plan_build(duration_nanos, conflict_checks, stage_count);
}
