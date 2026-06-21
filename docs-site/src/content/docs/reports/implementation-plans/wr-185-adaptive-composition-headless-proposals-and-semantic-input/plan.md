---
title: Adaptive Composition Headless Proposals And Semantic Input Implementation Plan
description: Decision-complete WR-185 contract for derived adaptive projection, proposal-only interaction, semantic input, conformance fixtures, accessibility metadata, and measurable performance.
status: accepted
owner: ui
layer: report
canonical: false
last_reviewed: 2026-06-20
wr: WR-185
milestone: PM-UI-COMPOSITION-006
related_designs:
  - ../../../design/accepted/adaptive-ui-composition-design.md
  - ../../../design/accepted/app-neutral-ui-composition-design.md
  - ../../design/ui-composition-visual-direction/selection.md
related_adrs:
  - ../../../adr/accepted/0013-app-neutral-ui-composition-clean-cutover.md
---

# WR-185: Adaptive Composition Headless Proposals And Semantic Input

## Authority

WR-184 is runtime-proven and Region Compass is selected. WR-185 may implement
headless mechanism only. Editor/Draw chrome, app providers, native windows,
OS lifecycle, and structural commit execution remain forbidden until later
checkpoints.

`ui_adaptive_composition` owns derived projection, constraints, hit testing,
snap/dock targets, previews, interaction sessions, edit classification, and
promotion deltas. It reads `CompositionSnapshot` and emits proposals. It never
holds `&mut CompositionState`, calls `apply_transaction`, runs app commands, or
persists adaptive state.

Dependency direction:

```text
ui_composition + ui_input + ui_math
  <- ui_adaptive_composition
  <- ui_testing headless conformance fixtures
```

ADR 0013 and the accepted designs cover the decision. Stop for a new ADR if
implementation requires commit authority, app semantics, native-window state,
or a dependency into editor/engine/apps.

## State And Edit Classification

`AdaptiveProjectionState` records source definition/state revisions, immutable
projected region/index storage, target constraints, drawers/overflow, focus and
accessibility projection, and sorted diagnostics. It is transient and never
serializes as canonical layout.

Every edit is exactly one of:

- `TransientAdaptive`: hover, drawer, compact substitution, projected bounds,
  focus preview, or active pointer/session state;
- `StructuralTransaction`: a proposal carrying expected revision and ordered
  typed structural intent for host policy to materialize into ordered
  `CompositionCommand` values and submit;
- `PromotionCandidate`: an explicit validated `AdaptivePromotionDelta` for a
  separately named/saved layout.

Saving canonical state excludes transient projection. Promotion is explicit;
adaptive state never silently overwrites the source definition.

## Module Boundaries

Create `domain/ui/ui_adaptive_composition` with subdomains:

- `projection/`: constraints, immutable projected graph, drawers/overflow;
- `interaction/`: hit index, drag/resize sessions, previews, cancel/rollback;
- `proposal/`: typed dock/snap/reflow proposals and edit classification;
- `promotion/`: validated promotion delta only;
- `accessibility/`: labels, focus order, text scale, contrast, reduced motion,
  touch target and controller-parity metadata;
- `diagnostic/`: `ui_adaptive_composition.*` code/severity/stage/subject/message;
- `fixture/`: deterministic 128/64/4 and 2048/1024/16 benchmark layouts and
  clone/allocation probes.

`ui_input/src/semantic.rs` adds app-neutral semantic actions: focus direction,
activate, cancel, tab cycle, enter move mode, enter resize mode, commit, and
rollback. Sources are pointer, keyboard, touch, or abstract controller. No raw
gamepad buttons/axes enter adaptive code.

`ui_testing/src/adaptive_composition_fixture.rs` runs browser, terminal,
dashboard, mobile, and game structural fixtures. Fixtures declare expected
adaptive proposals and forbidden imports/behaviors; they implement no product
providers or behavior.

## Interaction And Accessibility Invariants

Sessions retain a shared immutable base projection plus bounded changed-region
preview state. Pointer updates query a prebuilt hit index and may not clone the
full composition or region graph. Cancel/Escape/rollback restores the exact
session baseline and emits no structural proposal. Commit emits a proposal but
does not mutate.

Headless tests prove keyboard-only movement/resizing, deterministic focus and
visible-focus metadata, high contrast, text scaling, reduced-motion zero
duration, minimum touch targets, controller semantic parity, cancel behavior,
and inspection labels for targets/previews/drawers/unavailable content.

## Performance And Proof

Benchmarks use the accepted normal, large, and 64-command layouts with warm-up
and at least 30 measured samples. Separate runners measure region hit testing,
proposal generation, preview projection, complete drag-frame update,
transaction validation, committed mutation, canonical serialization, and
validation/deserialization. Raw artifacts live under the crate benchmark
artifact directory; prose results live in docs reports.

The accepted p95 budgets are 0.25/0.75/1.00/2.00/1.50/1.50/20/20 ms in that
order. Absolute acceptance is reference-desktop evidence; CI records trends.
Instrumentation must prove zero full graph/state clones per pointer move and no
per-frame allocation proportional to total graph size.

The 64-command validation and committed-mutation cases run against the large
2,048-region layout, as required by the accepted adaptive design. A narrow
closeout correction may optimize
`ui_composition/src/transaction/apply.rs::CompositionState::apply_authorized_mode`
and `ui_composition/src/history/journal.rs::CompositionHistory` to remove
redundant full-definition clones, keep the journal as the single entry owner,
use undo/redo journal indices, and batch repeated resize application while
preserving ordered transaction meaning, atomic validation, history, and
rollback semantics. These are the only core-composition source files authorized
by WR-185; they grant no adaptive mutation entrypoint.

Resize-only transactions may use an inductive validation scope: ratified input
already satisfies global graph invariants, and validated split-resize commands
cannot change topology, identity, ownership, location, or ordering. The batch
must still reject a missing or non-split region atomically. Every command class
that can change a global invariant continues through full candidate validation.
`ui_composition/tests/transaction_atomicity.rs` must prove both paths.

## Tests And Sequence

1. Add workspace crate/dependencies and semantic input contracts/tests.
2. Implement diagnostics, constraints, projection, and immutable hit index.
3. Implement proposal/preview/drag/resize/cancel without mutation authority;
   proposals carry typed intent and never fabricate topology-dependent core
   commands.
4. Implement promotion delta and exact edit classification.
5. Add accessibility metadata and semantic-source parity tests.
6. Add neutral fixtures and forbidden-import/product guards.
7. Add deterministic benchmark builders, separate runners, allocation/clone
   probes, docs, evidence, and closeout.
8. Enforce every p95 budget in the benchmark process and prove the large-layout
   64-command commit case without weakening the fixture or threshold.

Validation:

```text
cargo fmt --all --check
cargo test -p ui_adaptive_composition
cargo test -p ui_input semantic
cargo test -p ui_testing adaptive_composition
cargo test -p ui_composition
cargo bench -p ui_adaptive_composition --bench adaptive_composition
task ui:dependencies
task docs:validate
task planning:validate
```

Stop on direct composition mutation, app/product semantics, raw gamepad input,
non-deterministic proposals/diagnostics, full-graph per-move clone, linear
per-frame allocation, incomplete accessibility metadata, forbidden imports,
editor/Draw/engine/ui_surface changes, or a large-layout benchmark budget
breach. Closeout may claim `runtime_proven`, not product runtime integration or
perfectionist completion.

Edge docking is explicitly host-materialized. The adaptive crate does not know
identity-allocation policy, source-stack compaction, app extension deltas, or
window lifecycle, so it must never encode left/right/top/bottom as a center
`move_unit` command. Headless tests cover all five zones and prove active
proposal/session source contains no `CompositionCommand` authority.
