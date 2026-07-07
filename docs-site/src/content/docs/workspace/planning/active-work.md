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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-008`

Title: `Runtime Evaluation, State Snapshot, and Invalidation`

State: active planning only after Phase 007 completion truth. No Phase 008 implementation PR is authorized by this record.

Lifecycle state: `active-planning` for Phase 008 only.

Owner: `engine::plugins::ui` owns the generic runtime evaluation, UI session snapshot, invalidation, reports, diagnostics, and trace records. Existing evaluator/runtime-view/render-data owners remain the source for their contracts. Host/app owners still own app-state mutation. RenderPlugin remains outside runtime evaluation ownership and consumes only downstream published render frames.

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
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 007, `E6` PR #91 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment in the closeout report.

Complete investigation gate: complete for Phase 008 active planning through the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, and Phase 007 closeout evidence. A separate Phase 008 activation record must re-check exact dependencies and focused tests before implementation.

Complete design gate: complete for Phase 008 active planning through the accepted cutover plan, architecture record, and Phase 007 closeout. Implementation remains blocked until a separate active-implementation decision records exact scope, owner checks, validation, evidence expectation, stop conditions, principle compliance, and module decomposition.

Implementation authorization status: `not-authorized`; active planning only.

Phase 007 completion truth:

```text
PR #91 merged into main at 5dd90a2caf1bb7e4d5710830499df1d122fe587f.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-007-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 008 handoff contract from accepted cutover authority:

```text
engine/src/plugins/ui/source.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/trace.rs
engine/Cargo.toml dependencies on selected evaluator/runtime-view/render-data crates
focused engine tests for output facts, state snapshots, dirty records, and frame payload creation
```

Required Phase 008 evidence from accepted cutover authority:

```text
mounted screen source/program facts feed evaluator/runtime-view path
Counter output text changes after host mutation
frame payload is derived from runtime/evaluator output
runtime report includes source, program, runtime-view, output, diagnostics, and invalidation facts
UI session snapshot/replay is stable by source/runtime IDs
dirty records name source, host-data, session, layout, text, theme, primitive, surface, and render-publication causes
trace adds runtime evaluation and state/invalidation facts
```

Principle compliance matrix:

```text
KISS: Phase 008 should add one source/program-to-runtime-evaluation path, one snapshot shape, and one invalidation record path.
DRY: Phase 008 must reuse existing source, program, trace, evaluator/runtime-view/render-data contracts instead of inventing duplicate UI truth.
YAGNI: Phase 008 must not add render publication, product app packaging, source reload/persistence, SDF/world-space behavior, or generic frameworks.
SOLID: source facts, runtime evaluation, snapshots, invalidation causes, diagnostics, reports, and trace facts must remain separately owned.
Separation of Concerns: UiPlugin owns generic runtime evaluation state; host/app owners own mutation; RenderPlugin remains outside source/program/evaluator ownership.
Avoid Premature Optimization: no per-element incremental rendering claims without dirty-scope proof.
Law of Demeter: runtime evaluation should depend on direct source/program/evaluator/runtime-view contracts instead of reaching into renderer primitives or product state.
```

Module decomposition map:

```text
engine/src/plugins/ui/source.rs: source/program facts and runtime-evaluation input facts only.
engine/src/plugins/ui/resources.rs: runtime evaluation state, snapshot, and dirty-record resources only.
engine/src/plugins/ui/report.rs: runtime evaluation, output, snapshot, and invalidation reporting only.
engine/src/plugins/ui/diagnostics.rs: stable runtime evaluation/invalidation diagnostics only.
engine/src/plugins/ui/trace.rs: runtime evaluation and state/invalidation trace facts only.
engine/Cargo.toml: selected evaluator/runtime-view/render-data dependencies only after source inspection in the activation PR.
focused engine tests: output facts, Counter text-after-mutation proof, state snapshots, dirty records, and frame payload creation.
```

Maintainability review status: complete for Phase 008 planning. Stop before implementation if activation needs a broader module map than the files named here.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: completed by Phase 005.
Mounted sessions: completed by Phase 006.
Host action dispatch and trace: completed by Phase 007.
Runtime evaluation/invalidation: active-planning Phase 008.
Render boundary/publication: downstream Phases 009-010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 008 validation envelope from cutover and workflow authority:

```text
cargo test -p engine <Phase 008 focused runtime-evaluation filter selected by activation PR>
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: focused engine tests must prove mounted screen source/program facts feed the evaluator/runtime-view path, Counter output text changes after host mutation, frame payloads derive from runtime/evaluator output, session snapshot/replay is stable by source/runtime IDs, dirty records name the required causes, runtime reports include source/program/runtime-view/output/diagnostics/invalidation facts, and trace records runtime evaluation plus state/invalidation facts.

Stop conditions: stop if frame output skips source/program/evaluator evidence, a new execution strategy is invented without accepted design, renderer primitives become UI source truth, per-element incremental rendering is claimed without dirty-scope proof, render publication/source reload/persistence enters the PR, `apps/ui_counter_runtime` product packaging enters the PR, or Phase 009+ files become necessary.

Known blockers: no Phase 008 implementation branch is authorized yet. The selected evaluator/runtime-view/render-data dependencies and focused test filter must be confirmed by a separate activation PR after this closeout/planning truth merges. Phase 009 and later remain blocked until Phase 008 is reviewed, merged, and completion truth is recorded.

Next action: create a separate `PT-UI-RUNTIME-PLATFORM-008 - Runtime Evaluation, State Snapshot, and Invalidation` activation branch/PR after this closeout/planning truth merges. Keep Phase 008 implementation blocked until that activation record is merged.

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
