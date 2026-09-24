---
title: "ECS Macros Crate"
description: "Documentation for ECS Macros Crate."
status: active
owner: ecs
layer: domain
canonical: true
last_reviewed: 2026-04-27
---

# ECS Macros Crate

Proc-macro derives for `ecs`.

## Purpose

- Provide `#[derive(Component)]` and `#[derive(Bundle)]` for `ecs`.

## Ownership Boundaries

- Owns derive macro expansion only.
- Does not own ECS runtime behavior.
