---
title: WR-139 Evaluator / Host Design Plan
description: Design/planning contract for PM-UI-PROGRAM-005 under WR-139.
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

# WR-139 Evaluator / Host Design Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-005` - Evaluator / Host Design
- Stage: Stage 4
- Roadmap item: `WR-139` - Evaluator / Host Design
- Authority: design/planning only.
- This plan does not authorize product/runtime code, crate creation, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, or shared foundation/meta extraction.
- This plan does not close the milestone. Closeout requires separate manual evidence or an explicit `agent_closeout` automation run.

## Production Planning Output

# Production Plan Readiness

Production track: PT-UI-PROGRAM - UI Program Platform Proof
Production milestone: PM-UI-PROGRAM-005 - Evaluator / Host Design
Production milestone state: designing
Roadmap item: WR-139 - Evaluator / Host Design
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-138:completed
Milestone links WR item: yes
Contract target: docs-site/src/content/docs/reports/implementation-plans/wr-139-evaluator-host-design/plan.md
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

- UiEvaluator
- editor host contract
- game host contract
- world-space host contract
- headless host contract
- host consumption of UiEventPacket
- frame submission boundary
- UiEventPacket HostCommand DomainCommand mapping rules
- editor inspector proof scenario
- game HUD proof scenario
- world-space entity prompt proof scenario
- headless fixture proof scenario

## Required Decisions

- UiEvaluator consumes UiRuntimeArtifactTables plus host snapshots and produces UiOutput without owning editor, game, world, or renderer truth.
- EditorHost owns editor integration and maps UiEventPacket to editor HostCommand and optional DomainCommand.
- GameHost owns HUD/game integration and maps route packets to game commands through inspectable host route maps.
- WorldSpaceHost owns projection, anchor, lifetime, visibility, and data feed inputs while ECS does not own UI semantics.
- HeadlessHost owns deterministic fixture evaluation, diagnostics, source-map checks, and artifact reproducibility evidence.
- Frame submission is a structural UiFrame or equivalent render-data boundary; renderer resources are not UI truth.

## Exact Write Scope

- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-archive.yaml`
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`
- `docs-site/src/content/docs/reports/track-execution-manifests/pt-ui-program/manifest.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-139-evaluator-host-design/plan.md`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-program-005-evaluator-host-design/closeout.md`
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

- Stage 4 design evidence defines all four host contracts and proof scenarios.
- Stage 4 design evidence defines UiEventPacket to HostCommand and optional DomainCommand mapping rules.
- Stage 4 design evidence preserves ECS and renderer boundaries.
- Stage 4 design evidence preserves no product code, no crates, no placeholder folders, no Stage 6 work, no MaterialProgram implementation, and no foundation/meta extraction.

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

- stop until PM-UI-PROGRAM-004 is completed
- stop until Track Expansion creates or links the owning WR
- stop if host, event, route, command, capability, or headless boundaries remain implicit
- Stop after writing the design/planning contract and rerun `task production:next -- --track PT-UI-PROGRAM`.
- Stop before closeout unless rerun with `--allow agent_closeout` and the required evidence and validation pass.

## Closeout Expectation

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-005-evaluator-host-design/closeout.md`
- Closeout must prove the Stage 4 design sections exist, answer or explicitly block open questions, and preserve all forbidden-scope constraints.
