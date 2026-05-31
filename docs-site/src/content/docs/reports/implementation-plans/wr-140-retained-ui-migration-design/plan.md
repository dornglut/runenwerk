---
title: WR-140 Retained UI Migration Design Plan
description: Design/planning contract for PM-UI-PROGRAM-006 under WR-140.
status: active
owner: ui
layer: workspace / domain/ui
canonical: false
last_reviewed: 2026-05-25
related:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/track-execution-manifests/pt-ui-program.yaml
related_designs:
  - ../../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../../design/active/ui-program-architecture.md
---

# WR-140 Retained UI Migration Design Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-006` - Retained UI Migration Design
- Stage: Stage 5
- Roadmap item: `WR-140` - Retained UI Migration Design
- Authority: design/planning only.
- This plan does not authorize product/runtime code, crate creation, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, or shared foundation/meta extraction.
- This plan does not close the milestone. Closeout requires separate manual evidence or an explicit `agent_closeout` automation run.

## Production Planning Output

# Production Plan Readiness

Production track: PT-UI-PROGRAM - UI Program Platform Proof
Production milestone: PM-UI-PROGRAM-006 - Retained UI Migration Design
Production milestone state: designing
Roadmap item: WR-140 - Retained UI Migration Design
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-139:completed
Milestone links WR item: yes
Contract target: docs-site/src/content/docs/reports/implementation-plans/wr-140-retained-ui-migration-design/plan.md
Next action: design_first

## Readiness Notes

- Design or gate repair must happen before implementation planning.

## Prompt Template

- docs-site/src/content/docs/workspace/prompt-templates/production-implementation-contract.md


## Source Documents

- `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`
- `docs-site/src/content/docs/workspace/track-execution-manifest.md`
- `docs-site/src/content/docs/workspace/design-track-roadmap-governance.md`

## Required Design Sections

- migration from retained UI to UiProgram path
- current retained UI compatibility
- dual lowering or strangler strategy
- retained runtime compatibility adapters
- exact boundary for old ui_runtime compatibility
- rules for when old paths can be deleted
- proof prerequisites before replacement
- no broad retained UI rewrite

## Required Decisions

- Retained/current UI remains compatible until a bounded proof slice intentionally replaces a named surface.
- Authored UI definitions may dual-lower to retained UI and UiProgram during migration.
- Old ui_runtime compatibility becomes strangler infrastructure and must not remain the future semantic core.
- Compatibility adapters may bridge retained rendering/input only where a WR names the bounded surface and rollback path.
- Old paths may be deleted only after replacement proof, diagnostics parity, fixture evidence, compatibility closeout, and explicit deletion WR authority.
- Broad retained UI rewrite is forbidden; migration proceeds by bounded proof surfaces.

## Exact Write Scope

- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`
- `docs-site/src/content/docs/reports/track-execution-manifests/pt-ui-program/manifest.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-140-retained-ui-migration-design/plan.md`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-program-006-retained-ui-migration-design/closeout.md`
- `generated: production docs from task production:render`
- `generated: roadmap docs from task roadmap:render`
- `docs-site/src/content/docs/design/active/ui-program-architecture.md`
- `docs-site/src/content/docs/design/active/ui-program-contract-design.md`

## Forbidden Scope

- no product code from this manifest alone
- no new crates
- no crate renames
- no placeholder future folders
- no UI runtime implementation before the owning implementation milestone
- no shared foundation/meta extraction
- no MaterialProgram implementation
- no RenderPlan substitution for MaterialProgram as the second-domain proof

## Acceptance Checklist

- Stage 5 design evidence defines the dual-lowering or strangler path and rollback expectations.
- Stage 5 design evidence names the exact old ui_runtime compatibility boundary.
- Stage 5 design evidence defines deletion prerequisites for old retained UI paths.
- Stage 5 design evidence preserves no product code, no crates, no placeholder folders, no Stage 6 work, no MaterialProgram implementation, and no foundation/meta extraction.

## Validation Commands

- `task production:render`
- `task production:validate`
- `task production:check`
- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task docs:validate`
- `task planning:validate`

## Stop Conditions

- stop until PM-UI-PROGRAM-005 is completed
- stop until Track Expansion creates or links the owning WR
- stop if compatibility, dual-lowering, rollback, replacement, or drift guard policy remains undecided
- Stop after writing the design/planning contract and rerun `task production:next -- --track PT-UI-PROGRAM`.
- Stop before closeout unless rerun with `--allow agent_closeout` and the required evidence and validation pass.

## Closeout Expectation

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-006-retained-ui-migration-design/closeout.md`
- Closeout must prove the Stage 5 design sections exist, answer or explicitly block open questions, and preserve all forbidden-scope constraints.
