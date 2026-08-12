---
title: Runenwerk Domain Workbench North Star Compatibility Pointer
description: Temporary noncanonical pointer for two legacy UI design consumers while RunenUI historical authority is reconciled under issue 205.
status: active
owner: workspace
layer: history
canonical: false
last_reviewed: 2026-08-12
replacement_docs:
  - ../../architecture/runenwerk-platform-architecture.md
  - ../superseded/runenwerk-domain-workbench-north-star.md
removal_condition: Remove after ui-program-architecture.md and ui-program-proof-slice-plan.md are reconciled by the separate RunenUI legacy/adoption cleanup under issue 205.
---

# Runenwerk Domain Workbench North Star Compatibility Pointer

This path is **not current architecture authority**.

Current Runenwerk-wide architecture is the
[Runenwerk Platform Architecture](../../architecture/runenwerk-platform-architecture.md),
backed by ADR 0017, ADR 0018, and ADR 0019.

The original Domain Workbench / Meta Kernel design is retained only as historical
evidence in
[Superseded Runenwerk Domain Workbench North Star](../superseded/runenwerk-domain-workbench-north-star.md).

## Why this pointer exists

Repository validation for the #251 cutover identified exactly two tracked Markdown
consumers of the old active path:

```text
ui-program-architecture.md
ui-program-proof-slice-plan.md
```

Both belong to the older Runenwerk-local UI design authority that #205 already plans to
reconcile separately with standalone RunenUI. Rewriting those large UI designs inside
#251 would mix the platform-spine cutover with that separate lifecycle decision.

This pointer therefore preserves those two concrete links without preserving the old
Meta Kernel as current authority.

## Removal condition

Delete this pointer after the separate RunenUI legacy/adoption cleanup migrates or
retires both consumers. Do not add new references to this file.
