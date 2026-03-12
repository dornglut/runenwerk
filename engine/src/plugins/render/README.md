# Render Plugin

Render plugin entry surface for the active engine runtime path.

## Start Here

- Plugin entry: `engine/src/plugins/render/plugin.rs`
- Domain re-exports: `engine/src/plugins/render/domain.rs`
- Architecture details: `engine/src/plugins/render/docs/README.md`

## Module Map

- `render_graph_registry/`
  - Feature-owned pass/pipeline/resource registration APIs.
- `render_executor_registry/`
  - Executor registration and frame data access.
- `shader_manager/`
  - Shader source registration, discovery, and hot reload.
- `renderer/`
  - Graph compile/execute flow and frame submission plumbing.
- `submit.rs`
  - Systems wired into `RenderPrepare` and `RenderSubmit`.

## Notes

- This file exists to keep plugin docs shape consistent with other plugins.
- Longer design notes live under `engine/src/plugins/render/docs/`.
