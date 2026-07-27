---
title: RunenGPU Proof Workload Strategy
description: Critical workload selection for deterministic conformance, boundary integration, operational pressure, recovery, performance characterization, visual showcases, offline output, and later RunenRender proofs.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-27
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
  - ./runengpu-render-s0-file-disposition.md
  - ./runengpu-public-api-ergonomics-review.md
  - ./runengpu-industry-comparison.md
  - ./runengpu-runenrender-application-domain-fit.md
---

# RunenGPU Proof Workload Strategy

## Decision summary

RunenGPU must not rely on one visually impressive example as proof of correctness.
The proof portfolio has six distinct roles:

```text
deterministic conformance
    exact results and narrow failure diagnosis

boundary integration
    multiple resources and stages crossing accepted owners

operational pressure
    quotas, saturation, progress, cancellation, and shutdown

recovery and reproducibility
    generations, loss, reconstruction, and capture bundles

performance characterization
    cost dimensions and narrow direct-WGPU comparison

visual showcase
    representative application value with tolerant evidence
```

Correctness gates are never replaced by performance or visual evidence.

## Selection criteria

A workload is evaluated against:

1. determinism;
2. boundary coverage;
3. isolation and diagnosis quality;
4. portability across WGPU backends and WebGPU where relevant;
5. current repository reuse;
6. human interpretability;
7. complexity cost;
8. pressure/recovery coverage;
9. ability to compare the framework path with a narrow direct backend path;
10. freedom from product, ECS, window, codec, and domain ownership.

No single workload scores best across all criteria. The portfolio is deliberately a
ladder.

## Current repository candidates

### Prefix scan and compaction

Current GPU primitives already include counters, inclusive/exclusive `u32` prefix
scan, compaction/scatter support, generated indirect arguments, temporary storage,
and multi-stage dispatch plans.

**Role:** required deterministic G5 conformance and G6 GPU-driven composition.

**Why:** exact integer output, current ownership fit, multiple stages, temporary
resources, completion, and readback.

### Game of Life

The current example provides deterministic integer state, fixed seed/tick behavior,
ping-pong storage, compute dispatch, and a simple visualization.

**Role:** required stateful G5 integration and first deterministic offline sequence.

**Why:** repeated submissions, exact CPU oracle, checksums, selected cells, and clear
source/runtime ownership separation.

### Known image processing

A known-image convolution, Sobel filter, or small reconstruction pipeline exercises
texture upload, storage/sampled use, intermediates, copies, padded readback, and
artifact encoding without a scene renderer.

**Role:** conditional G5 texture proof and narrow direct-WGPU performance comparison.

### Known-pattern offscreen draw

A small offscreen render with known clear and selected pixels isolates graphics
pipeline creation, attachments, draw, copy, and readback.

**Role:** required G6 graphics conformance.

### Compute-generated indirect draw

Compute produces arguments and data consumed by graphics, with order inferred from
resource access.

**Role:** required G6 composition proof.

### Boids

The current boids example exercises fixed-step simulation, double-buffered agents,
spatial-grid construction, atomic counts, prefix scan, scatter, neighborhood
simulation, compute-to-graphics sharing, instanced drawing, and presentation.

**Role:** representative G6 integration and G7 surface showcase.

Boids is not the primary correctness oracle because atomic and floating-point
behavior make exact cross-backend state or pixel equality inappropriate.

### Procedural sky and SDF terrain

The current procedural terrain example is the first bounded RunenRender semantic
proof after accepted RunenGPU cutover.

**Role:** first image-formation proof. It must not define RunenGPU concepts.

### Synthetic volume

A fixed integer volume with a deterministic transfer function and selected CPU ray
oracle exercises volume providers, interval integration, multi-target output, changed
bricks, and memory pressure.

**Role:** later RunenRender provider and application-domain proof.

## Accepted correctness ladder

### G5 — headless execution and transfers

#### Required deterministic conformance

**`u32` prefix scan and readback**

- fixed integer input;
- inclusive and exclusive expected outputs;
- exactly 4,097 elements so the proof crosses the accepted workgroup hierarchy;
- temporary scan storage;
- exact full-output and total-count verification;
- no window, renderer, ECS, or product types.

#### Required stateful integration

**Headless Game of Life**

- fixed accepted seed;
- dimensions exactly 160×90;
- exactly 16 steps;
- ping-pong storage;
- full-grid CPU oracle;
- exact live-cell count `2,063`;
- exact FNV-1a-64 checksum `0xBD710B88594CD584`;
- accepted selected-cell assertions;
- asynchronous completion and readback;
- source state prepared outside RunenGPU.

#### Conditional texture proof

**Known integer compute-to-texture artifact**

- deterministic pattern;
- texture-to-buffer readback;
- row-padding normalization;
- Runenwerk-owned PNG encoding;
- exact or documented integer result checks.

#### Optional extensions

- reduction;
- histogram;
- image convolution;
- compaction output.

### G6 — offscreen graphics and shared consumers

#### Required graphics conformance

**Offscreen known-pattern draw**

- no surface;
- one graphics pipeline;
- known clear and draw result;
- selected-pixel readback;
- exact attachment/load/store evidence.

#### Required GPU-driven composition

**Compute-generated indirect draw**

- compute-generated arguments and source data;
- graphics consumes them;
- dependency inferred from accepted G3 access;
- structural and selected-pixel assertions.

#### Required representative integration

**Offscreen boids**

- accepted simulation and draw path migrated through adapters;
- one shared RunenGPU context;
- no surface requirement;
- bounded finite state and resource invariants;
- tolerant artifact comparison;
- no exact cross-backend floating-point pixel promise.

### G7 — surfaces and lifecycle outcomes

Reuse accepted G6 workloads:

- present known-pattern draw;
- present boids interactively;
- resize and reconfigure surfaces;
- test outdated/lost/out-of-memory outcomes;
- test multiple surfaces where supported;
- prove offscreen and surface targets share semantic work rather than duplicate paths.

### G8 and GX — retained conformance

The standalone suite retains the narrow G5/G6 proofs. Unsupported hardware paths
produce structured skip/environment evidence. A showcase never replaces exact
contract tests.

## Operational pressure matrix

These proofs begin only in the owning phase.

### G5 progress and saturation

#### In-flight submission saturation

- configure a small accepted submission quota;
- fill it with valid work;
- assert the next submission returns the documented pressure outcome;
- drive progress;
- assert capacity becomes available;
- prove no accepted submission loses its terminal outcome.

#### Pending-readback saturation

- configure a small readback pool/quota;
- submit multiple readbacks without consuming completions;
- assert deterministic rejection or bounded wait behavior;
- complete and release readbacks;
- prove backing memory returns to the pool.

#### Upload/staging pressure

- exceed the configured staging budget with valid data;
- assert structured pressure rather than unbounded growth or silent drop;
- prove accepted uploads remain intact.

#### Callback/reentrancy proof

- completion callbacks attempt allowed follow-up work;
- internal locks are not held during consumer callback invocation;
- callback ordering and exactly-once delivery are documented;
- native polling and WebGPU event-loop paths reach equivalent terminal semantics.

#### Shutdown with pending work

- accept submissions and readbacks;
- begin shutdown at defined lifecycle points;
- prove every accepted item receives completion, cancellation, loss, or shutdown
  outcome exactly once;
- prove no indefinite wait is required by the API contract.

#### Cancellation lifecycle

Test cancellation:

```text
before preparation
before backend submission
after backend submission
while readback mapping is pending
during shutdown
```

The terminal meaning differs by point but is never ambiguous.

## Cache and pipeline characterization

### G4 compatibility proof

- create cache facts for one admitted adapter/context;
- reject altered WGPU version, backend, adapter, driver, shader, interface,
  capability, or descriptor facts;
- accept matching facts where WGPU/backend permits;
- prove cache rejection falls back safely and preserves semantics.

### G6 cold/warm proof

For known compute and graphics pipelines, record:

- source preparation time;
- shader admission/translation time;
- pipeline creation time;
- first submission latency;
- warm repeated submission latency;
- cache hit/miss/rejection facts;
- artifact equality.

No global performance threshold is a correctness gate. Regressions require separately
accepted budgets and controlled environments.

## Direct-WGPU comparison

At least these workloads require a narrow direct-WGPU baseline:

```text
prefix scan or known compute kernel
known-image processing
known-pattern offscreen draw
```

The direct path and RunenGPU path must use equivalent:

- shaders;
- resource sizes and formats;
- dispatch/draw counts;
- adapter/backend selection;
- artifact checks;
- warm-up policy.

Measure:

- CPU preparation and validation time;
- allocations and bytes;
- command recording;
- submission overhead;
- staging/readback bytes;
- pipeline cold/warm cost;
- GPU timestamps where supported.

The comparison answers whether RunenGPU adds acceptable cost for its correctness and
reuse value. It does not justify bypassing validation.

## Device loss and reconstruction matrix

### G7 required scenarios

#### Source-backed resources

- invalidate one context generation;
- report the logical resources as reconstructable;
- recreate backend realizations from accepted source/prepared data;
- prove old-generation values are rejected.

#### Externally reconstructed imports

- invalidate the device;
- report the external owner and required reimport facts;
- reject use until the owner provides a new-generation import.

#### Non-reconstructable resources

- invalidate the device;
- report permanent loss explicitly;
- do not fabricate empty or zeroed replacement content.

#### Surface-only outcomes

Separate outdated/reconfigure, lost/recreate, and out-of-memory/product-policy paths.
RunenGPU reports facts; Runenwerk chooses retry/degrade/exit.

## Reproducibility and capture proof

### G8 bundle

Runenwerk assembles a versioned bundle containing:

- exact framework and adapter revisions;
- normalized capability/limit facts;
- permitted adapter/backend/driver facts;
- prepared-work graph inspection facts;
- workload labels and typed identities represented through stable capture keys;
- seeds, fixed-time configuration, and source generations;
- artifact manifests and checksums;
- privacy/redaction metadata;
- schema versions.

Proof requirements:

- the same accepted inputs reproduce equivalent deterministic artifacts;
- unsupported volatile fields are omitted or marked diagnostic-only;
- runtime handles and memory addresses are absent;
- capture growth is bounded;
- malformed or incompatible bundles fail validation.

## RunenRender proof sequence

### R1/R2 incremental prepared scene

Use a technical-digital-twin style proof:

- insert independent provider/material/overlay contributions;
- replace one contribution;
- remove one contribution;
- retire one producer;
- compare final output and prepared facts with an equivalent full rebuild;
- measure affected versus unaffected preparation work;
- report fallback to full rebuild explicitly when narrow changes are unavailable.

### R3 provider maturity

1. procedural/analytic field terrain;
2. overlay composition;
3. synthetic integer volume;
4. population or regional summary only after an accepted consumer;
5. fiber/liquid and hardware-specialized providers remain deferred.

Each provider proof uses only the narrow capability interfaces it needs.

### R6 cache/history invalidation

- change one scene/provider/material/view/quality generation at a time;
- prove affected cache/history entries invalidate;
- prove unrelated entries remain reusable when correctness facts permit;
- change device generation and invalidate all GPU-realized entries;
- reject stale entries rather than using them as quality degradation.

### R8 renderer operational characterization

Record:

- full versus incremental preparation cost;
- provider query counts and divergence evidence;
- cache hit/miss/invalidation;
- current-frame and history-dependent paths;
- CPU/GPU memory high-water marks;
- pipeline cold/warm cost inherited through RunenGPU;
- capture reproducibility;
- comparison with a simpler renderer/direct path for the same bounded proof.

## Offline image and video ownership

Offline sequencing remains a Runenwerk tool/application concern:

```text
fixed clock and job configuration
    -> RunenGPU compute/readback
    -> optional RunenRender image formation
    -> ordered PNG/EXR sequence and manifest
    -> optional external video encoder
```

The batch runner owns:

- seeds and fixed timing;
- bounded in-flight work/readbacks;
- filenames and manifests;
- retry/failure policy;
- PNG/EXR encoding;
- external MP4/WebM encoder integration.

RunenGPU owns completion/readback facts. RunenRender owns image-formation semantics.
Neither owns video codecs.

## Final portfolio

```text
G5 correctness
    prefix scan/readback
    headless Game of Life
    known integer texture processing

G5 operations
    submission/readback/upload saturation
    progress and callback proof
    cancellation and pending-work shutdown

G6 graphics and cost
    known-pattern offscreen draw
    compute-generated indirect draw
    direct-WGPU narrow comparisons
    offscreen boids

G7 lifecycle
    surface outcomes
    device generations
    reconstruction matrix

G8 conformance
    cache behavior
    reproducibility bundle
    operational residual audit

RunenRender
    procedural terrain
    incremental prepared scene
    synthetic volume
    cache/history invalidation
    renderer cost characterization
```

This portfolio preserves exact narrow evidence, adds operational failure-path proof,
and keeps attractive demonstrations in their proper supporting role.
