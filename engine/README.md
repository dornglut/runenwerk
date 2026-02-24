# Engine Crate

## Purpose

Hosts the runtime, plugin composition, and core engine-facing feature implementations.

## Usage

- Crate: `engine`
- Entry surface: runtime/app APIs under `engine::runtime` and `engine::platform`.
- Features are organized under `engine/src/plugins/*`.

## Ownership Boundaries

- Owns engine runtime loop, plugin wiring, and integrated feature implementations.
- Consumes ECS/scheduler crates for data model and execution ordering.
- Does not own ECS core internals or scheduler core internals.

## Extension Points

- Add plugins under `engine/src/plugins/*`.
- Register plugins through app/runtime composition paths.
- Extend plugin-local `README.md` and `requests.md` for feature evolution.
