---
title: UI Component Platform Authoring Kit Design
description: Implemented Runenwerk-local authoring helpers for constructing ordinary ControlPackage and ControlKind contracts without bypassing validation or ownership boundaries.
status: implemented
owner: ui
layer: domain
canonical: true
last_reviewed: 2026-09-13
related_designs:
  - ./ui-component-platform-control-kernel-design.md
  - ./ui-component-platform-story-proof-envelope-design.md
  - ../active/runenwerk-ui-story-driven-golden-workflow-design.md
  - ./editor-ui-runtime-v2-and-interaction-formation-design.md
related_docs:
  - ../../domain/ui/architecture.md
  - ../../domain/ui/roadmap.md
---

# UI Component Platform Authoring Kit Design

## Status

Implemented in the current Runenwerk-local `ui_controls` stack. This document describes the landed authoring contract rather than a future implementation pass.

## Implemented contract

The authoring kit constructs ordinary package/module/kind descriptors; it is not a parallel package model. Current implementation lives under:

```text
domain/ui/ui_controls/src/authoring/mod.rs
```

The landed public concepts include:

- `ControlKindAuthoringSpec`;
- `ControlKernelAuthoring` and `ControlAuthoredKernels`;
- `ControlSchemaAuthoring`;
- `ControlEvidenceAuthoring`;
- `ControlModuleAuthoringBuilder`;
- package authoring/building helpers exported by `ui_controls`.

The authoring path derives stable namespaced ids, creates the five control-kernel roles, attaches schema/route/evidence requirements, preserves descriptor-only compatibility defaults, and leaves runtime mount eligibility conservative.

## Landed ownership

`ui_controls` owns this construction convenience. Package validation, registry insertion, route ownership, story proof, renderer/runtime behavior, and product mutation retain their own owners. The manual descriptor path remains legitimate; authoring helpers do not become hidden global authority.

## Intentional landed differences

The proposal suggested starting with one `authoring.rs` file. The implementation matured into `src/authoring/mod.rs`; that physical split is implementation structure, not a semantic divergence.

Candidate names were also refined around the actually required construction stages. The durable contract is the responsibility split above, not preservation of every proposal-era type name.

## Boundary rules

- Authoring helpers must produce normal validated descriptors.
- They must not bypass `ControlPackageDescriptor` validation or registry checks.
- They must not convert evidence requirements into proof.
- They must not grant runtime mount eligibility.
- They must not absorb Gallery, Workbench, UI Designer, renderer, ECS, app, editor, or game behavior.
- Shared framework extraction remains separate from this Runenwerk-local implemented contract.

## Evidence

Current `ui_controls` authoring source and focused package/authoring tests exercise the builder path and its fail-closed interaction with package validation. Historical phase handoff text remains provenance rather than current authority.
