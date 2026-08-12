---
title: Superseded Runenwerk Capability Workbench Target Architecture
description: Supersession marker for the former Capability Cells, Platform Planes, and Maturity Layers Runenwerk-wide target.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-08-12
replacement_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../../adr/accepted/0018-semantic-federation-and-physical-realization.md
  - ../../adr/accepted/0019-batteries-included-application-composition.md
  - ./editor-tool-suite-registry-and-workbench-host-design.md
---

# Superseded Runenwerk Capability Workbench Target Architecture

This document no longer defines a Runenwerk-wide future target.

The platform-wide Workbench direction is now governed by the
[Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md)
and [ADR 0018](../../adr/accepted/0018-semantic-federation-and-physical-realization.md):
independent semantic owners may expose optional owner-selected inspection projections
to a federated Workbench without adopting a universal Workbench ontology, registry,
identity system, or execution runtime.

The former broad vocabulary is not current platform doctrine:

```text
Capability Cells
Platform Planes
Maturity Layers
family-wide capability-cell identities
a Runenwerk-wide capability registry/policy-plane ontology
```

Do not reintroduce those concepts merely to make tools composable.

The useful concrete editor/tool-host direction has a narrower current owner in
[Editor Tool Suite Registry And Workbench Host Design](./editor-tool-suite-registry-and-workbench-host-design.md),
which keeps tool-suite/workbench composition, provider routing, app-owned integration,
and domain-owned semantics explicit.

Application-level batteries-included composition is separately governed by
[ADR 0019](../../adr/accepted/0019-batteries-included-application-composition.md).

## Why this marker remains temporarily

Current guidance still contains historical links to this path, including the generic
authority-centered architecture guideline. Repository code search is unavailable during
#251, so this path remains only as a non-authoritative supersession marker until those
secondary links can be command-verified and migrated.

Removal condition: delete this marker in a later #205 reference/lifecycle cleanup once
all inbound references are verified. It must not return to the active-design index or
serve as Runenwerk's specialization of the authority-centered doctrine.
