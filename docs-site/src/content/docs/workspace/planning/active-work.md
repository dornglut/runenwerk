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
  - ../specs/pt-ui-runtime-platform-012.ron
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

State: active implementation authorization for exactly one bounded Phase 012 implementation PR after this authorization record merges.

Lifecycle state: `active-implementation` for Phase 012 only.

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
docs-site/src/content/docs/workspace/specs/pt-ui-runtime-platform-012.ron
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` current source/design/planning inspection by path, `E5` local command validation for completed Phase 011 and this activation PR docs validation, `E6` PR #104 implementation merge metadata and PR #105 closeout merge metadata, `E8` accepted architecture/workflow/planning authority, and `E9` Phase 011 code/test plus validation plus authority alignment from the closeout report.

Complete investigation gate: complete for Phase 012 implementation authorization. Current source inspection found no `apps/ui_counter_runtime` crate, found root workspace membership for existing apps only, inspected `App::run`, `App::headless`, `run_for_frames`, existing app crate patterns, `UiPlugin` mount/source/action/evaluation/trace/frame-publication APIs, current focused engine tests, and the proof-local `ui_app_integration` crate boundary.

Complete design gate: complete for Phase 012 implementation authorization through the accepted cutover plan, runtime architecture, Phase 011 closeout evidence, this active-work contract, the Phase 012 phase spec, principle compliance matrix, module decomposition map, validation envelope, evidence expectations, and stop conditions.

Implementation authorization status: authorized after this activation record merges. The next implementation PR must stay inside this Phase 012 contract and must not include Phase 013, Phase 014, source reload/persistence, SDF, SpatialCanvas, render backend, graph, shader, or generic framework work.

Phase spec: `docs-site/src/content/docs/workspace/specs/pt-ui-runtime-platform-012.ron`.

Phase 011 completion truth:

```text
PR #104 merged into main at 15e213a08dbf79f65e0851fe5be9f853f157b48b.
PR head before merge: a6232278e41202cd331051f347d3db892988f38c.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-011-closeout.md.
Closeout PR #105 merged into main at 29966360b8ef7f49d5cb324d41bc61d18d23f8cd.
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
capabilities: counter.write for mutations, counter.read for rendering count
normal app authors must not see route maps, event packets, host adapters, or render registries
```

Phase 012 source/path inventory:

```text
Cargo.toml:
  workspace members include apps/runenwerk_editor, apps/runenwerk_draw, and apps/runenwerk_runtime_preview
  apps/ui_counter_runtime is absent and must be added as a new workspace member

apps/runenwerk_runtime_preview/src/main.rs:
  existing app command pattern parses --headless and uses App::headless / App::run_for_frames for headless operation

engine/src/app/domain/app.rs and engine/src/app/runtime/lifecycle.rs:
  App::new, App::headless, add_plugin, insert_resource, set_title, world/world_mut, run, run_for_frames, and run_for_ticks are public composition hooks

engine/src/plugins/ui/app_ext.rs and mount.rs:
  app.mount_ui and app.ui().mount record mounted sessions but only store screen identity today

engine/src/plugins/ui/screen.rs and source.rs:
  UiScreen, IntoUi, UiTypedSource, lowering, and UiRuntimeEvaluationInput provide the typed source/program path

engine/src/plugins/ui/action.rs, host.rs, events.rs, report.rs, resources.rs, trace.rs:
  dispatch_ui_action, UiActionHandler, UiHostActionExecutor, UiRuntimeEvaluationResource, UiRuntimeTraceResource, and reports provide the typed action/mutation/evaluation/trace path

engine/src/plugins/ui/render_publish.rs:
  UiPlugin publishes evaluated frames through SurfaceFrameSubmissionRegistryResource for RenderPlugin to consume generically

engine/tests/ui_mount_api.rs, ui_typed_contracts.rs, ui_action_dispatch.rs, ui_runtime_evaluation.rs, ui_render_publication.rs:
  current focused tests prove each piece separately but do not provide a runnable product crate

domain/ui/ui_app_integration:
  proof-local ECS-backed counter evidence exists, but crate docs state it must not become the public runtime owner

docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md:
  current-facts table still named the retired render-owned scene/debug collector; this activation corrects that stale authority fact using Phase 011 closeout evidence before authorizing implementation
```

Phase 012 handoff contract:

```text
create the ui_counter_runtime app crate and add it to the workspace
provide a ui_counter_runtime binary whose default window title is Runenwerk UI Counter Runtime
install RenderPlugin, UiPlugin, and a product-owned CounterPlugin
CounterPlugin owns Counter { value: i64 }, action descriptors, host intents, host-owned mutation, source/evaluation systems, and product trace/JSONL export wiring
CounterPlugin must mount CounterScreen through app.mount_ui(CounterScreen)
CounterScreen must implement UiScreen/IntoUi through accepted typed source/program contracts and expose header, count, Increment, Decrement, Reset, trace console, and status line
agent script mode must resolve semantic actions through the same route/capability/payload/host dispatch path as human interaction
human interaction must use the same dispatch path; if pointer/window integration cannot be implemented inside allowed scope, stop and update authority before widening
runtime/evaluator output must change after accepted host mutation
UiPlugin must publish the evaluated frame through the existing generic surface-frame seam and RenderPlugin must consume it without UI semantic ownership
JSONL trace output must record action, mutation, evaluation, and frame publication facts
manual human and agent run commands plus observed behavior must be recorded in the PR
do not use ui_app_integration as the product runtime owner or app runtime dependency
```

Allowed files/crates:

```text
Cargo.toml only to add apps/ui_counter_runtime as a workspace member and workspace dependency entries only if strictly required
apps/ui_counter_runtime/Cargo.toml
apps/ui_counter_runtime/src/**
apps/ui_counter_runtime/tests/**
assets/ui_counter_runtime/scripts/increment_reset.ron
engine/src/plugins/ui/action.rs only for narrow public-path fixes required by the Counter product proof
engine/src/plugins/ui/app_ext.rs only for narrow mounting ergonomics required by CounterScreen
engine/src/plugins/ui/diagnostics.rs only for narrow diagnostics required by product proof
engine/src/plugins/ui/events.rs only for narrow event-to-action product proof wiring
engine/src/plugins/ui/host.rs only for narrow host-owned mutation proof fixes
engine/src/plugins/ui/mod.rs only for exports required by the product crate
engine/src/plugins/ui/plugin.rs only for scheduling/resource wiring required by product proof
engine/src/plugins/ui/render_publish.rs only for narrow publication evidence fixes, not render ownership changes
engine/src/plugins/ui/report.rs only for narrow report accessors needed by JSONL/proof output
engine/src/plugins/ui/resources.rs only for mounted source/evaluation/trace product wiring
engine/src/plugins/ui/schedule.rs only for a named UiRuntimeSet if needed by product systems
engine/src/plugins/ui/screen.rs only for narrow typed screen contract fixes
engine/src/plugins/ui/source.rs only for narrow typed source/evaluation contract fixes
engine/src/plugins/ui/trace.rs only for generic UI runtime trace JSONL/proof accessors
engine/tests/ui_counter_runtime_product.rs or similarly named focused engine test only if an engine UI public-path gap cannot be proven in the app crate
docs-site/src/content/docs/reports/proofs/pt-ui-runtime-platform-012-*.md only if the implementation PR records durable proof artifacts in docs
```

Forbidden files/crates:

```text
source reload/persistence implementation
SDF or SpatialCanvas implementation
render backend rewrite beyond product proof consumption
graph execution rewrite or shader changes
source/program/action semantic rewrites
host mutation or action-dispatch semantic rewrites
scene/debug overlay producer migration work
broad render-data primitive/model rewrites
domain/ui/ui_app_integration/** changes
ui_app_integration runtime dependency from apps/ui_counter_runtime
engine/src/runtime/** window runner changes
engine/src/plugins/input/** input system changes
apps/runenwerk_editor/**
apps/runenwerk_draw/**
apps/runenwerk_runtime_preview/**
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

Validation envelope:

```text
cargo fmt --check
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

Evidence expectation: the implementation PR must include current source inspection, focused command validation, PR metadata/check evidence, forbidden-scope proof, human and agent command output, JSONL trace artifact evidence, and explicit proof that app authors use `RenderPlugin`, `UiPlugin`, `CounterPlugin`, and `app.mount_ui(CounterScreen)` without route maps, event packets, host adapters, or render registries. Highest expected evidence class before merge is `E9` when source/test/runtime proof, local validation, and accepted authority align.

Principle compliance matrix:

| Principle | Required Phase 012 evidence | Stop signal |
|---|---|---|
| KISS | One product crate composes existing engine/UI APIs and a narrow CounterPlugin. | A new framework, second runtime, or generic app platform enters the PR. |
| DRY | Counter action descriptors, routes, and capabilities have one product-owned source of truth. | Separate human and agent route/action definitions diverge. |
| YAGNI | Only Counter product, proof commands, and generic UI trace access needed by the product are added. | Source reload, persistence, SDF, backend, graph, shader, or broad renderer work appears. |
| SOLID | App/plugin owns Counter state and mutation; UiPlugin owns generic UI runtime; RenderPlugin consumes generic frames. | Generic controls, renderer, or agent scripts mutate Counter directly. |
| Separation of Concerns | Product app, engine UI runtime fixes, assets/scripts, and tests stay in their owning files. | Product behavior is embedded in engine runtime or proof-local ui_app_integration owns public runtime. |
| Avoid Premature Optimization | Whole-frame publication and existing evaluation are enough for product proof. | Incremental redraw/cache claims are added without Phase 013 authority. |
| Law of Demeter | Product uses direct public APIs; normal app code does not reach into render registries or host route maps. | App authors must manually edit registries, route maps, event packets, or render submissions. |

Module decomposition map:

| Module / file | Responsibility | Public API exported | Tests proving it | Split trigger |
|---|---|---|---|---|
| `apps/ui_counter_runtime/src/main.rs` | CLI mode selection, human/headless command entry, trace output path handling. | Binary only. | `cargo run -p ui_counter_runtime`, app crate tests. | CLI parsing becomes reusable framework or persistence enters. |
| `apps/ui_counter_runtime/src/lib.rs` and product modules | CounterPlugin, Counter resource, CounterScreen, actions, script runner, proof helpers. | App crate API only as needed for tests. | `cargo test -p ui_counter_runtime`. | Product code starts duplicating engine UiPlugin contracts. |
| `assets/ui_counter_runtime/scripts/increment_reset.ron` | Canonical agent script fixture. | None. | Headless run command and app tests. | Multiple script languages or persistence fixtures appear. |
| `engine/src/plugins/ui/**` allowed files | Narrow generic runtime access/wiring needed to make public product path real. | Only direct UiPlugin/public-path additions required by Counter app. | Focused engine tests plus app crate tests. | Changes touch input/runtime/render ownership outside UiPlugin. |
| `docs-site/src/content/docs/reports/proofs/**` optional | Durable human/agent proof report if PR body is insufficient. | Docs only. | Docs validation. | It becomes planning authority or closeout content before merge. |

Stop conditions: stop if the product cannot launch by command; if implementation needs `engine/src/runtime/**`, `engine/src/plugins/input/**`, render backend/graph/shader, source reload/persistence, SDF, SpatialCanvas, or generic framework work; if `ui_app_integration` becomes the product runtime owner; if human and agent paths use different route/capability/payload/host checks; if agent mode mutates Counter directly; if normal app authors must see route maps/event packets/host adapters/render registries; if source/program/runtime/action/mutation/render facts cannot be proven; or if validation cannot be reported honestly.

Known blockers: Phase 013 and Phase 014 remain blocked until Phase 012 is implemented, reviewed, merged, and completion truth is recorded.

Next action: after this activation PR merges, create exactly one bounded `PT-UI-RUNTIME-PLATFORM-012 — Runtime Counter App Product` implementation branch/PR from current `main`. Keep it draft until the Phase 012 validation and required docs/diff/status commands are clean. Do not start Phase 013.

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
