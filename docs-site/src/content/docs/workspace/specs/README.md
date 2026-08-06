---
title: Workspace Specs
description: Machine-oriented handoff contracts derived from accepted Markdown authority.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-04
related_docs:
  - ../authority-model.md
  - ../engineering-workflow.md
  - ../operating-model.md
  - ./phase-implementation-spec.md
  - ../../design/active/runengpu-g4b-contracts-g4c-delivery-design.md
---

# Workspace Specs

Workspace specs are compact handoff contracts derived from accepted Markdown authority.

They exist to help humans and agents carry exact phase constraints into implementation
without turning a prompt into a full design document.

## Authority rule

Markdown remains the primary design, process, and planning authority unless an accepted
design explicitly grants a spec contract status for a specific scope.

A spec must not become parallel authority. If a spec and accepted Markdown disagree,
update the owning Markdown authority or decision record first, then align the spec.

## Active spec docs

- [Phase Implementation Spec](phase-implementation-spec.md)

## RunenGPU implementation specs

- [PT-RUNENGPU-G4A Context Admission](pt-runengpu-g4a-context-admission.ron)
- [PT-RUNENGPU-G4B Program, Resource Interface, and Layout](pt-runengpu-g4b-program-interface-layout.ron)
- [PT-RUNENGPU-G4C Realization Program Umbrella](pt-runengpu-g4c-wgpu-realization-cutover.ron)
- [PT-RUNENGPU-G4C1 Resource Realization](pt-runengpu-g4c1-resource-realization.ron)
- [PT-RUNENGPU-G4C2 Program and Binding Realization](pt-runengpu-g4c2-program-binding-realization.ron)
- [PT-RUNENGPU-G4C3 Pipeline Realization and Cutover](pt-runengpu-g4c3-pipeline-cutover.ron)

The accepted and planned order is:

```text
G4A accepted
    -> G4B blocked until accepted issue #209 and activated issue #187
        -> G4C1 blocked until accepted G4B
            -> G4C2 blocked until accepted G4C1
                -> G4C3 blocked until accepted G4C2
```

The G4C umbrella specification is not directly implementable. Each G4C child requires
its own issue, branch, pull request, exact-head validation, complete-diff review, and
accepted predecessor merge.

## Format rule

Use RON for phase implementation specs because Runenwerk is Rust-native and a phase
spec is one structured contract document.

Do not use JSONL as the primary phase spec format.

Use JSONL for append-only streams such as runtime traces, agent output, validation or
proof logs, and any future track-manager execution ledger.

## Tooling rule

No dedicated spec validator is required yet.

Any future validator must remain subordinate to accepted Markdown authority and the
repository validation commands defined by [Engineering Workflow](../engineering-workflow.md).
