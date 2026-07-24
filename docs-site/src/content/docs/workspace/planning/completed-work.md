---
title: Completed Work
description: Concise index of completed Runenwerk programs and durable evidence.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-24
related_docs:
  - ./roadmap.md
  - ./active-work.md
  - ../../reports/closeouts/README.md
---

# Completed Work

This page is a concise index. Pull requests, accepted ADRs/designs, and closeout reports own detailed evidence.

## Repository workflow and validation

- Issues and PRs `#122`, `#123`, and `#124` retired the production-track database, execution locks and contract packs, truth certificates, batch/worktree orchestration, generated prompts, quiet/full gates, workflow-only Python environment, and generated machine state.
- Issue `#135` / PR `#136` completed final repository workflow and documentation-surface pruning.
- Issues `#150` and `#154` / PR `#155` restored the complete Rust 1.97 and documentation validation baseline.
- Issue `#137` / PR `#138` adopted the pinned organization-owned reusable Rust workflow while retaining repository-owned `cargo validate` authority.
- Issue `#151` was closed as superseded by the complete shared-workflow adoption.
- PR `#141` aligned the root architecture summary with the canonical foundation crate inventory.

Permanent CI and `cargo validate` are the merge baseline. No retired workflow platform remains active.

## Repository-family architecture

- PR `#120`: public-readiness license and ignore policy.
- PR `#121`: public-facing README.
- Issue `#125` / PR `#126`: corrected repository-family GPU/render ownership and accepted `RunenRender -> RunenGPU`.
- Issue `#127` / PR `#128`: complete S0 current-source, identity, consumer, lifecycle, shader, macro, and file-disposition inventory.
- Issue `#129` / PR `#130`: original G1A implementation specification.

PR `#132` was closed without merge because it contained only temporary automation scaffolding and no Rust implementation. A later critical review identified that scalar-only graph-local resource IDs do not reject foreign handles when two flows allocate the same local value; the active G1A specification now requires a private owner scope.

## RunenSDF

- PR `#116`: internal SDF boundary correction.
- PR `#118`: standalone transfer authority and closeout.
- `dornglut/runen-sdf` PR `#1`: standalone source and conformance at source-transfer revision `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`.
- `dornglut/runen-sdf` PR `#2`: standalone authority closeout.
- `dornglut/runen-sdf` PR `#4`: shared validation workflow adoption.
- `dornglut/runen-sdf` PR `#5`: current `dornglut/*` repository authority and durable namespace validation, merged as `ffa970f3eb7fd9ebaa1cfc67665e3e3128cd0676`.

- Issue `#133` / PR `#157`: complete consumer census proved zero real internal-package consumers; retired `domain/sdf`, workspace and lockfile authority, stale local framework docs, and the transient census workflow; added durable no-return validation without adding an unused external dependency. See [PT-RUNENSDF-004 closeout](../../reports/closeouts/pt-runensdf-004-internal-sdf-retirement-closeout.md).

Runenwerk now has one reusable SDF source authority: `dornglut/runen-sdf`. Runenwerk retains only product/world integration such as `domain/world_sdf`.

## UI history

The former Runenwerk UI component and runtime-platform programs established substantial internal implementation and closeout evidence through PRs `#37`–`#107`. Reusable UI framework authority subsequently moved to `dornglut/runen-ui`.

Detailed historical Runenwerk UI evidence remains in `reports/closeouts`, accepted architecture/design documents, and Git history. It does not authorize new RunenUI framework work in Runenwerk.

## Evidence rule

A completed planning or architecture PR proves only the scope it changed. An internal future-transferable slice does not establish standalone package conformance or external extraction readiness. Do not infer runtime, Cargo, platform, or manual behavior validation that the exact PR did not run and record.
