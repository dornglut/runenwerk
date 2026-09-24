---
title: Superseded Runenwerk Domain Workbench North Star
description: Supersession record for the pre-ADR-0017 Domain Workbench Meta Kernel and universal Domain Program target.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-08-12
replacement_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ../../guidelines/domain-program-architecture-pattern.md
provenance:
  - ../../reports/investigations/2026-08-12-semantic-federation-and-inspection-provenance.md
---

# Superseded Runenwerk Domain Workbench North Star

This document no longer defines current Runenwerk architecture.

Its useful product ambition is preserved by the current
[Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md):
Runenwerk remains a coherent engine, tool, authoring, diagnostics, and Workbench
platform with rich lineage and inspectability.

The old implementation ontology is superseded. In particular, do **not** use this record
to justify:

```text
every major domain as a Domain Program
Meta Kernel as a pre-authorized substrate
universal Runen IDs
schema/program/package/evaluator/compiler/host/artifact registries
a universal typed graph substrate
a universal compiler/evaluator runtime
a universal domain pipeline
UI -> second domain -> automatic platform-kernel extraction
```

Current authority is:

1. [ADR 0017](../../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md)
   for authority, cross-owner consistency, graph semantics, incremental correctness,
   extraction gates, and progressive disclosure;
2. [ADR 0018](../../adr/accepted/0018-semantic-federation-and-physical-realization.md)
   for semantic federation, physical realization, and optional Workbench inspection;
3. [ADR 0019](../../adr/accepted/0019-batteries-included-application-composition.md)
   for one `App` composition root and batteries-included product ergonomics;
4. [Domain Program Architecture Pattern](../../guidelines/domain-program-architecture-pattern.md)
   when a specific domain genuinely needs durable authored/versioned/compiled intent.

The detailed research ancestry and rejected universalizations are retained in the
[Semantic Federation and Inspection Provenance Investigation](../../reports/investigations/2026-08-12-semantic-federation-and-inspection-provenance.md)
and Git history.

This record remains only as historical design evidence. New work must follow the
replacement authority above.
