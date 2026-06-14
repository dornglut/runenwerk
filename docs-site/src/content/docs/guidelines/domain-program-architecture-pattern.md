---
title: Domain Program Architecture Pattern
description: Stable architecture pattern for domain-owned programs, typed graphs, compiler/evaluator boundaries, runtime artifacts, hosts, and extraction gates.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-06-14
related:
  - ./runenwerk-architecture.md
  - ./architecture.md
  - ../design/active/runenwerk-domain-workbench-north-star.md
  - ../design/active/ui-program-architecture.md
  - ../design/active/ui-program-architecture-owner-map.md
---

# Domain Program Architecture Pattern

## Purpose

This document extracts the reusable architecture pattern behind `UiProgram`
without extracting a generic framework or new foundation crate.

The pattern applies when a domain needs a durable, versioned, inspectable
program contract that connects authoring, validation, compilation, evaluation,
runtime artifacts, hosts, diagnostics, source maps, fixtures, and migration.

This is a guideline for future domain-program tracks such as `MaterialProgram`,
`RenderPlan`, `WorldProgram`, `SimulationProgram`, `ToolProgram`,
`AssetImportProgram`, `AnimationProgram`, `BehaviorProgram`, or
`GameplayProgram`.

It does not authorize product implementation, crate creation, shared
`foundation/meta` extraction, or a generic graph runtime.

## Core Rule

```text
Domains own meaning.
The platform owns structure.
```

The platform may standardize structural concepts:

```text
program identity
program versions
source maps
typed graph shape
schema references
capability references
package references
compiler/evaluator contracts
runtime artifact manifests
diagnostics
fixtures
migrations
host contracts
proof evidence
```

The platform must not own domain meaning:

```text
buttons
materials
render passes
world regions
gameplay rules
editor tools
asset import semantics
simulation behavior
```

Those meanings stay inside the owning domain.

## When A System Should Become A Domain Program

A system should become a domain program only when several of these are true:

- it is authored or generated from durable source;
- it needs stable identity and versioning;
- it needs typed graph structure or typed relationships;
- it must be inspected, diffed, migrated, or fixture-tested;
- it has more than one host, target profile, or runtime context;
- hot paths need compiled or evaluated runtime artifacts;
- source maps and diagnostics matter for user-facing repair;
- implementation correctness needs semantic conformance evidence.

Small helpers, one-off runtime structs, local editor panels, and narrow
implementation details should not become domain programs.

## Standard Pipeline

The preferred shape is:

```text
authored domain source
-> normalized domain model
-> domain program
-> typed domain graphs
-> compiler and/or evaluator
-> runtime artifacts
-> output facts
-> host effects
```

The important separation is:

```text
authoring model != domain program != runtime artifact
```

Do not collapse these layers for convenience. A domain program is the durable
cross-layer contract. Runtime artifacts are derived products optimized for
execution. Hosts perform environment-specific effects.

## Layer Responsibilities

### Authoring Source

Authoring source is the user-facing or tool-facing form.

It may contain friendly structure, templates, unresolved references, editor
metadata, or source locations. It is not the hot-path runtime format.

The authoring owner is responsible for:

- source identity;
- normalization inputs;
- validation diagnostics;
- migration from older source versions;
- source locations for later source-map attachment.

### Normalized Model

The normalized model is canonical source after validation, migration, and
normalization.

It should be deterministic, source-map capable, and suitable as input to a
domain program builder.

### Domain Program

A domain program is the durable executable contract for one domain.

It should contain:

- program id and version;
- source references and source maps;
- typed domain graphs;
- schema references;
- package and capability requirements;
- validation metadata;
- dependency metadata;
- diagnostics;
- inspection hooks;
- migration metadata;
- fixture references;
- runtime artifact description.

The program is domain-owned. A `UiProgram` should not become a
`MaterialProgram`, and a `MaterialProgram` should not reuse UI semantics.

### Typed Graphs

Graphs describe domain relationships inside a program.

Correct direction:

```text
TypedGraph<DomainGraphKind>
```

Rejected direction:

```text
UniversalNodeGraph
```

The platform may eventually share graph identity, node identity, edge identity,
source-map attachment, traversal, serialization, and diagnostic hook vocabulary.
It must not own the meaning of domain nodes, edges, ports, kernels, or passes.

### Compiler

A compiler transforms a domain program into optimized runtime artifacts.

It should own:

- package resolution;
- capability checks;
- cache keys;
- artifact construction;
- source-map preservation;
- diagnostics from unresolved or incompatible program inputs.

Hot paths should consume artifacts, not generic authoring graphs.

### Evaluator

An evaluator deterministically executes or analyzes a program or artifact.

It should produce facts:

- output packets;
- event packets;
- diagnostics;
- traces;
- inspection reports;
- proof artifacts;
- runtime artifact evidence.

Evaluators should not hide side effects. Host effects belong at host
boundaries.

### Runtime Artifacts

Runtime artifacts are optimized derived products.

They may contain:

- manifests;
- runtime tables;
- cache keys;
- package and capability records;
- source-map tables;
- diagnostic tables;
- target-profile metadata;
- invalidation metadata.

Artifacts must not become source truth. They are reproducible products of
programs and declared inputs.

### Hosts

Hosts connect evaluated outputs to concrete environments.

Examples:

- editor host;
- game host;
- world-space host;
- headless test host;
- CLI host;
- remote or preview host.

Host contracts are domain-facing boundaries. Concrete app, editor, game,
renderer, or runtime integration stays outside the pure domain-program crate
unless explicitly owned by that domain.

## Foundation Boundary

Foundation crates may define shared vocabulary only.

Current vocabulary direction:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
foundation/resource_ref
```

Foundation must not own:

- domain program meaning;
- command execution;
- editor policy;
- ECS mutation;
- renderer product truth;
- global registries;
- AI runtime behavior;
- generic graph interpretation;
- domain validation rules.

## Extraction Rule

Use this sequence:

```text
design the pattern
-> prove one domain
-> prove a second domain
-> extract only repeated domain-neutral primitives
```

Do not create a shared foundation or platform crate from a single proving
domain.

A primitive may move into shared foundation or platform ownership only when:

- at least two domains need it;
- its contract is domain-agnostic;
- it does not weaken domain meaning;
- it improves inspection, validation, migration, testing, or runtime artifacts;
- versioning implications are documented;
- runtime overhead is explicit and acceptable;
- docs and tests exist;
- an accepted extraction design authorizes the exact scope.

## Explicit Non-Goals

This pattern does not authorize:

- `foundation/meta`;
- a generic `DomainProgram` crate;
- a generic graph runtime;
- universal node types;
- a generic compiler framework;
- a generic evaluator framework;
- one artifact model for every domain;
- moving product semantics into foundation;
- renderer-owned product truth;
- ECS-owned domain semantics.

These may only be revisited after two or more proving domains expose the same
domain-neutral primitive and an accepted extraction design approves the exact
boundary.

## Current Proving Domains

`UiProgram` is the first concrete proving-domain implementation.

Its current owner map and proof surface live under `domain/ui/` and are governed
by:

- [`ui-program-architecture.md`](../design/active/ui-program-architecture.md)
- [`ui-program-architecture-owner-map.md`](../design/active/ui-program-architecture-owner-map.md)

The next useful proving domain should be `MaterialProgram`, `RenderPlan`, or
another non-UI domain that can reuse the same architecture spine without sharing
UI meaning.

## Domain Program Checklist

Before creating or completing a domain-program track, verify:

- the domain owns its meaning explicitly;
- authoring, program, artifact, evaluator, and host boundaries are separate;
- graph families are typed and domain-specific;
- source maps and diagnostics are first-class;
- compiler and evaluator timing is explicit;
- runtime artifacts are optimized and inspectable;
- fixture and headless proof paths exist;
- migration and compatibility rules exist;
- host effects are explicit;
- renderer, ECS, apps, and adapters do not own domain truth;
- future shared extraction is blocked unless independently authorized.

## Usage For Future Tracks

For a new domain-program track, start with:

```text
1. Define the domain-owned program contract.
2. Define typed graph families.
3. Define compiler/evaluator responsibilities.
4. Define runtime artifact families.
5. Define host contracts.
6. Define diagnostics, source maps, fixtures, and migration.
7. Define conformance evidence.
8. Prove the domain without extracting shared foundation code.
```

Only after a second domain proves the same shape should shared abstractions be
considered.
