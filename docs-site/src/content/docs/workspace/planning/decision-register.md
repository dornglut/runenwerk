---
title: Decision Register
description: Navigation to durable Runenwerk decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-13
related_docs:
  - ../authority-model.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ./roadmap.md
---

# Decision Register

Durable architectural decisions belong in accepted ADRs or accepted designs. This page is
navigation only; it does not duplicate their content or track live implementation state.

## Repository family

- [ADR 0014 — Repository Family Extraction Boundaries](../../adr/accepted/0014-repository-family-extraction-boundaries.md)
- [ADR 0015 — Separate GPU Execution from Rendering](../../adr/accepted/0015-separate-gpu-execution-from-rendering.md)

## Work and sequence authority

GitHub issues and the Engineering Portfolio own live work. The [roadmap](roadmap.md) owns
only durable high-level sequence and dependency direction. Pull requests own delivery and
review evidence.

When a new durable decision is required, add or revise its owning ADR/design instead of
extending this page into a parallel decision database.
