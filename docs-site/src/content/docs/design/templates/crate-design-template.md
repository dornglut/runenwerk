---
title: Crate Design Template
description: Template for crate-level architecture design documents.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-10-29
publication: repository-current
draft: true
pagefind: false
sidebar:
  hidden: true
---

# [Crate name] — Design

> **Using this template:** Copy its relevant structure, not its template-page
> settings. Replace the title, owner, layer, status, publication and review date
> for the new document. The `draft: true`, `pagefind: false`, and hidden-sidebar
> values hide **this template page**; do not carry them into a published
> document unless the owning documentation policy requires them. Remove this
> instruction after copying. Follow the [design lifecycle](../README.md)
> and accepted ownership rules.

## Purpose

What existing responsibility or requirement justifies this crate? Link the
owning issue and source-backed need.

## Scope and non-scope

- **Owns:** [what the crate is responsible for]
- **Does not own:** [neighboring behaviors, policies, or framework authority]

## Architectural position and dependencies

- **Layer:** [foundation / domain / engine-runtime / app / adapter-tool / other accepted owner]
- **Permitted dependencies:** [actual consumers and dependencies, grounded in current architecture]
- **Forbidden dependencies:** [edges that would violate the accepted design]

Use the current Runenwerk architecture and workspace inventory; do not turn an
integration crate into authority for a separately owned framework.

## State and ownership

Who creates, changes, validates, and observes each important concept? What
invariants must hold, and who enforces them?

## Public interface and compatibility

Which APIs or formats are public contracts, which are internal, and what
compatibility guarantees actually apply?

## Failures and diagnostics

How do invalid inputs, ordering/lifetime failures, and boundary violations
appear to a caller? List diagnostic code families only if this crate owns them.

## Operations and extension points (when relevant)

Describe owned mutating commands, read-only inspection, ratification of external
state, or supported extension seams **only when they exist or are required**.
Do not invent subsystems merely to fill headings.

## Persistence and migration (when relevant)

Identify formats, versioning, migration, rollback and compatibility obligations.
If there is no persisted state or migration, say so briefly.

## Validation and tests

State the observable positive and negative cases that prove the invariants,
critical failures, consumer integration, and applicable platforms. Point to
repository-owned tests and required canonical validation, not invented results.

## Alternatives and trade-offs

What simpler option (including keeping the current boundary) was considered?
Why does this ownership and abstraction remain justified?

## Open questions

Record unresolved design decisions and their owner; a template does not itself
authorize changes to public contracts or architecture.
