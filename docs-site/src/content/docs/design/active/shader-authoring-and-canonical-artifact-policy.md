---
title: Shader Authoring and Canonical Artifact Policy
description: Runenwerk-owned policy for shader source authoring, deterministic canonical WGSL production, reload/publication, diagnostics, and persisted artifact provenance above standalone RunenGPU.
status: active
owner: workspace
layer: tooling
canonical: true
last_reviewed: 2026-09-13
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../accepted/runenrender-decomposition-design.md
  - ./runenrender-internal-decomposition-execution-plan.md
  - ../../reports/design/runengpu-phase-requirements-proof-matrix.md
---

# Shader Authoring and Canonical Artifact Policy

## Purpose and authority

This document owns only the Runenwerk side of shader and GPU-program authoring:

```text
human or generated authoring source
    -> Runenwerk-owned toolchain and publication policy
        -> deterministic canonical WGSL artifact
            -> standalone RunenGPU public program admission
```

Reusable runtime GPU program admission, compiler-derived shader facts, effective
program-interface semantics, capability derivation, layout/binding compatibility,
pipeline realization, execution, and backend caches belong exclusively to
[`dornglut/runen-gpu`](https://github.com/dornglut/runen-gpu/blob/main/ARCHITECTURE.md).
Runenwerk must not maintain a shadow copy of those contracts.

RunenRender owns renderer shader/kernel meaning and semantic renderer variants. A
non-render consumer owns its own kernel meaning. Runenwerk owns how authoring sources
are discovered, transformed, published, diagnosed, reproduced, and integrated into
products.

The current Runenwerk workspace consumes `runen-gpu` at an exact accepted revision.
That exact dependency is the Runenwerk compatibility claim; a moving standalone branch
is not. Standalone RunenGPU may evolve independently under its own repository authority.

## Runenwerk-owned authoring and toolchain policy

Runenwerk owns:

- shader source roots and filesystem paths;
- module/package resolution for authoring sources;
- frontend/compiler selection, pinned version, options, and target profile;
- deterministic translation into canonical WGSL;
- source dependency graphs and complete invalidation;
- generated-artifact provenance and source maps;
- file watching and reload scheduling;
- atomic publication and last-known-good product fallback;
- developer diagnostic presentation;
- build-time versus development-time compiler invocation;
- product trust policy for generated or third-party source;
- persisted build/capture/reproducibility envelope policy;
- artifact retention, redaction, migration, and support-bundle policy.

These concerns remain above RunenGPU. They do not become public RunenGPU source kinds,
package-resolution contracts, compiler APIs, cache identities, or backend semantics.

## Boundary to standalone RunenGPU

Runenwerk supplies only artifacts and admission choices accepted by the standalone
RunenGPU public contract. It does not redefine:

- `GpuProgramSourceKey` or source-revision consistency;
- which shader facts are compiler-derived;
- program-interface, binding, stage-I/O, or layout semantics;
- capability requirements derived from an admitted program;
- specialization, pipeline, or runtime-binding compatibility;
- context/device-generation-bound realization;
- backend/private compiler representation;
- execution, submission, readback, surface, or device-loss semantics.

When Runenwerk needs a reusable runtime GPU-program capability, that capability must be
changed in `dornglut/runen-gpu` under its own authority. A Runenwerk adapter may prepare
or translate product/source facts, but it may not become a second program-admission
implementation.

## Canonical runtime artifact

Canonical WGSL is the portable runtime artifact boundary used by current RunenGPU.
Runenwerk authoring may start from plain WGSL or from a separately accepted frontend,
but the frontend/toolchain step completes before RunenGPU runtime admission.

Runenwerk therefore treats the authoring artifact as:

```text
authoring sources
+ pinned frontend/compiler identity and options
+ complete dependency graph
    -> deterministic canonical WGSL bytes
        + source-map/provenance evidence
```

Filesystem paths, package names, compiler handles, frontend IR, and backend-native
shader modules are authoring/build facts. They are not silently promoted into reusable
RunenGPU runtime identity.

A generated artifact is not considered publishable merely because translation
succeeded. It must still be accepted by the standalone RunenGPU public boundary used by
the product.

## Authoring-language disposition

### Plain WGSL

Plain WGSL remains the baseline for direct authoring, generated programs, minimal
fixtures, fallback, and debugging. It is also the canonical runtime output expected from
higher-level authoring frontends.

### WESL

WESL remains the preferred near-term modular-authoring candidate because it can lower
module composition to canonical WGSL before runtime admission. Adoption requires a
separate bounded proof and does not make WESL a RunenGPU runtime source variant.

A valid proof must establish deterministic imports/module linking, explicit module
roots, complete dependency invalidation, deterministic WGSL, source-mapped diagnostics,
and compatibility with the current standalone RunenGPU admission boundary.

### Slang

Slang remains a future advanced-authoring candidate for modules, interfaces, generics,
specialization, and richer frontend tooling. It is not a baseline dependency until a
separately accepted proof establishes the required portable WGSL/WebGPU and product
platform paths.

Any future relationship remains:

```text
Slang source
    -> pinned Runenwerk-owned Slang toolchain
        -> canonical WGSL
            -> standalone RunenGPU admission
```

Slang reflection or frontend IR may assist authoring diagnostics. It does not become a
second runtime program-interface authority.

### Other source and IR forms

HLSL, GLSL, SPIR-V, and other source/import forms require actual product pressure and a
separately accepted authoring path. Backend/compiler IR and generated backend-native
modules remain implementation/tooling detail rather than stable Runenwerk or RunenGPU
interchange authority.

No custom Runen shader language, parser, preprocessor, or syntax extension is authorized
by this policy.

## Frontend promotion gate

A frontend beyond direct WGSL is accepted only after a bounded owner proves all
applicable items:

1. pinned compiler/frontend version and options;
2. deterministic canonical WGSL generation or normalization;
3. complete dependency discovery and invalidation;
4. diagnostics mapped back to authoring files/spans;
5. preserved entry-point and binding identities required by the generated WGSL;
6. successful standalone RunenGPU program admission;
7. required host-buffer, matrix, array, structure, and specialization conformance;
8. validation against the product's pinned runtime compiler/backend stack;
9. representative compute/render execution for the declared target profile;
10. structured unsupported-feature rejection before publication;
11. atomic reload that preserves the last-known-good artifact after failure;
12. reproduction facts sufficient for the owning Runenwerk artifact envelope;
13. no second runtime source/interface/pipeline authority;
14. no render-loop network/package resolution or unbounded compiler invocation;
15. no transfer of RunenRender or application semantics into RunenGPU.

An experimental frontend remains tooling evidence until its declared mandatory profile
passes. Its existence does not authorize a new runtime source kind.

## Reload and publication

Development reload is a Runenwerk product/tooling workflow:

```text
source change
    -> dependency invalidation
    -> deterministic rebuild
    -> canonical WGSL
    -> standalone RunenGPU admission
    -> atomic product publication
```

A failed rebuild or admission must not replace the last-known-good published artifact.
Diagnostics retain source provenance and should map back to authoring locations where the
frontend can provide that mapping.

Watching, debounce/coalescing, retry behavior, publication scheduling, user-facing
severity, and last-known-good policy remain product concerns. RunenGPU may report
structured admission/runtime failures but does not own Runenwerk reload UX or recovery.

## Persisted build and reproducibility artifacts

If Runenwerk persists a shader/program build envelope, that envelope is a Runenwerk-owned
versioned artifact. It may record:

- authoring-language and compiler/frontend identity;
- pinned compiler version/options/target profile;
- complete source/dependency digest;
- canonical WGSL and its digest;
- source maps and source provenance;
- selected product-specific admission inputs where required for reproduction;
- exact integrated RunenGPU revision;
- schema/version, migration, validation, retention, and redaction policy.

Such an envelope is not a stable RunenGPU format merely because it contains data later
consumed by RunenGPU. Runtime handles, backend objects, private compiler IR, and
unversioned diagnostics are not persistence identity.

## Stop conditions

Stop and require a new architecture decision if correct authoring would require:

- a second runtime program-admission or program-interface authority in Runenwerk;
- exposing private RunenGPU compiler/backend types as authoring authority;
- moving source roots, package resolution, file watching, product reload, or persisted
  build policy into RunenGPU;
- moving renderer semantic shader meaning into RunenGPU;
- a custom Runen shader language or Runenwerk-specific source syntax;
- nondeterministic generated WGSL that cannot be normalized safely;
- runtime network package resolution or unbounded compilation in latency-critical loops;
- compatibility aliases or parallel source paths merely to preserve predecessor shape.

## Historical predecessor

The former `runengpu-shader-authoring-artifact-boundary.md` mixed Runenwerk authoring
policy with RunenGPU runtime-admission semantics before the standalone authority
transfer. Issue #494 retired that mixed owner after proving the reusable runtime side is
represented by `dornglut/runen-gpu`. The exact predecessor remains available through Git
history as provenance; it is not current RunenGPU authority.
