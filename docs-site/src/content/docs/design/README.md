---
title: Design Documents
description: Lifecycle guidance for Runenwerk architecture and implementation design documents.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-09-14
---

# Design Documents

Design documents record architecture and implementation intent that is broader than one local code comment but not necessarily durable enough to require an ADR.

Use the lifecycle folders deliberately.

## `active/`

Use for designs that are still being discussed, implemented, or validated.

Active does not mean provisional implementation permission. Current implementation work still requires an owning GitHub issue, bounded scope, and reviewed pull-request delivery.

Examples:

```text
active/editor-asset-pipeline-and-content-workflow-design.md
```

## `accepted/`

Use when the architectural direction has been accepted but implementation has not yet been checked completely against it.

Accepted design is architecture authority, not an implementation-completeness claim.

## `implemented/`

Use only after the accepted design has been checked against current code, tests, closeout evidence, and known drift.

An implemented design may still describe current compatibility/migration seams, but it must not claim a stronger owner or target than accepted ADRs and architecture spines permit.

## `deferred/`

Use for a preserved design direction that is intentionally not active.

Deferred documents do not authorize implementation.

## `superseded/`

Use for designs replaced by newer architecture, ADRs, or guidelines.

A superseded design is historical evidence. Its old planning language does not reactivate work or remain current semantic authority.

When moving a design to `superseded/`:

- set `status: superseded`;
- set `canonical: false`;
- name the current authority through `superseded_by` or an equivalent explicit status section;
- repair current references semantically;
- repair historical references by path only when history should remain unchanged;
- do not leave a forwarding copy at the old path.

## Authority Rules

Code and tests own current behavior.

Accepted ADRs own durable decisions and prohibitions. Canonical architecture spines own current repository/domain architecture. Designs refine those authorities for bounded topics.

When an older design conflicts with a newer accepted ADR, architecture spine, or owner source, the newer/current owner wins; reconcile or supersede the design instead of treating two documents as parallel authority.

Historical implementation plans, closeouts, audits, and roadmap-intake records are provenance unless current governance explicitly says otherwise.
