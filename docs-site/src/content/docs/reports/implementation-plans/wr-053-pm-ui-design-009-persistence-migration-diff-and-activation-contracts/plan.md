---
title: WR-053 PM-UI-DESIGN-009 Persistence Migration Diff And Activation Contracts Plan
description: Promotion and implementation-readiness contract for the bounded PM-UI-DESIGN-009 generic UI definition persistence, migration dry-run, deterministic diff, and activation gate slice.
status: active
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md
  - ../../../design/accepted/ui-designer-canonical-ir-and-composition-design.md
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
  - ../../../design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md
related_adrs:
  - ../../../adr/accepted/0001-use-domain-owned-commands.md
  - ../../../adr/accepted/0004-separate-description-from-execution.md
  - ../../../adr/accepted/0005-projections-are-derived-state.md
  - ../../../adr/superseded/0012-capability-workbench-clean-break.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-053 PM-UI-DESIGN-009 Persistence Migration Diff And Activation Contracts Plan

## Goal

Promote `PM-UI-DESIGN-009` as one bounded generic UI definition persistence,
migration dry-run, deterministic diff, and activation gate contract slice.

The slice must add runtime-neutral persistence declarations, migration requests,
dry-run migration reports, unknown-field preservation policy, deterministic
diff descriptors, activation requests, activation decisions, and blocking
diagnostics in `domain/ui/ui_definition`. It must keep app-hosted project IO,
diff review UI, provider sessions, runtime activation plumbing, renderer
resources, screenshot capture, gameplay mutation, and production-readiness
hardening outside generic UI definition ownership.

This contract does not implement product code. It records the promotion and
implementation-readiness decisions for `WR-053`.

## Source Of Truth

- Production track: `PT-UI-DESIGN`.
- Production milestone: `PM-UI-DESIGN-009`.
- Bounded implementation row: `WR-053`.
- Accepted PM-009 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`.
- Accepted PM-002 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Accepted PM-008 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-live-preview-fixtures-scenarios-and-target-matrix-design.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-UI-DESIGN-009 --roadmap WR-053`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-UI-DESIGN-002` remains completed with accepted Canonical UI IR and
  migration design evidence.
- `PM-UI-DESIGN-008` remains completed with bounded preview fixture closeout
  evidence.
- `WR-046` remains support-only evidence for the UI Designer doctrine and
  target-boundary ratification.
- `WR-052` remains completed prerequisite evidence for preview fixtures and
  target matrix contracts.
- `PM-UI-DESIGN-009` remains `ready_next` and links `WR-053`.
- `WR-053` remains `ready_next`, blocker `B2`, and depends only on completed or
  support-only prerequisite rows.
- The accepted PM-009 persistence, migration, diff, and activation design
  remains accepted and valid.
- No current-candidate WR row conflicts with `WR-053` write scopes.

The expected promotion command is:

```text
task roadmap:promote -- --id WR-053 --state current_candidate --evidence "PM-UI-DESIGN-009 accepted persistence design and production plan establish the bounded generic UI definition persistence, migration dry-run, deterministic diff, and activation gate contract at docs-site/src/content/docs/reports/implementation-plans/wr-053-pm-ui-design-009-persistence-migration-diff-and-activation-contracts/plan.md."
```

After promotion, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, docs validate,
planning validate, and `task ai:goal -- --track PT-UI-DESIGN` before product
code changes.

## Allowed Write Scope

Implementation may touch only:

- `domain/ui/ui_definition` for generic persistence package ids, schema/version
  ids, migration request/report contracts, unknown-field preservation policy,
  deterministic diff descriptors, activation request/decision contracts, and
  diagnostics;
- `docs-site/src/content/docs/design/accepted/ui-designer-persistence-migration-diff-and-activation-design.md`
  only if implementation proves that the accepted PM-009 design needs a narrow
  clarification;
- this implementation plan;
- roadmap and production-track source/generated files required by promotion,
  validation, and closeout;
- the eventual PM-009 closeout evidence after implementation validation passes.

No app-hosted persistence UI, project save/load UI, diff review UI, provider
session orchestration, runtime activation plumbing, renderer resources,
screenshot capture, gameplay mutation, or production readiness work is in scope
for this first row.

## Owning Modules

Implementation should keep generic persistence and activation ownership inside
the UI definition domain:

- `domain/ui/ui_definition/src/migration.rs` module `migration` currently owns
  versioned authored UI template migration and should either be extended or be
  split behind a subdomain module boundary if the first implementation needs
  multiple files.
- A new `domain/ui/ui_definition/src/persistence_activation/` subsystem with
  `mod.rs` is appropriate if the implementation needs persistence package ids,
  migration modes, diff descriptors, activation reports, and focused tests
  without overloading the existing migration file.
- `domain/editor/editor_definition` and future game UI target domains own
  target-specific persisted extension vocabulary and adapters; both remain
  out-of-scope for this first generic contract implementation.
- `apps/runenwerk_editor`, provider, runtime, renderer, and project file IO
  layers remain out-of-scope except as future consumers.

## Implementation Scope

Add a generic persistence and activation contract:

1. Define stable document/package ids, schema version ids, migration modes,
   unknown-field preservation policies, diff ids, activation request ids, and
   activation decision ids.
2. Define migration dry-run requests and reports that never mutate the active
   definition set.
3. Define deterministic diff descriptors that can report changed paths,
   before/after schema versions, changed values when display-safe, and
   non-deterministic diff diagnostics.
4. Define activation requests and decisions that require successful migration
   and diff evidence before activation.
5. Reject unsupported schema versions, incompatible migrations, unknown
   required fields, unpreservable unknown fields, non-deterministic
   serialization or diff output, missing migration reports, missing diff
   descriptors, target-profile incompatibility, expected diagnostic mismatch,
   and preview-only activation.
6. Add focused unit tests in `domain/ui/ui_definition`.

## Non-Goals

Do not implement:

- app-hosted persistence UI, project save/load UI, or diff review UI;
- concrete project file IO in `apps/runenwerk_editor`;
- runtime activation plumbing;
- provider session orchestration;
- screenshot capture, visual regression, or renderer golden evidence;
- accessibility/performance reporting from `PM-UI-DESIGN-010`;
- gameplay mutation, concrete editor command execution, or project storage
  ownership inside generic UI definition contracts.

## Acceptance Criteria

`WR-053` implementation is complete only when all criteria below are true:

- Generic persistence, migration dry-run, deterministic diff, activation
  request, activation decision, and diagnostic contracts exist in
  `domain/ui/ui_definition`.
- Current-version authored UI definitions can pass migration without
  diagnostics.
- Unsupported schema versions and incompatible migrations block activation.
- Compatible unknown fields are preserved where policy allows, and
  unpreservable unknown fields produce blocking diagnostics.
- Non-deterministic serialization, missing diff descriptors, preview-only
  activation, and expected diagnostic mismatches fail closed.
- Focused tests prove editor/workbench and game-runtime examples without shared
  project IO, provider, runtime, renderer, screenshot, or gameplay ownership.

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
cargo test -p ui_definition migration
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

- `WR-053` cannot be promoted legally;
- `PM-UI-DESIGN-009` is no longer `ready_next` or no longer links `WR-053`;
- the accepted PM-009 design is missing, no longer accepted, or contradicted by
  newer accepted design/ADR evidence;
- implementation would move project IO, provider sessions, runtime state,
  renderer handles, screenshots, gameplay truth, concrete editor command
  execution, or direct mutation into generic UI persistence contracts;
- implementation requires write scopes outside this contract;
- focused tests cannot prove schema/version gates, dry-run migration reports,
  unknown-field preservation policy, deterministic diff descriptors, activation
  decisions, target-profile diagnostics, expected diagnostic checks, and
  preview-only activation behavior.

## Closeout Requirements

Closeout for this contract-writing action records this plan path, changed
roadmap path, validation results, the `task ai:goal -- --track PT-UI-DESIGN`
rerun, and confirmation that no Rust product code changed.

Implementation closeout must be created only after product implementation and
focused validation pass.

## Perfectionist Closeout Audit

Expected completion quality for the first `WR-053` implementation is
`bounded_contract`. It can claim `runtime_proven` only if a later explicit
activation path proves user-visible project IO, runtime consumption, and
activation report usage, which is outside this first generic contract slice.

`perfectionist_verified` is not expected for `WR-053` because app-hosted
persistence UI, project save/load, user-facing diff review, runtime activation
plumbing, accessibility/performance evidence, and production readiness remain
later milestones.

## Contract-Writing Closeout Evidence

Status as of 2026-05-22: completed for the contract-writing action only.

Changed artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-053-pm-ui-design-009-persistence-migration-diff-and-activation-contracts/plan.md`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Validation:

- `task docs:validate` passed after the contract was added.
- `task roadmap:render` passed after the contract path was added to `WR-053`
  write scopes.
- `task roadmap:validate` passed after the contract path was added to
  `WR-053` write scopes.
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
- Roadmap metadata changed only to include this contract path in `WR-053` write
  scopes. The row remained `ready_next` until explicit promotion.
- The next action after this closeout is to rerun
  `task ai:goal -- --track PT-UI-DESIGN`, then promote `WR-053` only if the
  production-plan preflight still reports it as promotable.

## Promotion And Implementation Readiness Evidence

Status as of 2026-05-22: `WR-053` is promoted to `current_candidate`.

Promotion evidence:

- `task production:plan -- --milestone PM-UI-DESIGN-009 --roadmap WR-053`
  reported promotion preflight status `promotable`.
- `task roadmap:promote -- --id WR-053 --state current_candidate --evidence
  "PM-UI-DESIGN-009 accepted persistence design and production plan establish
  the bounded generic UI definition persistence, migration dry-run,
  deterministic diff, and activation gate contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-053-pm-ui-design-009-persistence-migration-diff-and-activation-contracts/plan.md."`
  promoted `WR-053` to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after promotion.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task docs:validate` and `task planning:validate` passed after promotion.

Implementation may start only from this contract and must stay inside the
allowed write scope, focused test set, stop conditions, and closeout
requirements above.
