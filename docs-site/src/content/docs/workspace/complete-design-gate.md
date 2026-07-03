---
title: Complete Design Gate
description: Mandatory complete-design gate before Runenwerk planning authorizes implementation.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-03
related_docs:
  - ./start-here.md
  - ./workflow-lifecycle.md
  - ./complete-investigation-gate.md
  - ./authority-model.md
  - ./operating-model.md
  - ./planning/README.md
  - ./routines/architecture-governance-review-routine.md
  - ./routines/implementation-routine.md
  - ./routines/pr-review-routine.md
  - ../guidelines/programming-principles.md
---

# Complete Design Gate

## Purpose

This document defines the mandatory complete-design gate for Runenwerk work.

Use this gate before planning may authorize implementation for architecture-sensitive, reusable, platform, public API, workflow, domain-boundary, or production-track work.

The gate exists to make implementation execute a complete contract on the first pass. It prevents designs and plans from relying on vague exclusions, hidden assumptions, post-implementation correction cycles, or unreviewed maintainability debt.

## Prerequisite

Use [`complete-investigation-gate.md`](complete-investigation-gate.md) before this gate when the current reality, owner, vocabulary, capability inventory, alternatives, evidence, or confidence level is not already proven.

A design may not convert unknown investigation findings into assumptions. Unknowns remain blockers until investigation records them or resolves them.

## Core rule

Implementation may start only after the complete target and implementation shape are understood and recorded.

```text
Complete investigation first.
Complete design second.
Complete planning contract third.
Implementation fourth.
Completion only after the declared contract is proven.
```

A design may be delivered through multiple named phases, but the target design must not be incomplete in thought. Every capability that is not delivered by the current phase must have a named owning layer, named downstream contract, and activation condition.

Do not use weak planning language to hide missing decisions.

```text
Forbidden as design substitutes:
  minimal
  narrow
  just enough
  placeholder
  partial
  MVP
  later
  future work
  out of scope without owner
  deferred without reactivation condition
  temporary without removal condition
  allowed file path as a substitute for module decomposition
```

Acceptable wording must name the owning contract.

```text
Use instead:
  complete target
  full capability map
  full support matrix
  complete owner map
  complete ergonomic contract
  complete validation envelope
  complete acceptance gate
  complete implementation contract
  module decomposition map
  maintainability review status
  named downstream owner
  named downstream phase
  named activation condition
  explicit non-owned responsibility
```

## Required when

Apply this gate whenever work touches any of these:

```text
public API
reusable platform capability
domain boundary
durable vocabulary
workflow authority
production-track phase
app composition
host integration
renderer-neutral contract
input behavior
accessibility behavior
inspection/catalog/report surface
cross-domain dependency
new crate or shared extraction
```

For local bug fixes or behavior-preserving refactors, use the implementation or code-refactor routine. If the work reveals missing ownership, incomplete design, maintainability risk, or public API uncertainty, stop and apply the complete investigation gate before continuing.

## Complete design checklist

A design is complete only when it records all applicable items below.

```text
Lifecycle:
  current lifecycle state
  intended state transition
  implementation authorization status
  investigation gate status

Authority:
  owning design, ADR, domain docs, tests, and planning files
  reference-only files
  known conflicts and the authority file that resolves them

Problem and target:
  problem statement
  complete target capability
  current reality from investigation
  non-owned responsibilities
  success definition

Owner map:
  owning domain/crate/subsystem
  participating crates
  forbidden owners
  host/product/editor/game boundary
  app/layout boundary
  runtime/proof boundary

Vocabulary and naming:
  durable internal/platform names
  user-facing/facade names
  compatibility names
  names forbidden from public API
  rename, alias, and migration decisions

Maintainability and decomposition:
  module decomposition map
  responsibility per file/module
  public re-export shape
  maximum acceptable file/module growth risk
  split condition for any compound file
  tests mapped to modules
  reason if a single file is intentionally kept

Ergonomics and usability:
  authoring path
  safe defaults
  configuration surface
  inspection/debug path
  recovery actions
  accessibility expectations
  input expectations
  failure diagnostics

Feature support:
  delivered capabilities for this contract
  named downstream capabilities
  explicitly non-owned capabilities
  support evidence for each delivered capability
  owner and activation condition for each downstream capability

Future-use-case pressure:
  future consumers
  needs from the current contract
  responsibilities outside the current contract
  scope-leak risks
  guardrails that keep the current owner honest

Hierarchy and composition:
  app/layout hierarchy owner
  retained UI/runtime hierarchy owner
  semantic/product hierarchy owner
  containment allowed in this contract
  mutation and persistence forbidden in this contract

Validation and evidence:
  unit/integration/proof tests
  docs validation
  dependency checks
  inspection/catalog/report evidence
  static mount or generated proof evidence where applicable
  local command status or connector-mode unavailability

Implementation contract:
  exact files/crates allowed
  exact files/crates forbidden
  implementation sequence
  validation envelope
  evidence expectation
  completion criteria

Stop conditions:
  missing owner
  missing evidence
  dependency exception
  new crate need
  shared framework extraction pressure
  product/editor/app leakage
  renderer/backend ownership leak
  host/session effect leak
  vocabulary drift
  compound implementation file without a decomposition decision
```

If any required item is unknown, the result is not an implementation plan. It remains investigation or design work.

## Complete planning checklist

Planning may move work to `active-implementation` only when the planning record points to a design or accepted authority that satisfies the complete design checklist.

The active planning record must name:

```text
ID
Title
State
Owner
Authority
Investigation gate status
Complete design gate status
Implementation contract
Allowed files/crates
Forbidden files/crates
Module decomposition map
Maintainability review status
Validation envelope
Evidence expectation
Feature support matrix
Future-use-case pressure matrix
Hierarchy/composition matrix when relevant
Known risks
Stop conditions
Next action
```

Do not mark implementation active from accepted direction alone. Accepted direction approves the target shape; active implementation requires the complete contract above.

## Feature support matrix

Use this structure in designs or planning records for reusable, platform, or public API work.

```text
| Capability | Status in current contract | Owner | Evidence required | Downstream owner / activation condition |
|---|---|---|---|---|
| <capability> | Delivered | <owner> | <tests/proofs/reports> | — |
| <capability> | Named downstream contract | <future owner> | <handoff evidence> | <phase/design/condition> |
| <capability> | Explicit non-owned responsibility | <outside owner> | <boundary evidence> | <owning design/contract> |
```

The matrix must avoid unowned buckets. Every capability has an owner and evidence path.

## Future-use-case pressure matrix

Use this structure when a contract is intended to be reusable or generic.

```text
| Future use case | Needs from current contract | Must own outside current contract | Scope-leak risk | Required guard |
|---|---|---|---|---|
| <consumer> | <facts/API/contracts needed> | <semantic owner> | <how scope can leak> | <rule/test/proof that blocks leak> |
```

A pressure matrix does not authorize those consumers for implementation. It proves that the current contract does not block or absorb them incorrectly.

## Hierarchy and composition matrix

Use this structure when work touches layout, retained trees, surfaces, canvases, graphs, editor state, or app composition.

```text
| Hierarchy layer | Owner | Current contract may do | Current contract must not do |
|---|---|---|---|
| App/layout hierarchy | <owner> | <allowed role> | <forbidden mutation/persistence> |
| Runtime/retained hierarchy | <owner> | <allowed containment> | <forbidden semantic ownership> |
| Product/semantic hierarchy | <owner> | <facts consumed or emitted> | <forbidden ownership> |
```

This matrix prevents one reusable platform feature from becoming a product scene graph, app composition system, renderer backend, or editor document model by accident.

## Module decomposition matrix

Use this structure when the implementation contract has more than one responsibility or when a file would mix declarations, validation, projection, runtime proof, rendering proof, tests, migration, or host integration.

```text
| Module / file | Responsibility | Public API exported | Tests proving it | Split trigger |
|---|---|---|---|---|
| <path> | <single responsibility> | <pub items or none> | <test/proof> | <condition requiring further split> |
```

A single file is acceptable only when it has one cohesive responsibility, no independent validation/projection/runtime sub-responsibilities, and an explicit reason recorded in the design or planning record.

## Ergonomics and usability matrix

Use this structure for public APIs, reusable controls, editor-facing behavior, and platform features.

```text
| Actor action | Complete authoring path | Default behavior | Inspectable evidence | Failure/recovery behavior |
|---|---|---|---|---|
| <action> | <builder/descriptor/API path> | <defaults> | <catalog/report/inspection facts> | <diagnostic/recovery> |
```

The design must state how the feature is pleasant to use, easy to inspect, and recoverable when invalid.

## Acceptance rule

A reviewer must reject implementation authorization if the design or planning record lacks required complete-design evidence.

A reviewer must reject completion if the implementation does not prove the declared contract.

```text
No complete investigation gate when required -> no complete design gate.
No complete design gate -> no active implementation.
No module decomposition map for compound implementation -> no active implementation.
No complete implementation contract -> no coding task.
No complete evidence -> no completion claim.
```

## Reporting requirement

Final reports for work using this gate must include:

```text
Investigation gate status:
Complete design gate status:
Feature support matrix status:
Future-use-case pressure matrix status:
Hierarchy/composition matrix status:
Module decomposition status:
Ergonomics/usability status:
Implementation contract status:
Validation status:
Remaining blockers:
Next state transition:
```
