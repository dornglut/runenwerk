---
title: Phase Implementation Spec
description: Compact RON handoff contract for one bounded implementation phase.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ./README.md
  - ./templates/phase-implementation-spec.ron
  - ../authority-model.md
  - ../workflow-lifecycle.md
  - ../complete-investigation-gate.md
  - ../complete-design-gate.md
  - ../complete-merge-readiness-gate.md
  - ../routines/track-orchestration-routine.md
  - ../routines/implementation-routine.md
---

# Phase Implementation Spec

A phase implementation spec is a compact handoff contract for exactly one bounded implementation phase.

It is derived from accepted Markdown authority:

```text
accepted architecture/design docs
active-work entry
roadmap entry
production-track entry
decision-register entry
investigation/design/merge gates where required
closeout evidence from prior phases where relevant
```

## Authority rule

Markdown remains the primary design and process authority.

A phase spec must not become parallel authority. It is a machine-oriented contract only when the accepted workflow or design grants that role for the named phase scope.

When a phase spec disagrees with accepted Markdown authority:

```text
1. Update or supersede the owning Markdown authority first.
2. Record the decision when lifecycle or policy changes.
3. Align the phase spec after the authority decision.
```

Do not treat the spec as a shortcut around complete investigation, complete design, planning, merge readiness, or phase closeout.

## Format decision

Use RON for phase implementation specs.

Reason:

```text
Runenwerk is Rust-native.
A phase spec is one structured contract document, not an event stream.
RON remains readable in code review and can carry nested contract structure cleanly.
```

Do not use JSONL as the primary phase spec format.

Use JSONL for append-only event streams:

```text
runtime traces
agent output
validation/proof logs
future track-manager execution ledgers
```

## Required fields

A phase implementation spec should include:

```text
phase id
title
lifecycle state
implementation authorization status
owner
authority docs
complete investigation gate status
complete design gate status
principle compliance status
module decomposition status
maintainability review status
allowed paths
forbidden paths
public API surface
invariants
acceptance criteria
validation commands
evidence expectations
stop conditions
closeout expectations
next phase activation condition
```

## Lifecycle rule

A spec may describe a phase in `active-planning` before implementation is authorized.

A spec may support `active-implementation` only when the planning record separately authorizes:

```text
exact owner
complete implementation contract
allowed files/crates
forbidden files/crates
validation envelope
evidence expectation
stop conditions
complete investigation gate evidence where applicable
complete design gate evidence where applicable
principle compliance evidence where applicable
module decomposition evidence where applicable
maintainability review evidence where applicable
```

## Authorization status rule

Use an explicit implementation authorization status so agents cannot infer permission from accepted direction.

Recommended status values:

```text
active-planning-only
active-implementation-authorized
blocked
completed
```

A spec with `active-planning-only` or `blocked` may guide design, planning, review, or prompt generation. It must not be used to start implementation.

A spec with `active-implementation-authorized` is valid only when the corresponding planning record also authorizes active implementation.

## Scope rule

One spec covers one phase.

Do not use a single spec to implement an entire production track. A track manager may hold the whole goal, but each implementation agent receives one phase.

## Validation rule

The spec records validation commands and evidence expectations. It does not prove validation was run.

Reports must distinguish:

```text
command validation run
command validation unavailable
manual validation by inspection
CI validation
user-reported validation
blocked claims
```

Do not claim validation that was not run.

## Tooling rule

No validator or script is required by this workflow layer yet.

Validator support is downstream work. It may be added only after the spec shape is stable enough and an accepted design grants tooling an explicit contract role.
