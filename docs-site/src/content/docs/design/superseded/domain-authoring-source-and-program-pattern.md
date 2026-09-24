---
title: Superseded Domain Authoring Source And Program Pattern
description: Supersession record for the former cross-domain universal source/program lifecycle proposal.
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
  for current family-wide architecture;
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

This record remains only as historical design evidence. New work must follow the
replacement authority above.
