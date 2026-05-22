---
title: WR-047 PM-UI-DESIGN-004 Visual Layout Core Editing Plan
description: Promotion and implementation-readiness contract for the bounded PM-UI-DESIGN-004 visual layout core edit operation slice.
status: active
owner: editor
layer: domain/ui-definition
canonical: false
last_reviewed: 2026-05-22
related_designs:
  - ../../../design/accepted/ui-designer-visual-layout-and-interface-composition-design.md
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

# WR-047 PM-UI-DESIGN-004 Visual Layout Core Editing Plan

## Goal

Promote and implement `PM-UI-DESIGN-004` as one bounded definition-layer
visual layout edit operation slice.

The slice must let typed UI definitions express layout edits through stable
authored ids, deterministic textual diffs, preview-only activation rejection,
and source-mapped layout/composition diagnostics. It must preserve the rule
that Designer documents are source truth only for UI/interface definitions,
while editor/workbench and game-runtime projections remain derived state.

This contract does not implement product code. It records the promotion and
implementation-readiness decisions for `WR-047`.

## Source Of Truth

- Production track: `PT-UI-DESIGN`.
- Production milestone: `PM-UI-DESIGN-004`.
- Bounded implementation row: `WR-047`.
- Accepted PM-004 design:
  `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`.
- Accepted Canonical UI IR design:
  `docs-site/src/content/docs/design/accepted/ui-designer-canonical-ir-and-composition-design.md`.
- Accepted target projection design:
  `docs-site/src/content/docs/design/accepted/ui-designer-target-projection-profiles-design.md`.
- Active platform doctrine:
  `docs-site/src/content/docs/design/active/ui-designer-interface-lab-platform-design.md`.
- Accepted ADRs:
  `docs-site/src/content/docs/adr/accepted/0004-separate-description-from-execution.md`
  and
  `docs-site/src/content/docs/adr/accepted/0005-projections-are-derived-state.md`.

## Promotion Readiness

`task production:plan -- --milestone PM-UI-DESIGN-004 --roadmap WR-047`
classified the next action as `write_promotion_contract` and reported roadmap
promotion preflight status `promotable`.

Promotion is honest only if all of these remain true immediately before
promotion:

- `PM-UI-DESIGN-002` and `PM-UI-DESIGN-003` remain completed with accepted
  design and closeout evidence.
- `WR-046` remains support-only evidence for the UI Designer doctrine and
  target-boundary ratification.
- `PM-UI-DESIGN-004` remains `ready_next` and links `WR-047`.
- `WR-047` remains `ready_next`, blocker `B2`, and depends only on completed
  or support-only prerequisite rows.
- The accepted PM-004 visual layout design remains accepted and valid.
- No current-candidate WR row conflicts with `WR-047` write scopes.

The expected promotion command is:

```text
task roadmap:promote -- --id WR-047 --state current_candidate --evidence "PM-UI-DESIGN-004 accepted visual layout design and production plan establish the bounded definition-layer visual layout core edit implementation contract."
```

After promotion, run roadmap render, roadmap validate, roadmap check,
production render, production validate, production check, docs validate,
planning validate, and `task ai:goal -- --track PT-UI-DESIGN` before product
code changes.

## Allowed Write Scope

Implementation may touch only:

- `domain/ui/ui_definition` for generic UI definition, layout edit operation,
  deterministic diff, source-map, and diagnostic contracts;
- `docs-site/src/content/docs/design/accepted/ui-designer-visual-layout-and-interface-composition-design.md`
  only if implementation proves that the accepted PM-004 design needs a narrow
  clarification;
- `docs-site/src/content/docs/reports/implementation-plans/wr-047-pm-ui-design-004-visual-layout-core-editing/plan.md`
  for this contract;
- roadmap and production-track source/generated files required by promotion,
  validation, and closeout;
- the eventual PM-004 closeout evidence after implementation validation passes.

No app-hosted Designer/Lab UI, editor-specific visual shell, game-runtime
projection runtime, theme token system, component recipe library, binding
system, preview matrix, persistence activation flow, or production-readiness
hardening is in scope for this first row.

## Owning Modules

The implementation should stay in the UI definition domain:

- `domain/ui/ui_definition/src/node.rs` module `node`: reuse
  `UiNodeDefinition`, `UiNodeChildren`, `UiLayoutDefinition`,
  `UiStackLayoutDefinition`, and `UiSplitLayoutDefinition` as the typed layout
  definition surface. Extend only if the visual edit contract needs explicit
  layout metadata that belongs to canonical UI definitions.
- `domain/ui/ui_definition/src/identity.rs` module `identity`: reuse stable
  `UiDefinitionId` and related definition identity types. Add new operation or
  source-map ids here only if they follow existing identity validation and are
  part of the UI definition contract.
- `domain/ui/ui_definition/src/template.rs` module `template`: keep template
  reference replacement edits typed through existing template identifiers.
- `domain/ui/ui_definition/src/form.rs` module `form`: keep form-specific
  semantics outside generic layout edits unless tests prove an edit operation
  needs to preserve existing form node contracts.
- `domain/ui/ui_definition/src/lib.rs` and `domain/ui/ui_definition/src/mod.rs`
  module exports: expose only the focused public types needed by normal
  visual layout editing.

If a larger subsystem is needed, use a subdomain folder with a `mod.rs`
boundary under `domain/ui/ui_definition/src`, for example
`visual_layout/`. Do not introduce catch-all `utils`, `helpers`, or `_internal`
modules.

## Implementation Scope

Add a definition-owned visual layout edit path:

1. Define a typed edit operation contract with operation id, source document id,
   target authored node path, expected stable authored id, target profile, edit
   kind, structural preconditions, replacement value or patch, source location,
   and preview-only flag.
2. Cover the first bounded edit kinds: insert node, remove node, move node,
   reorder sibling, change stack axis, change split ratio, wrap selection in a
   container, unwrap container, and replace template reference.
3. Preserve stable authored ids for move, reorder, wrap, unwrap, and edit
   operations. Allocate new ids only for newly created authored nodes.
4. Produce a deterministic textual diff contract with canonical field ordering,
   stable child ordering, stable id formatting, source-map paths, and explicit
   before/after values.
5. Reject activation for preview-only edits and for edits that cannot produce a
   deterministic textual patch.
6. Emit typed layout/composition diagnostics with stable code, severity, source
   location, affected target profile, affected host/suite/surface when known,
   owning domain, edit operation id, activation impact, and suggested fix.
7. Keep target-profile compatibility validation data-oriented and
   definition-owned. Do not make editor app state, runtime widget ids, ECS
   entity ids, provider handles, renderer handles, or active sessions part of
   Canonical UI IR.
8. Add focused unit tests in `domain/ui/ui_definition` for stable ids,
   deterministic diff output, preview-only rejection, source-mapped
   diagnostics, and target-profile compatibility failures.

## Non-Goals

Do not implement:

- app-hosted visual Designer/Lab UI;
- drag handles, canvas interaction, selection chrome, or editor panels;
- game-runtime UI projection execution;
- theme/token resolution from `PM-UI-DESIGN-005`;
- component, surface, or widget recipe libraries from `PM-UI-DESIGN-006`;
- view-model capability or intent binding from `PM-UI-DESIGN-007`;
- live preview fixture/scenario matrices from `PM-UI-DESIGN-008`;
- persistence migration, activation pipelines, or repository file IO from
  `PM-UI-DESIGN-009`;
- production readiness, accessibility/performance golden evidence, or visual
  regression harnesses from `PM-UI-DESIGN-010`.

## Acceptance Criteria

`WR-047` implementation is complete only when all criteria below are true:

- Core visual layout edit operations are represented by explicit typed domain
  contracts in `domain/ui/ui_definition`.
- Move, reorder, wrap, unwrap, and layout property edits preserve stable ids.
- Edits that can activate produce deterministic textual diff evidence.
- Preview-only edits cannot activate or persist through the edit contract.
- Invalid layout edits produce source-mapped diagnostics with activation impact
  and target-profile context.
- Tests prove target-profile compatibility diagnostics without moving target
  projection ownership into app or runtime layers.
- Public exports make the normal visual layout edit workflow discoverable
  without exposing unrelated advanced internals.

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
cargo test -p ui_definition visual_layout
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

- `WR-047` cannot be promoted legally;
- `PM-UI-DESIGN-004` is no longer `ready_next` or no longer links `WR-047`;
- the accepted PM-004 design is missing, no longer accepted, or contradicted by
  newer accepted design/ADR evidence;
- a new ADR is required because implementation would change source truth,
  dependency direction, target-profile ownership, runtime activation authority,
  or direct domain mutation rules;
- implementation requires write scopes outside this contract;
- implementation needs app-hosted visual UI, runtime projection execution,
  persistence activation, or any downstream PM-005 through PM-010 capability;
- product code cannot define focused stable-id, deterministic-diff,
  preview-only, source-map diagnostic, and target-profile compatibility tests.

## Closeout Requirements

Closeout for this contract-writing action records:

- changed contract path:
  `docs-site/src/content/docs/reports/implementation-plans/wr-047-pm-ui-design-004-visual-layout-core-editing/plan.md`;
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
focused validation pass. The closeout must then update:

- `docs-site/src/content/docs/reports/closeouts/pm-ui-design-004-visual-layout-and-interface-composition/closeout.md`;
- `docs-site/src/content/docs/workspace/roadmap-items.yaml` or
  `docs-site/src/content/docs/workspace/roadmap-archive.yaml`, depending on
  the final WR state;
- `docs-site/src/content/docs/workspace/production-tracks.yaml`;
- generated roadmap and production docs.

## Perfectionist Closeout Audit

Expected completion quality for the first `WR-047` implementation is
`bounded_contract`. It can claim `runtime_proven` only if a later explicit
runtime projection and preview path proves user-visible runtime consumption,
which is outside this first definition-layer slice.

`perfectionist_verified` is not expected for `WR-047` because app-hosted visual
editing, preview matrices, persistence activation, accessibility/performance
evidence, and production readiness remain later milestones. Those known gaps
must stay visible in the roadmap or production closeout evidence.

The anti-drift guard for this row is focused test evidence that prevents:

- descriptor-only edit types with no deterministic diff path;
- preview-only edits activating successfully;
- stable ids changing during move/reorder operations;
- source-map diagnostics losing target-profile or activation-impact context;
- app, runtime, provider, renderer, or session identifiers becoming Canonical
  UI IR source truth.

## Contract-Writing Closeout Evidence

Status as of 2026-05-22: completed for the contract-writing action only.

Changed artifacts:

- `docs-site/src/content/docs/reports/implementation-plans/wr-047-pm-ui-design-004-visual-layout-core-editing/plan.md`
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`

Validation:

- `task docs:validate` passed after the contract was added.
- `task roadmap:validate` passed after the contract path was added to
  `WR-047` write scopes.
- `task roadmap:check` passed after roadmap render refreshed generated
  roadmap docs.
- `task planning:validate` passed after the contract and roadmap metadata were
  updated, covering roadmap validation/check, production validation/check, and
  docs validation.
- `task ai:goal -- --track PT-UI-DESIGN` was rerun after validation and
  reported `PM-UI-DESIGN-004` next legal action as
  `prepare_wr_promotion_contract`, with `WR-047` still `ready_next`.

Closeout result:

- No Rust product code changed.
- No production-track state changed during this contract-writing action.
- Roadmap metadata changed only to include this contract path in `WR-047`
  write scopes. The row remains `ready_next` until explicit promotion.
- The next action after this closeout is to promote `WR-047` only if the
  production-plan preflight still reports it as promotable.

## Promotion And Implementation Readiness Evidence

Status as of 2026-05-22: `WR-047` is promoted to `current_candidate`.

Promotion evidence:

- `task production:plan -- --milestone PM-UI-DESIGN-004 --roadmap WR-047`
  reported promotion preflight status `promotable`.
- `task roadmap:promote -- --id WR-047 --state current_candidate --evidence
  "PM-UI-DESIGN-004 accepted visual layout design and production plan
  establish the bounded definition-layer visual layout core edit
  implementation contract at
  docs-site/src/content/docs/reports/implementation-plans/wr-047-pm-ui-design-004-visual-layout-core-editing/plan.md."`
  promoted `WR-047` to `current_candidate`.
- `task roadmap:render`, `task roadmap:validate`, and `task roadmap:check`
  passed after promotion.
- `task production:render`, `task production:validate`, and
  `task production:check` passed after promotion.
- `task docs:validate` and `task planning:validate` passed after promotion.
- `task ai:goal -- --track PT-UI-DESIGN` reported `PM-UI-DESIGN-004` next
  legal action as `execute_next_wr_implementation_contract`.
- A final `task production:plan -- --milestone PM-UI-DESIGN-004 --roadmap
  WR-047` rerun reported `WR-047` as `current_candidate` and next action
  `write_implementation_contract`.

Implementation may start only from this contract and must stay inside the
allowed write scope, focused test set, stop conditions, and closeout
requirements above.
