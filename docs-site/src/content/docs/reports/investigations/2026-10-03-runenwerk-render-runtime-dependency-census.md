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

## GPU primitives, resources, pipelines, and residency

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/gpu_primitives/compaction.rs` | `7846d1477465b41e33f84093ae251d0f756c1776` | `root-facade` | `runen_gpu` | — | 3 |
| `engine/src/plugins/render/gpu_primitives/counters.rs` | `d9a407413ea826d52a1f3f3c656b293bd853cfb3` | `root-facade` | `runen_gpu` | — | 6 |
| `engine/src/plugins/render/gpu_primitives/draw_args.rs` | `f505dde4f2398ab54062a6abc0268f6c40853fec` | `graph`, `root-facade` | `runen_gpu` | — | 6 |
| `engine/src/plugins/render/gpu_primitives/mod.rs` | `ef64a027c5fe7d0a0f24f89bf53221a05776ce60` | — | — | — | 5 |
| `engine/src/plugins/render/gpu_primitives/plan.rs` | `10712f9ba1b1484a793e14ea1568438174bcf3a3` | `root-facade` | `runen_gpu`, `naga` | — | 24 |
| `engine/src/plugins/render/gpu_primitives/scan.rs` | `6de141e4b0d885848680deb92bd798ccb4b70548` | `root-facade` | `runen_gpu` | — | 9 |
| `engine/src/plugins/render/resource/dynamic_target.rs` | `48770b1b1b4aef0a01505dce6c89d0a853ddfe4d` | — | — | — | 28 |
| `engine/src/plugins/render/resource/mod.rs` | `f2187e0748f9af2082dbaf13de7a97884b861481` | — | — | — | 6 |
| `engine/src/plugins/render/resource/transient.rs` | `bd0bb7bc35963bccbcac59815dbb9aeecbddc126` | — | `runen_gpu` | — | 8 |
| `engine/src/plugins/render/resource/usages.rs` | `4a5befbe46e2b790504c20e0e4b913aaa19737e2` | — | `runen_gpu` | — | 3 |
| `engine/src/plugins/render/pipelines/cache.rs` | `276a3a7274a37ba4ac0d7bce71681928e58c3891` | — | — | — | 4 |
| `engine/src/plugins/render/pipelines/flow_keys.rs` | `127cd86e94a7346eb3a26d259eef170a9e180412` | `root-facade` | `runen_gpu` | — | 15 |
| `engine/src/plugins/render/pipelines/mod.rs` | `f7d51c37b8a874c2767498ef5cd09a518aa4019a` | — | — | — | 2 |
| `engine/src/plugins/render/residency/handle.rs` | `bcfccb7b2d4827cdc0071e4915dc23f075fa655e` | — | — | — | 4 |
| `engine/src/plugins/render/residency/mod.rs` | `68b929ddc6eae6813f9160a4204445923de2b9dd` | — | — | — | 2 |
| `engine/src/plugins/render/residency/resource.rs` | `b0328f8444b4829fcebc18da93d3d1f99b14dd61` | `root-facade` | — | — | 17 |

## Graph and shader

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/graph/diagnostics.rs` | `fd1ecf3a57da19370b531d78b1e7a15c5661607d` | `root-facade` | `runen_gpu` | — | 25 |
| `engine/src/plugins/render/graph/execution_plan.rs` | `5e52c98e625d685af06ebb034da49d31875f5a3f` | `api`, `features`, `root-facade` | `runen_gpu` | — | 29 |
| `engine/src/plugins/render/graph/flow_graph.rs` | `56a0b39650e9e1636d42f16859cee027cfb1d4cf` | `root-facade` | — | — | 5 |
| `engine/src/plugins/render/graph/merge.rs` | `8cca40a6c574648b1d33f92adb66daa9d7392e96` | `api`, `composition`, `root-facade` | `runen_gpu` | — | 4 |
| `engine/src/plugins/render/graph/mod.rs` | `e1143b1d38bfc4eb4f586c4dc094a1000f125fbe` | — | — | — | 22 |
| `engine/src/plugins/render/graph/pass_graph.rs` | `595b57b953f09799010fd97b6ce923e6a17b90ec` | `api`, `root-facade` | `runen_gpu` | — | 48 |
| `engine/src/plugins/render/graph/pass_shape.rs` | `25b91af5a3533526d2b5f42377cc30b3e60d90f6` | — | — | — | 1 |
| `engine/src/plugins/render/graph/planning.rs` | `08243fae2992287b9cf993bd6a5b1c2b7ed72c7c` | `api`, `root-facade` | `runen_gpu` | — | 16 |
| `engine/src/plugins/render/graph/prepared_validation.rs` | `90b796eb59e49fbcf56dd13c8341a3b7a172f737` | `features`, `root-facade` | `runen_gpu` | — | 21 |
| `engine/src/plugins/render/graph/resource_graph.rs` | `8d871d416502bad683ddb2c95e7c5cc5820e6dd3` | `root-facade` | `runen_gpu` | — | 9 |
| `engine/src/plugins/render/graph/validation.rs` | `94ee37bb94fa85d0530bc4996c067f5e96db71ba` | `api`, `resource`, `root-facade` | `runen_gpu` | — | 4 |
| `engine/src/plugins/render/graph/validation_builtin_ui.rs` | `67fede64b82f28961a114f12106581ea946a0604` | — | — | — | 1 |
| `engine/src/plugins/render/shader/helpers.rs` | `0661e990c8de6bc56680c591ceb0ae10897ad216` | — | — | — | 7 |
| `engine/src/plugins/render/shader/hot_reload.rs` | `f5ebdf34915335903afe68f26319d81bd50363dc` | — | — | — | 1 |
| `engine/src/plugins/render/shader/mod.rs` | `dca94b63f3c3443aa0297403436dfe29037edb51` | — | — | `shared` | 3 |
| `engine/src/plugins/render/shader/registry.rs` | `d6d23227c0d925829b3a4f3c5ea8d3a26a2afd2a` | — | — | — | 31 |
| `engine/src/plugins/render/shader/types.rs` | `558cc9eb0868288ae8154bf5b4b61be247a14cde` | — | — | — | 13 |

## Material compiler

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/material_compiler/bindings.rs` | `b251fa5c6ba06ea18dcb89915ee795508ee4b837` | — | — | — | 11 |
| `engine/src/plugins/render/material_compiler/diagnostics.rs` | `5dd4ca45818d9fa279b0eedfcfe4b7a7f3537a0a` | — | — | — | 1 |
| `engine/src/plugins/render/material_compiler/identity.rs` | `58f1b54a20c3ae05b2c77bd7830958afcf7b58fd` | — | — | — | 6 |
| `engine/src/plugins/render/material_compiler/mod.rs` | `b753fc95718479ee338c8c212582c72faafe13c8` | — | — | — | 5 |
| `engine/src/plugins/render/material_compiler/tests.rs` | `977443d269e9a934bd25a9c7dd1fa41c5c9ff134` | — | — | — | 0 |
| `engine/src/plugins/render/material_compiler/types.rs` | `136e3eab2907025a9215306ad7bf0454ebadffaa` | — | — | — | 9 |
| `engine/src/plugins/render/material_compiler/validation.rs` | `e790fe52a36f7220f881c135d9c4532b0db4c99c` | — | `naga` | — | 3 |
| `engine/src/plugins/render/material_compiler/wgsl/literals.rs` | `8eb99c67827fac5f832faaf644a4635d661f2d13` | — | — | — | 4 |
| `engine/src/plugins/render/material_compiler/wgsl/mod.rs` | `3bef6a66c3a71a55fae415aaa0b96f5d7141774b` | — | — | — | 3 |
| `engine/src/plugins/render/material_compiler/wgsl/preview.rs` | `a7e61f6ece0aaee32abd6369fbefc3371fd8b6b0` | — | — | — | 1 |
| `engine/src/plugins/render/material_compiler/wgsl/program.rs` | `f9478fcfa7280214a849a149bd5a32e139545a85` | — | — | — | 3 |
| `engine/src/plugins/render/material_compiler/wgsl/scene.rs` | `3034a1c78dcb3e93c28709322013ec4899ffab49` | — | — | — | 3 |

## Procedural and runtime

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/procedural/authoring.rs` | `cfdae5c1767d2619a73ecbdc64c785a44e0ef6b4` | `api`, `root-facade` | `runen_gpu` | — | 11 |
| `engine/src/plugins/render/procedural/camera.rs` | `f2cdf173c9ae3dc40de1a2fb01e8186e5ca1a09b` | `root-facade` | — | — | 18 |
| `engine/src/plugins/render/procedural/descriptors.rs` | `666ae867e576eba8ae4fae52b2c2b3da188f902a` | `root-facade` | `runen_gpu` | — | 36 |
| `engine/src/plugins/render/procedural/lowering.rs` | `3d1966a49d9ecf6f3e080943e9872f71958fb9ee` | `api`, `root-facade` | `runen_gpu` | — | 4 |
| `engine/src/plugins/render/procedural/mod.rs` | `3404b0f84e160cbc64a50e67aa3e67f3dd810824` | — | — | — | 7 |
| `engine/src/plugins/render/procedural/population/mod.rs` | `422f3c5f2674b19c486a919ab4a94266b7421baa` | — | — | — | 1 |
| `engine/src/plugins/render/procedural/population/uniform_grid.rs` | `b47561c3acb68861f72ac2e469b2eff32cd4c55b` | `gpu_primitives`, `root-facade` | `runen_gpu` | — | 14 |
| `engine/src/plugins/render/procedural/validation.rs` | `2f5c55dce41b4b4cd086f115f1724ca712e2c4c3` | `root-facade` | — | — | 2 |
| `engine/src/plugins/render/runtime/debug_eval.rs` | `2e39a16906024732e5ab9e4f1b3e64c01e592139` | `frame` | — | `inspect` | 3 |
| `engine/src/plugins/render/runtime/dynamic_targets.rs` | `cf447bf763f455c172dc2136223818c85cd7a127` | `backend`, `root-facade` | — | — | 11 |
| `engine/src/plugins/render/runtime/dynamic_texture_uploads.rs` | `20d664f6975f8b09bbceb9389cc2e2e554bc2363` | `root-facade` | — | — | 14 |
| `engine/src/plugins/render/runtime/frame_diagnostics.rs` | `56bccabb9cd058ca7c8a53bbc3087ad2d9f6faca` | `inspect`, `pipelines` | — | — | 5 |
| `engine/src/plugins/render/runtime/frame_prepare.rs` | `884b8079dc330f869c4cffb08b2b480e548e6fb9` | `backend`, `inspect`, `root-facade` | `runen_gpu` | `scene` | 3 |
| `engine/src/plugins/render/runtime/frame_submit.rs` | `2a546b351cd2ef6b3dc657f2e86a29f81cccb770` | `backend`, `inspect`, `renderer`, `root-facade` | `runen_render` | `inspect`, `pipelines`, `time` | 1 |
| `engine/src/plugins/render/runtime/mod.rs` | `61a57613c929f77422b94e2da1b87c1fe584cf06` | — | — | — | 11 |

## Inspect — part 1

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/inspect/artifacts.rs` | `821da159412b7461e7590f2c4cc21b951a32f4fb` | — | — | — | 5 |
| `engine/src/plugins/render/inspect/budgets.rs` | `e12b3ac7015ed18aa503d387a7727a2bd4e59ff8` | — | — | — | 14 |
| `engine/src/plugins/render/inspect/capture.rs` | `b1860af952bfc6fec51f27bbf1ff3084f57269e9` | — | — | — | 30 |
| `engine/src/plugins/render/inspect/config.rs` | `b70a9967f57a9cef1132a54c263fafe248e23f67` | `api` | — | — | 24 |
| `engine/src/plugins/render/inspect/frame_history.rs` | `bf7d1129b29235d8a72ee3221ddf6d38eda00f5d` | `renderer` | — | — | 25 |
| `engine/src/plugins/render/inspect/gpu_residency.rs` | `b417dbac892b14978b26ba134d4cfcc9f3b29294` | `root-facade` | — | — | 5 |
| `engine/src/plugins/render/inspect/graph_dump.rs` | `7f26da8be94ba7042bb7f991fed666d4bd1f6656` | `root-facade` | — | — | 5 |
| `engine/src/plugins/render/inspect/material_handoff.rs` | `951201cb1ca0a557e570fb7e990646a9a94af155` | `root-facade` | — | — | 7 |
| `engine/src/plugins/render/inspect/material_production.rs` | `ab35b8e0dc8607f39caafd2a6638558b4b92c2c5` | — | — | — | 16 |
| `engine/src/plugins/render/inspect/mod.rs` | `1f21df7514b080b1be63c4f3baad4d72fb65a361` | — | — | — | 32 |
| `engine/src/plugins/render/inspect/pass_provenance.rs` | `cd4067f9240a408666dd6488ea73b2cc5898e5fd` | `pipelines`, `root-facade` | `runen_gpu` | — | 7 |
| `engine/src/plugins/render/inspect/pipeline_fallback.rs` | `8845cf0d593881820805647eeb6c8de0c6137fd6` | `pipelines`, `shader` | — | — | 9 |
| `engine/src/plugins/render/inspect/plan.rs` | `a7f405e834e869c95d5dbc3bfaea5046f3f91dc6` | `graph`, `root-facade` | `runen_gpu` | — | 14 |
| `engine/src/plugins/render/inspect/prepared_frame.rs` | `632ade45a4a898072cdc1c7cefde5a63669013a0` | `root-facade` | — | — | 19 |
| `engine/src/plugins/render/inspect/producer.rs` | `81b15ac873d28d27af1101d2f770f1c18fc5191d` | — | — | `diagnostics` | 7 |
| `engine/src/plugins/render/inspect/product_visual_evidence.rs` | `cee6182ce2ae5da2e4238faf4f1d202cb4f2884e` | `features`, `root-facade` | — | — | 26 |
| `engine/src/plugins/render/inspect/query_snapshot.rs` | `a651f0fd8338573cc1c57b28837545db4bb03385` | — | — | — | 3 |

## Inspect — part 2

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/inspect/ray_query.rs` | `4fc6b43f9203620f3ed0460bc28d7f3c58e85d85` | — | — | — | 22 |
| `engine/src/plugins/render/inspect/readiness.rs` | `ed4c39679a8d49a20b95eff83e5fe47da1498afa` | — | — | — | 22 |
| `engine/src/plugins/render/inspect/report.rs` | `e5ebdf31822b49917cca4d651e31ff63cc1ce95c` | — | — | — | 13 |
| `engine/src/plugins/render/inspect/resource_inspector.rs` | `d848f185004877629ad8666d9e0dd7b95d01ce69` | `root-facade` | `runen_gpu` | — | 11 |
| `engine/src/plugins/render/inspect/scale_production.rs` | `9fadf23105f98434595b9b371e4f04cb92483d91` | — | — | — | 16 |
| `engine/src/plugins/render/inspect/scale_visibility.rs` | `994cafa4f9923b4b44505ae24a3cb0f982f7f1c3` | — | — | — | 15 |
| `engine/src/plugins/render/inspect/sdf_production.rs` | `44f816547e0bcc1721a9f563191e7a47ae2c0f8a` | `features` | — | — | 16 |
| `engine/src/plugins/render/inspect/sdf_raymarch.rs` | `dbf75d06cafc9ad9019eeaeb5b33b2c2326267f2` | `features` | — | — | 2 |
| `engine/src/plugins/render/inspect/sdf_residency.rs` | `6bb333e710744923e247074447d683469fdd629c` | `features` | — | — | 8 |
| `engine/src/plugins/render/inspect/temporal.rs` | `bac29efece842d5ffa704aeb601507e72c1fc837` | `root-facade` | — | — | 23 |
| `engine/src/plugins/render/inspect/temporal_production.rs` | `eb832b872c2ca4bcac18a94198da2d271db3e4d8` | — | — | — | 16 |
| `engine/src/plugins/render/inspect/temporal_upscaling.rs` | `c2c2363001c072eaae1252b8dadc8837e47ffd79` | — | — | — | 15 |
| `engine/src/plugins/render/inspect/texture_preview.rs` | `a39ebb053d36df8ff3cd148d24710dccdc9bc0ab` | `root-facade` | `runen_gpu` | — | 6 |
| `engine/src/plugins/render/inspect/texture_view.rs` | `448096d4ca9dd1fb9abaf25b49dc85ba6aa3d229` | `root-facade` | `runen_gpu` | — | 5 |
| `engine/src/plugins/render/inspect/timings.rs` | `61d2c8a73f416d11492c4563fd763c78ab110a27` | `graph`, `renderer`, `shader` | — | — | 38 |
| `engine/src/plugins/render/inspect/world_runtime.rs` | `690810473b069f5c85a52b429cbc2ec848004867` | — | — | — | 1 |

## Renderer — part 1

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/renderer/dynamic_targets.rs` | `b70d255704c947efa1058786b97142864356f48b` | `root-facade` | `runen_gpu` | — | 17 |
| `engine/src/plugins/render/renderer/extract.rs` | `49c3604c406f144e3b59cfb242f915496fb2954d` | `features` | — | — | 5 |
| `engine/src/plugins/render/renderer/frame_bindings.rs` | `57554d5be74d5b181e16bd716c33af972693d778` | — | — | — | 8 |
| `engine/src/plugins/render/renderer/mod.rs` | `4c2fcc9dd44ad33b0f2d5737e478110c553aadf4` | `backend`, `features`, `frame`, `graph`, `inspect`, `root-facade`, `shader` | `runen_render`, `runen_gpu` | — | 29 |
| `engine/src/plugins/render/renderer/pipeline_cache.rs` | `93c253953c1701aab1e37f73bc7c58123f0734ff` | `pipelines`, `root-facade` | `runen_gpu` | — | 8 |
| `engine/src/plugins/render/renderer/prepare.rs` | `ca2528dd85eb563cc587bd0355a78395f452f097` | `features`, `root-facade` | `runen_gpu` | — | 4 |
| `engine/src/plugins/render/renderer/render_flow/bindings.rs` | `3ce8f0be21a5bacb2572e1fd52b28504e1b342cc` | `pipelines`, `root-facade` | `runen_gpu` | — | 2 |
| `engine/src/plugins/render/renderer/render_flow/canonical_work.rs` | `21fb4565a654a61d91e65dde24175eac97e86654` | `root-facade` | `runen_gpu` | — | 10 |
| `engine/src/plugins/render/renderer/render_flow/capture.rs` | `52b64bf51ceeb32be36a6db9879d019f3f9711d0` | `inspect` | `runen_gpu` | — | 18 |
| `engine/src/plugins/render/renderer/render_flow/execute.rs` | `081651d2ffecfb47f225264a64eae425b9472ed4` | `graph`, `root-facade` | `runen_render`, `runen_gpu` | — | 5 |
| `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs` | `e1d1d05637d84ed3615c436c98616bc562feb821` | `adapters`, `root-facade` | `runen_gpu` | — | 19 |
| `engine/src/plugins/render/renderer/render_flow/logical_copy.rs` | `4b1ad459c6178e0ceacd8f7142fe7082c3d20c6b` | `root-facade` | `runen_gpu` | — | 3 |
| `engine/src/plugins/render/renderer/render_flow/logical_operations.rs` | `bdcb7202c3083688459b37f40a72f1d2cddf85c6` | `graph`, `root-facade` | `runen_gpu` | — | 8 |
| `engine/src/plugins/render/renderer/render_flow/logical_timing.rs` | `11600fe0b303488ad626a388b1117c238d54c611` | — | `runen_gpu` | — | 10 |

## Renderer — part 2

| File | Blob | Render deps | Framework deps | Peer deps | Public items |
| --- | --- | --- | --- | --- | ---: |
| `engine/src/plugins/render/renderer/render_flow/mod.rs` | `27270899b298a4272cfca21651194788f24c5574` | `api`, `backend`, `frame`, `graph`, `inspect`, `pipelines`, `root-facade` | `runen_gpu` | — | 10 |
| `engine/src/plugins/render/renderer/render_flow/observation.rs` | `40fe96320f15e3932666d7dbfdad04c0a0414eb2` | — | `runen_gpu` | — | 4 |
| `engine/src/plugins/render/renderer/render_flow/occurrences.rs` | `008c39146112a91365eb06afd4f716fe5aeb1396` | `root-facade` | `runen_gpu` | — | 3 |
| `engine/src/plugins/render/renderer/render_flow/pipeline_realization.rs` | `40f52e8144c846e3aadfadcd20d7fa45dbe7907f` | `pipelines`, `root-facade` | `runen_gpu` | — | 1 |
| `engine/src/plugins/render/renderer/render_flow/preflight_cache.rs` | `a6fc3338f36f8966de66f611a5e44b785421e899` | `graph`, `root-facade` | — | — | 2 |
| `engine/src/plugins/render/renderer/render_flow/program_sources.rs` | `042e5708a6dcab347b2df9e1cf2c4d6f816e1303` | — | `runen_gpu` | — | 7 |
| `engine/src/plugins/render/renderer/render_flow/provenance.rs` | `e71f9217f6f4495db48edee2bd16e399e6b16a27` | `features`, `root-facade` | — | — | 18 |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs` | `3b72f3defffd83a98fcb09bae5dd805a83038211` | `root-facade` | `runen_gpu` | — | 11 |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/inspect.rs` | `fb756eab9b222a3bff67823d475ee8abd1a11095` | — | — | — | 1 |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/realize.rs` | `51dca0a726acf28976a8aaa21dc26a1cfa28ac37` | `root-facade` | `runen_gpu` | — | 8 |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs` | `74fdc58680ee1eedc9feb131d7ade3bfc068a7f2` | — | `runen_gpu` | — | 13 |
| `engine/src/plugins/render/renderer/resource_descriptors.rs` | `70516ff5e65e9835d72844057c05b3b5cb0ff150` | — | `runen_gpu` | — | 7 |
| `engine/src/plugins/render/renderer/setup.rs` | `3efbab08b81c76ea1125d5ca559ecb2f87cd5520` | `features`, `graph`, `inspect`, `root-facade` | `runen_render`, `runen_gpu` | — | 33 |
