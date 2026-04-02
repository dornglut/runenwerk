---
title: Roadmap Overview
description: Overview of the engine-agnostic domain roadmaps.
---

## Current Domain Priority

ECS core primitives are now considered usable platform foundation.
Near-term roadmap priority is integration and correctness across engine/world/runtime boundaries,
not additional basic ECS primitives.

## Domain Roadmap (Post-ECS Foundation)

| Priority | Area | Goal | Notes |
| --- | --- | --- | --- |
| **1 (Now)** | World correctness | Stabilize dirty-chunk invalidation, runtime record bootstrap, authoritative build/integration contracts, and collision-readiness semantics. | Engine-facing correctness gate for all downstream work. |
| **1 (Now)** | Runtime alignment | Keep world maintenance on fixed-step simulation cadence with explicit scheduling relative to simulation/replication. | Required for deterministic multiplayer/world sync. |
| **1 (Now)** | Integration contracts | Centralize world edit ingress and bounds-driven invalidation bridges; remove game-level ad hoc dirty map writes. | Reduces duplication and policy drift. |
| **2 (Next)** | World/render boundary | Move from stringly world contribution IDs to typed IDs and explicit residency invalidation intent contracts. | Improves safety and cache coherency. |
| **2 (Next)** | Streaming contracts | Extend world-owned replication/interest baseline with richer per-connection policy and selective payload shaping. | Foundation for large-world region workflows. |
| **3 (Later)** | Region-oriented infrastructure | Promote region-centric change journals and invalidation surfaces to support SAG-style world-region pipelines. | Do not treat spatial hash backend as SAG itself. |

## Current Execution Notes

- World correctness gate v2 is complete in engine/world:
  - build payload dispatch now derives deterministic payloads from op-window edits in the active path
  - collision authority readiness and fail-safe semantics are explicit and instrumented
  - world revision progression is tied to accepted integrations
- Fixed-step scheduling alignment is now explicit for key Cavern Hunt gameplay modules relative to world integration and replication.
- Bounds-driven world->render invalidation bridge is now active with deterministic dedupe and no-loss queueing across render cache resource loss/recreation.
- Typed chunk-linked world contribution IDs are now active at world->render prepare handoff; chunk identity no longer depends on string labels in core world payloads.
- World runtime now produces replication header/content/op-window/residency state directly; Cavern checkpoint capture consumes that resource instead of rebuilding from disparate world resources.
- Per-connection streaming interest cursor/resync progression is now world-owned and advanced by net replication send/ack/disconnect paths.
- Region-centric baseline is now active: world runtime maintains a deterministic region invalidation journal and replication projects region/chunk invalidation deltas from that world-owned surface.
- Connection-aware checkpoint shaping now consumes acknowledged region progression and relevant chunk sets, emitting selective incremental world payloads instead of implicit full payload fallback.
- Minimal world observability now includes streaming/journal progression metrics in runtime inspector/debug overlay snapshots.
- Cavern snapshot restore no longer rebuilds legacy collision-field authority from topology.
- Cavern runtime plugin wiring and geometry-edit runtime path no longer initialize/mutate legacy collision-field authority resources.
- Cavern runtime resource marker wiring no longer registers legacy `CavernCollisionField` as an ECS runtime resource component.
- Cavern worldgen bootstrap no longer reads legacy geometry resources for extraction-seal IDs or initial seed bounds; compatibility metadata now comes from edit outcomes + topology bounds.
- Material runtime scaffolding now uses world authority revision directly without legacy geometry-graph fallback.
- Cavern runtime hard-cut is complete for authority surfaces:
  - legacy `domain/world/collision_field` module removed from runtime domain surface
  - runtime worldgen/loot/snapshot paths no longer use `CavernGeometryGraph` authority resources
  - active snapshot wire contract is `CavernRunSnapshotV2` / `CavernRunDeltaV2` with explicit V1 decode rejection

## Explicitly Deferred

- New basic ECS primitives as the main roadmap axis.
- Octree/BVH backend work before world integration gates are closed.
- Editor/product/history/networking expansions that are not required by world correctness and integration milestones.
