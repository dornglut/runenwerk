---
title: PT-RUNENGPU-G1A Owner-Scoped GPU Work-Resource Identity Closeout
description: Main-branch implementation, identity migration, fallible authoring, foreign-handle rejection, validation, and remaining-boundary evidence for RunenGPU G1A.
status: completed
owner: workspace
layer: reports
canonical: true
last_reviewed: 2026-07-26
related_docs:
  - ../../workspace/specs/pt-runengpu-g1a-logical-work-resource-identity.ron
  - ../../workspace/planning/completed-work.md
  - ../../workspace/planning/roadmap.md
  - ../../adr/accepted/0015-separate-gpu-execution-from-rendering.md
  - ../investigations/runengpu-render-s0-inventory.md
---

# PT-RUNENGPU-G1A Owner-Scoped GPU Work-Resource Identity Closeout

## Outcome

RunenGPU G1A is implemented, reviewed, exact-head validated, and merged on Runenwerk `main` as the first internal future-transferable RunenGPU boundary.

The slice replaced renderer-owned scalar logical resource identity with GPU-owned owner-scoped identity, made every resource-allocating authoring path structurally fallible, and proved that foreign-flow resource handles cannot collide with unrelated local resources.

G1A does not create an external `runen-gpu` package, establish standalone conformance, or authorize later G2-G8 implementation.

## Accepted revisions

| Evidence | Revision |
|---|---|
| Owning issue | `#131` |
| Implementation pull request | `#164` |
| Accepted implementation base | `88e9de3c94145c39391550efaf82a341630f081a` |
| Reviewed implementation head | `fbce0bf1a871cadb92c823fe47178db5f0e44630` |
| Accepted merge on `main` | `5bbdab36ae661d99432bfe5d215062c397aac975` |

The implementation PR contained one commit and changed 67 files. It changed no Cargo manifest, workspace membership, dependency, lockfile, documentation, or workflow authority.

## Delivered contract

### GPU-owned identity

`engine::plugins::gpu` now owns:

- public `GpuWorkResourceId` with private owner-scope and nonzero local components;
- equality, ordering, hashing, `Debug`, and `Display` over the complete identity;
- owner-controlled `GpuWorkResourceIdAllocator`;
- structured `GpuWorkResourceIdAllocationError` exhaustion;
- a crate-private owner bridge seeded from the already-created nonzero `RenderFlowId` value.

The GPU identity module imports no renderer, ECS, WGPU, Winit, application, or domain types.

### Old-authority removal

The implementation deleted:

- `RenderResourceId`;
- `RenderResourceIdSequence`.

No compatibility alias, deprecated parallel name, forwarding re-export, duplicate allocator, source include, or compatibility module remains.

### Fallible authoring

The implementation introduced structured `RenderFlowAuthoringError` and migrated every accepted resource-allocating flow, pass, procedural, GPU-primitive, application, example, benchmark, and test authoring path.

Production paths propagate through `Result`, `?`, or explicit structured mapping. Non-allocating fluent methods remain infallible. The allocation path contains no production panic, `unwrap`, `expect`, sentinel identity, duplicate identity, silent fallback, wrap, or saturation into reuse.

### Foreign-handle correctness

Owner scope participates in identity and resource lookup. Equal local components allocated by different live flows produce distinct full identities.

Focused tests prove that:

- a foreign `UniformHandle` is rejected even when the receiving flow owns the same local sequence value;
- a foreign `StorageArrayHandle` is rejected under the same collision condition;
- a foreign handle never resolves to an unrelated local resource.

## Exact scope

The accepted migration was bound by three inventories:

| Inventory | Files | Result |
|---|---:|---|
| Identity consumers | 38 | migrated or compiler-backed unchanged |
| Core and direct fallible authoring | 27 | migrated or compiler-backed unchanged |
| Transitive application-builder callers | 6 | migrated to explicit successful-construction assertions |

The implementation changed exactly 67 authorized files, including four new GPU API or module files. No consumer outside the accepted inventories was left unresolved, and no forbidden later-phase behavior was introduced.

## Validation evidence

Serialized exact-tree prepublication workflow run `30172679473` passed and produced the 67-file blob manifest used for the atomic implementation commit.

Permanent exact-head workflow run `30173979203` passed on reviewed head `fbce0bf1a871cadb92c823fe47178db5f0e44630`.

Passed gates included:

- exact 67-file scope enforcement;
- zero remaining old-name references;
- forbidden-constructor and forbidden-import guards;
- `cargo +stable fmt --all --check`;
- focused identity, exhaustion, foreign-handle, resource, and authoring-error tests;
- `cargo test -p engine --lib --locked`;
- `cargo test -p engine --tests --locked`;
- `cargo test -p runenwerk_draw --locked`;
- `cargo test -p runenwerk_editor --locked`;
- strict all-target Clippy for engine, draw, and editor;
- canonical `cargo +stable validate`;
- final diff and clean tracked-state checks.

No unresolved review thread or submitted review requiring action remained at acceptance.

## Temporary artifact disposition

Temporary PR `#165` existed only to produce the completed phase-spec artifact from accepted merge `5bbdab36ae661d99432bfe5d215062c397aac975`.

The artifact changed only:

- `lifecycle_state` to `completed`;
- `implementation_authorization_status` to `implemented-and-merged`;
- `source_baseline` to the accepted implementation base `88e9de3c94145c39391550efaf82a341630f081a`.

PR `#165` was closed without merge. No temporary workflow or trigger file became product or documentation authority.

## Remaining boundaries

G1A deliberately does not provide:

- capability and resource-descriptor ownership correction;
- access, lifetime, hazard, or work-fragment contracts;
- shader or pipeline admission;
- WGPU realization;
- compute, upload, or readback contracts;
- offscreen graphics or shared non-render consumer proof;
- surface generations or device outcomes;
- diagnostics, shutdown, or standalone conformance;
- an external RunenGPU repository or Runenwerk cutover.

Those remain G2-G8 and GX work. G1A completion must not be interpreted as extraction readiness.

## Next safe action

Author one decision-complete G2 capabilities and resource-descriptor specification from merged G1A `main` facts.

Before G2 source implementation:

1. inventory current capability and resource-descriptor declarations, consumers, validation, runtime realization, and ownership seams;
2. define the future-transferable GPU-owned contract and renderer-owned policy boundary;
3. bind exact files, migrations, tests, dependency guards, exclusions, and stop conditions;
4. create and accept the owning G2 issue and specification;
5. implement only that bounded slice with exact-head evidence.

Do not create the external RunenGPU repository, start G3-G8, or infer later contracts from this closeout.
