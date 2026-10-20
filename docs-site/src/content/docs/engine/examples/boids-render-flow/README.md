---
title: Boids Render Flow Example
description: GPU-resident flocking and species examples using ordinary RenderFlow fixed-step execution.
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-10-20
publication: primary
---

# Boids Render Flow Example

A windowed, GPU-resident particle simulation authored through the public `RenderFlow` and
procedural population APIs. Both the original flocking law and an alternative three-species
interaction law use the same eight-stage compute graph, grid-based neighbor search, instanced
draw, and Present path.

## Run

```bash
cargo run -p engine --example boids_render_flow
cargo run -p engine --example boids_render_flow -- --species
```

The `--species` mode assigns deterministic instance-index identities to three species with
asymmetric attraction, repulsion, and short-range exclusion. The original law uses alignment,
cohesion, and separation. Colors distinguish species; neither mode is a general-purpose authored
rule system.

## Flow and ownership

The authored passes execute in this order:

1. `boids.seed_or_hold` — initialize on the first source step; preserve retained state thereafter
2. `boids.grid.clear_counts`
3. `boids.grid.count_cells`
4. `boids.grid.scan_counts`
5. `boids.grid.reset_cursors`
6. `boids.grid.scatter_sorted_indices`
7. `boids.grid.simulate_neighbors`
8. `boids.grid.publish_draw` — publish the final GPU witness and render-facing instances
9. `boids.draw` — procedural impostors through `assets/shaders/boids_compose.wgsl`
10. `boids.present` — present `boids.color` to the native surface

The eight compute passes use `assets/shaders/boids_compute.wgsl`. The bounded uniform grid uses
explicit cell counts, offsets, cursors, and agent-count-sized sorted indices; it rejects
overflow rather than dropping neighbors. Retained ping-pong buffers and grid scratch storage
request descriptor-level zero initialization. Zero-filled buffers are **not** equivalent to
semantically seeded particle state.

The renderer owns GPU preparation and submission through RunenGPU G5. The Boids producer owns
source-time inputs and simulation meaning; GPU terminal completion alone never advances its
verified simulation clock. `BoidsGpuStepSchema` captures each compute pass's uniform bytes and
dispatch dimensions at source time. The procedural camera uses `ProceduralCamera2d` with a
vertical-axis viewport fit so impostors maintain consistent world-space proportions across
aspect ratios.

## Fixed-step pacing and verification

`boids.fixed_step` admits a maximum of **four outstanding source steps**, including accepted but
unverified work. Inputs already captured remain immutable and ordered. If the GPU cannot accept
additional work, excess host fixed-time opportunities are **explicitly time-dilated**, not
queued for later replay or counted as completed Boids ticks. The source's requested and verified
tick identities therefore remain contiguous, but its simulation may run slower than wall time
under sustained pressure. A consumer with mutable external inputs must snapshot those inputs at
its own source time.

Each accepted batch requires a **24-byte GPU-produced final-stage buffer witness** from its
exact G5 submission: final source tick, GPU-derived grid population, and the position/velocity
of one simulated particle. Only a completed submission with the expected identity, ready
readback, correct source tick, population, and valid sampled motion permits `commit_verified`.
This is a bounded integrity and freshness check, **not** a numerical oracle for every particle.

Full-frame RGBA/BGRA readback is **not** part of ordinary Boids execution. Opt-in native visual
conformance additionally checks actual GPU-rendered image content and changes between frames.
Rendered motion does not prove full particle-buffer correctness.

## Failure and recovery

Failed or uncertain accepted GPU work, failed required readback, and a changed GPU-context
generation **fail closed** before further source admission. Do not retry an accepted interval
against unverified retained state.

The accepted recovery policy for this **ephemeral example** is to terminate the old run and
start a **new process, GPU context, RenderFlow, and simulation epoch at tick zero**. Previous
particle positions and velocities are **not restored**. A controlled native source-failure test
verifies fail-stop and successful fresh-process startup for both laws; it does **not** inject
physical device loss, partial GPU writes, or prove in-process restoration. State-preserving
consumers need their own verified checkpoint/reimport contract.

## Tests and diagnostics

```bash
cargo +stable test -p engine --example boids_render_flow --locked
cargo run -p engine --example boids_render_flow -- --evidence
cargo bench -p engine --bench render_flow_planning
```

The example tests cover graph and shader declarations, historical source capture, resource
initialization, pacing, and verification. The `--evidence` mode prints authored pass,
bounded-grid, timing-availability, and camera/resize evidence; the benchmark measures planning
and evidence generation, not GPU shader time.

To run the actual native GPU smoke (repeat with `-- --species` for the second law):

```bash
RUNENWERK_BOIDS_NATIVE_SMOKE=1 cargo +stable run -p engine --example boids_render_flow --locked
```

| Setting | Effect |
| --- | --- |
| `RUNENWERK_BOIDS_NATIVE_SMOKE=1` | Require 120 native submitted Presents by default, terminal GPU results, verified ticks, and changing images |
| `RUNENWERK_BOIDS_NATIVE_NO_READBACK=1` | With smoke enabled, omit **full-image** readback; the mandatory compact GPU witness remains required |
| `RUNENWERK_BOIDS_NATIVE_PRESENT_TARGET=600` | With smoke enabled, request a longer bounded run (supported range: 120–1200) |
| `RUNENWERK_BOIDS_NATIVE_SOURCE_FAULT_AFTER_TICK=12` | With smoke enabled, deliberately stop after the specified verified tick for the controlled failure fixture |

The smoke has a 180-second safety deadline and closes its native window on success. It records
submitted-Present rates, intervals, CPU encoding stages, bounded outstanding demand, and
deferred/time-dilated steps. These are **host observations**, not hardware scanout FPS or GPU
shader duration. The CI lane exercises both simulation laws with and without optional image
capture.

The ordinary interactive example approves native close intent without further GPU-step
admission. Console interruption is not equivalent to a normal window close. For validation
requirements and native test commands, see the repository
[TESTING.md](https://github.com/dornglut/runenwerk/blob/main/TESTING.md).
