---
title: Roadmap
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workflow-lifecycle.md
  - ../routines/track-orchestration-routine.md
  - ../specs/phase-implementation-spec.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runensdf-extraction-design.md
  - ../../reports/closeouts/pt-runensdf-002-boundary-correction-closeout.md
  - ../../design/active/runenecs-boundary-repair-execution-plan.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../reports/investigations/runenrender-extraction-investigation.md
  - ../specs/pt-runengpu-g1-identities-errors.ron
  - ./active-work.md
  - ./production-tracks.md
  - ./completed-work.md
  - ./decision-register.md
---

# Roadmap

## Program destination

```text
RunenSDF --------------------+
RunenECS --------------------+
RunenUI ---------------------+--> Runenwerk integration --> applications
                             |
RunenRender --> RunenGPU ----+
other GPU consumers -> RunenGPU
```

Runenwerk remains the integration and product repository. Framework repositories
do not depend on Runenwerk. RunenRender depends on the lower-level RunenGPU
framework. RunenUI remains an independent peer.

## Current state

```text
Repository-family charter       complete through PR #109
RunenSDF boundary correction    complete through PR #116
RunenSDF standalone transfer    active in its separate thread/branch
RunenECS                         R1 planned; no implementation authorization
GPU/render split architecture   active planning on docs/gpu-render-split-architecture
Old RunenRender R1              retired before implementation
RunenGPU G1                     planning-only; blocked by S0 local gate and activation
RunenRender implementation      blocked by accepted RunenGPU boundary
```

The GPU/render architecture branch must be rebased after the active SDF planning
PR changes shared planning authority. No SDF lifecycle fact in this branch may
silently replace the SDF track's newer truth.

## Immediate sequence

```text
1. Continue the separately authorized RunenSDF transfer track.
2. Review and merge PT-GPU-RENDER-SPLIT-001 after rebasing shared planning files.
3. Complete the local S0 GPU/render file, consumer, command, shader, surface,
   identity, and benchmark inventory.
4. Update and separately activate PT-RUNENGPU-G1.
5. Complete G1-G9 internal RunenGPU decomposition and conformance.
6. Transfer RunenGPU externally and cut Runenwerk over cleanly.
7. Complete R1-R8 internal RunenRender decomposition on external RunenGPU.
8. Transfer RunenRender externally and cut Runenwerk over cleanly.
9. Add RunenUI, RunenSDF, and simulation adapters where justified.
10. Begin the advanced field-ray transport roadmap.
```

## PT-GPU-RENDER-SPLIT-001

State: `active-planning`

Implementation authorization: documentation and investigation only

Mission:

- establish RunenGPU as a separate lower-level framework;
- change RunenRender to use RunenGPU instead of owning WGPU;
- classify the current renderer/GPU module families and identities;
- replace the old renderer-only decomposition plan;
- retire the unimplemented RunenRender R1 specification;
- prepare one planning-only RunenGPU G1 contract.

Exit gate:

- ADR 0015 accepted in the branch;
- RunenGPU and revised RunenRender designs aligned;
- canonical repository/planning/root summaries aligned;
- old R1 removed from active authority;
- module-family connector inventory recorded;
- docs validation and diff hygiene pass;
- branch rebased against current main after active SDF planning changes.

This phase authorizes no Rust, Cargo, shader, WGPU, or external-repository source
change.

## S0 — Complete GPU/render inventory

Before implementation, produce:

- complete local file list;
- complete import and consumer map;
- every current renderer/GPU ID, allocator, raw conversion, and stable-use map;
- every shader, pipeline, macro, example, test, and benchmark map;
- device/surface/window/drop-order control flow;
- CPU planning, GPU, surface, and performance command baseline;
- exact file-level move/stay/redesign/delete matrix.

Required command families are recorded in the investigation and execution plan.

S0 completion permits G1 activation planning. It does not itself authorize code.

## RunenGPU internal phases

| Phase | Purpose | State |
|---|---|---|
| `G1` | GPU-owned runtime identities, structured errors, dependency guards | planning-only; blocked by S0 and activation |
| `G2` | resource descriptors, access, initialization, ownership, lifetimes | blocked by G1 |
| `G3` | bounded compute/render/copy work fragments and deterministic work graph | blocked by G2 |
| `G4` | shader, pipeline, parameter, byte/binding, and optional macro ABI boundary | blocked by G3 |
| `G5` | headless WGPU device, compute execution, upload, submission, readback | blocked by G4 |
| `G6` | offscreen render/copy execution and compute-to-render proof | blocked by G5 |
| `G7` | surfaces, generations, acquire/present, retirement, device-loss facts | blocked by G6 |
| `G8` | one render and one non-render consumer sharing public GPU seams | blocked by G7 |
| `G9` | internal public-boundary, validation, platform, and performance conformance | blocked by G8 |

Only G1 may receive the next implementation specification. Later specs are written
just in time after the preceding phase closes.

## External RunenGPU phases

| Phase | Purpose | State |
|---|---|---|
| `GX1` | transfer accepted internal packages to `Crystonix/runen-gpu`; validate standalone and public downstream use | blocked by G9 |
| `GX2` | pin exact revision, migrate Runenwerk consumers, delete internal GPU source, remove migration seams | blocked by GX1 |

The completed cutover leaves one external GPU authority and no duplicate context,
allocator, resource namespace, or execution path.

## RunenRender internal phases

RunenRender implementation waits for the accepted external RunenGPU cutover.
Read-only planning may continue earlier.

| Phase | Purpose | State |
|---|---|---|
| `R1` | renderer-semantic identities and structured errors, distinct from GPU handles | blocked by GX2 |
| `R2` | immutable prepared render scene and contribution lifecycle | blocked by R1 |
| `R3` | semantic render planning separated from GPU work planning | blocked by R2 |
| `R4` | render-specific GPU realization through RunenGPU only | blocked by R3 |
| `R5` | renderer material/shader interface versus host authoring/reload boundary | blocked by R4 |
| `R6` | logical targets, overlays, output color, and presentation boundary | blocked by R5 |
| `R7` | migrate scene/world/material/SDF/UI/editor/procedural/debug adapters and prove parity | blocked by R6 |
| `R8` | standalone/public-boundary, headless, parity, and performance conformance | blocked by R7 |

## External RunenRender phases

| Phase | Purpose | State |
|---|---|---|
| `RX1` | transfer `runenrender_core` and `runenrender_gpu`; validate standalone/downstream | blocked by R8 |
| `RX2` | pin exact revision, migrate adapters, delete internal framework source, close provenance | blocked by RX1 |

## Integration adapter phases

### RunenUI

```text
RunenUI paint output
    -> Runenwerk-owned adapter
    -> RunenRender overlay contribution
    -> RunenGPU execution
```

Blocked until the renderer-neutral RunenUI paint protocol and both framework
boundaries are stable. Neither core depends on the other.

### RunenSDF

```text
RunenSDF field contract
    -> Runenwerk-owned adapter
    -> RunenRender provider
    -> optional RunenGPU realization
```

The adapter initially remains in Runenwerk. A bridge package requires a second
independent host.

### Simulations and procedural systems

Fluid, wind, vegetation, fire, and generation systems may publish RunenGPU work
and RunenRender providers through explicit adapters. Their algorithms and source
state remain domain-owned.

## Advanced renderer roadmap

After both framework cutovers:

```text
F1 reference implicit solid renderer
F2 shell, fiber, liquid, volume, and analytic providers
F3 unified many-light direct transport
F4 sparse directional world-space radiance cache
F5 lobe-separated reconstruction and bounded history
F6 preview/standard/high/ultra/reference quality continuum
F7 endless-world visibility and transport horizons
F8 material, transport, and display stylization
F9 world/vegetation/water/character production proofs
F10 authoring integration, profiling, capture, and hardening
```

These features extend the accepted framework. They do not justify delaying the
boundary extraction or adding more experimental systems to the old monolith.

## RunenECS track

The accepted sequence remains independent:

```text
R1 entity identity and structured errors
R2 atomic structural mutation
R3 query and SystemParam unsafe boundaries
R4 explicit reflection and macro migration
R5 remove spatial and geometry ownership
R6 messaging split
R7 change, ownership, and networking separation
R8 neutralize runen_schedule
R9 standalone conformance and performance baseline
```

No RunenECS Rust implementation is authorized by the GPU/render split.

## Parallelism policy

Allowed:

- independently authorized RunenSDF work;
- local S0 GPU/render inventory and validation;
- read-only future-phase design;
- independent RunenUI work;
- benchmark and command discovery.

Serialized or explicitly coordinated:

- canonical planning/root summaries;
- Cargo manifests and lockfile;
- current renderer/GPU identity files;
- WGPU ownership changes;
- source transfers and cutovers.

Forbidden without phase authority:

- structural GPU or renderer implementation;
- external RunenGPU/RunenRender source transfer;
- duplicate GPU/renderer paths;
- implementing the retired old RunenRender R1;
- broad field/path-tracing rewrite in the current plugin;
- direct RunenUI/RunenRender core dependency;
- compatibility packages, source mirrors, or a universal shared core.
