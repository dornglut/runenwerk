---
title: Repository Family Architecture
description: Canonical repository ownership, dependency direction, integration, operational contracts, release, conformance, and clean-cutover rules for the Runen framework family.
status: active
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-27
related_docs:
  - ../workspace/planning/active-work.md
  - ../workspace/planning/roadmap.md
  - ../reports/investigations/repository-family-current-state-investigation.md
  - ../reports/investigations/runenrender-extraction-investigation.md
  - ../reports/investigations/runengpu-g3-access-work-graph-investigation.md
  - ../reports/investigations/runengpu-industry-comparison.md
  - ../reports/investigations/runengpu-runenrender-application-domain-fit.md
  - ../design/active/runensdf-extraction-design.md
  - ../design/active/runenecs-extraction-boundary-design.md
  - ../design/active/runengpu-architecture-design.md
  - ../design/active/runengpu-g3-access-work-graph-design.md
  - ../design/active/runenrender-decomposition-design.md
  - ../design/active/runen-family-operational-hardening-design.md
  - ../workspace/specs/pt-runengpu-g3-access-work-graph.ron
  - ../adr/accepted/0014-repository-family-extraction-boundaries.md
  - ../adr/accepted/0015-separate-gpu-execution-from-rendering.md
---

# Repository Family Architecture

## Purpose

Runenwerk is the integration and product repository for independently useful
framework repositories. This document owns repository-level boundaries, dependency
direction, integration policy, family-wide operational doctrine, and clean cutovers.
Framework designs own subsystem contracts.

## Repository family

```text
product       repository                    package       crate
RunenSDF      dornglut/runen-sdf            runen-sdf     runen_sdf
RunenECS      target dornglut/runen-ecs     governed separately
RunenGPU      target dornglut/runen-gpu     runen-gpu     runen_gpu
RunenRender   target dornglut/runen-render  runen-render  runen_render
RunenUI       dornglut/runen-ui             existing workspace topology
Runenwerk     dornglut/runenwerk            workspace      integration/product
```

RunenGPU and RunenRender each begin with one public package. Internal modules carry
responsibility boundaries until a real second consumer, backend, release unit, ABI,
or compile-time boundary proves another package is needed.

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

ADR 0015 accepts one direct foundational framework dependency:

```text
RunenRender -> RunenGPU
```

No dependency cycle is allowed.

## Current program state

| Framework | Current state | Authorized work |
|---|---|---|
| RunenSDF | standalone authority in `dornglut/runen-sdf`; duplicate Runenwerk source retired through issue `#133` / PR `#157` | standalone roadmap and independently accepted adapters only |
| RunenECS | internal ownership and safety repair required | separately bounded investigation/design/repair |
| RunenGPU | S0, G1A, and G2 complete; G3 planning accepted through issue `#174` / PR `#175` at merge `5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8` | issue `#176` operational hardening; issue `#177` queued and blocked until `#176` acceptance |
| RunenRender | architecture corrected to consume RunenGPU | S0/design and documentation hardening only; internal proof waits for accepted external RunenGPU cutover |
| RunenUI | independent repository/workstream | governed in RunenUI |

Current source location is implementation evidence, not permanent ownership.

## Repository missions

### RunenSDF

Owns reusable signed-field mathematics, validated field vocabulary, numerical policy,
bounds, composition, transforms, capabilities, and CPU reference queries.

Does not own world streaming, ECS, rendering, GPU resources, materials, or product
policy.

### RunenECS

Owns entity/component/resource lifecycle, storage/query semantics, deferred structural
mutation, system access contracts, explicit reflection, and ECS-local scheduling
integration.

Does not own general spatial indexing, engine frame/tick policy, rendering extraction,
networking, replay, world streaming, or product lifecycle.

### RunenGPU

Owns normalized capabilities, contexts, logical resources, access, graph-time
initialization, hazards, generic work, submissions, uploads/readback, low-level
surfaces, WGPU realization, backend outcomes, progress facts, pressure outcomes,
device generations, and GPU diagnostics.

Does not own image formation, simulation algorithms, field mathematics, ECS, UI,
windows/event loops, shader filesystem policy, artifact codecs, or product recovery.

### RunenRender

Owns prepared render scenes, views, providers/interactions, materials/media,
emitters/environments, visibility, transport, radiance caches, history,
reconstruction, overlays, color, presentation intent, and lowering into RunenGPU
workloads.

Does not own WGPU, general GPU execution, ECS extraction, field mathematics, UI
semantics, native windows, shader file watching, vertical-domain products, or
Runenwerk lifecycle.

### RunenUI

Owns semantic UI, state/actions, focus/accessibility, layout/style/text, hit testing,
and renderer-neutral paint output. It does not depend on RunenRender or RunenGPU by
default.

### Runenwerk

Owns:

- application and engine lifecycle;
- frame/tick and domain scheduling;
- windows/event loops and native host policy;
- ECS and domain extraction;
- scene, world, material-authoring, SDF, UI, editor, simulation, and product adapters;
- shader source discovery/revision/watch/reload policy;
- product capability/quality selection;
- cross-framework composition and tested compatibility;
- product recovery decisions;
- diagnostics presentation and support artifacts;
- reproducibility bundles and persisted capture schemas;
- offline job sequencing and artifact encoding;
- application binaries and tools.

## Adapter rule

A framework must remain useful without its Runenwerk adapter.

Adapters translate:

- identities;
- prepared inputs and outputs;
- lifecycle and generation facts;
- diagnostics and provenance;
- resource/source ownership;
- product policy into accepted framework requirements.

Adapters must not:

- duplicate framework algorithms;
- mirror authoritative source;
- introduce writable parallel authority;
- expose broad compatibility facades;
- hide dependency cycles;
- preserve private reach-through after cutover;
- silently convert structured pressure/failure into logs.

Keep a bridge in Runenwerk until independent consumers prove stable reusable
ownership.

## Family-wide operational doctrine

### Accepted-work integrity

Once a framework accepts work, it must eventually report exactly one terminal
outcome. Accepted work, completion notifications, requested artifacts, and
non-discardable source state are never silently dropped.

### Structured pressure

Every bounded queue, staging arena, readback pool, retained cache, history set, and
capture buffer returns a structured pressure outcome or an explicitly bounded wait.

Permitted pressure strategies are:

```text
reject with facts
wait with an explicit bound
shed discardable derived work
request caller-owned quality reduction
```

Unbounded implicit growth is not a default policy.

### Derived caches

Derived caches are non-authoritative, discardable, reconstructable, keyed by all
correctness facts, source-generation-bound, validated before reuse, and versioned
when persisted. A cache hit changes cost, not semantics.

### Compatibility manifest

Runenwerk owns a tested compatibility manifest for the exact framework and adapter
revisions it integrates. It may include WGPU/backend family and persisted artifact
schema facts.

The manifest does not create a shared `RunenCore` package and is not imported back
into framework repositories as product policy.

### Recovery ownership

Frameworks classify loss, invalidate generations, and report reconstruction facts.
Runenwerk decides whether a product retries, recreates, degrades, pauses, exits, or
asks the user for action.

Source-backed, externally reconstructed, and non-reconstructable values remain
explicitly distinct.

### Reproducibility bundle

Runenwerk may assemble a versioned namespaced bundle containing framework revisions,
capabilities, device/backend facts where permitted, prepared-work diagnostics,
scene/view generations, seeds, fixed-time inputs, provenance, artifacts, checksums,
and privacy/redaction metadata.

Runtime handles, pointers, memory addresses, and unversioned diagnostic strings are
never persisted as authority.

## RunenUI rendering relationship

```text
RunenUI paint scene
    -> Runenwerk bridge
        -> RunenRender overlay contribution
            -> RunenGPU work
```

The bridge consumes accepted paint primitives, not widget state or actions.
RunenRender does not shape text or perform UI hit testing.

## RunenSDF rendering/GPU relationship

RunenSDF remains CPU/backend-neutral. A Runenwerk or future reusable adapter may
translate accepted field contracts into render providers or GPU work while preserving
numerical and capability semantics. RunenSDF never depends back on the adapter,
RunenRender, or RunenGPU.

## One-package initial rule

Do not create speculative package trees merely to draw architecture boundaries.
Initial targets remain:

```text
runen-sdf
runen-ecs
runen-gpu
runen-render
runen-ui
```

Additional packages require a proven independent dependency subset, second backend,
release/versioning unit, required proc macro, platform/MSRV separation, or externally
used conformance package.

## No shared-core magnet

Do not create `RunenCore`, a universal ID repository, universal diagnostics package,
or generic plugin/meta-framework to simplify extraction. Values live with the
repository whose invariants they express. Adapters map values explicitly.

## Identity and diagnostics

Each framework owns opaque runtime identities for its concepts. Runtime IDs are not
silently serialized or transmitted. Stable formats require explicit identifiers,
versions, validation, and migration.

Diagnostics are repository-namespaced:

```text
runensdf.*
runenecs.*
runengpu.*
runenrender.*
runenui.*
runenwerk.*
```

Adapters add integration context instead of replacing failures with strings.

## Toolchain and release policy

Every extracted repository defines:

- Rust edition and declared MSRV;
- formatting, locked tests, strict Clippy, rustdoc, and docs validation;
- publication/API stability state;
- license and source provenance;
- dependency and feature policy;
- public downstream conformance.

Before stable publication, Runenwerk uses an exact commit or exact pre-release
version. Moving branch dependencies are forbidden.

## Persisted formats

Rust API compatibility and persisted-format compatibility are separate.

Every persisted source, artifact, trace, replay, cache, compatibility manifest,
capture bundle, or wire format names:

- owning repository;
- format identifier and version;
- validation/compatibility policy;
- migration behavior;
- deterministic encoding requirements where relevant;
- privacy/redaction policy where relevant.

Internal runtime packets and IDs are not stable formats by default.

## Conformance

Every framework requires:

- unit, negative, and property/invariant tests for owned semantics;
- at least one downstream public-API consumer;
- stable and declared-MSRV validation;
- formatting, locked tests, strict Clippy, rustdoc, docs, metadata, license, and
  provenance checks;
- no Runenwerk source include, mirror, submodule, or compatibility package;
- operational pressure, shutdown, and recovery proof once the framework owns those
  capabilities.

Runenwerk owns cross-repository integration tests, compatibility-manifest proof,
product recovery proof, and persisted reproducibility/capture validation.

Evidence distinguishes deterministic contract proof from environment-dependent
GPU/window/runtime proof.

## Performance and anti-cheating

A framework boundary is not justified by architecture diagrams alone.

RunenGPU and RunenRender must characterize:

- CPU preparation/validation cost;
- allocations and memory high-water marks;
- cold/warm shader and pipeline cost;
- staging/readback pressure;
- GPU timing where supported;
- full versus incremental scene preparation;
- derived-cache behavior;
- direct narrow alternatives for representative proofs.

No consumer may bypass the public boundary to make benchmarks appear favorable.
Performance budgets require separately accepted controlled specifications.

## Clean-cutover rule

Each extraction proceeds:

1. inventory current source and every consumer;
2. accept a decision-complete boundary;
3. correct and prove the future public boundary inside Runenwerk;
4. establish independent conformance;
5. create/populate the external repository;
6. pin Runenwerk to an exact revision;
7. migrate active consumers;
8. delete the original implementation;
9. remove temporary migration seams;
10. update authority and record closeout.

Temporary duplication may exist only on an unmerged branch. Compatibility packages,
forwarding namespaces, mirrors, submodules, and branch dependencies do not survive a
completed cutover.

If Runenwerk has no active consumer, removing the internal implementation does not
require adding an unused external dependency.

## GPU/render sequencing

```text
S0 inventory
-> internal RunenGPU G1A-G8 proof
-> external RunenGPU cutover
-> internal RunenRender proof on RunenGPU
-> external RunenRender cutover
-> reusable adapter review
-> advanced renderer work
```

RunenGPU moves before RunenRender because the renderer depends on it. Advanced
renderer/provider work must not harden accidental mixed ownership before the
foundational cutovers.

## Extraction gates

No external transfer begins until the track proves:

- complete source and consumer inventory;
- decision-complete public ownership/API direction;
- no unresolved dependency cycle;
- independent downstream conformance;
- validation and versioning policy;
- identity, diagnostics, persisted-format, pressure, and recovery decisions;
- exact move/stay/redesign/delete map;
- provenance and clean-cutover strategy;
- current exact-head CI;
- acceptable measured boundary overhead;
- no private reach-through or duplicate execution path.
