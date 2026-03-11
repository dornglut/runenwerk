# ECS Phase 4 Benchmark Artifacts

This folder stores Phase 4 benchmark/profiling outputs and the Phase 5 decision report.

## Workload Coverage

- W1: broad transform-style update (`Query<(&mut Position, &Velocity)>`)
- C2: broad no-filter read query (`Query<&Position>`)
- C3: broad simple write query (`Query<&mut Velocity>`)
- W2: gameplay mixed workload (`With`/`Without`/`Changed`/`Added`)
- W3: structural churn (`Commands`, spawn/insert/remove/despawn)
- W4: event-heavy workload (`EventReader`/`EventWriter` + queries)
- W5: scheduler stress (conflicts, stage execution)
- Engine scenario: headless runtime mixed-frame workload

## Runners

```powershell
# Comparable Phase 3.5/4 workload family (W1-W5 + engine)
cargo bench -p ecs --bench phase35 --features telemetry -- --quick
cargo run -p ecs --example phase35_profile --features telemetry --release
cargo bench -p engine --bench phase35_runtime -- --quick

# Phase 4-expanded suite (adds C2/C3)
cargo bench -p ecs --bench phase4 --features telemetry -- --quick
cargo run -p ecs --example phase4_profile --features telemetry --release
cargo bench -p engine --bench phase4_runtime -- --quick
```

## Captured Outputs

- `phase4_baseline_ecs_bench.txt`
- `phase4_baseline_profile.txt`
- `phase4_baseline_engine_bench.txt`
- `phase4_optimized_ecs_bench.txt`
- `phase4_optimized_profile.txt`
- `phase4_optimized_engine_bench.txt`
- `phase4_optimized_ecs_bench_v2.txt`
- `phase4_optimized_profile_v2.txt`
- `phase4_optimized_engine_bench_v2.txt`
- `PHASE5_DECISION_REPORT.md`

## Notes

- `phase4_optimized_*` files are the direct Phase 4 baseline-vs-optimized comparison set.
- `*_v2` files are the expanded Phase 4 suite captures (explicit C2/C3 and `phase4_runtime`).
- Quick Criterion mode is used for repeatable local/CI loops; profile runs provide cost attribution.
