---
title: "ECS Macros Crate"
description: "Documentation for ECS Macros Crate."
---

# ECS Macros Crate

Proc-macro derives for `ecs`.

## Purpose

- Provide `#[derive(Component)]` and `#[derive(Bundle)]` for `ecs`.

## Ownership Boundaries

- Owns derive macro expansion only.
- Does not own ECS runtime behavior.
