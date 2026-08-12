---
title: Superseded Domain Authoring Source And Program Pattern
description: Supersession marker for the former cross-domain universal source/program lifecycle proposal.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-08-12
replacement_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
---

# Superseded Domain Authoring Source And Program Pattern

This cross-domain proposal no longer defines a lifecycle that every Runenwerk domain
must follow.

Use:

- [Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md)
  for the current family-wide architecture;
- [Domain Program Architecture Pattern](../../guidelines/domain-program-architecture-pattern.md)
  when an owning domain genuinely needs a durable source/program/compiler/evaluator
  lifecycle;
- [ADR 0018](../../adr/accepted/0018-semantic-federation-and-physical-realization.md)
  for semantic federation and shared-extraction constraints.

The following former assumptions are not current platform authority:

```text
all Runenwerk domains use one Source -> Domain Program lifecycle
stable IDs / versions / source-map / diagnostic envelopes are automatically shared
TypedGraph<DomainGraphKind> is a pre-authorized platform substrate
UI + one non-UI proof automatically establishes those shared shapes
RenderPlan is a generic Domain Program example
```

Domain-owned source/program/compiler/evaluator structures remain valid where their
actual semantics and lifecycle require them. Shared extraction remains blocked until
structurally different real domains prove the same neutral primitive and a separate
accepted extraction decision authorizes it.

## Why this marker remains temporarily

Current active design material still contains links to this historical proposal and
repository code search is unavailable during #251. The path remains only as a
non-authoritative supersession marker to avoid an unverified broken-link migration.

Removal condition: a later #205 reference/lifecycle cleanup may delete this marker after
all inbound links have been command-verified and migrated. It must not return to the
active-design index or be cited as the platform-wide authoring model.
