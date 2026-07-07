---
title: PT-UI-RUNTIME-PLATFORM-011 Closeout
description: Closeout evidence for the Scene/Debug Overlay Producer Migration and Retirement phase.
status: completed
owner: scene-debug-render
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
  - ../../workspace/specs/pt-ui-runtime-platform-011.ron
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
---

# PT-UI-RUNTIME-PLATFORM-011 Closeout

ID: `PT-UI-RUNTIME-PLATFORM-011`

Title: `Scene/Debug Overlay Producer Migration and Retirement`

Completed on: 2026-07-07

Owner: scene/debug producer-owned publication through
`SurfaceFrameSubmissionRegistryResource`; RenderPlugin remains the generic
preparation/submission owner.

## Contract promised

Phase 011 promised one bounded Scene/Debug Overlay Producer Migration and
Retirement PR.

Promised handoff contract:

```text
replace the scene overlay producer collection by a scene-owned generic SurfaceFrameSubmission publication path
replace the debug metrics overlay producer collection by a debug-owned generic SurfaceFrameSubmission publication path
remove RenderPlugin scheduling/import/export of collect_runtime_ui_frame_submissions_system
delete or fully retire engine/src/plugins/render/runtime/ui_submission.rs so RenderPlugin no longer owns UI semantic producer collection
preserve existing producer ids, route, order, shader-id behavior, empty-frame removal behavior, and prepared UI contribution behavior unless a focused test proves intentional retirement
do not alter UiPlugin render publication, source/program/action semantics, host mutation, route policy, render backend behavior, graph execution, shader code, or Counter product scope
prove no public manual add_ui_* registration chain is introduced or remains as a compatibility escape hatch
```

Promised scope:

```text
engine/src/plugins/render/runtime/ui_submission.rs
engine/src/plugins/render/runtime/mod.rs
engine/src/plugins/render/plugin.rs only to remove the render-owned legacy collection system import/schedule/export
engine/src/plugins/scene/plugin.rs only if scene-owned publication needs a RenderPrepare scheduling hook
engine/src/plugins/scene/lifecycle/overlay_update.rs only if scene-owned publication can be attached to existing scene overlay update without changing scene behavior
engine/src/plugins/scene/runtime/overlay_ui.rs only if scene-owned publication needs a narrow helper that preserves frame generation
engine/src/plugins/debug_metrics/mod.rs only for debug-owned publication/removal and tests
engine/src/state.rs only if UiOverlayState debug-frame storage is intentionally retired with evidence
engine/tests/runtime_ui_producer_migration.rs or a similarly named focused Phase 011 integration test
engine/tests/runtime_surface_guard.rs only for source-guard assertions
engine/src/plugins/scene/tests/scene_tests.rs only if integration tests cannot prove behavior
```

Forbidden scope:

```text
engine/src/plugins/ui/** runtime implementation changes
engine/src/plugins/render/runtime/frame_prepare.rs
engine/src/plugins/render/runtime/frame_submit.rs
engine/src/plugins/render/renderer/**
engine/src/plugins/render/graph/**
engine/src/plugins/render/backend/**
engine/src/plugins/render/shader/**
apps/ui_counter_runtime product packaging
source reload/persistence implementation
SDF or SpatialCanvas implementation
source/program/action semantic changes
host mutation or action-dispatch behavior changes
broad ui_render_data primitive/model rewrites
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
tools/docs validator or script changes
```

## Contract delivered

PR #104 merged Phase 011 into `main` at merge commit
`15e213a08dbf79f65e0851fe5be9f853f157b48b`.

PR head before merge:
`a6232278e41202cd331051f347d3db892988f38c`.

Delivered scope:

```text
Scene overlay publication moved into the scene-owned update path.
ScenePlugin initializes SurfaceFrameSubmissionRegistryResource for producer-owned publication.
scene_overlay_update_system publishes producer id 1 with route screen, layer 0, priority 0, the current scene overlay frame, and the existing optional rect shader id lookup.
scene_overlay_update_system removes producer id 1 when no scene manager exists or when the scene overlay frame is empty.
DebugMetricsPlugin initializes SurfaceFrameSubmissionRegistryResource for producer-owned publication.
debug_metrics_overlay_system publishes producer id 2 with route screen, layer 100, priority 0, and the current debug metrics frame.
debug_metrics_overlay_system removes producer id 2 when the debug overlay frame is empty while preserving UiOverlayState.debug_frame behavior.
Debug metrics publication runs in UiRuntimeSet::RenderPublication so RenderPlugin generic preparation sees the producer-owned frame in the same RenderPrepare pass.
RenderPlugin no longer imports, exports, or schedules collect_runtime_ui_frame_submissions_system.
engine/src/plugins/render/runtime/ui_submission.rs is deleted.
runtime_ui_producer_migration tests prove scene-owned publication, debug-owned publication, RenderPlugin collector retirement, and primary-surface ordering.
runtime_surface_guard now prevents reintroducing the retired render-owned collector path.
```

## Files changed

PR #104 changed exactly:

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

Diff stat:

```text
8 files changed, 329 insertions(+), 84 deletions(-)
```

## Validation run

Command validation run before merge on PR head
`a6232278e41202cd331051f347d3db892988f38c`:

```text
cargo fmt --check
cargo test -p engine runtime_ui_producer_migration
cargo test -p engine scene_registered_apps_publish_overlay_frame_with_buttons
cargo test -p engine debug_metrics_plugin_populates_overlay_draw_state
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine runtime_surface_guard
cargo test -p engine ui_render_publication
cargo test -p engine --test render_flow_v2
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
gh pr view 104 --json number,url,state,isDraft,headRefName,headRefOid,baseRefName,mergeable,reviewDecision,statusCheckRollup
```

Results:

```text
cargo fmt --check: passed.
cargo test -p engine runtime_ui_producer_migration: passed; 4 focused tests passed.
cargo test -p engine scene_registered_apps_publish_overlay_frame_with_buttons: passed; 1 focused scene overlay regression passed.
cargo test -p engine debug_metrics_plugin_populates_overlay_draw_state: passed; 1 focused debug overlay regression passed.
cargo test -p engine surface_frame_submission: passed; 4 generic registry tests passed. The command also matched one publication test by filter name.
cargo test -p engine render_output_proof: passed; 1 focused render-output proof test passed.
cargo test -p engine runtime_surface_guard: passed; 3 guard tests passed, including the retired collector guard.
cargo test -p engine ui_render_publication: passed; 4 UiPlugin publication regression tests passed.
cargo test -p engine --test render_flow_v2: passed; 15 integration tests passed.
cargo test -p engine: passed; engine lib reported 262 passed and 1 existing ignored GPU timing evidence test, with integration tests and doctests clean.
python tools/docs/validate_docs.py: passed.
git diff --check: passed.
git diff --check main...HEAD: passed.
git status --short --branch: clean on the Phase 011 implementation branch before merge.
git diff --stat main...HEAD: 8 files changed, 329 insertions, 84 deletions.
gh pr view 104: PR was ready for review, mergeable, head a6232278e41202cd331051f347d3db892988f38c, and had no status checks reported.
```

Validation unavailable:

```text
No GitHub Actions check evidence was available because GitHub reported no checks for the branch.
```

## Evidence classes used

| Claim | Evidence class | Source / command / artifact | Freshness | Confidence | Decision impact |
|---|---|---|---|---|---|
| Phase 011 PR merged into `main` | E6 | PR #104 metadata and merge commit `15e213a08dbf79f65e0851fe5be9f853f157b48b` | current on 2026-07-07 | high | authorizes closeout |
| Delivered file scope matches Phase 011 contract | E3 | PR #104 changed-file list and diff stat | current PR head | high | supports completion |
| Scene overlay publication is scene-owned | E3 + E5 | `scene/lifecycle/overlay_update.rs`, `scene/plugin.rs`, and `runtime_ui_producer_migration` tests | current PR head | high | supports completion |
| Debug overlay publication is debug-owned | E3 + E5 | `debug_metrics/mod.rs` and `runtime_ui_producer_migration` / debug overlay tests | current PR head | high | supports completion |
| RenderPlugin no longer owns scene/debug UI semantic collection | E3 + E5 | deleted `ui_submission.rs`, RenderPlugin schedule/export diff, `runtime_surface_guard`, and focused migration tests | current PR head | high | supports completion |
| Existing UiPlugin render publication did not regress | E5 | `cargo test -p engine ui_render_publication` | current PR head | high | supports completion |
| Later-phase forbidden scope is absent | E3 | changed-file inspection, diff stat, and guard/source search evidence | current PR head | high | supports completion |
| Phase 011 was authorized by accepted planning/design authority | E8 | active-work, roadmap, production-track, cutover plan, architecture docs, and Phase 011 spec | current `main` before implementation | high | supports merge and closeout |
| Validation and authority align | E9 | changed files, focused tests, local commands, and accepted Phase 011 contract agree | current PR head | high | supports completion |

Highest evidence class reached: `E9`.

## Merge readiness status

Merge readiness was satisfied before merge.

Merge readiness evidence:

```text
Scope: ready; changed files matched the Phase 011 contract and did not include Phase 012-014 implementation.
Authority: ready; active-work, roadmap, production-track, cutover plan, architecture docs, Phase 010 closeout, and Phase 011 spec authorized Phase 011 only.
Principles: ready; publication moved to direct owners, duplicate render-owned collection was removed, no speculative public API/product/reload/backend work entered the PR, and RenderPlugin stayed generic.
Maintainability: ready; scene generation, debug generation, producer publication, render preparation, and guard/integration tests remain separated by owner.
Validation: ready locally; no hosted checks were configured/reported.
Lifecycle: ready for merge with closeout required before Phase 012 implementation.
Review: ready; PR #104 had no review threads, no review comments, no requested reviewers, and no unresolved findings when inspected. Local PR review and merge-readiness routine found no blockers.
Merge mechanics: ready; PR #104 was mergeable and was merged with expected head SHA.
Post-merge truth: this closeout records completion truth before any Phase 012 implementation starts.
```

Branch cleanup:

```text
PR #104 merged with a merge commit.
Remote branch cleanup was not claimed by this closeout.
Local checkout fast-forwarded to main at 15e213a08dbf79f65e0851fe5be9f853f157b48b before this closeout branch was created.
```

## Boundary and non-goal evidence

Preserved non-goals:

```text
no engine/src/plugins/ui/** runtime implementation changes
no apps/ui_counter_runtime product packaging
no source reload or persistence implementation
no SDF or SpatialCanvas implementation
no render backend rewrite, graph execution rewrite, frame prepare rewrite, frame submit rewrite, or shader changes
no source/program/action semantic changes
no host mutation or action-dispatch behavior changes
no broad ui_render_data primitive/model rewrite
no foundation/meta
no domain/app_program
no generic plugin framework
no phase spec validator implementation
no tools/docs validator or script changes
```

Dependency boundary:

```text
No new crates or dependencies were added.
Scene and debug owners publish their own UI frame submissions through the generic registry.
RenderPlugin prepares generic producer/surface/frame payloads and no longer queries SceneResource or UiOverlayState for scene/debug UI semantic frames.
UiPlugin render publication remains independent and continues to use the generic producer/surface-frame seam from Phase 010.
```

## Complete gate status

Complete investigation gate status: complete for Phase 011 through accepted
runtime-platform authority, Phase 010 closeout evidence, activation source/path
inspection, PR #104 changed-file inspection, and focused validation.

Complete design gate status: complete for Phase 011 through the accepted full
cutover plan, architecture record, Phase 010 closeout evidence, active-work
contract, Phase 011 phase spec, principle compliance matrix, module
decomposition map, implementation evidence, and validation.

Phase completion drift check: complete in this closeout. Planning still showed
Phase 011 as active implementation after PR #104 merged; this closeout resolves
that drift and opens Phase 012 as active planning only.

## Known gaps

Intentional downstream gaps:

```text
runtime Counter app product remains unimplemented until Phase 012
source reload and persistence remain unimplemented until Phase 013
adoption lock remains unimplemented until Phase 014
```

These are not Phase 011 blockers.

## Drift found

Planning drift found after merge:

```text
active-work.md, roadmap.md, production-tracks.md, completed-work.md, and decision-register.md still needed Phase 011 post-merge truth after PR #104 merged.
```

Implementation drift found:

```text
No forbidden Phase 012-014 implementation scope was found in PR #104.
```

## Follow-up

Next safe action:

```text
Open PT-UI-RUNTIME-PLATFORM-012 as active planning only.
Do not start Phase 012 implementation until this closeout/planning truth is merged and a separate active-implementation authorization confirms the exact Phase 012 files, command proof, focused tests, validation commands, evidence requirements, and stop conditions from accepted Markdown authority.
```

## Evidence links

```text
PR #104: https://github.com/Crystonix/Runenwerk/pull/104
Phase 011 head commit: a6232278e41202cd331051f347d3db892988f38c
Phase 011 merge commit: 15e213a08dbf79f65e0851fe5be9f853f157b48b
Active work: ../../workspace/planning/active-work.md
Roadmap: ../../workspace/planning/roadmap.md
Production track: ../../workspace/planning/production-tracks.md
Completed work: ../../workspace/planning/completed-work.md
Decision register: ../../workspace/planning/decision-register.md
Phase 011 spec: ../../workspace/specs/pt-ui-runtime-platform-011.ron
Cutover plan: ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
Runtime architecture: ../../architecture/live-uiplugin-runtime-platform-architecture.md
```
