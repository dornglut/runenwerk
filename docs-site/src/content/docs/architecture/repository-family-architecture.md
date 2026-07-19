---
title: Repository Family Architecture
description: Canonical ownership, dependency, release, conformance, and clean-cutover rules for extracting RunenSDF, RunenECS, and RunenRender while keeping Runenwerk as the integration product.
status: active
owner: workspace
layer: architecture
canonical: true
last_reviewed: 2026-07-19
related_docs:
  - ../workspace/planning/active-work.md
  - ../workspace/planning/roadmap.md
  - ../workspace/planning/production-tracks.md
  - ../reports/investigations/repository-family-current-state-investigation.md
  - ../design/active/runensdf-extraction-design.md
  - ../design/active/runenecs-extraction-boundary-design.md
  - ../design/active/runenrender-decomposition-design.md
  - ../adr/accepted/0014-repository-family-extraction-boundaries.md
---

# Repository Family Architecture

## Purpose

Runenwerk is becoming an integration product over independently usable framework
repositories. This document is the canonical architecture for that repository
family and the extraction program that creates it.

The active program covers:

- `RunenSDF`;
- `RunenECS`;
- `RunenRender`;
- their explicit Runenwerk adapters and integration tests.

RunenUI is developed under separate authority. This program treats it only as a
future external producer that may later connect through generic host and render
contracts. No RunenUI implementation, milestone, or API design is authorized by
this document.

## Target dependency direction

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk integration and adapters --> applications
RunenRender ----+
RunenUI --------+   (future, governed by separate authority)
```

Framework repositories must not depend on Runenwerk.

The default rule is that framework repositories also do not depend on one
another. A direct framework-to-framework dependency requires a separate ADR that
proves independent usefulness, optionality where appropriate, and absence of a
Runenwerk-specific adapter concern.

In particular:

- `RunenRender` must not require RunenECS, RunenSDF, or RunenUI;
- `RunenECS` must not require Runenwerk geometry, rendering, networking, or app
  lifecycle;
- `RunenSDF` must not require Runenwerk geometry, world, renderer, or ECS types;
- adapters map repository-local contracts explicitly inside Runenwerk.

## Repository missions

### RunenSDF

RunenSDF owns reusable signed-distance-field mathematics and CPU reference
queries:

- field sampling;
- samples and distance facts;
- bounded/unbounded field bounds;
- primitives;
- boolean and smooth composition;
- transforms;
- gradients and normals;
- projection, closest-point, raymarch, classification, and sweep queries;
- numerical policy and deterministic reference tests.

RunenSDF does not own world chunks, streaming, ECS components, renderer passes,
GPU resources, material semantics, or Runenwerk product policy.

### RunenECS

RunenECS owns reusable entity-component-system semantics:

- entity and component lifecycles;
- storage and queries;
- resources;
- deferred commands;
- change detection;
- system access declarations;
- ECS schedule semantics when those semantics are proven ECS-specific;
- external macros and public conformance;
- repository-local diagnostics and introspection.

RunenECS does not automatically own general spatial indexing, engine frame/tick
policy, rendering extraction, transport, prediction, rollback, scene management,
or world streaming.

### RunenRender

RunenRender will own backend-neutral render planning and a conventional WGPU
backend after those boundaries are proven inside Runenwerk.

Candidate neutral ownership includes:

- render graph declarations and validation;
- pass and resource contracts;
- generic frame planning;
- generic producer submissions;
- backend capability contracts;
- backend-neutral diagnostics.

Candidate WGPU ownership includes:

- instance, adapter, device, and queue management;
- GPU resources and uploads;
- pipelines and command encoding;
- WGPU surface configuration, acquisition, presentation, and device-loss facts.

RunenRender does not own ECS extraction, Runenwerk scene/world/material/SDF
semantics, application lifecycle, editor policy, or UI semantics.

### Runenwerk

Runenwerk remains the product integration authority. It owns:

- engine and application lifecycle;
- frame/tick policy;
- windows and event-loop policy;
- ECS-to-render extraction;
- scene, world, material, SDF, editor, and future UI adapters;
- product feature composition;
- cross-repository diagnostics and compatibility tests;
- application binaries and product evidence.

## Adapter rule

A framework core must remain useful without its Runenwerk adapter. An adapter may
depend on the framework and Runenwerk contracts, but the reverse dependency is
forbidden.

Adapters own translation only. They must not duplicate framework algorithms,
retain mirrored source, introduce writable parallel authorities, or become broad
facade repositories.

## No shared-core magnet

Do not create `RunenCore`, `foundation/meta`, a universal ID repository, a
universal diagnostics repository, or a generic plugin framework merely to make
extraction convenient.

Values belong with the repository whose invariants they express. Cross-repository
mapping is explicit at adapter boundaries.

## Identity policy

Each repository owns opaque identities for its own runtime and persisted
concepts. Process-local runtime identities are not silently serialized. Stable
persisted identifiers require an explicit format, version, validation policy,
and migration contract.

Adapters may maintain maps between identities but must not make distinct identity
families interchangeable.

## Diagnostics policy

Diagnostic codes are repository-namespaced:

```text
runensdf.*
runenecs.*
runenrender.*
runenwerk.*
```

Adapters preserve upstream diagnostic identity and add integration context. They
do not replace upstream failures with unstructured strings.

## Toolchain and release policy

Every extracted repository must define:

- Rust edition;
- MSRV;
- stable formatting policy;
- locked test and Clippy policy;
- package publication state;
- license and provenance;
- API stability status.

Before stable publication, Runenwerk integrations use an exact Git commit or an
exact pre-release version. Moving branch dependencies are forbidden.

After publication, each repository follows semantic versioning and records a
compatibility matrix in Runenwerk.

## Persisted-format policy

Rust API compatibility and persisted-format compatibility are separate.

Every persisted source, artifact, trace, replay, cache, or wire format must name:

- owning repository;
- format identifier;
- schema/version;
- compatibility policy;
- validation behavior;
- migration behavior;
- deterministic encoding requirements where applicable.

Internal runtime packets are not treated as stable persisted formats by default.

## Conformance policy

Every framework repository must have:

- unit and property tests for owned invariants;
- public downstream-consumer tests;
- negative boundary tests;
- stable and MSRV validation;
- documentation/link checks;
- metadata, license, and dependency checks.

Runenwerk owns the cross-repository integration matrix. Integration tests must
prove that framework cores remain buildable without Runenwerk and that adapters
preserve one-way dependency direction.

## Provenance and licensing

Extraction must preserve authorship, license, and source provenance. Each
repository receives a migration report identifying:

- source commit and paths;
- transferred responsibilities;
- redesigned responsibilities;
- responsibilities retained in Runenwerk;
- deleted original paths;
- compatibility and validation evidence.

Git submodules, mirrored directories, and long-lived copied source are forbidden.

## Clean-cutover rule

Each extraction uses one coherent cutover:

1. investigate current source and all consumers;
2. accept a decision-complete boundary design;
3. correct the boundary inside Runenwerk;
4. prove independent conformance;
5. create and populate the external repository;
6. pin Runenwerk to the external repository;
7. migrate consumers;
8. delete the original implementation;
9. remove temporary migration seams;
10. update current authority and close the phase.

Temporary parallel code may exist on an unmerged extraction branch. It must not
survive the final merged state.

Compatibility crates, deprecated forwarding modules, source mirrors, and writable
parallel APIs are forbidden unless a separate ADR proves a time-bounded external
compatibility requirement. Internal convenience is not sufficient.

## Track sequencing

Three tracks may progress in parallel at different maturity levels:

```text
RunenSDF    implementation-active after its design gate
RunenECS    investigation/design-active before extraction
RunenRender investigation and internal-decomposition-active before extraction
```

The current order is:

1. repository-family charter and planning reset;
2. RunenSDF complete investigation and boundary design;
3. RunenECS complete investigation and architecture decisions;
4. RunenRender complete semantic inventory and seam design;
5. RunenSDF extraction;
6. RunenECS boundary repair and extraction;
7. RunenRender internal decomposition;
8. RunenRender external extraction.

RunenRender moves last because it currently combines neutral planning, WGPU,
window/surface, ECS, scene, material, SDF, UI, editor, diagnostics, and runtime
integration concerns.

## Shared-file coordination

The following files have one active owner at a time:

```text
Cargo.toml
Cargo.lock
ARCHITECTURE.md
DOMAIN_MAP.md
DEPENDENCY_RULES.md
TESTING.md
workspace/planning/active-work.md
workspace/planning/roadmap.md
workspace/planning/production-tracks.md
workspace/planning/decision-register.md
```

Parallel tracks use track-specific investigation, design, proof, and closeout
files. Shared authority changes occur only when a phase changes lifecycle state.

## Extraction gates

No external repository transfer begins until the relevant track proves:

- complete source and consumer inventory;
- decision-complete public API and ownership;
- no unresolved dependency cycle;
- independent downstream consumer;
- validation and conformance plan;
- versioning, diagnostics, identity, and persisted-format decisions;
- exact move/stay/redesign/delete map;
- clean-cutover and rollback strategy;
- repository creation and provenance plan.

RunenRender additionally requires an internal package proof: Runenwerk must use
the separated renderer through the same boundary intended for external consumers.
