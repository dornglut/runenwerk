---
title: Runen Family Operational Hardening Investigation
description: Current-main census of RunenGPU and RunenRender progress, pressure, cache, surface, recovery, capture, incremental-scene, diagnostics, performance, and backend-reach-through gaps.
status: active
owner: workspace
layer: investigation
canonical: false
last_reviewed: 2026-07-27
related_docs:
  - ../../architecture/repository-family-architecture.md
  - ../../design/active/runengpu-architecture-design.md
  - ../../design/active/runengpu-g3-access-work-graph-design.md
  - ../../design/active/runenrender-decomposition-design.md
  - ../../design/active/runen-family-operational-hardening-design.md
  - ./runengpu-industry-comparison.md
  - ./runengpu-proof-workload-strategy.md
  - ./runengpu-runenrender-application-domain-fit.md
  - ../../workspace/planning/active-work.md
  - ../../workspace/planning/roadmap.md
---

# Runen Family Operational Hardening Investigation

## Question

Which current Runenwerk GPU/render behaviors already provide useful evidence, which
operational contracts are missing, and which existing values must remain transitional
rather than becoming RunenGPU or RunenRender public authority?

## Baseline and scope

Exact accepted baseline:

```text
repository  dornglut/runenwerk
main        5c82cc54d5ac51aeb2fd8e3da916ed895f8058e8
merge       Plan RunenGPU G3 access and work graph (#175)
```

This investigation is documentation-only. It does not modify accepted G3 semantics or
authorize Rust changes.

## Evidence inspected

### Canonical and planning authority

- repository-family architecture;
- ADR 0015;
- RunenGPU architecture and G3 design/specification;
- RunenRender decomposition design;
- GPU/render decomposition execution plan;
- roadmap, active work, and completed work;
- industry comparison, public-API review, proof strategy, S0 inventories, G2/G3
  investigations, and G1A/G2 closeouts.

### Current source

```text
engine/src/plugins/render/backend/device.rs
engine/src/plugins/render/backend/wgpu_ctx.rs
engine/src/plugins/render/backend/surface.rs
engine/src/plugins/render/backend/pipeline_cache.rs
engine/src/plugins/render/pipelines/cache.rs
engine/src/plugins/render/pipelines/keys.rs
engine/src/plugins/render/renderer/render_flow/gpu_timing.rs
engine/src/plugins/render/frame/contribution_registry.rs
engine/src/plugins/render/frame/contributions.rs
engine/src/plugins/render/residency/resource.rs
engine/src/plugins/render/inspect/budgets.rs
engine/src/plugins/render/inspect/capture.rs
engine/src/plugins/render/inspect/gpu_residency.rs
engine/src/plugins/render/inspect/graph_dump.rs
```

S0 paths were used only as discovery evidence and were rechecked against current
files.

## Finding 1 — progress is local and synchronously blocking

The GPU timing readback path currently:

```text
map_async
-> callback sends into a channel
-> device.poll(PollType::wait_indefinitely)
-> receiver.recv
-> map bytes and publish timing evidence
```

Facts:

- the current caller drives progress;
- the wait is unbounded;
- the readback is consumed synchronously;
- callback delivery is coupled to native WGPU polling behavior;
- there is no shared progress executor or explicit web/native normalization;
- there is no framework-wide pending-map/readback quota;
- cancellation and shutdown are not part of this path's contract.

This does not belong in G3. It is direct evidence for G5 progress, callback,
completion, pressure, cancellation, and shutdown requirements.

### Answer: who currently drives progress?

`read_gpu_pass_timing_evidence` drives `Device::poll` on the calling thread and then
blocks on a standard channel receive. The current source does not expose a reusable
progress-owner abstraction or thread/executor contract.

### Answer: callbacks and reentrancy

The map callback only sends a result through a channel. This path does not visibly
invoke arbitrary consumer callbacks under an internal lock. However, there is no
family-wide rule preventing other future completion callbacks from running while
registry, staging, queue, or completion locks are held. G5 must bind that rule and
test reentrancy explicitly.

## Finding 2 — device admission exposes only minimal current facts

`request_device_and_queue`:

- requests `TIMESTAMP_QUERY` only when supported;
- uses `Limits::default()`;
- selects `MemoryHints::Performance`;
- disables experimental features and tracing;
- returns `Arc<Device>`, `Arc<Queue>`, and one timing-capability structure.

Missing future authority:

- normalized portability class;
- admitted capability/limit report beyond the current timestamp fact;
- context/device generation;
- device-lost classification and callback policy;
- reconstruction report;
- cache compatibility identity;
- fallback/degradation record;
- structured admission provenance.

These are G4/G7 concerns.

## Finding 3 — current WGPU context is mixed and exposes raw backend authority

`WgpuCtx` currently owns:

```text
Instance
Adapter
BTreeMap<RenderSurfaceId, SurfaceState>
public Arc<Device>
public Arc<Queue>
RenderBackendTimingCapabilities
```

It also directly consumes `winit::Window`, configures surfaces, and uses
`block_on` for creation.

Consequences:

- RunenGPU, native-window integration, and current render-surface identity are mixed;
- `device` and `queue` are public fields, so current code has backend reach-through;
- there is no context generation or terminal state;
- no device-loss recovery/recreation contract exists;
- no separate headless context construction is represented here;
- surface ownership and thread-affinity facts are implicit.

This is transitional evidence for G4/G7 and Runenwerk adapters. It must not be copied
as the future public API.

## Finding 4 — surfaces have registry lifecycle but no backend generation/loss model

The current surface registry:

- maps `NativeWindowId` to `RenderSurfaceId`;
- records `Registered`, `MissingNativeWindow`, and `Retired` states;
- retains retired records for audit;
- clamps dimensions to at least one;
- uses `PresentMode::Fifo` and desired maximum frame latency `2`;
- configures surfaces directly through WGPU;
- maps an absent surface to `SurfaceError::Lost` in `WgpuCtx`.

Missing future authority:

- surface generation;
- acquired-image lease generation and reuse rejection;
- explicit outdated/lost/out-of-memory classification flow;
- thread-affinity and event-loop ownership;
- recreate/reconfigure facts separated from product policy;
- device-generation relationship;
- structured multi-surface capability and fallback facts.

G7 owns these facts. Runenwerk continues to own native-window and product recovery
policy.

## Finding 5 — pipeline cache is currently statistics, not a backend cache contract

Current `PipelineCacheResource` stores only:

```text
hits
misses
failures
```

`backend/pipeline_cache.rs` merely aliases that resource. `PipelineKey` is a
`Cow<'static, str>` semantic key.

Current source does not provide:

- WGPU `PipelineCache` data;
- persisted cache bytes;
- WGPU/version/backend/adapter/driver compatibility keys;
- shader/interface/capability/descriptor hashes;
- validation or migration of cache data;
- cold/warm pipeline cost evidence;
- cache rejection reasons.

The current string key may remain local authoring/integration evidence, but it cannot
be the complete future backend cache identity. G4 owns compatible cache facts; G6/G8
own characterization and conformance.

## Finding 6 — current budgets mix enforcement and retrospective readiness

### Readiness budgets

`inspect/budgets.rs` evaluates thresholds after reports exist for:

- frame/preflight/pass/GPU timing;
- dynamic target count;
- dynamic upload bytes;
- capture count/failures;
- validation and product-surface diagnostics.

These are useful readiness measurements, but they do not enforce bounds on the
underlying queue, memory, staging, readback, capture, or completion systems.

### Residency budget

`RenderGpuResidencyResource` has actual bounded policy for product cache entries:

- default maximum 64 entries;
- default 64 MiB resident bytes;
- default 8 MiB uploads per frame;
- priority and hard-pin behavior;
- generation/source-state invalidation;
- deterministic eviction of non-pinned entries;
- accepted/rejected/allocated/preserved/invalidated/evicted journal evidence.

This is valuable design evidence, but it is mixed with product identities,
field-product authority, render cache handles, and product residency/query policy.
It cannot move wholesale into RunenGPU or RunenRender.

The reusable lesson is:

- explicit budgets;
- deterministic pressure decisions;
- source-generation invalidation;
- inspectable journal;
- hard-pin exception reporting;
- no silent eviction of authoritative work.

### Answer: current bounds

Current product residency is bounded. Generic GPU submissions, uploads, mapped
readbacks, completion records, capture bytes, and pipeline data do not yet share an
accepted reusable quota contract.

## Finding 7 — captures are useful runtime evidence but not a stable format

Current captures contain:

```text
frame index
flow/pass labels
stage
string resource id
texture class
width/height
string format
optional full RGBA8 byte vector
terminal code and string reason
```

`RenderCapturedTextureState` clears and replaces captures per observed frame. This
provides frame-local inspection and pixel assertions.

Missing stable persisted/capture authority:

- schema identifier/version;
- stable typed capture key independent of runtime labels;
- source and compatibility manifest;
- device/backend facts;
- color-space and row-layout facts;
- artifact checksum and external file reference;
- retention/size quota;
- privacy/redaction policy;
- migration/validation policy;
- clear distinction between diagnostic strings and stable fields.

The full `Vec<u8>` may be large and currently has no family-level retention bound.
Runenwerk should own a versioned reproducibility/capture bundle. RunenGPU and
RunenRender provide namespaced facts only.

### Answer: current capture stability

The current capture structs are runtime inspection values. Nothing in the inspected
source establishes them as stable persisted, replay, wire, cache, or external
formats. Their string identities and in-memory bytes must not be stabilized
accidentally.

## Finding 8 — graph dumps are diagnostic strings over transitional authority

The graph dump currently serializes human-readable lines containing:

- string flow ID;
- resource debug output and lifetime;
- pass IDs/kinds;
- broad reads/writes;
- manual `depends_on`;
- execution order;
- whole-resource lifetime windows;
- compiler diagnostic count;
- source paths and provenance strings for fragment merges.

This is useful diagnosis but is not a versioned graph schema. It also reflects the
old broad resource/pass authority that G3 is designed to replace.

G3 structured prepared-work facts should become the source for future inspection.
Runenwerk may render those facts as text or bundle them, but current line ordering and
strings are not stable persistence authority.

## Finding 9 — prepared contributions are frame snapshots, not incremental renderer authority

Current contribution collection:

- is coupled to `ecs::World` and typed resource lookup;
- registers collectors through string collector and payload-kind IDs;
- uses `TypeId` and type names for current resource requirements;
- stores each frame in `PreparedFrameContributions.by_feature`;
- inserts or overwrites one contribution per `RenderFeatureId`;
- carries feature-specific payload enums and product/domain values;
- exposes runtime signatures and inspection strings.

Useful current properties:

- deterministic `BTreeMap` storage;
- explicit diagnostics and fallback policies;
- frame preparation separated from execution;
- typed source-resource lookup inside Runenwerk;
- explicit feature readiness/missing/disabled status.

Missing future RunenRender authority:

- renderer-local producer identity and generation;
- independent contribution identity;
- deterministic insert/replace/remove/retire-producer lifecycle across frames;
- changed-region and affected-generation tracking;
- narrow provider/material/view/overlay invalidation;
- proof that unrelated contributions avoid full rebuild;
- stable separation from ECS `TypeId` and feature-specific payload enums.

### Answer: current incremental cost

Current source provides frame-local insertion and overwrite in a feature-keyed map,
but no accepted renderer-owned incremental prepared-scene contract or cost evidence.
R1/R2 must compare incremental updates with equivalent full rebuilds.

## Finding 10 — source-backed invalidation exists locally, but device recovery does not

The product residency model invalidates entries when product generation or source
state changes. It distinguishes selected/requested/resident/preserved/invalidated/
evicted/rejected and records diagnostics.

This is strong evidence for source-generation-bound derived caches. It is not device
recovery:

- cache handles are local monotonically allocated values;
- no context/device generation is attached;
- imported versus source-backed versus non-reconstructable GPU realizations are not
  represented as a generic contract;
- no device-loss event invalidates all backend realizations;
- no external owner reimport handshake exists.

G7 must own the generic reconstruction matrix. Runenwerk owns product action.

## Finding 11 — current benchmarks do not prove abstraction overhead

The retained proof strategy contains strong future correctness workloads, but the
current evidence inspected does not establish equivalent direct-WGPU baselines for:

- graph preparation and validation cost;
- known compute work;
- known image processing;
- known-pattern graphics;
- cold/warm pipeline realization;
- staging/readback high-water marks;
- full versus incremental prepared-scene cost.

G6 and R8 must provide controlled comparisons. Performance remains diagnostic until
a separately accepted budget binds environment and thresholds.

## Finding 12 — ownership remains consistent with S0, but paths are transitional

Current-source evidence confirms the accepted disposition:

```text
RunenGPU future owner
    device/context admission
    pipeline realization/cache facts
    generic progress/submission/readback
    low-level surface outcomes

RunenRender future owner
    prepared scene/contributions
    provider and image-formation semantics
    render-derived caches/history

Runenwerk retained owner
    ECS collection
    native windows
    shader files/hot reload
    product residency/quality policy
    capture persistence and artifact encoding
    diagnostics presentation
    recovery decisions
```

No evidence requires changing ADR 0015 or accepted G3 semantics.

## Answer matrix

| Issue question | Current answer | Required owner/phase |
|---|---|---|
| Who drives `Device::poll`? | GPU timing readback caller, synchronously, on its calling thread. | RunenGPU G5 progress model. |
| Can callbacks run under locks/reenter? | Current timing callback only sends to a channel; no family-wide prohibition or proof exists. | G5 callback/reentrancy contract and tests. |
| In-flight bounds? | Product residency has explicit bounds; generic submissions/uploads/readbacks/completions/captures do not have one accepted shared contract. | G5/G8; capture retention in Runenwerk. |
| Cache keys/persistence? | Pipeline cache is stats plus string key; no backend cache compatibility/persistence authority. | G4/G6/G8. |
| Device/surface loss? | Surface registry has registered/retired; acquisition returns raw WGPU errors; no context generation/reconstruction model. | G7 plus Runenwerk recovery. |
| Reconstruction facts? | Product cache uses source generation/state; no generic source-backed/imported/non-reconstructable device-loss matrix. | G7. |
| Capture identity/privacy/stability? | Runtime string identities and optional full RGBA8 bytes; no schema, retention, redaction, or stable format. | Runenwerk bundle at G8/R8. |
| Incremental prepared scene? | Frame-local feature map insertion/overwrite; no producer lifecycle or incremental proof. | RunenRender R1/R2. |
| Benchmark coverage? | Correctness/showcase candidates exist; no accepted direct-WGPU cost baseline. | G6/R8. |
| Raw WGPU reach-through? | `WgpuCtx` publicly exposes `Arc<Device>` and `Arc<Queue>` and directly owns Winit surfaces. | G4/G8 containment and source guards. |

## Decisions

1. Accepted G3 work/access/graph semantics remain unchanged.
2. Add one canonical operational-hardening design rather than a new phase or shared
   package.
3. Map progress/pressure to G5, portability/cache to G4, performance to G6,
   generations/recovery to G7, and conformance/capture audit to G8.
4. Map incremental prepared scenes to R1/R2, narrow provider maturity to R3,
   cache/history invalidation to R6, and renderer performance/capture proof to R8.
5. Preserve product residency as Runenwerk/source-domain evidence; do not copy its
   product IDs or policies into RunenGPU.
6. Preserve capture/artifact persistence in Runenwerk.
7. Treat current string keys, graph lines, runtime signatures, TypeIds, raw WGPU
   fields, and cache handles as transitional/non-stable unless a later accepted
   specification says otherwise.

## Evidence gaps retained

This documentation slice does not execute device-loss injection, WebGPU runtime
proofs, saturation tests, pipeline cache reuse, or direct-WGPU benchmarks. Those are
implementation-phase evidence requirements, not facts that documentation may invent.

## Next action

Accept this documentation-only hardening slice, then reverify issue `#177` against the
resulting exact `main` before implementing G3. Do not begin G4-G8 or RunenRender Rust
work from this investigation.
