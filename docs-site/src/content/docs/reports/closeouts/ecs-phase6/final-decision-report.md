---
title: "Phase 6 Final Decision Report"
description: "Historical closeout decision for the completed Runenwerk ECS Phase 6 program."
status: completed
owner: ecs
layer: reports
canonical: false
last_reviewed: 2026-09-15
---

# Phase 6 Final Decision Report

The predecessor Phase 6 closeout roadmap is retained in Git history. This report preserves the final point-in-time decision and evidence from that completed program.

## 1. Decision

Decision: **Phase 6 closeout criteria are met with documented performance tradeoffs.**

The architecture was considered closed out because:

- archetype-backed dense typed columns were the canonical component storage model
- dominant mutable query shapes ran archetype-row-native
- `Changed<T>` / `Added<T>` query semantics were archetype row metadata driven
- reporting/introspection APIs were explicitly retained as log/tick infrastructure
- required tests, benchmarks, profiles, and written artifacts were complete

These statements describe the completed Runenwerk predecessor implementation. Current reusable RunenECS semantics and implementation authority belong to standalone `dornglut/runen-ecs`.

## 2. Evidence Summary

### 2.1 Required validations completed

- `cargo test -p ecs` passed
- `cargo bench -p ecs --bench phase6 --features telemetry -- --quick` passed
- `cargo run -p ecs --example phase6_profile --features telemetry --release` passed
- `cargo bench -p engine --bench phase6_runtime -- --quick` passed

### 2.2 Comparison-set completed

Compared Phase 6 results against:

- historical Phase 5B snapshot values captured prior to closeout docs cleanup
- refreshed same-session baseline (`domain/ecs/benchmarks/phase6/phase5b_refresh_*`)
- Phase 6 artifacts (`domain/ecs/benchmarks/phase6/phase6_*`)

Those predecessor artifact paths are historical provenance and are not current repository navigation.

## 3. Benchmark Questions (Roadmap)

### 3.1 Query path

Question: did archetype ownership/fetch materially reduce dominant query costs?

- For archetype-native dominant mutable tuple (`C4`), large-size cost remained improved vs historical Phase 5B (`C4@200k: 348.37 ms -> 292.14 ms`, `-16.14%`).
- Same-session comparisons were mixed; several broad-loop forms were better than refresh at some sizes while others regressed.
- Cumulative telemetry remained query- and stage-dominated, with runtime plan/flush secondary in absolute totals.

Question: did `Changed` / `Added` checks remain acceptable?

- `changed_check_nanos` and `added_check_nanos` remained in expected order of magnitude.
- `W2@20k` median remained above same-session refresh (`12.417 ms` vs `10.987 ms`, `+13.02%`), and this tradeoff was documented.

### 3.2 Runtime path

Question: did the engine mixed frame improve or remain acceptable?

- Versus historical Phase 5B: both sizes improved (`5k: -15.16%`, `20k: -12.01%`).
- Versus same-session refresh: near parity at 5k (`+1.80%`) and improved at 20k (`-8.27%`).

Question: did scheduler/flush become more visible after storage/query improvements?

- Runtime plan/flush remained secondary relative to stage/query totals in mixed workloads.
- Cumulative `runtime_plan_nanos`/`runtime_flush_nanos` were substantially smaller than `runtime_stage_nanos` in absolute terms.

### 3.3 Structural path

Question: did churn regress enough to matter materially?

- W3/W4 medians were slower than both historical and refreshed baselines in this run.
- Regressions were documented and treated as known closeout tradeoffs; no correctness regressions were observed.

Question: were swap-remove and migration costs acceptable?

- Correctness invariants held under tests and runtime churn scenarios.
- Performance was accepted for roadmap closeout, with regressions explicitly recorded for follow-on tuning outside that closeout scope.

## 4. Required Metrics (Phase 6)

From `phase6_profile.txt` cumulative snapshot:

- `query_matching_nanos`: 27,022,800
- `query_iter_nanos`: 115,336,400
- `query_get_nanos`: 0
- `query_single_nanos`: 0
- `changed_check_nanos`: 29,779,800
- `added_check_nanos`: 27,808,000
- `runtime_plan_nanos`: 8,435,300
- `runtime_stage_nanos`: 262,304,400
- `runtime_flush_nanos`: 6,480,000
- `event_reader_nanos`: 1,100
- `event_writer_nanos`: 224,300

Notes:

- `query_get_nanos` and `query_single_nanos` are zero because the benchmark/profile workloads were iter-path focused and did not call `get`/`single`.
- The profile output explicitly reported this zero-value reason per workload block.

## 5. Final Closeout Statement

Phase 6 is closed out as a historical Runenwerk engineering phase. The predecessor roadmap and deleted local domain paths remain recoverable from Git history.

The retained benchmark and closeout evidence records what the predecessor ECS implementation proved at that time; it does not define current standalone RunenECS architecture.
