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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-009`

Title: `SurfaceFrame Generic Producer Boundary`

State: active planning only after Phase 008 completion truth. No Phase 009 implementation PR is authorized by this record.

Lifecycle state: `active-planning` for Phase 009 only.

Owner: render frame/submission contracts and `ui_render_data` own the producer-generic surface/frame vocabulary at the accepted seam. `engine::plugins::ui` remains the source/program/evaluation owner and may consume the downstream seam only after the boundary exists. RenderPlugin consumes generic producer/surface/frame packets without owning UI semantics.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 008, `E6` PR #94 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment in the closeout report.

Complete investigation gate: complete for Phase 009 active planning through the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, and Phase 008 closeout evidence. A separate Phase 009 activation record must map the exact render frame/submission contracts, `ui_render_data` names/types, and migration tests before implementation.

Complete design gate: complete for Phase 009 active planning through the accepted cutover plan, architecture record, and Phase 008 closeout. Implementation remains blocked until a separate active-implementation decision records exact scope, owner checks, validation, evidence expectation, stop conditions, principle compliance, and module decomposition.

Implementation authorization status: `not-authorized`; active planning only.

Phase 008 completion truth:

```text
PR #94 merged into main at be5b790e38b7f80ad17092fa0cb75e87eef4d849.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-008-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 009 handoff contract from accepted cutover authority:

```text
only the render frame/submission contracts named by the phase PR
only ui_render_data names/types touched by the accepted migration map
existing render submission registry/resource paths required for the generic producer seam
focused migration tests and compile checks
```

Required Phase 009 evidence from accepted cutover authority:

```text
migration map lists every renamed type/module/function
producer-generic names replace UI-specific render ownership at the accepted seam before UiPlugin publishes durable frames
producer id and surface identity are generic concepts, not UiPlugin concepts
RenderPlugin consumes generic producer/surface/frame packets
scene/debug paths remain named as migration inputs, not hidden parallel paths
external docs no longer imply RenderPlugin owns UI semantics
```

Principle compliance matrix:

```text
KISS: Phase 009 should rename or introduce only the accepted producer-generic seam, not rewrite the renderer backend.
DRY: Phase 009 must remove duplicate UI-specific render ownership names at the seam instead of adding a parallel path.
YAGNI: Phase 009 must not publish UiPlugin frames, migrate overlays, package the Counter app, add reload/persistence, or start SDF/world-space work.
SOLID: producer identity, surface identity, frame packet shape, and runtime UI semantics must remain separately owned.
Separation of Concerns: RenderPlugin consumes generic frame packets; UiPlugin remains outside Phase 009 publication until Phase 010.
Avoid Premature Optimization: no backend rewrite or performance claim belongs in the boundary rename/migration phase.
Law of Demeter: RenderPlugin should depend on generic producer/surface/frame contracts, not `UiScreen`, `IntoUi`, actions, host mutation, or route policy.
```

Module decomposition map:

```text
render frame/submission contracts: producer-generic type/module/function names selected by the activation PR only.
ui_render_data: accepted seam names/types touched by the migration map only.
existing render submission registry/resource paths: generic producer seam wiring only.
tests: focused migration tests and compile checks that prove old UI-specific ownership names are not retained at the accepted seam.
```

Maintainability review status: complete for Phase 009 planning. Stop before implementation if activation cannot name the exact seam, migration map, and focused validation without broad render-backend churn.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: completed by Phase 005.
Mounted sessions: completed by Phase 006.
Host action dispatch and trace: completed by Phase 007.
Runtime evaluation/invalidation: completed by Phase 008.
SurfaceFrame generic producer boundary: active-planning Phase 009.
UiPlugin render publication: downstream Phase 010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 009 validation envelope from cutover and workflow authority:

```text
cargo test <focused Phase 009 migration filter selected by activation PR>
cargo test <relevant crate/package command selected by activation PR>
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: the activation PR must inspect current render frame/submission contracts and publish a migration map before implementation. Focused Phase 009 tests must prove producer-generic names at the accepted seam, generic producer id and surface identity, RenderPlugin consumption of generic packets, and named scene/debug migration inputs without hidden parallel paths.

Stop conditions: stop if the rename becomes broad or unreviewable, source/program/action semantics change, the phase becomes a render backend rewrite, genericization creates a second runtime path, UiPlugin render publication enters the PR, scene/debug overlay migration enters the PR, or Phase 010+ files become necessary.

Known blockers: no Phase 009 implementation branch is authorized yet. The exact render seam, `ui_render_data` migration map, focused tests, and relevant cargo command must be confirmed by a separate activation PR after this closeout/planning truth merges. Phase 010 and later remain blocked until Phase 009 is reviewed, merged, and completion truth is recorded.

Next action: create a separate `PT-UI-RUNTIME-PLATFORM-009 - SurfaceFrame Generic Producer Boundary` activation branch/PR after this closeout/planning truth merges. Keep Phase 009 implementation blocked until that activation record is merged.

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
