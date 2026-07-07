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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-003`

Title: `UiPlugin Foundation`

State: active-implementation authorization recorded for one bounded Phase 003 PR. No runtime code is changed by this planning record.

Lifecycle state: `active-implementation` for Phase 003 only.

Owner: `engine::plugins::ui` owns the UiPlugin foundation shell. Domain UI crates own UI semantics; RenderPlugin owns render preparation/submission consumption only.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path and `E8` accepted architecture/workflow/planning authority. Phase 003 implementation must add `E5` local command validation before merge readiness can be claimed.

Complete investigation gate: complete. Phase 003 inherits the completed `PT-UI-RUNTIME-PLATFORM-001` investigation and the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, runtime architecture, trace/agent requirements, source reload/persistence boundaries, and producer-generic render-boundary ordering.

Complete design gate: complete for Phase 003. The accepted cutover plan authorizes only the UiPlugin foundation shell as the first runtime implementation phase after `PT-WORKFLOW-TRACK-ORCHESTRATION-001` merged and closeout truth was recorded.

Implementation contract: create the engine UI plugin foundation only. The Phase 003 PR may add the `engine::plugins::ui` module root, `UiPlugin`, schedule labels, resources, reports, diagnostics, plugin export wiring, and focused tests proving plugin install/resource initialization behavior.

Allowed files/crates for current focus:

```text
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/schedule.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/mod.rs
engine/Cargo.toml only if justified by focused tests or feature wiring
focused engine tests for plugin installation/resource initialization
```

Non-owned files/crates for current focus:

```text
public AppUiExt code
app.mount_ui implementation
UiScreen / IntoUi implementation
UiActionHandler implementation
render adapter code
SurfaceFrame generic producer boundary implementation code
scene/debug overlay producer migration implementation code
source reload/persistence implementation code
apps/ui_counter_runtime implementation
SDF/world-space/SpatialCanvas implementation
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Principle compliance matrix:

```text
KISS: pass; direct engine plugin foundation with no public mounting/action/render path.
DRY: pass; no duplicate UI source, host, surface, evaluator, or render semantics.
YAGNI: pass; no AppUiExt, typed screen/action API, generic plugin framework, validator, SDF, or product app.
SOLID: pass; plugin, schedule, resources, report, and diagnostics have separate owners.
Separation of Concerns: pass; engine composition shell stays separate from domain UI semantics and render consumption.
Avoid Premature Optimization: pass; no render genericization or runtime trace extraction in Phase 003.
Law of Demeter: pass; Phase 003 may expose only direct plugin/resources/report contracts.
```

Module decomposition map:

```text
engine/src/plugins/ui/mod.rs: module boundary and public re-exports.
engine/src/plugins/ui/plugin.rs: UiPlugin install/build behavior.
engine/src/plugins/ui/schedule.rs: UI runtime schedule labels only.
engine/src/plugins/ui/resources.rs: stable default engine resources for future mounts/sessions/reports.
engine/src/plugins/ui/report.rs: inspection/report shell for install/resource state.
engine/src/plugins/ui/diagnostics.rs: deterministic diagnostic vocabulary for foundation failures.
engine/src/plugins/mod.rs: plugin module registration only.
```

Maintainability review status: complete for Phase 003. The phase is intentionally a foundation shell; split or stop if app mounting, typed screens/actions, mounted sessions, render publication, trace, product app, or persistence starts entering this PR.

Feature support matrix:

```text
UiPlugin install/resource shell: delivered by Phase 003.
Public mounting API: downstream Phase 004.
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

Validation envelope:

```text
cargo test -p engine ui_plugin
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused engine tests must prove `UiPlugin` installs without panicking, duplicate install behavior is deterministic or idempotent, default resources are stable, schedule labels exist, and no render/backend ownership changes are made.

Stop conditions: stop if Phase 003 requires `foundation/meta`, `domain/app_program`, a generic plugin framework, domain UI depending on engine, public mounting API, typed screen/source/action contracts, mounted session runtime, host action dispatch, runtime trace, render publication, scene/debug migration, `apps/ui_counter_runtime`, source reload/persistence, SDF/world-space/SpatialCanvas, or a render backend rewrite.

Known blockers: no Phase 003 implementation branch has been merged yet. Phase 004 and later remain blocked until Phase 003 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` implementation branch/PR from current `main` after this planning truth is merged. Keep the PR draft until focused Phase 003 validation and the required docs/diff/status commands are clean.

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
