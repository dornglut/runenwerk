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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-006`

Title: `Mounted Surface Session Runtime`

State: active planning only after Phase 005 completion truth. No Phase 006 implementation is authorized by this planning record.

Lifecycle state: `active-planning` for Phase 006 only.

Owner: `engine::plugins::ui` owns the engine-facing mounted-session registry and reports. `ui_surface` owns surface definition, mounted instance, session-retention, validation, and diagnostic semantics; RenderPlugin owns render preparation/submission consumption only.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 005, `E6` PR merge/check metadata for PR #85, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment for Phase 005 closeout.

Complete investigation gate: complete for opening Phase 006 active planning. Phase 006 inherits the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, and the Phase 005 closeout evidence.

Complete design gate: complete for Phase 006 active planning through the accepted cutover plan and Phase 005 closeout. Phase 006 implementation remains blocked until a separate active-implementation authorization records the exact contract.

Implementation authorization status: `blocked-pending-active-implementation-authorization`.

Phase 005 completion truth:

```text
PR #85 merged into main at 6226470defa7a72a567fc03c1bc3783e63e2c2c8.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-005-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 006 handoff contract from accepted cutover authority:

```text
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/mount.rs
engine/src/plugins/ui/report.rs
engine/Cargo.toml dependency on ui_surface if not already present
focused engine tests for mount/unmount/generation/session reports
```

Principle compliance matrix:

```text
KISS: planning-only; Phase 006 must adapt Phase 004 mount requests into `ui_surface` mounted-instance/session records without inventing a second surface semantic model.
DRY: planning-only; Phase 006 must reuse `ui_surface` mounted-surface/session/validation contracts instead of duplicating them in engine.
YAGNI: planning-only; Phase 006 must not add host action dispatch, runtime trace, render publication, product app, source reload/persistence, SDF, or generic frameworks.
SOLID: planning-only; mount resources, mount behavior, reports, diagnostics, and focused tests must remain separately owned.
Separation of Concerns: planning-only; mounted session state belongs to UiPlugin resources using `ui_surface` semantics, not RenderPlugin or product/editor/game domains.
Avoid Premature Optimization: planning-only; no runtime evaluation, dirty tracking, or render publication belongs in Phase 006.
Law of Demeter: planning-only; public callers should keep using mount APIs while engine records session facts through direct `ui_surface` contracts.
```

Module decomposition map:

```text
engine/src/plugins/ui/resources.rs: mounted-session resource storage only.
engine/src/plugins/ui/mount.rs: mount/unmount/session behavior only.
engine/src/plugins/ui/report.rs: mounted-session reporting only.
engine/src/plugins/ui/diagnostics.rs: mounted-session diagnostics only if needed.
engine/Cargo.toml: `ui_surface` dependency only if not already present.
focused engine tests: mount/unmount/generation/session report evidence.
```

Maintainability review status: complete for Phase 006 active planning. Stop before implementation if a broader module map is needed.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: completed by Phase 005.
Mounted sessions: active-planning Phase 006.
Host action dispatch and trace: downstream Phase 007.
Runtime evaluation/invalidation: downstream Phase 008.
Render boundary/publication: downstream Phases 009-010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 006 validation envelope from cutover and workflow authority:

```text
cargo test -p engine ui_mount
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused engine tests must prove mount creates a `MountedSurfaceInstance`-compatible record, session identity, host identity, generation, retention, and diagnostics are recorded, unmount/remount behavior is deterministic, multiple mounted screens/surfaces do not collide, and engine does not invent duplicate surface/session semantics.

Stop conditions: stop if Phase 006 requires world-space UI, SDF, SpatialCanvas, product/editor/game semantics in domain UI, replacing `ui_surface` instead of adapting to it, host action dispatch, runtime trace, render publication, source reload/persistence, `apps/ui_counter_runtime`, `foundation/meta`, `domain/app_program`, a generic plugin framework, or a render backend rewrite.

Known blockers: no Phase 006 implementation branch is authorized. Phase 007 and later remain blocked until Phase 006 is separately authorized, reviewed, merged, and completion truth is recorded.

Next action: after this closeout/planning truth merges, create a separate Phase 006 activation PR that either authorizes exactly one bounded implementation PR or reports an authority conflict. Do not start Phase 006 implementation from this closeout branch.

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
