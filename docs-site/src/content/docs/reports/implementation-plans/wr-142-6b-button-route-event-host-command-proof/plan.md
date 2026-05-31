---
title: WR-142 6B Button Route Event Host Command Proof Implementation Plan
description: Product-code implementation planning contract for PM-UI-PROGRAM-008 under WR-142.
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

# WR-142 6B Button Route Event Host Command Proof Implementation Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-008` - 6B Button Route Event Host Command Proof
- Stage: Stage 6B
- Roadmap item: `WR-142` - 6B Button Route Event Host Command Proof
- Proof slice id: `6B`
- Proof slice title: 6B Button Route Event Host Command Proof
- Target control/surface: Button
- Implementation proof kind: `6b-button-route-event-host-command-proof`
- Authority: implementation planning only.
- This plan is the accepted production plan required before Manifest Runner V4 may run with `--allow product_code`.
- This plan does not execute product/runtime code and does not close the milestone.
- This plan does not authorize crate creation, placeholder future folders, MaterialProgram implementation, broad retained UI rewrite, or shared `foundation/meta` extraction.

## Production Planning Output

- Production track: `PT-UI-PROGRAM` - UI Program Platform Proof
- Production milestone: `PM-UI-PROGRAM-008` - 6B Button Route Event Host Command Proof
- Production milestone state after plan acceptance: `active`
- Roadmap item: `WR-142` - 6B Button Route Event Host Command Proof
- Roadmap planning state after plan acceptance: `current_candidate`
- Roadmap blocker after plan acceptance: `B2`
- Contract target: `docs-site/src/content/docs/reports/implementation-plans/wr-142-6b-button-route-event-host-command-proof/plan.md`
- Post-plan transition: Manifest Runner V2 records the milestone as `active` and the WR as `current_candidate`/`B2`.
- Next action after this plan is accepted: stop before product/runtime code unless rerun with explicit `--allow product_code` and all V4 gates pass.

## Source Documents

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`

## Exact Files/Modules Expected To Change

- `domain/ui/ui_widgets/src/button.rs`
- `domain/ui/ui_definition/src/slot.rs`
- `domain/ui/ui_definition/src/interaction.rs`
- `domain/ui/ui_input/src/routing.rs`
- `domain/ui/ui_runtime/src/input/pointer.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`

## Expected Methods/Functions

- Button route emission and interaction-state functions only.
- UiEventPacket / RouteId / HostCommand mapping helpers named by the 6B plan.

## Required Implementation Scope

- Button route/event/host-command implementation plan.
- UiEventPacket, RouteId, RouteSchemaVersion, RouteCapability, HostRouteMapVersion, HostCommand, and optional DomainCommand scope.
- Exact files/modules allowed and forbidden implementation scope.
- Runtime/test evidence, diagnostics, source-map evidence, rollback, and stop conditions.

## Required Decisions

- Routes are stable IDs, not free-form route strings.
- Button proof does not add a giant UiSemanticEvent enum.
- Hosts map UI events to commands through explicit route maps.

## Forbidden Files/Modules

- giant UiSemanticEvent enum
- hidden route-string behavior
- broad host rewrite
- new crates
- shared foundation/meta extraction

## Tests To Add/Change

- Focused Button route/event tests in the scoped UI widget, definition, input, or runtime modules only.
- Do not create new test folders, new crates, placeholder future modules, or broad fixture frameworks.

## Validation Commands

- `cargo test -p ui_widgets button`
- `cargo test -p ui_definition interaction`
- `cargo test -p ui_input routing`
- `cargo test -p ui_runtime build_ui_frame`
- `task docs:validate`
- `task planning:validate`

## Closeout Requirements

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-008-6b-button-route-event-host-command-proof/closeout.md`
- Closeout evidence must include:
- Button emits route-based UiEventPacket evidence with RouteId, RouteSchemaVersion, RouteCapability, HostRouteMapVersion, HostCommand, and optional DomainCommand.
- No central UiSemanticEvent enum variants or hidden route-string behavior.
- Closeout must explicitly state that no renderer-owned UI truth, ECS-owned UI semantics, MaterialProgram implementation, crates, placeholder folders, or `foundation/meta` extraction occurred.

## Compatibility / Rollback Plan

- Retained/current UI stays compatible; rollback is limited to the exact Button proof files.

## Acceptance Checklist

- Plan names every exact Button proof write scope.
- Plan preserves no crate creation, no MaterialProgram implementation, and no foundation/meta extraction.

## Stop Conditions

- stop if product_code permission is not explicitly granted
- stop if the active WR or accepted implementation plan is missing
- stop if validation fails
- stop after one implementation WR unless the runner recomputes and all closeout gates pass
- Stop before product/runtime code unless the command is rerun with `--allow product_code` and all V4 gates pass.
- Stop before the next milestone until this milestone has runtime/test closeout evidence.

## Next Command If Product Code Is Permitted

`task production:run-track -- --track PT-UI-PROGRAM --allow product_code --max-actions 1`
