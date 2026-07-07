---
title: Live UiPlugin Runtime Full Platform Cutover Plan
description: Full implementation-planning contract for the Live UiPlugin Runtime Platform cutover.
status: active
owner: ui
layer: design
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ./live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ./ui-framework-app-integration-direction-review.md
  - ../../architecture/ui-framework-architecture.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/decision-register.md
  - ../../workspace/complete-investigation-gate.md
  - ../../workspace/complete-design-gate.md
  - ../../workspace/complete-merge-readiness-gate.md
  - ../../workspace/evidence-quality-taxonomy.md
  - ../../guidelines/programming-principles.md
---

# Live UiPlugin Runtime Full Platform Cutover Plan

ID: `PT-UI-RUNTIME-PLATFORM-002`

Lifecycle state: `active-planning` full-platform cutover contract draft.

Implementation status: not started and not authorized by this planning PR.

## Purpose

This document turns the accepted `PT-UI-RUNTIME-PLATFORM-001` direction into a full implementation program. The next step is not a narrow `UiPlugin` skeleton plan. The next step is a complete cutover contract that is then executed through bounded implementation PRs.

Required position:

```text
Plan the whole Live UiPlugin Runtime Platform cutover now.
Implement it later through gated phase PRs.
Do not start runtime Rust work from this docs-only planning PR.
```

## Accepted basis

Authority comes from `PT-UI-RUNTIME-PLATFORM-001`:

```text
engine-owned UiPlugin runtime layer
reuse existing domain/ui contracts
app.mount_ui(Screen) as the normal app-authoring path
typed UiScreen / IntoUi / UiActionHandler / TryUiActionHandler
host-owned mutation through ui_hosts-compatible boundaries
mounted surface/session state through ui_surface-compatible boundaries
runtime/evaluator-backed output
UiPlugin-published frame submission
RenderPlugin consumes producer frame data without owning UI semantics
surface-frame genericization staged after the live runtime path is proven
```

This plan decomposes that accepted direction into implementation phases, evidence gates, validation, and stop conditions.

## Global invariants

| Invariant | Rule |
|---|---|
| Engine owns app/plugin composition | `engine::plugins::ui` may integrate with `App` and `Plugin`; domain UI crates must not depend on engine. |
| Domain UI owns UI semantics | Source, program, surface, host, evaluator, runtime-view, and frame contracts stay in their existing domain crates unless a later phase proves a small ownership adjustment. |
| App/host owns mutation | Generic UI controls, renderer code, and domain UI contracts must not mutate app state directly. |
| Render consumes output | RenderPlugin may consume producer/surface/frame facts; it must not own `UiScreen`, `IntoUi`, action routing, host mutation, or source/program semantics. |
| Public API stays ergonomic | Normal app authors use `app.mount_ui(Screen)` and typed handlers, not route maps, event packets, host adapters, or render registries. |
| Compatibility is explicit | Existing scene/debug overlay behavior must either remain unchanged or be migrated through named compatibility producers with tests. |
| Genericization is staged | `SurfaceFrame` vocabulary must not become a broad rename unless scoped, proven, and validated. |

## Phase map

| Phase | ID | Title | Main result |
|---|---|---|---|
| 002 | `PT-UI-RUNTIME-PLATFORM-002` | Full Platform Cutover Plan | This docs-only implementation-planning contract. |
| 003 | `PT-UI-RUNTIME-PLATFORM-003` | UiPlugin Foundation | Engine UI plugin module, resources, schedule/report shell. |
| 004 | `PT-UI-RUNTIME-PLATFORM-004` | App Mounting API | `app.mount_ui(Screen)` and `app.ui().mount(Screen)` record typed mount requests. |
| 005 | `PT-UI-RUNTIME-PLATFORM-005` | Typed Screen / Source / Action Contracts | `UiScreen`, `IntoUi`, `UiActionHandler`, and `TryUiActionHandler` lower into existing domain contracts. |
| 006 | `PT-UI-RUNTIME-PLATFORM-006` | Mounted Surface Session Runtime | Live mount/session registry using `ui_surface` contracts. |
| 007 | `PT-UI-RUNTIME-PLATFORM-007` | Host Action Dispatch | Typed event/action queue, host-owned mutation, fail-closed diagnostics. |
| 008 | `PT-UI-RUNTIME-PLATFORM-008` | Runtime Evaluation to Frame | Mounted screens produce evaluator/runtime-view-backed frame output. |
| 009 | `PT-UI-RUNTIME-PLATFORM-009` | UiPlugin Render Publication | UiPlugin publishes frame submissions; RenderPlugin consumes without owning UI semantics. |
| 010 | `PT-UI-RUNTIME-PLATFORM-010` | Scene/Debug Overlay Compatibility Cutover | Existing hardcoded overlay producers become explicit compatibility producers or remain unchanged with documented boundary. |
| 011 | `PT-UI-RUNTIME-PLATFORM-011` | SurfaceFrame Genericization | Producer-generic surface-frame vocabulary staged from current UI-specific submission names. |
| 012 | `PT-UI-RUNTIME-PLATFORM-012` | Live Counter App Proof | End-to-end app/plugin/mount/action/render proof using public API. |
| 013 | `PT-UI-RUNTIME-PLATFORM-013` | Closeout and Adoption Lock | Validation truth, merge readiness, remaining gaps, and next-track activation rules. |

## Per-phase contracts

### Phase 003 — UiPlugin Foundation

Allowed scope:

```text
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/schedule.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/mod.rs
engine/Cargo.toml only if justified
focused engine tests for plugin installation/resource initialization
```

Required evidence:

```text
UiPlugin installs without panicking
plugin install is idempotent or reports duplicate install deterministically
resources have stable default state
schedule labels exist without render/backend ownership changes
```

Stop if this requires `foundation/meta`, `domain/app_program`, a generic plugin framework, domain UI depending on engine, or a render backend rewrite.

### Phase 004 — App Mounting API

Allowed scope:

```text
engine/src/plugins/ui/app_ext.rs
engine/src/plugins/ui/mount.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/prelude.rs if accepted for public export
focused engine tests/examples proving app.mount_ui and app.ui().mount compile
```

Required evidence:

```text
normal path records a mount request without manual surface factory setup
advanced path records the same mount request with explicit configuration/reporting hooks
mount diagnostics include screen identity, mount source, and stable failure reason
normal users are not exposed to route maps, event packets, host adapters, or render registries
```

Stop if the common path needs manual host adapters, manual route maps, manual render submission writes, or private App internals that are not part of the accepted engine API.

### Phase 005 — Typed Screen / Source / Action Contracts

Allowed scope:

```text
engine/src/plugins/ui/screen.rs
engine/src/plugins/ui/source.rs
engine/src/plugins/ui/action.rs
engine/src/plugins/ui/host.rs
engine/src/plugins/ui/diagnostics.rs
engine/Cargo.toml dependency additions for selected domain/ui crates
focused engine tests plus comparison evidence from ui_app_integration where useful
```

Required evidence:

```text
typed screen lowers to ui_definition-compatible source records
typed source produces route/source-map facts
typed action handler emits host-owned mutation intent
action identity is stable and diagnostic-friendly
ui_app_integration remains proof evidence, not final framework owner
```

Stop if typed UI skips source/program facts, if generic controls mutate app state directly, or if a new broad runtime-platform domain crate becomes necessary.

### Phase 006 — Mounted Surface Session Runtime

Allowed scope:

```text
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/mount.rs
engine/src/plugins/ui/report.rs
engine/Cargo.toml dependency on ui_surface if not already present
focused engine tests for mount/unmount/generation/session reports
```

Required evidence:

```text
mount creates a MountedSurfaceInstance-compatible record
session identity, host identity, generation, retention, and diagnostics are recorded
unmount/remount behavior is deterministic
multiple mounted screens/surfaces do not collide
no duplicate surface/session semantic model is invented inside engine
```

Stop if this requires world-space UI, SDF, SpatialCanvas, product/editor/game semantics in domain UI, or replacing `ui_surface` instead of adapting to it.

### Phase 007 — Host Action Dispatch

Allowed scope:

```text
engine/src/plugins/ui/events.rs
engine/src/plugins/ui/action.rs
engine/src/plugins/ui/host.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/Cargo.toml dependency on ui_hosts if not already present
focused positive and negative engine tests
```

Required evidence:

```text
known action mutates only through app/host owner
unknown route does not mutate
schema mismatch does not mutate
capability mismatch does not mutate
payload mismatch does not mutate
missing host data does not mutate
action report records route/action/host/failure reason
```

Stop if errors become silent, partial mutation is possible on invalid input, or product/editor/game semantics move into generic UI.

### Phase 008 — Runtime Evaluation to Frame

Allowed scope:

```text
engine/src/plugins/ui/source.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/Cargo.toml dependencies on selected evaluator/runtime-view/render-data crates
focused engine tests for output facts and frame payload creation
```

Required evidence:

```text
mounted screen source/program facts feed evaluator/runtime-view path
Counter output text changes after host mutation
frame payload is derived from runtime/evaluator output
runtime report includes source, program, runtime-view, output, and diagnostics facts
visible output is not reported as success without upstream facts
```

Stop if frame output skips source/program/evaluator evidence, if a new execution strategy is invented without accepted design, or if renderer primitives become UI source truth.

### Phase 009 — UiPlugin Render Publication

Allowed scope:

```text
engine/src/plugins/ui/render_publish.rs
engine/src/plugins/ui/report.rs
existing render submission registry/resource paths only where needed
focused engine/render integration tests
```

Required evidence:

```text
UiPlugin publishes frame submission with producer id and surface identity
RenderPlugin consumes prepared payload without querying UiScreen, IntoUi, actions, host mutation, or route policy
render contribution is deterministic for the same runtime frame
missing UiPlugin frame reports a diagnostic instead of silent success
```

Stop if RenderPlugin becomes the UI runtime owner, pulls from app host state directly, or needs a broad backend rewrite.

### Phase 010 — Scene/Debug Overlay Compatibility Cutover

Allowed scope:

```text
engine/src/plugins/ui/compat_scene_overlay.rs
engine/src/plugins/ui/compat_debug_overlay.rs
specific existing render collection paths only if migration is explicitly scoped
focused compatibility tests proving existing overlay behavior
```

Required evidence:

```text
existing scene overlay output is unchanged or migration difference is documented
existing debug overlay output is unchanged or migration difference is documented
compatibility producer identity is explicit
RenderPlugin no longer permanently owns UI semantic producer collection unless a blocker is recorded
```

Stop if compatibility migration changes unrelated render behavior or creates a second UI runtime.

### Phase 011 — SurfaceFrame Genericization

Allowed scope:

```text
only the render frame/submission contracts named by the phase PR
only ui_render_data names/types touched by the accepted migration map
compatibility aliases only if sunset criteria are recorded
focused migration tests and compile checks
```

Required evidence:

```text
migration map lists every renamed type/module/function
old and new producer paths remain deterministic during transition
UiPlugin remains one producer, not the generic surface-frame owner
scene/debug compatibility producers still work
external docs no longer imply RenderPlugin owns UI semantics
```

Stop if the rename becomes broad/unreviewable, compatibility aliases become permanent without sunset plan, or source/program/action semantics must change.

### Phase 012 — Live Counter App Proof

Allowed scope:

```text
focused engine tests/examples for Counter app
engine/src/plugins/ui/** fixes only if required by proof and scoped
planning docs and proof report fixtures
```

Required evidence:

```text
public app/plugin setup is represented in test/proof form
CounterPlugin uses app.mount_ui(CounterScreen)
CounterScreen implements the accepted typed screen/source/action model
user action dispatch mutates Counter only through host-owned path
runtime/evaluator output changes after mutation
UiPlugin publishes render submission
RenderPlugin consumes the submission without UI semantic ownership
positive proof and fail-closed negative proof are both present
```

Stop if the proof cannot show source/program/runtime/action/mutation/render facts or if it needs proof-local `ui_app_integration` as the public runtime owner.

### Phase 013 — Closeout and Adoption Lock

Allowed scope:

```text
docs-site/src/content/docs/reports/closeouts/**
docs-site/src/content/docs/workspace/planning/active-work.md
docs-site/src/content/docs/workspace/planning/roadmap.md
docs-site/src/content/docs/workspace/planning/production-tracks.md
docs-site/src/content/docs/workspace/planning/decision-register.md
owning design docs only for final truth and known gaps
```

Required evidence:

```text
all implementation phase PRs and merge commits are recorded
final validation commands and results are recorded
remaining gaps are explicit
next-track activation is explicit
legacy/compatibility shims have owners and sunset criteria
```

Stop if final validation cannot be proven, planning truth disagrees with merged code, or next implementation starts before closeout truth is recorded.

## Cross-phase dependency graph

```text
003 UiPlugin foundation
  -> 004 App mounting API
  -> 005 Typed screen/source/action contracts
  -> 006 Mounted surface/session runtime
  -> 007 Host action dispatch
  -> 008 Runtime evaluation to frame
  -> 009 UiPlugin render publication
  -> 010 Scene/debug overlay compatibility cutover
  -> 011 SurfaceFrame genericization
  -> 012 Live Counter app proof
  -> 013 Closeout and adoption lock
```

`010` may start after `009` if existing scene/debug producer behavior is understood. `011` must not start before `009` proves the live producer path. `012` may accumulate evidence from earlier phases but must not be declared complete until render publication and fail-closed action dispatch are proven through the public app-facing path.

## Validation envelope

Every implementation phase must run the smallest focused crate/test set required by its scope plus docs and diff hygiene. The default final gate is:

```text
cargo test -p engine <focused phase tests>
phase-owned domain crate tests when touched or depended on
cargo test --workspace before broad runtime/render migration or final proof
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Do not claim validation that was not run.

## Merge-readiness report shape

Every implementation PR must report:

```text
branch/head SHA
base SHA or refreshed-from-main status
exact files changed
exact modules/functions/sections changed
validation commands run and results
proof/evidence artifacts added
stop conditions checked
known blockers and deferrals
next phase recommendation
```

## Current planning PR acceptance criteria

This planning PR is acceptable when it records:

```text
full phase map from planning through closeout
per-phase purpose, allowed files/crates, evidence, validation, and stop conditions
global invariants
cross-phase dependency graph
explicit correction from first-slice-only planning to full-platform cutover planning
planning-file state transition to PT-UI-RUNTIME-PLATFORM-002
no runtime implementation
```

## Next action after this PR merges

Open `PT-UI-RUNTIME-PLATFORM-003 — UiPlugin Foundation` as the first implementation PR. That PR must stay inside Phase 003 boundaries and must not opportunistically implement the whole platform.
