---
title: Material Program Authoring Pattern
description: Deferred material-domain instantiation of the domain authoring source/program pattern, covering material source graphs, material programs, compiler artifacts, packages, diagnostics, migration, live preview, SDF material concerns, and proof surfaces.
status: deferred
owner: material
layer: design
canonical: false
last_reviewed: 2026-07-08
related:
  - ../active/domain-authoring-source-and-program-pattern.md
  - ../active/runenwerk-domain-workbench-north-star.md
  - ../../guidelines/domain-program-architecture-pattern.md
---

# Material Program Authoring Pattern

This document is a design sketch for how MaterialProgram — the next named second-domain proof — instantiates the domain authoring source/program pattern. The active and material tracks are secondary to UI; ProcgenProgram is an additional deferred candidate, not a replacement for or competitor of MaterialProgram.

## Status

Deferred design sketch. This document does not authorize implementation, crate
creation, renderer changes, shader compiler work, material editor work, or shared
platform extraction.

The purpose is to record how the domain-authoring pattern should apply to a
non-UI proving domain once the UI proof is credible.

The frontmatter owner value was validated against the docs schema during PR readiness cleanup.

Material authoring remains owned by the material domain. The `owner` frontmatter value is schema-facing only.

## Decision

Materials should be authored as a material-owned domain program track:

```text
MaterialSource
-> MaterialSourceGraph
-> NormalizedMaterialGraph
-> MaterialProgram
-> MaterialCompiler / MaterialPreviewEvaluator
-> ShaderModuleArtifact / MaterialPipelineArtifact
-> RenderHost / PreviewHost integration
-> MaterialProofReport / diagnostics / migration reports
```

Materials must not reuse `UiSource`, UI control semantics, or renderer-owned
truth. Renderer hosts consume material artifacts. They do not own material source
or material program semantics.

## Package Contributions

Material packages contribute domain meaning:

```text
material node descriptors
parameter schemas
port schemas
value domains
compiler kernels
preview kernels
authoring metadata
fixtures
diagnostics
migrations
source-map behavior
inspection behavior
runtime artifact builders
```

Example base material nodes:

```text
TextureSample2d
ConstantColor
Multiply
Lerp
NormalMap
SdfBlend
Noise
Fresnel
RoughnessRemap
Curvature
AmbientOcclusion
TriplanarProjection
VolumeFieldSample
```

## Source And Graph Families

Material source is graph-shaped by default, but the public domain stage is still
`MaterialSource`, not a universal AST.

Candidate graph families:

```text
ShaderGraph
ParameterGraph
ResourceGraph
PreviewGraph
ValidationGraph
DiagnosticGraph
```

These are material-owned graph meanings over any future shared typed graph
substrate.

## SDF-First Material Concerns

Runenwerk material authoring should account for SDF-native rendering and world
procedural surfaces.

Required material concepts to investigate before implementation:

```text
surface classification
SDF field sampling
signed-distance gradient / normal inputs
curvature inputs
ambient-occlusion inputs
triplanar mapping
volume/material-space coordinates
world-space coordinates
procedural texture fields
distance-aware blending
surface/volume material split
weathering/erosion/snow/sand material modifiers
preview of SDF-driven material attributes
```

These are material-domain semantics. They must not move into generic platform or
UI layers.

## MaterialProgram Contract

A `MaterialProgram` should contain:

```text
program id and version
source references and source maps
required material packages
required render/backend capabilities
schema references
material typed graphs
port/value domain metadata
validation metadata
compiler contract
preview evaluator contract
fixture references
diagnostics
migration metadata
runtime artifact declarations
```

## Runtime Artifacts

Material compilers may produce:

```text
ShaderModuleArtifact
MaterialPipelineArtifact
MaterialParameterLayout
MaterialBindingLayout
MaterialPreviewArtifact
MaterialCacheKey
```

Artifacts are derived products. They must not become material source truth.

## Live Preview Loop

Material live preview should use the same source/program/artifact pipeline:

```text
edit material source node/property
-> validate source graph
-> normalize graph
-> form MaterialProgram diff
-> invalidate affected shader/material artifacts
-> compile or preview-evaluate affected subgraph
-> update preview surface
-> preserve preview camera/light/material-instance state by policy
-> show source-map diagnostics
```

Invalid material source keeps the last known good preview artifact and overlays
diagnostics.

## Host Profiles

Material host compatibility should consider:

```text
RendererHost
MaterialLabHost
GameRuntimeHost
PreviewHost
HeadlessValidationHost
BuildHost
```

A host compatibility report should state which nodes, capabilities, texture
features, shader stages, preview modes, SDF material features, and target profiles
are accepted or rejected.

## Proof Requirements

A first material proof should demonstrate:

```text
package contribution -> descriptor/catalog
source graph validation
program formation
compiler or preview evaluator output
artifact manifest
source-map attachment
diagnostics for invalid ports/types/capabilities
fixture reproducibility
host compatibility report
live-preview invalidation report
```

## Shared Structure Candidates

Material should help prove whether these are domain-neutral enough for later
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
UiSource reuse for materials
universal node graph semantics
shader logic in foundation
renderer-owned material truth
ECS-owned material source truth
generic compiler framework
generic evaluator framework
shared extraction before UI plus material proof evidence
```
