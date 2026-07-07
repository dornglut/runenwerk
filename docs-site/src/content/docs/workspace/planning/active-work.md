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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-007`

Title: `Host Action Dispatch and Runtime Trace`

State: active-implementation authorization recorded for one bounded Phase 007 PR. No runtime code is changed by this planning record.

Lifecycle state: `active-implementation` for Phase 007 only.

Owner: `engine::plugins::ui` owns generic UI action dispatch resources, reports, diagnostics, and trace records. Host/app owners own mutation of app state. `ui_hosts` owns host-facing capability/intent semantics if a Phase 007 implementation needs that existing contract. RenderPlugin owns render preparation/submission consumption only.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E6` PR merge/check metadata for PR #89, and `E8` accepted architecture/workflow/planning authority. Phase 006 implementation completion remains backed by `E5` local command validation and `E9` code/test plus validation plus authority alignment in the closeout report.

Complete investigation gate: complete for opening Phase 007 active implementation. Phase 007 inherits the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, and the Phase 006 closeout evidence.

Complete design gate: complete for Phase 007 implementation through the accepted cutover plan, Phase 006 closeout, and this planning authorization record.

Implementation authorization status: `active-implementation-authorized`.

Phase 006 completion truth:

```text
PR #88 merged into main at 82d6f00326cf2823eb91d3f655a730b962b355f6.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-006-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 007 handoff contract from accepted cutover authority:

```text
engine/src/plugins/ui/events.rs
engine/src/plugins/ui/action.rs
engine/src/plugins/ui/host.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/trace.rs
engine/Cargo.toml dependency on ui_hosts if not already present; engine already has ui_hosts from Phase 005
focused positive and negative engine tests, named for `cargo test -p engine ui_action`
```

Required Phase 007 evidence from accepted cutover authority:

```text
known action mutates only through app/host owner
unknown route does not mutate
schema mismatch does not mutate
capability mismatch does not mutate
payload mismatch does not mutate
missing host data does not mutate
action report records route/action/host/failure reason
generic UI-runtime trace records mounted/input/route/capability/dispatch/mutation/rejection/diagnostic events
```

Principle compliance matrix:

```text
KISS: Phase 007 should route typed UI action attempts through one clear dispatch/report/trace path.
DRY: Phase 007 must reuse existing typed action and host contracts instead of inventing a second action or capability model.
YAGNI: Phase 007 must not add runtime evaluation, render publication, product app behavior, reload/persistence, SDF, or generic frameworks.
SOLID: event input, action dispatch, host mutation boundary, reports, diagnostics, and trace facts must remain separately owned.
Separation of Concerns: generic UI dispatch belongs to UiPlugin; actual app mutation belongs to host/app owners; RenderPlugin remains outside action semantics.
Avoid Premature Optimization: no broad replay framework or engine-wide tracing substrate belongs in Phase 007.
Law of Demeter: dispatch should depend on direct typed action/host contracts and report stable failure reasons instead of reaching into product/editor/game state.
```

Module decomposition map:

```text
engine/src/plugins/ui/events.rs: generic UI input/action event envelope only.
engine/src/plugins/ui/action.rs: action dispatch contract extensions only.
engine/src/plugins/ui/host.rs: host-owned mutation boundary only.
engine/src/plugins/ui/report.rs: action dispatch reporting only if a shared report module is needed.
engine/src/plugins/ui/diagnostics.rs: stable action/dispatch failure diagnostics only.
engine/src/plugins/ui/trace.rs: first generic UI-runtime trace subset only.
engine/Cargo.toml: `ui_hosts` dependency only if not already present.
focused engine tests: positive dispatch and fail-closed negative cases.
```

Maintainability review status: complete for Phase 007 authorization. Stop if implementation needs a broader module map than the files named here.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: completed by Phase 005.
Mounted sessions: completed by Phase 006.
Host action dispatch and trace: active-implementation Phase 007.
Runtime evaluation/invalidation: downstream Phase 008.
Render boundary/publication: downstream Phases 009-010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 007 validation envelope from cutover and workflow authority:

```text
cargo test -p engine ui_action
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused engine tests must prove a known action mutates only through app/host ownership, each rejected path does not mutate, action reports name route/action/host/failure reason, and generic UI-runtime trace records mounted/input/route/capability/dispatch/mutation/rejection/diagnostic events without becoming Counter-specific.

Stop conditions: stop if errors become silent, partial mutation is possible on invalid input, product/editor/game semantics move into generic UI, trace is Counter-specific, trace extraction becomes an engine-wide framework, runtime evaluation/render publication/source reload/persistence enters the PR, `apps/ui_counter_runtime` enters the PR, or Phase 008+ files become necessary.

Known blockers: no Phase 007 implementation branch has been opened or merged yet. Phase 008 and later remain blocked until Phase 007 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded `PT-UI-RUNTIME-PLATFORM-007 - Host Action Dispatch and Runtime Trace` implementation branch/PR from current `main` after this planning truth is merged. Keep the PR draft until focused Phase 007 validation and the required docs/diff/status commands are clean.

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
