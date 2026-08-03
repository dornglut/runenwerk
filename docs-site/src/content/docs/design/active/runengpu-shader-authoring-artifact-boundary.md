---
title: RunenGPU Shader Authoring and Canonical WGSL Artifact Boundary
description: Ownership and promotion rules for shader meaning, authoring languages, canonical WGSL artifacts, explicit interfaces, and private WGPU realization.
status: active
owner: gpu
layer: framework/gpu
canonical: true
last_reviewed: 2026-08-02
related_docs:
  - ./runengpu-architecture-design.md
  - ./runengpu-g4-context-program-realization-design.md
  - ./runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../workspace/specs/pt-runengpu-g4b-program-interface-layout.ron
  - ../../workspace/specs/pt-runengpu-g4c-wgpu-realization-cutover.ron
---

# RunenGPU Shader Authoring and Canonical WGSL Artifact Boundary

## Status and authority

This focused design clarifies the accepted G4 program/source boundary from issue
`#182`, PR `#185`, and merge:

```text
62c3949d31a7c03f1f554f8108120d9767139123
```

It is delivered by issue `#203` before G4B implementation begins.

This design does not reopen the accepted G4A/G4B/G4C decomposition and does not
add a shader compiler, dependency, source kind, runtime path, package, stable source
format, or implementation authorization.

The accepted G4B decision remains:

```text
GpuProgramSource initially admits canonical WGSL source only.
```

The clarification is that the admitted runtime artifact and the human authoring
language are separate concerns.

## Decision summary

Runenwerk uses one portable runtime path:

```text
RunenRender or another GPU consumer
    owns shader or kernel meaning
        -> Runenwerk shader authoring/toolchain policy
            owns files, modules, compilation, source maps, reload and fallback
                -> canonical WGSL artifact
                    -> explicit RunenGPU program/interface descriptors
                        -> private WGPU/Naga realization
```

The bound language strategy is:

1. Plain WGSL remains a valid authoring source and the only initial RunenGPU runtime
   artifact.
2. WESL-to-WGSL is the preferred near-term modular authoring candidate, but requires a
   separately authorized deterministic toolchain proof before adoption.
3. Slang-to-WGSL is the preferred future advanced authoring candidate, but remains
   gated until its WebGPU/WGSL and Metal paths pass RunenGPU portability and interface
   conformance proof.
4. WESL and Slang do not become direct `GpuProgramSource` variants.
5. SPIR-V, GLSL, HLSL, DXIL, MSL, Naga IR and backend-native modules do not become
   public or stable RunenGPU authority through this decision.
6. No custom Runen shader language or custom source preprocessor is authorized.

## Why the distinction is required

A shader system contains several different kinds of authority that must not collapse:

```text
semantic meaning
human-authored source
module/import graph
compiler configuration
compiled portable artifact
entry-point and interface contract
backend shader module
pipeline realization
product reload and fallback state
```

Treating all of these as "the shader" would create one of two failures:

- RunenGPU would absorb filesystem, compiler, editor, package and product policy that
  prevents standalone extraction; or
- RunenRender would retain duplicate binding, interface, module and pipeline authority
  that G4 is explicitly removing.

The boundary therefore separates source production from GPU admission without creating
parallel runtime paths.

# Ownership

## RunenRender or another consumer owns meaning

RunenRender owns semantic rendering decisions, including:

- material, medium, lighting, transport and image-formation meaning;
- which semantic shader or kernel family is required;
- renderer-owned variants and feature combinations;
- semantic lowering from prepared render state to generic RunenGPU program, resource
  and work descriptors.

A non-render consumer owns its own kernel meaning. Simulation, SDF processing, image
processing and other GPU users may use RunenGPU without depending on RunenRender.

Neither RunenRender nor another consumer owns final GPU binding compatibility, backend
shader modules, pipeline layouts or WGPU pipelines.

## Runenwerk owns authoring and toolchain policy

Runenwerk retains:

- shader source roots and filesystem paths;
- module and package resolution policy;
- compiler executable or library selection and pinning;
- authoring-language version and compiler options;
- deterministic translation or compilation;
- source dependency graphs and invalidation;
- generated-artifact provenance and source maps;
- file watching and reload scheduling;
- atomic artifact publication;
- last-known-good artifacts and product fallback;
- developer diagnostics presentation;
- build-time versus development-time compiler invocation;
- persisted artifact, capture and reproducibility policy.

These concerns must not enter the future-transferable RunenGPU public API.

## RunenGPU owns admitted GPU contracts

RunenGPU owns:

- source keys, nonzero revisions and full canonical source consistency;
- the initial WGSL runtime source kind;
- typed compute, vertex and fragment entry points;
- typed `(group, binding)` identities;
- explicit binding declarations;
- explicit program interfaces;
- bind-group and pipeline-layout descriptors;
- specialization schemas and normalized values;
- generic compute and render pipeline descriptors;
- interface and runtime-binding compatibility validation;
- context/device-generation-bound module, layout, bind-group and pipeline realization;
- correctness-complete realization/cache keys and structured rejection.

A canonical WGSL artifact is not authoritative merely because a frontend generated it.
RunenGPU still validates source identity, declared entry points, explicit interfaces,
requirements and runtime binding compatibility.

## Private WGPU/Naga realization owns backend translation

The selected WGPU version may parse and translate accepted source for Vulkan, Metal,
Direct3D, OpenGL or browser WebGPU backends. That translation remains private backend
implementation.

Naga IR, backend shader modules and backend compiler outputs are not public identity,
persistence, replay, wire or cache formats.

# Canonical artifact contract

## Canonical WGSL

The first and only initial runtime artifact is complete WGSL source text accepted by the
pinned RunenGPU/WGPU implementation.

Canonical WGSL may come from:

- directly authored `.wgsl` source;
- a separately authorized WESL translation;
- a separately authorized Slang translation;
- a future separately authorized source frontend.

The source frontend does not change RunenGPU admission. Every accepted frontend must
produce the same artifact class:

```text
canonical WGSL source
+ semantic source key
+ nonzero source revision
+ full source-content identity
+ declared entry points
+ explicit RunenGPU interface reference
+ normalized requirements
+ bounded provenance
```

## Runtime identity

Filesystem paths, module names, package locations and compiler-specific handles are not
runtime source identity.

RunenGPU source identity continues to use:

- an opaque semantic `GpuProgramSourceKey`;
- a nonzero owner-local `GpuProgramSourceRevision`;
- full canonical WGSL source equality;
- a deterministic source digest for hashing and diagnostics only.

Reusing one key and revision with different canonical WGSL bytes remains a structured
`SourceRevisionConflict`.

## Build artifact envelope

A future Runenwerk-owned shader build artifact may record:

- authoring-language kind and version;
- compiler identity and version;
- normalized compiler options;
- module/dependency graph digest;
- canonical WGSL bytes and digest;
- source-map or diagnostic remapping data;
- declared entry points;
- expected explicit interface identity;
- provenance required for reproduction.

This is not a RunenGPU stable format. Any persisted envelope requires separate
Runenwerk-owned versioning and migration authority.

# Interface authority

## One explicit interface

The explicit RunenGPU program interface remains authoritative.

The following may provide evidence but cannot become competing authority:

```text
WESL declarations
Slang reflection
WGSL parser or backend reflection
Rust derives
material metadata
renderer pipeline keys
WGPU inferred layouts
```

Compiler or backend reflection may:

- prove that generated WGSL agrees with the explicit interface;
- identify an exact mismatch;
- improve diagnostics and source locations.

Reflection may not:

- silently assign or renumber binding groups or bindings;
- add or remove declarations from the explicit interface;
- infer host memory ABI as accepted truth;
- replace typed stage visibility, resource kind, format or array shape;
- mutate a pipeline layout after descriptor admission;
- allow one authoring frontend to produce a different runtime contract silently.

A disagreement is `ProgramInterfaceMismatch` or a more specific structured rejection.

## Rust byte preparation

Current `GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`, `GpuUniformField` and
associated derives remain transitional byte-preparation helpers under the accepted G4B
plan.

They do not prove:

- shader struct identity;
- binding group or binding identity;
- visibility;
- nested, array or runtime-array layout;
- matrix orientation;
- minimum binding size;
- storage ABI;
- complete program-interface compatibility.

No macro package or universal shader ABI derive is authorized by this design.

# Language disposition

## Plain WGSL

Plain WGSL is the bootstrap and runtime baseline because WebGPU shader modules use WGSL
source and WGPU enables WGSL input by default. WGPU's Rust API can optionally accept
additional source representations, but those are not equivalent to WebGPU's portable
source contract.

Plain WGSL remains appropriate for:

- small programs;
- generated programs;
- conformance fixtures;
- minimal examples;
- toolchain-independent fallback and debugging.

Its limited standardized module composition is an authoring concern, not a reason to
add more RunenGPU source variants.

## WESL

WESL is the preferred near-term modular authoring candidate because it extends WGSL
with module imports and related source-composition features, then translates those
enhancements to ordinary WGSL before WebGPU shader-module creation.

A first WESL proof should authorize only the smallest required surface:

- deterministic imports and module linking;
- explicit module roots;
- bounded conditional translation where a current consumer proves it;
- generated canonical WGSL;
- source-map or equivalent diagnostic remapping;
- complete dependency invalidation.

It should not initially authorize:

- network package resolution during builds or runtime;
- runtime compiler invocation in the render loop;
- experimental WESL extensions without separate evidence;
- Runenwerk-specific syntax extensions;
- a second direct runtime source path.

Production applications should be able to consume prebuilt canonical WGSL without
shipping the WESL compiler unless a separately accepted product requirement proves the
need.

## Slang

Slang is the preferred future advanced frontend candidate because its official language
and compiler provide modules, interfaces, generics, parameter blocks, specialization,
reflection and multiple target outputs.

It is not the baseline now. Official Slang project documentation currently classifies
Metal and WebGPU support as experimental and WGSL output as work in progress. Those are
precisely the paths required for RunenGPU's portable native-and-browser profile.

A future Slang integration must therefore compile outside RunenGPU:

```text
Slang source
    -> pinned Runenwerk-owned Slang compiler
        -> canonical WGSL
            -> explicit RunenGPU validation and realization
```

Slang reflection remains evidence. It does not replace the explicit RunenGPU interface.

A native Slang-to-SPIR-V path is not authorized as a parallel baseline. It may be
reconsidered only for an explicitly backend-specialized portability class with separate
need, proof and containment.

## HLSL, GLSL and SPIR-V

HLSL, GLSL and SPIR-V remain potential import or migration inputs only when actual
source assets justify them.

They do not become initial RunenGPU source kinds because that would:

- broaden the validation and compiler surface before a consumer proves need;
- create different browser and native paths;
- introduce language-specific layout, binding and specialization rules;
- weaken the single portable WGSL artifact boundary.

## Naga IR and backend outputs

Naga IR, SPIR-V generated internally for Vulkan, MSL generated for Metal, HLSL or DXIL
used by Direct3D, and GLSL generated for OpenGL/WebGL are private implementation facts.

They must not be serialized or exposed as stable RunenGPU source, cache, replay or
interchange authority.

## Custom language

No custom Runen shader language, parser, preprocessor or compiler is authorized.
Existing standards and toolchains must be exhausted before accepting the permanent
maintenance burden of custom language design, diagnostics, editor support, formatting,
debugging and cross-backend conformance.

# Frontend promotion gate

Any authoring frontend other than plain WGSL requires a separately accepted proof before
it becomes a supported Runenwerk path.

The proof must cover at least:

1. Pinned frontend compiler/version and normalized options.
2. Deterministic canonical WGSL generation or documented normalization before digesting.
3. Complete dependency discovery and invalidation.
4. Source diagnostics mapped back to authoring files and spans.
5. Explicit group/binding preservation.
6. Entry-point and stage preservation.
7. Reflection agreement with the explicit RunenGPU interface.
8. Host-buffer layout, matrix orientation, array and structure conformance.
9. Specialization normalization and deterministic artifact identity.
10. Successful validation by the pinned WGPU/Naga version.
11. Representative compute and render execution through Vulkan, Direct3D 12, Metal and
    browser WebGPU where supported by the repository's target matrix.
12. Structured rejection of unsupported frontend or target features before publication.
13. Atomic reload that preserves the last-known-good artifact after translation,
    validation or realization failure.
14. Reproducibility facts sufficient for a Runenwerk-owned build/capture envelope.
15. No second runtime admission, interface or pipeline authority.

A frontend remains experimental until every mandatory target in its declared profile
passes. Missing target support must be represented explicitly; it cannot be described
as portable.

# G4B and later-phase consequences

## G4B remains bounded

G4B should implement only the accepted WGSL-first program, interface, binding, layout,
specialization and generic pipeline contracts.

G4B must not:

- add WESL, Slang, HLSL, GLSL or SPIR-V dependencies;
- implement module discovery or compiler execution;
- add authoring-language paths to `GpuProgramSource`;
- create persisted shader artifact schemas;
- implement reflection as sole interface authority;
- add a macro package or universal shader ABI derive.

The generic public vocabulary should remain source-language-neutral where practical:

```text
GpuProgram
GpuProgramSource
GpuProgramEntryPoint
GpuProgramInterface
GpuComputePipelineDescriptor
GpuRenderPipelineDescriptor
```

WGSL is the only initial concrete source kind, not the semantic identity of the whole
framework.

## G4C remains backend realization

G4C consumes the canonical WGSL and explicit G4B descriptors to create private WGPU
shader modules, layouts, bind groups and pipelines.

Cache correctness includes canonical WGSL identity, explicit interface and pipeline
descriptors, context/device generation, enabled features, effective limits and pinned
backend compatibility facts. Authoring file paths and diagnostic source maps do not
split otherwise identical backend objects.

## Later RunenRender authoring

A later material graph, shader graph or renderer composition system remains
RunenRender-owned semantic authoring. It lowers to canonical program artifacts and
explicit interfaces rather than becoming a universal RunenGPU IR.

# Stop conditions

Stop and require a separate architecture decision if:

- a frontend cannot produce canonical WGSL for the declared portable profile;
- correct use requires a second runtime source or pipeline path;
- reflection must become the sole interface authority;
- an accepted stable source, artifact, ABI, cache, replay or wire format already exists;
- compiler policy cannot remain outside RunenGPU;
- frontend adoption requires RunenRender meaning inside RunenGPU;
- a custom shader language or Runenwerk-specific syntax extension appears necessary;
- mandatory target conformance cannot be represented honestly;
- generated WGSL or bindings are nondeterministic and cannot be normalized safely.

# References

Official sources reviewed on 2026-08-02:

- WebGPU/WGPU shader source model:
  `https://docs.rs/wgpu/latest/wgpu/enum.ShaderSource.html`
- WESL specification overview:
  `https://wesl-lang.dev/spec/README`
- WESL imports:
  `https://wesl-lang.dev/spec/Imports`
- Slang repository support matrix:
  `https://github.com/shader-slang/slang`
- Slang supported compilation targets:
  `https://shader-slang.org/slang/user-guide/targets`
- Slang modules and access control:
  `https://shader-slang.org/slang/user-guide/modules`
- Slang interfaces and generics:
  `https://docs.shader-slang.org/en/latest/external/slang/docs/user-guide/06-interfaces-generics.html`
- Slang reflection API:
  `https://shader-slang.org/slang/user-guide/reflection.html`

These external sources justify frontend evaluation only. Dornglut ownership and runtime
contracts remain governed by repository authority.
