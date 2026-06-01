---
title: WR-146 6F Headless Fixture Diagnostics Source Map Proof Implementation Plan
description: Product-code implementation planning contract for PM-UI-PROGRAM-012 under WR-146.
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

# WR-146 6F Headless Fixture Diagnostics Source Map Proof Implementation Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-012` - 6F Headless Fixture Diagnostics Source Map Proof
- Stage: Stage 6F
- Roadmap item: `WR-146` - 6F Headless Fixture Diagnostics Source Map Proof
- Proof slice id: `6F`
- Proof slice title: 6F Headless Fixture Diagnostics Source Map Proof
- Target control/surface: Headless fixture evaluation
- Implementation proof kind: `6f-headless-fixture-diagnostics-source-map-proof`
- Authority: implementation planning only.
- This plan is the accepted production plan required before Manifest Runner V5 may run with `--allow product_code --allow product_implementation`.
- This plan does not execute product/runtime code and does not close the milestone.
- This plan does not authorize crate creation, placeholder future folders, MaterialProgram implementation, broad retained UI rewrite, or shared `foundation/meta` extraction.

## Production Planning Output

- Production track: `PT-UI-PROGRAM` - UI Program Platform Proof
- Production milestone: `PM-UI-PROGRAM-012` - 6F Headless Fixture Diagnostics Source Map Proof
- Production milestone state after plan acceptance: `active`
- Roadmap item: `WR-146` - 6F Headless Fixture Diagnostics Source Map Proof
- Roadmap planning state after plan acceptance: `current_candidate`
- Roadmap blocker after plan acceptance: `B2`
- Contract target: `docs-site/src/content/docs/reports/implementation-plans/wr-146-6f-headless-fixture-diagnostics-source-map-proof/plan.md`
- Post-plan transition: Manifest Runner V2 records the milestone as `active` and the WR as `current_candidate`/`B2`.
- Next action after this plan is accepted: stop before product/runtime code unless rerun with explicit `--allow product_code --allow product_implementation` and all V5 gates pass.

## Source Documents

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`

## Exact Files/Modules Expected To Change

- `domain/ui/ui_definition/src/preview_fixture/mod.rs`
- `domain/ui/ui_definition/src/diagnostic.rs`
- `domain/ui/ui_definition/src/source.rs`
- `domain/ui/ui_runtime/src/output/build_ui_frame.rs`
- `domain/ui/ui_render_data/src/lib.rs`
- `docs-site/src/content/docs/reports/implementation-plans/wr-146-6f-headless-fixture-diagnostics-source-map-proof/proof-aggregation.yaml`

## Expected Methods/Functions

- Headless fixture, diagnostics, source-map, migration evidence, artifact manifest/table reproducibility helpers only.

## Required Implementation Scope

- Headless fixture diagnostics source-map implementation/hardening plan.
- Accepted 6A through 6E evidence aggregation and missing-behavior return rule.
- Exact files/modules allowed, runtime/test evidence, rollback, compatibility, and stop conditions.

## Required Decisions

- 6F may aggregate accepted 6A through 6E evidence but must not implement missing prior-slice behavior.
- Missing behavior returns to the owning milestone.
- 6F is an aggregation/proof slice. It may read accepted 6A through 6E closeouts and runtime evidence, but it must not patch prior product-code files or implement missing prior behavior.
- Missing prior evidence fails closed and must be returned to the owning earlier milestone instead of being repaired inside WR-146.

## Implementation Writer Strategy

- Strategy: `proof_aggregation_writer`
- Aggregation only: `true`
- Required prior milestones:
  - `PM-UI-PROGRAM-007`
  - `PM-UI-PROGRAM-008`
  - `PM-UI-PROGRAM-009`
  - `PM-UI-PROGRAM-010`
  - `PM-UI-PROGRAM-011`
- Required prior completion quality: `runtime_proven`
- Required evidence categories:
  - headless fixture
  - diagnostics
  - source-map proof
  - runtime artifact evidence
  - reproducibility evidence
- Required machine-readable aggregation artifact:
  - `docs-site/src/content/docs/reports/implementation-plans/wr-146-6f-headless-fixture-diagnostics-source-map-proof/proof-aggregation.yaml`
- Writer allowed scopes:
  - `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`
  - `docs-site/src/content/docs/reports/track-execution-manifests/pt-ui-program/manifest.md`
  - `docs-site/src/content/docs/reports/implementation-plans/wr-146-6f-headless-fixture-diagnostics-source-map-proof/plan.md`
  - `docs-site/src/content/docs/reports/implementation-plans/wr-146-6f-headless-fixture-diagnostics-source-map-proof/proof-aggregation.yaml`
- Writer forbidden scopes:
  - prior 6A through 6E product implementation files
  - missing prior-slice behavior
  - broad fixture framework
  - MaterialProgram implementation
  - shared `foundation/meta` extraction
- Writer closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-012-6f-headless-fixture-diagnostics-source-map-proof/closeout.md`
- The closeout path is not a writer output. It is reserved for the later runtime closeout action after product evidence exists.
- The writer must write the machine-readable aggregation artifact before PM-012 can close.
- The writer contract makes PM-012 eligible for the manifest runner gate, but this plan does not run `product_code` or write 6F implementation artifacts.

## Forbidden Files/Modules

- implementing missing 6A-6E behavior
- broad fixture framework
- new crates
- shared foundation/meta extraction
- prior 6A through 6E product implementation files
- MaterialProgram implementation

## Tests To Add/Change

- Focused 6F headless fixture, diagnostics, source-map, migration, and artifact reproducibility tests in scoped modules only.
- Do not create new test folders, new crates, placeholder future modules, or broad fixture frameworks.

## Validation Commands

- `cargo test -p ui_definition preview_fixture`
- `cargo test -p ui_definition diagnostic`
- `cargo test -p ui_definition source`
- `cargo test -p ui_runtime build_ui_frame`
- `cargo test -p ui_render_data`
- `task docs:validate`
- `task planning:validate`

## Closeout Requirements

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-012-6f-headless-fixture-diagnostics-source-map-proof/closeout.md`
- Closeout evidence must include:
- The machine-readable aggregation artifact at `docs-site/src/content/docs/reports/implementation-plans/wr-146-6f-headless-fixture-diagnostics-source-map-proof/proof-aggregation.yaml`.
- 6F aggregates accepted 6A through 6E evidence.
- 6F proves deterministic fixture/debug serialization, diagnostics, source maps, migration evidence, UiRuntimeArtifactManifest, UiRuntimeArtifactTables, and reproducibility.
- Missing behavior must return to the owning milestone.
- Closeout must explicitly state that no renderer-owned UI truth, ECS-owned UI semantics, MaterialProgram implementation, crates, placeholder folders, or `foundation/meta` extraction occurred.

## Compatibility / Rollback Plan

- Retained/current UI stays compatible; rollback is limited to the exact 6F proof files.

## Acceptance Checklist

- Plan names every exact 6F proof write scope.
- Plan preserves no broad fixture framework and no foundation/meta extraction.

## Stop Conditions

- stop if product_code permission is not explicitly granted
- stop if the active WR or accepted implementation plan is missing
- stop if validation fails
- stop after one implementation WR unless the runner recomputes and all closeout gates pass
- stop if any required prior milestone is not `runtime_proven`
- stop if any required prior closeout is missing
- stop if any required evidence category is missing
- stop if aggregation would modify prior proof-slice product files
- Stop before product/runtime code unless the command is rerun with `--allow product_code --allow product_implementation` and all V5 gates pass.
- Stop before the next milestone until this milestone has runtime/test closeout evidence.

## Next Command If Product Code Is Permitted

`task production:run-track -- --track PT-UI-PROGRAM --allow product_code --allow product_implementation --max-actions 1`
