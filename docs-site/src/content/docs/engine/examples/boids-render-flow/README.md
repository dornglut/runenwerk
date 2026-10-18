---
title: Boids Render Flow Example
description: Production procedural boids render-flow example with fixed-step evidence, procedural pass authoring, and aspect-correct impostors.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-10-18
publication: primary
---

# Boids Render Flow Example

Windowed boids simulation sample built on the public `RenderFlow` and
procedural population authoring APIs.

What it demonstrates:

- ECS simulation state (`BoidsRenderState`) projected into flow uniforms.
- ping-pong storage simulation (`boids.instances`) on a compute pass;
  both retained buffers and all count/scan/scatter scratch arrays opt into
  descriptor-guaranteed zero initialization for strict GPU read/write admission.
- procedural-owned draw authoring through `RenderFlow::procedural_pass_builder(...)`.
- bounded uniform-grid neighbor lookup instead of a production O(n^2) all-boids loop.
- reusable procedural camera projection through `ProceduralCamera2d`,
  `ProceduralCamera2dUniform`, and `ProceduralSpriteSizing`.
- smoothed visual heading stored separately from simulation velocity.
- graph-level fixed-step catch-up evidence; multi-step catch-up is owned by the
  render-flow graph, not hidden in this example.
- flow-owned color target and explicit terminal present pass.
- pure builtin compiled runtime path (no custom executor hooks).
- production evidence for no silent grid overflow, bounded submitted work,
  unsupported timing diagnostics, and pixel-space resize/aspect checks.

Flow chain:

- `boids.seed_or_hold` (compute, `assets/shaders/boids_compute.wgsl`)
- `boids.grid.clear_counts`
- `boids.grid.count_cells`
- `boids.grid.scan_counts`
- `boids.grid.reset_cursors`
- `boids.grid.scatter_sorted_indices`
- `boids.grid.simulate_neighbors` (compute over adjacent grid cells)
- `boids.grid.publish_draw` (compute publish to the render-facing instance buffer)
- `boids.draw` (graphics, `assets/shaders/boids_compose.wgsl`)
- `boids.present` (present, `boids.color` -> surface)

The grid path is bounded and total-count-sized. Cell counts, scan offsets,
scatter cursors, and sorted indices are explicit flow resources; dense cells
must not silently overflow or drop neighbors. Spatial hash and chunked
unbounded populations are intentionally out of scope for this example.

The simulation region is declared as `boids.fixed_step`. `BoidsGpuStepSchema`
captures immutable source-time uniforms and dispatch sizes for all eight compute
passes using exact IDs from the authored `RenderFlow`. The ECS fixed-time
counter describes **requested** source steps, never proof that GPU state has
advanced. The Engine admits an ordered, bounded prefix to RunenGPU G5 and
retains accepted source steps until its exact completed submission and producer-owned
source interval verification. GPU-resident normal simulation does not require a
GPU-to-CPU color readback. The native smoke explicitly opts into the full-frame
texture witness to test rendered pixels and motion.

On the first accepted source step, the shader seeds the retained boid population;
later source steps advance their own captured ticks, not a reconstructed host
frame delta. Both retained boid buffers and grid scratch resources use explicit
GPU descriptor `Zeroed` initialization for strict read-before-write admission.
Zero bytes are only safe descriptor coverage; they are **not** populated boids.

In ordinary execution the example accepts GPU progress after the exact G5
submission has terminated successfully, with its original device affinity
and immutable ordered source-tick interval verified. It does **not** claim
a CPU-side numerical oracle or full retained Boids-buffer verification.
A failed/unknown GPU write remains quarantined and requires fail-stop and
clean restart, never speculative replay.

The fixed-time counter remains the target for requested source ticks. The
producer captures at most one render-region batch (four steps) per Update
and never exceeds 512 outstanding inputs, including accepted/unverified work.
If GPU execution stalls, CPU fixed time may advance, but source capture is
**paused at capacity**: the uncaptured tick range remains an explicit
difference between the CPU target and the producer's requested cursor.
Once capacity returns the same tick identities are captured in order.
The example's simulation law and tick-dependent inputs are fixed at startup,
making this deferred generation equivalent to source-time generation.
A consumer with changing inputs must retain actual historical snapshots
instead. This is bounded, lossless tick-debt pacing, **not** a promise to
synchronize with wall time under sustained GPU overload.

Only the opt-in native conformance mode requires a full GPU draw readback.
It checks the exact G5 source receipt, byte format/length and nonbackground
image, then fingerprints deterministic pixel samples. Moving fingerprints
are visual evidence, not an agent/grid numerical oracle or checkpoint.
Normal Boids rendering does not request these mandatory pixel transfers.
Full buffer checkpointing and verified contents restoration remain unqualified.

Following a successfully verified batch, the source records its exact GPU
context/device generation. A subsequent generation mismatch **fails closed
before newer steps are admitted**. The bounded recovery policy is to stop and
clean-restart the application, beginning a new simulation from tick zero; it
does not replay prior accepted writes or pretend zero-initialized descriptors
restore the previous simulation. Device-failure injection and live
generation-reconstruction still require their own native evidence under
[#1291](https://github.com/dornglut/runenwerk/issues/1291).

The native CI also runs a **controlled source-owner failure fixture** after
twelve verified GPU ticks in each mode. It expects the windowed application to
return an error rather than accepting further source work, then launches a
fresh process for the ordinary 120-frame success qualification. This proves
fail-stop propagation and clean-start behavior across two distinct processes;
it does **not** simulate a real device loss, partially executed shader write,
or in-place recreation of a GPU context.

Native hosted Mesa/Vulkan smoke is opt-in with `RUNENWERK_BOIDS_NATIVE_SMOKE=1`
for both executable modes. Its default capture-enabled mode checks at least
120 **submitted Present opportunities**, correlated terminal G5 receipts,
semantically verified GPU steps, and changing GPU-rendered boid images, then
closes its window. Set `RUNENWERK_BOIDS_NATIVE_NO_READBACK=1` *alongside*
the smoke flag to test the actual GPU-resident normal-execution path without
the mandatory RGBA capture. In that mode the test requires 120 Present
submissions and terminal source progress but does **not** assert pixel content
or motion. Hosted CI runs both variants on each Boids law.

Each native smoke prints the host-observed submitted-Present rate, its
95th-percentile interval, maximum immutable steps outstanding, and maximum
CPU fixed-time ticks not yet captured into the GPU producer. Rates exclude
startup prior to the first submitted Present and are **not** GPU execution
times, actual monitor scanout, or stable benchmark measurements. Run the
no-readback mode in a **release build** on the owner GPU for the comparison
against the capture-enabled diagnostic mode. The hosted software Vulkan
figures cannot qualify RX 7900 XTX responsiveness.

This is not a complete Boids-buffer proof or a substitute for owner-native
original/species acceptance.

The draw pass uses a producer-owned `ProceduralCamera2d` with
`FillViewport { fixed_axis: Vertical }`. Renderer procedural code derives the
projection uniform from camera intent and target size; `PreparedViewFrame` does
not own camera truth. The boids shader projects world positions through
`ProceduralCamera2dUniform`, so landscape, portrait, square, and extreme aspect
surfaces keep equal projected world x/y scale.

Run the original flocking law:

```bash
cargo run -p engine --example boids_render_flow
```

Run the **alternative three-species local-rule experiment** through the same
Runenwerk App, ECS state, RenderFlow, fixed-step, GPU grid and instanced
impostor renderer:

```bash
cargo run -p engine --example boids_render_flow -- --species
```

The three example-local species use deterministic instance-index identities.
Their asymmetric attraction/repulsion law and short-range exclusion produce
different interactions from the default alignment/cohesion/separation model.
The color indicates species; the geometric impostor is **not** a world-solid
or the authority for any simulation state. Both laws remain shader-defined,
not general-purpose authored rule programs. There is no validated CPU oracle,
adaptive split/merge, persistent world truth, SDF interaction, or guaranteed
numerical determinism across backends. The goal is a bounded, visual
**Runenwerk-native feasibility comparison**, not architectural adoption.

Evidence:

```bash
cargo run -p engine --example boids_render_flow -- --evidence
```

The evidence output includes the canonical pass order, fixed-step contract,
grid capacity counters, unsupported GPU timing diagnostics, CPU preflight
timing, benchmark command, camera projection evidence for landscape, portrait,
square, and extreme aspect surfaces, and resize pixel evidence proving sprite
aspect reconstruction.

Benchmarks:

```bash
cargo bench -p engine --bench render_flow_planning
```

The benchmark suite includes procedural population cases for prefix scan
planning, scan/scatter/indirect-args planning, bounded-grid build planning,
boids production flow planning/preflight, and boids production evidence
reporting.

Tests:

```bash
cargo test -p engine --example boids_render_flow
```

### Sustained native performance qualification

The native smoke defaults to 120 submitted Presents. For a longer bounded
release-build run, set `RUNENWERK_BOIDS_NATIVE_PRESENT_TARGET=600` together
with `RUNENWERK_BOIDS_NATIVE_SMOKE=1`; the valid target is 120–1200. The test
retains enough native frame observations for that target and still enforces its
existing 180-second fail-closed deadline. Compare normal no-readback execution
(`RUNENWERK_BOIDS_NATIVE_NO_READBACK=1`) with the capture-enabled diagnostic
mode at the same resolution, adapter and law. The report additionally prints
averages of the existing renderer CPU-stage sum, flow encode and encode/submit
timings; the sum does not include the complete ECS Update or GPU execution.
Host-observed submitted-Present pacing is not display scanout latency.
