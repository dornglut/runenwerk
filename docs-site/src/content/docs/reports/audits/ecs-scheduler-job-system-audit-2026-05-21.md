---
title: ECS, Scheduler, and Job System Audit 2026-05-21
description: Code-grounded audit of ECS runtime, scheduler planning, multithreading, and runtime job-system design.
status: completed
owner: ecs
layer: domain/engine-runtime
last_reviewed: 2026-05-21
related:
  - ../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../design/active/ecs-parallel-system-execution-design.md
  - ../../domain/ecs/architecture.md
  - ../../domain/scheduler/README.md
  - ../../engine/roadmaps/runtime-product-job-executor-roadmap.md
  - ../../workspace/production-tracks.yaml
  - ../../workspace/roadmap-items.yaml
---

# ECS, Scheduler, and Job System Audit 2026-05-21

## Purpose

Map the current ECS, scheduler, multithreading, and runtime job-system design
against the codebase, then identify friction, gaps, future features,
ergonomics, architecture risks, redesign candidates, and refactors. This audit
is evidence for the ECS production track in
`docs-site/src/content/docs/workspace/production-tracks.yaml`.

## Architecture Decision

The current ownership split is sound and should be preserved:

- `domain/ecs` owns live ECS state, component/resource storage, queries, system
  params, deferred command descriptors, ECS runtime inspection, and ECS-facing
  diagnostics.
- `domain/scheduler` owns deterministic labels, dependency constraints, access
  conflict analysis, stages, waves, barriers, and plan diagnostics.
- `engine/src/runtime/jobs` owns worker threads, serial fallback, runtime job
  submission, panic capture, backpressure, generations, stale suppression, and
  product job staging.
- Engine runtime resources own product/query publication barriers. Product jobs
  must publish through explicit barriers, never by mutating live ECS truth from
  worker threads.

This matches
`docs-site/src/content/docs/design/accepted/execution-fabric-and-product-jobs-design.md`.
The active multithreaded path is runtime product jobs. ECS system execution
should remain serial until the parallel ECS design is accepted and has
serial-equivalence evidence.

## Current Implementation Map

| Area | Owning module/function | Current status |
| --- | --- | --- |
| ECS runtime execution | `domain/ecs/src/system/runtime.rs::Runtime`, `Runtime::run_schedule`, `Runtime::flush_stage_commands`, `query_access_to_system_access` | Serial execution over scheduler waves with deterministic deferred command flushes and barrier handler dispatch. |
| ECS plan reporting | `domain/ecs/src/system/plan_report.rs::RuntimePlanReport`, `Runtime::plan_report_for` | ECS-owned reporting exposes waves, barriers, conflicts, diagnostics, and product/query publication barriers. |
| Scheduler planning | `domain/scheduler/src/plan.rs::ExecutionScheduler`, `ExecutionScheduler::build_plan`, `ExecutionScheduler::run_schedule` | Deterministic graph planning computes stages/waves, conflicts, diagnostics, and automatic barriers; execution remains serial. |
| Scheduler access model | `domain/scheduler/src/access.rs::AccessKey`, `SystemAccess::conflicts_with` | Component/resource/broadcast/workqueue/tickbuffer/structural access conflicts are typed and diagnostic-friendly. Structural writes intentionally do not conflict because deferred commands merge at barriers. |
| Runtime frame lifecycle | `engine/src/runtime/frame_lifecycle.rs::run_runtime_frame` | Canonical frame execution finalizes the world after `FrameEnd`, with fixed-step tick finalization handled in `engine/src/runtime/fixed_step_executor.rs::run_fixed_steps`. |
| Product publication barriers | `engine/src/runtime/product_publication.rs::handle_product_publication_barrier` | Product outcomes publish only at `BarrierKind::ProductPublication`, sorted deterministically by stage and job identity. |
| Query snapshot barriers | `engine/src/runtime/query_snapshot.rs::handle_query_snapshot_publication_barrier` | Query snapshots publish only at `BarrierKind::QuerySnapshotPublication`, with strict/relaxed consumption diagnostics. |
| Runtime job executor | `engine/src/runtime/jobs/executor.rs::RuntimeJobExecutorResource`, `RuntimeJobExecutorResource::submit`, `RuntimeJobExecutorResource::drain_completed`, `RuntimeJobExecutorResource::diagnostics` | Serial, worker-pool, and work-stealing backends support generations, stale suppression, panic capture, bounded queues, and diagnostics. |
| Product job staging | `engine/src/runtime/jobs/product.rs::stage_completed_product_jobs` | Completed runtime jobs are sorted deterministically and staged into product publication and query snapshot resources. |
| Draw app product jobs | `apps/runenwerk_draw/src/runtime/plugin.rs::DrawingRuntimePlugin::build`, `apps/runenwerk_draw/src/runtime/systems.rs::process_draw_preview_ink_jobs_system` | Draw installs a worker-backed executor and uses runtime product jobs as the active multithreaded path. |
| World build jobs | `engine/src/plugins/world/build/jobs.rs::dispatch_world_build_jobs_system` | World build work is still synchronous ECS-system work, not unified under the runtime job executor. |

## Findings

| Priority | Finding | Evidence | Long-term direction |
| --- | --- | --- | --- |
| P1 | ECS parallel execution is not implementation-ready. | `docs-site/src/content/docs/design/active/ecs-parallel-system-execution-design.md` is active, not accepted. `domain/ecs/src/system/runtime.rs::Runtime::run_schedule` executes waves serially and uses `Rc<RefCell<_>>` deferred command buffers. | Keep runtime product jobs as the multithreaded path. Require accepted design/ADR, `Send + Sync` system-param contracts, per-wave command queues, deterministic merge, blocked-parallel diagnostics, and serial/parallel equivalence tests before public ECS parallel APIs. |
| P1 | Result ergonomics for scheduler plan inspection are too weak for tooling. | `domain/scheduler/src/plan.rs::ExecutionScheduler::plans` panics on rebuild failure, while `ExecutionScheduler::plan_for` returns `Option` and can hide rebuild errors. | Add `try_plans` and `try_plan_for` returning `Result`, migrate ECS and docs tooling to those APIs, then keep panic/option paths only as convenience wrappers. |
| P1 | Runtime lifecycle convergence still has a manual finalization path. | `engine/src/runtime/frame_lifecycle.rs::run_runtime_frame` owns canonical finalization, but `engine/src/plugins/scene/lifecycle/overlay_update.rs::update_scene_overlay_runtime_system` still calls `overlay_runtime.world.finalize_frame_boundary()`. | Decide whether overlay runtimes are intentionally nested independent worlds. If not, route them through the runtime lifecycle wrapper so finalization evidence is centralized. |
| P2 | Scheduler access identity is type-only for non-structural domains. | `domain/scheduler/src/access.rs::AccessKey::eq` ignores diagnostic `name` for component/resource/broadcast/workqueue/tickbuffer keys. | Introduce stable lane/source identity for multiple resources, streams, queues, or tick buffers of the same Rust type before richer job families need parallel lanes. |
| P2 | Automatic per-wave product/query barriers are safe but coarse. | `domain/scheduler/src/plan.rs::ExecutionScheduler::build_plan` inserts `ApplyDeferredCommands`, `ProductPublication`, and `QuerySnapshotPublication` barriers after every wave. | Add an explicit barrier policy per schedule/phase after product usage patterns stabilize. The default should preserve current serial-compatible safety. |
| P2 | The scheduler exposes no frame-level plan. | `domain/scheduler/src/plan.rs::ExecutionPhase::from_schedule_label` builds phase index `0` for each schedule plan. | Add a runtime-level frame plan report that composes `PreUpdate`, fixed steps, `Update`, render preparation, render submit, and frame end with global barriers. |
| P2 | Worker thread construction can panic. | `engine/src/runtime/jobs/executor.rs::WorkerPool::new` and `WorkStealingPool::new` use `expect` after thread spawn. | Return installation diagnostics or fall back to serial execution when worker thread creation fails. |
| P2 | World build jobs are not yet on the runtime job substrate. | `engine/src/plugins/world/build/jobs.rs::dispatch_world_build_jobs_system` builds jobs synchronously inside an ECS system. | Migrate world build, import, and field product workloads to `RuntimeJob` only after their product descriptors and barrier publication contracts are accepted. |
| P2 | Older ECS audit docs now conflict with current code. | `docs-site/src/content/docs/net/ecs-runtime-gap-summary.md` and `docs-site/src/content/docs/net/ecs-runtime-feature-inventory.md` predate stable runtime plan reporting, stable system identity, and named param diagnostics. | Refresh or archive stale audit pages so production planning does not reuse old gap states as current truth. |
| P3 | ECS architecture docs understate system arity. | `docs-site/src/content/docs/domain/ecs/architecture.md` says function system arity is 8, while `domain/ecs/src/system/runtime.rs::supports_max_function_system_arity_sixteen` verifies 16. | Update the architecture page and clarify the difference between function-system arity and grouped tuple-param ergonomics. |
| P3 | Scheduler module shape has a catch-all utility module. | `domain/scheduler/src/lib.rs` exports `utils`, and `domain/scheduler/src/utils.rs::export_dag_dot` violates the module-structure guideline against `utils.rs`. | Move DAG export into a named inspection/export module and keep the public export intentional. |
| P3 | Query snapshot source generation ignores messaging accesses. | `domain/ecs/src/query/snapshot.rs::query_snapshot_source_generation` computes source generation from component/resource changes only. | Keep this if snapshots are strictly ECS state snapshots. Extend it before query products depend on work queues, tick buffers, or broadcast streams. |

## Ergonomics Gaps

| Gap | Owning location | Why it matters |
| --- | --- | --- |
| Product-job authoring is scattered across engine and app code. | `engine/src/runtime/jobs/*`, `apps/runenwerk_draw/src/runtime/*` | Common ECS-to-runtime-job helpers would make the preferred path easier without moving worker ownership into `domain/ecs`. |
| Work queue and tick-buffer APIs are low-level. | `domain/ecs/src/system/params.rs`, `domain/ecs/src/world/messaging/*` | The primitives are useful, but common product/job workflows need policy names, diagnostics, and examples. |
| Conditional deferred command composition is still basic. | `domain/ecs/src/commands/*` | A command DSL can reduce boilerplate, but it must preserve deterministic ordering and failure isolation. |
| Event history/replay is not first-class. | `domain/ecs/src/world/messaging/*`, `domain/ecs/src/query/snapshot.rs` | Runtime/product debugging will need replayable event and snapshot provenance before complex gameplay and editor automation. |
| Public API discovery still depends heavily on docs. | `domain/ecs/src/lib.rs`, `domain/ecs/src/prelude.rs`, `docs-site/src/content/docs/domain/ecs/usage-guide.md` | The common happy path should make system registration, plan inspection, product barriers, and runtime jobs easy to find from imports and examples. |

## Future Feature Map

| Feature | Product-track phase | Required gates |
| --- | --- | --- |
| Result-returning scheduler inspection API | Runtime convergence and diagnostics | Regression tests for cycle/config errors, ECS plan-report migration, docs update. |
| Frame-level execution plan report | Scheduler plan ergonomics | Runtime lifecycle tests covering fixed-step zero/batched ticks and barrier order. |
| Explicit barrier policy | Scheduler plan ergonomics | Serial-compatible default, product/query publication tests, diagnostics for missing required barriers. |
| Stable access lanes | Scheduler plan ergonomics | Conflict tests for same-type distinct lanes and same-lane conflicts. |
| Runtime job executor hardening | Product job substrate hardening | Thread spawn failure path, serial fallback or diagnostic evidence, backpressure diagnostics remain stable. |
| World build/import product jobs | Product job substrate hardening | Accepted product descriptors, barrier publication, stale-generation behavior, deterministic staging tests. |
| ECS parallel waves | Parallel execution readiness and implementation | Accepted design/ADR, `Send + Sync` contracts, thread-safe per-wave command queues, deterministic merge, serial/parallel equivalence suite. |
| Event history and replay | ECS API ergonomics | Explicit retention policy, replay determinism tests, diagnostics and docs. |

## Redesign And Refactor Map

| Candidate | Exact location | Scope |
| --- | --- | --- |
| Scheduler inspection API redesign | `domain/scheduler/src/plan.rs::ExecutionScheduler::plans`, `ExecutionScheduler::plan_for`; `domain/ecs/src/system/runtime.rs::Runtime::plan_for`, `Runtime::plan_report_for` | Narrow API redesign from panic/option inspection to `Result` inspection. Preserve existing wrappers while migrating internal users. |
| Scheduler inspection module cleanup | `domain/scheduler/src/utils.rs::export_dag_dot`, `domain/scheduler/src/lib.rs` | Move `export_dag_dot` into `inspection` or `export` module to satisfy module-structure rules. |
| Lifecycle finalization ownership | `engine/src/runtime/frame_lifecycle.rs::run_runtime_frame`, `engine/src/plugins/scene/lifecycle/overlay_update.rs::update_scene_overlay_runtime_system` | Decide whether overlay runtime is a standalone runtime. If not, remove direct world-finalization ownership from plugin code. |
| Barrier policy model | `domain/scheduler/src/plan.rs::ExecutionScheduler::build_plan`, `engine/src/runtime/product_publication.rs`, `engine/src/runtime/query_snapshot.rs` | Add opt-in schedule/phase policy without weakening current default barriers. |
| Runtime job construction hardening | `engine/src/runtime/jobs/executor.rs::WorkerPool::new`, `WorkStealingPool::new`, `RuntimeJobExecutorResource::new` | Return fallible construction diagnostics or serial fallback instead of panics. |
| Product job helper layer | `engine/src/runtime/jobs/product.rs`, `apps/runenwerk_draw/src/runtime/systems.rs` | Create ergonomic helpers for common product job submission/staging patterns while keeping product descriptors in owning domains. |

## Validation Baseline

Existing tests and evidence that should remain fitness functions:

- `domain/scheduler/tests/scheduler.rs::scheduler_records_conflicts_but_stays_serial`
- `domain/scheduler/tests/scheduler.rs::scheduler_assigns_monotonic_ids_and_surfaces_them_in_plans`
- `domain/ecs/tests/runtime_phase3.rs::runtime_plan_report_exposes_system_slots_and_product_barriers`
- `domain/ecs/tests/runtime_phase3.rs::runtime_plan_report_exposes_conflict_diagnostics_with_access_labels`
- `domain/ecs/tests/runtime_phase3.rs::registered_product_publication_barrier_handler_runs_after_each_wave`
- `domain/ecs/tests/runtime_phase3.rs::failed_schedule_drops_stage_deferred_commands_instead_of_replaying_next_run`
- `engine/tests/runtime_app.rs::fixed_step_schedule_supports_zero_and_batched_ticks_per_frame`
- `engine/tests/runtime_app.rs::runtime_finalization_counts_match_fixed_tick_runs`
- `engine/tests/runtime_app.rs::runtime_finalization_skips_tick_boundary_when_no_fixed_steps_run`
- `engine/src/runtime/jobs/executor.rs` unit tests for serial, worker-pool, work-stealing, panic capture, stale suppression, and backpressure.
- `engine/src/runtime/jobs/product.rs` unit tests for deterministic serial/worker/work-stealing product staging.
- `apps/runenwerk_draw/tests/app_shell.rs::drawing_runtime_uses_worker_backed_jobs_by_default`

## Production Track Recommendation

Create `PT-ECS-FABRIC` as a cross-domain production track for the ECS execution
fabric. The track should not be a request to parallelize ECS immediately. It
should sequence:

1. Audit and baseline ownership.
2. Runtime convergence and diagnostic cleanup.
3. Product job substrate hardening.
4. Scheduler plan ergonomics and barrier policy.
5. ECS parallel execution readiness.
6. ECS parallel execution implementation only after the design is accepted.

The first implementation priority should be scheduler/runtime ergonomics and
diagnostics under `WR-002`. `WR-023` should remain design-first until the
parallel ECS safety contract is accepted.
