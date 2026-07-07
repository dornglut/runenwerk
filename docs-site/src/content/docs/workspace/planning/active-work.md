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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-010`

Title: `UiPlugin Render Publication`

State: active-implementation authorization recorded for one bounded Phase 010 PR. No runtime code is changed by this planning record.

Lifecycle state: `active-implementation` for Phase 010 only.

Owner: `engine::plugins::ui` owns publication from evaluated UiPlugin runtime frames into the generic surface-frame seam. Render frame/submission contracts own the producer/surface/frame vocabulary. RenderPlugin consumes generic packets without owning `UiScreen`, `IntoUi`, actions, host mutation, or route policy.

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
docs-site/src/content/docs/reports/closeouts/pt-workflow-track-orchestration-001-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-003-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-004-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-005-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-006-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-007-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-008-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-009-closeout.md
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E6` PR #98 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` Phase 009 code/test plus validation plus authority alignment from the closeout report.

Complete investigation gate: complete for opening Phase 010 active implementation. Phase 010 inherits the completed runtime-platform investigation/design authority, Phase 009 closeout evidence, and activation-time inspection of current `UiRuntimeEvaluationResource`, `UiRuntimeEvaluationReport`, `UiRuntimeFramePayload`, `UiRuntimeTraceResource`, UiPlugin wiring, generic `SurfaceFrameSubmissionRegistryResource`, `RenderFrameProducerId`, `RenderSurfaceId`, and focused UI/render tests.

Complete design gate: complete for Phase 010 implementation through the accepted cutover plan, architecture record, Phase 009 closeout, this activation record, and the current source inspection. The implementation must remain one bounded publication PR.

Implementation authorization status: `active-implementation-authorized`.

Phase 009 completion truth:

```text
PR #97 merged into main at 50e2dbdf1f9c076f4a76a04543274801d1f1649b.
Closeout PR #98 merged into main at 1b29eb58cdbea4d3c351403702373d013772d541.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-009-closeout.md.
```

Phase 010 handoff contract from accepted cutover authority:

```text
UiPlugin publishes frame submission with producer id and surface identity through the generic seam.
RenderPlugin consumes prepared payload without querying UiScreen, IntoUi, actions, host mutation, or route policy.
render contribution is deterministic for the same runtime frame.
missing UiPlugin frame reports a diagnostic instead of silent success.
frame publication trace records producer, surface, frame revision, dirty cause, and publication result.
trace adds UiFramePublished and UiFramePresented event families as Phase 010 UI-semantic facts.
```

Activation-time implementation map:

```text
add engine/src/plugins/ui/render_publish.rs for UiPlugin-owned publication target, result/report helpers, and the system/function that writes SurfaceFrameSubmission records
extend engine/src/plugins/ui/mod.rs exports for the publication API
extend engine/src/plugins/ui/plugin.rs only for publication resource initialization and RenderPrepare scheduling
extend engine/src/plugins/ui/schedule.rs with a RenderPublication set label
extend engine/src/plugins/ui/report.rs with frame publication/presentation report facts
extend engine/src/plugins/ui/trace.rs with UiFramePublished and UiFramePresented trace event facts
extend engine/src/plugins/ui/diagnostics.rs only for missing-frame or publication-rejected diagnostics
use existing SurfaceFrameSubmissionRegistryResource, SurfaceFrameSubmission, SurfaceFrameRoute, SurfaceFrameSubmissionOrder, RenderFrameProducerId, and RenderSurfaceId
add focused engine/tests/ui_render_publication.rs tests
```

Allowed files and crates for the Phase 010 implementation authorization:

```text
engine/src/plugins/ui/render_publish.rs
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/schedule.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/trace.rs
engine/src/plugins/ui/diagnostics.rs only for publication diagnostics
engine/src/plugins/ui/resources.rs only if publication needs a UiPlugin-owned publication resource or read-only helper access to latest evaluation reports
engine/src/plugins/render/features/ui/submission.rs only if a small generic-seam API extension is required; no rename or semantic rewrite
engine/src/plugins/render/features/ui/resource.rs only if tests need prepared payload access that already belongs to the generic seam; no renderer behavior rewrite
engine/tests/ui_render_publication.rs
engine/tests/ui_runtime_evaluation.rs only if existing helpers or assertions must align with publication facts
engine/tests/render_flow_v2.rs only if the prepared-frame integration proof needs an assertion against the new UiPlugin producer contribution
```

Forbidden files and crates:

```text
scene/debug overlay migration or retirement implementation
apps/ui_counter_runtime product packaging
source reload/persistence implementation
SDF or SpatialCanvas implementation
render backend rewrite, graph execution rewrite, or shader changes
source/program/action semantic changes outside publication facts
host mutation or action-dispatch behavior changes
broad ui_render_data primitive/model rewrites
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Acceptance criteria:

```text
UiPlugin publication uses a stable non-zero RenderFrameProducerId owned by UiPlugin render publication.
UiPlugin publication writes through SurfaceFrameSubmissionRegistryResource and includes an explicit RenderSurfaceId target.
Publication derives from the latest UiRuntimeEvaluationReport and UiRuntimeFramePayload, not directly from UiScreen, IntoUi, actions, or host mutation.
Prepared UI render-feature payloads include the UiPlugin producer contribution after RenderPlugin preparation.
Missing evaluation/frame payload records a diagnostic and report instead of silent success.
Trace records UiFramePublished and UiFramePresented facts with producer id, surface id, frame revision, dirty cause, and result.
The same runtime frame publishes deterministically through the same producer/surface key.
Scene/debug overlay producers remain present only as downstream Phase 011 migration inputs.
```

Principle compliance matrix:

```text
KISS: publish the current runtime frame through the existing generic seam; do not redesign rendering.
DRY: use Phase 009 SurfaceFrameSubmission contracts instead of adding a parallel UI-specific submission path.
YAGNI: do not package Counter, migrate overlays, implement reload/persistence, or start SDF/world-space work.
SOLID: UiPlugin owns runtime publication facts; RenderPlugin owns preparation and consumes generic packets.
Separation of Concerns: publication reads runtime reports and writes generic submissions, but does not own renderer backend execution.
Avoid Premature Optimization: no backend, graph, shader, batching, or incremental rendering optimization belongs in this phase.
Law of Demeter: RenderPlugin must not reach into UiScreen, IntoUi, actions, host mutation, or route policy.
```

Phase 010 validation envelope from cutover, architecture, and source inspection:

```text
cargo test -p engine ui_render_publication
cargo test -p engine ui_runtime_evaluation
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine --test render_flow_v2
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused tests and compile checks must prove UiPlugin publishes through the generic `SurfaceFrameSubmissionRegistryResource`, uses `RenderFrameProducerId` and `RenderSurfaceId`, records deterministic publication/presentation facts, preserves renderer ownership boundaries, and reports diagnostics on missing runtime frame payloads.

Stop conditions: stop if RenderPlugin becomes the UI runtime owner, pulls from app host state directly, requires a broad backend rewrite, requires scene/debug overlay retirement, creates a second UI runtime path, changes source/program/action semantics, cannot map publication to an explicit render surface, or needs files outside the allowed set.

Known blockers: no Phase 010 implementation branch has been opened or merged yet. Phase 011 and later remain blocked until Phase 010 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded `PT-UI-RUNTIME-PLATFORM-010 - UiPlugin Render Publication` implementation branch/PR from current `main` after this planning truth is merged. Keep the PR draft until focused Phase 010 validation and the required docs/diff/status commands are clean.

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
