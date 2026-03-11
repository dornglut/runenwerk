# ECS Phase 3.5 Benchmark Artifacts

This folder stores reproducible benchmark/profiling outputs for ECS Phase 3.5.

## Workload Coverage

- W1: broad transform-style update
- W2: gameplay mixed filters (`With`/`Without`/`Changed`/`Added`)
- W3: structural churn (`Commands`, spawn/insert/remove/despawn)
- W4: event-heavy workload (`EventReader`/`EventWriter` + queries)
- W5: scheduler stress (set edges, conflicts, stage execution)
- Engine scenario: headless runtime mixed frame workload

## Runners

```powershell
cargo bench -p ecs --bench phase35 --features telemetry -- --quick
cargo run -p ecs --example phase35_profile --features telemetry --release
cargo bench -p engine --bench phase35_runtime -- --quick
```

## Captured Outputs

- `baseline_ecs_bench.txt`
- `optimized_ecs_bench.txt`
- `baseline_profile.txt`
- `optimized_profile.txt`
- `baseline_engine_bench.txt`
- `optimized_engine_bench.txt`
- `PHASE4_DECISION_REPORT.md`

Notes:

- Quick Criterion mode is used to keep the full suite repeatable in local/CI loops.
- Telemetry output is used for cost attribution across query/filter/scheduler/runtime/flush paths.
