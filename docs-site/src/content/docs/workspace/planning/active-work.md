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

State: active planning after completed Phase 010 closeout truth. No Phase 011 runtime implementation is authorized by this closeout record.

Lifecycle state: `active-planning` for Phase 011 only.

Owner: render runtime producer collection owns the existing scene/debug overlay producer paths. Phase 011 planning must decide the exact replacement or retirement path through the producer-generic surface-frame seam without moving UI source, action, host mutation, or route policy into RenderPlugin.

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

Complete investigation gate: complete for opening Phase 011 active planning through accepted runtime-platform authority and Phase 010 closeout evidence. Not yet complete for implementation authorization until the Phase 011 source/path investigation is recorded.

Complete design gate: accepted cutover authority defines the Phase 011 target. Active implementation still requires a separate activation record with exact owner, files/crates, validation, evidence, principle checks, module decomposition, and stop conditions.

Implementation authorization status: blocked. This closeout opens active planning only.

Phase 010 completion truth:

```text
PR #101 merged into main at 8d6c13146deab870dca5533204067249aa2c1b90.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-010-closeout.md.
```

Phase 011 handoff contract from accepted cutover authority:

```text
every prior hardcoded scene/debug UI producer path is named
replacement path is named or retirement is justified
no compat_*.rs modules remain after merge
no prior manual UI registration path remains public
RenderPlugin no longer owns UI semantic producer collection
```

Active-planning investigation map:

```text
inspect engine/src/plugins/render/runtime/ui_submission.rs
inspect SceneResource overlay runtime frame collection and UiOverlayState debug frame collection
inspect SurfaceFrameSubmissionRegistryResource producer/surface semantics after Phase 010
inspect tests that prove scene overlay, debug overlay, render surface guard, render output proof, and render flow behavior
name every prior hardcoded scene/debug UI producer path before proposing changes
decide whether each path is replaced by the generic producer path or intentionally retired
derive exact allowed files, forbidden files, validation, evidence, and stop conditions before implementation authorization
```

Preliminary Phase 011 implementation scope from accepted cutover authority, not yet authorized:

```text
specific existing render collection paths identified by the Phase 011 investigation
engine/src/plugins/ui/** only if the new UiPlugin producer path needs migration helpers that are gone before merge
focused tests proving existing scene/debug overlay behavior through the generic producer path or proving intentional retirement
```

Forbidden files and crates until a separate active-implementation authorization narrows them:

```text
apps/ui_counter_runtime product packaging
source reload/persistence implementation
SDF or SpatialCanvas implementation
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
every hardcoded scene/debug overlay UI producer path is named
each named path is either replaced through the generic producer path or intentionally retired with evidence
no parallel prior/target runtime paths remain after merge
no public manual UI registration path remains as a compatibility escape hatch
RenderPlugin no longer owns UI semantic producer collection
existing scene/debug overlay behavior is proven or its retirement is explicitly justified
```

Validation envelope to define during Phase 011 activation:

```text
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine runtime_surface_guard
cargo test -p engine ui_plugin
cargo test -p engine --test render_flow_v2
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: Phase 011 activation must prove the current scene/debug overlay producer paths are fully named, classify whether each is migrated or retired, and define focused tests that prove no hidden render-owned UI semantic collection remains.

Stop conditions: stop if the path list is incomplete, if the PR would leave parallel prior/target runtime paths, if RenderPlugin keeps owning UI semantic producer collection, if source/program/action semantics change, if unrelated render behavior changes, or if exact allowed/forbidden files cannot be recorded before implementation.

Known blockers: Phase 011 is not implementation-authorized. Phase 012 and later remain blocked until Phase 011 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded Phase 011 active-implementation authorization branch/PR from current `main` after this closeout merges. Do not patch runtime code for Phase 011 until that authorization records exact allowed files, forbidden files, validation, evidence, and stop conditions.

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
