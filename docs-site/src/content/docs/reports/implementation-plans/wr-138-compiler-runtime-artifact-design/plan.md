---
title: WR-138 Compiler / Runtime Artifact Design Plan
description: Design/planning contract for PM-UI-PROGRAM-004 under WR-138.
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

# WR-138 Compiler / Runtime Artifact Design Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-004` - Compiler / Runtime Artifact Design
- Stage: Stage 3
- Roadmap item: `WR-138` - Compiler / Runtime Artifact Design
- Authority: design/planning only.
- This plan does not authorize product/runtime code, crate creation, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, or shared foundation/meta extraction.
- This plan does not close the milestone. Closeout requires separate manual evidence or an explicit `agent_closeout` automation run.

## Production Planning Output

# Production Plan Readiness

Production track: PT-UI-PROGRAM - UI Program Platform Proof
Production milestone: PM-UI-PROGRAM-004 - Compiler / Runtime Artifact Design
Production milestone state: designing
Roadmap item: WR-138 - Compiler / Runtime Artifact Design
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-137:completed
Milestone links WR item: yes
Contract target: docs-site/src/content/docs/reports/implementation-plans/wr-138-compiler-runtime-artifact-design/plan.md
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

- UiCompiler inputs and outputs
- UiRuntimeArtifactManifest
- UiRuntimeArtifactTables
- cache keys and invalidation
- host target profiles
- compiler/evaluator timing
- hot-path artifact boundary
- state layout tables
- binding snapshot tables
- collection diff plans
- text layout request tables

## Required Decisions

- UiCompiler consumes UiProgram, packages, themes, host profile, binding shape, route maps, source inputs, and schema versions.
- UiCompiler outputs an inspectable UiRuntimeArtifactManifest plus optimized UiRuntimeArtifactTables.
- Cache keys include source IDs, package IDs, schema IDs, route map versions, theme versions, host target profile, text policy, and compiler version.
- UiEvaluator runs after compilation against runtime artifact tables; hot paths must not interpret generic graphs by default.
- Runtime artifact tables include state layout, binding snapshots, collection diff plans, interaction routing, visual output, accessibility, inspection, and text layout request tables.

## Exact Write Scope

- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`
- `docs-site/src/content/docs/reports/track-execution-manifests/pt-ui-program/manifest.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-138-compiler-runtime-artifact-design/plan.md`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-program-004-compiler-runtime-artifact-design/closeout.md`
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

- Stage 3 design evidence separates durable manifest metadata from optimized in-memory tables.
- Stage 3 design evidence defines invalidation and cache key behavior.
- Stage 3 design evidence preserves the compiler/evaluator timing boundary and hot-path artifact rule.
- Stage 3 design evidence preserves no product code, no crates, no placeholder folders, no Stage 6 work, no MaterialProgram implementation, and no foundation/meta extraction.

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

- stop until PM-UI-PROGRAM-003 is completed
- stop until Track Expansion creates or links the owning WR
- stop if manifest/table split, artifact IDs, source maps, cache invalidation, or hot-path artifact strategy remains undecided
- Stop after writing the design/planning contract and rerun `task production:next -- --track PT-UI-PROGRAM`.
- Stop before closeout unless rerun with `--allow agent_closeout` and the required evidence and validation pass.

## Closeout Expectation

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-004-compiler-runtime-artifact-design/closeout.md`
- Closeout must prove the Stage 3 design sections exist, answer or explicitly block open questions, and preserve all forbidden-scope constraints.
