---
title: Repository Family Architecture
description: Canonical repository ownership, dependency direction, integration, release, conformance, and clean-cutover rules for the Runen framework family.
status: active
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-21
related_docs:
  - ../workspace/planning/active-work.md
  - ../workspace/planning/roadmap.md
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

Runenwerk is the integration and product repository for independently useful
framework repositories. This document owns repository-level boundaries,
dependency direction, and cutover rules. Framework designs own subsystem contracts.

## Repository family

```text
product       repository                 package       crate
RunenSDF      Crystonix/runen-sdf        runen-sdf     runen_sdf
RunenECS      Crystonix/runen-ecs        runen-ecs     runen_ecs
RunenGPU      Crystonix/runen-gpu        runen-gpu     runen_gpu
RunenRender   Crystonix/runen-render     runen-render  runen_render
RunenUI       Crystonix/runen-ui         runen-ui      runen_ui
Runenwerk     Crystonix/runenwerk        workspace      integration/product
```

Each new framework repository begins with one public package. Internal modules
carry responsibility boundaries until a real second consumer, backend, release
unit, ABI, or compile-time boundary proves another package is required.

Framework repositories do not depend on Runenwerk. Runenwerk may depend on exact
framework revisions directly or through explicit Runenwerk-owned adapters.

## Dependency direction

```text
RunenSDF ----+
RunenECS ----+--> Runenwerk adapters/integration --> applications
RunenUI -----+
                  |
                  +--> RunenRender --> RunenGPU
                  +--> non-render RunenGPU workloads
```

The default is framework independence. ADR 0015 accepts one direct foundational
framework dependency:

```text
RunenRender -> RunenGPU
```

RunenRender needs general GPU execution. RunenGPU remains independently useful for
compute, simulations, field realization, procedural tools, bakers, and offscreen
workloads.

No dependency cycle is allowed.

## Current program state

| Framework | Current state | Authorized work |
|---|---|---|
| RunenSDF | standalone transfer complete at `d52badefc640d6dc6dcdd40268af3aea1bb8eefe`; Runenwerk clean cutover not recorded complete | separately reviewed consumer audit/cutover only |
| RunenECS | internal ownership and safety repair required | investigation/design and bounded accepted repairs |
| RunenGPU | architecture accepted; current implementation still mixed into renderer | S0 inventory only until first phase is specified |
| RunenRender | architecture corrected to consume RunenGPU | S0 inventory and design only |
| RunenUI | independent repository/workstream | governed in RunenUI |

Current source location is implementation evidence, not permanent ownership.

## Repository missions

### RunenSDF

Owns reusable signed-field mathematics, validated field vocabulary, numerical
policy, bounds, composition, transforms, capabilities, and CPU reference queries.

Does not own world streaming, ECS, rendering, GPU resources, material semantics,
or Runenwerk product policy.

### RunenECS

Owns entity/component/resource lifecycle, storage/query semantics, deferred
structural mutation, system access contracts, explicit reflection, and ECS-local
scheduling integration.

Does not own general spatial indexing, engine frame/tick policy, rendering
extraction, networking, replay, world streaming, or product lifecycle.

### RunenGPU

Owns validated GPU capabilities, contexts, resources, access/lifetimes, hazards,
workloads, submissions, uploads/readback, low-level surfaces, WGPU realization,
backend outcomes, and GPU diagnostics.

Does not own image formation, simulation algorithms, field mathematics, ECS, UI,
windows/event loops, shader filesystem policy, or product recovery.

### RunenRender

Owns prepared render scenes, views, providers/interactions, materials/media,
emitters/environments, visibility, transport, radiance caches, history,
reconstruction, overlays, color, presentation intent, and lowering into RunenGPU
workloads.

Does not own WGPU directly, general GPU execution, ECS extraction, field/SDF
mathematics, UI semantics, native windows, shader file watching, or Runenwerk
lifecycle.

### RunenUI

Owns semantic UI, state/actions, focus/accessibility, layout/style/text, hit
testing, and renderer-neutral paint output.

RunenUI does not depend on RunenRender or RunenGPU by default. Standalone backends
remain valid.

### Runenwerk

Owns:

- application and engine lifecycle;
- frame/tick and domain scheduling;
- windows/event loops and native host policy;
- ECS and domain extraction;
- scene, world, material-authoring, SDF, UI, editor, simulation, and product
  adapters;
- shader source discovery/revision/watch/reload policy;
- product capability/quality selection;
- cross-framework composition;
- diagnostics presentation, artifacts, recovery, and runtime evidence;
- application binaries and tools.

## RunenUI rendering relationship

```text
RunenUI paint scene
    -> Runenwerk bridge
    -> RunenRender overlay contribution
    -> RunenGPU workloads
```

The bridge consumes accepted paint primitives, not widget state or actions.
RunenRender does not shape text or perform UI hit testing. RunenUI remains
independently usable.

## RunenSDF rendering/GPU relationship

RunenSDF remains CPU/backend-neutral. A Runenwerk or future reusable adapter may
translate accepted field contracts into render providers or GPU workloads.

The adapter must preserve numerical/capability semantics. RunenSDF never depends
back on the adapter, RunenRender, or RunenGPU.

## Adapter rule

A framework must remain useful without its Runenwerk adapter.

Adapters translate:

- identities;
- prepared inputs and outputs;
- lifecycle facts;
- diagnostics and provenance;
- resource/source ownership.

Adapters must not:

- duplicate framework algorithms;
- mirror source;
- introduce writable parallel authority;
- expose broad compatibility facades;
- hide dependency cycles;
- preserve private reach-through after cutover.

Keep a bridge in Runenwerk until independent consumers prove it has stable reusable
ownership.

## One-package initial rule

Do not create speculative package trees merely to draw architecture boundaries.

Initial targets are:

```text
runen-sdf
runen-ecs
runen-gpu
runen-render
runen-ui
```

Additional packages require evidence such as:

- an independently reusable dependency subset;
- a second backend with separate dependency pressure;
- a proc-macro that must be separately compiled and is proven necessary;
- a distinct release/versioning unit;
- no-std/platform separation;
- a stable test/conformance package used externally.

Module separation and private implementation boundaries are preferred before
package extraction.

## No shared-core magnet

Do not create `RunenCore`, a universal ID repository, a universal diagnostics
repository, or a generic plugin/meta-framework to make extraction convenient.

Values belong with the repository whose invariants they express. Adapters map
repository-local values explicitly.

## Identity and diagnostics

Each framework owns opaque runtime identities for its concepts. Runtime IDs are not
silently serialized or transmitted. Stable formats require explicit identifiers,
validation, versioning, and migration.

Diagnostics are repository-namespaced and preserve upstream identity:

```text
runensdf.*
runenecs.*
runengpu.*
runenrender.*
runenui.*
runenwerk.*
```

Adapters add integration context rather than replacing failures with strings.

## Toolchain and release policy

Every extracted repository defines:

- Rust edition and declared MSRV;
- formatting, locked tests, strict Clippy, rustdoc, and documentation validation;
- publication/API stability state;
- license and source provenance;
- dependency and feature policy;
- public downstream conformance.

Before stable publication, Runenwerk uses an exact commit or exact pre-release
version. Moving branch dependencies are forbidden.

## Persisted formats

Rust API compatibility and persisted-format compatibility are separate.

Every persisted source, artifact, trace, replay, cache, or wire format names:

- owning repository;
- format identifier/version;
- validation/compatibility policy;
- migration behavior;
- deterministic encoding requirements where relevant.

Internal runtime packets and IDs are not stable formats by default.

## Conformance

Every framework repository requires:

- unit, negative, and property/invariant tests for owned semantics;
- at least one downstream public-API consumer;
- stable and declared-MSRV validation;
- formatting, locked tests, strict Clippy, rustdoc, docs, metadata, license, and
  provenance checks;
- no Runenwerk source include, mirror, submodule, or compatibility package.

Runenwerk owns cross-repository integration tests. Evidence distinguishes
deterministic source/contract proof from environment-dependent GPU/window/runtime
proof.

## Clean-cutover rule

Each extraction proceeds:

1. inventory current source and every consumer;
2. accept a decision-complete boundary;
3. correct and prove the public boundary inside Runenwerk;
4. establish independent conformance;
5. create/populate the external repository;
6. pin Runenwerk to an exact revision;
7. migrate active consumers;
8. delete the original implementation;
9. remove temporary migration seams;
10. update authority and record closeout.

Temporary duplication may exist only on an unmerged branch. Compatibility
packages, forwarding namespaces, mirrors, submodules, and branch dependencies do
not survive a completed cutover.

If Runenwerk has no active consumer, removing the internal implementation does not
require adding an unused external dependency.

## GPU/render sequencing

The current combined renderer requires:

```text
S0 complete inventory
-> internal RunenGPU proof
-> external RunenGPU cutover
-> internal RunenRender proof on RunenGPU
-> external RunenRender cutover
-> reusable adapter review
-> advanced renderer work
```

RunenGPU moves before RunenRender because the renderer depends on it. Advanced
field-ray transport must not harden accidental current ownership before the
foundational cutovers.

## Extraction gates

No external transfer begins until the track proves:

- complete source and consumer inventory;
- decision-complete public ownership/API direction;
- no unresolved dependency cycle;
- independent downstream conformance;
- validation and versioning policy;
- identity, diagnostics, and persisted-format decisions;
- exact move/stay/redesign/delete map;
- provenance and clean-cutover strategy;
- current exact-head CI.

RunenGPU and RunenRender additionally require internal anti-cheating proof:
Runenwerk must consume the future public boundary with no private reach-through or
duplicate path.
