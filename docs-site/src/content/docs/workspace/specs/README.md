---
title: Workspace Specs
description: Machine-oriented handoff contracts derived from accepted Markdown authority.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../authority-model.md
  - ../operating-model.md
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ./phase-implementation-spec.md
---

# Workspace Specs

Workspace specs are compact handoff contracts derived from accepted Markdown authority.

They exist to help humans and agents carry exact phase constraints into implementation without turning a prompt into a full design document.

## Authority rule

Markdown remains the primary design, process, and planning authority unless an accepted design explicitly grants a spec contract status for a specific scope.

A spec must not become parallel authority. If a spec and accepted Markdown disagree, update the owning Markdown authority or decision record first, then align the spec.

## Active spec docs

- [Phase Implementation Spec](phase-implementation-spec.md)

## Format rule

Use RON for phase implementation specs because Runenwerk is Rust-native and a phase spec is one structured contract document.

Do not use JSONL as the primary phase spec format.

Use JSONL for append-only streams such as runtime traces, agent output, validation/proof logs, and any future track-manager execution ledger.

## Tooling rule

No validator is required for this workflow layer yet.

Validator or script support is downstream work and must not become workflow authority unless an accepted design grants it that role for a named scope.
