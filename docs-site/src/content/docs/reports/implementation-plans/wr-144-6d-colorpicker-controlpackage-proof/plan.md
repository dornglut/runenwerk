---
title: WR-144 6D ColorPicker ControlPackage Proof Implementation Plan
description: Product-code implementation planning contract for PM-UI-PROGRAM-010 under WR-144.
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

# WR-144 6D ColorPicker ControlPackage Proof Implementation Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-010` - 6D ColorPicker ControlPackage Proof
- Stage: Stage 6D
- Roadmap item: `WR-144` - 6D ColorPicker ControlPackage Proof
- Proof slice id: `6D`
- Proof slice title: 6D ColorPicker ControlPackage Proof
- Target control/surface: ColorPicker
- Implementation proof kind: `6d-colorpicker-controlpackage-proof`
- Authority: implementation planning only.
- This plan is the accepted production plan required before Manifest Runner V5 may run with `--allow product_code --allow product_implementation`.
- This plan does not execute product/runtime code and does not close the milestone.
- This plan does not authorize crate creation, placeholder future folders, MaterialProgram implementation, broad retained UI rewrite, or shared `foundation/meta` extraction.

## Production Planning Output

- Production track: `PT-UI-PROGRAM` - UI Program Platform Proof
- Production milestone: `PM-UI-PROGRAM-010` - 6D ColorPicker ControlPackage Proof
- Production milestone state after plan acceptance: `active`
- Roadmap item: `WR-144` - 6D ColorPicker ControlPackage Proof
- Roadmap planning state after plan acceptance: `current_candidate`
- Roadmap blocker after plan acceptance: `B2`
- Contract target: `docs-site/src/content/docs/reports/implementation-plans/wr-144-6d-colorpicker-controlpackage-proof/plan.md`
- Post-plan transition: Manifest Runner V2 records the milestone as `active` and the WR as `current_candidate`/`B2`.
- Next action after this plan is accepted: stop before product/runtime code unless rerun with explicit `--allow product_code --allow product_implementation` and all V5 gates pass.

## Source Documents

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`

## Exact Files/Modules Expected To Change

- `new: domain/ui/ui_widgets/src/color_picker.rs`
- `domain/ui/ui_widgets/src/lib.rs`
- `domain/ui/ui_theme/src/color.rs`
- `domain/ui/ui_definition/src/component_recipe/mod.rs`
- `domain/ui/ui_definition/src/preview_fixture/mod.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`

## Expected Methods/Functions

- Wheel-plus-triangle ColorPicker ControlPackage proof functions only.

## Required Implementation Scope

- ColorPicker ControlPackage implementation plan.
- Wheel-plus-triangle first proof scope and RGB cube deferral.
- Package/control/schema/kernel/fixture/diagnostic IDs and migration scope.
- Exact files/modules allowed, runtime/test evidence, rollback, compatibility, and stop conditions.

## Required Decisions

- ColorPicker is the first rich ControlPackage proof.
- RGB cube projection is deferred and must not enter the 6D proof.

## Forbidden Files/Modules

- RGB cube projection
- broad package framework
- new crates
- shared foundation/meta extraction

## Tests To Add/Change

- Focused ColorPicker ControlPackage tests in scoped widget, theme, definition, or runtime modules only.
- Do not create new test folders, new crates, placeholder future modules, or broad fixture frameworks.

## Validation Commands

- `cargo test -p ui_widgets color_picker`
- `cargo test -p ui_theme color`
- `cargo test -p ui_definition preview_fixture`
- `cargo test -p ui_runtime build_ui_frame`
- `task docs:validate`
- `task planning:validate`

## Closeout Requirements

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-010-6d-colorpicker-controlpackage-proof/closeout.md`
- Closeout evidence must include:
- ColorPicker proves package ID, control kind ID, property schema, state schema, event payload schema, layout kernel ID, interaction kernel ID, visual kernel ID, fixtures, diagnostics, and migration story.
- RGB cube projection remains deferred.
- Closeout must explicitly state that no renderer-owned UI truth, ECS-owned UI semantics, MaterialProgram implementation, crates, placeholder folders, or `foundation/meta` extraction occurred.

## Compatibility / Rollback Plan

- Retained/current UI stays compatible; rollback is limited to the exact ColorPicker proof files.

## Acceptance Checklist

- Plan names every exact ColorPicker proof write scope.
- Plan preserves no broad package framework and no foundation/meta extraction.

## Stop Conditions

- stop if product_code permission is not explicitly granted
- stop if the active WR or accepted implementation plan is missing
- stop if validation fails
- stop after one implementation WR unless the runner recomputes and all closeout gates pass
- Stop before product/runtime code unless the command is rerun with `--allow product_code --allow product_implementation` and all V5 gates pass.
- Stop before the next milestone until this milestone has runtime/test closeout evidence.

## Next Command If Product Code Is Permitted

`task production:run-track -- --track PT-UI-PROGRAM --allow product_code --allow product_implementation --max-actions 1`
