# ECS Phase 3.5 Decision Report

## 1. Scope

Phase 3.5 goals completed in this report:

- benchmark harness for W1-W5 plus engine/runtime scenario
- runtime/scheduler/query/filter instrumentation
- low-risk query/filter optimizations
- before/after measurement comparison
- recommendation for Phase 4 path

## 2. Implemented Optimizations

### Query path (low-risk)

- Candidate narrowing from smallest required component store in `World::matching_entities`.
  - File: `foundation/ecs/src/world/world_internal_impl.rs`
  - Symbol: `World::matching_entities`
- Direct per-entity filter/constraint check for `QueryState::get` (no full match-set rebuild).
  - File: `foundation/ecs/src/query/traits_and_state.rs`
  - Symbol: `QueryState::matches_entity`

### Change-filter path (low-risk)

- Added direct tick maps for `Changed<T>` / `Added<T>` checks.
  - Files:
    - `foundation/ecs/src/world/world_struct.rs`
    - `foundation/ecs/src/world/world_core_impl.rs`
    - `foundation/ecs/src/world/world_internal_impl.rs`
  - Symbols:
    - `World::component_entity_last_changed_ticks`
    - `World::component_entity_last_added_ticks`
    - `World::component_changed_for_entity_since`
    - `World::component_added_for_entity_since`
    - `World::record_component_change`

### Instrumentation

- ECS telemetry module and hot-path records.
  - Files:
    - `foundation/ecs/src/telemetry.rs`
    - `foundation/ecs/src/world/world_internal_impl.rs`
    - `foundation/ecs/src/query/traits_and_state.rs`
    - `foundation/ecs/src/system/runtime.rs`
    - `foundation/ecs/src/system/params.rs`
- Scheduler telemetry module and plan-build metrics.
  - Files:
    - `foundation/scheduler/src/telemetry.rs`
    - `foundation/scheduler/src/plan.rs`

## 3. Baseline vs Optimized Results

Sources:

- `baseline_ecs_bench.txt`, `optimized_ecs_bench.txt`
- `baseline_profile.txt`, `optimized_profile.txt`
- `baseline_engine_bench.txt`, `optimized_engine_bench.txt`

### ECS Bench (Criterion quick, representative points)

| Workload | Baseline | Optimized | Delta |
|---|---:|---:|---:|
| W1 entities=10k | 11.446 ms | 11.684 ms | +2.1% |
| W1 entities=50k | 71.408 ms | 72.927 ms | +2.1% |
| W1 entities=200k | 292.62 ms | 337.72 ms | +15.4% |
| W2 entities=5k | 526.77 ms | 4.533 ms | -99.1% |
| W2 entities=20k | 2.6705 s | 18.327 ms | -99.3% |
| W3 structural churn | 1.3891 ms | 1.0131 ms | -27.1% |
| W4 event-heavy | 1.0458 ms | 0.7607 ms | -27.3% |
| W5 plan build | 1.0420 ms | 0.7431 ms | -28.7% |
| W5 stage execute | 225.85 us | 208.12 us | -7.9% |

### Scenario latency bands (quick-mode p50/p95 approximation)

Criterion quick output reports a three-point band `[low mid high]`. For Phase 3.5 scenario tracking:

- `p50 ~= mid`
- `p95 ~= high`

| Scenario | Baseline p50 | Baseline p95 | Optimized p50 | Optimized p95 |
|---|---:|---:|---:|---:|
| W2 entities=5k | 526.77 ms | 581.66 ms | 4.5331 ms | 4.5343 ms |
| W2 entities=20k | 2670.5 ms | 2897.0 ms | 18.327 ms | 18.489 ms |
| W3 structural churn | 1.3891 ms | 1.3896 ms | 1.0131 ms | 1.0279 ms |
| W4 event-heavy | 1.0458 ms | 1.0836 ms | 0.7607 ms | 0.7752 ms |
| W5 plan build | 1.0420 ms | 1.0554 ms | 0.7431 ms | 0.7460 ms |
| W5 stage execute | 225.85 us | 227.96 us | 208.12 us | 214.35 us |
| Engine frame 5k | 1.6047 ms | 1.6219 ms | 1.7155 ms | 1.7307 ms |
| Engine frame 20k | 6.9558 ms | 6.9774 ms | 6.8758 ms | 7.0992 ms |

### Engine Runtime Scenario

| Workload | Baseline | Optimized | Delta |
|---|---:|---:|---:|
| Headless mixed frame, 5k entities | 1.6047 ms | 1.7155 ms | +6.9% |
| Headless mixed frame, 20k entities | 6.9558 ms | 6.8758 ms | -1.1% |

## 4. Cost Attribution (Telemetry)

### Mixed Gameplay (W2) was the dominant baseline bottleneck

Baseline W2 (`phase35_profile`):

- `wall_time_ms`: `147281.744`
- `added_check_nanos`: `145152876100`
- `changed_check_nanos`: `1683522500`
- `query_iter_nanos`: `147093386300`
- `runtime_stage_nanos`: `147258226600`

Optimized W2:

- `wall_time_ms`: `453.836`
- `added_check_nanos`: `31580100`
- `changed_check_nanos`: `23377300`
- `query_iter_nanos`: `205367800`
- `runtime_stage_nanos`: `418818700`

Interpretation:

- `Added<T>` check cost dropped by ~99.98% (dominant bottleneck removed).
- `Changed<T>` check cost dropped by ~98.6%.
- stage/runtime execution dropped by ~99.7% for this workload.

### Scheduler/flush characterization (optimized)

From cumulative optimized telemetry:

- `runtime_plan_nanos`: `8281600`
- `runtime_stage_nanos`: `446458000`
- `runtime_flush_nanos`: `5013100`

Approximate share:

- plan: ~1.8%
- stage execution: ~97.1%
- flush: ~1.1%

Interpretation:

- scheduler plan build and stage-end flush are measurable but not dominant.
- cost is still mainly in stage execution (query/filter/system work), not planner overhead.

## 5. Semantics and Safety Validation

Semantics preserved by tests:

- query/filter correctness (`Changed<T>`, `Added<T>`, tuple/optional forms)
- deterministic command flush at stage boundary
- scheduler conflict and ordering behavior
- command owner and cached param state safety paths

Notable regression additions:

- `query_get_respects_filters_and_changed_semantics`
- `changed_and_added_filters_handle_remove_then_reinsert`

File: `foundation/ecs/tests/world.rs`

## 6. Recommendation (Phase 4)

### Recommended path: Option A - Defer archetypes, continue incremental optimization

Rationale:

- The largest real bottleneck (change-filter checks) was resolved with low-risk internal changes.
- Engine-level scenario timing did not indicate a storage-redesign emergency.
- Scheduler planning/flush costs are not currently dominant enough to justify scheduler-first redesign.

### Rejected alternatives

- Option B (archetype/storage redesign) rejected for now:
  - after Phase 3.5 optimizations, representative workloads do not show an immediate architectural cliff.
- Option C (scheduler parallelism first) rejected for now:
  - current telemetry shows planner/flush overhead is a small fraction of runtime.

## 7. Next Decision Gates

Escalate to archetype/storage redesign only if, after another incremental pass:

- representative engine workloads remain query/storage bound at target entity scales, and
- further low-risk query/filter improvements stop yielding meaningful wins.

Otherwise continue incremental optimization with measurement-first discipline.
