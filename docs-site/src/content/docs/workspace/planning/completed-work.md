---
title: Completed Work
description: Concise index of completed Runenwerk programs and durable evidence.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-22
related_docs:
  - ./roadmap.md
  - ./active-work.md
  - ../../reports/closeouts/README.md
---

# Completed Work

This page is a concise index. Pull requests, accepted ADRs/designs, and closeout reports own detailed evidence.

## Repository workflow simplification

Issues and PRs `#122`, `#123`, and `#124` retired the production-track database, execution locks and contract packs, truth certificates, batch/worktree orchestration, generated prompts, quiet/full gates, workflow-only Python environment, and generated machine state. Permanent CI and `cargo validate` became the validation baseline.

Issue `#135` is a final surface-pruning follow-up, not a reopening of the retired workflow platform.

## Repository-family architecture

- PR `#120`: public-readiness license and ignore policy.
- PR `#121`: public-facing README.
- Issue `#125` / PR `#126`: corrected repository-family GPU/render ownership and accepted `RunenRender -> RunenGPU`.

## RunenSDF

- PR `#116`: internal SDF boundary correction.
- PR `#118`: standalone transfer authority and closeout.
- `Crystonix/runen-sdf` PR `#1`: standalone source and conformance at commit `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`.

The later Runenwerk consumer cutover and `domain/sdf` deletion remain separate pending work.

## RunenGPU and RunenRender

- Issue `#127` / PR `#128`: complete S0 current-source, identity, consumer, lifecycle, shader, macro, and file-disposition inventory.
- Issue `#129` / PR `#130`: accepted G1A implementation specification for logical GPU work-resource identity.

PR `#132` was closed without merge because it contained only temporary automation scaffolding and no Rust implementation.

## UI history

The former Runenwerk UI component and runtime-platform programs established substantial internal implementation and closeout evidence through PRs `#37`–`#107`. Reusable UI framework authority subsequently moved to the standalone `Crystonix/runen-ui` repository.

Detailed historical Runenwerk UI evidence remains in `reports/closeouts`, accepted architecture/design documents, and Git history. It does not authorize new RunenUI framework work in Runenwerk.

## Evidence rule

A completed planning or architecture PR proves only the scope it changed. Do not infer runtime, Cargo, platform, or manual behavior validation that the exact PR did not run and record.
