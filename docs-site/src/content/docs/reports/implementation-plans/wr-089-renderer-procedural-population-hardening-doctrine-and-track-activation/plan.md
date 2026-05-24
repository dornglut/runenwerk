---
title: WR-089 Renderer Procedural Population Hardening Doctrine And Track Activation Implementation Contract
description: Bounded implementation contract for activating the procedural population hardening production track without product code changes.
status: active
owner: engine
layer: engine-runtime / renderer
canonical: false
last_reviewed: 2026-05-24
related_designs:
  - ../../../design/active/renderer-procedural-population-hardening-platform-design.md
  - ../../../design/active/renderer-procedural-population-platform-design.md
related_reports:
  - ../../closeouts/pt-render-procedural-population-runtime-proven/closeout.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# WR-089 Renderer Procedural Population Hardening Doctrine And Track Activation Implementation Contract

## Goal

Implement `PM-RENDER-POP-HARDEN-001` / `WR-089` as doctrine and production
track activation only.

This slice creates the accepted planning surface for
`PT-RENDER-PROCEDURAL-POPULATION-HARDENING` and splits implementation into
`WR-090`, `WR-091`, `WR-092`, and `WR-094`, with `WR-093` reserved for
evidence, benchmark, documentation, and track closeout. It must not change
renderer product code.

## Source Of Truth

- `docs-site/src/content/docs/reports/closeouts/pt-render-procedural-population-runtime-proven/closeout.md`:
  identifies the exact gaps this track closes.
- `docs-site/src/content/docs/design/active/renderer-procedural-population-platform-design.md`:
  existing bounded population doctrine and non-goals.
- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`:
  new hardening doctrine for indirect draw validation, primitive dispatch, and
  graph catch-up scheduling, and procedural camera projection.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  production sequencing source.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  WR execution source.
- `engine/src/runtime/fixed_time.rs`:
  source truth for runtime `FixedTimeConfig`, `FixedTimeState`, and
  `CatchupBudget` resources that WR-092 must consume rather than duplicate.

## Readiness

Required workflow commands were run before this contract:

- `task roadmap:intake -- --idea "Renderer procedural population hardening track for indirect draw validation, reusable GPU primitive shader dispatch, and fixed-step graph catch-up scheduling after PT-RENDER-PROCEDURAL-POPULATION runtime_proven closeout"`
- `task ai:architecture-governance -- --task "Renderer procedural population hardening track proposal" --scope "engine/src/plugins/render/api, engine/src/plugins/render/graph, engine/src/plugins/render/renderer/render_flow, engine/src/plugins/render/gpu_primitives, engine/src/plugins/render/procedural/population, docs-site/src/content/docs/workspace, docs-site/src/content/docs/design"`
- `task roadmap:intake -- --idea "Renderer spatial hash and chunked unbounded procedural population design after bounded-grid procedural population evidence"`
- `task ai:architecture-governance -- --task "Renderer procedural population hardening revision for fixed-step input invariance, reusable procedural camera projection, and behavior-authoring follow-up" --scope "engine/src/plugins/render/procedural, engine/src/plugins/render/graph, engine/src/plugins/render/renderer/render_flow, engine/examples/boids_render_flow, assets/shaders/boids_*, docs-site/src/content/docs/workspace, docs-site/src/content/docs/design"`
- `task production:plan -- --milestone "PM-RENDER-POP-HARDEN-001" --roadmap "WR-089"`
- `task roadmap:intake -- --idea "Renderer procedural population behavior authoring and boids split merge dynamics after fixed-step and camera correctness"`

Architecture governance implication: this is architecture-sensitive renderer
work. The renderer remains a complicated-subsystem owner for derived execution
data. Product, gameplay, world, and streaming source truth stay outside the
renderer.

## Implementation Scope

Owned files and exact modules:

- `docs-site/src/content/docs/design/active/renderer-procedural-population-hardening-platform-design.md`:
  active doctrine for the hardening track.
- `docs-site/src/content/docs/design/active/README.md`:
  active design index entry.
- `docs-site/src/content/docs/workspace/production-tracks.yaml`:
  `PT-RENDER-PROCEDURAL-POPULATION-HARDENING` and milestones
  `PM-RENDER-POP-HARDEN-001` through `PM-RENDER-POP-HARDEN-006`.
- `docs-site/src/content/docs/workspace/roadmap-items.yaml`:
  `WR-089` through `WR-094` and dependency edges.
- `docs-site/src/content/docs/reports/implementation-plans/wr-089-renderer-procedural-population-hardening-doctrine-and-track-activation/plan.md`:
  this contract.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-renderer-procedural-population-hardening/proposal.yaml`:
  intake evidence refined to the doctrine slice.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-renderer-procedural-population-hardening/proposal.md`:
  human-readable intake evidence.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-renderer-procedural-population-behavior-authoring/proposal.yaml`:
  separate behavior-authoring intake refined to `WR-095` so `WR-094` remains
  available for the procedural camera slice.
- `docs-site/src/content/docs/reports/roadmap-intake/2026-05-24-renderer-procedural-population-behavior-authoring/proposal.md`:
  human-readable behavior-authoring intake evidence.
- `docs-site/src/content/docs/reports/implementation-plans/wr-092-fixed-step-graph-catch-up-scheduling/plan.md`:
  tightened contract requiring runtime fixed-time source reuse and
  iteration-scoped uniform projection.
- `docs-site/src/content/docs/reports/implementation-plans/wr-094-procedural-camera-and-view-projection/plan.md`:
  new procedural camera and view projection implementation contract.
- `docs-site/src/content/docs/reports/implementation-plans/wr-093-procedural-population-hardening-evidence-benchmarks-docs-and-closeout/plan.md`:
  closeout contract updated to depend on `WR-094` evidence.

## Acceptance Criteria

- The hardening track exists in `production-tracks.yaml` with target
  completion quality `runtime_proven`.
- `WR-089` is doctrine and activation only.
- `WR-090`, `WR-091`, `WR-092`, and `WR-094` are bounded implementation
  slices with explicit ownership, non-goals, validation, and closeout
  expectations.
- `WR-093` is the closeout slice and depends on `WR-094`.
- `WR-092` explicitly reuses `FixedTimeConfig`, `FixedTimeState`, and
  `CatchupBudget`, adds iteration-scoped uniform projection, and proves
  input/redraw-rate invariance.
- `WR-094` owns reusable procedural camera projection and defines fill-viewport
  aspect correctness without moving camera truth into `PreparedViewFrame`.
- Spatial hash and chunked unbounded populations are represented only by a
  separate intake proposal.
- Richer boids split/merge behavior is represented only by a separate
  behavior-authoring intake proposal.
- No product code changes are included in this slice.

## Non-Goals

- Do not harden indirect draw code in `WR-089`.
- Do not add primitive shaders in `WR-089`.
- Do not add graph catch-up scheduling in `WR-089`.
- Do not add procedural camera projection code in `WR-089`.
- Do not fold unbounded populations into this track.
- Do not fold behavior authoring or richer boids dynamics into this track.
- Do not claim `perfectionist_verified`.

## Stop Conditions

- Stop if the active design would move gameplay, world, product, or streaming
  source truth into renderer ownership.
- Stop if the generated intake remains a single monolithic hardening row.
- Stop if production or roadmap validation rejects the split WR sequence.

## Closeout Requirements

Closeout must live under:

`docs-site/src/content/docs/reports/closeouts/wr-089-renderer-procedural-population-hardening-doctrine-and-track-activation/closeout.md`

Completion quality target: `bounded_contract`.

Known quality gaps must include:

- indirect draw runtime hardening remains `WR-090`;
- primitive shader dispatch remains `WR-091`;
- graph catch-up scheduling remains `WR-092`;
- procedural camera and view projection remains `WR-094`;
- evidence, benchmarks, docs, and track closeout remain `WR-093`;
- behavior authoring and richer boids dynamics remain separate `WR-095`
  intake/design work;
- final no-gap renderer verification remains `PT-RENDER-PERFECTION`.

Validation:

- `task roadmap:render`
- `task roadmap:validate`
- `task roadmap:check`
- `task production:render`
- `task production:validate`
- `task production:check`
- `task docs:validate`
- `task planning:validate`

## Critical Review

The main failure mode is treating hardening as one broad renderer cleanup item.
The contract avoids that by making `WR-089` doctrine-only and putting every
runtime behavior change into a later WR with exact owning files. The second
failure mode is allowing boids to fake fixed time locally; WR-092 must consume
the runtime fixed-time source and add graph-level repeated execution plus
iteration-scoped uniform projection. The third failure mode is making camera
projection a boids-only draw-parameter patch or storing camera truth in
`PreparedViewFrame`; WR-094 keeps camera intent producer-owned and renderer
projection derived. Spatial hash/chunked unbounded populations and richer flock
behavior add different ownership questions, so they stay as separate intake and
design work.

