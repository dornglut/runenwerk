---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/ui-framework-architecture.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../design/active/ui-framework-app-integration-direction-review.md
  - ../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
  - ../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
  - ../../design/active/ui-component-platform-overlay-popup-layering-design.md
  - ../../design/active/ui-component-platform-text-editing-design.md
  - ../../design/active/ui-component-platform-generic-text-design.md
  - ../../design/active/ui-component-platform-surface2d-design.md
---

# Roadmap

This is the Markdown-first roadmap record for scriptless workflow.

## Current entries

### PT-UI-FRAMEWORK-APP-INTEGRATION-001

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-001`

Title: UI Framework App Integration Direction Review

State: accepted direction; no longer the current implementation focus

Lifecycle state: `accepted-direction`

Authority: `ui-framework-app-integration-direction-review.md`.

Evidence: PR #70 accepted the direction that App/Plugin/ECS-hosted UI must lower through `ui_definition`, `UiProgram`, `UiStory`, runtime/evaluator artifacts, and host-owned mutation instead of continuing the manual `app_program` proof or promoting SpatialCanvas as the app-framework answer.

Next action: Keep as accepted direction authority. The first proof slice, `PT-UI-FRAMEWORK-APP-INTEGRATION-002`, is completed and now serves as evidence for runtime-platform planning.

### PT-UI-FRAMEWORK-APP-INTEGRATION-002

ID: `PT-UI-FRAMEWORK-APP-INTEGRATION-002`

Title: ECS-backed Counter UI Story Proof

State: completed through PR #72 and closeout truth

Lifecycle state: `completed`

Authority: `ecs-backed-counter-ui-story-proof-planning.md`.

Evidence: PR #72 merged the `ui_app_integration` proof into `main` at `e093eb1affdc465b96430200960f8e3cdca0d26b`. Closeout evidence records code-authored Counter and Win source records, lowering through `ui_definition` and `ui_program`, route/event evidence, route-missing diagnostics, route-resolved host mutation, ECS-backed Counter mutation, positive proof flow, fail-closed cases, no public `AppUiExt`, no engine `UiPlugin`, no render adapter/runtime-visible render proof, no SDF/SpatialCanvas world-space implementation, no `foundation/meta`, no `domain/app_program`, and no generic plugin framework.

Next action: Keep as completed proof evidence. Do not reopen this slice unless future inspection finds the recorded boundary was violated.

### PT-UI-RUNTIME-PLATFORM-001

ID: `PT-UI-RUNTIME-PLATFORM-001`

Title: Live UiPlugin Runtime and Generic Surface-Frame Rendering

State: completed design-gate hardening through merged PR #74; implementation not authorized by this entry

Lifecycle state: `completed` design-gate hardening / superseded by full cutover-plan authority

Authority:

```text
../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
../../architecture/ui-framework-architecture.md
../../design/active/ui-framework-app-integration-direction-review.md
../../reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
```

Evidence: PR #74 merged the docs-only investigation/design-gate hardening. The accepted target remains `RenderPlugin + UiPlugin + AppPlugin`, `app.mount_ui(Screen)`, typed `UiScreen`, typed `IntoUi`, typed `UiActionHandler` / `TryUiActionHandler`, host-owned mutation, reuse of existing `ui_surface` and `ui_hosts` owners, and staged generic surface-frame render publication. Runtime Rust implementation remains outside PR #74.

Gate status:

```text
Complete investigation gate: complete for PR #74 design-gate hardening.
Complete design gate: complete for opening implementation planning.
Implementation authorization: still forbidden by this entry.
```

Next action: Use as authority for `PT-UI-RUNTIME-PLATFORM-002`, the completed full platform cutover plan.

### PT-UI-RUNTIME-PLATFORM-002

ID: `PT-UI-RUNTIME-PLATFORM-002`

Title: Live UiPlugin Runtime Full Platform Cutover Plan

State: completed / accepted full-platform cutover planning through merged PR #76

Lifecycle state: `completed` docs-only planning; implementation not authorized by this entry

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
../../architecture/ui-framework-architecture.md
```

Evidence: PR #76 merged the full cutover-plan docs into `main` at merge commit `1697942c968afd9648872c202972826dc4c406b2`. The platform is planned as a full cutover, then implemented through bounded phase PRs: UiPlugin foundation, app mounting API, typed screen/source/action contracts, mounted surface/session runtime, host action dispatch with UI-runtime trace, runtime evaluation with state snapshot and invalidation, producer-generic surface-frame boundary, UiPlugin render publication, scene/debug overlay producer migration/retirement, runnable human/agent Counter app product, source reload and persistence contract, and closeout/adoption lock. SDF UI backend work is assigned to downstream `PT-UI-RENDER-BACKEND-SDF-001`.

Gate status:

```text
Complete investigation gate: inherited from PT-UI-RUNTIME-PLATFORM-001 and extended with render/app-engine feature mapping.
Complete design gate: accepted for the full cutover contract through PR #76.
Implementation authorization: forbidden until the next phase opens separately with exact active-implementation scope.
```

Next action: Keep as upstream cutover authority. The Phase 003 implementation follow-up is fulfilled by PR #79; use this entry as context for Phase 004 planning.

### PT-WORKFLOW-TRACK-ORCHESTRATION-001

ID: `PT-WORKFLOW-TRACK-ORCHESTRATION-001`

Title: Track Orchestration and Phase Spec Handoff Workflow

State: completed through merged PR #77 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../routines/track-orchestration-routine.md
../task-cards/track-manager-task.md
../specs/phase-implementation-spec.md
../specs/templates/phase-implementation-spec.ron
../authority-model.md
../planning/active-work.md
../planning/decision-register.md
```

Evidence: PR #77 merged the missing track-manager workflow layer into `main` at `8b7a6b558bef79303e66d6a9f329dc71e00a0931`. It formalizes that a one-shot track goal is valid as manager intent, but each implementation agent receives exactly one bounded phase. It also defines RON phase specs as structured handoff contracts derived from accepted Markdown authority and reserves JSONL for append-only trace/log streams. Closeout evidence lives in `../../reports/closeouts/pt-workflow-track-orchestration-001-closeout.md`.

Gate status:

```text
Complete investigation gate: satisfied by workflow authority inspection for docs-only process hardening.
Complete design gate: satisfied for the workflow docs/spec handoff contract.
Implementation authorization: completed as workflow authority only; runtime implementation remains owned by separate runtime phases.
```

Next action: Keep as completed workflow evidence. The Phase 003 implementation follow-up is fulfilled by PR #79; use the routine and phase-spec rules for Phase 004 planning and later phases.

### PT-UI-RUNTIME-PLATFORM-003

ID: `PT-UI-RUNTIME-PLATFORM-003`

Title: UiPlugin Foundation

State: completed through merged PR #79 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../design/active/live-uiplugin-runtime-and-surface-frame-rendering-design.md
../../reports/investigations/live-uiplugin-runtime-current-state-investigation.md
../../reports/closeouts/pt-workflow-track-orchestration-001-closeout.md
../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Implementation contract:

```text
Create only the engine UiPlugin foundation shell: module root, UiPlugin install/build behavior, schedule labels, default resources, report shell, diagnostics shell, plugin export wiring, and focused engine tests for install/resource initialization.
```

Evidence:

```text
PR #79 merged into main at 0135850277e904b4be2c336e3ef6507b3fc88b72.
Delivered files: engine/src/plugins/mod.rs, engine/src/plugins/ui/*, and engine/tests/ui_plugin_foundation.rs.
Validation: cargo test -p engine ui_plugin, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md.
```

Known non-goals:

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

Gate status:

```text
Complete investigation gate: complete through PT-UI-RUNTIME-PLATFORM-001 and PT-UI-RUNTIME-PLATFORM-002 authority.
Complete design gate: complete for Phase 003 through the accepted full cutover plan and Phase 003 contract.
Merge readiness: satisfied before PR #79 merged; no hosted checks were reported for the branch.
```

Next action: Keep Phase 003 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-004 — App Mounting API` planning.

### PT-UI-RUNTIME-PLATFORM-004

ID: `PT-UI-RUNTIME-PLATFORM-004`

Title: App Mounting API

State: completed through merged PR #82 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-003-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Implementation contract from accepted cutover authority:

```text
Implement public app mounting ergonomics after Phase 003 closeout truth is merged.
The normal path records a mount request without manual surface factory setup.
The advanced app.ui().mount path records the same mount request with explicit configuration/reporting hooks.
Mount diagnostics include screen identity, mount source, and stable failure reason.
Normal users are not exposed to route maps, event packets, host adapters, or render registries.
```

Delivered files:

```text
engine/src/plugins/ui/app_ext.rs
engine/src/plugins/ui/mount.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/mod.rs
engine/src/prelude.rs
engine/tests/ui_mount_api.rs
```

Forbidden files/crates:

```text
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

Evidence:

```text
PR #82 merged into main at 9fb86f0d426385be7e425ff943c7a9d5450e1edb.
Validation: cargo test -p engine ui_mount, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md.
```

Gate status:

```text
Complete investigation gate: complete through accepted runtime-platform authority and Phase 004 closeout evidence.
Complete design gate: complete for Phase 004 through the accepted full cutover plan and Phase 004 contract.
Merge readiness: satisfied before PR #82 merged; no hosted checks were reported for the branch.
```

Next action: Keep Phase 004 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-005 — Typed Screen / Source / Action Contracts` planning.

### PT-UI-RUNTIME-PLATFORM-005

ID: `PT-UI-RUNTIME-PLATFORM-005`

Title: Typed Screen / Source / Action Contracts

State: completed through PR #85 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-004-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Planning contract from accepted cutover authority:

```text
Typed screens lower to ui_definition-compatible source records.
Typed source produces route/source-map facts.
Typed action handlers emit host-owned mutation intent.
Action identity is stable and diagnostic-friendly.
ui_app_integration remains proof evidence, not final framework owner.
```

Allowed files/crates for a future Phase 005 implementation authorization:

```text
engine/src/plugins/ui/screen.rs
engine/src/plugins/ui/source.rs
engine/src/plugins/ui/action.rs
engine/src/plugins/ui/host.rs
engine/src/plugins/ui/diagnostics.rs
engine/Cargo.toml dependency additions for selected domain/ui crates
focused engine tests plus comparison evidence from ui_app_integration where useful
```

Forbidden files/crates:

```text
mounted session runtime
host action dispatch runtime
runtime trace implementation
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

Completion evidence:

```text
PR #85 merged into main at 6226470defa7a72a567fc03c1bc3783e63e2c2c8.
Validation: cargo test -p engine ui_typed, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-005-closeout.md.
```

Gate status:

```text
Complete investigation gate: complete through accepted runtime-platform authority and Phase 005 closeout evidence.
Complete design gate: complete for Phase 005 through the accepted full cutover plan and Phase 005 contract.
Merge readiness: satisfied before PR #85 merged; no hosted checks were reported for the branch.
```

Next action: Keep Phase 005 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-006 — Mounted Surface Session Runtime` planning.

### PT-UI-RUNTIME-PLATFORM-006

ID: `PT-UI-RUNTIME-PLATFORM-006`

Title: Mounted Surface Session Runtime

State: completed through PR #88 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-005-closeout.md
../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Planning contract from accepted cutover authority:

```text
mount creates a MountedSurfaceInstance-compatible record
session identity, host identity, generation, retention, and diagnostics are recorded
unmount/remount behavior is deterministic
multiple mounted screens/surfaces do not collide
no duplicate surface/session semantic model is invented inside engine
```

Delivered files:

```text
Cargo.lock
engine/Cargo.toml
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/mount.rs
engine/tests/ui_mount_api.rs
```

Forbidden files/crates:

```text
world-space UI implementation
SDF or SpatialCanvas implementation
product/editor/game semantics in domain UI
replacing ui_surface instead of adapting to it
host action dispatch runtime
runtime trace implementation
render publication or render adapter code
SurfaceFrame generic producer boundary implementation code
scene/debug overlay producer migration implementation code
source reload/persistence implementation code
apps/ui_counter_runtime implementation
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Gate status:

```text
Complete investigation gate: complete through accepted runtime-platform authority and Phase 006 closeout evidence.
Complete design gate: complete for Phase 006 through the accepted full cutover plan and Phase 006 closeout evidence.
Merge readiness: satisfied before PR #88 merged; no hosted checks were reported for the branch.
```

Completion evidence:

```text
PR #88 merged into main at 82d6f00326cf2823eb91d3f655a730b962b355f6.
Validation: cargo test -p engine ui_mount, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md.
```

Next action: Keep Phase 006 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-007 - Host Action Dispatch and Runtime Trace` planning.

### PT-UI-RUNTIME-PLATFORM-007

ID: `PT-UI-RUNTIME-PLATFORM-007`

Title: Host Action Dispatch and Runtime Trace

State: active implementation authorization recorded; implementation PR not yet opened

Lifecycle state: `active-implementation`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Planning contract from accepted cutover authority:

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

Allowed files/crates for the Phase 007 implementation authorization:

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

Forbidden files/crates:

```text
runtime evaluation/state snapshot/invalidation implementation
render publication or render adapter code
SurfaceFrame generic producer boundary implementation code
scene/debug overlay producer migration implementation code
source reload/persistence implementation code
apps/ui_counter_runtime implementation
world-space UI implementation
SDF or SpatialCanvas implementation
product/editor/game semantics in generic UI
Counter-specific trace model
engine-wide trace framework
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Gate status:

```text
Complete investigation gate: complete for active implementation through accepted runtime-platform authority and Phase 006 closeout evidence.
Complete design gate: complete for implementation through the accepted cutover plan and Phase 006 closeout evidence.
Implementation authorization: active for exactly one bounded Phase 007 implementation PR.
```

Validation envelope:

```text
cargo test -p engine ui_action
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Next action: Open exactly one bounded Phase 007 implementation branch/PR after this planning authorization merges. Keep it draft until focused Phase 007 validation and required docs/diff/status commands are clean.

### PT-UI-COMPONENT-PLATFORM-013

ID: `PT-UI-COMPONENT-PLATFORM-013`

Title: Overlay / Popup / Layering full implementation

State: completed through merged PR #44

Lifecycle state: `completed`

Evidence: PR #44 merged into `main` at `6f2d3827f315191d7aeaf68a64f523627197cad8`. Evidence covers package-backed overlay declarations, base-control overlay lowering, main-path package validation, catalog projection, inspection projection, normalized input fact consumption, runtime overlay proof, proof-frame projection, static mount proof, route-guard evidence, and full local validation gate passed on 2026-07-02.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-014

ID: `PT-UI-COMPONENT-PLATFORM-014`

Title: Text Editing / Editable Text Behavior

State: completed through merged PR #46

Lifecycle state: `completed`

Authority: `ui-component-platform-text-editing-design.md`.

Evidence: PR #46 merged into `main` at `6d9bf983c77a32c701681ff55a05e1f9ebcdeed1`. Main contains package-backed editable-text declarations, InspectorField text-editing lowering, package descriptor wiring, package validation, catalog projection, inspection projection, normalized text edit/composition/selection facts, runtime text-editing proof, proof-frame projection, static mount validation, focused tests, final proof-frame cleanup, and full local validation gate passed on 2026-07-02.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-015

ID: `PT-UI-COMPONENT-PLATFORM-015`

Title: Generic Text

State: completed through baseline PR #48 and hardening PR #49

Lifecycle state: `completed`

Authority: `ui-component-platform-generic-text-design.md`.

Evidence: PR #48 merged into `main` at `91cea8b8f0dfc38143de77ba931bc81ffc91dcff`. PR #49 merged into `main` at `338a8092d534dbb412da89363d50a46cd5efeae9` and completed the hardening pass. Final validation passed with the recorded package/workspace/docs/diff gate.

Next action: Keep as completed dependency.

### PT-UI-COMPONENT-PLATFORM-016

ID: `PT-UI-COMPONENT-PLATFORM-016`

Title: Surface2D

State: completed through docs-hardening PR #62 and implementation PR #61

Lifecycle state: `completed`

Authority: `ui-component-platform-surface2d-design.md`.

Evidence: PR #62 merged docs-only workflow, principle, decomposition, and merge-readiness hardening at `6cfb82b81aa5478496ff6cbf3fa2eea607777aaf`. PR #61 squash-merged the Phase 16 Surface2D implementation at `2e803620c91726fb599c5e5c4eee4b3984cd4a9d`. Post-merge validation from `main` passed with the recorded Surface2D focused commands, workspace tests, docs validation, and diff check.

Next action: Keep as completed dependency. Keep Phase 17 SpatialCanvas as downstream planning only until runtime platform planning settles its accepted implementation path.

## Future app-framework follow-ups

These are downstream planning candidates only. They are not implementation work and must wait for their own accepted planning/design contracts.

- `PT-UI-FRAMEWORK-APP-INTEGRATION-003 - Public AppUiExt Ergonomics`
- `PT-UI-FRAMEWORK-APP-INTEGRATION-004 - Authoring Frontends and Execution Strategy Model`

`PT-UI-FRAMEWORK-APP-INTEGRATION-003` is superseded/absorbed by `PT-UI-RUNTIME-PLATFORM-001` and `PT-UI-RUNTIME-PLATFORM-002` as the broader live runtime platform track. Do not treat standalone public `AppUiExt` ergonomics as the immediate next implementation target.

`PT-UI-FRAMEWORK-APP-INTEGRATION-004` should define how Rust builders, templates, visual designer output, compiler DSLs, immediate-mode adapters, reactive adapters, retained execution, ECS-driven execution, and SDF/world-space targets share source/program/event/story contracts through the accepted route and proof model.

## Rules

- Markdown must be enough to understand the current state.
- Completed work belongs in `completed-work.md`.
- Deferred work belongs in `deferred-work.md`.
- Strategic track order belongs in `production-tracks.md`.
- Use `../workflow-lifecycle.md` before changing lifecycle state.
- Accepted direction does not authorize implementation.
