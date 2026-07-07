---
title: PT-UI-RUNTIME-PLATFORM-010 Closeout
description: Closeout evidence for the UiPlugin Render Publication phase.
status: completed
owner: ui
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

# PT-UI-RUNTIME-PLATFORM-010 Closeout

ID: `PT-UI-RUNTIME-PLATFORM-010`

Title: `UiPlugin Render Publication`

Completed on: 2026-07-07

Owner: `engine::plugins::ui` render publication path, with one scheduling-only
`RenderPlugin` ordering edge.

## Contract promised

Phase 010 promised one bounded UiPlugin Render Publication PR.

Promised handoff contract:

```text
UiPlugin publishes frame submission with producer id and surface identity through the generic seam.
RenderPlugin consumes prepared payload without querying UiScreen, IntoUi, actions, host mutation, or route policy.
render contribution is deterministic for the same runtime frame.
missing UiPlugin frame reports a diagnostic instead of silent success.
frame publication trace records producer, surface, frame revision, dirty cause, and publication result.
trace adds UiFramePublished and UiFramePresented event families as Phase 010 UI-semantic facts.
```

Promised scope:

```text
engine/src/plugins/ui/render_publish.rs
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/schedule.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/trace.rs
engine/src/plugins/ui/diagnostics.rs only for publication diagnostics
engine/src/plugins/ui/resources.rs only if publication needs a UiPlugin-owned publication resource or read-only helper access to latest evaluation reports
engine/src/plugins/render/features/ui/submission.rs only if a small generic-seam API extension is required; no rename or semantic rewrite
engine/src/plugins/render/features/ui/resource.rs only if tests need prepared payload access that already belongs to the generic seam; no renderer behavior rewrite
engine/src/plugins/render/plugin.rs only to order prepare_ui_feature_resource_system after UiRuntimeSet::RenderPublication; no resource ownership, collection behavior, frame prepare, frame submit, backend, graph, shader, or renderer behavior rewrite
engine/tests/ui_render_publication.rs
engine/tests/ui_runtime_evaluation.rs only if existing helpers or assertions must align with publication facts
engine/tests/render_flow_v2.rs only if the prepared-frame integration proof needs an assertion against the new UiPlugin producer contribution
```

Forbidden scope:

```text
scene/debug overlay migration or retirement implementation
apps/ui_counter_runtime product packaging
source reload/persistence implementation
SDF or SpatialCanvas implementation
render backend rewrite, graph execution rewrite, or shader changes
source/program/action semantic changes outside publication facts
host mutation or action-dispatch behavior changes
broad ui_render_data primitive/model rewrites
foundation/meta
domain/app_program
generic plugin framework
phase spec validator implementation
any tools/docs validator or script changes
```

## Contract delivered

PR #101 merged Phase 010 into `main` at merge commit
`8d6c13146deab870dca5533204067249aa2c1b90`.

PR head before merge:
`79420eb642cecd43b495208f4749b2af0818bae5`.

Delivered scope:

```text
UiPlugin now owns a stable UiRuntime frame producer id and publication target resource.
UiPlugin publishes the latest UiRuntimeEvaluationReport frame payload through SurfaceFrameSubmissionRegistryResource keyed by RenderFrameProducerId and RenderSurfaceId.
UiPlugin publication records UiRuntimeFramePublicationReport facts for published and missing-evaluation outcomes.
Missing runtime evaluation records a FramePublicationRejected diagnostic and removes the previous UiPlugin surface-scoped submission so stale frames are not presented as success.
UiRuntimeTraceResource records UiFramePublished and UiFramePresented events with producer id, render surface id, frame revision, dirty cause, and publication status.
UiPlugin installs publication resources and schedules publication in RenderPrepare.
RenderPlugin orders prepare_ui_feature_resource_system after UiRuntimeSet::RenderPublication and before RenderRuntimeSet::FramePrepare.
PreparedUiFrameResource receives the UiPlugin producer contribution when RenderPlugin and UiPlugin run together.
```

## Files changed

PR #101 changed exactly:

```text
engine/src/plugins/render/features/ui/submission.rs
engine/src/plugins/render/plugin.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/plugin.rs
engine/src/plugins/ui/render_publish.rs
engine/src/plugins/ui/report.rs
engine/src/plugins/ui/schedule.rs
engine/src/plugins/ui/trace.rs
engine/tests/ui_plugin_foundation.rs
engine/tests/ui_render_publication.rs
```

Diff stat:

```text
11 files changed, 846 insertions(+), 3 deletions(-)
```

## Validation run

Command validation run before merge on PR head
`79420eb642cecd43b495208f4749b2af0818bae5`:

```text
cargo fmt --check
cargo test -p engine ui_render_publication
cargo test -p engine ui_runtime_evaluation
cargo test -p engine surface_frame_submission
cargo test -p engine render_output_proof
cargo test -p engine --test render_flow_v2
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
gh pr checks 101
```

Results:

```text
cargo fmt --check: passed.
cargo test -p engine ui_render_publication: passed; 4 focused tests passed.
cargo test -p engine ui_runtime_evaluation: passed; 2 focused tests passed.
cargo test -p engine surface_frame_submission: passed; 4 focused seam tests passed. The command also matched one publication test by filter name.
cargo test -p engine render_output_proof: passed; 1 focused test passed.
cargo test -p engine --test render_flow_v2: passed; 15 integration tests passed.
cargo test -p engine: passed on rerun; engine lib reported 262 passed and 1 ignored, with integration tests and doctests clean.
python tools/docs/validate_docs.py: passed.
git diff --check: passed.
git diff --check main...HEAD: passed.
git status --short --branch: clean on the Phase 010 branch after force-with-lease update.
git diff --stat main...HEAD: 11 files changed, 846 insertions, 3 deletions.
gh pr checks 101: no checks reported for the branch.
```

Validation note:

```text
The first post-amend broad cargo test -p engine run hit an unrelated transient timeout in runtime_job_work_stealing_executor_matches_serial_results.
The exact failed test passed on rerun.
The required broad cargo test -p engine command then passed on rerun before merge.
```

Validation unavailable:

```text
No GitHub Actions check evidence was available because GitHub reported no checks for the branch.
```

## Evidence classes used

| Claim | Evidence class | Source / command / artifact | Freshness | Confidence | Decision impact |
|---|---|---|---|---|---|
| Phase 010 PR merged into `main` | E6 | PR #101 metadata and merge commit `8d6c13146deab870dca5533204067249aa2c1b90` | current on 2026-07-07 | high | authorizes closeout |
| Delivered file scope matches Phase 010 contract | E3 | PR #101 changed-file list and `git diff --stat main...HEAD` before merge | current PR head | high | supports completion |
| UiPlugin publishes through the generic surface-frame seam | E3 + E5 | `render_publish.rs`, `SurfaceFrameSubmissionRegistryResource`, and `cargo test -p engine ui_render_publication` | current PR head | high | supports completion |
| RenderPlugin consumes the UiPlugin producer without owning UI semantics | E3 + E5 | `engine/src/plugins/render/plugin.rs` scheduling edge and focused publication integration test | current PR head | high | supports completion |
| Missing evaluation does not silently succeed or keep stale surface submissions | E3 + E5 | publication diagnostic/report code, `remove_for_surface`, focused publication and submission tests | current PR head | high | supports completion |
| Publication and presentation trace facts are recorded | E3 + E5 | `trace.rs`, `report.rs`, and focused publication trace assertions | current PR head | high | supports completion |
| Later-phase forbidden scope is absent | E3 | changed-file inspection and forbidden-scope proof | current PR head | high | supports completion |
| Phase 010 was authorized by accepted planning/design authority | E8 | active-work, roadmap, production-tracks, cutover plan, architecture docs, and activation correction decision | current `main` before merge | high | supports merge and closeout |
| Validation and authority align | E9 | changed files, focused tests, local commands, and accepted Phase 010 contract agree | current PR head | high | supports completion |

Highest evidence class reached: `E9`.

## Merge readiness status

Merge readiness was satisfied before merge.

Merge readiness evidence:

```text
Scope: ready; changed files matched the Phase 010 contract plus the accepted small generic seam helper.
Authority: ready; active-work, roadmap, production-track, cutover plan, architecture docs, and activation correction authorized Phase 010 only.
Principles: ready; the patch reuses the producer-generic seam, keeps UI publication owned by UiPlugin, and avoids speculative downstream product/reload/backend work.
Maintainability: ready; publication target/system, reports, diagnostics, trace facts, generic seam helper, and tests remain separated by responsibility.
Validation: ready locally after rerun; no hosted checks were configured/reported.
Lifecycle: ready for merge with closeout required before Phase 011 implementation.
Review: ready; PR #101 had no review threads, no review comments, no requested reviewers, and no unresolved findings when inspected.
Merge mechanics: ready; PR #101 was mergeable with merge state CLEAN and was merged with expected head SHA.
Post-merge truth: this closeout records completion truth before any Phase 011 implementation starts.
```

Branch cleanup:

```text
PR #101 merged with squash merge.
Remote branch cleanup was not claimed by this closeout.
Local checkout fast-forwarded to main at 8d6c13146deab870dca5533204067249aa2c1b90 before this closeout branch was created.
```

## Boundary and non-goal evidence

Preserved non-goals:

```text
no scene/debug overlay migration or retirement implementation
no apps/ui_counter_runtime product packaging
no source reload or persistence implementation
no SDF or SpatialCanvas implementation
no render backend rewrite, graph execution rewrite, frame submit rewrite, or shader changes
no source/program/action semantic changes outside publication facts
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
UiPlugin publishes evaluated runtime frame payload facts as a producer.
RenderPlugin consumes producer/surface/frame packets and only knows the UiRuntimeSet label for scheduling order; it does not query UiScreen, IntoUi, actions, host mutation, or route policy.
Scene/debug overlay producers remain present as downstream Phase 011 migration inputs.
```

## Complete gate status

Complete investigation gate status: complete for Phase 010 through accepted
runtime-platform investigation/design authority, Phase 009 closeout evidence,
activation-time source inspection, PR #101 changed-file inspection, and focused
publication/render validation.

Complete design gate status: complete for Phase 010 through the accepted full
cutover plan, architecture record, Phase 009 closeout evidence, active-work
contract, roadmap entry, production-track entry, activation correction
decision, principle compliance matrix, and module decomposition map.

Phase completion drift check: complete in this closeout. Planning still showed
Phase 010 as active implementation after PR #101 merged; this closeout resolves
that drift and opens Phase 011 as active planning only.

## Known gaps

Intentional downstream gaps:

```text
scene/debug overlay producer migration and retirement remains unimplemented until Phase 011
runtime Counter app product remains unimplemented until Phase 012
source reload and persistence remain unimplemented until Phase 013
adoption lock remains unimplemented until Phase 014
```

These are not Phase 010 blockers.

## Drift found

Planning drift found after merge:

```text
active-work.md, roadmap.md, production-tracks.md, completed-work.md, and decision-register.md still needed Phase 010 post-merge truth after PR #101 merged.
```

Implementation drift found:

```text
No forbidden Phase 011-014 implementation scope was found in PR #101.
```

## Follow-up

Next safe action:

```text
Open PT-UI-RUNTIME-PLATFORM-011 as active planning only.
Do not start Phase 011 implementation until this closeout/planning truth is merged and a separate active-implementation authorization confirms the exact Phase 011 files, focused tests, validation commands, evidence requirements, and stop conditions from accepted Markdown authority.
```

## Evidence links

```text
PR #101: https://github.com/Crystonix/Runenwerk/pull/101
Phase 010 head commit: 79420eb642cecd43b495208f4749b2af0818bae5
Phase 010 merge commit: 8d6c13146deab870dca5533204067249aa2c1b90
Active work: ../../workspace/planning/active-work.md
Roadmap: ../../workspace/planning/roadmap.md
Production track: ../../workspace/planning/production-tracks.md
Completed work: ../../workspace/planning/completed-work.md
Decision register: ../../workspace/planning/decision-register.md
Cutover plan: ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
Runtime architecture: ../../architecture/live-uiplugin-runtime-platform-architecture.md
```
