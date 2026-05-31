---
title: WR-135 UI Program Platform Proof Track Governance Contract
description: Design-first governance contract for PM-UI-PROGRAM-001, activating PT-UI-PROGRAM as the UiProgram platform proof track without authorizing product code or shared foundation/meta extraction.
status: active
owner: ui
layer: workspace / domain/ui
canonical: true
last_reviewed: 2026-05-31
related_designs:
  - ../../../design/active/runenwerk-domain-workbench-north-star.md
  - ../../../design/active/ui-program-architecture.md
  - ../../../design/active/ui-program-proof-slice-plan.md
related_reports:
  - ../../roadmap-intake/2026-05-31-activate-pt-ui-program-as-the-uiprogram-/proposal.yaml
  - ../../track-execution-manifests/pt-ui-program/manifest.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-deferred.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-135 UI Program Platform Proof Track Governance Contract

## Goal

Activate `PT-UI-PROGRAM` as the dedicated production track for the UiProgram
platform proof while keeping the work docs/governance-only.

This contract is produced from:

```text
task production:plan -- --milestone PM-UI-PROGRAM-001 --roadmap WR-135
```

The command currently classifies the next action as `design_first`. This
contract records the production-track boundary, Stage 0 through Stage 7
sequence, source documents, validation gates, stop conditions, and closeout
requirements before any UiProgram design, migration, or proof-slice
implementation may begin.

## Source Of Truth

- Production track and milestone:
  `docs-site/src/content/docs/workspace/production-tracks.yaml` track
  `PT-UI-PROGRAM`, milestone `PM-UI-PROGRAM-001`.
- Roadmap row:
  `docs-site/src/content/docs/workspace/roadmap-deferred.yaml` item `WR-135`.
- Platform north-star:
  `docs-site/src/content/docs/design/active/runenwerk-domain-workbench-north-star.md`.
- UiProgram proving-domain architecture:
  `docs-site/src/content/docs/design/active/ui-program-architecture.md`.
- Bounded proof-slice plan:
  `docs-site/src/content/docs/design/active/ui-program-proof-slice-plan.md`.
- Track Execution Manifest:
  `docs-site/src/content/docs/reports/track-execution-manifests/pt-ui-program/manifest.md`.

## Readiness

`task production:plan -- --milestone PM-UI-PROGRAM-001 --roadmap WR-135`
reports:

```text
Production milestone state: active
Roadmap planning_state: blocked_deferred
Roadmap blocker: B4
Roadmap dependencies: none
Next action: design_first
Contract target: docs-site/src/content/docs/reports/implementation-plans/wr-135-ui-program-platform-proof-track-governance-and-activation/plan.md
```

`WR-135` is not ready for product implementation. The only allowed work under
this contract is governance work that makes the production track and roadmap
state explicit.

## Architecture Governance Review

Ownership:

- `domain/ui` owns the future UiProgram proof contracts and all UI-owned
  schema, control, binding, state, compiler, evaluator, artifact, host-contract,
  render-data, text, accessibility, geometry, and testing responsibilities.
- Concrete editor, game, world-space, and headless integrations stay outside
  `domain/ui` until a bounded stage names the integration surface.
- Workspace planning metadata belongs to
  `docs-site/src/content/docs/workspace`.
- Proof-slice contracts and closeouts belong under
  `docs-site/src/content/docs/reports`.

Dependency direction:

```text
foundation -> domain/ui -> domain/editor or engine hosts -> apps
```

Forbidden outcomes:

- no shared `foundation/meta` extraction;
- no new crates;
- no crate renames;
- no placeholder future folders;
- no ECS-owned UI semantics;
- no renderer-owned product truth;
- no generic node soup;
- no giant `UiSemanticEvent` enum;
- no broad runtime implementation from this governance milestone.

ADR need: no ADR is required for this governance activation while it records
the accepted direction and preserves existing dependency direction. Require a
new accepted design or ADR before changing dependency direction, extracting
shared platform primitives, moving UI truth into a renderer, or moving UI
semantics into ECS.

## Implementation Scope

Allowed write scope for `PM-UI-PROGRAM-001`:

- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- `docs-site/src/content/docs/workspace/track-execution-manifests/pt-ui-program.yaml`;
- generated production docs from `task production:render`;
- roadmap intake and generated roadmap docs from the repository workflow;
- the generated Track Execution Manifest report;
- this implementation-plan contract;
- a future closeout for `PM-UI-PROGRAM-001` / `WR-135`.

Forbidden write scope:

- product code;
- Rust crates or crate manifests;
- placeholder module folders;
- runtime implementation;
- MaterialProgram implementation;
- shared `foundation/meta` or platform primitive extraction.

## Milestone Boundary

`PM-UI-PROGRAM-001` promotes the accepted design into production planning only.
It may establish:

- the `PT-UI-PROGRAM` production track;
- the ordered milestones `PM-UI-PROGRAM-001` through
  `PM-UI-PROGRAM-013`;
- required design dependencies on the north-star, UI architecture, and
  proof-slice plan;
- the Track Execution Manifest for Stage 0 through Stage 7;
- `runtime_proven` as the final track target, defined by real runtime,
  headless, artifact, diagnostic, and source-map evidence across 6A through 6F;
- the non-extraction rule that completing the UI track only enables the
  MaterialProgram second-domain proof handoff.

It must not complete, start, or partially implement Stage 1 through Stage 7.

## Proof-Slice Order

The production track keeps this order:

1. `PM-UI-PROGRAM-001` - ADR / Design Promotion.
2. `PM-UI-PROGRAM-002` - UI Program Contract Design.
3. `PM-UI-PROGRAM-003` - Control Package Proof Design.
4. `PM-UI-PROGRAM-004` - Compiler / Runtime Artifact Design.
5. `PM-UI-PROGRAM-005` - Evaluator / Host Design.
6. `PM-UI-PROGRAM-006` - Retained UI Migration Design.
7. `PM-UI-PROGRAM-007` - 6A Label Structural UiFrame Text Proof.
8. `PM-UI-PROGRAM-008` - 6B Button Route Event Host Command Proof.
9. `PM-UI-PROGRAM-009` - 6C InspectorField Binding State Proof.
10. `PM-UI-PROGRAM-010` - 6D ColorPicker ControlPackage Proof.
11. `PM-UI-PROGRAM-011` - 6E World Space Host Boundary Proof.
12. `PM-UI-PROGRAM-012` - 6F Headless Fixture Diagnostics Source Map Proof.
13. `PM-UI-PROGRAM-013` - Runtime Proven Closeout And MaterialProgram Handoff.

Each milestone after `PM-UI-PROGRAM-001` needs its own WR or accepted future WR
candidate, production plan, validation, and closeout before implementation or
closeout work.

## Acceptance Criteria

- `PT-UI-PROGRAM` exists as a separate active production track and is not folded
  into `PT-GAME-RUNTIME-UI` or previous UI Designer tracks.
- `PM-UI-PROGRAM-001` is linked to `WR-135` and remains docs/governance-only.
- The track records the accepted design dependencies:
  `runenwerk-domain-workbench-north-star.md`,
  `ui-program-architecture.md`, `ui-program-proof-slice-plan.md`, and the
  Track Execution Manifest.
- The track represents the full Stage 0 through Stage 7 sequence, not only the
  Stage 6 runtime proof slices.
- The track target is `runtime_proven`, defined as real runtime, headless,
  artifact, diagnostic, and source-map evidence across 6A through 6F.
- Completing `PT-UI-PROGRAM` does not authorize shared `foundation/meta`
  extraction; it only proves the UI domain and enables the MaterialProgram
  second-domain proof handoff.
- `PM-UI-PROGRAM-012` is recorded as an aggregation milestone that cannot
  implement missing behavior from 6A through 6E.
- `PM-UI-PROGRAM-013` records the UI proof result and creates or links the
  MaterialProgram second-domain proof path without starting MaterialProgram
  implementation or authorizing shared extraction.

## Validation

Run after metadata or contract changes:

```text
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-UI-PROGRAM
task production:plan -- --milestone PM-UI-PROGRAM-001 --roadmap WR-135
```

If roadmap source files are edited after intake, also run the roadmap render,
validate, and check tasks directly or through `task planning:validate`.

## Stop Conditions

Stop and report instead of continuing if:

- a validator fails;
- `WR-135` cannot be linked to `PM-UI-PROGRAM-001`;
- the production track would reopen completed UI Designer tracks;
- the track would be merged into `PT-GAME-RUNTIME-UI`;
- a change would authorize product code, crates, placeholder folders, runtime
  implementation, MaterialProgram implementation, or shared `foundation/meta`;
- Stage 1 through Stage 7 work is requested before `PM-UI-PROGRAM-001` is
  complete and a dedicated owning WR or accepted future WR candidate exists.

## Closeout Requirements

The future `PM-UI-PROGRAM-001` closeout must record:

- files changed;
- generated production and roadmap docs updated;
- validation output for production, planning, and docs checks;
- confirmation that no product code, crates, placeholder folders, runtime
  implementation, MaterialProgram implementation, or shared extraction was
  authorized;
- confirmation that Stage 1 cannot begin until a dedicated Stage 1 WR or
  accepted future WR candidate and production plan exist.

## Perfectionist Closeout Audit

`PM-UI-PROGRAM-001` can close only as `bounded_contract`. It cannot claim
`runtime_proven` because it provides no runtime proof. The track as a whole can
claim `runtime_proven` only after Stages 1 through 5 complete design and
migration planning, 6A through 6F produce real runtime, headless, artifact,
diagnostic, and source-map evidence, and `PM-UI-PROGRAM-013` records the
MaterialProgram handoff without authorizing extraction.
