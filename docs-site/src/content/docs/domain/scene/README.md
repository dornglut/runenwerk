---
title: Scene
description: Current crate documentation for the scene domain crate.
status: active
owner: scene
layer: domain
canonical: true
last_reviewed: 2026-04-28
---

# Scene

`scene` currently owns engine-agnostic transform value types shared by scene,
editor, and runtime integration code.

## Purpose

Use this crate when a domain or app needs stable transform contracts without
depending on engine scene runtime internals.

## Public Surface

- `LocalTransform`: local translation/rotation/scale data.
- `WorldTransform`: resolved world-space transform data.
- `Vec3Value` and `QuatValue`: serializable math value wrappers.

## Ownership Boundary

`scene` owns transform data contracts. It does not own scene lifecycle,
registration, ECS storage, render extraction, editor outliner behavior, or asset
loading.

## Related Crates

- `engine/src/plugins/scene` owns runtime scene lifecycle and publication.
- `domain/editor/*` consumes scene contracts for editor presentation and
  persistence.
