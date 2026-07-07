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

State: completed through merged PR #91 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-006-closeout.md
../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md
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

Delivered files/crates:

```text
engine/src/plugins/ui/events.rs
engine/src/plugins/ui/action.rs
engine/src/plugins/ui/host.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/trace.rs
focused positive and negative engine tests in engine/tests/ui_action_dispatch.rs
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
Complete investigation gate: completed for Phase 007 through accepted runtime-platform authority and closeout evidence.
Complete design gate: completed for Phase 007 through the accepted cutover plan and closeout evidence.
Implementation authorization: completed; no further Phase 007 implementation PR is open.
```

Completion evidence:

```text
PR #91 merged into main at 5dd90a2caf1bb7e4d5710830499df1d122fe587f.
Validation: cargo test -p engine ui_action, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md.
```

Next action: Keep Phase 007 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-008 - Runtime Evaluation, State Snapshot, and Invalidation` planning.

### PT-UI-RUNTIME-PLATFORM-008

ID: `PT-UI-RUNTIME-PLATFORM-008`

Title: Runtime Evaluation, State Snapshot, and Invalidation

State: completed through merged PR #94 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-007-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Planning contract from accepted cutover authority:

```text
mounted screen source/program facts feed evaluator/runtime-view path
Counter output text changes after host mutation
frame payload is derived from runtime/evaluator output
runtime report includes source, program, runtime-view, output, diagnostics, and invalidation facts
UI session snapshot/replay is stable by source/runtime IDs
dirty records name source, host-data, session, layout, text, theme, primitive, surface, and render-publication causes
trace adds runtime evaluation and state/invalidation facts
```

Delivered files/crates for the Phase 008 implementation:

```text
engine/src/plugins/ui/source.rs
engine/src/plugins/ui/resources.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/trace.rs
engine/Cargo.toml dependencies on ui_artifacts, ui_binding, ui_evaluator, ui_runtime_view, and ui_state; engine already has ui_runtime and ui_render_data
focused engine tests in engine/tests/ui_runtime_evaluation.rs, named for `cargo test -p engine ui_runtime_evaluation`
```

Forbidden files/crates:

```text
render publication or render adapter code
SurfaceFrame generic producer boundary implementation code
scene/debug overlay producer migration implementation code
source reload/persistence implementation code
apps/ui_counter_runtime product packaging
world-space UI implementation
SDF or SpatialCanvas implementation
product/editor/game semantics in generic UI
renderer primitives as UI source truth
new execution strategy without accepted design
per-element incremental rendering claims without dirty-scope proof
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Gate status:

```text
Complete investigation gate: complete for active implementation through accepted runtime-platform authority, Phase 007 closeout evidence, and activation-time source inspection of existing evaluator/runtime-view/render-data contracts.
Complete design gate: complete for implementation through the accepted cutover plan and Phase 007 closeout evidence.
Implementation authorization: completed through merged PR #94. No further Phase 008 implementation PR is authorized.
```

Validation envelope:

```text
cargo test -p engine ui_runtime_evaluation
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git status --short --branch
git diff --stat main...HEAD
```

Completion evidence:

```text
PR #94 merged into main at be5b790e38b7f80ad17092fa0cb75e87eef4d849.
Validation: cargo test -p engine ui_runtime_evaluation, cargo test -p engine, cargo fmt, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-008-closeout.md.
```

Next action: Keep Phase 008 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-009 - SurfaceFrame Generic Producer Boundary` planning.

### PT-UI-RUNTIME-PLATFORM-009

ID: `PT-UI-RUNTIME-PLATFORM-009`

Title: SurfaceFrame Generic Producer Boundary

State: completed through merged PR #97 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-008-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Planning contract from accepted cutover authority:

```text
migration map lists every renamed type/module/function
producer-generic names replace UI-specific render ownership at the accepted seam before UiPlugin publishes durable frames
producer id and surface identity are generic concepts, not UiPlugin concepts
RenderPlugin consumes generic producer/surface/frame packets
scene/debug paths remain named as migration inputs, not hidden parallel paths
external docs no longer imply RenderPlugin owns UI semantics
```

Activation-time migration map:

```text
UiFrameProducerId -> RenderFrameProducerId
UiFrameSubmission -> SurfaceFrameSubmission
UiFrameSubmissionOrder -> SurfaceFrameSubmissionOrder
UiFrameSubmissionRegistryResource -> SurfaceFrameSubmissionRegistryResource
UiFrameRoute -> SurfaceFrameRoute
PreparedUiFrameSubmission -> PreparedSurfaceFrameSubmission
PreparedUiFrameContribution stays named as the UI render-feature payload
PreparedUiFrameResource stays named as the prepared UI render-feature resource
UiFrameSubmissionRenderOutputProof -> SurfaceFrameSubmissionRenderOutputProof
prepare_ui_feature_resource_system keeps its name but consumes SurfaceFrameSubmissionRegistryResource
collect_runtime_ui_frame_submissions_system keeps its name and writes through SurfaceFrameSubmissionRegistryResource
```

Allowed files/crates for the Phase 009 implementation authorization:

```text
engine/src/plugins/render/api/ids.rs
engine/src/plugins/render/features/ui/submission.rs
engine/src/plugins/render/features/ui/prepared.rs
engine/src/plugins/render/features/ui/resource.rs
engine/src/plugins/render/features/ui/render_output_proof.rs
engine/src/plugins/render/features/ui/mod.rs only if public exports need adjustment
engine/src/plugins/render/frame/mod.rs
engine/src/plugins/render/frame/contributions.rs only for renamed prepared submission payload references
engine/src/plugins/render/frame/packet.rs only for renamed prepared submission payload references
engine/src/plugins/render/plugin.rs
engine/src/plugins/render/runtime/ui_submission.rs
engine/src/plugins/render/runtime/frame_prepare.rs only if renamed resource/type references require adjustment
engine/src/plugins/render/renderer/prepare.rs only if renamed prepared payload references require adjustment
apps/runenwerk_draw/src/runtime/plugin.rs
apps/runenwerk_draw/src/runtime/systems.rs
apps/runenwerk_draw/tests/app_shell.rs
apps/runenwerk_editor/src/runtime/app.rs
apps/runenwerk_editor/src/runtime/plugin.rs
apps/runenwerk_editor/src/runtime/systems/frame_submit.rs
apps/runenwerk_editor/src/runtime/ui_gallery.rs
apps/runenwerk_editor/tests/startup_render_smoke.rs
apps/runenwerk_editor/tests/viewport_architecture_guards.rs
engine tests or focused render-feature tests needed to prove the migration map
```

Forbidden files/crates:

```text
UiPlugin render publication implementation
scene/debug overlay producer migration or retirement implementation
source reload/persistence implementation
apps/ui_counter_runtime product packaging
SDF or SpatialCanvas implementation
source/program/action semantic changes
broad render backend rewrite
second runtime path
compatibility shims as the durable public API instead of the generic seam
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Gate status:

```text
Complete investigation gate: complete for active implementation through accepted runtime-platform authority, Phase 008 closeout evidence, and activation-time source inspection of current UI-named render submission contracts, generic RenderFrameProducerId, prepared frame packets, app producer callsites, and render feature preparation path.
Complete design gate: complete for implementation through the accepted cutover plan, architecture record, Phase 008 closeout evidence, and this activation record.
Implementation authorization: completed through PR #97.
```

Validation envelope:

```text
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine render_flow_v2
cargo test -p runenwerk_draw app_shell
cargo test -p runenwerk_editor startup_render_smoke
cargo test -p runenwerk_editor viewport_architecture_guards
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
```

Completion evidence:

```text
PR #97 merged into main at 50e2dbdf1f9c076f4a76a04543274801d1f1649b.
Validation: cargo fmt, focused surface_frame_submission and render_output_proof tests, render_flow_v2 integration tests, draw/editor focused tests, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md.
```

Next action: Keep Phase 009 as completed evidence. Use the closeout as the prerequisite for `PT-UI-RUNTIME-PLATFORM-010 - UiPlugin Render Publication` planning.

### PT-UI-RUNTIME-PLATFORM-010

ID: `PT-UI-RUNTIME-PLATFORM-010`

Title: UiPlugin Render Publication

State: completed through merged PR #101 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-009-closeout.md
../../reports/closeouts/pt-ui-runtime-platform-010-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Planning contract from accepted cutover authority:

```text
UiPlugin publishes frame submission with producer id and surface identity through the generic seam
RenderPlugin consumes prepared payload without querying UiScreen, IntoUi, actions, host mutation, or route policy
render contribution is deterministic for the same runtime frame
missing UiPlugin frame reports a diagnostic instead of silent success
frame publication trace records producer, surface, frame revision, dirty cause, and publication result
```

Delivered scope:

```text
UiPlugin owns a stable runtime frame producer id and publication target resource.
UiPlugin publishes the latest UiRuntimeEvaluationReport frame payload through SurfaceFrameSubmissionRegistryResource keyed by RenderFrameProducerId and RenderSurfaceId.
Publication reports, diagnostics, and UiFramePublished/UiFramePresented trace events are recorded.
Missing runtime evaluation records a diagnostic and removes stale UiPlugin surface-scoped submissions.
RenderPlugin scheduling orders generic UI feature preparation after UiRuntimeSet::RenderPublication without taking UI semantic ownership.
Focused tests prove direct publication, missing-evaluation diagnostics, stale-submission cleanup, and RenderPlugin/UiPlugin RenderPrepare integration.
```

Files changed:

```text
engine/src/plugins/render/features/ui/submission.rs
engine/src/plugins/render/plugin.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/render_publish.rs
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/schedule.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/trace.rs
engine/tests/ui_plugin_foundation.rs
engine/tests/ui_render_publication.rs
```

Completion evidence:

```text
PR #101 merged into main at 8d6c13146deab870dca5533204067249aa2c1b90.
PR head before merge: 79420eb642cecd43b495208f4749b2af0818bae5.
Diff stat: 11 files changed, 846 insertions, 3 deletions.
Validation: cargo fmt --check, focused ui_render_publication/ui_runtime_evaluation/surface_frame_submission/render_output_proof tests, render_flow_v2 integration tests, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-010-closeout.md.
```

Gate status:

```text
Complete investigation gate: complete for Phase 010 through accepted runtime-platform authority, Phase 009 closeout evidence, activation-time source inspection, PR #101 file inspection, and validation.
Complete design gate: complete for Phase 010 through the accepted cutover plan, architecture record, active-work contract, activation correction, and closeout evidence.
Merge readiness: satisfied before merge; no hosted checks were configured/reported.
```

Known downstream gaps:

```text
scene/debug overlay producer migration and retirement remains Phase 011
runtime Counter app product remains Phase 012
source reload and persistence remains Phase 013
adoption lock remains Phase 014
```

Next action: Keep Phase 010 as completed evidence. Phase 011 activation now consumes this closeout as its prerequisite and authorizes exactly one bounded Scene/Debug Overlay Producer Migration and Retirement implementation PR after the activation record merges.

### PT-UI-RUNTIME-PLATFORM-011

ID: `PT-UI-RUNTIME-PLATFORM-011`

Title: Scene/Debug Overlay Producer Migration and Retirement

State: completed through merged PR #104 and closeout truth

Lifecycle state: `completed`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-010-closeout.md
../../reports/closeouts/pt-ui-runtime-platform-011-closeout.md
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Delivered scope:

```text
ScenePlugin/scene-owned runtime code publishes the scene overlay frame through SurfaceFrameSubmissionRegistryResource with producer id 1, screen route, layer 0, priority 0, optional rect shader id lookup, and empty/no-manager removal.
DebugMetricsPlugin/debug-owned runtime code publishes or removes the debug metrics frame through SurfaceFrameSubmissionRegistryResource with producer id 2, screen route, layer 100, priority 0, and preserved UiOverlayState.debug_frame behavior.
RenderPlugin no longer imports, exports, or schedules collect_runtime_ui_frame_submissions_system.
engine/src/plugins/render/runtime/ui_submission.rs is deleted.
PreparedUiFrameResource receives scene/debug producer contributions through the generic seam when owner plugins run.
Focused migration and guard tests prove producer-owned publication, RenderPlugin collector retirement, scene/debug overlay preservation, and UiPlugin publication non-regression.
```

Files changed:

```text
engine/src/plugins/debug_metrics/mod.rs
engine/src/plugins/render/plugin.rs
engine/src/plugins/render/runtime/mod.rs
engine/src/plugins/render/runtime/ui_submission.rs
engine/src/plugins/scene/lifecycle/overlay_update.rs
engine/src/plugins/scene/plugin.rs
engine/tests/runtime_surface_guard.rs
engine/tests/runtime_ui_producer_migration.rs
```

Completion evidence:

```text
PR #104 merged into main at 15e213a08dbf79f65e0851fe5be9f853f157b48b.
PR head before merge: a6232278e41202cd331051f347d3db892988f38c.
Diff stat: 8 files changed, 329 insertions, 84 deletions.
Validation: cargo fmt --check, focused runtime_ui_producer_migration/scene/debug/surface/render-output/guard/UiPlugin publication tests, render_flow_v2 integration tests, cargo test -p engine, docs validation, diff hygiene, branch status, and diff stat passed before merge.
Closeout report: ../../reports/closeouts/pt-ui-runtime-platform-011-closeout.md.
```

Gate status:

```text
Complete investigation gate: complete for Phase 011 through accepted runtime-platform authority, Phase 010 closeout evidence, activation source/path inspection, PR #104 changed-file inspection, and validation.
Complete design gate: complete for Phase 011 through the accepted cutover plan, architecture record, active-work contract, Phase 011 spec, implementation evidence, and closeout evidence.
Merge readiness: satisfied before merge; no hosted checks were configured/reported.
```

Known downstream gaps:

```text
runtime Counter app product remains Phase 012
source reload and persistence remains Phase 013
adoption lock remains Phase 014
```

Next action: Keep Phase 011 as completed evidence. Use this closeout as the prerequisite evidence for the `PT-UI-RUNTIME-PLATFORM-012 — Runtime Counter App Product` activation record.

### PT-UI-RUNTIME-PLATFORM-012

ID: `PT-UI-RUNTIME-PLATFORM-012`

Title: Runtime Counter App Product

State: active implementation authorization for exactly one bounded Phase 012 implementation PR after this authorization record merges

Lifecycle state: `active-implementation`

Authority:

```text
../../design/active/live-uiplugin-runtime-full-cutover-plan.md
../../architecture/live-uiplugin-runtime-platform-architecture.md
../../reports/closeouts/pt-ui-runtime-platform-011-closeout.md
../specs/pt-ui-runtime-platform-012.ron
../routines/track-orchestration-routine.md
../routines/implementation-routine.md
../complete-investigation-gate.md
../complete-design-gate.md
../complete-merge-readiness-gate.md
../evidence-quality-taxonomy.md
```

Implementation contract:

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

Activation-time source/path inventory:

```text
apps/ui_counter_runtime is absent and must be added as a new workspace member.
Existing app crates are apps/runenwerk_editor, apps/runenwerk_draw, and apps/runenwerk_runtime_preview.
App composition uses App::new, App::headless, add_plugin, set_title, run, run_for_frames, and run_for_ticks.
UiPlugin provides app.mount_ui, typed screen/source/action contracts, host-owned dispatch, runtime evaluation, generic trace, and generic frame publication.
Focused engine tests prove each runtime piece separately but no runnable Counter product exists yet.
ui_app_integration is proof-local and must not become the product runtime owner.
Architecture current-facts drift about the retired scene/debug collector is corrected by the activation record before implementation starts.
```

Allowed files/crates:

```text
Cargo.toml only for the apps/ui_counter_runtime workspace member and strictly required workspace dependency entries
apps/ui_counter_runtime/**
assets/ui_counter_runtime/scripts/increment_reset.ron
engine/src/plugins/ui/** narrow public-path fixes required by the product proof
engine/tests/ui_counter_runtime_product.rs or similarly named focused engine test only for engine UI public-path gaps
optional docs-site/src/content/docs/reports/proofs/pt-ui-runtime-platform-012-*.md proof report
```

Forbidden files/crates:

```text
domain/ui/ui_app_integration/** changes
ui_app_integration runtime dependency from apps/ui_counter_runtime
engine/src/runtime/**
engine/src/plugins/input/**
apps/runenwerk_editor/**
apps/runenwerk_draw/**
apps/runenwerk_runtime_preview/**
render backend, graph, shader, source reload, persistence, SDF, SpatialCanvas, generic framework, phase spec validator, or docs validator work
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

Evidence expectation: source/test/runtime proof plus human and agent command output, JSONL trace artifact evidence, PR metadata/check evidence, forbidden-scope proof, and authority alignment. Highest expected evidence class before merge is `E9`.

Implementation authorization: authorized after this activation record merges. Open exactly one bounded Phase 012 implementation PR from current `main`; keep it draft until validation is clean. Do not start Phase 013.

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
