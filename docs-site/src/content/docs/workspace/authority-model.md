---
title: Authority Model
description: Conflict-resolution order for behavior, architecture, planning, and delivery evidence.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ./start-here.md
  - ./engineering-workflow.md
  - ./documentation-structure.md
---

# Authority Model

Every artifact has one job. Cross-link instead of copying decisions or live state.

## Current behavior

```text
code and executable tests
-> fixtures, captures, and runtime evidence
-> implementation documentation
-> summaries and historical reports
```

A design or roadmap may describe the target and the gap, but it does not override current behavior.

## Durable architecture

```text
accepted ADR or accepted design
-> concise root summary
-> active implementation issue
-> pull-request description
-> historical report
```

Implementation conforms to accepted architecture or explicitly revises the owning decision.

## Active work

```text
GitHub issue
-> maintained roadmap summary
-> pull-request delivery state
-> historical report
```

Do not place transient branch heads, daily execution state, or CI runs in stable architecture documents.

## Validation evidence

```text
exact-head CI
-> local command output identified by commit
-> user-reported command output
-> source inspection
-> unverified claim
```

Inspection can establish structure and likely behavior. It is not executed validation or runtime proof.

## Artifact ownership

- **Code and tests:** current behavior and executable invariants.
- **ADRs and accepted designs:** durable decisions, ownership, dependency direction, migration, and deletion contracts.
- **GitHub issues:** proposed, active, blocked, deferred, and completed work.
- **Roadmap:** high-level sequencing and dependencies.
- **Pull requests:** diff review, delivery evidence, and merge decision.
- **Root documents:** concise navigation and stable repository shape.
- **Reports and archive:** historical evidence and context only.

## Conflict handling

1. Name the exact conflicting claim.
2. Identify its owning authority.
3. Inspect current code and evidence.
4. Correct the lower-authority source or explicitly revise the owner.
5. Record unresolved uncertainty as a blocker.

Do not resolve conflicts by creating another parallel authority.
