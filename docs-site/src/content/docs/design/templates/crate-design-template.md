---
title: Crate Design Template
description: Template for crate-level architecture design documents.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# Crate Design Template

## Purpose

State why the crate exists.

## Scope

List what this crate owns.

## Non-Scope

List what this crate explicitly does not own.

## Architectural Position

Layer:

```text
foundation | domain | engine/runtime | app | adapter/tool | test-support
```

Allowed dependencies:

```text
...
```

Forbidden dependencies:

```text
...
```

## Ownership Rules

Define who creates, mutates, validates, and consumes the concepts owned by this crate.

## Public API Policy

State which modules/types are stable public contracts and which are internal implementation details.

## Invariants

List invariants that must always hold.

## Failure Modes

Describe how invalid input, invalid state, and boundary violations are reported.

## Diagnostics

List diagnostic code families owned by this crate.

## Ratification

Describe what generated, imported, projected, migrated, or externally modified state this crate must ratify.

## Commands

Describe mutating command boundaries owned by this crate.

## Inspection

Describe read-only DTOs or snapshots exposed for tools, tests, debug UIs, or AI-assisted workflows.

## Persistence and Versioning

Describe serialization, versioning, and migration policy.

## Extension Points

Describe intended extension mechanisms.

## Testing Strategy

List required unit, invariant, ratification, command, projection, golden, or smoke tests.

## Negative Doctrine

State what this crate is not.
