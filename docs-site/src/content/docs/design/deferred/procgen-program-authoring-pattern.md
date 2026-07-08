---
title: Procgen Program Authoring Pattern
description: Deferred procedural-generation domain instantiation of the domain authoring source/program pattern, covering procgen source graphs, procgen programs, deterministic evaluators, runtime artifacts, hosts, diagnostics, migration, live preview, spatial scope, and proof surfaces.
status: deferred
owner: procgen
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ../active/domain-authoring-source-and-program-pattern.md
  - ../active/runenwerk-domain-workbench-north-star.md
  - ../../guidelines/domain-program-architecture-pattern.md
---

# Procgen Program Authoring Pattern

This document is a design sketch for how ProcgenProgram instantiates the domain authoring source/program pattern. MaterialProgram remains the next named second-domain proof; this file records ProcgenProgram as an additional deferred candidate, not a replacement or successor to the MaterialProgram proof path.

## Status

Deferred design sketch. This document does not authorize implementation, crate
creation, world runtime changes, terrain generation work, editor tooling, or
shared platform extraction.

The purpose is to record how the domain-authoring pattern should apply to a
non-UI proving domain after UI proof evidence is credible.

The frontmatter owner value was validated against the docs schema during PR readiness cleanup.

Procgen authoring remains owned by the procgen/world-generation domain. The `owner` frontmatter value is schema-facing only.

## Decision

Procedural generation should be authored as a procgen-owned domain program
track:

```text
ProcgenSource
-> ProcgenSourceGraph
-> NormalizedProcgenGraph
-> ProcgenProgram
-> ProcgenEvaluator / ProcgenCompiler
-> WorldChunkRecipe / SpawnTableArtifact / FieldCacheArtifact
-> EditorHost / GameRuntimeHost / HeadlessBakeHost integration
-> ProcgenProofReport / diagnostics / migration reports
```

Procgen must not reuse `UiSource`, UI control semantics, a universal graph
runtime, hidden random state, or ECS-owned source truth.

## Package Contributions

Procgen packages contribute domain meaning:

```text
rule node descriptors
field node descriptors
biome constraints
scatter operators
erosion operators
structure placement operators
determinism policies
seed policies
budget classes
evaluator kernels
compiler/baker kernels
authoring metadata
fixtures
diagnostics
migrations
source-map behavior
inspection behavior
runtime artifact builders
```

Example procgen nodes:

```text
HeightNoise
BiomeMask
ScatterPoints
RiverErosion
StructurePlacement
SpawnRule
DistanceFieldMask
DirtyRegionRebuild
```

## Spatial Scope Vocabulary

Procgen must define spatial scope explicitly before implementation.

Candidate vocabulary:

```text
World
SuperRegion
Region
Chunk
Cell
Layer
Field
Volume
StreamingWindow
DirtyRegion
DependencyTile
PreviewBounds
BakeBounds
```

Spatial scope affects determinism, invalidation, streaming, preview, and proof
comparison. It must not be hidden inside ECS queries or renderer-side caches.

## Source And Graph Families

Procgen source is usually graph-shaped, but the public domain stage is
`ProcgenSource`, not a generic AST.

Candidate graph families:

```text
FieldGraph
BiomeGraph
RegionGraph
StructureGraph
SpawnGraph
DependencyGraph
BudgetGraph
ValidationGraph
DiagnosticGraph
```

These graph meanings are procgen-owned. A future shared typed graph substrate may
provide IDs, edges, ports, traversal, source maps, and diagnostics attachment
only after extraction gates pass.

## ProcgenProgram Contract

A `ProcgenProgram` should contain:

```text
program id and version
source references and source maps
required procgen packages
required host/runtime capabilities
schema references
procgen typed graphs
determinism metadata
seed policy metadata
budget policy metadata
streaming/invalidation metadata
validation metadata
evaluator contract
compiler/baker contract
fixture references
diagnostics
migration metadata
runtime artifact declarations
```

## Runtime Artifacts

Procgen compilers/evaluators may produce:

```text
WorldChunkRecipe
SpawnTableArtifact
FieldCacheArtifact
BiomeMaskArtifact
StructurePlacementArtifact
DirtyRegionInvalidationPlan
ProcgenCacheKey
```

Artifacts are derived products. They must not become procgen source truth.

## Live Preview And Bake Split

Procgen needs separate preview and bake modes.

Preview evaluator:

```text
bounded
fast
partial
interactive
may use lower resolution
must report approximations
must preserve determinism where claimed
```

Bake evaluator/compiler:

```text
complete for declared bounds
strictly deterministic
cacheable
fixture-comparable
streaming-compatible
```

Live preview loop:

```text
edit procgen source node/property
-> validate source graph
-> normalize graph
-> form ProcgenProgram diff
-> compute dirty spatial scope
-> invalidate affected field/chunk/spawn artifacts
-> preview affected bounds
-> preserve preview camera/selection/seed state by policy
-> show source-map diagnostics
```

Invalid procgen source keeps the last known good preview artifact and overlays
diagnostics.

## Host Profiles

Procgen host compatibility should consider:

```text
EditorHost
GameRuntimeHost
HeadlessBakeHost
PreviewHost
WorldSpaceHost
BuildHost
```

A host compatibility report should state which nodes, capabilities, determinism
policies, budget classes, preview modes, streaming modes, and bake profiles are
accepted or rejected.

## Required Domain Concerns

Procgen must define first-class handling for:

```text
determinism
seed policy
budget policy
preview versus bake
streaming compatibility
incremental invalidation
dirty-region rebuilding
spatial dependency tracking
reproducible fixture output
host-specific capability limits
```

## Proof Requirements

A first procgen proof should demonstrate:

```text
package contribution -> descriptor/catalog
source graph validation
program formation
deterministic evaluator output
artifact manifest
source-map attachment
diagnostics for invalid seeds/types/capabilities/budgets
fixture reproducibility
host compatibility report
live-preview invalidation report
```

Proof comparison modes:

```text
exact hash comparison for discrete artifacts
tolerant numeric comparison for float fields
seed reproducibility check
budget compliance check
dirty-region minimality check
streaming window consistency check
```

## Shared Structure Candidates

Procgen should help prove whether these are domain-neutral enough for later
extraction:

```text
package manifest envelope
extension-point manifest envelope
source-map envelope
diagnostic envelope
capability reference model
typed graph structural substrate
artifact manifest envelope
host compatibility matrix
proof report envelope
incremental invalidation report envelope
```

## Rejected

Do not implement or authorize:

```text
UiSource reuse for procgen
universal node graph semantics
hidden global random state
ECS-owned procgen source truth
runtime interpretation of generic graphs in hot paths by default
generic compiler framework
generic evaluator framework
shared extraction before UI plus procgen proof evidence
```
