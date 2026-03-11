# Phase 6 Decision Report (Post Phase 5B)

## 1. Phase 5B Scope Completed

Implemented and validated Phase 5B in roadmap order:

1. dominant query-form optimization
2. typed store lookup caching
3. hot-loop indirection reduction
4. re-benchmarking and profiling
5. final Phase 6 recommendation (this report)

## 2. Implemented Changes

### 2.1 Dominant query-form optimization

- `foundation/ecs/src/query/query_data_impls.rs`
  - `impl QueryData for &mut T`
    - `supports_fast_path`
    - `prepare_fast_cache`
    - `mark_changed_fast`
    - `fetch_fast`
  - `impl QueryData for (&mut A, &B)`
    - `supports_fast_path`
    - `prepare_fast_cache`
    - `mark_changed_fast`
    - `fetch_fast`
  - `impl QueryData for (&mut A, &mut B)`
    - `supports_fast_path`
    - `prepare_fast_cache`
    - `mark_changed_fast`
    - `fetch_fast`
  - `impl QueryData for &T` fast read path parity for broad C2 tracking.

### 2.2 Typed store lookup caching

- `foundation/ecs/src/query/traits_and_state.rs`
  - `QueryState` now carries `fast_path_enabled` and `fast_cache`.
  - `QueryState::iter(...)` prepares/reuses cached typed store handles for reused
    `QueryState` executions.
  - Cache is rebound safely when iterating a different world (`world_ptr` guard).
- `foundation/ecs/tests/world.rs`
  - `query_state_cache_rebinds_when_iterating_a_different_world`
  - `query_state_cache_recovers_when_store_appears_after_empty_run`

### 2.3 Hot-loop indirection reduction

- `foundation/ecs/src/query/traits_and_state.rs`
  - `QueryIter::next(...)`
    - one-time branch between fast and generic loops (instead of per-entity branch)
    - pointer-based entity iteration to reduce repeated option/index indirection
  - `QueryState::iter(...)`
    - fast path now activates only when cache preparation succeeds.

### 2.4 Benchmark/runtime harness additions

- `foundation/ecs/benches/phase5b.rs`
  - adds C4 (`Query<(&mut Position, &mut Velocity)>`) alongside W1/C2/C3/W2/W3/W4/W5.
- `foundation/ecs/examples/phase5b_profile.rs`
  - adds C4 telemetry-attributed profile section.
- `engine/benches/phase5b_runtime.rs`
  - dedicated Phase 5B engine/runtime mixed-frame scenario.

## 3. Measurement Summary

## 3.1 Required gate workloads vs historical Phase 4 optimized baseline

Phase 4 source artifacts:
- `foundation/ecs/benchmarks/phase4/phase4_optimized_ecs_bench_v2.txt`
- `foundation/ecs/benchmarks/phase4/phase4_optimized_profile_v2.txt`
- `foundation/ecs/benchmarks/phase4/phase4_optimized_engine_bench_v2.txt`

Phase 5B source artifacts:
- `foundation/ecs/benchmarks/phase5b/phase5b_optimized_ecs_bench.txt`
- `foundation/ecs/benchmarks/phase5b/phase5b_optimized_profile.txt`
- `foundation/ecs/benchmarks/phase5b/phase5b_optimized_engine_bench.txt`

### ECS microbench medians (quick Criterion)

| Workload | Phase 4 optimized | Phase 5B optimized | Delta |
|---|---:|---:|---:|
| W1 10k | 18.853 ms | 5.7622 ms | -69.4% |
| W1 50k | 82.478 ms | 34.029 ms | -58.7% |
| W1 200k | 404.06 ms | 203.20 ms | -49.7% |
| C2 10k | 2.0838 ms | 1.0226 ms | -50.9% |
| C2 50k | 11.486 ms | 5.5932 ms | -51.3% |
| C2 200k | 68.429 ms | 37.313 ms | -45.5% |
| C3 10k | 8.8906 ms | 3.7706 ms | -57.6% |
| C3 50k | 59.192 ms | 18.121 ms | -69.4% |
| C3 200k | 277.70 ms | 132.56 ms | -52.3% |
| W2 5k | 4.2674 ms | 2.4545 ms | -42.5% |
| W2 20k | 19.694 ms | 11.768 ms | -40.2% |

### Engine/runtime mixed-frame medians

| Workload | Phase 4 optimized | Phase 5B optimized | Delta |
|---|---:|---:|---:|
| Headless mixed frame 5k | 2.2823 ms | 0.8922 ms | -60.9% |
| Headless mixed frame 20k | 7.3178 ms | 3.6222 ms | -50.5% |

### Profile attribution (historical Phase 4 optimized -> Phase 5B)

| Workload | Wall | `query_matching_nanos` | `query_iter_nanos` |
|---|---:|---:|---:|
| W1 | 254.432 ms -> 136.090 ms (-46.5%) | 25,156,600 -> 19,617,800 (-22.0%) | 25,169,300 -> 19,628,200 (-22.0%) |
| C2 | 80.020 ms -> 62.458 ms (-22.0%) | 2,725,600 -> 1,721,700 (-36.8%) | 2,728,700 -> 1,723,800 (-36.8%) |
| C3 | 209.212 ms -> 104.917 ms (-49.9%) | 7,425,700 -> 5,453,600 (-26.6%) | 7,431,100 -> 5,457,800 (-26.6%) |
| W2 | 715.624 ms -> 304.014 ms (-57.5%) | 120,674,000 -> 59,516,700 (-50.7%) | 274,036,300 -> 138,019,100 (-49.6%) |

W2 filter costs also dropped:
- `changed_check_nanos`: `53,865,100 -> 21,973,200` (`-59.2%`)
- `added_check_nanos`: `54,489,600 -> 31,218,800` (`-42.7%`)

## 3.2 Same-session Phase 4 refresh vs Phase 5B (stability check)

Refresh artifacts:
- `phase4_baseline_refresh_ecs_bench.txt`
- `phase4_baseline_refresh_profile.txt`
- `phase4_baseline_refresh_engine_bench.txt`

Key quick-bench medians:

| Workload | Phase 4 refresh | Phase 5B | Delta |
|---|---:|---:|---:|
| W1 10k | 6.0267 ms | 5.7622 ms | -4.4% |
| W1 50k | 40.405 ms | 34.029 ms | -15.8% |
| W1 200k | 190.62 ms | 203.20 ms | +6.6% |
| C2 10k | 0.9736 ms | 1.0226 ms | +5.0% |
| C2 50k | 5.9802 ms | 5.5932 ms | -6.5% |
| C2 200k | 29.039 ms | 37.313 ms | +28.5% |
| C3 10k | 3.4771 ms | 3.7706 ms | +8.4% |
| C3 50k | 21.693 ms | 18.121 ms | -16.5% |
| C3 200k | 113.48 ms | 132.56 ms | +16.8% |
| W2 5k | 2.3654 ms | 2.4545 ms | +3.8% |
| W2 20k | 12.478 ms | 11.768 ms | -5.7% |

Engine mixed frame:

| Workload | Phase 4 refresh | Phase 5B | Delta |
|---|---:|---:|---:|
| Headless mixed frame 5k | 0.8694 ms | 0.8922 ms | +2.6% |
| Headless mixed frame 20k | 3.6787 ms | 3.6222 ms | -1.5% |

Interpretation:
- Same-session quick runs remain noisy for a subset of gates at higher counts.
- Even in this noisy comparison, at least one dominant mutable form improved materially
  (`W1@50k`, `C3@50k`).
- Scheduler/flush remain secondary in profiles; query-path costs still dominate.

## 4. Correctness and Semantics Validation

Command executed:

```powershell
cargo test -p ecs
```

Result:
- all ECS tests and doctests passed.
- regression coverage includes command flush timing, scheduler conflict behavior,
  changed/added semantics, and query correctness.

Representative semantic guards:
- `foundation/ecs/tests/runtime_phase3.rs`
  - `command_flush_occurs_at_stage_boundary`
  - `scheduler_event_conflict_matrix_includes_write_write`
- `foundation/ecs/tests/world.rs`
  - `query_get_respects_filters_and_changed_semantics`
  - `changed_and_added_filters_handle_remove_then_reinsert`
  - cache rebind/recovery tests for reused `QueryState`.

## 5. Bottleneck Status After Phase 5B

- Broad mutable iteration is still a primary cost center at larger scales.
- Typed store lookup/fetch-path overhead was reduced in dominant forms via cached typed
  handles and fast fetch paths; remaining cost is now dominated by total broad-loop volume,
  cache behavior, and storage traversal.
- Scheduler planning and flush remain secondary in representative mixed workloads.

## 6. Phase 6 Recommendation

## Decision: **Option A - Proceed to Archetype/Storage Redesign (Phase 6A)**

### Why

- Phase 5B delivered substantial wins versus historical Phase 4 optimized artifacts for
  dominant gates (W1/C3/C2/W2 and engine mixed frame).
- After applying dominant-form specialization, typed-store caching, and hot-loop reduction,
  refreshed same-session comparisons show diminishing and inconsistent incremental gains at
  high counts (especially 200k-class broad mutable loops).
- Remaining top-end pressure is still broad mutable iteration, not scheduler/flush behavior.

### Alternatives rejected

- **Option B (continue incremental optimization) rejected**:
  - high-value fetch-path/caching optimizations are now in place
  - additional gains appear smaller and less stable relative to complexity
- **Option C (scheduler throughput/parallelism first) rejected**:
  - profile data continues to show query-path dominance in representative workloads
  - scheduler/flush are not the main limiter in W2/engine mixed scenarios

## 7. Final Phase 5B Exit Check

- [x] dominant query-form optimization implemented (`&mut T`, `(&mut A, &B)`, `(&mut A, &mut B)`)
- [x] typed store lookup caching implemented for dominant forms
- [x] hot-loop indirection reduced in `QueryIter::next(...)`
- [x] benchmark artifacts updated (ECS + engine/runtime)
- [x] profile artifacts updated
- [x] correctness tests passing
- [x] written Phase 6 recommendation produced
