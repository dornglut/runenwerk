---
title: Repository Family Architecture
description: Canonical repository ownership, dependency direction, integration, release, conformance, and clean-cutover rules for RunenSDF, RunenECS, RunenGPU, RunenRender, RunenUI, and Runenwerk.
status: active
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workspace/planning/active-work.md
  - ../workspace/planning/roadmap.md
  - ../workspace/planning/production-tracks.md
  - ../reports/investigations/repository-family-current-state-investigation.md
  - ../reports/investigations/runenrender-extraction-investigation.md
  - ../design/active/runensdf-extraction-design.md
  - ../design/active/runenecs-extraction-boundary-design.md
  - ../design/active/runengpu-architecture-design.md
  - ../design/active/runenrender-decomposition-design.md
  - ../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../adr/accepted/0015-separate-gpu-execution-from-rendering.md
---

# Repository Family Architecture

## Purpose

Runenwerk is the integration and product repository for a family of independently
usable frameworks. This document owns repository-level boundaries. Track designs
own subsystem APIs and implementation details.

The target family is:

```text
RunenSDF --------------------+
RunenECS --------------------+
RunenUI ---------------------+--> Runenwerk integration --> applications
                             |
RunenRender --> RunenGPU ----+
other GPU consumers -> RunenGPU
```

Framework repositories must not depend on Runenwerk. Runenwerk may depend on
framework public APIs directly or through explicit Runenwerk-owned adapters.

## Current program state

| Track | Current state | What is authorized |
|---|---|---|
| RunenSDF | boundary corrected; standalone transfer handled by its active track | only its bounded SDF phase authority |
| RunenECS | architecture and R1 planning recorded | no Rust implementation without separate activation |
| RunenGPU/RunenRender | split architecture and connector inventory recorded on a parallel branch | documentation/inventory only until S0 and G1 activation gates pass |
| RunenUI | independent repository and workstream | no RunenUI implementation is authorized by this program |

“Complete” means required commands and evidence actually passed. Connector
inspection is source evidence, not command validation.

## Dependency direction

Default:

```text
framework core -> lower-level external libraries only
Runenwerk adapter -> framework + Runenwerk contracts
Runenwerk product -> Runenwerk integration + selected frameworks
```

Direct framework dependencies are exceptional and require accepted ownership
evidence.

Accepted exception:

```text
RunenRender -> RunenGPU
```

This dependency is accepted because RunenGPU owns independently useful lower-level
GPU execution shared by rendering and non-render compute. It is not a Runenwerk
integration responsibility.

Other default rules:

- RunenRender does not require RunenECS, RunenSDF, or RunenUI;
- RunenGPU does not require RunenRender or any domain framework;
- RunenECS does not require Runenwerk geometry, rendering, networking, replay, or
  application lifecycle;
- RunenSDF does not require Runenwerk geometry, world, renderer, ECS, material, or
  GPU types;
- RunenUI does not require RunenGPU, RunenRender, or Runenwerk;
- Runenwerk owns cross-domain composition and initial integration adapters.

## Repository missions

### RunenSDF

RunenSDF owns reusable signed-field mathematics, validated field vocabulary,
numerical policy, spatial bounds, field composition, capabilities, and CPU
reference queries.

It does not own world streaming, ECS components, renderer passes, GPU resources,
material semantics, or Runenwerk product policy.

### RunenECS

RunenECS owns entity/component/resource lifecycle semantics, storage/query
contracts, deferred mutation, system access, explicit reflection, and ECS-local
integration with a neutral scheduler package.

It does not own general spatial indexing, engine frame policy, rendering
extraction, transport, networking, rollback, replay, scene management, or world
streaming.

### RunenGPU

RunenGPU owns general GPU execution.

Required package candidates:

```text
runengpu_core
runengpu_wgpu
```

RunenGPU may own:

- GPU runtime identities;
- normalized capabilities;
- backend-neutral resources, access, lifetimes, and work graphs;
- shader/pipeline admission and realization contracts;
- compute/render/copy execution;
- headless execution, upload, readback, and completion;
- low-level surfaces and device outcomes;
- GPU diagnostics and provenance.

RunenGPU does not own rendering, materials, light transport, simulations, SDF,
UI, ECS, world generation, windows, or product recovery.

`runengpu_core` contains no WGPU/Winit/Runenwerk/renderer/domain dependency.
`runengpu_wgpu` contains WGPU realization but no Winit/Runenwerk/renderer/domain
semantics.

### RunenRender

RunenRender owns image formation over RunenGPU.

Required package candidates:

```text
runenrender_core
runenrender_gpu
```

RunenRender may own:

- prepared render scenes and contributions;
- views and logical targets;
- render providers/interactions;
- materials, media, emitters, and environments;
- visibility, transport, radiance caches, and reconstruction;
- overlays, output color intent, and render diagnostics;
- render-specific lowering into RunenGPU work.

RunenRender does not own concrete WGPU devices, queues, surfaces, generic
resources, ECS extraction, source authoring, simulations, SDF mathematics, UI
semantics, windows, or product policy.

No `runenrender_wgpu` package is planned.

### RunenUI and rendering

RunenUI owns semantic UI, state, layout/style/control behavior, accessibility, hit
testing, text shaping, and renderer-neutral paint output.

RunenRender owns image formation and may consume a prepared overlay contribution.
RunenGPU owns execution of any GPU workloads used by a chosen UI backend.

Initial integration remains Runenwerk-owned:

```text
RunenUI paint output
    -> Runenwerk adapter
    -> RunenRender overlay contribution
    -> RunenGPU execution
```

RunenUI may retain standalone backends. Its core/runtime do not depend on
RunenRender or RunenGPU.

### Runenwerk

Runenwerk owns:

- engine and application lifecycle;
- frame/tick and domain scheduling;
- native windows and event-loop policy;
- GPU context creation and product capability selection;
- ECS/domain extraction;
- scene, world, material, SDF, editor, simulation, and UI adapters;
- shader source discovery and hot-reload policy;
- product feature and quality composition;
- diagnostics presentation, artifact policy, and recovery;
- cross-repository integration and runtime evidence.

## Adapter rule

A framework remains useful without its Runenwerk adapter. An adapter may depend on
Runenwerk and the framework(s) it translates, but owners do not depend back on the
adapter.

Adapters translate:

- identities;
- inputs and outputs;
- lifecycles and generations;
- diagnostics and provenance;
- resource ownership.

They must not copy algorithms, mirror source, create parallel writable authority,
expose broad compatibility facades, or hide cycles.

## No shared-core magnet

Do not create `RunenCore`, `foundation/meta`, a universal ID repository, a universal
diagnostics repository, or a generic plugin framework merely to simplify
extraction.

RunenGPU is not a universal engine core. It owns one bounded responsibility: GPU
execution.

Values remain with the repository whose invariants they express. Adapters map
repository-local values explicitly.

## Identity and diagnostics

Each repository owns opaque identities for its concepts:

```text
runensdf.*
runenecs.*
runengpu.*
runenrender.*
runenui.*
runenwerk.*
```

GPU resource/work identities remain distinct from renderer provider/material/view
identities and from Runenwerk/ECS identities.

Process-local IDs are not silently serialized. Persisted identities require an
explicit owner, version, validation, and migration contract.

Adapters preserve upstream diagnostic identity and add integration context rather
than replacing failures with strings.

## Toolchain and release policy

Every extracted repository defines:

- Rust edition and MSRV;
- formatting, locked tests, and denied-warning policy;
- package publication and API stability state;
- license, security policy, and provenance;
- dependency and feature policy.

Before stable publication, Runenwerk uses an exact commit or exact prerelease.
Moving branch dependencies are forbidden. Published packages follow semantic
versioning and Runenwerk records compatibility.

## Persisted formats

Rust API compatibility and persisted-format compatibility are separate.

Every persisted source, artifact, trace, replay, cache, or wire format names:

- owning repository;
- identifier/version;
- validation and compatibility policy;
- migration behavior;
- deterministic encoding requirements where applicable.

Internal runtime packets and GPU handles are not stable formats by default.

## Conformance

Every framework requires:

- unit, negative, and property tests for owned invariants;
- at least one public downstream consumer;
- stable and MSRV validation;
- documentation/link validation;
- metadata, dependency, license, and provenance checks.

Runenwerk owns cross-repository integration. Evidence distinguishes deterministic
source/contract proof from environment-dependent GPU, window, and runtime proof.

Additional RunenGPU proof:

- headless compute;
- offscreen graphics;
- one render and one non-render consumer sharing a context;
- no public WGPU leakage from core;
- structured surface/device outcomes;
- no duplicate GPU path.

Additional RunenRender proof:

- prepared-scene and contribution boundaries;
- no host/ECS reach-back;
- RunenGPU-only execution;
- headless offscreen output;
- current behavior parity;
- no duplicate renderer path.

## Clean-cutover rule

Each extraction proceeds:

1. inventory source and all consumers;
2. accept a decision-complete boundary;
3. correct and prove the boundary inside Runenwerk;
4. prove independent conformance and anti-cheating use;
5. create/populate the external repository;
6. validate standalone and select an exact revision;
7. pin Runenwerk and migrate consumers;
8. delete original implementation;
9. remove temporary seams;
10. close provenance and compatibility evidence.

Temporary duplication may exist only on an unmerged extraction branch.
No forwarding package, source mirror, submodule, moving branch dependency, or
parallel writable authority survives cutover.

## Track sequencing

```text
RunenSDF
    active standalone transfer track

RunenECS
    ordered internal repairs before extraction

RunenGPU
    complete S0 inventory
    -> G1-G9 internal decomposition/conformance
    -> external transfer and Runenwerk cutover

RunenRender
    read-only planning may overlap
    -> implementation after accepted RunenGPU boundary
    -> internal decomposition/conformance
    -> external transfer and Runenwerk cutover

RunenUI
    independent workstream
```

Only one track at a time owns shared manifests, lockfiles, root summaries, and
canonical planning merges. Track-specific investigation and design may proceed in
parallel.

## Extraction gates

No external source transfer begins until a track proves:

- complete source and consumer inventory;
- decision-complete ownership and public API direction;
- no unresolved dependency cycle;
- independent downstream conformance;
- validation and versioning policy;
- diagnostics, identity, and persisted-format decisions;
- exact move/stay/redesign/delete map;
- clean-cutover, provenance, and rollback strategy.

RunenGPU and RunenRender additionally require internal anti-cheating proof:
Runenwerk must consume separated packages through the same public boundaries
intended for external users, with no private reach-through or duplicate path.
