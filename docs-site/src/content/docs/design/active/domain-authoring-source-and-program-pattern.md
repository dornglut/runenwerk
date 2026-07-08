---
title: Domain Authoring Source And Program Pattern
description: Cross-domain authoring pattern for source, normalized models, domain programs, runtime artifacts, hosts, diagnostics, fixtures, migrations, and proof reports without creating a universal AST or shared meta-framework.
status: active
owner: workspace
layer: design
canonical: true
last_reviewed: 2026-07-08
related:
  - ./runenwerk-domain-workbench-north-star.md
  - ../../guidelines/domain-program-architecture-pattern.md
  - ./ui-program-architecture.md
  - ./runenwerk-typed-app-composition-plugin-framework-design.md
---

# Domain Authoring Source And Program Pattern

## Status

Active architecture direction for how Runenwerk domains should be authored and
lowered. This document defines vocabulary and boundaries only. It does not
authorize implementation, crate creation, `foundation/meta`, a generic graph
runtime, a universal AST, a generic compiler/evaluator framework, or shared
platform extraction.

## Decision

Runenwerk domains use this lifecycle:

```text
Authoring Source
-> Source Model
-> Normalized Domain Model
-> Domain Program
-> Runtime Artifact
-> Evaluator / Compiler Output
-> Host Integration
-> Proof / Diagnostics / Migration Reports
```

The lifecycle is common. Domain meaning is not common.

Correct principle:

```text
Domains own meaning.
The platform owns structure.
```

The platform may standardize stable structure such as IDs, versions, package
manifests, source-map envelopes, diagnostic envelopes, capability references,
artifact manifests, host profiles, compatibility reports, proof report envelopes,
and eventually carefully bounded typed graph substrate primitives.

The platform must not own the meaning of controls, material nodes, procgen rules,
render passes, animation states, behavior nodes, gameplay effects, editor tools,
asset import rules, or domain compiler/evaluator behavior.

## Vocabulary

### Authoring Source

Durable user-facing or tool-facing source. Authoring source may be textual,
visual, graph-based, Rust-authored, data-driven, generated, imported, or hybrid.
It is not the hot-path runtime format.

Use `Source`, not `SourceAst`, as the public architecture term. ASTs are one
possible representation of source, not the source stage itself.

### Source Model

The in-memory representation of authoring source after parse, import, build,
projection, or visual-editor export. A source model may be tree-shaped,
graph-shaped, document-shaped, template-shaped, or package-backed.

Concrete source body names are domain-specific:

```text
UiSource
MaterialSourceGraph
ProcgenSourceGraph
RenderPlanSource
ToolSource
AssetImportSource
BehaviorSourceGraph
```

### AST

A concrete syntax-tree representation for textual or tree-shaped source bodies.
`Ast` is not the generic platform stage and must not replace `Source` in public
architecture vocabulary.

### Source Graph

A concrete source-body representation for graph-shaped authoring. Graphs are
common, but source graph meaning stays domain-owned.

### Normalized Domain Model

Canonical validated source after migration, reference resolution, package
resolution, schema checks, and deterministic normalization. It should preserve
source-map provenance and attach diagnostics.

### Domain Program

A durable, versioned, inspectable executable contract for one domain. Examples:

```text
UiProgram
MaterialProgram
ProcgenProgram
RenderPlan
AnimationProgram
BehaviorProgram
GameplayProgram
AssetImportProgram
ToolProgram
```

A domain program contains domain-owned typed graphs, required packages,
capabilities, schemas, source maps, diagnostics, fixture references, migration
metadata, compiler/evaluator contracts, and runtime artifact declarations.

### Runtime Artifact

An optimized derived product created from a domain program. Runtime artifacts may
be cached, hashed, diffed, inspected, invalidated, and reproduced. They must not
become source truth.

### Host

A concrete environment that consumes evaluated facts or runtime artifacts and
performs side effects. Examples include editor, game runtime, world-space,
headless test, CLI, preview, remote-devtools, build, and CI hosts.

## Generalization Matrix

| Structure | Generalize? | Rule |
|---|---:|---|
| Stable IDs | Yes | Domain-neutral. |
| Versions | Yes | Needed for source, program, artifact, migration, and proof. |
| Source-map envelope | Yes | Domain-neutral provenance. |
| Diagnostic envelope | Yes | Envelope can be shared; diagnostic meaning stays domain-owned. |
| Capability references | Yes | Needed for validation, host compatibility, and trust policy. |
| Package manifests | Later | Extract only after UI plus one non-UI proof. |
| Extension-point manifests | Later | Likely shared, but must remain typed and bounded. |
| Typed graph substrate | Later | Share structure only, not node/edge/port meaning. |
| Artifact manifest envelope | Later | Strong candidate after two domain proofs. |
| Host compatibility matrix | Later | Strong candidate after UI plus one non-UI host proof. |
| Proof report envelope | Later | Strong candidate after proof surfaces repeat. |
| Compiler/evaluator traits | Not yet | Easy to over-generalize before second proof. |
| Universal AST | Never | Too narrow and semantically misleading. |
| Universal node graph | Never | Becomes untyped node soup. |
| ECS app model | Never | ECS is runtime fabric, not domain source truth. |

## Domain Instantiations

| Domain | Source | Program | Artifact | Host examples |
|---|---|---|---|---|
| UI | `UiSource` | `UiProgram` | UI runtime artifacts, UI output, frames | editor, game HUD, world-space, headless, preview |
| Materials | `MaterialSourceGraph` | `MaterialProgram` | shader modules, material pipeline artifacts | renderer, material lab, game, preview |
| Procgen | `ProcgenSourceGraph` | `ProcgenProgram` | chunk recipes, spawn tables, field caches | editor, game, headless bake, preview |
| Render | `RenderPlanSource` | `RenderPlan` | render graph artifacts, GPU resource plans | renderer, editor preview, game |
| Animation | `AnimationSourceGraph` | `AnimationProgram` | runtime animation graph, baked tables | game, editor preview, tests |
| Behavior | `BehaviorSourceGraph` | `BehaviorProgram` | runtime behavior plan | game, simulation, headless |
| Asset import | `AssetImportSource` | `AssetImportProgram` | import recipes, cache products | editor, CLI, build host |
| Tools | `ToolSource` | `ToolProgram` | command tables, input plans, preview plans | editor, remote tools, headless |

## Relationship To ECS And Graphs

Most domains contain graphs, but not every source should be forced into one graph
shape. The correct platform direction is:

```text
TypedGraph<DomainGraphKind>
```

not:

```text
UniversalNodeGraph
```

ECS belongs to the runtime fabric. ECS may execute optimized artifacts, hold live
host/runtime state, schedule systems, and bridge concrete runtime behavior. ECS
must not own source truth, package catalogs, domain program semantics, material
node meaning, UI control meaning, procgen rule meaning, or app model truth.

## Required Reports

Any serious domain-authoring track should define report surfaces for:

```text
source validation
normalization
program formation
package resolution
capability checks
compiler/evaluator output
artifact construction
host compatibility
proof execution
migration
assembly where applicable
```

## Rejected Vocabulary

Do not introduce these as durable public architecture names:

```text
UniversalAst
UniversalNodeGraph
GlobalStore
EcsAppModel
RendererSourceTruth
FoundationMeta
GenericCompilerFramework
GenericEvaluatorFramework
GenericFeature
```

## Extraction Rule

Use this sequence:

```text
design the pattern
-> prove UI
-> prove one non-UI domain
-> extract only repeated domain-neutral primitives
```

A shared primitive may be extracted only when at least two domains need it, it is
truly domain-agnostic, it does not weaken domain ownership, versioning and
runtime overhead are documented, tests and docs exist, and an accepted extraction
design authorizes the exact scope.
