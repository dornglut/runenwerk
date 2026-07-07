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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-004`

Title: `App Mounting API`

State: active-implementation authorization recorded for one bounded Phase 004 PR. No runtime code is changed by this planning record.

Lifecycle state: `active-implementation` for Phase 004 only.

Owner: `engine::plugins::ui` owns the App-facing mounting API and mount-request resources. Domain UI crates continue to own UI semantics; RenderPlugin owns render preparation/submission consumption only.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 003, `E6` PR merge/check metadata for PR #79, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment for Phase 003 closeout.

Complete investigation gate: complete for opening Phase 004 active planning. Phase 004 inherits the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, and the Phase 003 closeout evidence.

Complete design gate: complete for Phase 004 implementation through the accepted cutover plan, Phase 003 closeout, and this planning authorization record.

Implementation authorization status: `active-implementation-authorized`.

Phase 003 completion truth:

```text
PR #79 merged into main at 0135850277e904b4be2c336e3ef6507b3fc88b72.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-003-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 004 handoff contract from accepted cutover authority:

```text
add engine/src/plugins/ui/app_ext.rs
add engine/src/plugins/ui/mount.rs
extend engine/src/plugins/ui/resources.rs only for mount-request storage
extend engine/src/plugins/ui/diagnostics.rs only for mount diagnostics
optionally update engine/src/prelude.rs only if accepted for public export
add focused engine tests/examples proving app.mount_ui and app.ui().mount compile and record equivalent mount requests
```

Principle compliance matrix:

```text
KISS: planning-only; Phase 004 must expose a direct App -> UiPlugin mount request path without manual route/render setup.
DRY: planning-only; Phase 004 must reuse the existing UiPlugin foundation resources and not duplicate source/program/surface semantics.
YAGNI: planning-only; Phase 004 must not add typed screen/source/action contracts, sessions, trace, render publication, product app, SDF, or generic frameworks.
SOLID: planning-only; app extension, mount request storage, diagnostics, and tests must remain separately owned.
Separation of Concerns: planning-only; engine mount recording must stay separate from domain UI semantics and render consumption.
Avoid Premature Optimization: planning-only; no runtime evaluation, dirty tracking, or render publication belongs in Phase 004.
Law of Demeter: planning-only; public callers should use direct app/ui mounting APIs instead of route maps, event packets, host adapters, or render registries.
```

Module decomposition map:

```text
engine/src/plugins/ui/app_ext.rs: public App-facing extension API only.
engine/src/plugins/ui/mount.rs: mount request/config/report types only.
engine/src/plugins/ui/resources.rs: mount request queue/storage only, reusing Phase 003 foundation state.
engine/src/plugins/ui/diagnostics.rs: mount diagnostic additions only.
engine/src/prelude.rs: optional public export only if accepted by the Phase 004 implementation authorization.
focused engine tests/examples: App mounting API compile and request-recording evidence.
```

Maintainability review status: complete for Phase 004 authorization. Stop if implementation needs a broader module map than the files named here.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: active-planning Phase 004.
Typed screen/source/action contracts: downstream Phase 005.
Mounted sessions: downstream Phase 006.
Host action dispatch and trace: downstream Phase 007.
Runtime evaluation/invalidation: downstream Phase 008.
Render boundary/publication: downstream Phases 009-010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 004 validation envelope from cutover authority:

```text
cargo test -p engine ui_mount
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused engine tests/examples must prove the normal `app.mount_ui` path and the advanced `app.ui().mount` path record equivalent mount requests, mount diagnostics include screen identity/mount source/stable failure reason, and normal users are not exposed to route maps, event packets, host adapters, or render registries.

Stop conditions: stop if Phase 004 requires manual host adapters, manual route maps, manual render submission writes, private App internals outside the accepted API, typed screen/source/action implementation, mounted session runtime, host action dispatch, runtime trace, render publication, source reload/persistence, `apps/ui_counter_runtime`, SDF/world-space/SpatialCanvas, `foundation/meta`, `domain/app_program`, a generic plugin framework, or a render backend rewrite.

Known blockers: no Phase 004 implementation branch has been opened or merged yet. Phase 005 and later remain blocked until Phase 004 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API` implementation branch/PR from current `main` after this planning truth is merged. Keep the PR draft until focused Phase 004 validation and the required docs/diff/status commands are clean.

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
