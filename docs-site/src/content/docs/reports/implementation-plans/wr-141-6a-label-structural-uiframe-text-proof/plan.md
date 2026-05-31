---
title: WR-141 6A Label Structural UiFrame Text Proof Implementation Plan
description: Product-code implementation planning contract for PM-UI-PROGRAM-007 under WR-141.
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

# WR-141 6A Label Structural UiFrame Text Proof Implementation Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-007` - 6A Label Structural UiFrame Text Proof
- Stage: Stage 6A
- Roadmap item: `WR-141` - 6A Label Structural UiFrame Text Proof
- Authority: implementation planning only.
- This plan is the accepted production plan required before Manifest Runner V4 may run with `--allow product_code`.
- This plan does not execute product/runtime code and does not close the milestone.
- This plan does not authorize crate creation, placeholder future folders, MaterialProgram implementation, broad retained UI rewrite, or shared `foundation/meta` extraction.

## Production Planning Output

- Production track: `PT-UI-PROGRAM` - UI Program Platform Proof
- Production milestone: `PM-UI-PROGRAM-007` - 6A Label Structural UiFrame Text Proof
- Production milestone state after plan acceptance: `active`
- Roadmap item: `WR-141` - 6A Label Structural UiFrame Text Proof
- Roadmap planning state after plan acceptance: `current_candidate`
- Roadmap blocker after plan acceptance: `B2`
- Contract target: `docs-site/src/content/docs/reports/implementation-plans/wr-141-6a-label-structural-uiframe-text-proof/plan.md`
- Post-plan transition: Manifest Runner V2 records the milestone as `active` and the WR as `current_candidate`/`B2`.
- Next action after this plan is accepted: stop before product/runtime code unless rerun with explicit `--allow product_code` and all V4 gates pass.

## Source Documents

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`
- `docs-site/src/content/docs/workspace/track-execution-manifest.md`
- `docs-site/src/content/docs/workspace/design-track-roadmap-governance.md`

## Exact Files/Modules Expected To Change

- `domain/ui/ui_widgets/src/label.rs`
- `domain/ui/ui_text/src/layout.rs`
- `domain/ui/ui_render_data/src/lib.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`
- `domain/ui/ui_runtime/src/output/mod.rs`
- `domain/ui/ui_definition/src/source.rs`

## Expected Methods/Functions

- Label text output construction in the scoped `ui_widgets` label module.
- Font/style intent conversion in the scoped `ui_text` layout module.
- Text layout request and metric handoff in the scoped `ui_text` and `ui_runtime` modules.
- Structural `UiFrame` output assembly in the scoped `ui_runtime` output module.
- Source/source-map attachment path in the scoped `ui_definition` source module.
- Diagnostics and runtime artifact evidence only where they fit inside the exact scoped modules.

## Required Implementation Scope

- Label text output
- font/style intent
- text layout request
- structural UiFrame assertion
- text/render boundary
- no renderer-owned UI truth
- retained/current UI compatibility
- source-map evidence
- diagnostics evidence
- runtime artifact evidence

## Required Decisions

- 6A is limited to Label plus structural UiFrame text proof only.
- The 6A plan may touch only existing UI modules listed in write_scope.
- Focused tests must be added inside the exact write scope; no new test folders or crates are authorized.
- Retained/current UI compatibility remains mandatory during 6A.
- Product code remains blocked until V4 is explicitly rerun with product_code allowed.

## Forbidden Files/Modules

- no new crates
- no placeholder future folders
- no broad retained UI rewrite
- no Button implementation
- no InspectorField implementation
- no ColorPicker implementation
- no 6B through 6F
- no MaterialProgram implementation
- no foundation/meta extraction
- no renderer-owned UI semantics
- no ECS-owned UI semantics
- broad retained UI replacement
- renderer-owned UI truth

## Tests To Add/Change

- Add focused Label text output assertions inside the scoped `ui_widgets` or `ui_runtime` files only.
- Add focused structural `UiFrame` assertions inside the scoped `ui_runtime` output files only.
- Add focused text layout request, source-map, diagnostic, and runtime artifact assertions only inside the exact write scope.
- Do not create new test folders, new crates, placeholder future modules, or broad fixture frameworks.

## Validation Commands

- `cargo test -p ui_widgets label`
- `cargo test -p ui_text layout`
- `cargo test -p ui_runtime build_ui_frame`
- `task docs:validate`
- `task planning:validate`

## Closeout Requirements

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-007-6a-label-structural-uiframe-text-proof/closeout.md`
- Closeout evidence must include runtime/test evidence for Label text output, font/style intent, text layout request, structural `UiFrame` assertion, source maps, diagnostics, runtime artifact evidence, and retained/current UI compatibility.
- Closeout must explicitly state that no renderer-owned UI truth, ECS-owned UI semantics, Button, InspectorField, ColorPicker, 6B through 6F, MaterialProgram, crates, placeholder folders, or `foundation/meta` extraction occurred.

## Compatibility / Rollback Plan

- Retained/current UI remains compatible unless the 6A implementation WR names and proves a bounded replacement surface.
- Any runtime change must be reversible by reverting only the exact scoped files/modules listed in this plan.
- Existing retained UI behavior must remain the fallback until 6A closeout proves the structural `UiFrame` Label path.

## Acceptance Checklist

- The implementation plan names every exact write_scope path.
- The implementation plan includes exact forbidden scope, validation commands, closeout evidence, compatibility/rollback plan, and stop conditions.
- The implementation plan forbids Button, InspectorField, ColorPicker, 6B through 6F, MaterialProgram, crate creation, placeholder folders, broad retained UI rewrite, renderer-owned UI semantics, ECS-owned UI semantics, and foundation/meta extraction.
- No product/runtime code is changed by agent_design.

## Stop Conditions

- stop until PM-UI-PROGRAM-006 is completed
- stop until Track Expansion creates or links the owning WR
- stop until task production:plan creates the bounded 6A contract
- stop if text/render ownership, structural UiFrame evidence, or retained compatibility cannot be proven
- Stop before product/runtime code unless the command is rerun with `--allow product_code` and all V4 gates pass.
- Stop before PM-008 / 6B regardless of validation outcome.

## Next Command If Product Code Is Permitted

`task production:run-track -- --track PT-UI-PROGRAM --allow product_code --max-actions 1`
