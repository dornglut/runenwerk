# Render Plugin

Render runtime orchestration for the engine runtime path.

## Start Here

- Plugin entry: `engine/src/plugins/render/plugin.rs`
- Public flow API: `engine/src/plugins/render/api/flow.rs`
- Public contribution API: `engine/src/plugins/render/composition/contribution.rs`
- Architecture docs index: `engine/src/plugins/render/docs/index.md`

## Subdomain Ownership

- `backend/`
  - Backend device/surface/format policy, pipeline cache, resource allocator, and compiled pass execution.
- `graph/`
  - Canonical flow graph, pass graph, resource graph, planning, merge, and validation.
- `renderer/`
  - Per-frame orchestration (`extract`, `frame_bindings`, `graph_execution`, `prepare`, `submit`).
- `shader/`
  - Shader registry/types/helpers/hot-reload entry.
- `pipelines/`
  - Pipeline keys, cache policy, specialization contracts.
- `resource/`
  - Render resource descriptors, imports, lifetime classes, and transient alias planning helpers.
- `composition/`
  - `RenderFlowContribution`, namespace validation, flow merge integration, and asset-fragment hot-reload foundations.
- `inspect/`
  - Render inspection surfaces (graph dump, resource/texture inspection, timing summaries).
- `api/`
  - Public authoring surface for `RenderFlow`, pass builders, IDs, and param projection bindings.
- `params/`
  - GPU parameter conversion traits and types (`GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`).

## Guides

- Render docs index (reference): [../../../docs/reference/plugins/render/index.md](../../../docs/reference/plugins/render/index.md)
- Render flow usage: [../../../docs/reference/plugins/render/render-flow-usage-guide.md](../../../docs/reference/plugins/render/render-flow-usage-guide.md)
- GPU params: [../../../docs/reference/plugins/render/gpu-params-guide.md](../../../docs/reference/plugins/render/gpu-params-guide.md)
- Flow contributions: [../../../docs/reference/plugins/render/render-flow-contributions.md](../../../docs/reference/plugins/render/render-flow-contributions.md)
- Roadmap (source of truth): [./docs/roadmap.md](./docs/roadmap.md)
