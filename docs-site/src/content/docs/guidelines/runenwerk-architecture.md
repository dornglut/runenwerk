---
title: Superseded Runenwerk Multi-Reality Architecture Doctrine
description: Supersession marker for the former multi-reality and nine-layer governing doctrine.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-08-12
replacement_docs:
  - ../architecture/runenwerk-platform-architecture.md
  - ../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../adr/accepted/0019-batteries-included-application-composition.md
---

# Superseded Runenwerk Multi-Reality Architecture Doctrine

This page no longer defines Runenwerk's governing architecture.

Use the [Runenwerk Platform Architecture](../architecture/runenwerk-platform-architecture.md)
as the current top-down spine.

The former doctrine's useful owner-specific lessons remain valid where an owning domain
proves them, including:

- authored, formed, instantiated, and simulated state may need distinct contracts;
- transient session/tool state must not silently become authoritative state;
- retention and recovery requirements differ by owner;
- cross-scope migration, trust, retries, compensation, and terminal failure need
  explicit contracts where they exist;
- not every subsystem needs journaling, replay stability, mergeability, or durable
  history.

Those lessons no longer justify a universal `Reality` ontology, one observation-frame
or expression-frame system, generic propagation structures, family-wide reconciliation/
stability/retention enums, global-looking version/transaction families, or nine
platform layers as the governing model.

Current accepted laws are more precise:

- [ADR 0017](../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md)
  owns authority, owner-local consistency/admission, graph/feedback distinctions,
  incremental correctness, retention pressure, capability terminology, and extraction;
- [ADR 0018](../adr/accepted/0018-semantic-federation-and-physical-realization.md)
  owns the positive semantic-federation and physical-realization model;
- [ADR 0019](../adr/accepted/0019-batteries-included-application-composition.md)
  owns application/product composition and progressive-disclosure ergonomics.

## Why this marker remains temporarily

Current documentation still contains links to this historic doctrine. During #251 the
GitHub repository code-search endpoint is unavailable, so the path remains as an
explicit non-authoritative marker rather than being deleted before every inbound
reference can be command-verified.

Removal condition: a later #205 reference/lifecycle cleanup may delete this page after
all current inbound references have been verified and migrated. No new document should
cite this page as architecture authority.
