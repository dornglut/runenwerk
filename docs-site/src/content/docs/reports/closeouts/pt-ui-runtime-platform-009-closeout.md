---
title: PT-UI-RUNTIME-PLATFORM-009 Closeout
description: Closeout evidence for the SurfaceFrame Generic Producer Boundary phase.
status: completed
owner: render
layer: reports
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/production-tracks.md
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/decision-register.md
  - ../../workspace/routines/track-orchestration-routine.md
  - ../../workspace/routines/phase-completion-drift-check-routine.md
  - ../../workspace/complete-merge-readiness-gate.md
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
---

# PT-UI-RUNTIME-PLATFORM-009 Closeout

ID: `PT-UI-RUNTIME-PLATFORM-009`

Title: `SurfaceFrame Generic Producer Boundary`

Completed on: 2026-07-07

Owner: render frame/submission contracts

## Contract promised

Phase 009 promised one bounded SurfaceFrame Generic Producer Boundary PR.

Promised migration map:

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

Promised scope:

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

Forbidden scope:

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

## Contract delivered

PR #97 merged Phase 009 into `main` at merge commit
`50e2dbdf1f9c076f4a76a04543274801d1f1649b`.

PR head before merge:
`79d5c595da1401821e3c0edd472d6796755b8e6e`.

Delivered scope:

```text
RenderFrameProducerId is now the producer identity for the producer-facing surface-frame submission seam.
UiFrameProducerId was removed from the accepted render API id seam.
SurfaceFrameSubmission, SurfaceFrameSubmissionOrder, SurfaceFrameRoute, and SurfaceFrameSubmissionRegistryResource replaced the UI-named ownership seam.
PreparedSurfaceFrameSubmission replaced PreparedUiFrameSubmission while PreparedUiFrameContribution and PreparedUiFrameResource stayed UI render-feature payload/resource names.
SurfaceFrameSubmissionRenderOutputProof replaced the UI-named render output proof.
RenderPlugin initializes and consumes SurfaceFrameSubmissionRegistryResource.
Scene/debug overlay and app/editor producers compile against the generic seam as named migration inputs.
```

## Files changed

PR #97 changed exactly:

```text
apps/runenwerk_draw/src/runtime/plugin.rs
apps/runenwerk_draw/src/runtime/systems.rs
apps/runenwerk_draw/tests/app_shell.rs
apps/runenwerk_editor/src/runtime/app.rs
apps/runenwerk_editor/src/runtime/plugin.rs
apps/runenwerk_editor/src/runtime/systems/frame_submit.rs
apps/runenwerk_editor/src/runtime/ui_gallery.rs
apps/runenwerk_editor/tests/startup_render_smoke.rs
apps/runenwerk_editor/tests/viewport_architecture_guards.rs
engine/src/plugins/render/api/ids.rs
engine/src/plugins/render/features/ui/prepared.rs
engine/src/plugins/render/features/ui/render_output_proof.rs
engine/src/plugins/render/features/ui/resource.rs
engine/src/plugins/render/features/ui/submission.rs
engine/src/plugins/render/frame/mod.rs
engine/src/plugins/render/plugin.rs
engine/src/plugins/render/runtime/ui_submission.rs
```

Diff stat:

```text
17 files changed, 159 insertions(+), 151 deletions(-)
```

## Validation run

Command validation run before merge on PR head
`79d5c595da1401821e3c0edd472d6796755b8e6e`:

```text
cargo fmt
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine --test render_flow_v2
cargo test -p runenwerk_draw --test app_shell
cargo test -p runenwerk_editor --test startup_render_smoke
cargo test -p runenwerk_editor --test viewport_architecture_guards
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
gh pr checks 97
```

Results:

```text
cargo fmt: passed.
cargo test -p engine surface_frame_submission: passed; 3 focused tests passed.
cargo test -p engine render_output_proof: passed; 1 focused test passed.
cargo test -p engine --test render_flow_v2: passed; 15 integration tests passed.
cargo test -p runenwerk_draw --test app_shell: passed; 48 tests passed.
cargo test -p runenwerk_editor --test startup_render_smoke: passed; 2 tests passed.
cargo test -p runenwerk_editor --test viewport_architecture_guards: passed; 56 tests passed.
cargo test -p engine: passed; 261 tests passed with the existing ignored GPU timing runtime-evidence test, integration tests, and doctests clean.
python tools/docs/validate_docs.py: passed.
git diff --check: passed.
git diff --check main...HEAD: passed.
git status --short --branch: clean on the Phase 009 branch before merge.
git diff --stat main...HEAD: 17 files changed, 159 insertions, 151 deletions.
gh pr checks 97: no checks reported for the branch.
```

Validation note:

```text
cargo test -p engine render_flow_v2 matched 0 tests because the argument is a test-name filter. It was superseded by cargo test -p engine --test render_flow_v2, which ran the integration suite.
```

Validation unavailable:

```text
No GitHub Actions check evidence was available because GitHub reported no checks for the branch.
```

## Evidence classes used

| Claim | Evidence class | Source / command / artifact | Freshness | Confidence | Decision impact |
|---|---|---|---|---|---|
| Phase 009 PR merged into `main` | E6 | PR #97 metadata and merge commit `50e2dbdf1f9c076f4a76a04543274801d1f1649b` | current on 2026-07-07 | high | authorizes closeout |
| Delivered file scope matches Phase 009 contract | E3 | PR #97 file list and `git diff --stat main...HEAD` before merge | current PR head | high | supports completion |
| Producer identity is generic at the accepted seam | E3 + E5 | `RenderFrameProducerId`, `SurfaceFrameSubmission`, and focused submission tests | current PR head | high | supports completion |
| Prepared UI render-feature payloads are built from generic surface-frame submissions | E3 + E5 | `PreparedSurfaceFrameSubmission`, `prepare_ui_feature_resource_system`, and render flow tests | current PR head | high | supports completion |
| Render output proof uses the generic surface-frame seam | E3 + E5 | `SurfaceFrameSubmissionRenderOutputProof` and focused proof test | current PR head | high | supports completion |
| App/editor producers compile against the generic seam | E3 + E5 | draw/editor runtime callsites and focused app tests | current PR head | high | supports completion |
| Later-phase forbidden scope is absent | E3 | changed-file inspection and forbidden-scope proof | current PR head | high | supports completion |
| Phase 009 was authorized by accepted planning/design authority | E8 | active-work, roadmap, production-tracks, cutover plan, architecture docs | current `main` before merge | high | supports merge and closeout |
| Validation and authority align | E9 | changed files, focused tests, local commands, and accepted Phase 009 contract agree | current PR head | high | supports completion |

Highest evidence class reached: `E9`.

## Merge readiness status

Merge readiness was satisfied before merge.

Merge readiness evidence:

```text
Scope: ready; changed files matched the Phase 009 contract.
Authority: ready; active-work, roadmap, production-track, cutover plan, and architecture docs authorized Phase 009 only.
Principles: ready; the migration removes duplicate UI-specific ownership names and reuses the existing generic RenderFrameProducerId/RenderSurfaceId seam.
Maintainability: ready; producer identity, surface identity, prepared UI payloads, and render-output proof remain separated by responsibility.
Validation: ready locally; no hosted checks were configured/reported.
Lifecycle: ready for merge with closeout required before Phase 010 implementation.
Review: ready; PR #97 had no comments, reviews, review requests, or unresolved findings when inspected.
Merge mechanics: ready; PR #97 was mergeable with merge state CLEAN.
Post-merge truth: this closeout records completion truth before any Phase 010 implementation starts.
```

Branch cleanup:

```text
PR #97 merged with squash merge.
Remote branch codex/pt-ui-runtime-platform-009-surface-frame-boundary was deleted by the merge command.
Local checkout fast-forwarded to main at 50e2dbdf1f9c076f4a76a04543274801d1f1649b.
```

## Boundary and non-goal evidence

Preserved non-goals:

```text
no engine/src/plugins/ui/** changes
no apps/ui_counter_runtime product packaging
no UiPlugin render publication implementation
no scene/debug overlay retirement or behavioral migration beyond type-name callsite updates
no source reload or persistence implementation
no SDF or SpatialCanvas implementation
no render backend rewrite, graph execution rewrite, or shader changes
no broad ui_render_data primitive/model rewrite
no source/program/action semantic changes
no foundation/meta
no domain/app_program
no generic plugin framework
no phase spec validator implementation
no tools/docs validator or script changes
```

Dependency boundary:

```text
No new crates or dependencies were added.
RenderPlugin consumes generic surface-frame submissions without owning UiScreen, IntoUi, actions, host mutation, or route policy.
UiPlugin render publication remains deferred to Phase 010.
```

## Complete gate status

Complete investigation gate status: complete for Phase 009 through the accepted
runtime-platform investigation/design authority, Phase 008 closeout evidence,
activation-time source inspection, current PR file inspection, and validation of
the generic surface-frame seam.

Complete design gate status: complete for Phase 009 through the accepted full
cutover plan, active-work contract, roadmap entry, production-track entry,
principle compliance matrix, and module decomposition map.

Phase completion drift check: complete in this closeout. Planning still showed
Phase 009 as active implementation after PR #97 merged; this closeout resolves
that drift and opens Phase 010 as active planning only.

## Known gaps

Intentional downstream gaps:

```text
UiPlugin render publication remains unimplemented until Phase 010
scene/debug overlay migration remains unimplemented until Phase 011
runtime Counter app product remains unimplemented until Phase 012
source reload and persistence remain unimplemented until Phase 013
adoption lock remains unimplemented until Phase 014
```

These are not Phase 009 blockers.

## Drift found

Planning drift found after merge:

```text
active-work.md, roadmap.md, production-tracks.md, completed-work.md, and decision-register.md still needed Phase 009 post-merge truth after PR #97 merged.
```

Implementation drift found:

```text
No forbidden Phase 010-014 implementation scope was found in PR #97.
```

## Follow-up

Next safe action:

```text
Open PT-UI-RUNTIME-PLATFORM-010 as active planning only.
Do not start Phase 010 implementation until this closeout/planning truth is merged and a separate active-implementation authorization confirms the exact Phase 010 files, focused tests, validation commands, evidence requirements, and stop conditions from accepted Markdown authority.
```

## Evidence links

```text
PR #97: https://github.com/Crystonix/Runenwerk/pull/97
Phase 009 head commit: 79d5c595da1401821e3c0edd472d6796755b8e6e
Phase 009 merge commit: 50e2dbdf1f9c076f4a76a04543274801d1f1649b
Active work: ../../workspace/planning/active-work.md
Roadmap: ../../workspace/planning/roadmap.md
Production track: ../../workspace/planning/production-tracks.md
Completed work: ../../workspace/planning/completed-work.md
Decision register: ../../workspace/planning/decision-register.md
Cutover plan: ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
Runtime architecture: ../../architecture/live-uiplugin-runtime-platform-architecture.md
```
