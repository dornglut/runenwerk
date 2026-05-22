---
title: WR-049 PM-UI-DESIGN-005 Theme Token Graph And Mode Resolution Plan
description: Promotion and implementation-readiness contract for the bounded PM-UI-DESIGN-005 generic UI theme token graph slice.
status: active
owner: editor
layer: domain/ui-theme
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/active/ui-designer-interface-lab-platform-design.md
related_adrs:
  - ../../../adr/accepted/0004-separate-description-from-execution.md
  - ../../../adr/accepted/0005-projections-are-derived-state.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-049 PM-UI-DESIGN-005 Theme Token Graph And Mode Resolution Plan

## Goal

Promote and implement `PM-UI-DESIGN-005` as one bounded generic UI theme token
graph and mode-resolution slice.

The slice must add deterministic token graph contracts, alias resolution,
mode/state precedence, accessibility conflict diagnostics, preview-only
activation rejection, and target-profile compatibility diagnostics in
`domain/ui/ui_theme`. It must not implement app-hosted Theme Designer UI,
editor-specific package storage, game-runtime package loading, renderer
material lowering, or downstream PM-006 through PM-010 capabilities.

This contract does not implement product code. It records the promotion and
implementation-readiness decisions for `WR-049`.

## Source Of Truth

- Production track: `PT-UI-DESIGN`.
- Production milestone: `PM-UI-DESIGN-005`.
- Bounded implementation row: `WR-049`.
- Accepted PM-005 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md`.
- Accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Accepted target projection design:
  `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md`.
- Active platform doctrine:
  `docs-site/src/content/docs/design/active/ui-designer-interface-lab-platform-design.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-UI-DESIGN-005 --roadmap WR-049`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-UI-DESIGN-002` remains completed with accepted design and closeout
  evidence.
- `WR-046` remains support-only evidence for the UI Designer doctrine and
  target-boundary ratification.
- `PM-UI-DESIGN-005` remains `ready_next` and links `WR-049`.
- `WR-049` remains `ready_next`, blocker `B2`, and depends only on completed
  or support-only prerequisite rows.
- The accepted PM-005 theme token design remains accepted and valid.
- No current-candidate WR row conflicts with `WR-049` write scopes.

The expected promotion command is:

```text
task roadmap:promote -- --id WR-049 --state current_candidate --evidence "PM-UI-DESIGN-005 accepted theme token design and production plan establish the bounded generic UI theme token graph implementation contract."
```

After promotion, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, docs validate,
planning validate, and `task ai:goal -- --track PT-UI-DESIGN` before product
code changes.

## Allowed Write Scope

Implementation may touch only:

- `domain/ui/ui_theme` for generic token graph contracts, deterministic
  resolution, provenance, and diagnostics;
- `domain/ui/ui_definition` only for narrow token/theme/state/skin reference
  contracts needed by Canonical UI IR;
- `docs-site/src/content/docs/design/accepted/ui-designer-theme-tokens-modes-skins-and-state-variants-design.md`
  only if implementation proves that the accepted PM-005 design needs a narrow
  clarification;
- `docs-site/src/content/docs/reports/implementation-plans/wr-049-pm-ui-design-005-theme-token-graph-and-mode-resolution/plan.md`
  for this contract;
- roadmap and production-track source/generated files required by promotion,
  validation, and closeout;
- the eventual PM-005 closeout evidence after implementation validation passes.

No app-hosted Theme Designer UI, editor-specific package persistence,
game-runtime package loading, renderer material lowering, component recipe
library, binding, preview matrix, persistence activation, or production
readiness work is in scope for this first row.

## Owning Modules

Implementation should keep generic styling ownership inside the UI theme
domain:

- `domain/ui/ui_theme/src/theme.rs` module `theme`: keep `ThemeTokens` as the
  formed compatibility packet and add token graph entry points only if they
  remain runtime-neutral.
- `domain/ui/ui_theme/src/color.rs`, `spacing.rs`, `radius.rs`, and
  `typography.rs` modules: preserve primitive token value ownership.
- A new `domain/ui/ui_theme/src/token/` subsystem with `mod.rs` is appropriate
  if the implementation needs token ids, graph declarations, aliases,
  selectors, provenance, and diagnostics.
- `domain/ui/ui_theme/src/lib.rs` should expose only focused public types
  needed by normal token graph resolution.
- `domain/ui/ui_definition/src` may add references only if needed to keep
  Canonical UI IR token references typed. It must not form concrete editor or
  runtime theme values.

## Implementation Scope

Add a generic token graph and resolver:

1. Define stable token ids, token families, typed values, alias targets, source
   package ids, target-profile ids, component scopes, state selectors, mode
   selectors, platform selectors, and accessibility selectors.
2. Resolve layers deterministically in the accepted PM-005 order.
3. Preserve provenance for winning and losing sources.
4. Reject alias cycles, missing aliases, token family mismatches, malformed
   values, duplicate selectors, incompatible mode/state combinations,
   accessibility conflicts, unsupported target profiles, and preview-only
   activation attempts.
5. Produce typed diagnostics with source path, target profile, owning domain,
   source package, activation impact, and suggested fix.
6. Keep existing `ThemeTokens` formation usable as the compatibility output for
   current UI formation.
7. Add focused unit tests in `domain/ui/ui_theme`.

## Non-Goals

Do not implement:

- app-hosted Theme Designer or Interface Lab UI;
- editor-specific theme package storage or project file IO;
- game-runtime theme package loading;
- renderer material lowering, shader styles, or GPU resource policy;
- component recipe libraries from `PM-UI-DESIGN-006`;
- view-model capability or intent binding from `PM-UI-DESIGN-007`;
- preview fixture matrices from `PM-UI-DESIGN-008`;
- persistence activation from `PM-UI-DESIGN-009`;
- production readiness from `PM-UI-DESIGN-010`.

## Acceptance Criteria

`WR-049` implementation is complete only when all criteria below are true:

- Generic UI theme token graph contracts exist in `domain/ui/ui_theme`.
- Alias resolution is deterministic and rejects cycles and missing aliases.
- Mode/state/platform/accessibility precedence is deterministic and inspectable.
- Target-profile compatibility failures are blocking diagnostics.
- Preview-only overrides cannot activate.
- Focused tests prove the token graph behavior without app/runtime/editor shell
  ownership leaks.

## Validation

Required validation for this contract-writing action:

```text
task docs:validate
task planning:validate
task ai:goal -- --track PT-UI-DESIGN
```

Required validation after promotion and before any later implementation
closeout:

```text
cargo test -p ui_theme token
cargo test -p ui_theme
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

- `WR-049` cannot be promoted legally;
- `PM-UI-DESIGN-005` is no longer `ready_next` or no longer links `WR-049`;
- the accepted PM-005 design is missing, no longer accepted, or contradicted by
  newer accepted design/ADR evidence;
- implementation would move source truth into app, runtime, editor shell,
  renderer, material, provider, or project layers;
- implementation requires write scopes outside this contract;
- focused tests cannot prove deterministic ordering, alias failures,
  mode/state precedence, accessibility diagnostics, target-profile diagnostics,
  and preview-only activation rejection.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-049-pm-ui-design-005-theme-token-graph-and-mode-resolution/plan.md`;
- changed roadmap path:
  `docs-site/src/content/docs/workspace/roadmap-items.yaml`;
- `task docs:validate` result;
- `task roadmap:validate` result;
- `task roadmap:check` result;
- `task planning:validate` result;
- `task ai:goal -- --track PT-UI-DESIGN` rerun result;
- confirmation that no Rust product code changed for this contract-writing
  action.

Implementation closeout must be created only after product implementation and
focused validation pass.

## Perfectionist Closeout Audit

Expected completion quality for the first `WR-049` implementation is
`bounded_contract`. It can claim `runtime_proven` only if a later explicit
runtime projection and preview path proves user-visible style consumption,
which is outside this first generic token graph slice.

`perfectionist_verified` is not expected for `WR-049` because app-hosted theme
editing, preview matrices, persistence activation, accessibility/performance
evidence, and production readiness remain later milestones.

## Contract-Writing Closeout Evidence

Status as of 2026-05-22: completed for the contract-writing action only.

Changed artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-049-pm-ui-design-005-theme-token-graph-and-mode-resolution/plan.md`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Validation:

- `task docs:validate` passed after the contract was added.
- `task roadmap:validate` passed after the contract path was added to
  `WR-049` write scopes.
- `task roadmap:check` passed after roadmap render refreshed generated
  roadmap docs.
- `task planning:validate` passed after the contract and roadmap metadata were
  updated, covering roadmap validation/check, production validation/check, and
  docs validation.
- `task ai:goal -- --track PT-UI-DESIGN` was rerun after validation and
  reported `PM-UI-DESIGN-005` next legal action as
  `prepare_wr_promotion_contract`, with `WR-049` still `ready_next`.

Closeout result:

- No Rust product code changed.
- No production-track state changed during this contract-writing action.
- Roadmap metadata changed only to include this contract path in `WR-049`
  write scopes. The row remains `ready_next` until explicit promotion.
- The next action after this closeout is to promote `WR-049` only if the
  production-plan preflight still reports it as promotable.

## Promotion And Implementation Readiness Evidence

Status as of 2026-05-22: `WR-049` is promoted to `current_candidate`.

Promotion evidence:

- `task production:plan -- --milestone PM-UI-DESIGN-005 --roadmap WR-049`
  reported promotion preflight status `promotable`.
- `task roadmap:promote -- --id WR-049 --state current_candidate --evidence
  "PM-UI-DESIGN-005 accepted theme token design and production plan establish
  the bounded generic UI theme token graph implementation contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-049-pm-ui-design-005-theme-token-graph-and-mode-resolution/plan.md."`
  promoted `WR-049` to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after promotion.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task docs:validate` and `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-UI-DESIGN` reported `PM-UI-DESIGN-005` next
  legal action as `execute_next_wr_implementation_contract`.
- A final `task production:plan -- --milestone PM-UI-DESIGN-005 --roadmap
  WR-049` rerun reported `WR-049` as `current_candidate` and next action
  `write_implementation_contract`.

Implementation may start only from this contract and must stay inside the
allowed write scope, focused test set, stop conditions, and closeout
requirements above.
