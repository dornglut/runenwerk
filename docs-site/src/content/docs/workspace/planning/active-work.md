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
  - ../../reports/closeouts/pt-ui-runtime-platform-011-closeout.md
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-012`

Title: `Runtime Counter App Product`

State: active planning after completed Phase 011 closeout truth. No Phase 012 runtime implementation is authorized by this closeout record.

Lifecycle state: `active-planning` for Phase 012 only.

Owner: the app/product layer must provide a runnable `ui_counter_runtime` app that installs `RenderPlugin`, `UiPlugin`, and `CounterPlugin`, mounts `CounterScreen`, drives typed actions through host-owned mutation, publishes frames through the generic seam, and exposes human and agent proof paths.

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
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-010-closeout.md
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-011-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 011, `E6` PR #104 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` Phase 011 code/test plus validation plus authority alignment from the closeout report.

Complete investigation gate: complete for opening Phase 012 active planning through accepted runtime-platform authority and Phase 011 closeout evidence. Not yet complete for implementation authorization until the current Counter app/product path, required app crate/workspace wiring, command behavior, trace output, public-path proof, and product validation envelope are inspected and recorded.

Complete design gate: accepted cutover authority and runtime architecture define the Phase 012 target. Active implementation still requires a separate activation record with exact owner, files/crates, validation, evidence, principle checks, module decomposition, and stop conditions.

Implementation authorization status: blocked. This closeout opens active planning only.

Phase 011 completion truth:

```text
PR #104 merged into main at 15e213a08dbf79f65e0851fe5be9f853f157b48b.
PR head before merge: a6232278e41202cd331051f347d3db892988f38c.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-011-closeout.md.
```

Phase 012 product contract from accepted cutover authority:

```text
binary: ui_counter_runtime
window title: Runenwerk UI Counter Runtime
mounted screen type: CounterScreen
host plugin type: CounterPlugin
host resource: Counter { value: i64 }
visible structure: header, count label, Increment / Decrement / Reset actions, trace console, status line
routes: counter.increment, counter.decrement, counter.reset, counter.read
normal app authors must not see route maps, event packets, host adapters, or render registries
```

Active-planning investigation map:

```text
inspect whether apps/ui_counter_runtime already exists and how workspace members are declared
inspect existing app examples for RenderPlugin / UiPlugin / product plugin composition patterns
inspect UiPlugin public mounting, typed screen/source/action, host action dispatch, runtime evaluation, frame publication, and trace APIs that the product must consume
inspect how current input/headless runtime commands can support human and agent paths
derive exact allowed files, forbidden files, validation envelope, evidence expectation, principle checks, module decomposition, and stop conditions before implementation authorization
```

Preliminary Phase 012 implementation scope from accepted cutover authority, not yet authorized:

```text
apps/ui_counter_runtime/**
root Cargo.toml workspace member entry
focused engine tests/examples for Counter app
engine/src/plugins/ui/** fixes only if required by product proof and scoped
planning docs and proof report fixtures
```

Forbidden files and crates until a separate active-implementation authorization narrows them:

```text
source reload/persistence implementation
SDF or SpatialCanvas implementation
render backend rewrite beyond product proof consumption
graph execution rewrite or shader changes
source/program/action semantic rewrites
host mutation or action-dispatch semantic rewrites
scene/debug overlay producer migration work
broad render-data primitive/model rewrites
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Acceptance criteria required before Phase 012 can close:

```text
cargo run -p ui_counter_runtime starts the human app
cargo run -p ui_counter_runtime -- --headless --agent-script <script> --trace-jsonl <path> --exit-after-script runs agent mode
cargo test -p ui_counter_runtime proves the app path
app installs RenderPlugin, UiPlugin, and CounterPlugin
CounterPlugin uses app.mount_ui(CounterScreen)
CounterScreen implements the accepted typed screen/source/action model and the architecture-owned product screen contract
UI exposes increment, decrement, and reset actions
human interaction and agent scripts use the same route/capability/payload checks
user/agent interaction mutates Counter only through host-owned path
runtime/evaluator output changes after mutation
UiPlugin publishes through the generic producer/surface-frame seam
RenderPlugin consumes the submission without UI semantic ownership
console/history view shows recent generic UI-runtime trace events
JSONL trace records action, mutation, evaluation, and frame publication facts
manual run instructions and observed behavior are recorded in the PR
```

Validation envelope to define during Phase 012 activation:

```text
cargo run -p ui_counter_runtime
cargo run -p ui_counter_runtime -- --headless --agent-script <script> --trace-jsonl <path> --exit-after-script
cargo test -p ui_counter_runtime
focused engine tests required by any touched engine UI path
cargo test --workspace if the app product proof touches cross-crate workspace behavior
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Evidence expectation: Phase 012 activation must prove the current app/product surface and command path inventory, define human and agent proof artifacts, and identify focused tests that prove source/program/action/mutation/evaluation/render/trace facts without direct host mutation shortcuts.

Stop conditions: stop if the product cannot be launched by command, if proof-local `ui_app_integration` becomes the public runtime owner, if agent mode mutates host state directly, if source/program/runtime/action/mutation/render facts cannot be proven, if unrelated render/backend/source-reload/persistence behavior is needed, or if exact allowed/forbidden files cannot be recorded before implementation.

Known blockers: Phase 012 is not implementation-authorized. Phase 013 and Phase 014 remain blocked until Phase 012 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded Phase 012 active-implementation authorization branch/PR from current `main` after this closeout merges. Do not patch `apps/ui_counter_runtime` or product runtime code until that authorization records exact allowed files, forbidden files, validation, evidence, and stop conditions.

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
