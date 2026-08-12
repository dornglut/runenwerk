---
title: Start Here
description: Entry point for Runenwerk engineering work.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-12
related_docs:
  - ./engineering-workflow.md
  - ./authority-model.md
  - ./documentation-structure.md
  - ../architecture/runenwerk-platform-architecture.md
---

# Start Here

## Repository work

1. Read [`AGENTS.md`](../../../../../AGENTS.md) for the concise operating rules.
2. Inspect the owning code and tests.
3. Read the [Runenwerk Platform Architecture](../architecture/runenwerk-platform-architecture.md)
   and the relevant accepted ADR or owner design for architectural work.
4. Use the active GitHub issue when the task is already planned.
5. Run focused checks while editing and `cargo validate` before merge.

## Primary authority

- [Engineering Workflow](engineering-workflow.md) — process and validation.
- [Authority Model](authority-model.md) — conflict resolution and artifact ownership.
- [Documentation Structure](documentation-structure.md) — where durable documents belong.
- [Runenwerk Platform Architecture](../architecture/runenwerk-platform-architecture.md) — current top-down platform spine.
- [Roadmap](planning/roadmap.md) — high-level sequencing.
- [Repository-family architecture](../architecture/repository-family-architecture.md) — framework ownership.
- [Dependency Rules](../guidelines/dependency-rules.md) — dependency and cutover rules.

## Architecture cold start

For a cross-domain architecture task, use this order:

```text
Runenwerk Platform Architecture
  -> owning accepted ADR(s)
  -> owning subsystem/framework architecture
  -> active issue and delivery design
  -> current code/tests
```

Accepted ADRs override conflicting active/historical design material.

## Review

A pull request is reviewed against its actual diff, owning tests, accepted authority, acceptance criteria, and exact-head CI. Historical reports and superseded workflow/design pages do not authorize new work.
