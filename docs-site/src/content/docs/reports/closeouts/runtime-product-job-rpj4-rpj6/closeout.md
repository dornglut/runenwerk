---
title: Runtime Product Job RPJ4-RPJ6 Closeout
description: Completion and drift-check record for Draw responsiveness, cache identity, and work-stealing runtime job substrate.
status: completed
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-14
related_designs:
  - ../../../design/accepted/execution-fabric-and-product-jobs-design.md
  - ../../../design/accepted/field-product-contracts-diagnostics-and-residency-design.md
related_roadmaps:
  - ../../../engine/roadmaps/runtime-product-job-executor-roadmap.md
  - ../../../apps/runenwerk-draw/roadmap.md
  - ../../../workspace/roadmap-index.md
related_apps:
  - ../../../apps/runenwerk-draw/README.md
related_domains:
  - ../../../domain/drawing/README.md
---

# Runtime Product Job RPJ4-RPJ6 Closeout

## Status

Complete as of 2026-05-14 for the local runtime product-job substrate through
Draw responsiveness, backend-neutral cache identity, and the first
work-stealing executor backend.

This closeout does not complete persistent package sidecars, disk cache
archives, GPU product jobs, ECS parallel waves, or cross-process/distributed
execution.

## Completion Evidence

- `runenwerk_draw` keeps pen input immediate through `StrokePrimitive` while
  committed and preview tile formation run as owned runtime job snapshots.
- Draw installs a bounded worker executor by default while the engine default
  remains serial.
- `domain/product` now exposes `ProductCacheIdentity` derived from
  `ProductDescriptorCore` lineage, source revision, formation version, product
  kind, scale band, and producer/upstream inputs.
- `domain/drawing` tile formation includes quality class in determinism,
  descriptor generation, cache keys, product scale band, and render selection;
  preview and final quality tiles have distinct identities before persistence.
- `engine/src/runtime/jobs` preserves serial and fixed worker-pool modes and
  adds a work-stealing backend behind the existing `RuntimeJob` API.
- Runtime job diagnostics now expose executor mode, worker configuration,
  pending handles, latest generations, recent rejected/failed/stale issues, and
  per-drain activity.

## Deferred Work

- Persistent product caches, cache pruning, and Draw package sidecars.
- GPU job execution and readback publication through product/query barriers.
- Full ECS parallel waves and `Send`/`Sync` system-param policy.
- Cross-process/distributed jobs and their trust, artifact, timeout, and retry
  policy.
- General runtime job diagnostics UI beyond inspection DTOs.

## Validation

Validation passed on 2026-05-14:

- `cargo fmt --all -- --check`
- `cargo test -p product`
- `cargo test -p drawing --test ink_tile`
- `cargo test -p engine runtime_job`
- `cargo test -p runenwerk_draw --test app_shell`
- `cargo test --workspace --all-features --quiet`
- `python tools/docs/validate_docs.py`
