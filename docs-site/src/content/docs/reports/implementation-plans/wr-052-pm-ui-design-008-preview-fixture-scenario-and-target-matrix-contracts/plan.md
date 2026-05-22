---
title: WR-052 PM-UI-DESIGN-008 Preview Fixture Scenario And Target Matrix Contracts Plan
description: Promotion and implementation-readiness contract for the bounded PM-UI-DESIGN-008 generic UI preview fixture, scenario, target matrix, and evidence descriptor slice.
status: active
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
related_adrs:
  - ../../../adr/accepted/0004-separate-description-from-execution.md
  - ../../../adr/accepted/0005-projections-are-derived-state.md
  - ../../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-052 PM-UI-DESIGN-008 Preview Fixture Scenario And Target Matrix Contracts Plan

## Goal

Promote `PM-UI-DESIGN-008` as one bounded generic UI preview fixture, scenario,
target matrix, and evidence descriptor contract slice.

The slice must add runtime-neutral fixture declarations, scenario declarations,
target matrix declarations, preview validation modes, expected diagnostic/state
references, matrix axes, and evidence descriptor diagnostics in
`domain/ui/ui_definition`. It must keep preview execution, screenshots,
renderer golden comparison, provider sessions, game-runtime replay, app windows,
and runtime state outside generic UI definition ownership.

This contract does not implement product code. It records the promotion and
implementation-readiness decisions for `WR-052`.

## Source Of Truth

- Production track: `PT-UI-DESIGN`.
- Production milestone: `PM-UI-DESIGN-008`.
- Bounded implementation row: `WR-052`.
- Accepted PM-008 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`.
- Accepted PM-007 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md`.
- Accepted target projection design:
  `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-UI-DESIGN-008 --roadmap WR-052`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-UI-DESIGN-004` remains completed with bounded visual layout closeout
  evidence.
- `PM-UI-DESIGN-007` remains completed with bounded binding and intent closeout
  evidence.
- `WR-046` remains support-only evidence for the UI Designer doctrine and
  target-boundary ratification.
- `WR-047` and `WR-051` remain completed prerequisite rows.
- `PM-UI-DESIGN-008` remains `ready_next` and links `WR-052`.
- `WR-052` remains `ready_next`, blocker `B2`, and depends only on completed or
  support-only prerequisite rows.
- The accepted PM-008 preview fixture design remains accepted and valid.
- No current-candidate WR row conflicts with `WR-052` write scopes.

The expected promotion command is:

```text
task roadmap:promote -- --id WR-052 --state current_candidate --evidence "PM-UI-DESIGN-008 accepted preview fixture design and production plan establish the bounded generic UI preview fixture, scenario, target matrix, and evidence descriptor contract at docs-site/src/content/docs/reports/implementation-plans/wr-052-pm-ui-design-008-preview-fixture-scenario-and-target-matrix-contracts/plan.md."
```

After promotion, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, docs validate,
planning validate, and `task ai:goal -- --track PT-UI-DESIGN` before product
code changes.

## Allowed Write Scope

Implementation may touch only:

- `domain/ui/ui_definition` for generic fixture ids, scenario ids, matrix ids,
  data-state kinds, target-profile compatibility, scenario steps, policy
  references, matrix axes, expected diagnostics, expected state references,
  evidence descriptors, validation modes, and diagnostics;
- `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`
  only if implementation proves that the accepted PM-008 design needs a narrow
  clarification;
- this implementation plan;
- roadmap and production-track source/generated files required by promotion,
  validation, and closeout;
- the eventual PM-008 closeout evidence after implementation validation passes.

No app-hosted Preview Lab UI, screenshot capture, renderer golden comparison,
provider session orchestration, game-runtime replay loading, persistence
activation, or production readiness work is in scope for this first row.

## Owning Modules

Implementation should keep generic preview fixture ownership inside the UI
definition domain:

- `domain/ui/ui_definition/src` module `ui_definition` owns Canonical UI IR and
  should own generic preview fixture, scenario, matrix, and evidence descriptor
  contracts.
- A new `domain/ui/ui_definition/src/preview_fixture/` subsystem with `mod.rs`
  is appropriate if implementation needs fixture ids, scenario ids, target
  matrix ids, axes, validation, diagnostics, and tests.
- `domain/editor/editor_definition` and future game UI target domains own
  target-specific fixture adapters; both remain out-of-scope for this first
  generic contract implementation.
- `apps/runenwerk_editor`, renderer, runtime, and provider layers remain
  out-of-scope except as future consumers.

## Implementation Scope

Add a generic preview fixture contract:

1. Define stable fixture ids, scenario ids, matrix ids, data-state kinds,
   target-profile ids, matrix axes, scenario steps, policy references, expected
   diagnostic refs, expected state refs, evidence descriptor ids, and
   diagnostics.
2. Define fixture declarations that describe deterministic preview inputs
   without owning provider or runtime state.
3. Define scenario declarations that describe replayable interaction steps
   without direct domain mutation.
4. Define target matrix declarations across target profile, platform,
   accessibility, localization, input, size, and performance axes.
5. Reject unknown fixture/scenario refs, unsupported target profiles, missing
   data packages, denied capabilities, stale evidence, incompatible axes,
   invalid scenario steps, expected diagnostic mismatches, and preview-only
   activation.
6. Add focused unit tests in `domain/ui/ui_definition`.

## Non-Goals

Do not implement:

- app-hosted Preview Lab UI;
- visual capture, screenshot diffing, or renderer golden comparison;
- provider session orchestration;
- game-runtime replay loading;
- persistence activation from `PM-UI-DESIGN-009`;
- production readiness from `PM-UI-DESIGN-010`.

## Acceptance Criteria

`WR-052` implementation is complete only when all criteria below are true:

- Generic preview fixture, scenario, target matrix, and evidence descriptor
  contracts exist in `domain/ui/ui_definition`.
- Fixture, scenario, and matrix ids are stable and deterministic.
- Fixture data states cover empty, loading, error, denied, offline, heavy, and
  accessibility modes.
- Target-profile, capability, missing data, scenario step, matrix axis, expected
  diagnostic mismatch, and preview-only activation failures produce blocking
  diagnostics.
- Focused tests prove editor/workbench and game-runtime examples without shared
  runtime, provider, screenshot, or renderer ownership.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-UI-DESIGN
```

Required validation after promotion and before any later implementation
closeout:

```text
cargo test -p ui_definition preview_fixture
cargo test -p ui_definition
task docs:validate
task roadmap:render
task roadmap:validate
task roadmap:check
task production:render
task production:validate
task production:check
task planning:validate
task ai:goal -- --track PT-UI-DESIGN
```

## Stop Conditions

Stop before implementation if:

- `WR-052` cannot be promoted legally;
- `PM-UI-DESIGN-008` is no longer `ready_next` or no longer links `WR-052`;
- the accepted PM-008 design is missing, no longer accepted, or contradicted by
  newer accepted design/ADR evidence;
- implementation would move provider sessions, runtime state, renderer handles,
  screenshots, app windows, gameplay truth, or direct mutation into generic UI
  fixtures;
- implementation requires write scopes outside this contract;
- focused tests cannot prove target profiles, fixture data states, scenario
  validation, matrix diagnostics, expected diagnostic checks, and preview-only
  activation behavior.

## Closeout Requirements

Closeout for this contract-writing action records this plan path, changed
roadmap path, validation results, the `task ai:goal -- --track PT-UI-DESIGN`
rerun, and confirmation that no Rust product code changed.

Implementation closeout must be created only after product implementation and
focused validation pass.

## Perfectionist Closeout Audit

Expected completion quality for the first `WR-052` implementation is
`bounded_contract`. It can claim `runtime_proven` only if a later explicit
Preview Lab path proves user-visible preview consumption, screenshots, or
runtime replay, which is outside this first generic contract slice.

`perfectionist_verified` is not expected for `WR-052` because visual capture,
renderer golden comparison, persistence activation, accessibility/performance
evidence, and production readiness remain later milestones.

## Contract-Writing Closeout Evidence

Status as of 2026-05-22: completed for the contract-writing action only.

Changed artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-052-pm-ui-design-008-preview-fixture-scenario-and-target-matrix-contracts/plan.md`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Validation:

- `task docs:validate` passed after the contract was added.
- `task roadmap:render` passed after the contract path was added to `WR-052`
  write scopes.
- `task roadmap:validate` passed after the contract path was added to
  `WR-052` write scopes.
- `task roadmap:check` passed after roadmap render refreshed generated roadmap
  docs.
- `task production:render` passed after the contract was added.
- `task production:validate` passed after the contract was added.
- `task production:check` passed after production render refreshed generated
  production docs.
- `task planning:validate` passed after the contract and roadmap metadata were
  updated, covering roadmap validation/check, production validation/check, and
  docs validation.

Closeout result:

- No Rust product code changed.
- No production-track state changed during this contract-writing action.
- Roadmap metadata changed only to include this contract path in `WR-052` write
  scopes. The row remained `ready_next` until explicit promotion.
- The next action after this closeout is to rerun
  `task ai:goal -- --track PT-UI-DESIGN`, then promote `WR-052` only if the
  production-plan preflight still reports it as promotable.

## Promotion And Implementation Readiness Evidence

Status as of 2026-05-22: `WR-052` is promoted to `current_candidate`.

Promotion evidence:

- `task production:plan -- --milestone PM-UI-DESIGN-008 --roadmap WR-052`
  reported promotion preflight status `promotable`.
- `task roadmap:promote -- --id WR-052 --state current_candidate --evidence
  "PM-UI-DESIGN-008 accepted preview fixture design and production plan
  establish the bounded generic UI preview fixture, scenario, target matrix, and
  evidence descriptor contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-052-pm-ui-design-008-preview-fixture-scenario-and-target-matrix-contracts/plan.md."`
  promoted `WR-052` to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after promotion.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task docs:validate` and `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-UI-DESIGN` reported `PM-UI-DESIGN-008` next
  legal action as `execute_next_wr_implementation_contract`.

Implementation may start only from this contract and must stay inside the
allowed write scope, focused test set, stop conditions, and closeout
requirements above.
