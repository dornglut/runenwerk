---
title: Runenwerk Render Runtime Lexical Dependency Census
description: Immutable accepted-revision source dependency appendix for investigation #1138, recording explicit render-group, framework, peer-plugin, and public-surface references per central Render Rust file.
status: active
owner: engine
layer: investigation
canonical: false
last_reviewed: 2026-10-03
publication: reference
pagefind: false
related_docs:
  - ./2026-10-03-runenwerk-render-runtime-decomposition.md
  - ./2026-10-03-runenwerk-render-runtime-source-census.md
---

# Runenwerk Render Runtime Lexical Dependency Census

This appendix closes the exact dependency-census requirement for investigation #1138.

```text
RUNENWERK_REVISION=8d0bad6c7e4c24f2579bd89c3f797946af85dba3
PATH_PREFIX=engine/src/plugins/render/
```

Every row below is derived from the exact file bytes fetched at that immutable revision. The
dependency columns are an **exact lexical source-reference census**, not a compiler-resolved Rust
type graph:

- **Render deps** records explicit cross-top-level Render module references, plus `root-facade`
  when a file imports re-exported `crate::plugins::render::{...}` / root symbols whose owning
  submodule is intentionally hidden by the facade.
- **Framework deps** records direct `runen_render`, `runen_gpu`, `runen_shader`, `wgpu`,
  and `naga` source references.
- **Peer deps** records direct references to other `crate::plugins::<peer>` owners.
- **Public items** is the count of lexically declared `pub` / `pub(...)` structs, enums,
  traits, functions, aliases, constants, statics, modules, and uses in the file. It is a surface
  pressure indicator, not a claim that every item is externally reachable after module visibility.

Relative `super::...` paths are resolved against the file's module path before assigning a
top-level Render dependency. Same-top-level-module references are omitted from **Render deps**.


## Root, adapters, and params

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/app_ext.rs` | `8fe36a7dbb2a164c8cbdf617f127592bcbca2ecf` | `inspect`, `root-facade` | — | — | 1 |
| `engine/src/plugins/render/gpu_context_policy.rs` | `124f212ed4233bcd1b3efa53a40e73b8c7a56ce4` | — | `runen_gpu` | — | 2 |
| `engine/src/plugins/render/mod.rs` | `2e673aaae7108834518ee930b77e96314bb89dde` | — | — | — | 42 |
| `engine/src/plugins/render/native_host.rs` | `f9adc551e92faaa9eb034cd32822f888beaa2bc7` | `backend`, `renderer` | — | — | 5 |
| `engine/src/plugins/render/plugin.rs` | `b850ab0417e85ab967c717684b2104c03541718b` | `backend`, `composition`, `features`, `frame`, `inspect`, `pipelines`, `residency`, `runtime`, `shader` | — | `scene`, `ui` | 3 |
| `engine/src/plugins/render/readiness.rs` | `c2257ef4c864fd9a9135d9ca690b6472be87c53b` | — | — | — | 9 |
| `engine/src/plugins/render/texture_upload.rs` | `b0a0221a7974c0c86cbd54d7fb7d2e626a0a6d3c` | `root-facade` | `runen_gpu` | — | 2 |
| `engine/src/plugins/render/adapters/gpu_capabilities.rs` | `7bd8b8e4ec45acc85e863a3070257fb4c0acd663` | `graph`, `root-facade` | `runen_gpu` | — | 6 |
| `engine/src/plugins/render/adapters/gpu_data.rs` | `25ac5ec41d973733066e1ffb4df8d1a0a33cdaac` | `root-facade` | `runen_gpu` | — | 11 |
| `engine/src/plugins/render/adapters/gpu_resources.rs` | `2fa8106dad686ca9d1409064fddf19986ca224e3` | `root-facade` | `runen_gpu` | — | 84 |
| `engine/src/plugins/render/adapters/gpu_work.rs` | `5d211514155119d9d21a6bb43775b29fef288e5f` | — | `runen_gpu` | — | 15 |
| `engine/src/plugins/render/adapters/mod.rs` | `f181b2da9f78c99100d9972d24c54c30ed7a2aca` | — | — | — | 4 |
| `engine/src/plugins/render/params/gpu_params.rs` | `bd1a983cc36555eadc32563cada1472ccb94d805` | — | — | — | 3 |
| `engine/src/plugins/render/params/gpu_value.rs` | `c0c79a339bedd9492dc54ecff30980787567e09e` | — | — | — | 5 |
| `engine/src/plugins/render/params/mod.rs` | `f8bb7609fa145b0b8c051107d495c244bf2b4f5c` | — | — | — | 4 |

## API and backend

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/api/bindings.rs` | `d9f53d542c28b461013149d8d844022e35bbd6b0` | `graph`, `renderer`, `root-facade` | `runen_gpu` | — | 31 |
| `engine/src/plugins/render/api/dispatch.rs` | `e52c799a076d3ccd1382d270ef3d60e80f3ba28d` | — | — | — | 7 |
| `engine/src/plugins/render/api/errors.rs` | `e42ee224b71d6843a6a7bf7341d9b3163f44848a` | `gpu_primitives`, `procedural`, `root-facade` | `runen_gpu` | — | 1 |
| `engine/src/plugins/render/api/flow.rs` | `33fb10c1564f92c2ef44d60a22b9183134142e1f` | `graph`, `procedural`, `renderer`, `root-facade` | `runen_gpu` | — | 53 |
| `engine/src/plugins/render/api/handles.rs` | `c58bb8bce85edc499ded43bf4da527a600dc6747` | `root-facade` | `runen_gpu` | — | 8 |
| `engine/src/plugins/render/api/ids.rs` | `5ce8fd5a703a9abe85bf8e378b847ecec6cab1cb` | — | — | — | 4 |
| `engine/src/plugins/render/api/mod.rs` | `07f26158f6bbb465665848d5ab456d2e4334c37b` | — | — | — | 14 |
| `engine/src/plugins/render/api/passes.rs` | `323b863fbed0a71adc41133d2c66fda5e26e27cf` | `graph`, `root-facade` | `runen_gpu` | — | 101 |
| `engine/src/plugins/render/backend/execution.rs` | `c699aab724572667df4da77f20badcf899cadaea` | `graph` | — | — | 1 |
| `engine/src/plugins/render/backend/formats.rs` | `17b885be5901d1eecc2ef7c2f2ac6ebe04613fc6` | — | `runen_gpu` | — | 1 |
| `engine/src/plugins/render/backend/mod.rs` | `1a18b45bd9b54c8ac3db9129e918461c2d647b40` | — | — | — | 6 |
| `engine/src/plugins/render/backend/pipeline_cache.rs` | `de09f631b8ff381a235f96d5bb089a08318f2475` | `pipelines` | — | — | 2 |
| `engine/src/plugins/render/backend/resource_allocator.rs` | `f2745f426d3a44a8452a1aaea73ab1e91214fe14` | `root-facade` | `runen_gpu` | — | 16 |
| `engine/src/plugins/render/backend/surface.rs` | `8bdf7704022049943d5d9da9c738849329ed4ffd` | — | `runen_gpu` | — | 18 |
| `engine/src/plugins/render/backend/wgpu_ctx.rs` | `2e54a0e741d09ce9546f73e52ae0ccd9dc05b8ce` | `root-facade` | `runen_gpu` | — | 10 |

## Composition and frame

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/composition/fragment_registry.rs` | `2c4ba88ba5da2f133326bb8d938bd36cdb642bad` | `graph`, `root-facade` | `runen_gpu` | — | 14 |
| `engine/src/plugins/render/composition/fragment_validation.rs` | `915e45ddae6fd9af26fbb2b612503f9be97ac520` | — | `runen_gpu` | — | 1 |
| `engine/src/plugins/render/composition/fragments.rs` | `d065b8b85edafb05eda6b997614b6212d9991c96` | `root-facade` | `runen_gpu` | — | 88 |
| `engine/src/plugins/render/composition/hot_reload.rs` | `fb1f44ee9f7dd8b9db9696dfcdf0dc4b012aa1cf` | — | — | — | 3 |
| `engine/src/plugins/render/composition/integration.rs` | `f25c68cfad836fe1727639b9afe3239e3e0067c3` | `api`, `graph`, `root-facade` | — | — | 9 |
| `engine/src/plugins/render/composition/mod.rs` | `4b05b7ee13b039d7074733fa1c68462634572f69` | — | — | — | 11 |
| `engine/src/plugins/render/frame/context.rs` | `06c635b0e4719311108ebf571e19a56bde05a19c` | — | — | — | 1 |
| `engine/src/plugins/render/frame/contribution_diagnostics.rs` | `7d810a1578e2d60cb8146daa25a963b427db9a85` | `api`, `features`, `root-facade` | — | — | 8 |
| `engine/src/plugins/render/frame/contribution_registry.rs` | `235fb55b6166257f111d8632c673a39242f427b1` | `api`, `features`, `root-facade` | — | — | 37 |
| `engine/src/plugins/render/frame/contributions.rs` | `788cafa021ea2fb99219a9e2603cacea066dbffc` | `api`, `features` | — | — | 92 |
| `engine/src/plugins/render/frame/fixed_resolution.rs` | `8cd3f120e5698daf7eb6a008298046b5df3ef5f8` | `backend`, `root-facade` | `runen_gpu` | — | 16 |
| `engine/src/plugins/render/frame/mod.rs` | `978afe9d5f67ef25c663aac728f79888e80979f8` | `features` | — | — | 19 |
| `engine/src/plugins/render/frame/packet.rs` | `91eb2580f44bff5d94b4f70f278ec10072fd7e39` | `backend`, `root-facade` | `runen_render`, `runen_gpu` | — | 70 |
| `engine/src/plugins/render/frame/product_selection.rs` | `74bc215ba503d768ec4649803f35666dae68f3c1` | `root-facade` | — | — | 9 |
| `engine/src/plugins/render/frame/product_surface.rs` | `69d18b4250b2f26bd6adb15b25a55ac5463e04e8` | `root-facade` | — | — | 64 |
| `engine/src/plugins/render/frame/view.rs` | `5ed3103a3ae35cde8e54d5bb785229f10a7652c6` | — | — | — | 6 |

## Features

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/features/caves/mod.rs` | `f8a8c2c98c89f000c83c3b0d767843a7166fe44c` | — | — | — | 1 |
| `engine/src/plugins/render/features/detail/mod.rs` | `37ee3e53bdbbf27c82e5550a4431373bd739261a` | — | — | — | 2 |
| `engine/src/plugins/render/features/editor_picking/mod.rs` | `ea4696c18aa00ab2c0b1c51d386efeac4d5d8e79` | — | — | — | 1 |
| `engine/src/plugins/render/features/editor_picking/resource.rs` | `7da2305af55d3950f37560e4e006c2223844bdf0` | — | — | — | 9 |
| `engine/src/plugins/render/features/mod.rs` | `855bc0559c499616e4fc052764e2397be8cc2ca8` | `api`, `frame` | — | — | 60 |
| `engine/src/plugins/render/features/particle_vfx/mod.rs` | `562fabc1a0ee8b4631bb0c2560fefaf67cdaa310` | `root-facade` | — | — | 32 |
| `engine/src/plugins/render/features/ui/descriptor.rs` | `5a3cd8469e83a8bc992d5f5f5d0a3205154627c2` | — | — | — | 2 |
| `engine/src/plugins/render/features/ui/mod.rs` | `458bdd8d729bde4f2b875c98b2169b3bb6a94bb7` | — | — | — | 10 |
| `engine/src/plugins/render/features/ui/prepared.rs` | `04607c0f9c6dea33b0c10b176360f8bd833224bd` | — | — | — | 6 |
| `engine/src/plugins/render/features/ui/render_output_proof.rs` | `5561b9a0ae6c09e1381581746c8e647bbf4e0ad9` | `api` | — | — | 4 |
| `engine/src/plugins/render/features/ui/resource.rs` | `2d410ae44b8964080fe2a2ffc40052e284506821` | `backend` | — | — | 13 |
| `engine/src/plugins/render/features/ui/submission.rs` | `95fdd837065f1ebb4bd70a367fcf66d7b45c8f75` | `api`, `backend` | — | — | 26 |
| `engine/src/plugins/render/features/world/lod.rs` | `883c5ea708a9390ebab916e8bc80db44b4c34a57` | — | — | — | 4 |
| `engine/src/plugins/render/features/world/mod.rs` | `d145b086a11a82e3b3e6a83b00616dd034edee40` | — | — | — | 10 |
| `engine/src/plugins/render/features/world/runtime_cache.rs` | `5ce49f6bd277bdfd97ccd6f4dd996aeaee3216d7` | `root-facade` | — | — | 4 |
| `engine/src/plugins/render/features/world/sdf_raymarch.rs` | `73ef3e05efcb131c299dd1a19bf64103c5da4be6` | — | — | — | 17 |
| `engine/src/plugins/render/features/world/sdf_residency.rs` | `4f23b698c1ae975b7101f013c44a9447368bb6fd` | — | — | — | 25 |
| `engine/src/plugins/render/features/world/visuals/mod.rs` | `e0329f47e0f2b2ebb252e5cda20e8ca910181fb4` | `root-facade` | — | — | 31 |
