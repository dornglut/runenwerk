---
title: "Render Plugin"
description: "Documentation for Render Plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-05-16
---

# Render Plugin

Render runtime orchestration for the engine runtime path.

## Start Here

- Plugin entry: `engine/src/plugins/render/plugin.rs`
- Public flow API: `engine/src/plugins/render/api/flow.rs`
- Architecture/docs index: `engine/src/plugins/render/docs/index.md`

## Subdomain Ownership

- `backend/`
  - Backend device/surface/format policy, pipeline cache, resource allocator, and compiled pass execution.
- `graph/`
  - Canonical flow graph, pass graph, resource graph, planning, and validation.
- `frame/`
  - Neutral prepare/submit boundary packet types (`PreparedRenderFrame`, context/views/feature contributions).
- `features/`
  - Render feature registry, dependency ordering, and contribution fallback policies.
- `renderer/`
  - Per-frame orchestration and execution (`extract`, `frame_bindings`, `prepare`, `submit`).
- `shader/`
  - Shader registry/types/helpers/hot-reload entry.
- `pipelines/`
  - Pipeline keys, cache policy, specialization contracts.
- `resource/`
  - Render resource descriptors, imports, lifetime classes, and transient alias planning helpers.
- `composition/`
  - Direct flow registry integration and runtime compilation wiring.
- `inspect/`
  - Render inspection surfaces (graph dump, resource/texture inspection, prepared-frame product surface inspection, timing summaries).
- `api/`
  - Public authoring surface for `RenderFlow`, pass builders, typed handles, and param projection bindings.
- `params/`
  - GPU parameter conversion traits and types (`GpuParams`, `GpuUniform`, `GpuStorage`, `ToGpuValue`).

## Guides

- Render docs index (reference): [../../../docs/reference/plugins/render/index.md](../../reference/plugins/render/index.md)
- Render flow usage: [../../../docs/reference/plugins/render/render-flow-usage-guide.md](../../reference/plugins/render/render-flow-usage-guide.md)
- GPU params: [../../../docs/reference/plugins/render/gpu-params-guide.md](../../reference/plugins/render/gpu-params-guide.md)
- Public API reference: [../../../docs/reference/plugins/render/public-api-reference.md](../../reference/plugins/render/public-api-reference.md)
- Render target architecture: [../../../docs/reference/plugins/render/render-target-architecture.md](../../reference/plugins/render/render-target-architecture.md)
- Fully featured renderer roadmap: [../../../docs/roadmaps/fully-featured-renderer-roadmap.md](../../roadmaps/fully-featured-renderer-roadmap.md)
- Final migration roadmap: [../../../docs/roadmaps/render-final-architecture-migration.md](../../roadmaps/render-final-architecture-migration.md)
