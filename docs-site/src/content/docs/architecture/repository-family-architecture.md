---
title: Repository Family Architecture
description: Canonical repository ownership, dependency direction, integration, release, conformance, and clean-cutover rules for RunenSDF, RunenECS, RunenRender, RunenUI, and Runenwerk.
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

Runenwerk is the integration and product repository for a family of independently
usable framework repositories. This document owns only repository-level
boundaries. Track documents own subsystem APIs and implementation choices.

The family is:

```text
RunenSDF -------+
RunenECS -------+--> Runenwerk integration/adapters --> applications
RunenRender ----+
RunenUI --------+
```

Framework repositories must not depend on Runenwerk. Runenwerk may depend on
framework public APIs directly or through explicit Runenwerk-owned adapters.

## Current program state

| Track | Current state | What is authorized |
|---|---|---|
| RunenSDF | architecture ready; source/API investigation substantially complete; executable consumer verification pending | documentation and verification only until a bounded implementation phase is activated |
| RunenECS | architectural boundary investigation in progress | investigation and design only |
| RunenRender | semantic boundary investigation in progress | investigation and internal-decomposition design only |
| RunenUI | independent repository and workstream | no RunenUI implementation is authorized by this program |

“Complete” means the repository workflow gates and required command evidence have
actually passed. Connector-only inspection is not command validation.

## Dependency direction

The default dependency rule is:

```text
framework core -> lower-level external libraries only
Runenwerk adapter -> one framework + Runenwerk contracts
Runenwerk product -> Runenwerk integration + selected frameworks
```

Framework-to-framework dependencies are not the default. A direct dependency
requires a separate ADR proving that it is independently useful, correctly owned,
and not merely avoiding a Runenwerk adapter.

In particular:

- RunenRender does not require RunenECS, RunenSDF, or RunenUI;
- RunenECS does not require Runenwerk geometry, rendering, networking, replay, or
  application lifecycle;
- RunenSDF does not require Runenwerk geometry, world, renderer, ECS, or material
  types;
- RunenUI does not require RunenRender or Runenwerk;
- Runenwerk owns cross-domain composition and translation.

## Repository missions

### RunenSDF

RunenSDF owns reusable signed-field mathematics, validated field vocabulary,
numerical policy, spatial bounds, field composition, and CPU reference queries.
The exact sample and query contracts remain owned by the RunenSDF design and must
be proven safe for the supported algorithms before implementation.

It does not own world streaming, ECS components, renderer passes, GPU resources,
material semantics, or Runenwerk product policy.

### RunenECS

RunenECS owns entity/component/resource lifecycle semantics, storage and query
contracts, deferred mutation, system access contracts, explicit reflection, and
ECS-local scheduling integration over a neutral scheduler package.

Durable repository authority does not freeze a specific storage implementation.
Archetypes, sparse sets, dense columns, or other internal structures may evolve
while public lifecycle, query, iteration, and mutation guarantees remain stable.

General spatial indexing, engine frame/tick policy, rendering extraction,
transport, replication, prediction, rollback, replay policy, scene management,
and world streaming remain outside ECS core.

Messaging and change-journal facilities remain provisional until consumer
inventory proves their independent ECS role.

### RunenRender

RunenRender will own proven backend-neutral render planning and a conventional
WGPU backend after those boundaries are demonstrated inside Runenwerk.

Required initial package candidates are:

```text
runenrender_core
runenrender_wgpu
```

A proc-macro package is optional and is created only if the existing derives are
retained after GPU ABI review. Proc-macro mechanics alone do not justify keeping
the derives.

RunenRender does not own ECS extraction, scene/world/material/SDF/UI semantics,
Runenwerk application lifecycle, editor policy, native-window policy, or product
feature selection.

Backend-neutral core contracts must not silently standardize WGSL/WGPU memory
layout. Backend-specific layout realization belongs with the WGPU ABI owner.

### RunenUI and RunenRender

RunenUI and RunenRender are independent peers:

```text
RunenUI
  owns semantic UI, hit testing, UI layout/style/control behavior,
  and renderer-neutral UI paint output

RunenRender
  owns general render planning, resources, pipelines, WGPU execution,
  surfaces, and backend diagnostics

Runenwerk integration
  translates accepted RunenUI paint output into generic RunenRender work
```

RunenUI may retain lightweight standalone backends for independent UI use. That
does not make RunenRender depend on RunenUI or require RunenUI to depend on
RunenRender. The integration contract is defined only after both public
boundaries are accepted.

### Runenwerk

Runenwerk owns:

- engine and application lifecycle;
- frame/tick policy and plugin composition;
- native windows and event-loop policy;
- ECS-to-render extraction;
- scene, world, material, SDF, editor, and UI adapters;
- product feature composition;
- cross-repository compatibility tests and diagnostics;
- application binaries and runtime evidence.

## Adapter rule

A framework must remain useful without its Runenwerk adapter. An adapter may
depend on the framework and Runenwerk contracts, but neither owner depends back
on the adapter.

Adapters translate identities, inputs, outputs, lifecycles, diagnostics, and
resource ownership. They must not:

- copy framework algorithms;
- mirror framework source;
- introduce a writable parallel authority;
- expose broad compatibility facades;
- hide a dependency cycle.

## No shared-core magnet

Do not create `RunenCore`, `foundation/meta`, a universal ID repository, a
universal diagnostics repository, or a generic plugin framework merely to make
extraction convenient.

Values belong with the repository whose invariants they express. Adapters map
repository-local values explicitly.

## Identity and diagnostics

Each repository owns opaque identities for its own runtime concepts. Process-local
identities are not silently serialized. Stable persisted identities require an
explicit format, validation, version, and migration contract.

Diagnostics are repository-namespaced:

```text
runensdf.*
runenecs.*
runenrender.*
runenui.*
runenwerk.*
```

Adapters preserve upstream diagnostic identity and add integration context rather
than replacing failures with unstructured strings.

## Toolchain and release policy

Every extracted repository defines:

- Rust edition and MSRV;
- formatting, locked test, and denied-warning policy;
- package publication state and API stability status;
- license, security policy, and source provenance;
- dependency and feature policy.

Before stable publication, Runenwerk uses an exact commit or exact pre-release
version. Moving branch dependencies are forbidden. Published repositories follow
semantic versioning, and Runenwerk records a compatibility matrix.

## Persisted formats

Rust API compatibility and persisted-format compatibility are separate.

Every persisted source, artifact, trace, replay, cache, or wire format names:

- owning repository;
- format identifier and version;
- validation and compatibility policy;
- migration behavior;
- deterministic encoding requirements where applicable.

Internal runtime packets are not stable formats by default.

## Conformance

Every framework repository requires:

- unit, negative, and property tests for owned invariants;
- at least one downstream public-API consumer;
- stable and MSRV validation;
- documentation/link validation;
- metadata, dependency, license, and provenance checks.

Runenwerk owns cross-repository integration tests. Evidence must distinguish
deterministic source/contract proof from environment-dependent GPU, window, and
runtime proof.

## Clean-cutover rule

Each extraction proceeds as follows:

1. inventory current source and all consumers;
2. accept a decision-complete boundary design;
3. correct the boundary inside Runenwerk;
4. prove independent conformance;
5. create and populate the external repository;
6. pin Runenwerk to an exact external revision;
7. migrate active consumers;
8. delete the original implementation;
9. remove temporary migration seams;
10. update authority and close the phase.

Temporary duplication may exist only on an unmerged extraction branch.
Compatibility packages, forwarding namespaces, source mirrors, and submodules do
not survive the completed cutover unless a separate ADR proves a time-bounded
external compatibility requirement.

## Track sequencing

Tracks may progress at different maturity levels:

```text
RunenSDF
  verify investigation -> correct boundary -> extract first

RunenECS
  complete investigation -> close ownership decisions -> repair -> extract

RunenRender
  complete inventory -> decompose internally -> prove public seams -> extract last
```

Only one track at a time owns shared manifests, the lockfile, and canonical
planning summaries. Track-specific investigations and designs may proceed in
parallel.

## Extraction gates

No external source transfer begins until the track proves:

- complete source and consumer inventory;
- decision-complete public ownership and API direction;
- no unresolved dependency cycle;
- independent downstream conformance;
- validation and versioning policy;
- diagnostics, identity, and persisted-format decisions;
- exact move/stay/redesign/delete map;
- clean-cutover, provenance, and rollback strategy.

RunenRender additionally requires an internal anti-cheating proof: Runenwerk must
consume the separated renderer through the same public boundary intended for
external users, with no private reach-through or duplicate path.