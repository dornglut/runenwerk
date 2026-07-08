---
title: Material Program Authoring Pattern
description: Deferred material-domain instantiation of the domain authoring source/program pattern, covering material source graphs, material programs, compiler artifacts, packages, diagnostics, migration, and proof surfaces.
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

## Status

Deferred design sketch. This document does not authorize implementation, crate
creation, renderer changes, shader compiler work, material editor work, or shared
platform extraction.

The purpose is to record how the domain-authoring pattern should apply to a
non-UI proving domain once the UI proof is credible.

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
features, shader stages, preview modes, and target profiles are accepted or
rejected.

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
