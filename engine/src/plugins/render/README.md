# Render Plugin

Render runtime orchestration for the engine runtime path.

## Start Here

- Plugin entry: `engine/src/plugins/render/plugin.rs`
- Public flow API: `engine/src/plugins/render/api/flow.rs`
- Public contribution API: `engine/src/plugins/render/composition/contribution.rs`
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
- `resource/`
  - Render resource descriptors, imports, lifetime classes, and transient alias planning helpers.
- `composition/`
  - `RenderFlowContribution`, namespace validation, flow merge integration, and asset-fragment hot-reload foundations.
- `sdf/`
  - SDF render integration path (extract/bindings/fields/raymarch/materials/debug views).
- `debug/`
  - Render inspection surfaces (overlays, texture inspector, timings, graph dumps).

## Guides

- Render docs index (reference): [../../../docs/reference/plugins/render/index.md](../../../docs/reference/plugins/render/index.md)
- Render flow usage: [../../../docs/reference/plugins/render/render-flow-usage-guide.md](../../../docs/reference/plugins/render/render-flow-usage-guide.md)
- GPU params: [../../../docs/reference/plugins/render/gpu-params-guide.md](../../../docs/reference/plugins/render/gpu-params-guide.md)
- Flow contributions: [../../../docs/reference/plugins/render/render-flow-contributions.md](../../../docs/reference/plugins/render/render-flow-contributions.md)
- Target architecture: [./docs/target-architecture.md](./docs/target-architecture.md)
- Target roadmap: [./docs/target-architecture-roadmap.md](./docs/target-architecture-roadmap.md)
