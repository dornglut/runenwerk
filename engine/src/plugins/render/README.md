# Render Plugin

Render runtime orchestration for the engine runtime path.

## Start Here

- Plugin entry: `engine/src/plugins/render/plugin.rs`
- Public domain surface: `engine/src/plugins/render/domain/mod.rs`
- Architecture docs index: `engine/src/plugins/render/docs/index.md`

## Subdomain Ownership

- `backend/`
  - Backend device/surface/format policy and `WgpuCtx`.
- `frame_graph/`
  - Graph ids/spec/builders/registry/runtime graph resources/executor interfaces/validation.
- `renderer/`
  - Per-frame lifecycle (`extract`, `prepare`, `graph_execution`, `render_flow`, `submit`).
- `shader/`
  - Shader registry/types/helpers/hot-reload entry.
- `pipelines/`
  - Pipeline keys, cache policy, specialization contracts.
- `resources/`
  - Render frame bindings and explicit texture/buffer/transient ownership.
- `sdf/`
  - SDF render integration path (extract/bindings/fields/raymarch/materials/debug views).
- `debug/`
  - Render inspection surfaces (overlays, texture inspector, timings, graph dumps).

## Guides

- Happy-path usage guide (local): [./docs/usage-guide.md](./docs/usage-guide.md)
- Happy-path usage guide (reference): [../../../docs/reference/plugins/render/usage-guide.md](../../../docs/reference/plugins/render/usage-guide.md)
- Docs index: [./docs/index.md](./docs/index.md)
- Architecture: [./docs/architecture.md](docs/target-architecture.md)
- Frame graph: [./docs/frame-graph.md](./docs/frame-graph.md)
- SDF rendering: [./docs/sdf-rendering.md](./docs/sdf-rendering.md)
- Debug views: [./docs/debug-views.md](./docs/debug-views.md)
