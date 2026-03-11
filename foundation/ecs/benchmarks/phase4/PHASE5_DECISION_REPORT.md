# Phase 5 Decision Report (Post Phase 4)

## 1. Phase 4 Scope Completed

Implemented and validated Phase 4 roadmap work in priority order:

1. broad-query baseline refresh (`foundation/ecs/benchmarks/phase4/phase4_baseline_*`)
2. query-path specialization
3. allocation reduction
4. re-benchmarking and profiling
5. explicit Phase 5 recommendation (this report)

## 2. Implemented Changes

### 2.1 Query-path specialization

- `foundation/ecs/src/world/world_internal_impl.rs`
  - `World::matching_entities_into(...)`
    - candidate-root selection from smallest required component store
    - single-component broad-query fast path
    - specialized retain branches for common small filter sets (1-2 required/excluded stores)
    - pre-resolved required/excluded store slices to reduce repeated map lookups
- `foundation/ecs/src/query/access_and_filters.rs`
  - `QueryFilter::needs_tick_filter()` added
  - `Changed<T>` and `Added<T>` mark tick-filter requirement
  - tuple filter impls propagate tick-filter requirement

### 2.2 Allocation reduction

- `foundation/ecs/src/query/traits_and_state.rs`
  - `QueryState` now owns reusable scratch-vector pool for matched entities
  - `iter()`/`single()` reuse pooled buffers instead of fresh `Vec<Entity>` churn
  - `QueryIter` returns entity buffers to pool on drop
  - no-tick-filter queries avoid extra materialization/filter pass

### 2.3 Safety/correctness regression coverage

- `foundation/ecs/tests/world.rs`
  - `broad_query_state_reuse_tracks_current_entities`
  - `broad_without_filter_reuse_stays_correct_after_component_toggle`

### 2.4 Phase 4 benchmark/runtime harness expansion

- `foundation/ecs/benches/phase4.rs`
  - adds C2 broad read-only and C3 broad write-only workloads
- `foundation/ecs/examples/phase4_profile.rs`
  - adds C2/C3 telemetry-attributed profile sections
- `engine/benches/phase4_runtime.rs`
  - dedicated Phase 4 engine/runtime scenario bench
- bench targets added:
  - `foundation/ecs/Cargo.toml` -> `[[bench]] name = "phase4"`
  - `engine/Cargo.toml` -> `[[bench]] name = "phase4_runtime"`

## 3. Measurement Summary

## 3.1 Phase 4 baseline vs optimized (comparable W1-W5 suite)

Source files:
- baseline: `phase4_baseline_ecs_bench.txt`, `phase4_baseline_profile.txt`, `phase4_baseline_engine_bench.txt`
- optimized: `phase4_optimized_ecs_bench.txt`, `phase4_optimized_profile.txt`, `phase4_optimized_engine_bench.txt`

### ECS microbench medians (quick Criterion)

| Workload | Baseline | Optimized | Delta |
|---|---:|---:|---:|
| W1 10k | 14.411 ms | 11.424 ms | -20.7% |
| W1 50k | 122.15 ms | 86.871 ms | -28.9% |
| W1 200k | 534.47 ms | 347.48 ms | -35.0% |
| W2 5k | 4.0957 ms | 3.5383 ms | -13.6% |
| W2 20k | 22.485 ms | 19.403 ms | -13.7% |
| W3 | 910.15 us | 286.21 us | -68.6% |
| W4 | 812.63 us | 500.83 us | -38.4% |
| W5 plan | 985.13 us | 854.00 us | -13.3% |
| W5 stage | 222.63 us | 186.08 us | -16.4% |

### Telemetry/profile attribution highlights

- cumulative `query_matching_nanos`: `229,488,700 -> 98,736,400` (`-57.0%`)
- cumulative `query_iter_nanos`: `320,527,900 -> 186,122,600` (`-41.9%`)
- W2 `query_matching_nanos`: `150,841,300 -> 70,817,600` (`-53.1%`)
- W2 `query_iter_nanos`: `241,858,600 -> 158,184,000` (`-34.6%`)

Engine scenario (quick Criterion):

| Workload | Baseline | Optimized | Delta |
|---|---:|---:|---:|
| Headless mixed frame 5k | 1.8151 ms | 1.4074 ms | -22.5% |
| Headless mixed frame 20k | 8.6392 ms | 7.3101 ms | -15.4% |

## 3.2 Phase 4-expanded suite (C2/C3 added)

Source files:
- `phase4_optimized_ecs_bench_v2.txt`
- `phase4_optimized_profile_v2.txt`
- `phase4_optimized_engine_bench_v2.txt`

Key C2/C3 medians:

| Workload | 10k | 50k | 200k |
|---|---:|---:|---:|
| C2 broad read-only | 2.0838 ms | 11.486 ms | 68.429 ms |
| C3 broad write-only | 8.8906 ms | 59.192 ms | 277.70 ms |

This confirms explicit Phase 4 coverage for roadmap C2/C3.

## 3.3 Required cross-phase comparison (phase3 baseline vs phase3.5 optimized vs phase4 optimized)

Reference artifacts:
- phase3 baseline: `foundation/ecs/benchmarks/phase35/baseline_*`
- phase3.5 optimized: `foundation/ecs/benchmarks/phase35/optimized_*`
- phase4 optimized: `foundation/ecs/benchmarks/phase4/phase4_optimized_*`

Observations:
- W2-class mixed/filter workloads remain massively better than phase3 baseline (seconds -> milliseconds after phase3.5/phase4 work).
- Broad-query throughput in absolute terms is sensitive to run conditions across phases; phase4 decisions were therefore made from same-phase baseline-vs-optimized comparisons plus telemetry attribution.

## 4. Bottleneck Attribution After Phase 4

- Query-path matching/iteration overhead dropped materially.
- `Changed<T>`/`Added<T>` checks remain controlled and are no longer dominant.
- Stage-end command flush remains small in mixed/gameplay profiles.
- Scheduler planning can spike in scheduler-stress workloads, but is not the dominant gameplay-frame cost.
- Remaining heavy path is broad mutable iteration at high entity counts (especially write-heavy paths like C3 and W1@200k).

## 5. Recommendation

## Decision: **Option B - Continue Incremental Optimization (Phase 5B)**

### Why

- Phase 4 achieved meaningful broad-query gains and measurable allocation-path wins without semantic or API changes.
- Evidence still points to optimizable internal overhead (query fetch/matching path) before storage redesign is mandatory.
- Scheduler/flush costs are not the primary limiter in representative mixed gameplay scenarios.

### Alternatives rejected

- **Option A (immediate archetype/storage redesign)** rejected for now:
  - current data still shows meaningful wins from low-risk internals
  - storage-model replacement is not yet proven necessary by Phase 4 evidence
- **Option C (scheduler throughput/parallelism first)** rejected for now:
  - scheduler/flush costs are measurable but generally secondary to broad query-path cost

## 6. Suggested Phase 5B Focus

1. cache typed store lookups for dominant query tuple forms in `QueryState`
2. reduce per-entity fetch indirection in hot mutable broad loops
3. keep C2/C3/W1 large-world benchmarks as mandatory regression gates
4. tighten benchmark stability protocol (CPU affinity/longer sampling) before architecture-level decisions

If these incremental steps fail to move W1/C3 materially at 50k-200k scales, promote to storage redesign in the next decision gate.
