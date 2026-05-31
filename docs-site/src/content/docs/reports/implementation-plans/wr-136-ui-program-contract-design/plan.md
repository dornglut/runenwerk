---
title: WR-136 UI Program Contract Design Plan
description: Design/planning contract for PM-UI-PROGRAM-002 under WR-136.
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

# WR-136 UI Program Contract Design Plan

## Status And Authority

- Production milestone: `PM-UI-PROGRAM-002` - UI Program Contract Design
- Roadmap item: `WR-136` - UI Program Contract Design
- Authority: design/planning only.
- This plan does not authorize product/runtime code, crate creation, placeholder future folders, Stage 6 proof work, MaterialProgram implementation, or shared foundation/meta extraction.
- This plan does not close the milestone. Closeout requires separate manual evidence or a future `agent_closeout` automation layer.

## Production Planning Output

# Production Plan Readiness

Production track: PT-UI-PROGRAM - UI Program Platform Proof
Production milestone: PM-UI-PROGRAM-002 - UI Program Contract Design
Production milestone state: designing
Roadmap item: WR-136 - UI Program Contract Design
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: WR-135:completed
Milestone links WR item: yes
Contract target: docs-site/src/content/docs/reports/implementation-plans/wr-136-ui-program-contract-design/plan.md
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

## Required Stage 1 Sections

- UiProgram graph ownership and graph list
- UiSchemaValue primitive/object/list/optional/route/opaque-host-reference model
- Schema versioning and validation rules
- Stable ID namespace/version/collision/deprecation/migration policy
- StateGraph and UiStateModel ownership split
- BindingGraph read/write/snapshot/dirty/collection/authorization contract
- VisualGraph and structural UiFrame renderer boundary
- Text/render boundary for shaping, glyph identity, atlas keys, invalidation, and renderer residency
- UiEventPacket and schema-based payload contract
- RouteId, RouteSchemaVersion, RouteCapability, and HostRouteMapVersion
- HostCommand and DomainCommand mapping rules
- Source-map attachment points
- Diagnostics attachment points
- Open questions and blocked decisions

## Required Decisions

- UiProgram owns typed UI graph truth and remains domain/ui-owned during the proof.
- UiSchemaValue remains UI-owned until the Second-Domain Extraction Gate.
- Routes are stable IDs with schema versions and capability requirements, not magic strings.
- UiEventPacket uses schema-based payloads and never grows into a giant UiSemanticEvent enum.
- Renderer handles are derived execution resources and never UI truth.
- Stage 6 runtime proof work remains blocked until Stages 1 through 5 close.

## Exact Write Scope

- `docs-site/src/content/docs/workspace/production-tracks.yaml`
- `docs-site/src/content/docs/workspace/roadmap-deferred.yaml`
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`
- `docs-site/src/content/docs/reports/track-execution-manifests/pt-ui-program/manifest.md`
- `docs-site/src/content/docs/reports/implementation-plans/wr-136-ui-program-contract-design/plan.md`
- `docs-site/src/content/docs/reports/closeouts/pm-ui-program-002-ui-program-contract-design/closeout.md`
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

- Design sections exist and are internally consistent.
- No product/runtime code, crate, placeholder folder, 6A, MaterialProgram, or foundation/meta scope is touched.
- PM-002 open questions are answered or explicitly blocked.
- The implementation/design plan exists at the linked WR-136 plan path.
- Docs, production, roadmap, manifest, and planning validation pass.
- agent_design output stops before closeout; agent_closeout handles bounded-contract closeout separately.

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

- stop until PM-UI-PROGRAM-001 is completed
- stop until Track Expansion creates or links the owning WR
- stop if graph ownership, schema/event/route boundary, or stable ID policy remains undecided
- Stop after writing the design/planning contract and rerun `task production:next -- --track PT-UI-PROGRAM`.
- Stop before closeout unless rerun with `--allow agent_closeout` and the required evidence and validation pass.

## Closeout Expectation

- Expected closeout path: `docs-site/src/content/docs/reports/closeouts/pm-ui-program-002-ui-program-contract-design/closeout.md`
- Closeout must prove the Stage 1 design sections exist, answer or explicitly block open questions, and preserve all forbidden-scope constraints.
