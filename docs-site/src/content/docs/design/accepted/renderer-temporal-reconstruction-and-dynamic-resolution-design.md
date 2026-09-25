---
title: Renderer Temporal Reconstruction And Dynamic Resolution Platform
description: Active design for TAA, TAAU, fixed and dynamic internal resolution, motion-vector/depth/exposure products, and FSR-style optional adapters.
status: accepted
owner: engine
layer: engine-runtime / renderer / postprocess
canonical: true
last_reviewed: 2026-05-23
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

History validity is signature-keyed. A history sample is invalid when viewport
identity, output resolution, internal resolution, camera projection, jitter
sequence, reconstruction mode, input-product generation, or adapter capability
state no longer matches the prepared frame contract.

## Invariants

- Native rendering and portable TAA/TAAU remain the baseline. Vendor adapters
  are optional capability paths and cannot be required for correctness.
- Internal resolution is always reported separately from output resolution.
  Native, fixed, and dynamically adapted resolution are distinct typed policies;
  none may hide quality, timing, or fallback state.
- Fixed execution may replace only the selected alias-capable scene flow's
  automatic main invocation. Unrelated flows, including UI, remain on their
  native-output path.
- Renderer helper/resolve flows that are meaningful only when explicitly
  invoked use typed explicit-invocation flow policy; they must not rely on a
  global disable-default-flows switch or registration churn.
- Rejected fixed execution falls back to native resolution before any partial
  internal-routing publication. A simple spatial resolve proves portable
  plumbing only and is not TAAU reconstruction-quality evidence.
- Missing motion vectors, depth, exposure, reactive masks, SDF, ray-query, or
  adapter capability must produce typed diagnostics, not silent reconstruction.
- History reuse must fail closed on signature mismatch, missing inputs,
  disocclusion risk, or invalidated producer generations.
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
