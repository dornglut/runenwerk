---
title: Domain Program Architecture Pattern
description: Stable architecture pattern for domain-owned programs, typed graphs, compiler/evaluator boundaries, runtime artifacts, hosts, and extraction gates.
status: active
owner: workspace
layer: workspace
canonical: true
last_reviewed: 2026-08-11
related:
  - ./runenwerk-architecture.md
  - ./architecture.md
  - ../design/active/runenwerk-domain-workbench-north-star.md
  - ../design/active/ui-program-architecture.md
  - ../design/active/ui-program-architecture-owner-map.md
  - ../adr/accepted/0017-cross-authority-consistency-and-graph-semantics.md
---

# Domain Program Architecture Pattern

## Purpose

This document records a reusable architecture pattern for domains that need a
durable, versioned, inspectable program contract. Historical Runenwerk-local
`UiProgram` work contributed evidence for the pattern, but it is not by itself a
current cross-domain proving implementation and does not authorize shared platform
extraction.

The pattern applies when a domain needs a durable, versioned, inspectable
program contract that connects authoring, validation, compilation, evaluation,
runtime artifacts, hosts, diagnostics, source maps, fixtures, and migration.

This is a guideline for future domain-program tracks such as `MaterialProgram`,
`RenderProgram`, `WorldProgram`, `SimulationProgram`, `ToolProgram`,
`AssetImportProgram`, `AnimationProgram`, `BehaviorProgram`, or
`GameplayProgram`.

Canonical RunenRender owns the distinct term `RenderPlan` for a per-request,
device-independent plan produced by `RenderMethod` before `AdmittedRenderPlan`.
Generic domain-program guidance must not assign `RenderPlan` a competing durable
program meaning.

It does not authorize product implementation, crate creation, shared
`foundation/meta` extraction, or a generic graph runtime.

## Core Rule

```text
Domains own meaning.
The platform owns structure only after repeated neutral structure is proven.
```

A platform boundary may standardize repeated structural concepts after the
shared-extraction gate is satisfied, for example:

```text
program versions
source maps
typed graph shape
schema references
capability requirements
package references
compiler/evaluator contracts
runtime artifact manifests
diagnostics
fixtures
migrations
host contracts
proof evidence
```

This list is not a pre-approved shared type, registry, identity, crate, or runtime
inventory. Each shared primitive still requires concrete repeated pressure and a
separate accepted extraction decision.

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

It should contain the domain-owned subset that its real consumers require, which
may include:

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

This is a design checklist, not a mandatory universal record layout. The program
is domain-owned. A UI program should not become a `MaterialProgram`, and a
`MaterialProgram` should not reuse UI semantics.

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

Shared graph structure may be extracted only after structurally different domains
prove the same neutral invariant/operation shape. Structural similarity does not
make semantic dependency, execution ordering, resource hazard, invalidation
dependency, or containment the same graph meaning.

The platform must not own the meaning of domain nodes, edges, ports, kernels, or
passes. ADR 0017 owns the family-wide graph taxonomy and feedback law.

### Compiler

A compiler transforms a domain program into optimized runtime artifacts.

It may own domain-specific responsibilities such as:

- package resolution;
- capability requirement checks;
- cache keys;
- artifact construction;
- source-map preservation;
- diagnostics from unresolved or incompatible program inputs.

Hot paths should consume artifacts, not generic authoring graphs.

A compiler contract is not automatically a candidate for a generic compiler
framework. Shared extraction requires independent repeated proof.

### Evaluator

An evaluator deterministically executes or analyzes a program or artifact.

It should produce owner-defined facts such as:

- output packets;
- event packets;
- diagnostics;
- traces;
- inspection reports;
- proof artifacts;
- runtime artifact evidence.

Evaluators should not hide side effects. Host effects belong at host
boundaries.

An evaluator contract is domain-owned unless a later extraction decision proves a
repeated neutral primitive.

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

Artifacts must not become source truth merely because they are executable or
cached. Their exact semantic role, validity, lineage, and lifetime remain owned by
the producing domain.

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

Shared foundation/platform vocabulary is not presumed. A current low-level crate
may own a narrow reusable contract, but its existence does not authorize a future
family-wide meta layer.

Examples of current or historical low-level vocabulary include:

```text
foundation/id
foundation/id_macros
foundation/diagnostics
foundation/ratification
foundation/schema
foundation/commands
foundation/resource_ref
```

Each such boundary is judged by its actual owner and consumers. ADR 0014 and ADR
0017 reject a universal identity repository, diagnostics repository,
`foundation/meta`, or another shared substrate created merely for architectural
uniformity.

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
design locally
-> prove one domain
-> prove a structurally different second domain
-> characterize repeated burden and cost
-> extract only the repeated domain-neutral primitive through a separate decision
```

Do not create a shared foundation or platform crate from a single proving
domain.

Before a primitive moves into shared foundation or platform ownership, require all
of the following:

1. at least two structurally different domains prove the same invariant or
   operation shape;
2. repeated implementation or maintenance burden is concrete;
3. the proposed contract contains no proving-domain semantic branches, enum cases,
   or vocabulary;
4. it remains meaningful if either proving domain disappears;
5. dependency direction remains valid and independent peer frameworks are not
   forced onto Runenwerk-owned meta-infrastructure;
6. ordinary owner APIs remain understandable without first learning the shared
   substrate;
7. runtime, serialization, versioning, memory, and cognitive cost are
   characterized where relevant;
8. a separate accepted extraction design authorizes exactly the repeated
   primitive.

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

These may only be revisited after structurally different proving domains expose
the same domain-neutral primitive and an accepted extraction design approves the
exact boundary.

## Current Proving Evidence

Historical Runenwerk-local `UiProgram` designs remain useful evidence for source
normalization, inspectability, diagnostics, and program/artifact separation, but
they are not current family-wide implementation authority. Standalone
`dornglut/runen-ui` owns current RunenUI architecture and does not currently expose
or depend on a Runenwerk-owned `UiProgram` substrate.

Therefore no current UI claim is sufficient to authorize shared Domain Program
infrastructure. A future extraction decision must identify at least two current,
structurally different owner domains that independently prove the exact repeated
primitive.

Issue #205 owns the broader disposition of historical Runenwerk-local UI design
material. This guideline does not pre-decide whether those documents are retained
as history, merged, superseded, or deleted.

## Domain Program Checklist

Before creating or completing a domain-program track, verify:

- the domain owns its meaning explicitly;
- authoring, program, artifact, evaluator, and host boundaries are separate where
  the domain needs those stages;
- graph families are typed and domain-specific;
- source maps and diagnostics are first-class where user repair requires them;
- compiler and evaluator timing is explicit;
- runtime artifacts are optimized and inspectable;
- fixture and headless proof paths exist where needed;
- migration and compatibility rules exist for durable contracts;
- host effects are explicit;
- renderer, ECS, apps, and adapters do not own domain truth by convenience;
- future shared extraction is blocked unless independently authorized;
- ordinary APIs remain understandable without mandatory knowledge of internal
  graphs, plans, or common substrate.

## Usage For Future Tracks

For a new domain-program track, start with:

```text
1. Define the domain-owned program contract.
2. Define only the graph families the domain actually needs.
3. Define compiler/evaluator responsibilities where formation requires them.
4. Define runtime artifact families.
5. Define host contracts.
6. Define diagnostics, source maps, fixtures, and migration where required.
7. Define conformance evidence.
8. Prove the domain without extracting shared foundation code.
```

Only after a structurally different second domain proves the same neutral
primitive should shared extraction be considered.