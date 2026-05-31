---
title: WR-145 6E World Space Host Boundary Proof Implementation Plan
description: Product-code implementation planning contract for PM-UI-PROGRAM-011 under WR-145.
status: active
owner: ui
layer: domain/ui
canonical: false
last_reviewed: 2026-05-25
related:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/track-execution-manifests/pt-ui-program.yaml
related_designs:
  - ../../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../../design/active/ui-program-architecture.md
  - ../../../design/active/ui-program-proof-slice-plan.md
---

# WR-145 6E World Space Host Boundary Proof Implementation Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-011` - 6E World Space Host Boundary Proof
- Stage: Stage 6E
- Roadmap item: `WR-145` - 6E World Space Host Boundary Proof
- Proof slice id: `6E`
- Proof slice title: 6E World Space Host Boundary Proof
- Target control/surface: World-space anchored prompt
- Implementation proof kind: `6e-world-space-host-boundary-proof`
- Authority: implementation planning only.
- This plan is the accepted production plan required before Manifest Runner V5 may run with `--allow product_code --allow product_implementation`.
- This plan does not execute product/runtime code and does not close the milestone.
- This plan does not authorize crate creation, placeholder future folders, MaterialProgram implementation, broad retained UI rewrite, or shared `foundation/meta` extraction.

## Production Planning Output

- Production track: `PT-UI-PROGRAM` - UI Program Platform Proof
- Production milestone: `PM-UI-PROGRAM-011` - 6E World Space Host Boundary Proof
- Production milestone state after plan acceptance: `active`
- Roadmap item: `WR-145` - 6E World Space Host Boundary Proof
- Roadmap planning state after plan acceptance: `current_candidate`
- Roadmap blocker after plan acceptance: `B2`
- Contract target: `docs-site/src/content/docs/reports/implementation-plans/wr-145-6e-world-space-host-boundary-proof/plan.md`
- Post-plan transition: Manifest Runner V2 records the milestone as `active` and the WR as `current_candidate`/`B2`.
- Next action after this plan is accepted: stop before product/runtime code unless rerun with explicit `--allow product_code --allow product_implementation` and all V5 gates pass.

## Source Documents

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`

## Exact Files/Modules Expected To Change

- `domain/ui/ui_surface/src/mount.rs`
- `domain/ui/ui_surface/src/presentation.rs`
- `domain/ui/ui_surface/src/observation.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`
- `domain/ui/ui_definition/src/source.rs`

## Implementation Writer Strategy

- Strategy: `patch_writer`
- New file policy: `existing_files_only`
- Allowed writer files:
- `domain/ui/ui_surface/src/mount.rs`
- `domain/ui/ui_surface/src/presentation.rs`
- `domain/ui/ui_surface/src/observation.rs`
- `domain/ui/ui_definition/src/source.rs`
- Required outputs:
- anchored world-space prompt host input model
- projection, lifetime, visibility, and data-feed proof
- route/event/host-command boundary without ECS-owned UI semantics
- retained/current UI compatibility through existing files only
- Forbidden writer files:
- `foundation/meta`
- `domain/material`
- Forbidden patterns:
- `UiSemanticEvent`
- `MaterialProgram`
- `foundation/meta`

## Expected Methods/Functions

- World-space anchored prompt projection, lifetime, visibility, and data-feed host-boundary helpers only.

## Required Implementation Scope

- World-space prompt host-boundary implementation plan.
- Anchor, projection, lifetime, visibility, and data-feed ownership scope.
- Exact files/modules allowed, runtime/test evidence, rollback, compatibility, and stop conditions.

## Required Decisions

- ECS may provide identity, anchor, lifetime, visibility input, and data feed only.
- ECS must not own UI semantics.

## Forbidden Files/Modules

- ECS-owned UI semantics
- broad world UI framework
- new crates
- shared foundation/meta extraction

## Tests To Add/Change

- Focused world-space prompt host-boundary tests in scoped ui_surface, ui_runtime, or ui_definition modules only.
- Do not create new test folders, new crates, placeholder future modules, or broad fixture frameworks.

## Validation Commands

- `cargo test -p ui_surface mount`
- `cargo test -p ui_surface presentation`
- `cargo test -p ui_surface observation`
- `cargo test -p ui_runtime build_ui_frame`
- `task docs:validate`
- `task planning:validate`

## Closeout Requirements

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-011-6e-world-space-host-boundary-proof/closeout.md`
- Closeout evidence must include:
- World-space prompt proves anchor, projection, lifetime, visibility, and host-fed data without ECS-owned UI semantics.
- Closeout must explicitly state that no renderer-owned UI truth, ECS-owned UI semantics, MaterialProgram implementation, crates, placeholder folders, or `foundation/meta` extraction occurred.

## Compatibility / Rollback Plan

- Retained/current UI stays compatible; rollback is limited to the exact world-space prompt proof files.

## Acceptance Checklist

- Plan names every exact world-space prompt proof write scope.
- Plan preserves no broad world UI framework and no foundation/meta extraction.

## Stop Conditions

- stop if product_code permission is not explicitly granted
- stop if the active WR or accepted implementation plan is missing
- stop if validation fails
- stop after one implementation WR unless the runner recomputes and all closeout gates pass
- Stop before product/runtime code unless the command is rerun with `--allow product_code --allow product_implementation` and all V5 gates pass.
- Stop before the next milestone until this milestone has runtime/test closeout evidence.

## Next Command If Product Code Is Permitted

`task production:run-track -- --track PT-UI-PROGRAM --allow product_code --allow product_implementation --max-actions 1`
