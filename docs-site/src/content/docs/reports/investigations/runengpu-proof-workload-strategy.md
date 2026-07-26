---
title: RunenGPU Proof Workload Strategy
description: Critical workload selection for deterministic conformance, boundary integration, visual showcases, offline output, and later RunenRender proofs.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-26
related_docs:
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-internal-decomposition-execution-plan.md
  - ../../workspace/planning/roadmap.md
  - ../../workspace/planning/active-work.md
  - ./runengpu-render-s0-file-disposition.md
  - ./runengpu-public-api-ergonomics-review.md
---

# RunenGPU Proof Workload Strategy

## Decision summary

RunenGPU must not rely on one visually impressive example as proof of correctness.

The proof portfolio has four distinct roles:

```text
deterministic conformance
    exact results, narrow failure diagnosis, required gate

boundary integration
    multiple resources and work stages, required gate

visual showcase
    representative application value, tolerant evidence

benchmark or stress workload
    scale and performance evidence, never correctness authority
```

The earlier proposal to use only cellular automata for G5 and boids for G6 was directionally useful but incomplete. Cellular automata and boids remain valuable, but neither should carry all phase acceptance responsibility.

## Selection criteria

A proof workload is evaluated against:

1. **Determinism** — whether expected output can be checked exactly or only within tolerance.
2. **Boundary coverage** — which capabilities, resources, accesses, transfers, pipelines, and lifecycle paths it exercises.
3. **Isolation** — whether a failure identifies one framework contract or a large mixed application.
4. **Portability** — whether the proof is reasonable across WGPU backends and common GPU classes.
5. **Current reuse** — whether Runenwerk already contains accepted code, shaders, tests, or consumers for the workload.
6. **Human value** — whether the result is understandable without specialized tooling.
7. **Complexity cost** — how much unrelated domain or rendering policy must be introduced.

No single workload scores best across all criteria. The portfolio therefore uses a ladder rather than a winner-takes-all example.

## Current repository candidates

### Prefix scan, counters, compaction, and indirect arguments

Current generic GPU primitives already define:

- counter reset;
- inclusive and exclusive `u32` prefix scan;
- scatter/compaction building blocks;
- generated indirect draw arguments;
- explicit resource access declarations;
- temporary scan storage;
- multi-stage dispatch plans.

The S0 disposition assigns these primitives to RunenGPU.

**Best use:** deterministic G5 conformance and G6 GPU-driven drawing.

**Strengths:**

- integer results are exactly checkable;
- exercises uploads, multiple dispatches, temporary resources, dependencies, completion, and readback;
- directly proves authority already intended to move to RunenGPU;
- reusable by boids, procedural generation, culling, visibility, and GPU-driven rendering.

**Weakness:** not visually compelling by itself.

### Game of Life

The current `game_of_life_sdf` example already has:

- a fixed-size integer grid;
- deterministic seed and tick state;
- ping-pong storage;
- compute dispatch;
- fullscreen composition;
- windowed Runenwerk integration.

**Best use:** G5 stateful-compute integration and the first deterministic offline sequence.

**Strengths:**

- exact state checksums and selected-cell assertions are possible;
- proves repeated submission and ping-pong ownership;
- easy to explain and visually inspect;
- can produce images with either host-side conversion or compute-written texture output.

**Weakness:** by itself it does not sufficiently exercise generic multi-stage GPU primitives or graphics composition.

### Boids

The current `boids_render_flow` example already performs:

- fixed-step simulation;
- double-buffered agents;
- bounded uniform-grid construction;
- atomic counting;
- prefix scan;
- scatter into sorted indices;
- neighborhood simulation;
- compute-to-graphics resource sharing;
- procedural instanced drawing;
- presentation.

**Best use:** G6 mixed compute-and-graphics showcase and later G7 interactive proof.

**Strengths:**

- reuses a substantial real workload;
- covers many resource and scheduling paths;
- demonstrates why RunenGPU must support non-render compute and rendering on one context;
- produces an understandable visual result.

**Weaknesses:**

- atomic ordering and floating-point accumulation make exact cross-backend state or pixel equality inappropriate;
- broad scope makes failures harder to diagnose;
- the current example also contains ECS projection, shader-file policy, surface behavior, and procedural render semantics that must remain outside RunenGPU.

Boids is therefore a required integration target only after narrow conformance proofs exist. It is not the primary correctness oracle.

### Minimal fullscreen and known-color rendering

The current `render_flow_fullscreen_minimal` example proves graph authoring only. It does not yet provide a sufficient hardware execution artifact.

**Best use after redesign:** minimal G6 offscreen graphics conformance.

The improved proof should render a known integer-friendly pattern or small triangle into an offscreen texture, read it back, and validate selected pixels with documented tolerance.

**Strengths:**

- isolates graphics pipeline, attachment, clear/load/store, draw, copy, and readback behavior;
- failure diagnosis remains narrow;
- establishes a graphics baseline before boids.

### Post-process compositor

The current `render_flow_postprocess_compositor` combines compute and fullscreen work, but currently demonstrates planning rather than a complete executable artifact.

**Best use after redesign:** G6 texture-processing integration.

An executable variant should upload a known source image, apply a small convolution or separable blur, and read back the result.

This proves texture upload, sampled/storage usage, intermediate textures, compute or fullscreen processing, copies, and image readback without requiring a scene renderer.

### Procedural sky and SDF terrain

The current `procedural_sky_sdf_terrain` example is a single semantic fullscreen image-formation workload.

**Best use:** first internal RunenRender semantic proof after standalone RunenGPU is accepted.

It is simpler than boids as a renderer proof because it needs one prepared view/request and one image-formation pass, but it should not define RunenGPU semantics.

### SDF flow with history

The current `sdf_render_flow` includes compute preparation, fullscreen composition, history copy, and presentation.

**Best use:** later RunenRender temporal/history-resource proof.

It should not be the first graphics proof because history and temporal ownership add complexity unrelated to basic RunenGPU execution.

## External sample patterns

Mature GPU ecosystems commonly separate small exact samples from larger demonstrations:

- WGPU provides a standalone minimal compute example and separate feature examples such as boids.
- Bevy lists compute Game of Life and GPU readback as separate examples.
- Vulkan Samples pairs minimal API samples with a two-pass compute N-body simulation and supports headless screenshot workflows.
- NVIDIA CUDA samples cover exact data-parallel algorithms such as reduction, histogram, and prefix scan alongside N-body, particles, image filters, Mandelbrot, fluids, and graphics interop.

This supports a portfolio design rather than choosing one universal example.

## Alternative workload assessment

| Candidate | Best role | Decision |
|---|---|---|
| Vector addition | minimal compute tutorial | Do not use as a primary gate; too weak and unrelated to current authority. |
| Reduction or histogram | deterministic compute/readback | Good optional G5 conformance extension after prefix scan. |
| Prefix scan and compaction | deterministic multi-stage compute | Required G5 gate; best fit with current RunenGPU ownership. |
| Integer cellular automaton | stateful repeated compute | Required G5 integration proof. |
| Integer procedural texture | compute-to-texture export | Good G5 texture-readback proof when storage-texture support is in scope. |
| Image convolution or Sobel filter | texture upload/process/readback | Strong G5/G6 transfer and texture integration proof. |
| Mandelbrot or Julia set | compute-to-image showcase | Optional; floating-point boundary pixels weaken exact portability. |
| Reaction-diffusion | stateful texture showcase | Useful later offline showcase; not a conformance oracle. |
| N-body | compute-and-render showcase | Valid alternative to boids, but duplicative because boids already exists and exercises more current primitives. |
| Boids | broad compute-and-render integration | Required G6 showcase, checked structurally and with tolerant visual evidence. |
| Fluid or smoke simulation | multi-pass stress workload | Defer until after extraction; excessive early complexity and float variability. |
| FFT ocean | compute/graphics stress and spectral processing | Defer; introduces FFT and domain-specific image formation. |
| GPU mesh generation or marching cubes | procgen-to-render integration | Later RunenGPU/RunenRender/source-domain proof, not initial extraction. |
| SDF terrain/raymarching | semantic renderer proof | Use for RunenRender, not RunenGPU conformance. |
| Path tracing | advanced renderer and accumulation | Defer well beyond initial RunenRender proof. |

## Accepted proof ladder

### G5 — headless execution and transfers

Required narrow conformance:

1. **`u32` prefix scan and readback**
   - fixed integer input;
   - inclusive and exclusive expected outputs;
   - multi-workgroup input large enough to require temporary scan storage;
   - exact output and total-count verification;
   - no window, renderer, ECS, or product types.

Required stateful integration:

2. **headless Game of Life**
   - fixed seed, dimensions, and number of ticks;
   - ping-pong buffers;
   - exact checksum plus selected-cell assertions;
   - asynchronous completion and readback;
   - source state prepared outside RunenGPU.

Conditional texture proof when G5 includes storage-texture readback:

3. **integer compute-to-texture artifact**
   - deterministic pattern or cellular-automaton visualization;
   - texture-to-buffer readback;
   - row-padding normalization;
   - PNG export performed by a Runenwerk test/tool adapter, not RunenGPU.

Optional extensions:

- reduction;
- histogram;
- image convolution.

### G6 — offscreen graphics and shared consumers

Required narrow graphics conformance:

1. **offscreen known-pattern draw**
   - no surface;
   - one graphics pipeline;
   - known clear and draw result;
   - texture readback and selected-pixel checks.

Required GPU-driven composition proof:

2. **compute-generated indirect draw**
   - counter/scan/compaction or generated draw arguments;
   - compute writes data consumed by graphics;
   - dependency inferred from resource access;
   - structural and selected-pixel assertions.

Required representative integration:

3. **offscreen boids**
   - existing spatial-grid simulation and draw path migrated through accepted adapters;
   - one shared RunenGPU context;
   - no surface requirement;
   - artifact generation succeeds;
   - validation uses invariants, counts, finite ranges, pass/resource evidence, and tolerant image checks rather than exact cross-backend pixels.

Optional texture-pipeline integration:

- executable image convolution or post-process compositor.

### G7 — surfaces and device outcomes

G7 should reuse accepted G6 workloads instead of introducing another showcase:

- present the known-pattern graphics proof;
- present boids interactively;
- test resize, surface reconfiguration, multiple surfaces where supported, and device/surface outcome reporting.

The same semantic work should be capable of targeting offscreen and surface-backed outputs without duplicate execution authority.

### G8 and GX — conformance and extraction

The standalone conformance suite must retain the narrow G5/G6 proofs. Showcase examples supplement but do not replace them.

Hardware-dependent visual evidence is separate from deterministic contract tests. Unsupported or unavailable hardware paths report structured skips or environment evidence rather than silently passing.

### RunenRender proof sequence

After standalone RunenGPU and clean Runenwerk cutover:

1. procedural sky/SDF terrain as the first simple semantic image-formation proof;
2. boids render adapter as shared simulation-to-render integration;
3. SDF history flow as a later temporal/history proof;
4. more advanced scene, material, lighting, reconstruction, or transport examples only as their owning phases require them.

## Offline image and video output

Offline sequencing is a Runenwerk tool/application concern built on RunenGPU readback and optional RunenRender image formation.

Recommended order:

1. Game of Life PNG sequence after G5 if host-side visualization or storage-texture export is available.
2. Boids offscreen PNG sequence after G6.
3. RunenRender-owned SDF or scene sequences after the relevant renderer proof.

The batch runner owns:

- fixed frame/tick timing;
- seed and input configuration;
- bounded in-flight readbacks;
- ordered frame filenames;
- run and per-frame manifests;
- retry/failure policy;
- PNG or later EXR encoding.

RunenGPU owns completion and readback facts. RunenRender owns image-formation semantics. Neither owns MP4/WebM codecs. Video encoding remains an external encoder integration over an accepted image sequence and manifest.

## Final decision

The preferred portfolio is:

```text
G5 conformance
    prefix scan/readback

G5 stateful integration
    headless Game of Life

G6 graphics conformance
    offscreen known-pattern draw

G6 GPU-driven composition
    compute-generated indirect draw

G6 showcase
    offscreen boids

G7 surface proof
    reuse known-pattern draw and boids

RunenRender first proof
    procedural sky/SDF terrain

Offline output
    Game of Life sequence first, boids sequence second
```

This portfolio is stronger than using Game of Life and boids alone. It preserves understandable visual results while keeping exact, narrow, portable conformance evidence available for diagnosis and extraction readiness.
