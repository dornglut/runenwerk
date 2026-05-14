---
title: Runtime Product Job RPJ7A Cache Policy Closeout
description: Completion record for backend-neutral runtime cache policy and the Draw in-memory cache proof.
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
related_reports:
  - ../runtime-product-job-rpj4-rpj6/closeout.md
---

# Runtime Product Job RPJ7A Cache Policy Closeout

## Status

Complete as of 2026-05-14 for backend-neutral runtime cache policy and the
Draw in-memory committed-tile cache proof.

This closeout does not complete persistent package sidecars, disk cache
archives, cache pruning, GPU product jobs, ECS parallel waves, or
cross-process/distributed execution.

## Completion Evidence

- `domain/product` exposes `ProductCacheKey` derived from
  `ProductCacheIdentity` and portable cache decision DTOs for hit, miss, stale,
  rejected, write-failed, and preserved last-good states.
- `engine/src/runtime/product_cache.rs` owns a metadata-only
  `RuntimeProductCacheResource` with cache entries, decision history, and
  diagnostics. It stores descriptors and identities, not product payload bytes,
  package paths, renderer handles, or app policy.
- Draw owns the first payload proof: app runtime state maps drawing tile source
  keys to cached `ProductCacheKey` values and keeps in-memory tile payloads in
  the app layer.
- Draw committed tile publication checks the app-owned payload cache and engine
  runtime metadata before submitting a committed tile job. Accepted cache hits
  stage products through the existing product publication and query snapshot
  barriers without submitting another runtime job.
- Runtime cache metadata is updated only after product publication accepts
  formed descriptors.

## Deferred Work

- Persistent product caches, cache pruning, and Draw package sidecars.
- Disk artifact format, trust policy, package migration, and corruption
  recovery beyond in-memory diagnostics.
- GPU job execution and readback publication.
- Full ECS parallel waves.
- Cross-process/distributed job execution.

## Validation

Validation passed on 2026-05-14:

- `cargo fmt --all -- --check`
- `cargo test -p engine runtime_job`
- `cargo test -p engine runtime_product_cache`
- `cargo test -p product`
- `cargo test -p drawing --test ink_tile`
- `cargo test -p runenwerk_draw --test app_shell`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features --message-format=short -- -D warnings`
- `cargo test --workspace --all-features --quiet`
- `python tools/docs/validate_docs.py`
- `./quiet_full_gate.sh`
