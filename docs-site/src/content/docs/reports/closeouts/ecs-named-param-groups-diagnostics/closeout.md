---
title: ECS Named Param Groups Diagnostics Closeout
description: Closeout for first-class named ECS SystemParam groups with nested runtime diagnostics.
status: completed
owner: net
layer: domain
canonical: true
last_reviewed: 2026-05-16
related_roadmaps:
  - ../../../net/ecs-runtime-prioritized-roadmap.md
related_reports:
  - ../wr-025-interaction-v2-doctrine-repair/closeout.md
---

# ECS Named Param Groups Diagnostics Closeout

## Status

Complete as of 2026-05-16.

This repair turns grouped system params from opaque tuple aliases into an ECS-owned diagnostic surface. It keeps scheduler access truth in `SystemAccess`, keeps extraction/access ownership in `domain/ecs`, and keeps the scheduler generic by storing only nested param descriptors.

## Completion Evidence

- `domain/scheduler/src/system.rs` now models param descriptors as leaves or groups, with optional child names and nested children.
- `domain/ecs/src/system/param_metadata.rs` converts scheduler descriptors into recursive `ParamSlotMetadata` with path-based ids.
- `domain/ecs_macros/src/lib.rs` adds `#[derive(ecs::SystemParam)]` for named structs so grouped params can preserve field names, child access, and recursive diagnostics.
- `domain/ecs/src/system/params.rs` keeps tuple params as low-level support while exposing indexed children instead of one opaque tuple leaf.
- `apps/runenwerk_draw/src/runtime/systems.rs` uses a named `DrawingFrameSubmissionResources` param group for frame submission resources.

## Validation

Required validation for this slice:

- `cargo check -p ecs_macros`
- `cargo check -p ecs`
- `cargo check -p runenwerk_draw`
- `cargo test -p ecs`
- `cargo clippy -p ecs --all-targets --all-features --message-format=short -- -D warnings`
- `cargo clippy -p runenwerk_draw --all-targets --all-features --message-format=short -- -D warnings`
- `task docs:validate`
- `task roadmap:validate`
- `./quiet_full_gate.sh`

## Drift Notes

- WR-025 remains UI/editor interaction work. This closeout only records the separate Draw gate-hygiene follow-up that exposed the need for named ECS param groups.
- Tuple params remain supported for small local composition, but named struct groups are the preferred long-term app-facing pattern when a group carries domain meaning.
