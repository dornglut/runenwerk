# Phase 6 Final Decision Report

Roadmap source of truth: [`/foundation/ecs/docs/roadmaps/phase6-closeout-roadmap.md`](../../roadmaps/phase6-closeout-roadmap.md)

## 1. Decision

Decision: **Phase 6 closeout criteria are met with documented performance tradeoffs.**

The architecture is considered closed out because:

- archetype-backed dense typed columns are the canonical component storage model
- dominant mutable query shapes run archetype-row-native
- `Changed<T>` / `Added<T>` query semantics are archetype row metadata driven
- reporting/introspection APIs are explicitly retained as log/tick infrastructure
- required tests, benchmarks, profiles, and written artifacts are complete

## 2. Evidence Summary

### 2.1 Required validations completed

- `cargo test -p ecs` passed
- `cargo bench -p ecs --bench phase6 --features telemetry -- --quick` passed
- `cargo run -p ecs --example phase6_profile --features telemetry --release` passed
- `cargo bench -p engine --bench phase6_runtime -- --quick` passed

### 2.2 Comparison-set completed

Compared Phase 6 results against:

- historical Phase 5B snapshot values captured prior to closeout docs cleanup
- refreshed same-session baseline (`foundation/ecs/benchmarks/phase6/phase5b_refresh_*`)
- current Phase 6 artifacts (`foundation/ecs/benchmarks/phase6/phase6_*`)

## 3. Benchmark Questions (Roadmap)

### 3.1 Query path

Question: did archetype ownership/fetch materially reduce dominant query costs?

- For archetype-native dominant mutable tuple (`C4`), large-size cost improved vs historical Phase 5B (`C4@200k: 348.37 ms -> 290.51 ms`, `-16.61%`).
- Same-session comparisons are mixed; several broad-loop forms are better than refresh at 10k/50k while others regress at 200k.
- Telemetry cumulative `query_matching_nanos` and `query_iter_nanos` are materially lower than historical Phase 5B (`-77.18%` and `-44.08%` respectively), and slightly lower than same-session Phase 5B refresh (`-1.22%` and `-2.16%`).

Question: did `Changed` / `Added` checks remain acceptable?

- `changed_check_nanos` and `added_check_nanos` remained in the same order of magnitude as refreshed baseline.
- W2 wall time remained near refreshed baseline (`313.328 ms -> 310.587 ms`, `-0.87%`).

### 3.2 Runtime path

Question: did the engine mixed frame improve or remain acceptable?

- Versus historical Phase 5B: both sizes improved (`5k: -14.89%`, `20k: -3.85%`).
- Versus same-session refresh: near parity (`5k: +2.12%`, `20k: +0.24%`).

Question: did scheduler/flush become more visible after storage/query improvements?

- Runtime plan/flush remain secondary relative to stage/query totals in mixed workloads.
- Cumulative `runtime_plan_nanos`/`runtime_flush_nanos` are higher than same-session refresh, but still much smaller than `runtime_stage_nanos` in absolute terms.

### 3.3 Structural path

Question: did churn regress enough to matter materially?

- W3/W4 medians are slower than both historical and refreshed baselines in this run.
- Regressions are documented and treated as known closeout tradeoffs; no correctness regressions were observed.

Question: are swap-remove and migration costs acceptable?

- Correctness invariants held under tests and runtime churn scenarios.
- Performance remains acceptable for roadmap closeout, with regressions explicitly recorded for follow-on tuning outside this closeout scope.

## 4. Required Metrics (Phase 6)

From `phase6_profile.txt` cumulative snapshot:

- `query_matching_nanos`: 26,556,300
- `query_iter_nanos`: 109,008,700
- `query_get_nanos`: 0
- `query_single_nanos`: 0
- `changed_check_nanos`: 26,700,800
- `added_check_nanos`: 27,022,800
- `runtime_plan_nanos`: 10,488,000
- `runtime_stage_nanos`: 259,089,100
- `runtime_flush_nanos`: 7,375,600
- `event_reader_nanos`: 1,100
- `event_writer_nanos`: 210,500

Notes:

- `query_get_nanos` and `query_single_nanos` are zero because the benchmark/profile workloads are iter-path focused and do not call `get`/`single`.

## 5. Final Closeout Statement

Phase 6 is closed out as an engineering phase per
`/foundation/ecs/docs/roadmaps/phase6-closeout-roadmap.md`.

The architecture, benchmarks, and documentation now consistently describe the ECS as archetype-backed with explicit separation between query/filter semantics (archetype metadata) and reporting/introspection history (change logs/ticks).
