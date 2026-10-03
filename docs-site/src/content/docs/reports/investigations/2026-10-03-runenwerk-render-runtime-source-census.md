---
title: Runenwerk Render Runtime Exact Source Census
description: Immutable Git-tree census appendix for the retained Runenwerk render runtime reviewed by investigation #1138.
status: active
owner: engine
layer: investigation
canonical: false
last_reviewed: 2026-10-03
publication: reference
pagefind: false
related_docs:
  - ./2026-10-03-runenwerk-render-runtime-decomposition.md
---

# Runenwerk Render Runtime Exact Source Census

This appendix records the exact central Rust source inventory used by investigation #1138.

```text
RUNENWERK_REVISION=8d0bad6c7e4c24f2579bd89c3f797946af85dba3
PATH_PREFIX=engine/src/plugins/render/
RUST_FILE_COUNT=184
RUST_SOURCE_BYTES=2442976
```

The inventory comes from the immutable recursive Git tree for the revision above. It is a physical
source census, not an ownership decision by itself. The companion investigation classifies the
architectural responsibility and dependency direction of these groups.

## (root)

Files: 7; bytes: 32220.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/app_ext.rs` | 4462 | `8fe36a7dbb2a164c8cbdf617f127592bcbca2ecf` |
| `engine/src/plugins/render/gpu_context_policy.rs` | 1787 | `124f212ed4233bcd1b3efa53a40e73b8c7a56ce4` |
| `engine/src/plugins/render/mod.rs` | 1346 | `2e673aaae7108834518ee930b77e96314bb89dde` |
| `engine/src/plugins/render/native_host.rs` | 8911 | `f9adc551e92faaa9eb034cd32822f888beaa2bc7` |
| `engine/src/plugins/render/plugin.rs` | 8678 | `b850ab0417e85ab967c717684b2104c03541718b` |
| `engine/src/plugins/render/readiness.rs` | 2129 | `c2257ef4c864fd9a9135d9ca690b6472be87c53b` |
| `engine/src/plugins/render/texture_upload.rs` | 4907 | `b0a0221a7974c0c86cbd54d7fb7d2e626a0a6d3c` |

## adapters

Files: 5; bytes: 133184.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/adapters/gpu_capabilities.rs` | 21168 | `7bd8b8e4ec45acc85e863a3070257fb4c0acd663` |
| `engine/src/plugins/render/adapters/gpu_data.rs` | 8640 | `25ac5ec41d973733066e1ffb4df8d1a0a33cdaac` |
| `engine/src/plugins/render/adapters/gpu_resources.rs` | 28212 | `2fa8106dad686ca9d1409064fddf19986ca224e3` |
| `engine/src/plugins/render/adapters/gpu_work.rs` | 74773 | `5d211514155119d9d21a6bb43775b29fef288e5f` |
| `engine/src/plugins/render/adapters/mod.rs` | 391 | `f181b2da9f78c99100d9972d24c54c30ed7a2aca` |

## api

Files: 8; bytes: 125266.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/api/bindings.rs` | 16173 | `d9f53d542c28b461013149d8d844022e35bbd6b0` |
| `engine/src/plugins/render/api/dispatch.rs` | 1918 | `e52c799a076d3ccd1382d270ef3d60e80f3ba28d` |
| `engine/src/plugins/render/api/errors.rs` | 1355 | `e42ee224b71d6843a6a7bf7341d9b3163f44848a` |
| `engine/src/plugins/render/api/flow.rs` | 69622 | `33fb10c1564f92c2ef44d60a22b9183134142e1f` |
| `engine/src/plugins/render/api/handles.rs` | 1077 | `c58bb8bce85edc499ded43bf4da527a600dc6747` |
| `engine/src/plugins/render/api/ids.rs` | 159 | `5ce8fd5a703a9abe85bf8e378b847ecec6cab1cb` |
| `engine/src/plugins/render/api/mod.rs` | 246 | `07f26158f6bbb465665848d5ab456d2e4334c37b` |
| `engine/src/plugins/render/api/passes.rs` | 34716 | `323b863fbed0a71adc41133d2c66fda5e26e27cf` |

## backend

Files: 7; bytes: 29982.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/backend/execution.rs` | 553 | `c699aab724572667df4da77f20badcf899cadaea` |
| `engine/src/plugins/render/backend/formats.rs` | 302 | `17b885be5901d1eecc2ef7c2f2ac6ebe04613fc6` |
| `engine/src/plugins/render/backend/mod.rs` | 241 | `1a18b45bd9b54c8ac3db9129e918461c2d647b40` |
| `engine/src/plugins/render/backend/pipeline_cache.rs` | 206 | `de09f631b8ff381a235f96d5bb089a08318f2475` |
| `engine/src/plugins/render/backend/resource_allocator.rs` | 2698 | `f2745f426d3a44a8452a1aaea73ab1e91214fe14` |
| `engine/src/plugins/render/backend/surface.rs` | 15977 | `8bdf7704022049943d5d9da9c738849329ed4ffd` |
| `engine/src/plugins/render/backend/wgpu_ctx.rs` | 10005 | `2e54a0e741d09ce9546f73e52ae0ccd9dc05b8ce` |

## composition

Files: 6; bytes: 46225.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/composition/fragment_registry.rs` | 6267 | `2c4ba88ba5da2f133326bb8d938bd36cdb642bad` |
| `engine/src/plugins/render/composition/fragment_validation.rs` | 16328 | `915e45ddae6fd9af26fbb2b612503f9be97ac520` |
| `engine/src/plugins/render/composition/fragments.rs` | 20308 | `d065b8b85edafb05eda6b997614b6212d9991c96` |
| `engine/src/plugins/render/composition/hot_reload.rs` | 604 | `fb1f44ee9f7dd8b9db9696dfcdf0dc4b012aa1cf` |
| `engine/src/plugins/render/composition/integration.rs` | 2383 | `f25c68cfad836fe1727639b9afe3239e3e0067c3` |
| `engine/src/plugins/render/composition/mod.rs` | 335 | `4b05b7ee13b039d7074733fa1c68462634572f69` |

## features

Files: 18; bytes: 133279.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/features/caves/mod.rs` | 231 | `f8a8c2c98c89f000c83c3b0d767843a7166fe44c` |
| `engine/src/plugins/render/features/detail/mod.rs` | 418 | `37ee3e53bdbbf27c82e5550a4431373bd739261a` |
| `engine/src/plugins/render/features/editor_picking/mod.rs` | 36 | `ea4696c18aa00ab2c0b1c51d386efeac4d5d8e79` |
| `engine/src/plugins/render/features/editor_picking/resource.rs` | 2412 | `7da2305af55d3950f37560e4e006c2223844bdf0` |
| `engine/src/plugins/render/features/mod.rs` | 19423 | `855bc0559c499616e4fc052764e2397be8cc2ca8` |
| `engine/src/plugins/render/features/particle_vfx/mod.rs` | 19341 | `562fabc1a0ee8b4631bb0c2560fefaf67cdaa310` |
| `engine/src/plugins/render/features/ui/descriptor.rs` | 502 | `5a3cd8469e83a8bc992d5f5f5d0a3205154627c2` |
| `engine/src/plugins/render/features/ui/mod.rs` | 226 | `458bdd8d729bde4f2b875c98b2169b3bb6a94bb7` |
| `engine/src/plugins/render/features/ui/prepared.rs` | 1305 | `04607c0f9c6dea33b0c10b176360f8bd833224bd` |
| `engine/src/plugins/render/features/ui/render_output_proof.rs` | 3396 | `5561b9a0ae6c09e1381581746c8e647bbf4e0ad9` |
| `engine/src/plugins/render/features/ui/resource.rs` | 12281 | `2d410ae44b8964080fe2a2ffc40052e284506821` |
| `engine/src/plugins/render/features/ui/submission.rs` | 10714 | `95fdd837065f1ebb4bd70a367fcf66d7b45c8f75` |
| `engine/src/plugins/render/features/world/lod.rs` | 1144 | `883c5ea708a9390ebab916e8bc80db44b4c34a57` |
| `engine/src/plugins/render/features/world/mod.rs` | 212 | `d145b086a11a82e3b3e6a83b00616dd034edee40` |
| `engine/src/plugins/render/features/world/runtime_cache.rs` | 1190 | `5ce49f6bd277bdfd97ccd6f4dd996aeaee3216d7` |
| `engine/src/plugins/render/features/world/sdf_raymarch.rs` | 11993 | `73ef3e05efcb131c299dd1a19bf64103c5da4be6` |
| `engine/src/plugins/render/features/world/sdf_residency.rs` | 27552 | `4f23b698c1ae975b7101f013c44a9447368bb6fd` |
| `engine/src/plugins/render/features/world/visuals/mod.rs` | 20903 | `e0329f47e0f2b2ebb252e5cda20e8ca910181fb4` |

## frame

Files: 10; bytes: 195955.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/frame/context.rs` | 205 | `06c635b0e4719311108ebf571e19a56bde05a19c` |
| `engine/src/plugins/render/frame/contribution_diagnostics.rs` | 2099 | `7d810a1578e2d60cb8146daa25a963b427db9a85` |
| `engine/src/plugins/render/frame/contribution_registry.rs` | 17991 | `235fb55b6166257f111d8632c673a39242f427b1` |
| `engine/src/plugins/render/frame/contributions.rs` | 58327 | `788cafa021ea2fb99219a9e2603cacea066dbffc` |
| `engine/src/plugins/render/frame/fixed_resolution.rs` | 35513 | `8cd3f120e5698daf7eb6a008298046b5df3ef5f8` |
| `engine/src/plugins/render/frame/mod.rs` | 570 | `978afe9d5f67ef25c663aac728f79888e80979f8` |
| `engine/src/plugins/render/frame/packet.rs` | 44517 | `91eb2580f44bff5d94b4f70f278ec10072fd7e39` |
| `engine/src/plugins/render/frame/product_selection.rs` | 10083 | `74bc215ba503d768ec4649803f35666dae68f3c1` |
| `engine/src/plugins/render/frame/product_surface.rs` | 25346 | `69d18b4250b2f26bd6adb15b25a55ac5463e04e8` |
| `engine/src/plugins/render/frame/view.rs` | 1304 | `5ed3103a3ae35cde8e54d5bb785229f10a7652c6` |

## gpu_primitives

Files: 6; bytes: 50217.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/gpu_primitives/compaction.rs` | 4525 | `7846d1477465b41e33f84093ae251d0f756c1776` |
| `engine/src/plugins/render/gpu_primitives/counters.rs` | 1925 | `d9a407413ea826d52a1f3f3c656b293bd853cfb3` |
| `engine/src/plugins/render/gpu_primitives/draw_args.rs` | 3755 | `f505dde4f2398ab54062a6abc0268f6c40853fec` |
| `engine/src/plugins/render/gpu_primitives/mod.rs` | 166 | `ef64a027c5fe7d0a0f24f89bf53221a05776ce60` |
| `engine/src/plugins/render/gpu_primitives/plan.rs` | 34225 | `10712f9ba1b1484a793e14ea1568438174bcf3a3` |
| `engine/src/plugins/render/gpu_primitives/scan.rs` | 5621 | `6de141e4b0d885848680deb92bd798ccb4b70548` |

## graph

Files: 12; bytes: 217188.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/graph/diagnostics.rs` | 6995 | `fd1ecf3a57da19370b531d78b1e7a15c5661607d` |
| `engine/src/plugins/render/graph/execution_plan.rs` | 31059 | `5e52c98e625d685af06ebb034da49d31875f5a3f` |
| `engine/src/plugins/render/graph/flow_graph.rs` | 1145 | `56a0b39650e9e1636d42f16859cee027cfb1d4cf` |
| `engine/src/plugins/render/graph/merge.rs` | 19949 | `8cca40a6c574648b1d33f92adb66daa9d7392e96` |
| `engine/src/plugins/render/graph/mod.rs` | 518 | `e1143b1d38bfc4eb4f586c4dc094a1000f125fbe` |
| `engine/src/plugins/render/graph/pass_graph.rs` | 14863 | `595b57b953f09799010fd97b6ce923e6a17b90ec` |
| `engine/src/plugins/render/graph/pass_shape.rs` | 5031 | `25b91af5a3533526d2b5f42377cc30b3e60d90f6` |
| `engine/src/plugins/render/graph/planning.rs` | 9942 | `08243fae2992287b9cf993bd6a5b1c2b7ed72c7c` |
| `engine/src/plugins/render/graph/prepared_validation.rs` | 53815 | `90b796eb59e49fbcf56dd13c8341a3b7a172f737` |
| `engine/src/plugins/render/graph/resource_graph.rs` | 2078 | `8d871d416502bad683ddb2c95e7c5cc5820e6dd3` |
| `engine/src/plugins/render/graph/validation_builtin_ui.rs` | 2564 | `67fede64b82f28961a114f12106581ea946a0604` |
| `engine/src/plugins/render/graph/validation.rs` | 69229 | `94ee37bb94fa85d0530bc4996c067f5e96db71ba` |

## inspect

Files: 33; bytes: 404334.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/inspect/artifacts.rs` | 9908 | `821da159412b7461e7590f2c4cc21b951a32f4fb` |
| `engine/src/plugins/render/inspect/budgets.rs` | 9914 | `e12b3ac7015ed18aa503d387a7727a2bd4e59ff8` |
| `engine/src/plugins/render/inspect/capture.rs` | 8839 | `b1860af952bfc6fec51f27bbf1ff3084f57269e9` |
| `engine/src/plugins/render/inspect/config.rs` | 8228 | `b70a9967f57a9cef1132a54c263fafe248e23f67` |
| `engine/src/plugins/render/inspect/frame_history.rs` | 29298 | `bf7d1129b29235d8a72ee3221ddf6d38eda00f5d` |
| `engine/src/plugins/render/inspect/gpu_residency.rs` | 5353 | `b417dbac892b14978b26ba134d4cfcc9f3b29294` |
| `engine/src/plugins/render/inspect/graph_dump.rs` | 3576 | `7f26da8be94ba7042bb7f991fed666d4bd1f6656` |
| `engine/src/plugins/render/inspect/material_handoff.rs` | 15833 | `951201cb1ca0a557e570fb7e990646a9a94af155` |
| `engine/src/plugins/render/inspect/material_production.rs` | 17968 | `ab35b8e0dc8607f39caafd2a6638558b4b92c2c5` |
| `engine/src/plugins/render/inspect/mod.rs` | 1409 | `1f21df7514b080b1be63c4f3baad4d72fb65a361` |
| `engine/src/plugins/render/inspect/pass_provenance.rs` | 4038 | `cd4067f9240a408666dd6488ea73b2cc5898e5fd` |
| `engine/src/plugins/render/inspect/pipeline_fallback.rs` | 12567 | `8845cf0d593881820805647eeb6c8de0c6137fd6` |
| `engine/src/plugins/render/inspect/plan.rs` | 13401 | `a7f405e834e869c95d5dbc3bfaea5046f3f91dc6` |
| `engine/src/plugins/render/inspect/prepared_frame.rs` | 25270 | `632ade45a4a898072cdc1c7cefde5a63669013a0` |
| `engine/src/plugins/render/inspect/producer.rs` | 17453 | `81b15ac873d28d27af1101d2f770f1c18fc5191d` |
| `engine/src/plugins/render/inspect/product_visual_evidence.rs` | 17985 | `cee6182ce2ae5da2e4238faf4f1d202cb4f2884e` |
| `engine/src/plugins/render/inspect/query_snapshot.rs` | 3328 | `a651f0fd8338573cc1c57b28837545db4bb03385` |
| `engine/src/plugins/render/inspect/ray_query.rs` | 17190 | `4fc6b43f9203620f3ed0460bc28d7f3c58e85d85` |
| `engine/src/plugins/render/inspect/readiness.rs` | 21326 | `ed4c39679a8d49a20b95eff83e5fe47da1498afa` |
| `engine/src/plugins/render/inspect/report.rs` | 9762 | `e5ebdf31822b49917cca4d651e31ff63cc1ce95c` |
| `engine/src/plugins/render/inspect/resource_inspector.rs` | 5790 | `d848f185004877629ad8666d9e0dd7b95d01ce69` |
| `engine/src/plugins/render/inspect/scale_production.rs` | 14497 | `9fadf23105f98434595b9b371e4f04cb92483d91` |
| `engine/src/plugins/render/inspect/scale_visibility.rs` | 8336 | `994cafa4f9923b4b44505ae24a3cb0f982f7f1c3` |
| `engine/src/plugins/render/inspect/sdf_production.rs` | 16725 | `44f816547e0bcc1721a9f563191e7a47ae2c0f8a` |
| `engine/src/plugins/render/inspect/sdf_raymarch.rs` | 743 | `dbf75d06cafc9ad9019eeaeb5b33b2c2326267f2` |
| `engine/src/plugins/render/inspect/sdf_residency.rs` | 8708 | `6bb333e710744923e247074447d683469fdd629c` |
| `engine/src/plugins/render/inspect/temporal_production.rs` | 18154 | `eb832b872c2ca4bcac18a94198da2d271db3e4d8` |
| `engine/src/plugins/render/inspect/temporal_upscaling.rs` | 12983 | `c2c2363001c072eaae1252b8dadc8837e47ffd79` |
| `engine/src/plugins/render/inspect/temporal.rs` | 33039 | `bac29efece842d5ffa704aeb601507e72c1fc837` |
| `engine/src/plugins/render/inspect/texture_preview.rs` | 9580 | `a39ebb053d36df8ff3cd148d24710dccdc9bc0ab` |
| `engine/src/plugins/render/inspect/texture_view.rs` | 5278 | `448096d4ca9dd1fb9abaf25b49dc85ba6aa3d229` |
| `engine/src/plugins/render/inspect/timings.rs` | 17272 | `61d2c8a73f416d11492c4563fd763c78ab110a27` |
| `engine/src/plugins/render/inspect/world_runtime.rs` | 583 | `690810473b069f5c85a52b429cbc2ec848004867` |

## material_compiler

Files: 12; bytes: 106986.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/material_compiler/bindings.rs` | 10258 | `b251fa5c6ba06ea18dcb89915ee795508ee4b837` |
| `engine/src/plugins/render/material_compiler/diagnostics.rs` | 3190 | `5dd4ca45818d9fa279b0eedfcfe4b7a7f3537a0a` |
| `engine/src/plugins/render/material_compiler/identity.rs` | 10700 | `58f1b54a20c3ae05b2c77bd7830958afcf7b58fd` |
| `engine/src/plugins/render/material_compiler/mod.rs` | 3537 | `b753fc95718479ee338c8c212582c72faafe13c8` |
| `engine/src/plugins/render/material_compiler/tests.rs` | 23568 | `977443d269e9a934bd25a9c7dd1fa41c5c9ff134` |
| `engine/src/plugins/render/material_compiler/types.rs` | 2485 | `136e3eab2907025a9215306ad7bf0454ebadffaa` |
| `engine/src/plugins/render/material_compiler/validation.rs` | 3222 | `e790fe52a36f7220f881c135d9c4532b0db4c99c` |
| `engine/src/plugins/render/material_compiler/wgsl/literals.rs` | 9843 | `8eb99c67827fac5f832faaf644a4635d661f2d13` |
| `engine/src/plugins/render/material_compiler/wgsl/mod.rs` | 343 | `3bef6a66c3a71a55fae415aaa0b96f5d7141774b` |
| `engine/src/plugins/render/material_compiler/wgsl/preview.rs` | 5340 | `a7e61f6ece0aaee32abd6369fbefc3371fd8b6b0` |
| `engine/src/plugins/render/material_compiler/wgsl/program.rs` | 16928 | `f9478fcfa7280214a849a149bd5a32e139545a85` |
| `engine/src/plugins/render/material_compiler/wgsl/scene.rs` | 17572 | `3034a1c78dcb3e93c28709322013ec4899ffab49` |

## params

Files: 3; bytes: 6476.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/params/gpu_params.rs` | 618 | `bd1a983cc36555eadc32563cada1472ccb94d805` |
| `engine/src/plugins/render/params/gpu_value.rs` | 5662 | `c0c79a339bedd9492dc54ecff30980787567e09e` |
| `engine/src/plugins/render/params/mod.rs` | 196 | `f8bb7609fa145b0b8c051107d495c244bf2b4f5c` |

## pipelines

Files: 3; bytes: 14024.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/pipelines/cache.rs` | 999 | `276a3a7274a37ba4ac0d7bce71681928e58c3891` |
| `engine/src/plugins/render/pipelines/flow_keys.rs` | 12958 | `127cd86e94a7346eb3a26d259eef170a9e180412` |
| `engine/src/plugins/render/pipelines/mod.rs` | 67 | `f7d51c37b8a874c2767498ef5cd09a518aa4019a` |

## procedural

Files: 8; bytes: 47552.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/procedural/authoring.rs` | 5662 | `cfdae5c1767d2619a73ecbdc64c785a44e0ef6b4` |
| `engine/src/plugins/render/procedural/camera.rs` | 9707 | `f2cdf173c9ae3dc40de1a2fb01e8186e5ca1a09b` |
| `engine/src/plugins/render/procedural/descriptors.rs` | 8130 | `666ae867e576eba8ae4fae52b2c2b3da188f902a` |
| `engine/src/plugins/render/procedural/lowering.rs` | 3974 | `3d1966a49d9ecf6f3e080943e9872f71958fb9ee` |
| `engine/src/plugins/render/procedural/mod.rs` | 267 | `3404b0f84e160cbc64a50e67aa3e67f3dd810824` |
| `engine/src/plugins/render/procedural/population/mod.rs` | 44 | `422f3c5f2674b19c486a919ab4a94266b7421baa` |
| `engine/src/plugins/render/procedural/population/uniform_grid.rs` | 13045 | `b47561c3acb68861f72ac2e469b2eff32cd4c55b` |
| `engine/src/plugins/render/procedural/validation.rs` | 6723 | `2f5c55dce41b4b4cd086f115f1724ca712e2c4c3` |

## renderer

Files: 27; bytes: 588626.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/renderer/dynamic_targets.rs` | 21781 | `b70d255704c947efa1058786b97142864356f48b` |
| `engine/src/plugins/render/renderer/extract.rs` | 34896 | `49c3604c406f144e3b59cfb242f915496fb2954d` |
| `engine/src/plugins/render/renderer/frame_bindings.rs` | 1959 | `57554d5be74d5b181e16bd716c33af972693d778` |
| `engine/src/plugins/render/renderer/mod.rs` | 37379 | `4c2fcc9dd44ad33b0f2d5737e478110c553aadf4` |
| `engine/src/plugins/render/renderer/pipeline_cache.rs` | 9797 | `93c253953c1701aab1e37f73bc7c58123f0734ff` |
| `engine/src/plugins/render/renderer/prepare.rs` | 63048 | `ca2528dd85eb563cc587bd0355a78395f452f097` |
| `engine/src/plugins/render/renderer/render_flow/bindings.rs` | 31596 | `3ce8f0be21a5bacb2572e1fd52b28504e1b342cc` |
| `engine/src/plugins/render/renderer/render_flow/canonical_work.rs` | 22980 | `21fb4565a654a61d91e65dde24175eac97e86654` |
| `engine/src/plugins/render/renderer/render_flow/capture.rs` | 17218 | `52b64bf51ceeb32be36a6db9879d019f3f9711d0` |
| `engine/src/plugins/render/renderer/render_flow/execute.rs` | 66869 | `081651d2ffecfb47f225264a64eae425b9472ed4` |
| `engine/src/plugins/render/renderer/render_flow/gpu_timing.rs` | 24081 | `e1d1d05637d84ed3615c436c98616bc562feb821` |
| `engine/src/plugins/render/renderer/render_flow/logical_copy.rs` | 11616 | `4b1ad459c6178e0ceacd8f7142fe7082c3d20c6b` |
| `engine/src/plugins/render/renderer/render_flow/logical_operations.rs` | 20306 | `bdcb7202c3083688459b37f40a72f1d2cddf85c6` |
| `engine/src/plugins/render/renderer/render_flow/logical_timing.rs` | 7382 | `11600fe0b303488ad626a388b1117c238d54c611` |
| `engine/src/plugins/render/renderer/render_flow/mod.rs` | 3968 | `27270899b298a4272cfca21651194788f24c5574` |
| `engine/src/plugins/render/renderer/render_flow/observation.rs` | 14471 | `40fe96320f15e3932666d7dbfdad04c0a0414eb2` |
| `engine/src/plugins/render/renderer/render_flow/occurrences.rs` | 17553 | `008c39146112a91365eb06afd4f716fe5aeb1396` |
| `engine/src/plugins/render/renderer/render_flow/pipeline_realization.rs` | 16872 | `40f52e8144c846e3aadfadcd20d7fa45dbe7907f` |
| `engine/src/plugins/render/renderer/render_flow/preflight_cache.rs` | 11786 | `a6fc3338f36f8966de66f611a5e44b785421e899` |
| `engine/src/plugins/render/renderer/render_flow/program_sources.rs` | 6911 | `042e5708a6dcab347b2df9e1cf2c4d6f816e1303` |
| `engine/src/plugins/render/renderer/render_flow/provenance.rs` | 32058 | `e71f9217f6f4495db48edee2bd16e399e6b16a27` |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources.rs` | 24242 | `3b72f3defffd83a98fcb09bae5dd805a83038211` |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/inspect.rs` | 6217 | `fb756eab9b222a3bff67823d475ee8abd1a11095` |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/realize.rs` | 21381 | `51dca0a726acf28976a8aaa21dc26a1cfa28ac37` |
| `engine/src/plugins/render/renderer/render_flow/runtime_resources/resolve.rs` | 20710 | `74fdc58680ee1eedc9feb131d7ade3bfc068a7f2` |
| `engine/src/plugins/render/renderer/resource_descriptors.rs` | 5963 | `70516ff5e65e9835d72844057c05b3b5cb0ff150` |
| `engine/src/plugins/render/renderer/setup.rs` | 35586 | `3efbab08b81c76ea1125d5ca559ecb2f87cd5520` |

## residency

Files: 3; bytes: 35961.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/residency/handle.rs` | 519 | `bcfccb7b2d4827cdc0071e4915dc23f075fa655e` |
| `engine/src/plugins/render/residency/mod.rs` | 67 | `68b929ddc6eae6813f9160a4204445923de2b9dd` |
| `engine/src/plugins/render/residency/resource.rs` | 35375 | `b0328f8444b4829fcebc18da93d3d1f99b14dd61` |

## resource

Files: 4; bytes: 16542.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/resource/dynamic_target.rs` | 10346 | `48770b1b1b4aef0a01505dce6c89d0a853ddfe4d` |
| `engine/src/plugins/render/resource/mod.rs` | 128 | `f2187e0748f9af2082dbaf13de7a97884b861481` |
| `engine/src/plugins/render/resource/transient.rs` | 5341 | `bd0bb7bc35963bccbcac59815dbb9aeecbddc126` |
| `engine/src/plugins/render/resource/usages.rs` | 727 | `4a5befbe46e2b790504c20e0e4b913aaa19737e2` |

## runtime

Files: 7; bytes: 225685.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/runtime/debug_eval.rs` | 33192 | `2e39a16906024732e5ab9e4f1b3e64c01e592139` |
| `engine/src/plugins/render/runtime/dynamic_targets.rs` | 7916 | `cf447bf763f455c172dc2136223818c85cd7a127` |
| `engine/src/plugins/render/runtime/dynamic_texture_uploads.rs` | 8402 | `20d664f6975f8b09bbceb9389cc2e2e554bc2363` |
| `engine/src/plugins/render/runtime/frame_diagnostics.rs` | 56822 | `56bccabb9cd058ca7c8a53bbc3087ad2d9f6faca` |
| `engine/src/plugins/render/runtime/frame_prepare.rs` | 56704 | `884b8079dc330f869c4cffb08b2b480e548e6fb9` |
| `engine/src/plugins/render/runtime/frame_submit.rs` | 62019 | `2a546b351cd2ef6b3dc657f2e86a29f81cccb770` |
| `engine/src/plugins/render/runtime/mod.rs` | 630 | `61a57613c929f77422b94e2da1b87c1fe584cf06` |

## shader

Files: 5; bytes: 33274.

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/render/shader/helpers.rs` | 3234 | `0661e990c8de6bc56680c591ceb0ae10897ad216` |
| `engine/src/plugins/render/shader/hot_reload.rs` | 152 | `f5ebdf34915335903afe68f26319d81bd50363dc` |
| `engine/src/plugins/render/shader/mod.rs` | 6207 | `dca94b63f3c3443aa0297403436dfe29037edb51` |
| `engine/src/plugins/render/shader/registry.rs` | 19571 | `d6d23227c0d925829b3a4f3c5ea8d3a26a2afd2a` |
| `engine/src/plugins/render/shader/types.rs` | 4110 | `558cc9eb0868288ae8154bf5b4b61be247a14cde` |

