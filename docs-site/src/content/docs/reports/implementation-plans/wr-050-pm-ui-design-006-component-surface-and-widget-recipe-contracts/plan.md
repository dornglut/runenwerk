---
title: WR-050 PM-UI-DESIGN-006 Component Surface And Widget Recipe Contracts Plan
description: Promotion and implementation-readiness contract for the bounded PM-UI-DESIGN-006 generic UI recipe contract slice.
status: active
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
related_adrs:
  - ../../../adr/accepted/0004-separate-description-from-execution.md
  - ../../../adr/accepted/0005-projections-are-derived-state.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-050 PM-UI-DESIGN-006 Component Surface And Widget Recipe Contracts Plan

## Goal

Promote `PM-UI-DESIGN-006` as one bounded generic UI recipe contract slice.

The slice must add runtime-neutral component, widget, and surface recipe
contracts in `domain/ui/ui_definition`: stable recipe ids, recipe kinds, slots,
token-family requirements, accessibility metadata, layout/focus/navigation
descriptors, target-profile compatibility, deterministic expansion into
Canonical UI IR, and typed diagnostics.

This contract does not implement product code. It records the promotion and
implementation-readiness decisions for `WR-050`.

## Source Of Truth

- Production track: `PT-UI-DESIGN`.
- Production milestone: `PM-UI-DESIGN-006`.
- Bounded implementation row: `WR-050`.
- Accepted PM-006 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md`.
- Accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Accepted target projection design:
  `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md`.
- Accepted theme token design:
  `docs-site/src/content/docs/design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-UI-DESIGN-006 --roadmap WR-050`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-UI-DESIGN-002` remains completed with accepted design and closeout
  evidence.
- `PM-UI-DESIGN-005` remains completed with bounded token graph closeout
  evidence.
- `WR-046` remains support-only evidence for the UI Designer doctrine and
  target-boundary ratification.
- `WR-049` remains completed bounded theme token graph evidence.
- `PM-UI-DESIGN-006` remains `ready_next` and links `WR-050`.
- `WR-050` remains `ready_next`, blocker `B2`, and depends only on completed or
  support-only prerequisite rows.
- The accepted PM-006 recipe design remains accepted and valid.
- No current-candidate WR row conflicts with `WR-050` write scopes.

The expected promotion command is:

```text
task roadmap:promote -- --id WR-050 --state current_candidate --evidence "PM-UI-DESIGN-006 accepted recipe design and production plan establish the bounded generic UI component, widget, and surface recipe contract at docs-site/src/content/docs/reports/implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md."
```

After promotion, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, docs validate,
planning validate, and `task ai:goal -- --track PT-UI-DESIGN` before product
code changes.

## Allowed Write Scope

Implementation may touch only:

- `domain/ui/ui_definition` for generic recipe ids, declarations, slots,
  token-reference requirements, accessibility metadata, target-profile
  compatibility, deterministic expansion, and diagnostics;
- `domain/ui/ui_theme` only for narrow token reference or token-family contract
  alignment needed by recipes;
- `docs-site/src/content/docs/design/accepted/ui-designer-component-surface-and-widget-recipe-library-design.md`
  only if implementation proves that the accepted PM-006 design needs a narrow
  clarification;
- `docs-site/src/content/docs/reports/implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md`
  for this contract;
- roadmap and production-track source/generated files required by promotion,
  validation, and closeout;
- the eventual PM-006 closeout evidence after implementation validation passes.

No app-hosted recipe browser, editor-specific recipe package persistence,
game-runtime package loading, renderer/material lowering, view-model binding,
intent activation, live preview matrix, persistence activation, or production
readiness work is in scope for this first row.

## Owning Modules

Implementation should keep generic recipe ownership inside the UI definition
domain:

- `domain/ui/ui_definition/src` module `ui_definition` owns Canonical UI IR and
  should own recipe contracts unless implementation proves a subdomain folder is
  clearer.
- A new `domain/ui/ui_definition/src/component_recipe/` subsystem with `mod.rs`
  is appropriate if the implementation needs recipe ids, declarations, slots,
  expansion, diagnostics, and tests.
- `domain/ui/ui_theme/src/token/mod.rs` module `token` remains the source for
  token ids and token families. PM-006 should consume those contracts, not
  duplicate token graph behavior.
- `domain/editor/editor_definition` and `apps/runenwerk_editor` remain
  out-of-scope for this first generic recipe contract implementation.

## Implementation Scope

Add a generic recipe contract and deterministic expander:

1. Define stable recipe ids, recipe kinds, source package ids, source locations,
   target-profile ids, slot ids, accepted child recipe kinds, token-family
   requirements, accessibility metadata, focus/navigation descriptors, and
   recipe diagnostics.
2. Define component, widget, and surface recipe declarations over Canonical UI
   IR node templates without app/runtime/provider ownership.
3. Expand a recipe declaration into Canonical UI IR deterministically while
   preserving stable authored ids.
4. Reject duplicate recipe ids, unknown child recipes, invalid slot children,
   missing required slots, missing required token families, missing
   accessibility metadata, invalid focus/navigation descriptors, unsupported
   target profiles, and preview-only activation attempts.
5. Produce typed diagnostics with source path, target profile, owning domain,
   recipe id, slot path, source package, activation impact, and suggested fix.
6. Add focused unit tests in `domain/ui/ui_definition`.

## Non-Goals

Do not implement:

- app-hosted component, widget, or surface recipe UI;
- editor-specific recipe package persistence or project file IO;
- game-runtime recipe package loading;
- renderer material lowering, shader styles, or GPU resource policy;
- view-model capability or intent binding from `PM-UI-DESIGN-007`;
- live preview fixture matrices from `PM-UI-DESIGN-008`;
- persistence activation from `PM-UI-DESIGN-009`;
- production readiness from `PM-UI-DESIGN-010`.

## Acceptance Criteria

`WR-050` implementation is complete only when all criteria below are true:

- Generic UI component, widget, and surface recipe contracts exist in
  `domain/ui/ui_definition`.
- Recipe expansion into Canonical UI IR is deterministic and preserves stable
  ids.
- Slot, token-family, accessibility, focus/navigation, and target-profile
  failures produce blocking diagnostics.
- Preview-only recipe expansion cannot activate.
- Focused tests prove editor/workbench and game-runtime target-profile examples
  without app/runtime/editor shell ownership leaks.

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
cargo test -p ui_definition component_recipe
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

- `WR-050` cannot be promoted legally;
- `PM-UI-DESIGN-006` is no longer `ready_next` or no longer links `WR-050`;
- the accepted PM-006 design is missing, no longer accepted, or contradicted by
  newer accepted design/ADR evidence;
- implementation would move recipe source truth into app, runtime, editor shell,
  provider, renderer, material, gameplay, project, or session layers;
- implementation requires write scopes outside this contract;
- focused tests cannot prove deterministic expansion, stable ids, slot
  diagnostics, token-family diagnostics, accessibility diagnostics,
  target-profile diagnostics, and preview-only activation rejection.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task roadmap:validate` result;
- `task roadmap:check` result;
- `task production:validate` result;
- `task production:check` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-UI-DESIGN` rerun result;
- confirmation that no Rust product code changed for this contract-writing
  action.

Implementation closeout must be created only after product implementation and
focused validation pass.

## Perfectionist Closeout Audit

Expected completion quality for the first `WR-050` implementation is
`bounded_contract`. It can claim `runtime_proven` only if a later explicit
target-profile projection and preview path proves user-visible recipe
consumption, which is outside this first generic recipe contract slice.

`perfectionist_verified` is not expected for `WR-050` because app-hosted recipe
editing, preview matrices, persistence activation, accessibility/performance
evidence, and production readiness remain later milestones.

## Contract-Writing Closeout Evidence

Status as of 2026-05-22: completed for the contract-writing action only.

Changed artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Validation:

- `task docs:validate` passed after the contract was added.
- `task roadmap:validate` passed after the contract path was added to
  `WR-050` write scopes.
- `task roadmap:check` passed after roadmap render refreshed generated roadmap
  docs.
- `task production:validate` passed after promotion-contract metadata was
  updated.
- `task production:check` passed after production render refreshed generated
  production docs.
- `task planning:validate` passed after the contract and roadmap metadata were
  updated, covering roadmap validation/check, production validation/check, and
  docs validation.
- `task ai:goal -- --track PT-UI-DESIGN` was rerun after validation and
  reported `PM-UI-DESIGN-006` next legal action as
  `prepare_wr_promotion_contract`, with `WR-050` still `ready_next`.

Closeout result:

- No Rust product code changed.
- No production-track state changed during this contract-writing action.
- Roadmap metadata changed only to include this contract path in `WR-050` write
  scopes. The row remained `ready_next` until explicit promotion.
- The next action after this closeout was to promote `WR-050` only if the
  production-plan preflight still reported it as promotable.

## Promotion And Implementation Readiness Evidence

Status as of 2026-05-22: `WR-050` is promoted to `current_candidate`.

Promotion evidence:

- `task production:plan -- --milestone PM-UI-DESIGN-006 --roadmap WR-050`
  reported promotion preflight status `promotable`.
- `task roadmap:promote -- --id WR-050 --state current_candidate --evidence
  "PM-UI-DESIGN-006 accepted recipe design and production plan establish the
  bounded generic UI component, widget, and surface recipe contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-050-pm-ui-design-006-component-surface-and-widget-recipe-contracts/plan.md."`
  promoted `WR-050` to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after promotion.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task docs:validate` and `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-UI-DESIGN` reported `PM-UI-DESIGN-006` next
  legal action as `execute_next_wr_implementation_contract`.
- A final `task production:plan -- --milestone PM-UI-DESIGN-006 --roadmap
  WR-050` rerun reported `WR-050` as `current_candidate` and next action
  `write_implementation_contract`.

Implementation may start only from this contract and must stay inside the
allowed write scope, focused test set, stop conditions, and closeout
requirements above.
