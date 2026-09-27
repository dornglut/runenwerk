---
title: Renderer Temporal Reconstruction And Dynamic Resolution Platform
description: Active design for TAA, TAAU, fixed and dynamic internal resolution, motion-vector/depth/exposure products, and FSR-style optional adapters.
status: accepted
owner: engine
layer: engine-runtime / renderer / postprocess
canonical: true
last_reviewed: 2026-09-27
publication: reference
pagefind: false
related_designs:
  - ./render-product-graph-platform-design.md
  - ../implemented/render-product-surface-foundation-bundle-design.md
  - ./renderer-scale-residency-and-gpu-driven-visibility-design.md
  - ./sdf-world-rendering-and-raymarch-acceleration-design.md
---

# Renderer Temporal Reconstruction And Dynamic Resolution Platform

## Decision

Temporal reconstruction is a renderer platform, not a vendor-upscaler shortcut.
The first durable contract is backend-neutral history, jitter, motion vectors,
depth, exposure, reactive-mask style metadata, explicit native/fixed/dynamic
internal-resolution policy, and diagnostics. FSR-style or vendor-specific adapters come after the portable
contract exists.

The renderer may own derived temporal execution state: history textures,
history signatures, jitter phase, fixed/dynamic internal-resolution state,
reconstruction mode, timing, capability diagnostics, and adapter invocation records. Product,
scene, camera, material, SDF, ray-query, and exposure producers remain the
source of semantic truth for their inputs.

## RunenRender Semantic Sampling Boundary

Temporal reconstruction preserves the RunenRender distinction between requested semantic
support and the algorithm used to evaluate it.

For an image-like request, the unjittered renderer observation remains semantic truth.
When the requested sampling policy represents a perspective lattice-cell / pixel
footprint, the concrete support of one logical sample is derived from that observation
and the exact requested output lattice. Physical output pixels, the current internal
resolution, and a GPU viewport do not define that semantic footprint.

A temporal jitter phase is therefore renderer method/evaluation state:

```text
unjittered semantic observation
+ requested lattice/support
    -> semantic logical-sample support

semantic support
+ method-owned sample sequence / phase
    -> finite temporal samples

finite temporal samples
+ valid retained history
+ method-specific reconstruction evidence
    -> reconstructed finite estimate
```

Jitter must not be implemented by mutating source-owned camera truth or by silently
changing `observation_to_scene`. One offset finite sample does not by itself complete a
non-point footprint result. Any reconstructed semantic result must satisfy the owning
RunenRender method's finite-evaluation/result-formation contract.

The first portable temporal implementation may be **static-first**: a static scene and
unchanged semantic observation may reuse bounded retained color history while sampling
different admitted subpixel phases. Any incompatible scene, observation, requested
topology/support, reconstruction-method, sample-sequence, or consumed-input-generation
change invalidates that history and restarts evaluation.

That static-first proof establishes sampling/history/reconstruction mechanics only. It
does not establish moving-camera or moving-object reprojection, disocclusion handling,
motion-aware TAA, or TAAU. Those claims require truthful depth/motion/reprojection inputs
in a later accepted slice.

## Scope

This track covers:

- TAA and TAAU history workflows;
- jittered projection and history invalidation;
- motion vectors, depth, exposure, luminance, transparency/reactive masks, and
  disocclusion diagnostics;
- fixed or dynamically adapted internal render resolution separate from output resolution;
- raymarch and ray-query reconstruction inputs;
- optional FSR-style adapter hooks and unsupported capability diagnostics.

It does not make FSR, DLSS, XeSS, frame generation, or any vendor technology a
required baseline renderer path.

It also does not move camera truth, product freshness, exposure authority,
material reactivity semantics, SDF query policy, or ray-query acceleration
resource ownership into the renderer.

## Ownership Boundaries

- `engine/src/plugins/render` owns temporal reconstruction execution contracts,
  history allocation, jitter application, fixed/dynamic internal resolution,
  upscaler capability diagnostics, pass timing, and production evidence DTOs.
- Product Graph and product-surface producers own product lineage, freshness,
  authority class, fallback legality, and semantic availability of motion,
  depth, exposure, reactive, SDF, and ray-query inputs.
- Camera/scene producers own view/projection source truth. The renderer may
  consume prepared matrices and jitter offsets but must not become the canonical
  camera system.
- Optional adapter integrations own only adapter invocation and capability
  translation. Unsupported adapters must report typed diagnostics and fall back
  to portable native/TAA/TAAU behavior.

## Required Contracts

Temporal implementation rows must introduce explicit typed evidence for:

- output resolution versus internal render resolution, including explicit native,
  fixed, or dynamic resolution policy;
- fixed execution routing when active: selected flow identity, explicit bindable
  color-target alias, internal offscreen view/target identity, native-output
  resolve invocation, and producer-owned automatic-main replacement;
- jitter sequence identity and current jitter phase;
- history resource identity, signature, age, and invalidation reason;
- motion-vector, depth, exposure, luminance, reactive-mask, SDF, and ray-query
  input availability;
- reconstruction mode, native fallback mode, and adapter capability state;
- CPU/GPU timing and quality diagnostics;
- artifact paths, benchmark commands, and visible evidence for production
  closeout.

History validity is signature-keyed. A history sample is invalid when the retained
state's dependencies no longer match the prepared frame contract. For temporal
reconstruction those dependencies include the relevant scene state, unjittered
semantic observation, exact requested output topology and sampling support,
reconstruction method/revision, sample-sequence identity, and consumed input-product
generations. Internal/output extent and adapter capability are included whenever the
chosen realization depends on them. These derived compatibility facts do not become
semantic camera or product identity.

## Invariants

- Native rendering and portable TAA/TAAU remain the baseline. Vendor adapters
  are optional capability paths and cannot be required for correctness.
- Internal resolution is always reported separately from output resolution.
  Native, fixed, and dynamically adapted resolution are distinct typed policies;
  none may hide quality, timing, or fallback state.
- Fixed execution is scoped to one explicit render surface. It may replace only
  the selected alias-capable scene flow's automatic main invocation on that
  surface; other attached surfaces retain their own native routing. Fixed
  target/view/invocation identity includes the render-surface identity so
  multi-surface hosts cannot alias one fixed-resolution execution across windows.
  Every pass in the selected flow must be valid on both native-main and offscreen
  views, and the flow must route its selected color output exclusively through
  the bindable alias rather than hard-coding builtin `SurfaceColor`. Fixed
  execution and explicit native fallback therefore share one scene-flow
  definition. Unrelated flows, including UI, remain on their native-output path.
- Renderer helper/resolve flows that are meaningful only when explicitly
  invoked use typed explicit-invocation flow policy; they must not rely on a
  global disable-default-flows switch or registration churn.
- Rejected fixed execution falls back to native resolution before any partial
  internal-routing publication. When the selected alias contract is valid, the
  fallback publishes an explicit native-main invocation binding that alias to
  native `SurfaceColor`; execution evidence must reject retained fixed
  target/view/resolve state. A simple spatial resolve proves portable plumbing
  only and is not TAAU reconstruction-quality evidence.
- A reconstruction mode must explicitly declare the temporal inputs it requires.
  Missing required motion vectors, depth, exposure, reactive masks, SDF, ray-query,
  or adapter capability produces typed diagnostics rather than silent reconstruction.
  A bounded static-first mode may explicitly make motion/reprojection inputs
  inapplicable only while scene and semantic observation remain unchanged; any such
  change invalidates its history and it must not claim motion-aware TAA/TAAU.
- History reuse must fail closed on signature mismatch, missing required inputs,
  disocclusion risk where applicable, or invalidated producer generations.
- Temporal output cannot claim product freshness or scene authority beyond the
  prepared input evidence it consumed.

## Sequence

The accepted implementation sequence is:

1. `WR-070`: temporal inputs, history validity, jitter, diagnostics,
   native/fixed/dynamic internal/output resolution separation, and the bounded
   fixed internal-resolution execution path with explicit native fallback.
2. `WR-071`: optional upscaling adapters and ray reconstruction inputs,
   capability-gated with explicit unsupported diagnostics.
3. `WR-072`: temporal production evidence with examples, benchmark/report
   artifacts, public docs, timing, quality, and fallback proof.

## Evidence

Runtime evidence must report internal/output resolution and resolution policy, history validity,
motion-vector availability, reconstruction mode, upscaler capability state,
GPU timing, quality diagnostics, and fallback to native resolution when inputs
or backend capabilities are unavailable.

Completion cannot claim `runtime_proven` until the implementation rows provide
focused tests, examples, benchmark evidence, public docs, closeouts, and
production-track metadata. Completion cannot claim `perfectionist_verified`
until the final renderer production audit proves no known quality gaps remain.
