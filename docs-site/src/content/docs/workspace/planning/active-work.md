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

ID: `PT-UI-RUNTIME-PLATFORM-005`

Title: `Typed Screen / Source / Action Contracts`

State: active-implementation authorization recorded for one bounded Phase 005 PR. No runtime code is changed by this planning record.

Lifecycle state: `active-implementation` for Phase 005 only.

Owner: `engine::plugins::ui` owns the engine-facing typed screen/source/action facade. Existing domain UI crates continue to own UI source, program, lowering, host, and evaluator semantics; RenderPlugin owns render preparation/submission consumption only.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 004, `E6` PR merge/check metadata for PR #82, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment for Phase 004 closeout.

Complete investigation gate: complete for opening Phase 005 active planning. Phase 005 inherits the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, and the Phase 004 closeout evidence.

Complete design gate: complete for Phase 005 implementation through the accepted cutover plan, Phase 004 closeout, and this planning authorization record.

Implementation authorization status: `active-implementation-authorized`.

Phase 004 completion truth:

```text
PR #82 merged into main at 9fb86f0d426385be7e425ff943c7a9d5450e1edb.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-004-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 005 handoff contract from accepted cutover authority:

```text
add engine/src/plugins/ui/screen.rs
add engine/src/plugins/ui/source.rs
add engine/src/plugins/ui/action.rs
add engine/src/plugins/ui/host.rs
extend engine/src/plugins/ui/diagnostics.rs only for typed contract diagnostics
engine/Cargo.toml dependency additions only for selected domain/ui crates if justified
add focused engine tests plus comparison evidence from ui_app_integration where useful
```

Principle compliance matrix:

```text
KISS: planning-only; Phase 005 must expose a small engine facade over existing domain UI contracts instead of inventing a broad runtime domain.
DRY: planning-only; Phase 005 must reuse `ui_definition`, `ui_program`, `ui_program_lowering`, `ui_hosts`, and `ui_app_integration` proof evidence instead of duplicating source/action semantics.
YAGNI: planning-only; Phase 005 must not add sessions, trace, render publication, product app, source reload/persistence, SDF, or generic frameworks.
SOLID: planning-only; screen, source, action, host, diagnostics, and tests must remain separately owned.
Separation of Concerns: planning-only; typed contracts must lower into source/program/host facts without moving domain UI or render ownership into engine.
Avoid Premature Optimization: planning-only; no runtime evaluation, dirty tracking, or render publication belongs in Phase 005.
Law of Demeter: planning-only; public callers should use typed screen/action contracts instead of route maps, event packets, host adapters, or render registries.
```

Module decomposition map:

```text
engine/src/plugins/ui/screen.rs: typed screen facade only.
engine/src/plugins/ui/source.rs: source/source-map facade only.
engine/src/plugins/ui/action.rs: typed action contract facade only.
engine/src/plugins/ui/host.rs: host-owned mutation intent facade only.
engine/src/plugins/ui/diagnostics.rs: typed contract diagnostic additions only.
engine/Cargo.toml: selected domain/ui crate dependency additions only if justified by accepted ownership.
focused engine tests/examples: typed screen/source/action compile, lowering, identity, and no-mutation evidence.
```

Maintainability review status: complete for Phase 005 authorization. Stop if implementation needs a broader module map than the files named here.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: active-planning Phase 005.
Mounted sessions: downstream Phase 006.
Host action dispatch and trace: downstream Phase 007.
Runtime evaluation/invalidation: downstream Phase 008.
Render boundary/publication: downstream Phases 009-010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 005 validation envelope from cutover and workflow authority:

```text
cargo test -p engine ui_typed
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused engine tests/examples must prove typed screens lower to `ui_definition`-compatible source records, typed source produces route/source-map facts, typed action handlers emit host-owned mutation intent, action identity is stable and diagnostic-friendly, and `ui_app_integration` remains proof evidence rather than final framework owner.

Stop conditions: stop if typed UI skips source/program facts, if generic controls mutate app state directly, if a new broad runtime-platform domain crate becomes necessary, or if Phase 005 requires mounted session runtime, host action dispatch, runtime trace, render publication, source reload/persistence, `apps/ui_counter_runtime`, SDF/world-space/SpatialCanvas, `foundation/meta`, `domain/app_program`, a generic plugin framework, or a render backend rewrite.

Known blockers: no Phase 005 implementation branch has been opened or merged yet. Phase 006 and later remain blocked until Phase 005 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` implementation branch/PR from current `main` after this planning truth is merged. Keep the PR draft until focused Phase 005 validation and the required docs/diff/status commands are clean.

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
