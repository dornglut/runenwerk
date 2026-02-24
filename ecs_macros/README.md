# ECS Macros Crate

## Purpose

Provides procedural macros that support ECS ergonomics (for example derive macros used by ECS component types).

## Usage

- Crate: `ecs_macros`
- Consumed by `ecs`/`engine` via macro derives.

## Ownership Boundaries

- Owns macro expansion logic and compile-time code generation support.
- Does not own runtime ECS storage/query behavior.

## Extension Points

- Add derive and helper macros needed by ECS API ergonomics.
- Keep generated code aligned with `ecs` crate contracts.
