---
title: Workspace Specs
description: Bounded implementation handoff contracts derived from accepted Markdown and owning GitHub issues.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../operating-model.md
  - ./phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
---

# Workspace Specs

Workspace specs are compact handoff contracts derived from accepted Markdown and the
owning GitHub issue. They exist when a bounded implementation slice benefits from a
machine-readable, decision-complete constraint set without turning a prompt or issue
body into a second design document.

## Authority rule

A workspace spec is **subordinate handoff detail**, not a live work tracker.

Current authority is:

```text
accepted ADR / design / architecture
    -> durable technical decision

maintained roadmap
    -> durable sequence where sequence matters

owning GitHub issue
    -> active work, activation, current base and current state

pull request
    -> delivery, reviewed head, validation and acceptance evidence

workspace spec
    -> bounded implementation constraints derived from those owners
```

If a spec and accepted Markdown disagree, fix the owning durable authority first. If a
spec's lifecycle, base, issue, branch, CI or delivery field disagrees with current
GitHub state, **GitHub state wins**. Historical lifecycle fields in retained RON files
are snapshots of the handoff when it was authored; they are not maintained as a
parallel project database.

A spec must never grant implementation permission by itself, activate a phase, replace
an owning issue, certify a merge, or require consumers to infer current status from an
old lifecycle string.

## Current use

RON phase specs are retained only where they contain useful bounded implementation or
historical handoff detail. Their presence does not make recorded lifecycle state current
authority.

The retained RunenGPU G-phase RON files are **historical predecessor handoffs after the
completed standalone transfer**. Current reusable RunenGPU semantics, validation, work,
and repository state belong to `dornglut/runen-gpu`; Runenwerk's current compatibility
claim is its exact dependency pin plus integration validation. Do not derive a current
RunenGPU phase or implementation target from a retained Runenwerk RON file.

Older RunenECS, RunenSDF and UI RON files may likewise remain as historical handoff
snapshots until their owning documentation cleanup decides whether the detail is still
useful. Their presence does not make their recorded `lifecycle_state` current authority.

The generic [Phase Implementation Spec](phase-implementation-spec.md) is superseded as
a repository-wide lifecycle template. Its historical rules do not reactivate a phase
manager or machine-owned execution lifecycle.

## Format rule

RON is appropriate for one Rust-native structured handoff contract. It is not a
repository task database or append-only execution ledger.

Do not add a universal phase schema, generated run ledger, execution lock, truth
certificate, or track manager merely because several historical specs share fields.

## Tooling rule

No dedicated spec lifecycle validator is required. A future validator may check local
schema or deterministic constraints, but it must remain subordinate to accepted
Markdown, the owning GitHub issue, and the repository validation commands defined by
[TESTING.md](../../../../../../TESTING.md).

In particular, tooling must not update RON lifecycle fields to mirror GitHub state,
reject current GitHub authority because an older retained spec records an earlier
planning snapshot, or treat a predecessor RunenGPU spec as authority over standalone
`dornglut/runen-gpu`.
