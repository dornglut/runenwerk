---
title: Ratification Is Domain-Specific
description: Decision to use shared ratification vocabulary while keeping concrete validation rules in owning domains.
status: accepted
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-04-27
---

# ADR: Ratification Is Domain-Specific

## Status

Accepted

## Context

Generated, imported, projected, migrated, or externally modified state must be checked before acceptance.

## Decision

Use shared ratification vocabulary, but keep concrete validation rules in owning domains.

## Rejected Alternatives

One global validator; validation only in app code; trusting generated state because it came from a tool.

## Consequences

Each domain remains authoritative over its invariants while reports can be aggregated.
