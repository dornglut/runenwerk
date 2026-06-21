---
title: UI Composition Core Contracts And Invariants Implementation Plan
description: Decision-complete WR-181 contract for the app-neutral composition core, structural transactions, history, diagnostics, liveness, promotion primitives, and conformance fixtures.
status: accepted
owner: ui
layer: report
canonical: false
last_reviewed: 2026-06-19
wr: WR-181
milestone: PM-UI-COMPOSITION-002
related_designs:
  - ../../../design/accepted/app-neutral-ui-composition-design.md
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
related_roadmaps:
  - ../../../workspace/production-tracks.yaml
  - ../../../workspace/roadmap-items.yaml
---

# WR-181: UI Composition Core Contracts And Invariants

## Authority And Promotion

Checkpoint 1 is complete and Option 2, Region Compass, is selected. WR-181 is
promotable from `ready_next` to `current_candidate`; implementation may start
only after that promotion and the `crate_creation` intent lock are recorded.

This checkpoint creates `domain/ui/ui_composition` as the app-neutral
structural authority. It changes no editor, Draw, engine, windowing, chrome,
docking, adaptive projection, persistence storage, or legacy authority.

### Execution handoff amendment

On 2026-06-19 the isolated `agent_writer` backend stopped before changing files
because its external Codex quota was exhausted. The primary governed branch
completed the exact accepted implementation scope in-process. The contract now
uses `verification_writer` so the Track Execution Harness performs the declared
validation and resolver-backed evidence phases without replaying product edits.
The failed run ledger remains preserved as execution history; scope,
permissions, tests, evidence requirements, rollback policy, stop conditions,
and closeout quality are unchanged.

## Current Code Truth And Reuse Inventory

- `foundation/id::TypedId` is the reusable non-zero typed identity primitive.
  The new crate wraps it in domain-named durable ID types and enables its serde
  feature; no second allocator or weak string identity is added for structural
  IDs.
- `foundation/ratification` supplies generic ratification reports and severity.
  Composition owns its issue codes, stages, subjects, acceptance rules, and
  mapping into `foundation/diagnostics`.
- `foundation/diagnostics` supplies portable diagnostic code, domain, subject,
  severity, message, notes, and metadata contracts. Composition adds the
  mandatory stage and deterministic ordering at its boundary.
- Existing UI crates use `BTreeMap` and `BTreeSet` where deterministic order is
  part of public behavior. Composition follows that pattern.
- `domain/ui/ui_testing` is the existing UI conformance utility owner. It will
  host executable composition fixtures but no browser, terminal, dashboard,
  mobile, or game product behavior.
- `domain/ui/ui_surface` and editor workspace state are read-only reference
  material for later replacement mapping. WR-181 neither imports nor edits
  them.
- `domain/ui/ui-crate-ownership.toml` and
  `tools/checks/check_ui_layer_dependencies.py` are the dependency fitness
  function. `ui_composition` is classified in a new `composition` layer with
  no production dependency on any other UI crate; `ui_testing` may consume it.

## Exact Module Ownership

`domain/ui/ui_composition` uses responsibility modules with `mod.rs`
boundaries:

| Module | Ownership |
|---|---|
| `identity` | Typed structural IDs, revision IDs, and validated namespaced semantic/profile references. |
| `definition` | `CompositionDefinitionV1`, targets, roots, regions, mounted units, closed region algebra, fixed-point split fraction. |
| `content` | Opaque `MountedContentRef`, unavailable-content policy, and neutral liveness observations. |
| `diagnostic` | `ui_composition.*` codes, stages, subjects, stable records, ordering, and foundation-diagnostic conversion. |
| `validation` | Definition/graph invariants and transaction candidate validation. It mutates nothing. |
| `state` | Formation of ratified `CompositionState`, immutable snapshots, revision ownership, and state-to-definition projection. |
| `transaction` | Typed commands, narrow policy ports, authorization, atomic candidate application, rejection, and commit result. |
| `history` | Ordered journal plus structural undo/redo stacks; no product-content history. |
| `promotion` | Core-only `LayoutPromotion` candidate derived from ratified state. Persistence and app-extension snapshots remain WR-182. |
| `fixture` | App-neutral conformance fixture declaration and expectation vocabulary. |

`lib.rs` re-exports the normal formation, snapshot, transaction, history,
promotion, fixture, and diagnostic workflows. Internal graph rewrite helpers
remain private.

## Identity Decisions

Structural identities are distinct wrappers over non-zero `TypedId` values:

- `CompositionDefinitionId`
- `PresentationTargetId`
- `CompositionRootId`
- `RegionId`
- `MountedUnitId`
- `CompositionTransactionId`
- `CompositionFixtureId`

`DefinitionRevision` and `StateRevision` are distinct non-zero monotonic value
types. Transaction IDs do not establish ordering; journal ordering uses applied
revision and then transaction ID.

Extensible semantic references are validated namespaced strings with private
storage and fallible constructors:

- `HostProfileId`
- `TargetProfileId`
- `RegionProfileId`
- `ContentOwnerId`
- `ContentProfileId`
- `ContentInstanceRef`
- `CapabilityId`
- `AdaptiveProposalExpectationId`

They accept ASCII alphanumeric segments separated by `.`, with `_` and `-`
inside segments. Empty, whitespace, unnamespaced, or unsupported characters
reject. Display labels are separate optional metadata and never participate in
identity, equality, ordering, or validation.

## Definition And Content Contract

`CompositionDefinitionV1` is an immutable candidate containing schema version
1, definition ID, definition revision, sorted target/root/region/mounted-unit
records, and capability/profile references. Constructors preserve supplied
records so invalid fixtures can be represented; only formation produces
ratified state.

The closed `RegionKind` algebra is exactly:

- `Split { axis, fraction, first, second }`
- `Stack { ordered_units, active_unit }`
- `Overlay { base, ordered_overlays }`
- `MountPoint { mounted_unit }`

`SplitFraction` stores integer basis points and accepts only `1..=9999`.
Stack and overlay order are explicit vectors whose uniqueness is validated.

`MountedContentRef` has private `ContentOwnerId`, `ContentProfileId`, and
`ContentInstanceRef` fields with typed accessors. It implements no arbitrary
payload escape hatch and has no `UiProgram` dependency. `MountedUnitDefinition`
adds sorted capability references and `UnavailableContentPolicy`.

`ContentLiveness` is exactly `Resolved`, `Missing`, `Loading`, `Suspended`,
`Denied`, `UnsupportedProfile`, or `Crashed`. Liveness observations are kept
outside canonical structure and never make a valid composition invalid. The
fixture suite covers all seven states and confirms hide is legal only when the
mounted-unit profile explicitly permits it; projection fallback selection is
implemented by later consumer checkpoints.

## Formation And Invariants

`CompositionState::form(definition)` validates without mutating the candidate
and returns either ratified state or `CompositionRejection` with one or more
sorted diagnostics. Formation proves:

1. every structural and semantic ID is valid and unique in its namespace;
2. every reference resolves to the expected record kind;
3. every target has exactly one primary root and every root names one target;
4. region graphs are acyclic;
5. every non-root region has exactly one structural parent and every root
   region has exactly one root owner;
6. every mounted unit occurs in exactly one `Stack` or `MountPoint` location;
7. stack unit order is unique and non-empty, and `active_unit` belongs to it;
8. overlay region order is unique and does not include the base;
9. split children differ and the fraction is valid;
10. capability/profile reference sets are duplicate-free and deterministically
    ordered.

Empty compositions are rejected: at least one target, primary root, reachable
region, and mounted unit must exist. Unreachable records reject rather than
being silently discarded.

`CompositionSnapshot` is a read-only view exposing revision and ordered lookup
APIs. Callers cannot mutate internal maps or bypass transaction validation.

## Transaction And Policy Contract

`CompositionTransaction` contains a typed ID, expected state revision, and a
non-empty ordered command list. `CompositionCommand` covers:

- mount and unmount unit;
- activate unit;
- move unit and reorder stack;
- split and resize region;
- merge split;
- create, move, and close root;
- attach and detach presentation target.

Each command carries complete typed operands. Index-only or label-based
addressing is forbidden. Commands that could discard non-empty structure must
name an explicit legal destination or reject.

The core defines three narrow policy ports, not an app-host trait:

- `CompositionLifecyclePolicy`
- `CompositionCapabilityPolicy`
- `CompositionTargetPolicy`

Each evaluates the complete transaction against an immutable snapshot and
returns a typed acceptance or stable diagnostic rejection. Authorization
consumes the transaction and returns an opaque `AuthorizedTransaction`; callers
cannot alter commands after policy acceptance.

`CompositionState::apply_authorized` performs:

1. expected-revision and authorization-revision checks;
2. command application to an isolated candidate state;
3. complete invariant revalidation;
4. inverse-command derivation;
5. one atomic state/revision/journal swap on success.

Any failure leaves graph, revision, journal, undo stack, and redo stack byte-for-
byte equivalent to their pre-call values. Every rejection carries at least one
stable diagnostic. Duplicate transaction IDs and empty transactions reject.

## Structural History

The journal stores committed forward commands, derived inverse commands,
transaction ID, base revision, and applied revision. Undo/redo is LIFO and
structural only.

`undo(new_transaction_id, policies)` and
`redo(new_transaction_id, policies)` submit inverse or forward commands as new
transactions against the current revision, pass through all three policy ports,
and revalidate the complete resulting graph. Failed authorization, stale
revision, or invariant conflict changes neither state nor history stacks.

History never owns document editing, browser navigation, drawing strokes,
graph edits, terminal commands, game state, or provider/session state.

## Diagnostics

Every rejection path emits `CompositionDiagnosticRecord` with:

- code under `ui_composition.*`;
- severity;
- stage (`identity`, `formation`, `policy`, `transaction`, `history`,
  `promotion`, or `fixture`);
- typed subject kind and canonical subject ID;
- actionable message;
- sorted context entries.

Records sort by stage rank, severity rank, code, subject kind, and subject ID.
Tests lock code spelling and prove insertion-order-independent report order.

## Promotion Primitive

`CompositionState::promote_definition(new_definition_id)` produces a
`LayoutPromotion` containing source state revision and a normalized candidate
`CompositionDefinitionV1`. It excludes liveness observations and any adaptive
state. It does not write files, hash content, activate generations, or collect
app extensions; WR-182 owns those atomic bundle operations.

## Fixture Contract

`CompositionFixture` declares fixture ID, host profile, target profiles,
definition, expected mounted content references, expected validity, expected
capabilities, expected diagnostics, expected adaptive proposal IDs, and
forbidden imports/product behaviors.

`ui_testing::composition_fixture` provides five executable fixture constructors:
browser, terminal, dashboard, mobile, and game. They run core formation and
expectation checks only. They do not implement browsing, terminal execution,
dashboard data, mobile navigation, gameplay, providers, sessions, native
windows, rendering, or adaptive algorithms.

## Implementation Sequence

1. Add exact workspace membership/dependency entries and classify the crate in
   `ui-crate-ownership.toml`.
2. Implement identities, semantic references, definition records, content
   liveness, and public module surfaces.
3. Implement deterministic diagnostic records and validation reports.
4. Implement formation and immutable snapshots with full graph invariants.
5. Implement typed commands, policy authorization, isolated candidate apply,
   atomic commit, and inverse derivation.
6. Implement structural journal, undo/redo revalidation, and promotion
   projection.
7. Add the fixture contract and five `ui_testing` conformance fixtures.
8. Update UI ownership/dependency/architecture docs and focused usage guidance.
9. Run focused, property, dependency, docs, roadmap, and production validation;
   write resolver-backed runtime-test, fixture, and diagnostics evidence.

Each step must leave the crate compiling and focused tests green. No placeholder
module, `todo!`, panic-based normal error path, compatibility alias, or
success-shaped stub may remain.

## Tests And Acceptance

Focused tests must cover:

- every formation invariant with a stable diagnostic code;
- acyclic and cyclic graphs, duplicate/unreachable IDs, parent counts, stack
  activation, overlay order, split bounds, and unit location counts;
- stale revision, duplicate transaction ID, empty transaction, policy veto,
  invalid command operands, multi-command rollback, and successful commit;
- every command family and inverse derivation;
- undo/redo reauthorization, current-revision revalidation, conflict rejection,
  redo invalidation after a new commit, and unchanged state on failure;
- all content liveness states and hide-on-unavailable eligibility;
- normalized promotion excluding liveness/adaptive state;
- all five fixtures, expected diagnostics, and forbidden-product declarations;
- property tests over generated valid trees and one-fault mutations proving
  formation determinism, atomic rejection, and invariant preservation.

Required commands:

```text
cargo fmt --all --check
cargo test -p ui_composition
cargo test -p ui_testing composition_fixture
task ui:dependencies
task production:validate
task roadmap:validate
task docs:validate
task planning:validate
```

## Non-Goals

- canonical RON writers, BLAKE3 hashes, extension envelopes, generation
  activation, or storage adapters (WR-182);
- adaptive proposals, preview/reflow state, drag/resize sessions, hit testing,
  or Region Compass UI (WR-185 and later);
- editor, Draw, game, provider, session, native-window, engine, renderer, or OS
  integration;
- mutation or deletion of legacy workspace or `ui_surface` code;
- collaborative/network transactions, CRDT/OT, or extraction from `domain/ui`.

## Rollback And Stop Conditions

The checkpoint is additive and has no runtime consumers. If formation,
transaction, history, diagnostics, fixture, dependency, or governance checks
fail, reject the checkpoint and leave current runtime consumers unchanged.

Stop rather than broadening scope if app semantics enter the core, a command
cannot preserve atomicity, policy acceptance can be bypassed, a fixture needs
product behavior, foundation changes appear necessary, or exact evidence cannot
be produced.

## Closeout

Close only as `runtime_proven` after focused tests, property tests, dependency
checks, fixture evidence, diagnostics evidence, docs, roadmap, and production
validation pass. The closeout must list persistence, adaptive, editor, Draw,
cleanup, accessibility, performance, and final truth work as owned downstream
scope, not as defects hidden by this checkpoint.
