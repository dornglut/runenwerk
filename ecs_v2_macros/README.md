# ECS V2 Macros Crate

Proc-macro derives for `ecs_v2`.

## Purpose

- Provide `#[derive(Component)]` and `#[derive(Bundle)]` for `ecs_v2`.

## Ownership Boundaries

- Owns derive macro expansion only.
- Does not own ECS runtime behavior.

