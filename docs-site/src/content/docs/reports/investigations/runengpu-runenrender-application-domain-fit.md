---
title: RunenGPU and RunenRender Application-Domain Fit
description: Ranked evaluation of application domains that materially exercise RunenGPU and RunenRender without moving domain ownership into the frameworks.
status: active
owner: gpu
layer: investigation
canonical: false
last_reviewed: 2026-07-27
related_docs:
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ./runengpu-industry-comparison.md
  - ./runengpu-proof-workload-strategy.md
  - ../../workspace/planning/roadmap.md
---

# RunenGPU and RunenRender Application-Domain Fit

## Question

Where does the RunenGPU/RunenRender split provide unusually strong architectural
value, assuming both frameworks are implemented correctly, without turning them into
complete domain products?

## Evaluation method

Domains are ranked by:

1. fit with generic compute plus image formation;
2. benefit from field/procedural/non-mesh representations;
3. need for headless/offscreen and reproducible output;
4. value of sharing one GPU context across simulation and rendering;
5. pressure for explicit capabilities, readback, recovery, and diagnostics;
6. amount of unrelated domain infrastructure required before a useful proof exists;
7. availability of a bounded proof that can improve the frameworks themselves.

The ranking is strategic, not a promise to build each vertical product.

## Summary ranking

| Rank | Domain | Strategic fit | First useful proof | Main reason |
|---|---|---|---|---|
| 1 | Implicit CAD and computational fabrication | Very high | field-based part preview plus section/slice artifact | Native fit for analytic/field geometry, exact queries, compute, and offline output. |
| 2 | Scientific and medical volume visualization | Very high | deterministic volume ray integration with transfer-function variants | Strong volume/provider, readback, multi-view, and quality-policy pressure. |
| 3 | Robotics synthetic data and virtual sensors | High | fixed-seed depth/normal/segmentation sensor batch | Reproducible multi-target rendering and compute/readback share one context. |
| 4 | Geospatial and environmental simulation | High | clipmapped terrain/field with simulation overlay and regional summary | Exercises streaming, changed regions, large scale, and multi-resolution providers. |
| 5 | Technical digital twins | High | incremental machinery scene with live scalar/vector overlays | Strong prepared-scene update, diagnostics, and heterogeneous-provider pressure. |
| 6 | VFX and offline procedural generation | High | deterministic procedural sequence with manifests and EXR-ready outputs | Headless sequencing, caches, simulation/render composition, and capture matter. |
| 7 | Computational photography | Medium-high | known-image filter/reconstruction pipeline | Excellent texture compute and readback proof, weaker provider/scene pressure. |
| 8 | Browser-native visualization tools | Medium-high | WebGPU-compatible volume/field viewer | Validates portability and web progress differences, but constrains feature scope. |
| 9 | Rendering research | Medium | comparable estimator/provider experiments | Architecture is inspectable and replaceable, but research code can destabilize APIs. |
| 10 | Unconventional field/procedural games | Medium | procedural SDF terrain plus independent compute simulation | Strong long-term product fit but broad gameplay/editor scope weakens early isolation. |

## 1. Implicit CAD and computational fabrication

### Why it fits

Implicit and analytic models can remain authoritative without mandatory mesh
extraction. RunenGPU supplies generic field evaluation, scans, compaction, texture or
buffer output, readback, and offscreen execution. RunenRender supplies views,
materials, section visualization, overlays, provider queries, and image formation.

### Relevant contracts

- RunenGPU checked storage access, compute work, query/readback, offscreen graphics,
  capability admission, and reproducibility facts;
- RunenRender `Analytic`, `Procedural`, and field-backed `Solid` provider capabilities;
- incremental contribution replacement for parameter edits;
- current-frame sharp rendering and section/cut visualization;
- deterministic artifact generation.

### Missing domain-owned systems

CAD/fabrication must own:

- constraints and parametric feature history;
- topology/feature identity;
- units and tolerances;
- manufacturing rules;
- boolean robustness policy;
- slicing/toolpath generation;
- document formats and migration;
- certification and machine integration.

None belongs in RunenGPU or RunenRender.

### Candidate proof

A parameterized implicit mechanical part with:

- interactive material and section view;
- exact CPU reference probes at selected points;
- GPU field preview;
- deterministic slice image and occupancy/readback artifact;
- incremental parameter replacement without full-scene reconstruction.

### Strategic value

This is the strongest non-game differentiator because the representation and query
model are central rather than incidental.

## 2. Scientific and medical volume visualization

### Why it fits

Volume rendering needs interval traversal, transmittance, transfer functions,
multi-resolution data, headless output, and explicit quality degradation. It benefits
from a renderer that does not assume every visible object is a raster mesh.

### Relevant contracts

- RunenRender `Volume`, interval, transmittance, and regional-summary capabilities;
- RunenGPU storage textures/buffers, compute, offscreen targets, readback, limits,
  pressure outcomes, and device-loss facts;
- changed-region and source-generation invalidation;
- multi-view and slice targets;
- structured unsupported/degraded reporting.

### Missing domain-owned systems

The application owns:

- DICOM/NIfTI or scientific formats;
- clinical metadata and privacy;
- segmentation and measurement semantics;
- regulated validation;
- dataset streaming and provenance;
- calibrated transfer-function presets;
- annotation and collaboration.

### Candidate proof

A fixed synthetic integer volume with:

- CPU oracle for selected rays/slices;
- deterministic transfer-function variants;
- volume image, depth, and selected voxel readback;
- changed-brick update proof;
- bounded-memory pressure behavior.

### Strategic value

Very high technically, but production medical use requires domain governance far
beyond the frameworks.

## 3. Robotics synthetic data and virtual sensors

### Why it fits

Synthetic sensors need one prepared scene to produce color, depth, normals,
segmentation, optical flow, and other buffers under a fixed clock. Simulation and
render work must share resources without forcing the renderer to own robotics state.

### Relevant contracts

- generic RunenGPU compute/render composition;
- multiple logical targets and asynchronous readback;
- RunenRender prepared views and provider-independent interaction output;
- deterministic seeds, fixed-time manifests, and artifact provenance;
- reconstruction/device-generation facts for long batch jobs.

### Missing domain-owned systems

Robotics must own:

- robot dynamics and control;
- sensor calibration/noise models;
- coordinate-frame conventions;
- scenario generation;
- dataset schemas and labeling policy;
- physics and contact;
- evaluation metrics.

### Candidate proof

A fixed scene and trajectory producing:

```text
RGB
linear depth
surface normal
provider/instance segmentation
motion/velocity where supported
```

The proof validates artifact dimensions, finite ranges, selected rays, identities,
and reproducibility rather than demanding exact floating-point color across backends.

### Strategic value

High because it proves multi-output image formation and headless batch reliability.

## 4. Geospatial and environmental simulation

### Why it fits

Large terrain, atmosphere, water, vegetation, and scalar/vector fields need
multi-resolution representations and changed-region updates. Regional summaries and
procedural providers are more natural than one fully resident mesh scene.

### Relevant contracts

- `Procedural`, `Population`, `Volume`, and `RegionalSummary` candidates;
- incremental prepared-scene contributions;
- clipmap/page-like derived caches;
- compute simulation plus image formation;
- pressure outcomes and offscreen map products.

### Missing domain-owned systems

The application owns:

- coordinate reference systems and geodesy;
- GIS formats and tiling services;
- weather/hydrology/ecology models;
- authoritative world streaming;
- temporal datasets and uncertainty;
- map symbology and analysis tools.

### Candidate proof

A deterministic procedural height/field region with:

- near detailed provider and far regional summary;
- moving view and changed-region update;
- scalar simulation overlay;
- color image plus height/field readback;
- cache invalidation evidence.

### Strategic value

High, provided world streaming remains outside RunenRender.

## 5. Technical digital twins

### Why it fits

Digital twins combine conventional solids, analytic parts, sensor fields, flow
visualization, overlays, and frequently changing state. The prepared-scene boundary
can isolate authoritative plant state from rendering.

### Relevant contracts

- heterogeneous provider contributions;
- deterministic insert/replace/remove;
- material, overlay, and diagnostic provenance;
- incremental changed-region updates;
- capture bundles for incident reproduction;
- readback for inspection and reports.

### Missing domain-owned systems

The product owns:

- asset/plant schemas;
- telemetry ingestion;
- units and engineering semantics;
- alarms and historian integration;
- authorization and audit;
- simulation/calibration models;
- collaboration and workflow.

### Candidate proof

An incremental machine assembly where sensor updates replace only affected
contributions and overlays, while a reproducibility bundle recreates one inspected
frame.

### Strategic value

High integration value, but not a reason to put business/telemetry semantics in the
renderer.

## 6. VFX and offline procedural generation

### Why it fits

Offline procedural generation needs deterministic sequencing, simulations,
headless image formation, large readbacks, cache discipline, and external encoding.
It benefits from sharing compute and render work without requiring a window.

### Relevant contracts

- RunenGPU headless execution, completion, pressure, readback, and shutdown;
- RunenRender quality tiers, history, volumes, procedural providers, and color intent;
- Runenwerk-owned frame manifests and external encoders;
- reproducibility bundles and derived-cache versioning.

### Missing domain-owned systems

The application owns:

- asset graph and scene format;
- timeline and dependency scheduling;
- render-farm orchestration;
- color-management implementation;
- EXR/video encoding;
- artist workflow and review;
- license and storage policy.

### Candidate proof

A fixed-seed procedural sequence that produces ordered image frames, per-frame facts,
checksums, and a run manifest with bounded in-flight readbacks.

### Strategic value

High operational proof value even if Dornglut never becomes a VFX suite.

## 7. Computational photography

### Why it fits

Image filters, reconstruction, denoising, super-resolution, panorama, and feature
pipelines exercise texture uploads, compute graphs, intermediates, offscreen targets,
and readback with limited scene semantics.

### Relevant contracts

- RunenGPU texture/buffer work, copies, pressure, readback, and direct-WGPU comparison;
- limited RunenRender reconstruction/color policy when image-formation semantics are
  actually needed;
- deterministic integer or bounded-float test images.

### Missing domain-owned systems

The application owns camera models, RAW formats, metadata, calibration, ML models,
quality metrics, and editing workflow.

### Candidate proof

Known-image convolution/Sobel/reconstruction with CPU oracle, padded readback
normalization, and cold/warm pipeline characterization.

### Strategic value

Excellent RunenGPU proof; moderate justification for the full RunenRender provider
architecture.

## 8. Browser-native visualization tools

### Why it fits

WGPU/WebGPU makes the same logical contracts usable in native and browser
applications, but browser progress, feature, memory, and surface behavior differ.

### Relevant contracts

- normalized portability classes;
- automatic/browser-driven completion versus native polling;
- storage/format limits and structured degradation;
- prepared scenes and offscreen/surface target equivalence;
- no native-handle assumption.

### Missing domain-owned systems

The product owns web UI, workers, asset delivery, persistence, browser security,
accessibility, and deployment.

### Candidate proof

A small field or volume viewer with the same prepared scene on native WGPU and
WebGPU, reporting capability differences without separate semantic paths.

### Strategic value

Medium-high as a portability proof. Web constraints must not silently define the
entire native feature ceiling.

## 9. Rendering research

### Why it fits

Prepared scenes, provider capabilities, one transport family, inspectable work, and
structured diagnostics can support controlled experiments.

### Relevant contracts

- replaceable estimator/transport components;
- deterministic scene and capture facts;
- provider query instrumentation;
- direct narrow baseline comparisons;
- current-frame versus history policy.

### Missing domain-owned systems

Research tooling owns experiment definitions, datasets, statistical analysis,
publication artifacts, and prototype code lifecycle.

### Candidate proof

Two estimator or provider-query strategies evaluated on the same prepared scene with
identical artifacts and recorded work/cost facts.

### Strategic value

Useful for validating architecture, but research prototypes must not freeze unstable
public APIs.

## 10. Unconventional field/procedural games

### Why it fits

Dornglut's original goals—procedural worlds, field/SDF representations, simulations,
large view distances, many effects, and optional offline output—fit the architecture.

### Relevant contracts

- independent simulation and rendering work on one RunenGPU context;
- field/procedural providers without mandatory source meshes;
- current-frame sharp image formation;
- prepared-scene adapters from Runenwerk;
- scalable quality and history policies;
- surfaces plus headless/offscreen output.

### Missing domain-owned systems

Runenwerk owns gameplay, ECS, physics, world streaming, networking, authoring, editor,
assets, windows, input, audio, and product policy.

### Candidate proof

Procedural sky/SDF terrain plus one independent compute simulation and an offscreen
sequence, with no ECS or product types in either framework API.

### Strategic value

Strong long-term product value but a poor early conformance oracle because failures
span too many systems.

## Domain ownership guardrails

RunenGPU must not absorb:

- CAD constraints;
- medical formats or clinical semantics;
- robotics sensor/product policy;
- GIS coordinate systems;
- digital-twin telemetry schemas;
- VFX timeline/farm systems;
- photography algorithms as framework primitives;
- browser application architecture;
- research experiment management;
- gameplay/world policy.

RunenRender must not absorb those systems either. Reusable adapters may be proposed
only after two independent consumers prove stable ownership.

## Recommended proof progression

```text
near-term RunenGPU proof
    computational-photography style known-image processing
    robotics-style deterministic multi-target readback

first RunenRender proof
    procedural sky/SDF terrain

incremental scene proof
    technical digital-twin style insert/replace/remove

provider expansion evidence
    synthetic volume visualization

later stress/research evidence
    geospatial regional summaries
    VFX sequence and cache pressure
```

The progression reuses domain pressure without committing the frameworks to complete
vertical products.

## Decision

The architecture is most defensible where one product needs both reusable GPU
execution and representation-neutral image formation, especially when implicit,
analytic, volumetric, procedural, or multi-output content is central.

The strongest early external-looking proofs are computational fabrication, synthetic
volume visualization, robotics sensors, and deterministic offline procedural output.
Conventional raster games remain possible, but by themselves do not justify more than
WGPU plus an existing renderer.
