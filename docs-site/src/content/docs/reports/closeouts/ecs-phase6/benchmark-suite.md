---
title: "ECS Phase 6 Benchmark Artifacts"
description: "Historical benchmark evidence for the completed Runenwerk ECS Phase 6 program."
status: completed
owner: ecs
layer: reports
canonical: false
last_reviewed: 2026-09-15
---

# ECS Phase 6 Benchmark Artifacts

This folder stores historical measurement evidence from the completed Runenwerk ECS Phase 6 program. The predecessor closeout roadmap is retained in Git history rather than as current domain authority.

## Workload Coverage

- W1: broad transform-style update (`Query<(&mut Position, &Velocity)>`)
- C2: broad no-filter read query (`Query<&Position>`)
- C3: broad simple write query (`Query<&mut Velocity>`)
- C4: broad double-mutable query (`Query<(&mut Position, &mut Velocity)>`)
- W2: gameplay mixed workload (`With` / `Without` / `Changed` / `Added`)
- W3: structural churn (`Commands`, spawn/insert/remove/despawn)
- W4: event-heavy workload (`BroadcastReader` / `BroadcastWriter` + query)
- W5: scheduler stress (plan + stage execution)
- Engine scenario: headless runtime mixed-frame workload

## Required Phase 6 Commands

```powershell
cargo test -p ecs
cargo bench -p ecs --bench phase6 --features telemetry -- --quick
cargo run -p ecs --example phase6_profile --features telemetry --release
cargo bench -p engine --bench phase6_runtime -- --quick
```

## Comparison Refresh Commands

These commands supplied the comparison set recorded by the historical Phase 6 closeout plan.

```powershell
cargo bench -p ecs --bench phase5b --features telemetry -- --quick
cargo run -p ecs --example phase5b_profile --features telemetry --release
cargo bench -p engine --bench phase5b_runtime -- --quick
```

## Captured Outputs

- `phase6_test.txt`
- `phase6_ecs_bench.txt`
- `phase6_profile.txt`
- `phase6_engine_bench.txt`
- `phase5b_refresh_ecs_bench.txt`
- `phase5b_refresh_profile.txt`
- `phase5b_refresh_engine_bench.txt`
- `progress-report.md`
- `final-decision-report.md`

Historical comparison values are retained in the Phase 6 reports in this folder.
