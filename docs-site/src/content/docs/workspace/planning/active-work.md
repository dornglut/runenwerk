---
title: Active Work
description: Current bounded Runenwerk work and immediate next decisions.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-29
related_docs:
  - ../engineering-workflow.md
  - ./roadmap.md
  - ./completed-work.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runengpu-g4-context-program-realization-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../reports/investigations/runengpu-g4-context-program-realization-investigation.md
  - ../../reports/closeouts/pt-runengpu-g1a-closeout.md
  - ../../reports/closeouts/pt-runengpu-g2-implementation-closeout.md
  - ../../reports/closeouts/pt-runengpu-g3-implementation-closeout.md
  - ../specs/pt-runengpu-g4a-context-admission.ron
  - ../specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
---

# Active Work

GitHub issues and pull requests own live delivery. This page records the durable
cross-project state and the only authorized next slice.

## Accepted RunenGPU foundation

```text
S0 inventory                         complete
G1A logical work-resource identity   complete
G2 capabilities and resources        complete at 709aa6aced020ee99405e1e1c3dde7703c77a4d4
G3 decision phase                    complete at 5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
operational hardening                complete at 90d24abb93bff4b1d3f5b4743056bc00ff80d4b6
G3 Rust implementation               accepted at 39d6fe65a334502bdfba0b1a2ce3b365099fcf28
verified-head maintenance            accepted at 6bbd341691a34763ef54c8ca059940cac8981265
```

The commit after accepted G3 changes only validation/workflow authority. It changes no
RunenGPU or render architecture, source, dependency, manifest, or lockfile.

## Current decision authority

Issue `#182` and planning PR `#185` own the G4 decision phase. Their scope is planning
and documentation only. They bind three ordered implementation slices:

```text
G4A context and adapter/device admission
 -> G4B program, interface, binding and pipeline contracts
 -> G4C WGPU realization, cache compatibility and cutover
```

The planning change is accepted only when its exact reviewed head passes repository
validation and documentation build, no blocking review finding remains, and the diff
contains no Rust implementation.

After accepted planning:

- G4A is the only slice that may become active;
- G4B remains blocked by accepted G4A;
- G4C remains blocked by accepted G4B;
- G5, G7, RunenRender implementation and package extraction remain unauthorized.

The focused design and three specifications are the implementation handoff. The three
slices must not be collapsed into one issue or pull request.

## G4 ownership summary

### G4A

Owns async headless `GpuContext` request, context/device-generation identity,
normalized backend/adapter/device facts, deterministic requirement admission and
degradation, private WGPU instance/adapter/device/queue containment, and temporary host
compatibility without taking surface ownership from G7.

### G4B

Owns WGSL-first source keys/revisions, program and entry-point descriptors, typed
binding keys/declarations, explicit interfaces, bind-group and pipeline layouts,
specialization schemas/values, deterministic compute/render pipeline descriptors, and
compile-pass/compile-fail contract proof.

### G4C

Owns generation-bound WGPU resource/program/layout/bind-group/pipeline realization,
private registries, correctness-complete in-memory cache keys and rejection, complete
consumer migration, deletion of renderer-owned realization/cache authority, synthetic
logical handles, the temporary owner bridge, and G4-owned sidecar truth.

## Retained later-phase findings

Current timing readback remains migration evidence:

```text
map_async
-> device.poll(wait_indefinitely)
-> channel receive
-> decode/publish evidence
```

G5 must bind native/web progress ownership, pressure, callbacks, bounded waits,
completion, cancellation, readback, shutdown and delayed retirement. G4 does not
normalize this behavior into execution authority.

Current surfaces remain Winit/renderer-coupled migration evidence. G7 owns reusable
raw-handle admission, surface identity/configuration/acquisition/presentation, device
replacement, loss and reconstruction facts. G4 may retain only one explicitly temporary
host-compatibility seam.

Pipeline and compiler stalls, driver behavior and adapter availability are inherited
environment facts. G4 normalizes compatibility and diagnostics without claiming to
eliminate them.

## Queued RunenGPU program

- G5: headless execution, uploads, query-resolution encoding, submission, progress,
  pressure, completion, asynchronous readback, cancellation, pending-work shutdown and
  delayed retirement;
- G6: offscreen graphics, shared render/non-render consumers, direct-WGPU comparisons
  and cost characterization;
- G7: surfaces, thread affinity, device generations/loss and reconstruction facts;
- G8: operational conformance, reproducibility facts, diagnostics, shutdown, cache and
  residual reach-through audit;
- GX: external `dornglut/runen-gpu` clean cutover only after accepted G2-G8 evidence.

Only one implementation issue is active at a time.

## RunenRender boundary

RunenRender remains S0/design only until accepted external RunenGPU cutover and its own
separately bounded R-phase work. G4 is GPU/backend decontamination and substrate work;
it does not implement image-formation semantics. The current render tree is migration
evidence, not a wholesale rename or extraction target. RX remains a later mechanical
transfer/cutover, not the point where renderer architecture is invented.

## Retained proof portfolio

```text
G4 deterministic
    context/generation and admission algorithms
    descriptors, typed bindings and interface compatibility
    specialization and pipeline descriptor identity
    cache compatibility and stale/foreign rejection
    compile-pass/fail and source/dependency guards

G4 environment-dependent
    headless adapter/device request
    WGSL module and compute/render pipeline realization
    resource/layout/bind-group realization
    real format/alignment and cache behavior

G5 correctness
    exact 4,097-element prefix scan/readback
    160x90 Game of Life for 16 steps
    exact live count 2,063
    exact FNV-1a-64 0xBD710B88594CD584
    deterministic compute-to-texture

G5 operations
    submission/readback/upload saturation
    native/web progress and callback proof
    cancellation and pending-work shutdown

G6 graphics and cost
    known-pattern offscreen draw
    compute-generated indirect draw
    direct-WGPU narrow comparisons
    offscreen boids

G7 lifecycle
    surface outcomes
    device generations
    reconstruction matrix

G8 conformance
    cache behavior
    reproducibility facts
    recovery and residual audit

RunenRender
    procedural sky/SDF terrain
    incremental prepared scene
    synthetic volume
    cache/history invalidation
```

## Completed foundation

- workflow execution platform retirement: issues/PRs `#122`, `#123`, and `#124`;
- final repository-surface pruning: issue `#135`, PR `#136`;
- Rust 1.97 and documentation baseline recovery: issues `#150` and `#154`, PR `#155`;
- shared Rust validation adoption: issue `#137`, PR `#138`;
- root architecture foundation alignment: PR `#141`;
- GPU/render architecture correction: issue `#125`, PR `#126`;
- GPU/render S0 inventory: issue `#127`, PR `#128`;
- G1A implementation: issue `#131`, PR `#164`, merge `5bbdab36ae661d99432bfe5d215062c397aac975`;
- G2 decision and implementation: issues `#168` and `#172`, PRs `#171` and `#173`, merge `709aa6aced020ee99405e1e1c3dde7703c77a4d4`;
- G3 decision phase: issue `#174`, PR `#175`, merge `5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8`;
- Runen family operational hardening: issue `#176`, PR `#178`, merge `90d24abb93bff4b1d3f5b4743056bc00ff80d4b6`;
- G3 implementation: issue `#177`, PR `#181`, merge `39d6fe65a334502bdfba0b1a2ce3b365099fcf28`;
- verified-head validation maintenance: issue `#183`, PR `#184`, merge `6bbd341691a34763ef54c8ca059940cac8981265`;
- RunenSDF standalone transfer and Runenwerk duplicate-source retirement: issue `#133`, Runenwerk PRs `#118` and `#157`, and accepted `dornglut/runen-sdf` work.