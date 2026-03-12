# Phase 6 Progress Report

Roadmap source of truth: [`/foundation/ecs/docs/roadmaps/phase6-closeout-roadmap.md`](../../roadmaps/phase6-closeout-roadmap.md)

Run date: 2026-03-12

## 1. Command Execution Status

Required package commands were executed successfully:

- `cargo test -p ecs`
- `cargo bench -p ecs --bench phase6 --features telemetry -- --quick`
- `cargo run -p ecs --example phase6_profile --features telemetry --release`
- `cargo bench -p engine --bench phase6_runtime -- --quick`

Comparison refreshes were also executed in the same session:

- `cargo bench -p ecs --bench phase5b --features telemetry -- --quick`
- `cargo run -p ecs --example phase5b_profile --features telemetry --release`
- `cargo bench -p engine --bench phase5b_runtime -- --quick`

## 2. ECS Quick Bench Summary (Median)

Values are median times from Criterion output.

| Workload | Phase 5B historical | Phase 5B refresh | Phase 6 | Delta vs 5B historical | Delta vs 5B refresh |
|---|---:|---:|---:|---:|---:|
| W1 10k | 5.7622 ms | 7.2879 ms | 6.6239 ms | +14.95% | -9.11% |
| W1 50k | 34.029 ms | 39.478 ms | 45.100 ms | +32.53% | +14.24% |
| W1 200k | 203.20 ms | 198.21 ms | 200.99 ms | -1.09% | +1.40% |
| C2 10k | 1.0226 ms | 1.7632 ms | 1.6782 ms | +64.11% | -4.82% |
| C2 50k | 5.5932 ms | 12.024 ms | 9.8596 ms | +76.29% | -17.99% |
| C2 200k | 37.313 ms | 92.085 ms | 104.96 ms | +181.31% | +13.98% |
| C3 10k | 3.7706 ms | 5.1523 ms | 4.9299 ms | +30.75% | -4.32% |
| C3 50k | 18.121 ms | 34.384 ms | 28.994 ms | +59.99% | -15.68% |
| C3 200k | 132.56 ms | 174.54 ms | 173.06 ms | +30.55% | -0.85% |
| C4 10k | 9.5635 ms | 10.333 ms | 9.4910 ms | -0.76% | -8.15% |
| C4 50k | 48.897 ms | 59.570 ms | 56.627 ms | +15.81% | -4.94% |
| C4 200k | 348.37 ms | 284.46 ms | 290.51 ms | -16.61% | +2.13% |
| W2 5k | 2.4545 ms | 2.5320 ms | 2.6341 ms | +7.32% | +4.03% |
| W2 20k | 11.768 ms | 10.987 ms | 12.721 ms | +8.10% | +15.78% |
| W3 churn | 255.31 us | 537.59 us | 593.66 us | +132.53% | +10.43% |
| W4 event | 263.69 us | 428.40 us | 537.30 us | +103.76% | +25.42% |
| W5 plan | 675.84 us | 798.57 us | 706.29 us | +4.51% | -11.56% |
| W5 stage | 164.41 us | 153.10 us | 162.14 us | -1.38% | +5.90% |

## 3. Engine Quick Bench Summary (Median)

| Workload | Phase 5B historical | Phase 5B refresh | Phase 6 | Delta vs 5B historical | Delta vs 5B refresh |
|---|---:|---:|---:|---:|---:|
| Headless mixed frame 5k | 892.24 us | 743.63 us | 759.43 us | -14.89% | +2.12% |
| Headless mixed frame 20k | 3.6222 ms | 3.4743 ms | 3.4826 ms | -3.85% | +0.24% |

## 4. Telemetry Summary (Required Metrics)

Phase 6 values are from `phase6_profile.txt`.

### Cumulative snapshot (all workloads)

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

### W2 gameplay mixed (query/filter/runtime-heavy)

- `wall_time_ms`: 310.587
- `query_matching_nanos`: 19,753,700
- `query_iter_nanos`: 98,438,000
- `changed_check_nanos`: 26,700,800
- `added_check_nanos`: 27,022,800
- `runtime_plan_nanos`: 55,300
- `runtime_stage_nanos`: 240,613,700
- `runtime_flush_nanos`: 13,100

## 5. Preliminary Interpretation

- Dominant mutable archetype-native tuple form (`C4`) remained competitive and improved on the largest size versus historical Phase 5B (`-16.61%` at 200k).
- Same-session refresh comparison shows mixed outcomes across W1/C2/C3/W2 and event/churn workloads.
- Telemetry showed lower `query_matching_nanos` and `query_iter_nanos` than historical Phase 5B cumulative snapshots, while runtime planning/flush remained secondary in absolute terms.
- Event/churn and some broad-loop medians remain regression-sensitive under quick-mode variability; this is captured in the decision report.
