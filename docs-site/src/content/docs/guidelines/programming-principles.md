---
title: Runenwerk Programming Principles
description: Runenwerk adaptation of seven common programming principles for long-lived architecture, docs, and code review.
status: active
owner: workspace
layer: guidelines
canonical: true
last_reviewed: 2026-06-25
related_docs:
  - ../software-development/principles.md
  - ./architecture.md
  - ./code-patterns.md
---

# Runenwerk Programming Principles

This page adapts the common seven programming principles into Runenwerk's architecture and workflow rules.

The source list is:

1. KISS
2. DRY
3. YAGNI
4. SOLID
5. Separation of Concerns
6. Avoid Premature Optimization
7. Law of Demeter

Use these principles as a review lens. Do not use them as slogans that override domain ownership, dependency direction, accepted ADRs, or tests.

## 1. KISS: keep the owned path simple

Prefer the smallest readable design that protects the invariant.

In Runenwerk, simple does not mean under-modeled. It means the owner, contract, validation path, and failure behavior are obvious.

Reject clever routing, hidden global state, broad catch-all abstractions, and workflow indirection that makes ownership harder to see.

## 2. DRY: remove duplicate authority

Do not repeat source-of-truth content across root docs, workspace docs, generated views, plans, and reports.

One artifact should own each durable claim. Other files should point to it or summarize it briefly.

Duplication is acceptable only when it is a short entrypoint summary and the owning canonical file is linked.

## 3. YAGNI: do not build speculative surfaces

Do not add crates, commands, registries, extension points, generic APIs, planning tracks, generated views, or workflow layers until there is an accepted owner and near-term need.

Future-proofing means leaving clean seams, not adding unused machinery.

## 4. SOLID: keep responsibility and dependency boundaries honest

Use SOLID as a practical boundary check:

- one module or crate should have one clear reason to change;
- extension should happen through owned contracts, not by editing unrelated internals;
- substitutions must preserve documented behavior;
- interfaces should stay narrow and purpose-specific;
- stable policy should not depend on outer runtime, app, adapter, or tooling details.

## 5. Separation of Concerns: organize by responsibility

Separate domain meaning, runtime execution, app wiring, adapters, docs, tests, planning, and historical evidence.

A file or folder should answer what responsibility it owns. Avoid dumping unrelated process, product status, and implementation detail into the same document.

## 6. Avoid premature optimization: prove the bottleneck first

Do not optimize architecture, APIs, docs, or workflow for imagined scale before the actual pressure is visible.

Prefer clear contracts and focused validation first. Optimize only after evidence shows the bottleneck, cost, or maintainability issue.

## 7. Law of Demeter: depend on direct contracts

Code, docs, and workflows should talk to their direct owner or explicit contract, not reach through another layer's internals.

Use public APIs, DTOs, commands, ratifiers, schemas, routines, and task cards as boundaries. If a caller needs deep knowledge of another subsystem, the boundary is probably missing a contract.

## Review checklist

Before accepting non-trivial work, check:

```text
KISS: Is the path understandable without hidden ceremony?
DRY: Is there exactly one authority for each durable claim?
YAGNI: Is every new surface needed now?
SOLID: Does each owner have one clear responsibility and legal dependencies?
SoC: Are code, docs, plans, reports, and tooling separated by purpose?
Optimization: Is optimization driven by evidence instead of speculation?
Demeter: Does the change use direct contracts instead of reaching through internals?
```
