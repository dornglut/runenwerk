---
title: Superseded Domain Authoring Platform Overview
description: Supersession marker for the former Runenwerk-wide universal Domain Program lifecycle overview.
status: superseded
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-08-12
replacement_docs:
  - ./runenwerk-platform-architecture.md
  - ../guidelines/domain-program-architecture-pattern.md
  - ../adr/accepted/0018-semantic-federation-and-physical-realization.md
---

# Superseded Domain Authoring Platform Overview

This overview no longer defines a platform-wide Runenwerk authoring lifecycle.

Use the [Runenwerk Platform Architecture](./runenwerk-platform-architecture.md) for
current platform architecture and the
[Domain Program Architecture Pattern](../guidelines/domain-program-architecture-pattern.md)
for domains that genuinely need durable authored, versioned, inspectable, migratable,
compiled, or evaluable semantic intent.

The following former claims are **not** current universal Runenwerk doctrine:

```text
all domains follow one Source -> Domain Program -> Runtime Artifact lifecycle
UI is the platform-wide first proving domain
platform-owned shared IDs/manifests/envelopes/graph substrate are pre-authorized
Domains own meaning / platform owns structure implies common implementation ownership
```

ADR 0018 instead standardizes semantic reasoning questions while leaving domain
identities, revisions, storage, execution, and physical representation owner-specific.
Shared implementation still requires the accepted extraction gate.

A material, asset, procedural, or other authored domain may still use the Domain Program
pattern when its actual lifecycle warrants it. GPU resources, mounted UI runtime state,
render scene snapshots, network streams, spatial availability, scheduler readiness, and
other non-program contracts do not become Domain Programs merely to participate in the
platform.

## Why this marker remains temporarily

Current supporting documentation still contains historical links to this path and
repository code search is unavailable during #251. This page remains only as a
non-authoritative supersession marker until all inbound references can be verified.

Removal condition: a later #205 reference/lifecycle cleanup may delete this marker after
command-verified link migration. No new current architecture should cite it as an
authority.
