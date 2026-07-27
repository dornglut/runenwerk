---
title: Completed Work
description: Concise index of completed Runenwerk programs and durable evidence.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ./roadmap.md
  - ./active-work.md
  - ../../reports/closeouts/README.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../../reports/closeouts/pt-runen-family-operational-hardening-closeout.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../../reports/investigations/runen-family-operational-hardening-investigation.md
  - ../specs/pt-runengpu-g3-access-work-graph.ron
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

PR `#132` was closed without merge because it contained only temporary automation scaffolding and no Rust implementation. A later critical review identified that scalar-only graph-local resource IDs do not reject foreign handles when two flows allocate the same local value; the accepted G1A specification was corrected before implementation.

## RunenGPU

### G1A

- Issue `#131` / PR `#164`: implemented the first internal future-transferable RunenGPU slice at merge `5bbdab36ae661d99432bfe5d215062c397aac975`.
- `engine::plugins::gpu` owns owner-scoped `GpuWorkResourceId`, its owner-controlled fallible allocator, and structured allocation exhaustion.
- `RenderResourceId` and `RenderResourceIdSequence` were deleted without aliases, forwarding exports, or duplicate authority.
- Resource-allocating render-flow, pass, procedural, GPU-primitive, application, example, benchmark, and test authoring paths propagate structured `RenderFlowAuthoringError`.
- Foreign uniform and storage handles are rejected even when independent flows allocate equal local components.
- The GPU identity module remains independent of renderer, ECS, WGPU, Winit, application, and domain types.
- Detailed evidence is recorded in the [PT-RUNENGPU-G1A closeout](../../reports/closeouts/pt-runengpu-g1a-closeout.md).

### G2

- Issue `#172` / PR `#173` implemented normalized capability facts and requirements, validated logical resource descriptors, kind-typed handles, prepared-data boundaries, explicit non-optional render lowering, bounded adapters, full consumer migration, and deletion of replaced declaration authority.
- G2 changed no manifest, dependency, lockfile, workflow, or external-package authority and introduced no G3-G7 execution behavior or compatibility path.
- Independent review corrected target aliases to validated semantic binding keys and replaced ambiguous equality with explicit GPU allocation-compatibility and transitional declared-Rust-type predicates.
- G2 merged as `709aa6aced020ee99405e1e1c3dde7703c77a4d4`.
- Detailed evidence is recorded in the [PT-RUNENGPU-G2 implementation closeout](../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md).

### G3 planning

- Issue `#174` / PR `#175` completed the decision phase for checked buffer, texture, and query access; render attachments and canonical clear values; buffer zero; query-set-to-buffer resolution; graph-entry initialization; RAW/WAR/WAW hazards; typed import/export causality; operation-derived capabilities; immutable work fragments/nodes; and deterministic prepared-graph authority.
- Review corrected multisample texture resolution to a render-attachment relation, limited standalone clear work to buffer zeroing, added the real timestamp query-resolution path, rejected redundant explicit data edges, and kept runtime generations/retirement in G4/G5.
- G3 planning merged as `5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8`.
- Accepted authority is recorded in the [G3 design](../../design/active/runengpu-g3-access-work-graph-design.md), [investigation](../../reports/investigations/runengpu-g3-access-work-graph-investigation.md), and [implementation specification](../specs/pt-runengpu-g3-access-work-graph.ron).
- This planning completion does not prove Rust G3 implementation. Issue `#177` owns that work and must be reverified against the exact post-PR-`#178` `main` before source changes.

### Operational hardening

- Issue `#176` / PR `#178` completed the documentation-only current-source audit and operational-contract reconciliation after accepted G3 planning.
- The slice classified external-system weaknesses, inherited backend limits, and Runen-introduced risks; retained direct WGPU as the strongest substitute; and added strategic reevaluation gates.
- Family authority now binds accepted-work integrity, structured pressure, derived-cache doctrine, Runenwerk-owned compatibility/recovery/persisted reproducibility, and performance anti-cheating rules.
- Existing G4-G8 phases now own portability/cache compatibility, progress/pressure/completion/shutdown, direct-WGPU comparisons, device generations/loss/reconstruction, and operational conformance.
- RunenRender authority now uses `dornglut/runen-render`, narrow provider capabilities, deterministic incremental prepared-scene lifecycle, generation/changed-region cache invalidation, and R8 performance/capture proof.
- The proof portfolio retains exact 4,097-element prefix scan and 160×90/16-step Game-of-Life vectors with live count `2,063` and FNV-1a-64 `0xBD710B88594CD584`.
- Detailed evidence is recorded in the [Runen family operational-hardening closeout](../../reports/closeouts/pt-runen-family-operational-hardening-closeout.md).
- This entry becomes authoritative through the merge of PR `#178`; the candidate deliberately asserts no merge SHA.

## RunenSDF

- PR `#116`: internal SDF boundary correction.
- PR `#118`: standalone transfer authority and closeout.
- `dornglut/runen-sdf` PR `#1`: standalone source and conformance at source-transfer revision `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`.
- `dornglut/runen-sdf` PR `#2`: standalone authority closeout.
- `dornglut/runen-sdf` PR `#4`: shared validation workflow adoption.
- `dornglut/runen-sdf` PR `#5`: current `dornglut/*` repository authority and durable namespace validation, merged as `ffa970f3eb7fd9ebaa1cfc67665e3e3128cd0676`.
- Issue `#133` / PR `#157`: complete consumer census proved zero real internal-package consumers; retired `domain/sdf`, workspace and lockfile authority, stale local framework docs, and transient census workflow; added durable no-return validation without adding an unused external dependency. See [PT-RUNENSDF-004 closeout](../../reports/closeouts/pt-runensdf-004-internal-sdf-retirement-closeout.md).

Runenwerk now has one reusable SDF source authority: `dornglut/runen-sdf`. Runenwerk retains only product/world integration such as `domain/world_sdf`.

## UI history

The former Runenwerk UI component and runtime-platform programs established substantial internal implementation and closeout evidence through PRs `#37`–`#107`. Reusable UI framework authority subsequently moved to `dornglut/runen-ui`.

Detailed historical Runenwerk UI evidence remains in closeouts, accepted designs, and Git history. It does not authorize new RunenUI framework work in Runenwerk.

## Evidence rule

A completed planning or architecture PR proves only the scope it changed. An internal future-transferable slice does not establish standalone package conformance or external extraction readiness. Do not infer runtime, Cargo, platform, or manual behavior validation that the exact PR did not run and record.
