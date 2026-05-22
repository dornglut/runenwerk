---
title: WR-051 PM-UI-DESIGN-007 View-Model Capability And Intent Binding Contracts Plan
description: Promotion and implementation-readiness contract for the bounded PM-UI-DESIGN-007 generic UI binding and intent contract slice.
status: active
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-target-projection-profiles-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
related_adrs:
  - ../../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../../adr/accepted/0004-separate-description-from-execution.md
  - ../../../adr/accepted/0005-projections-are-derived-state.md
  - ../../../adr/accepted/0006-editor-surface-provider-plugin-seam.md
  - ../../../adr/accepted/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-051 PM-UI-DESIGN-007 View-Model Capability And Intent Binding Contracts Plan

## Goal

Promote `PM-UI-DESIGN-007` as one bounded generic UI binding and intent
contract slice.

The slice must add runtime-neutral read-only view-model binding declarations,
capability requirements, validation modes, and UI intent proposal declarations
in `domain/ui/ui_definition`. It must preserve domain-owned command and game
intent authority: UI definitions describe references and proposals, while the
owning editor/workbench or game target adapter validates and executes concrete
commands or intents.

This contract does not implement product code. It records the promotion and
implementation-readiness decisions for `WR-051`.

## Source Of Truth

- Production track: `PT-UI-DESIGN`.
- Production milestone: `PM-UI-DESIGN-007`.
- Bounded implementation row: `WR-051`.
- Accepted PM-007 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md`.
- Accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Accepted target projection design:
  `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md`.
- Accepted visual layout design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-UI-DESIGN-007 --roadmap WR-051`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-UI-DESIGN-003` remains completed with accepted target projection design
  and closeout evidence.
- `PM-UI-DESIGN-004` remains completed with bounded visual layout closeout
  evidence.
- `WR-046` remains support-only evidence for the UI Designer doctrine and
  target-boundary ratification.
- `WR-047` remains completed bounded visual layout evidence.
- `PM-UI-DESIGN-007` remains `ready_next` and links `WR-051`.
- `WR-051` remains `ready_next`, blocker `B2`, and depends only on completed or
  support-only prerequisite rows.
- The accepted PM-007 binding and intent design remains accepted and valid.
- No current-candidate WR row conflicts with `WR-051` write scopes.

The expected promotion command is:

```text
task roadmap:promote -- --id WR-051 --state current_candidate --evidence "PM-UI-DESIGN-007 accepted binding and intent design and production plan establish the bounded generic UI view-model capability and intent binding contract at docs-site/src/content/docs/reports/implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md."
```

After promotion, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, docs validate,
planning validate, and `task ai:goal -- --track PT-UI-DESIGN` before product
code changes.

## Allowed Write Scope

Implementation may touch only:

- `domain/ui/ui_definition` for generic read-only binding declarations,
  binding value types, view-model package references, capability requirements,
  validation modes, intent proposal declarations, target-profile compatibility,
  and diagnostics;
- `docs-site/src/content/docs/design/accepted/ui-designer-view-model-capability-and-intent-binding-design.md`
  only if implementation proves that the accepted PM-007 design needs a narrow
  clarification;
- `docs-site/src/content/docs/reports/implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md`
  for this contract;
- roadmap and production-track source/generated files required by promotion,
  validation, and closeout;
- the eventual PM-007 closeout evidence after implementation validation passes.

No app-hosted binding editor, editor-specific binding package persistence,
game-runtime view-model package loading, concrete command execution, concrete
game intent execution, preview fixture matrix, persistence activation, or
production readiness work is in scope for this first row.

## Owning Modules

Implementation should keep generic binding and intent ownership inside the UI
definition domain:

- `domain/ui/ui_definition/src` module `ui_definition` owns Canonical UI IR and
  should own generic binding and intent declaration contracts unless
  implementation proves a subdomain folder is clearer.
- A new `domain/ui/ui_definition/src/view_binding/` subsystem with `mod.rs` is
  appropriate if implementation needs binding ids, view-model package refs,
  capability requirements, intent declarations, validation, diagnostics, and
  tests.
- `domain/editor/editor_definition` owns editor/workbench-specific view-model
  package declarations and command/menu/shortcut intent adapters. It remains
  out-of-scope for this first generic contract implementation.
- Future game UI target domains own game-specific view-model packages and game
  intent adapters. They remain out-of-scope for this first generic contract
  implementation.
- `apps/runenwerk_editor` remains out-of-scope except as a future consumer.

## Implementation Scope

Add a generic binding and intent contract:

1. Define stable binding ids, intent ids, view-model package ids, field refs,
   value types, capability ids, validation modes, target-profile ids, trigger
   sources, payload binding refs, and diagnostics.
2. Define read-only binding declarations over Canonical UI IR without app,
   runtime, provider, editor shell, or game ownership.
3. Define intent declarations that produce proposals instead of executing
   concrete commands or mutating domain truth.
4. Reject unknown view-model packages, unknown fields, type mismatches, missing
   or stale data, unknown capability ids, denied capabilities, unsupported
   target profiles, unknown command or game intent descriptors, invalid payload
   bindings, command/shortcut/focus conflicts, direct mutation attempts, and
   preview-only activation attempts.
5. Produce typed diagnostics with source location, binding or intent id, target
   profile, trigger source, owning domain, source package, required or denied
   capability ids, activation impact, and suggested fix.
6. Add focused unit tests in `domain/ui/ui_definition`.

## Non-Goals

Do not implement:

- app-hosted binding or intent Designer/Lab UI;
- editor-specific binding package storage or project file IO;
- game-runtime view-model package loading;
- concrete editor command execution or concrete game intent execution;
- renderer material lowering, shader styles, or GPU resource policy;
- live preview fixture matrices from `PM-UI-DESIGN-008`;
- persistence activation from `PM-UI-DESIGN-009`;
- production readiness from `PM-UI-DESIGN-010`.

## Acceptance Criteria

`WR-051` implementation is complete only when all criteria below are true:

- Generic read-only binding and intent proposal contracts exist in
  `domain/ui/ui_definition`.
- Bindings preserve stable authored ids and cannot encode direct mutation.
- Binding value type, missing/stale package, capability denial, and
  target-profile failures produce blocking diagnostics.
- Intent declarations emit proposals and reject command, shortcut, payload, and
  focus conflicts with typed diagnostics.
- Preview, dry-run, and activation validation modes are explicit and fail
  closed.
- Focused tests prove editor/workbench and game-runtime examples without shared
  semantic authority.

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
cargo test -p ui_definition view_binding
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

- `WR-051` cannot be promoted legally;
- `PM-UI-DESIGN-007` is no longer `ready_next` or no longer links `WR-051`;
- the accepted PM-007 design is missing, no longer accepted, or contradicted by
  newer accepted design/ADR evidence;
- implementation would move binding source truth, command execution, game
  intent execution, provider sessions, editor shell state, runtime state, or
  direct domain mutation into generic UI definitions;
- implementation requires write scopes outside this contract;
- focused tests cannot prove read-only binding, proposal-only intents,
  capability denial, target-profile compatibility, conflict diagnostics, and
  preview/dry-run/activation validation behavior.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md`;
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

Expected completion quality for the first `WR-051` implementation is
`bounded_contract`. It can claim `runtime_proven` only if a later explicit
target-profile projection and preview path proves user-visible binding and
intent consumption, which is outside this first generic contract slice.

`perfectionist_verified` is not expected for `WR-051` because app-hosted
binding UI, preview matrices, persistence activation, accessibility/performance
evidence, and production readiness remain later milestones.

## Contract-Writing Closeout Evidence

Status as of 2026-05-22: completed for the contract-writing action only.

Changed artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Validation:

- `task docs:validate` passed after the contract was added.
- `task roadmap:render` passed after the contract path was added to `WR-051`
  write scopes.
- `task roadmap:validate` passed after the contract path was added to
  `WR-051` write scopes.
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
- Roadmap metadata changed only to include this contract path in `WR-051` write
  scopes. The row remained `ready_next` until explicit promotion.
- The next action after this closeout is to rerun
  `task ai:goal -- --track PT-UI-DESIGN`, then promote `WR-051` only if the
  production-plan preflight still reports it as promotable.

## Promotion And Implementation Readiness Evidence

Status as of 2026-05-22: `WR-051` is promoted to `current_candidate`.

Promotion evidence:

- `task production:plan -- --milestone PM-UI-DESIGN-007 --roadmap WR-051`
  reported promotion preflight status `promotable`.
- `task roadmap:promote -- --id WR-051 --state current_candidate --evidence
  "PM-UI-DESIGN-007 accepted binding and intent design and production plan
  establish the bounded generic UI view-model capability and intent binding
  contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-051-pm-ui-design-007-view-model-capability-and-intent-binding-contracts/plan.md."`
  promoted `WR-051` to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after promotion.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task docs:validate` and `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-UI-DESIGN` reported `PM-UI-DESIGN-007` next
  legal action as `execute_next_wr_implementation_contract`.
- A final `task production:plan -- --milestone PM-UI-DESIGN-007 --roadmap
  WR-051` rerun reported `WR-051` as `current_candidate` and next action
  `write_implementation_contract`.

Implementation may start only from this contract and must stay inside the
allowed write scope, focused test set, stop conditions, and closeout
requirements above.
