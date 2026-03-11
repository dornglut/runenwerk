# ECS Phase 5B Benchmark Artifacts

This folder stores the Phase 5B measurement set, including a refreshed
Phase 4 comparison baseline and the final Phase 6 recommendation.

## Workload Coverage

- W1: broad transform-style update (`Query<(&mut Position, &Velocity)>`)
- C2: broad no-filter read query (`Query<&Position>`)
- C3: broad simple write query (`Query<&mut Velocity>`)
- C4: broad double-mutable query (`Query<(&mut Position, &mut Velocity)>`)
- W2: gameplay mixed workload (`With`/`Without`/`Changed`/`Added`)
- W3: structural churn (`Commands`, spawn/insert/remove/despawn)
- W4: event-heavy workload (`EventReader`/`EventWriter` + queries)
- W5: scheduler stress (conflicts, stage execution)
- Engine scenario: headless runtime mixed-frame workload

## Runners

```powershell
# Phase 4 comparison refresh (same machine/session as Phase 5B run)
cargo bench -p ecs --bench phase4 --features telemetry -- --quick
cargo run -p ecs --example phase4_profile --features telemetry --release
cargo bench -p engine --bench phase4_runtime -- --quick

# Phase 5B optimized suite
cargo bench -p ecs --bench phase5b --features telemetry -- --quick
cargo run -p ecs --example phase5b_profile --features telemetry --release
cargo bench -p engine --bench phase5b_runtime -- --quick
```

## Captured Outputs

- `phase4_baseline_refresh_ecs_bench.txt`
- `phase4_baseline_refresh_profile.txt`
- `phase4_baseline_refresh_engine_bench.txt`
- `phase5b_optimized_ecs_bench.txt`
- `phase5b_optimized_profile.txt`
- `phase5b_optimized_engine_bench.txt`
- `PHASE6_DECISION_REPORT.md`

## Notes

- Historical Phase 4 optimized artifacts remain in `foundation/ecs/benchmarks/phase4/`.
- Phase 5B decisions are based on both:
  - historical Phase 4 optimized -> Phase 5B optimized deltas
  - refreshed same-session Phase 4 -> Phase 5B deltas
- Quick Criterion mode is retained for repeatable local/CI loops; profile runs provide
  hot-path attribution.
