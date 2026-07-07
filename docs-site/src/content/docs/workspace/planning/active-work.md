---
title: Active Work
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../evidence-quality-taxonomy.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../guidelines/programming-principles.md
  - ../../architecture/ui-framework-architecture.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-005-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-008-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md
  - ../../reports/closeouts/pt-ui-runtime-platform-010-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-011`

Title: `Scene/Debug Overlay Producer Migration and Retirement`

State: active implementation authorization for exactly one bounded Phase 011 implementation PR after this authorization record merges.

Lifecycle state: `active-implementation` for Phase 011 only.

Owner: scene and debug producer owners publish their own render-facing UI frame submissions through the producer-generic surface-frame seam. RenderPlugin owns generic preparation and submission only; it must stop owning scene/debug UI semantic producer collection.

Authority files:

```text
AGENTS.md
ARCHITECTURE.md
DOMAIN_MAP.md
TESTING.md
docs-site/src/content/docs/workspace/start-here.md
docs-site/src/content/docs/workspace/operating-model.md
docs-site/src/content/docs/workspace/authority-model.md
docs-site/src/content/docs/workspace/workflow-lifecycle.md
docs-site/src/content/docs/workspace/complete-investigation-gate.md
docs-site/src/content/docs/workspace/complete-design-gate.md
docs-site/src/content/docs/workspace/evidence-quality-taxonomy.md
docs-site/src/content/docs/workspace/complete-merge-readiness-gate.md
docs-site/src/content/docs/workspace/routines/implementation-routine.md
docs-site/src/content/docs/workspace/routines/pr-review-routine.md
docs-site/src/content/docs/workspace/routines/phase-completion-drift-check-routine.md
docs-site/src/content/docs/workspace/routines/roadmap-update-routine.md
docs-site/src/content/docs/workspace/planning/README.md
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/completed-work.md
docs-site/src/content/docs/workspace/planning/decision-register.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-009-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-010-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 010, `E6` PR #101 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` Phase 010 code/test plus validation plus authority alignment from the closeout report.

Complete investigation gate: complete for Phase 011 implementation authorization. Current source inspection named the hardcoded scene/debug producer collection in `engine/src/plugins/render/runtime/ui_submission.rs`, its RenderPlugin schedule/export points, the scene overlay frame producer state, the debug metrics frame producer state, the generic registry, and focused tests that currently cover overlay frame construction and render preparation.

Complete design gate: complete for Phase 011 implementation authorization through the accepted cutover plan, live runtime architecture, Phase 010 closeout evidence, this active-work contract, the Phase 011 phase spec, principle compliance matrix, module decomposition map, validation envelope, evidence expectations, and stop conditions.

Implementation authorization status: authorized after this activation record merges. The next implementation PR must stay inside this Phase 011 contract and must not include Phase 012 or later work.

Phase spec: `docs-site/src/content/docs/workspace/specs/pt-ui-runtime-platform-011.ron`.

Phase 010 completion truth:

```text
PR #101 merged into main at 8d6c13146deab870dca5533204067249aa2c1b90.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-010-closeout.md.
```

Phase 011 source/path inventory:

```text
engine/src/plugins/render/runtime/ui_submission.rs:
  SCENE_OVERLAY_UI_PRODUCER_ID = RenderFrameProducerId(1)
  DEBUG_METRICS_UI_PRODUCER_ID = RenderFrameProducerId(2)
  collect_runtime_ui_frame_submissions_system directly reads SceneResource.manager.overlay_runtime.ui.frame
  collect_runtime_ui_frame_submissions_system directly reads UiOverlayState.debug_frame
  scene overlay uses route screen and order layer 0 / priority 0
  debug metrics overlay uses route screen and order layer 100 / priority 0
  empty frames remove their legacy producer submissions

engine/src/plugins/render/runtime/mod.rs:
  exports collect_runtime_ui_frame_submissions_system from render runtime.

engine/src/plugins/render/plugin.rs:
  initializes SurfaceFrameSubmissionRegistryResource
  schedules collect_runtime_ui_frame_submissions_system in RenderPrepare
  schedules prepare_ui_feature_resource_system before RenderRuntimeSet::FramePrepare

engine/src/plugins/scene/lifecycle/overlay_update.rs and engine/src/plugins/scene/runtime/overlay_ui.rs:
  scene_overlay_update_system rebuilds manager.overlay_runtime.ui.frame from Update before RenderPrepare
  scene overlay frame generation must remain scene-owned

engine/src/plugins/debug_metrics/mod.rs:
  debug_metrics_overlay_system builds UiOverlayState.debug_frame in RenderPrepare
  debug metrics frame generation must remain debug-owned

engine/src/plugins/render/features/ui/submission.rs and resource.rs:
  SurfaceFrameSubmissionRegistryResource and prepare_ui_feature_resource_system are the generic producer seam and must remain generic, not scene/debug/UI-runtime semantic owners.

engine/src/plugins/ui/render_publish.rs:
  UiPlugin publication already uses the same generic seam and must remain independent from scene/debug migration.
```

Phase 011 handoff contract:

```text
replace the scene overlay producer collection by a scene-owned generic SurfaceFrameSubmission publication path
replace the debug metrics overlay producer collection by a debug-owned generic SurfaceFrameSubmission publication path
remove RenderPlugin scheduling/import/export of collect_runtime_ui_frame_submissions_system
delete or fully retire engine/src/plugins/render/runtime/ui_submission.rs so RenderPlugin no longer owns UI semantic producer collection
preserve the existing producer ids, route, ordering, shader-id behavior, empty-frame removal behavior, and prepared UI contribution behavior unless a focused test proves intentional retirement
do not alter UiPlugin render publication, source/program/action semantics, host mutation, route policy, render backend behavior, graph execution, shader code, or Counter product scope
prove no public manual add_ui_* registration chain is introduced or remains as a compatibility escape hatch
```

Allowed files/crates:

```text
engine/src/plugins/render/runtime/ui_submission.rs
engine/src/plugins/render/runtime/mod.rs
engine/src/plugins/render/plugin.rs only to remove the render-owned legacy collection system import/schedule/export
engine/src/plugins/scene/plugin.rs only if scene-owned publication needs a RenderPrepare scheduling hook
engine/src/plugins/scene/lifecycle/overlay_update.rs only if scene-owned publication can be attached to existing scene overlay update without changing scene behavior
engine/src/plugins/scene/runtime/overlay_ui.rs only if scene-owned publication needs a narrow helper that preserves existing frame generation
engine/src/plugins/debug_metrics/mod.rs only for debug-owned publication and tests
engine/src/state.rs only if UiOverlayState debug-frame storage is intentionally retired with evidence; otherwise leave it unchanged
engine/tests/runtime_ui_producer_migration.rs or a similarly named focused Phase 011 engine integration test
engine/tests/runtime_surface_guard.rs only to guard that overlay producers still route through ui_runtime::build_ui_frame and RenderPlugin no longer owns semantic collection
engine/src/plugins/scene/tests/scene_tests.rs only for focused scene overlay behavior assertions if integration tests cannot prove them
```

Forbidden files/crates:

```text
apps/ui_counter_runtime product packaging
source reload/persistence implementation
SDF or SpatialCanvas implementation
engine/src/plugins/ui/** runtime implementation changes
engine/src/plugins/render/runtime/frame_prepare.rs
engine/src/plugins/render/runtime/frame_submit.rs
engine/src/plugins/render/renderer/**
engine/src/plugins/render/graph/**
engine/src/plugins/render/backend/**
engine/src/plugins/render/shader/**
render backend rewrite
graph execution rewrite or shader changes
source/program/action semantic changes
host mutation or action-dispatch behavior changes
broad ui_render_data primitive/model rewrites
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Acceptance criteria required before Phase 011 can close:

```text
every hardcoded scene/debug overlay UI producer path named in this active-work record is replaced or intentionally retired
ScenePlugin or scene-owned runtime code publishes the scene overlay frame through SurfaceFrameSubmissionRegistryResource without RenderPlugin querying SceneResource for UI semantics
DebugMetricsPlugin or debug-owned runtime code publishes/removes the debug metrics frame through SurfaceFrameSubmissionRegistryResource without RenderPlugin querying UiOverlayState for UI semantics
RenderPlugin no longer imports, exports, or schedules collect_runtime_ui_frame_submissions_system
engine/src/plugins/render/runtime/ui_submission.rs is deleted or contains no semantic collection path after merge
no compat_*.rs module or public manual add_ui_* registration chain is introduced
PreparedUiFrameResource receives scene/debug producer contributions through the generic seam when the owning producer plugins run
existing scene/debug overlay behavior is proven by focused tests, or any retired behavior is explicitly justified with evidence
UiPlugin render publication tests still pass, proving Phase 010 behavior was not regressed
```

Validation envelope:

```text
cargo fmt --check
cargo test -p engine runtime_ui_producer_migration
cargo test -p engine scene_registered_apps_publish_overlay_frame_with_buttons
cargo test -p engine debug_metrics_plugin_populates_overlay_draw_state
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine runtime_surface_guard
cargo test -p engine ui_render_publication
cargo test -p engine --test render_flow_v2
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: the implementation PR must include source inspection evidence for the two producer paths above, focused command validation, PR metadata/check evidence, forbidden-scope proof, and explicit proof that RenderPlugin no longer owns scene/debug UI semantic collection. Highest expected evidence class before merge is `E9` when source/test inspection, local validation, and accepted authority align.

Principle compliance matrix:

| Principle | Required Phase 011 evidence | Stop signal |
|---|---|---|
| KISS | Direct producer owner -> generic submission registry -> prepared UI contribution path. | A new registry, adapter stack, or hidden RenderPlugin collection path appears. |
| DRY | One owner publishes each scene/debug producer submission. | Parallel render-owned and producer-owned paths remain after merge. |
| YAGNI | No new public API, product app, reload/persistence, backend, or framework surface. | A future-phase surface enters the PR. |
| SOLID | Scene/debug generation stays with scene/debug owners; RenderPlugin stays a generic consumer. | RenderPlugin keeps source/action/host/UI semantic responsibility. |
| Separation of Concerns | Publication, generation, render preparation, and tests stay in their owning modules. | Frame generation is moved into render or render preparation is moved into scene/debug. |
| Avoid Premature Optimization | Preserve whole-frame producer submissions; no incremental render claims. | The PR adds cache/incremental behavior without proof. |
| Law of Demeter | Producers publish through SurfaceFrameSubmissionRegistryResource; render reads prepared contributions. | Code reaches through SceneResource or UiOverlayState from RenderPlugin for UI semantics. |

Module decomposition map:

| Module / file | Responsibility | Public API exported | Tests proving it | Split trigger |
|---|---|---|---|---|
| `engine/src/plugins/scene/**` allowed files | Scene overlay frame publication through the generic seam while preserving scene frame generation. | None expected. | `runtime_ui_producer_migration`, `scene_registered_apps_publish_overlay_frame_with_buttons`. | More than publication helper/schedule wiring is needed. |
| `engine/src/plugins/debug_metrics/mod.rs` | Debug metrics frame publication/removal through the generic seam while preserving debug frame generation. | None expected. | `runtime_ui_producer_migration`, `debug_metrics_plugin_populates_overlay_draw_state`. | Debug metrics behavior changes beyond producer publication. |
| `engine/src/plugins/render/runtime/ui_submission.rs` | Legacy render-owned collection retired. | None. | `runtime_surface_guard`, source inspection. | Any retained semantic collection remains. |
| `engine/src/plugins/render/plugin.rs` and `runtime/mod.rs` | Remove legacy collection schedule/export only. | Existing RenderPlugin public type unchanged. | `runtime_ui_producer_migration`, `ui_render_publication`, `render_flow_v2`. | Any render preparation, frame submit, backend, graph, or shader behavior change is needed. |
| Focused tests | Prove producer-owned publication and no Phase 010 regression. | None. | The validation envelope above. | Tests require public APIs or fixtures outside allowed scope. |

Stop conditions: stop if the path list proves incomplete, if implementation needs files outside the allowed list, if the PR leaves parallel prior/target runtime paths, if RenderPlugin keeps owning UI semantic producer collection, if source/program/action semantics change, if unrelated render behavior changes, if a public manual registration escape hatch is introduced, if UiPlugin render publication regresses, or if validation cannot be reported honestly.

Known blockers: Phase 012 and later remain blocked until Phase 011 is implemented, reviewed, merged, and completion truth is recorded.

Next action: after this activation PR merges, create exactly one bounded `PT-UI-RUNTIME-PLATFORM-011 — Scene/Debug Overlay Producer Migration and Retirement` implementation branch/PR from current `main`. Keep it draft until the focused Phase 011 validation and required docs/diff/status commands are clean. Do not start Phase 012.

## Active-work rules

- One current focus is preferred.
- If no current focus exists, say that explicitly.
- Do not promote deferred work without recording the reason.
- Do not mark work completed without evidence.
- If generated views disagree, report them as stale mirrors.
- Use `../workflow-lifecycle.md` before changing active work state.
- Architecture acceptance does not authorize implementation. Use `active-implementation` only when exact scope, owner, validation, evidence expectation, stop conditions, principle compliance status, and module decomposition status are known.

## Update shape

```text
ID:
Title:
State:
Lifecycle state:
Owner:
Authority files:
Evidence classes:
Complete investigation gate:
Complete design gate:
Implementation contract:
Allowed files/crates:
Non-owned files/crates:
Known blockers:
Next action:
```
