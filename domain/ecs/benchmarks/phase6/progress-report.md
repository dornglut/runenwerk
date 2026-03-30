# Phase 6 Progress Report

Roadmap source of truth: [`/foundation/ecs/docs/roadmaps/phase6-closeout-roadmap.md`](../../roadmaps/phase6-closeout-roadmap.md)

Run date: 2026-03-12

## Reading guide

- Criterion quick bench tables (`## 2`, `## 3`) are median benchmark times from `cargo bench` output.
- Setup/run splits (`setup_time_ms`, `run_time_ms`) are from the profiling example output (`phase6_profile.txt`).
- These are different measurement modes and should not be compared as if they were identical timing pipelines.
- Telemetry-derived buckets (`query_total_ms`, `filter_total_ms`, `runtime_total_ms`, `event_total_ms`) overlap by design and should not be summed directly against wall time.

## 1. Command Execution Status

Required package commands were executed successfully in this rerun:

- `cargo test -p ecs`
- `cargo bench -p ecs --bench phase6 --features telemetry -- --quick`
- `cargo run -p ecs --example phase6_profile --features telemetry --release`
- `cargo bench -p engine --bench phase6_runtime -- --quick`

Comparison refresh commands were not rerun in this pass; Phase 5B refresh values below are reused from existing Phase 6 artifacts.

## 2. ECS Quick Bench Summary (Median)

Values are median times from Criterion output.

| Workload | Phase 5B historical | Phase 5B refresh | Phase 6 | Delta vs 5B historical | Delta vs 5B refresh |
|---|---:|---:|---:|---:|---:|
| W1 10k | 5.7622 ms | 7.2879 ms | 6.9539 ms | +20.68% | -4.58% |
| W1 50k | 34.029 ms | 39.478 ms | 40.976 ms | +20.41% | +3.79% |
| W1 200k | 203.20 ms | 198.21 ms | 217.95 ms | +7.26% | +9.96% |
| C2 10k | 1.0226 ms | 1.7632 ms | 1.7548 ms | +71.60% | -0.48% |
| C2 50k | 5.5932 ms | 12.024 ms | 9.8093 ms | +75.38% | -18.42% |
| C2 200k | 37.313 ms | 92.085 ms | 74.790 ms | +100.44% | -18.78% |
| C3 10k | 3.7706 ms | 5.1523 ms | 5.1804 ms | +37.39% | +0.55% |
| C3 50k | 18.121 ms | 34.384 ms | 31.797 ms | +75.47% | -7.52% |
| C3 200k | 132.56 ms | 174.54 ms | 193.12 ms | +45.68% | +10.65% |
| C4 10k | 9.5635 ms | 10.333 ms | 10.591 ms | +10.74% | +2.50% |
| C4 50k | 48.897 ms | 59.570 ms | 56.460 ms | +15.47% | -5.22% |
| C4 200k | 348.37 ms | 284.46 ms | 292.14 ms | -16.14% | +2.70% |
| W2 5k | 2.4545 ms | 2.5320 ms | 2.8390 ms | +15.67% | +12.12% |
| W2 20k | 11.768 ms | 10.987 ms | 12.417 ms | +5.51% | +13.02% |
| W3 churn | 255.31 us | 537.59 us | 608.82 us | +138.46% | +13.25% |
| W4 event | 263.69 us | 428.40 us | 451.72 us | +71.31% | +5.44% |
| W5 plan | 675.84 us | 798.57 us | 802.48 us | +18.74% | +0.49% |
| W5 stage | 164.41 us | 153.10 us | 170.34 us | +3.61% | +11.26% |

## 3. Engine Quick Bench Summary (Median)

| Workload | Phase 5B historical | Phase 5B refresh | Phase 6 | Delta vs 5B historical | Delta vs 5B refresh |
|---|---:|---:|---:|---:|---:|
| Headless mixed frame 5k | 892.24 us | 743.63 us | 756.98 us | -15.16% | +1.80% |
| Headless mixed frame 20k | 3.6222 ms | 3.4743 ms | 3.1871 ms | -12.01% | -8.27% |

## 4. Telemetry Summary (Required Metrics)

Phase 6 values are from `phase6_profile.txt`.

### Cumulative snapshot (all workloads)

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

Scheduler cumulative snapshot:

- `plan_build_calls`: 4
- `plan_build_nanos`: 1,139,000
- `plan_conflict_checks`: 5,704
- `plan_stage_count`: 70

### W2 mixed/composite gameplay schedule (20k x 20 runs)

- `run_time_ms`: 232.419
- `query_matching_nanos`: 20,061,700
- `query_iter_nanos`: 98,336,200
- `changed_check_nanos`: 29,779,800
- `added_check_nanos`: 27,808,000
- `runtime_plan_nanos`: 8,261,300
- `runtime_stage_nanos`: 224,084,200
- `runtime_flush_nanos`: 5,500

### Per-workload profile metadata and derived summary

Values are taken from `phase6_profile.txt` workload blocks.

| Workload | setup_time_ms | run_time_ms | entity_count | repetition_count | schedule_run_count | query_total_ms | filter_total_ms | runtime_total_ms | event_total_ms |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| W1 broad transform update (50k x 8) | 68.897 | 88.325 | 50000 | 8 | 0 | 0.974 | 0.000 | 0.000 | 0.000 |
| C2 broad no-filter read query (50k x 8) | 77.928 | 23.873 | 50000 | 8 | 0 | 1.362 | 0.000 | 0.000 | 0.000 |
| C3 broad simple write query (50k x 8) | 66.775 | 75.560 | 50000 | 8 | 0 | 0.971 | 0.000 | 0.000 | 0.000 |
| C4 broad double mutable query (50k x 8) | 70.445 | 127.030 | 50000 | 8 | 0 | 1.379 | 0.000 | 0.000 | 0.000 |
| W2 mixed/composite gameplay schedule (20k x 20 runs) | 69.473 | 232.419 | 20000 | 20 | 20 | 118.398 | 54.823 | 232.351 | 0.000 |
| W3 structural churn schedule (20k base, 128 churn x 20 runs) | 35.364 | 11.452 | 20000 | 20 | 20 | 10.195 | 0.000 | 11.448 | 0.000 |
| W4 event-heavy schedule (10k entities, 256 events x 20 runs) | 12.895 | 9.909 | 10000 | 20 | 20 | 0.943 | 0.000 | 9.899 | 0.204 |
| W5 scheduler stress schedule (16 registrations x 40 runs) | 1.407 | 11.644 | 0 | 40 | 40 | 0.000 | 0.000 | 8.752 | 0.000 |

## 5. Workload-class readout

- Broad-loop query workloads (`W1`, `C2`, `C3`) are mixed vs refresh: `C2` improved at 50k/200k while `W1` and `C3` remain regression-sensitive at larger sizes.
- Dominant archetype-native tuple workload (`C4`) keeps the strongest structural win at large size versus historical (`C4@200k: -16.14%`) with near-refresh parity (`+2.70%`).
- Mixed/composite schedule (`W2`) remains above refresh (`+13.02%` at 20k) while preserving expected query/filter/runtime behavior.
- Churn/event-heavy workloads (`W3`, `W4`) remain known regression areas versus both historical and refresh baselines.
- Scheduler stress (`W5`) is close to refresh on plan path and slightly above refresh on stage path in this rerun.

## 6. Preliminary Interpretation

### Fast scan

- Strongest win: `C4@200k` remains materially better than historical (`-16.14%`), though slightly above refresh (`+2.70%`).
- Mixed zone: broad-loop query forms (`W1`, `C2`, `C3`) do not move uniformly; some refresh-relative improvements exist alongside regressions.
- Known regression cluster: `W3` and `W4` remain slower versus both historical and refresh baselines; these are documented tradeoffs.
- Runtime signal: engine 20k mixed-frame improved versus both historical and refresh in this rerun.
- Measurement note: `query_get_nanos` and `query_single_nanos` are zero because these workloads are iter-path focused, and that is explicitly reported per workload block.

### Closeout context

- Phase closeout status is unchanged: this report updates readability and interpretation only.
- The authoritative closeout decision and rationale remain in [`final-decision-report.md`](final-decision-report.md).
