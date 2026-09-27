---
title: Runenwerk Terminal Automation Orchestration And Authored Scenario Design
description: Accepted design for shared launch-owned automation orchestration, typed authored scenario composition, replay steps, result history, and product-local terminal lowering after A10.
status: accepted
owner: engine
layer: engine-runtime / integration
canonical: true
last_reviewed: 2026-09-27
publication: reference
pagefind: false
related_adrs:
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0023-normalize-app-runtime-host-lifecycle-and-capability-ownership.md
related_designs:
  - ./runenwerk-application-automation-session-semantic-model.md
  - ./runenwerk-app-runtime-and-composition-semantic-model.md
  - ./runenwerk-physical-input-semantic-model.md
  - ./runenwerk-editor-coordination-semantic-model.md
  - ./runenwerk-render-lab-product-design.md
---

# Runenwerk Terminal Automation Orchestration And Authored Scenario Design

## Status and decision scope

This design is the A11 concrete orchestration decision derived from accepted automation work
through A10.

Reviewed accepted state at design preparation:

```text
RUNENWERK_MAIN=12bc8e6b69260ff1bf07e71fca61b35a63670f8b
A10_MERGE=2d571c380287b9d73f386db3389cec521310d8e5
RUNENINPUT_PIN=2751e19fa42255b86e786e7cd837198c917b7b25
```

The intervening accepted #970 change extends Render Lab temporal-quality terminal/runtime
behavior. It does not move automation authority, but it reinforces that product terminal
surfaces have independent responsibilities beyond automation.

This design refines the already accepted application-automation session model. It does not
authorize implementation by itself.

## Problem now proven by current source

The shared semantic model says one automation session owns sequencing, execution-mode
selection, cancellation, scoped cleanup, and current-run result history.

Current source only partially realizes that model.

`AutomationSession` currently owns:

- execution-local session and normalized-input source identity;
- ProductSemantic versus NormalizedInput mode checks;
- typed `AutomationOwnerAdapter` dispatch and query;
- normalized observation admission;
- condition polling;
- cancellation/finish;
- source-scoped continuity cleanup.

A9's accepted Render Lab terminal path does not use that session for persisted replay. It
currently performs:

```text
bounded file read
  -> V1 import
    -> replay-source map construction
      -> AppAutomationInputReplayExt replay
        -> direct owner-adapter query
          -> AppAutomationInputReplayExt teardown
```

That path is truthful, but it means a history added only around the current
`AutomationSession` methods would not describe the first accepted terminal workflow.

A10 then proved that the same A6/A8 replay machinery reaches a second independent product
through atomic tablet groups and Draw-owned document semantics. Editor already proves a
third pressure shape through product-semantic commands and lifecycle-sensitive targets.

The problem is therefore no longer lack of product pressure. The missing boundary is shared
orchestration over the already accepted mechanisms.

## Current mechanism disposition

| Mechanism | Disposition | Reason |
| --- | --- | --- |
| `AutomationSession` | adapt upward | existing Engine orchestration owner; do not create a competing runner state machine |
| `AutomationOwnerAdapter` | retain | keeps target/command/query/observation payloads owner-typed |
| A6 `AppAutomationInputReplayExt` | reuse | owns normalized trace replay/preflight/progress/teardown semantics |
| A8 V1 import/export | reuse | owns persisted normalized-trace schema and validation |
| A9 bounded file read and CLI parsing | retain product-local | filesystem path and terminal UX are caller concerns |
| A9 direct replay/query orchestration | adapt into shared session path | currently bypasses shared sequencing/history |
| Render Lab automation adapter | retain | product-owned camera query |
| Draw automation adapter | retain | product-owned document query |
| Editor automation adapter | retain | product-owned command/query and target identity |
| product headless builders | retain product-local | composition/fixture ownership stays with products |
| generic provider registry | reject for first slice | no current need and risks duplicated product authority |
| persisted authored scenario schema | defer | current pressure does not justify owner codec/selector persistence yet |
| central `runenctl` | defer | shared semantic layer should be proven before a central UX/catalog |

## Decision summary

The first shared terminal-automation layer is the **existing Engine automation session
extended to cover replay and ordered execution history**.

The accepted lowering is:

```text
product-local terminal / test caller
  -> product-owned launch and target selection
    -> Engine AutomationSession
      -> typed owner operation
      -> or accepted persisted-trace import/replay
      -> typed owner query/assertion
      -> scoped cleanup
      -> owner-neutral ordered step/result history
```

The session becomes the one orchestration boundary for both direct typed automation and
persisted normalized replay. A6/A8 remain the semantic owners of replay and persistence.

Do not introduce a parallel `AutomationRunner` state machine for the same lifetime.

## Authored scenario: first supported form

The first authored scenario representation is a **statically typed in-process scenario
program**, not a generic persisted AST.

Conceptually:

```rust
fn run_scenario(
    session: &mut AutomationSession,
    app: &mut App,
    product: &mut ProductAutomationContext,
) -> Result<ScenarioResult, ProductError> {
    // ordered authored intent
}
```

The exact Rust shape is implementation-owned, but the laws are fixed:

- step order is authored intent;
- product targets, commands, queries, observations, and assertions remain typed by the
  product owner;
- shared orchestration records owner-neutral execution knowledge;
- the program may include one persisted normalized-trace replay step;
- an observed trace never becomes the scenario merely because the scenario references it;
- no universal product command/query/target/observation enum is created.

This form is intentionally less ambitious than a human-editable scenario file. It gives the
shared executor real cross-product pressure without prematurely freezing selector codecs,
product payload serialization, or scenario migration policy.

## Why no generic scenario AST yet

A generic data AST would immediately need answers for persistence of owner-local values such
as:

- Editor mounted-unit selectors and tool commands;
- Draw document selectors and drawing assertions;
- Render Lab camera assertions;
- future owner-specific evidence requests.

Those values do not share one semantic owner.

Erasing them to strings or one cross-product enum would recreate the universal command bus
already rejected by the canonical automation design.

Therefore the first implementation may expose helper/building APIs for typed scenario
programs, but it MUST NOT introduce persisted generic owner payloads.

## Product construction and provider boundary

A11 does **not** require a new generic product-provider trait for the first implementation.

Current maintained products already own explicit construction:

```text
Render Lab -> build_headless_automation_app()
Draw       -> runtime::build_headless_app()
Editor     -> runtime::build_headless_app()
```

and already expose typed owner adapters.

The product-local caller therefore remains responsible for:

- selecting the product/workbench/fixture;
- constructing the launch-owned App;
- resolving product-local target identity/selectors;
- constructing typed commands/queries/assertions;
- presenting product-specific terminal help and diagnostics.

The shared session consumes those owner-local values through the existing typed adapter
boundary.

A future central terminal or repeated cross-product launch plumbing may justify a narrow
provider/harness trait. That future trait must be derived from actual duplicated launch
code, not introduced speculatively in A11.

## Launch and process boundary

The first shared orchestration implementation supports:

```text
launch-owned
+ headless Host
+ same-process orchestration
```

A product terminal executable may itself be the process entry point, as A9 already proves,
but the controlled App is constructed and owned in that process.

Not supported by this design's first implementation:

- attach to an already running application;
- local IPC;
- remote control;
- native-window automation orchestration;
- external OS event injection;
- claims about native raw-device wake/pacing.

Those require separate evidence and, for reachable control transports, separate security
decisions.

## Persisted trace as an explicit scenario step

A persisted A8 V1 normalized trace may be referenced by authored intent only as an explicit
NormalizedInput replay operation.

The layers remain:

```text
terminal/path UX
  -> bounded bytes supplied by caller
    -> A8 import/validation
      -> A6 replay with explicit source mapping / freshness assumptions
        -> product behavior
```

The session may coordinate these calls and record their result, but it MUST NOT:

- duplicate A8 DTOs or parser validation;
- reinterpret trace identity as product identity;
- infer an expected owner result from the trace;
- persist a product selector/assertion into the generic trace artifact;
- relabel normalized replay as NativeOs.

Filesystem path identity belongs to the terminal/tool caller, not the trace semantic model.

## Replay source mapping

A11 does not change A6's source-mapping contract.

Replay still requires a complete injective recorded-source to fresh replay-source map and
truthful target-state assumptions.

The first shared implementation MAY factor a session-local helper out of A9 only if it
preserves these laws and remains valid specifically for a fresh launch-owned headless target.

Otherwise the caller continues to supply the mapping.

No global mutable input-source allocator is introduced.

## Session-owned ordered history

The shared orchestration session retains one ordered current-run history.

A conceptual record is:

```text
AutomationStepRecord {
    sequence
    operation
    execution_mode?
    outcome
    progress?
    diagnostic_detail?
}
```

Exact Rust names are not fixed by this design.

### Operation classes

The history must be able to distinguish at least:

- product dispatch;
- owner query;
- owner assertion;
- normalized direct injection;
- persisted normalized-trace import/replay;
- condition wait;
- finish;
- cancel;
- replay cleanup when it is an independently failing lifecycle action.

### Outcome knowledge

The owner-neutral outcome vocabulary preserves at least:

- Dispatched;
- AdmittedOrDelivered;
- EffectConfirmed;
- AssertionPassed;
- AssertionFailed;
- Unsupported;
- Inconclusive;
- Cancelled;
- InfrastructureFailure.

Replay additionally retains bounded progress sufficient to identify:

- completed frame count;
- failing frame ordinal when known;
- failing group index when known.

The history does not serialize successful owner observations. Typed observations remain
returned to the typed caller.

Diagnostic text may be retained for infrastructure failure, but it is not a substitute for
the machine-readable outcome.

### Distinct evidence streams

Session history is not the normalized input trace.

Keep separate:

```text
A3/A4/A8 trace
  = exact admitted normalized input facts

session history
  = ordered orchestration operation/result knowledge

owner evidence
  = typed product/domain observations and artifacts
```

They may be correlated by run/step order in later tooling, but A11 does not merge them.

## Assertion semantics

The shared session may coordinate an assertion, but the product owner defines what the
assertion means.

A generic assertion step therefore consumes an owner-supplied typed predicate/assertion
operation and records only the resulting knowledge:

```text
AssertionPassed      # owner-defined assertion is definitively true
AssertionFailed      # owner-defined assertion is definitively false
Inconclusive         # required state/evidence was not established, for example timeout
InfrastructureFailure
```

A definitive mismatch MUST NOT be collapsed into `Inconclusive`; timeout/unknown and
observed-false are different knowledge states.

`AutomationStepResult` does not currently expose `AssertionFailed`. The first
implementation successor is authorized to add that owner-neutral result variant so the
shared history can preserve this distinction.

It MUST NOT copy product state into a universal assertion schema.

Dispatch remains distinct from effect confirmation, and effect confirmation remains
distinct from assertion success.

## Waiting and advancement

The existing session wait primitive polls a typed owner query with an explicit timeout.

A11 does not claim that polling alone advances an App or proves asynchronous readiness.

For a future wait whose condition can change only after App advancement, the selected
product/caller must supply the owner-correct advancement opportunity. Arbitrary sleep is not
a substitute.

The first implementation must not silently add hidden frame advancement to a generic wait.

## Finish, cancel, and replay cleanup

After A11, session completion semantics cover both:

- direct automation-owned normalized input from the session source;
- replay-owned input state created by a replay step coordinated by the session.

Cancellation or finish must clean only state owned by that automation execution.

If replay teardown can fail independently, that failure remains visible in history/result
knowledge. Cleanup failure must not be overwritten by a successful earlier query/assertion.

A6 currently has asymmetric cleanup that the session MUST preserve:

- replay rejection or failure before completion already cleans replay-owned input and does not
  leave an active replay lease;
- only a completed replay retains replay-owned source identity in the App until explicit teardown.

The session therefore tracks whether a coordinated replay actually completed before treating replay
teardown as pending. It MUST NOT call teardown after a failed replay and convert the original
failure into a secondary `ReplayNotActive` error.

A6 replay is currently a synchronous operation with no cancellation callback. The first shared
implementation therefore supports cancellation **between orchestration steps**, not interruption
inside one replay call. Internal replay failure still reports A6 partial progress. Mid-replay
operator cancellation requires a separate A6 cancellation design and is not implied by A11.

Unrelated physical/programmatic input remains untouched.

## Capability discovery

A11 keeps discovery **selected-product-local** for the first implementation.

The product caller already knows which product it launched and owns its CLI/help surface.
A central owner-neutral registry would add no required capability today.

A future provider may expose a bounded descriptor for:

- supported Host classes;
- supported execution modes;
- normalized-trace replay support;
- target-selection shape;
- owner command/query/assertion families;
- evidence kinds.

Such a descriptor is a projection of the selected owner contract. It never becomes an
independent mutable registry or a replacement for typed APIs.

## Terminal packaging alternatives

### Product-local terminal plus shared session — selected

Benefits:

- reuses the existing accepted Engine orchestration owner;
- keeps product CLI parsing and filesystem UX with the product;
- keeps commands/queries/targets statically typed;
- avoids root workspace product catalogs;
- allows A9 to delete direct orchestration duplication.

Cost:

- product binaries retain intentionally separate help/argument surfaces.

This is acceptable because terminal UX is not semantic authority.

### Central CLI with static product catalog — deferred

A central executable could compile all selected product crates and match a bounded product
name to product-specific code.

It is not needed yet and would couple release/build/package policy across products before a
second real terminal consumer exists.

### Generic product-provider trait — deferred

A provider trait could normalize launch and discovery, but current products do not yet
duplicate enough launch/orchestration code to justify its type/borrow complexity.

### Continue only product-local wrappers — rejected

This leaves replay, query, teardown, and history orchestration duplicated and fails the
accepted shared-session model now that two independent replay consumers exist.

## Pressure-test lowering

### Render Lab

```text
product CLI reads orbit-pan-zoom.ron within A8 byte bounds
  -> product builds fresh headless Render Lab App
    -> shared session coordinates persisted replay
      -> typed RenderLabAutomationAdapter camera query
        -> owner-specific camera assertion
          -> shared finish/cleanup
            -> ordered history
```

This proves normalized input and Render Lab camera semantics only.

### Draw

```text
typed in-process scenario receives persisted pen-stroke trace bytes
  -> product builds fresh headless Draw App
    -> shared session coordinates persisted tablet replay
      -> typed DrawingAutomationAdapter document query
        -> owner-specific stroke/sample assertion
          -> shared finish/cleanup
            -> ordered history
```

All-tablet atomicity remains A6/A8 input evidence. Drawing meaning remains Draw/Drawing
authority.

### Editor

```text
product builds fresh headless Full Editor
  -> product resolves current mounted viewport
    -> shared session dispatches ProductSemantic Rotate command
      -> typed owner query / optional wait
        -> owner-specific Rotate assertion
          -> shared finish
            -> ordered history
```

Mounted-unit identity remains lifecycle-sensitive Editor authority.

### Unsupported/native request

A headless scenario selecting NativeOs returns Unsupported and records Unsupported.

It MUST NOT fall back to normalized replay.

### Failure and cleanup

A stale target, malformed trace, replay failure after partial progress, timeout, cancellation,
or teardown failure produces truthful terminal knowledge. Cleanup is scoped to automation
ownership and does not manufacture releases for unrelated input.

## A9 migration and deletion path

A9 remains accepted evidence that terminal replay works.

The first A11 implementation successor should refactor only the generic orchestration parts
of A9:

- V1 import/replay coordination;
- replay outcome/progress classification;
- typed query routing through the shared session;
- replay cleanup coordination;
- history recording.

Render Lab retains:

- command-line parsing;
- path selection;
- bounded file read;
- product App construction;
- product camera output formatting.

After the refactor, the product-local duplicated helper code should be deleted rather than
kept as a compatibility path.

## First implementation successor

After A11 design acceptance, create exactly one bounded implementation issue.

That issue should:

1. extend the existing Engine `AutomationSession` rather than add a competing runner;
2. add ordered owner-neutral current-run history;
3. add the owner-neutral `AssertionFailed` result needed to distinguish definitive mismatch from `Inconclusive`;
4. make persisted normalized-trace replay a session-coordinated operation reusing A8/A6;
5. preserve typed owner results outside generic history;
6. make finish/cancel account for replay-owned teardown where the session coordinated replay;
7. route the accepted Render Lab A9 replay/query/cleanup through the shared session while
   leaving filesystem/CLI UX product-local;
8. prove Draw persisted tablet replay produces the same session/history shape;
9. prove Editor ProductSemantic dispatch/query produces the same session/history shape;
10. prove Unsupported never falls back and failure/cancellation cleanup is scoped;
11. prove failed replay does not schedule a bogus teardown, while completed replay retains teardown responsibility;
12. prove cancellation is truthful at orchestration-step boundaries without claiming mid-replay interruption;
13. add no persisted scenario AST, central CLI, provider registry, IPC, native automation,
    new crate, or universal product payload enum.

The implementation issue must re-census current writers. In particular, any Render Lab
writer must be accepted or path-disjoint before adapting A9 terminal code.

## Scenario persistence successor gate

Do not create a persisted scenario format merely after the first shared session
implementation lands.

Reassess only after a real caller needs to hand off an authored multi-step scenario across a
process or machine boundary.

That future design must explicitly solve:

- product selector/version identity;
- owner-local target selector persistence;
- owner command/query/assertion codecs;
- scenario schema versioning and migration;
- trace artifact reference/location semantics;
- environment/fixture requirements;
- untrusted input bounds and permissions.

A8 RON syntax is not precedent that forces the same encoding.

## Central CLI successor gate

A central `runenctl` becomes worth reconsidering only when at least two maintained product
terminal callers duplicate product-selection/launch/discovery plumbing after the shared
session exists.

Even then, the CLI remains a caller. It must not own product semantics.

## Security

The first implementation is launch-owned, local, same-process, and explicitly selected by
the caller.

It introduces no listening socket, background daemon, global recorder, remote endpoint, or
OS-wide automation permission.

Persisted trace bytes remain untrusted and are validated by A8.

Any future attach/remote/native control path requires separate security and Host decisions.

## ADR disposition

No new ADR is required.

This design does not transfer semantic authority between repositories or domains. It
concretizes the already accepted Runenwerk-owned automation session/orchestration boundary
using already accepted product adapters, App/Host semantics, A6 replay, and A8 persistence.

A future standalone automation repository, remotely reachable control service, or
cross-framework semantic transfer would require a new decision.

## Non-goals

This design does not authorize:

- production `runenctl`;
- generic persisted scenario AST or parser;
- trace-to-scenario automatic conversion;
- provider/plugin registry for products;
- IPC/socket/stdio attach protocol;
- remote control;
- native OS input injection;
- native-window replay fidelity claims;
- RunenInput semantic or persistence changes;
- RunenUI interaction replay absorption;
- Scene simulation replay absorption;
- Drawing-domain redesign;
- Editor identity redesign;
- renderer/performance work.

## Acceptance and successor gate

This design is acceptable only if a cold review confirms:

- the selected boundary reuses rather than duplicates `AutomationSession`, A6, and A8;
- all three product pressure tests lower without universal owner payload types;
- terminal/file UX remains caller-owned;
- replay and direct product operations share one ordered result-history owner;
- cancellation/cleanup ownership remains scoped;
- NativeOs remains explicitly unsupported where unproven;
- current docs and repository validation pass on the exact reviewed head;
- the diff remains design-only.

After accepted-main verification, create exactly one implementation issue for the bounded
shared-session delivery above.

Do not pre-authorize scenario persistence, a central CLI, attach, or native automation.
