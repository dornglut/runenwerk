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
  - ./roadmap.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Active Work

This file names the current planning focus for scriptless workflow. It stays short and points to the owning investigation/design records instead of duplicating them.

## Current focus

ID: `PT-UI-RUNTIME-PLATFORM-009`

Title: `SurfaceFrame Generic Producer Boundary`

State: active-implementation authorization recorded for one bounded Phase 009 PR. No runtime code is changed by this planning record.

Lifecycle state: `active-implementation` for Phase 009 only.

Owner: render frame/submission contracts and `ui_render_data` own the producer-generic surface/frame vocabulary at the accepted seam. `engine::plugins::ui` remains the source/program/evaluation owner and may consume the downstream seam only after the boundary exists. RenderPlugin consumes generic producer/surface/frame packets without owning UI semantics.

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
docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-008-closeout.md
docs-site/src/content/docs/reports/investigations/live-uiplugin-runtime-current-state-investigation.md
docs-site/src/content/docs/reports/closeouts/pt-ui-framework-app-integration-002-closeout.md
docs-site/src/content/docs/design/active/live-uiplugin-runtime-full-cutover-plan.md
docs-site/src/content/docs/architecture/live-uiplugin-runtime-platform-architecture.md
docs-site/src/content/docs/architecture/ui-framework-architecture.md
```

Evidence classes: `E3` source/design/planning inspection by path, `E5` local command validation for completed Phase 008, `E6` PR #95 merge/check metadata, `E8` accepted architecture/workflow/planning authority, and `E9` code/test plus validation plus authority alignment in the closeout report.

Complete investigation gate: complete for opening Phase 009 active implementation. Phase 009 inherits the completed `PT-UI-RUNTIME-PLATFORM-001` investigation, the `PT-UI-RUNTIME-PLATFORM-002` render/app-engine feature mapping, Phase 008 closeout evidence, and activation-time source inspection of the current UI-named render submission seam, generic `RenderFrameProducerId`, prepared frame packets, app producer callsites, and render feature preparation path.

Complete design gate: complete for Phase 009 implementation through the accepted cutover plan, architecture record, Phase 008 closeout, and this planning authorization record.

Implementation authorization status: `active-implementation-authorized`.

Phase 008 completion truth:

```text
PR #94 merged into main at be5b790e38b7f80ad17092fa0cb75e87eef4d849.
Closeout report: docs-site/src/content/docs/reports/closeouts/pt-ui-runtime-platform-008-closeout.md.
Remote phase branch was deleted by the merge command.
```

Phase 009 handoff contract from accepted cutover authority:

```text
rename the producer-facing UI frame submission seam to generic SurfaceFrame submission names
use existing generic RenderFrameProducerId for producer identity; remove UiFrameProducerId from the accepted seam
keep UiFrame as the renderer-neutral UI payload type from ui_render_data, not as producer ownership truth
preserve existing render surface identity through RenderSurfaceId
keep scene/debug overlay submission as named migration input only; do not retire or redesign it
focused migration tests and compile checks prove the generic seam and downstream callsites
```

Required Phase 009 evidence from accepted cutover authority:

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
PreparedUiFrameContribution stays named as the UI render-feature payload because it contains ui_render_data::UiFrame primitives and is not producer ownership truth
PreparedUiFrameResource stays named as the prepared UI render-feature resource
UiFrameSubmissionRenderOutputProof -> SurfaceFrameSubmissionRenderOutputProof
prepare_ui_feature_resource_system keeps its name but consumes SurfaceFrameSubmissionRegistryResource
collect_runtime_ui_frame_submissions_system keeps its name because scene/debug overlay migration is Phase 011; it must write through SurfaceFrameSubmissionRegistryResource
```

Principle compliance matrix:

```text
KISS: Phase 009 should rename or introduce only the accepted producer-generic seam, not rewrite the renderer backend.
DRY: Phase 009 must remove duplicate UI-specific render ownership names at the seam instead of adding a parallel path.
YAGNI: Phase 009 must not publish UiPlugin frames, migrate overlays, package the Counter app, add reload/persistence, or start SDF/world-space work.
SOLID: producer identity, surface identity, frame packet shape, and runtime UI semantics must remain separately owned.
Separation of Concerns: RenderPlugin consumes generic frame packets; UiPlugin remains outside Phase 009 publication until Phase 010.
Avoid Premature Optimization: no backend rewrite or performance claim belongs in the boundary rename/migration phase.
Law of Demeter: RenderPlugin should depend on generic producer/surface/frame contracts, not `UiScreen`, `IntoUi`, actions, host mutation, or route policy.
```

Allowed files and crates for the Phase 009 implementation authorization:

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

Forbidden files and crates:

```text
engine/src/plugins/ui/** except future Phase 010 publication integration
apps/ui_counter_runtime product packaging
scene/debug overlay retirement or behavioral migration
source reload/persistence implementation
SDF or SpatialCanvas implementation
render backend rewrite, graph execution rewrite, or shader changes
domain/ui/ui_render_data primitive/model rewrites beyond import/type reference adjustments required by the migration map
source/program/action semantics
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

Maintainability review status: complete for Phase 009 authorization. Stop if implementation needs files outside this migration map or cannot preserve existing behavior while renaming the accepted seam.

Feature support matrix:

```text
UiPlugin install/resource shell: completed by Phase 003.
Public mounting API: completed by Phase 004.
Typed screen/source/action contracts: completed by Phase 005.
Mounted sessions: completed by Phase 006.
Host action dispatch and trace: completed by Phase 007.
Runtime evaluation/invalidation: completed by Phase 008.
SurfaceFrame generic producer boundary: active-implementation Phase 009.
UiPlugin render publication: downstream Phase 010.
Scene/debug overlay migration: downstream Phase 011.
Runtime Counter product: downstream Phase 012.
Reload/persistence: downstream Phase 013.
Closeout/adoption lock: downstream Phase 014.
```

Phase 009 validation envelope from cutover and workflow authority:

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

Evidence expectation: focused tests and compile checks must prove `RenderFrameProducerId` is the producer identity for SurfaceFrame submissions, `RenderSurfaceId` remains the surface identity, `SurfaceFrameSubmissionRegistryResource` replaces the UI-named registry at the producer-facing seam, prepared UI render-feature payloads are built from generic surface-frame submissions, scene/debug overlay and app producers are named migration inputs that compile against the generic seam, RenderPlugin consumes prepared packets without `UiScreen`, `IntoUi`, actions, host mutation, or route policy, and no hidden parallel UI submission path remains.

Stop conditions: stop if the rename becomes broad or unreviewable, source/program/action semantics change, the phase becomes a render backend rewrite, genericization creates a second runtime path, compatibility shims become the durable public API instead of the generic seam, UiPlugin render publication enters the PR, scene/debug overlay migration/retirement enters the PR, or Phase 010+ files become necessary.

Known blockers: no Phase 009 implementation branch has been opened or merged yet. Phase 010 and later remain blocked until Phase 009 is reviewed, merged, and completion truth is recorded.

Next action: create exactly one bounded `PT-UI-RUNTIME-PLATFORM-009 - SurfaceFrame Generic Producer Boundary` implementation branch/PR from current `main` after this planning truth is merged. Keep the PR draft until focused Phase 009 validation and the required docs/diff/status commands are clean.

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
