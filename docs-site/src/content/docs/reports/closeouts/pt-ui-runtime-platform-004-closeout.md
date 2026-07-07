---
title: PT-UI-RUNTIME-PLATFORM-004 Closeout
description: Closeout evidence for the App Mounting API phase.
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

# PT-UI-RUNTIME-PLATFORM-004 Closeout

ID: `PT-UI-RUNTIME-PLATFORM-004`

Title: `App Mounting API`

Completed on: 2026-07-07

Owner: `engine::plugins::ui`

## Contract promised

Phase 004 promised one bounded engine UI App Mounting API PR.

Promised scope:

```text
engine/src/plugins/ui/app_ext.rs
engine/src/plugins/ui/mount.rs
engine/src/plugins/ui/resources.rs mount-request storage only
engine/src/plugins/ui/diagnostics.rs mount diagnostics only
engine/src/prelude.rs public App extension export if accepted
focused engine tests proving app.mount_ui and app.ui().mount compile and record equivalent mount requests
```

Forbidden scope:

```text
UiScreen / IntoUi implementation
UiActionHandler implementation
mounted session runtime
host action dispatch
runtime trace
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
tools/docs validator or script changes
```

## Contract delivered

PR #82 merged Phase 004 into `main` at merge commit
`9fb86f0d426385be7e425ff943c7a9d5450e1edb`.

Delivered scope:

```text
AppUiExt trait with app.mount_ui(...)
UiAppMounting facade with app.ui().mount(...)
UiMountRequest, UiMountConfig, UiMountReport, UiMountRecord, UiMountSource, and UiMountFailureReason types
UiMountRequestsResource mount-request storage and report history
UiMountDiagnostic data attached to rejected mount diagnostics
AppUiExt prelude export
focused ui_mount integration tests
```

`engine/src/plugins/ui/mod.rs` changed only to declare and re-export the new
Phase 004 modules required by Rust module wiring.

## Files changed

PR #82 changed exactly:

```text
engine/src/plugins/ui/app_ext.rs
engine/src/plugins/ui/diagnostics.rs
engine/src/plugins/ui/mod.rs
engine/src/plugins/ui/mount.rs
engine/src/plugins/ui/resources.rs
engine/src/prelude.rs
engine/tests/ui_mount_api.rs
```

Diff stat:

```text
7 files changed, 471 insertions(+)
```

## Validation run

Command validation run before merge on PR head
`fdf245a273bf65652308ecbe2add43cde72f1871`:

```text
cargo test -p engine ui_mount
cargo test -p engine
python tools/docs/validate_docs.py
git diff --check
git diff --check main...HEAD
git status --short --branch
git diff --stat main...HEAD
gh pr checks 82 --watch=false
```

Results:

```text
cargo test -p engine ui_mount: passed; 3 ui_mount integration tests passed.
cargo test -p engine: passed; engine suite passed with the existing ignored GPU timing runtime-evidence test.
python tools/docs/validate_docs.py: passed.
git diff --check: passed.
git diff --check main...HEAD: passed.
git status --short --branch: clean on the Phase 004 branch before merge.
git diff --stat main...HEAD: 7 files changed, 471 insertions.
gh pr checks 82 --watch=false: no checks reported for the branch.
```

Validation unavailable:

```text
No GitHub Actions check evidence was available because GitHub reported no checks for the branch.
```

## Evidence classes used

| Claim | Evidence class | Source / command / artifact | Freshness | Confidence | Decision impact |
|---|---|---|---|---|---|
| Phase 004 PR merged into `main` | E6 | PR #82 metadata and merge commit `9fb86f0d426385be7e425ff943c7a9d5450e1edb` | current on 2026-07-07 | high | authorizes closeout |
| Delivered file scope matches Phase 004 contract | E3 | PR #82 file list and `git diff --name-status main...HEAD` before merge | current PR head | high | supports completion |
| Normal mounting path records a request | E3 + E5 | `engine/src/plugins/ui/app_ext.rs`; `engine/tests/ui_mount_api.rs`; `cargo test -p engine ui_mount` | current PR head | high | supports completion |
| Advanced mounting path records an equivalent request with reporting hook | E3 + E5 | `engine/src/plugins/ui/app_ext.rs`; `engine/tests/ui_mount_api.rs`; focused test output | current PR head | high | supports completion |
| Mount diagnostics include screen identity, mount source, and stable failure reason | E3 + E5 | `engine/src/plugins/ui/diagnostics.rs`; `engine/tests/ui_mount_api.rs`; focused test output | current PR head | high | supports completion |
| Later-phase forbidden scope is absent | E3 | changed-file inspection and forbidden-term scan over changed paths | current PR head | high | supports completion |
| Phase 004 was authorized by accepted planning/design authority | E8 | active-work, roadmap, production-tracks, cutover plan, architecture docs | current `main` before merge | high | supports merge and closeout |
| Validation and authority align | E9 | changed files, local commands, and accepted Phase 004 contract agree | current PR head | high | supports completion |

Highest evidence class reached: `E9`.

## Merge readiness status

Merge readiness was satisfied before merge.

Merge readiness evidence:

```text
Scope: ready; changed files matched the Phase 004 contract, with ui/mod.rs limited to module wiring.
Authority: ready; active-work, roadmap, production-track, cutover plan, and architecture docs authorized Phase 004 only.
Principles: ready; direct mount recording, no speculative runtime/session/render framework.
Maintainability: ready; module decomposition matched the planning map.
Validation: ready locally; no hosted checks were configured/reported.
Lifecycle: ready for merge with closeout required before Phase 005 implementation.
Review: ready; PR #82 had no comments, reviews, review requests, or unresolved findings when inspected.
Merge mechanics: ready; PR #82 was mergeable with merge state `CLEAN`.
Post-merge truth: this closeout records the required completion truth before any Phase 005 implementation starts.
```

Branch cleanup:

```text
PR #82 merged with squash merge.
Remote branch codex/pt-ui-runtime-platform-004-app-mounting-api was deleted by the merge command.
Local checkout fast-forwarded to main at 9fb86f0d426385be7e425ff943c7a9d5450e1edb.
```

## Boundary and non-goal evidence

Preserved non-goals:

```text
no UiScreen or IntoUi implementation
no UiActionHandler or TryUiActionHandler implementation
no mounted session runtime
no host action dispatch
no runtime trace
no render adapter
no SurfaceFrame generic producer boundary implementation
no scene/debug overlay producer migration
no source reload or persistence implementation
no apps/ui_counter_runtime implementation
no SDF, world-space UI, or SpatialCanvas implementation
no foundation/meta
no domain/app_program
no generic plugin framework
no phase spec validator implementation
no tools/docs validator or script changes
```

Dependency boundary:

```text
No Cargo.toml changes were needed.
No new cross-crate dependency was added.
Domain UI crates still do not depend on engine.
RenderPlugin ownership was not changed.
```

## Complete gate status

Complete investigation gate status: complete for Phase 004 through the accepted
runtime-platform investigation/design authority and current PR file inspection.

Complete design gate status: complete for Phase 004 through the accepted full
cutover plan, active-work contract, roadmap entry, production-track entry,
principle compliance matrix, and module decomposition map.

Phase completion drift check: complete in this closeout. Planning still showed
Phase 004 as active implementation after PR #82 merged; this closeout resolves
that drift and opens Phase 005 as active planning only.

## Known gaps

Intentional downstream gaps:

```text
typed screen/source/action contracts remain unimplemented until Phase 005
mounted sessions remain unimplemented until Phase 006
host action dispatch and runtime trace remain unimplemented until Phase 007
runtime evaluation/invalidation remains unimplemented until Phase 008
producer-generic SurfaceFrame boundary remains unimplemented until Phase 009
UiPlugin render publication remains unimplemented until Phase 010
scene/debug overlay migration remains unimplemented until Phase 011
runtime Counter app product remains unimplemented until Phase 012
source reload and persistence remain unimplemented until Phase 013
adoption lock remains unimplemented until Phase 014
```

These are not Phase 004 blockers.

## Drift found

Planning drift found after merge:

```text
active-work.md, roadmap.md, production-tracks.md, completed-work.md, and decision-register.md still needed Phase 004 post-merge truth after PR #82 merged.
```

Implementation drift found:

```text
No forbidden Phase 005-014 implementation scope was found in PR #82.
```

## Follow-up

Next safe action:

```text
Open PT-UI-RUNTIME-PLATFORM-005 as active planning only.
Do not start Phase 005 implementation until this closeout/planning truth is merged and a separate active-implementation authorization confirms the exact Phase 005 contract from accepted Markdown authority.
```

## Evidence links

```text
PR #82: https://github.com/Crystonix/Runenwerk/pull/82
Phase 004 head commit: fdf245a273bf65652308ecbe2add43cde72f1871
Phase 004 merge commit: 9fb86f0d426385be7e425ff943c7a9d5450e1edb
Active work: ../../workspace/planning/active-work.md
Roadmap: ../../workspace/planning/roadmap.md
Production track: ../../workspace/planning/production-tracks.md
Completed work: ../../workspace/planning/completed-work.md
Decision register: ../../workspace/planning/decision-register.md
Cutover plan: ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
Runtime architecture: ../../architecture/live-uiplugin-runtime-platform-architecture.md
```
