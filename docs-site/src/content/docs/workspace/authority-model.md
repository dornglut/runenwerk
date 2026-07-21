---
title: Authority Model
description: Conflict-resolution order for Runenwerk behavior, architecture, planning, delivery evidence, and generated views.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ./start-here.md
  - ./engineering-workflow.md
  - ./documentation-structure.md
---

# Authority Model

Every repository artifact has one job. Avoid duplicating the same decision or state across several files.

## Authority order

When sources conflict, use this order for the specific claim being made.

### Current behavior

```text
code and executable tests
-> fixtures, captures, and runtime evidence
-> implementation documentation
-> summaries and reports
```

A design or roadmap cannot override current code behavior. It may state the intended target and the gap.

### Durable architecture and ownership

```text
accepted ADR or accepted architecture/design document
-> root architecture and dependency summaries
-> active implementation issue
-> pull-request description
-> historical report
```

Implementation must either conform to accepted architecture or update the owning decision explicitly.

### Active work and sequencing

```text
GitHub issue state and accepted milestone/roadmap decision
-> active roadmap view
-> pull-request state
-> generated planning view
-> historical report
```

Do not put volatile branch heads, transient CI runs, or daily execution state in stable architecture documents.

### Delivery and validation evidence

```text
exact-head CI
-> local command output identified by commit
-> user-reported command output
-> connector/manual inspection
-> unverified claim
```

Inspection can establish structure and likely behavior. It cannot be reported as command or runtime validation.

## Artifact responsibilities

- **Code and tests:** current behavior and executable invariants.
- **ADRs and accepted designs:** durable decisions, ownership, dependency direction, migration and deletion contracts.
- **GitHub issues:** proposed, active, blocked, deferred, and completed work.
- **Roadmap:** high-level sequencing and dependencies; not an execution ledger.
- **Pull requests:** diff review, delivery evidence, and merge decision.
- **Root summaries:** concise navigation and stable repository shape.
- **Generated views:** derivative convenience output; they must identify their source.
- **Historical and superseded documents:** context only; they do not authorize new work.

## Workflow authority

[Engineering Workflow](engineering-workflow.md) owns process and validation terminology.

Repository scripts may validate deterministic invariants. They do not grant permissions, certify truth, promote lifecycle state, or override accepted documents and GitHub state.

The legacy production-track, execution-lock, truth-certificate, batch, and generated-prompt systems are deprecated under issue `#122`. Their state is compatibility data until active consumers migrate, not higher-order authority.

## Single-source rule

Before adding or updating an artifact, identify the existing owner.

```text
one durable decision -> one accepted ADR or design
one active task -> one GitHub issue
one required validation baseline -> cargo validate
one implementation delivery -> one PR
one derived view -> one named source
```

Cross-link instead of copying full state.

## Conflict handling

When a conflict is found:

1. Name the exact claim in conflict.
2. Identify the owning authority for that claim.
3. Inspect current code and evidence.
4. Correct the lower-authority source or explicitly revise the owner.
5. Record unresolved uncertainty as a blocker.

Do not resolve conflicts by creating another parallel authority.
