---
title: Runenwerk Wider Render Integration Source Census
description: Exact accepted-revision inventory of retained Runenwerk render-facing integration outside engine/src/plugins/render for investigation #1138.
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
  - ./2026-10-03-runenwerk-render-runtime-dependency-census.md
---

# Runenwerk Wider Render Integration Source Census

```text
RUNENWERK_REVISION=8d0bad6c7e4c24f2579bd89c3f797946af85dba3
CENTRAL_RENDER_RUST_FILES=184
CENTRAL_RENDER_RUST_BYTES=2442976
WIDER_RENDER_INTEGRATION_UNIQUE_RUST_FILES=124
WIDER_RENDER_INTEGRATION_UNIQUE_RUST_BYTES=2707709
COMBINED_CONCRETE_RUNENWERK_RENDER_FOOTPRINT_RUST_FILES=308
COMBINED_CONCRETE_RUNENWERK_RENDER_FOOTPRINT_RUST_BYTES=5150685
```

## Inclusion rule

This appendix makes the investigation's wider-source scope reproducible instead of treating
"render-related" as a subjective directory label.

Outside `engine/src/plugins/render/**`, include:

1. every Rust file under `apps/runenwerk_render_lab/**`, because Render Lab is explicitly a
   rendering product/validation consumer;
2. every accepted-revision Rust file under Arena, Editor, Draw, Engine UI, Engine World, and other
   Engine host/scene/debug integration that has a direct lexical dependency on
   `engine::plugins::render` / `crate::plugins::render`;
3. every Engine example/test Rust file with a direct lexical dependency on Runenwerk Render,
   standalone `runen_render`, or standalone `runen_gpu`.

This is intentionally stricter than counting entire app/test/example directories. Files with no
direct render/framework edge are excluded even if they live beside render-facing code. The central
184-file Render tree is inventoried separately and is not duplicated here.

## Category summary

| Category | Files | Source bytes |
| --- | ---: | ---: |
| Render Lab — all Rust | 9 | 314532 |
| Arena — direct Render integration | 5 | 100114 |
| Editor — direct Render integration | 34 | 1372305 |
| Draw — direct Render integration | 4 | 192400 |
| Engine UI — direct Render integration | 4 | 44688 |
| Engine World — direct Render integration | 4 | 30275 |
| Other Engine host/scene/debug integration | 8 | 89154 |
| Engine examples/proofs — direct Render/RunenRender/RunenGPU | 21 | 168561 |
| Engine tests — direct Render/RunenRender/RunenGPU | 35 | 395680 |
| **Unique wider total** | **124** | **2707709** |

## Render Lab — all Rust

| Path | Bytes | Blob |
| --- | ---: | --- |
| `apps/runenwerk_render_lab/src/automation.rs` | 15185 | `ce514b6e3ae156df7b6fc5664c11dc0aaa126323` |
| `apps/runenwerk_render_lab/src/camera.rs` | 13448 | `8b89ab4a578498826cb8324329dbcc6bd8462a3e` |
| `apps/runenwerk_render_lab/src/comparison.rs` | 39742 | `1cb67dc5252da32a386b8da75944e2b7ad83d9ef` |
| `apps/runenwerk_render_lab/src/lib.rs` | 36047 | `0d4dd9f5f3dd11a2aa23a6d13867c858a0843522` |
| `apps/runenwerk_render_lab/src/main.rs` | 42316 | `739d15d345bc4395569943a1068f3725710553cc` |
| `apps/runenwerk_render_lab/src/native.rs` | 69811 | `4df6e0043616daa9dfab74cf65d518433c818b94` |
| `apps/runenwerk_render_lab/src/native/comparison_evidence.rs` | 13372 | `52beac3d2e1c854b39bcb0b851fb818248fa41a9` |
| `apps/runenwerk_render_lab/src/native/temporal_quality.rs` | 66029 | `eb3f2564c99e7cda99025dbb568c3ec636d0c557` |
| `apps/runenwerk_render_lab/tests/automation_terminal_replay.rs` | 18582 | `1703e5cedd81366cc09cc6c91acddaaae794ea8a` |

## Arena — direct Render integration

| Path | Bytes | Blob |
| --- | ---: | --- |
| `apps/runenwerk_arena/src/lib.rs` | 1113 | `2ea189f538e7a1c9de66b8409ccb01c17a6030e0` |
| `apps/runenwerk_arena/src/presentation.rs` | 61595 | `b30f0911020c1dcc01346beacd6f7c5d3c854bfd` |
| `apps/runenwerk_arena/tests/arena_presentation.rs` | 2525 | `2973b001faa3957feca0d0006963daa56eaa724b` |
| `apps/runenwerk_arena/tests/arena_presentation_gpu.rs` | 10959 | `2dd995bc78026c8988e7c171f8e9c5aa2bdf8ee2` |
| `apps/runenwerk_arena/tests/arena_world.rs` | 23922 | `c330b02e855d2b4129d1f50095fe661f2d69b906` |

## Editor — direct Render integration

| Path | Bytes | Blob |
| --- | ---: | --- |
| `apps/runenwerk_editor/src/bin/runenwerk_ui_gallery.rs` | 15054 | `de84bd220b0b15c02b51e646eb6f40ef718244f2` |
| `apps/runenwerk_editor/src/material_lab/default_material.rs` | 12304 | `76b14d43e2afb1054cb2a07bf6a8ec886062b737` |
| `apps/runenwerk_editor/src/material_lab/preview_surface.rs` | 2272 | `fe27ab4e7fe5d31f75f2ea2ceb643f332f8c989c` |
| `apps/runenwerk_editor/src/material_lab/renderer_handoff.rs` | 125307 | `e5c44a70eb4de621c61839144c8b8769aab299d3` |
| `apps/runenwerk_editor/src/material_lab/state/runtime.rs` | 16349 | `e36e3bc13f8b4cb99c44fd14af3d5700c5114ffe` |
| `apps/runenwerk_editor/src/material_lab/workflow/mod.rs` | 1916 | `c82c22a4330b2f053adca6f0f539503beedf3ef1` |
| `apps/runenwerk_editor/src/runtime/app.rs` | 15486 | `f792e08a52e36b3e37d01202fb2ab64213ccd0e0` |
| `apps/runenwerk_editor/src/runtime/composition/transition.rs` | 69790 | `eecab85760238485852b2ddea271bdddf777f752` |
| `apps/runenwerk_editor/src/runtime/expression/picking.rs` | 3446 | `c54c16385df09441049499a26db705e70bdfc293` |
| `apps/runenwerk_editor/src/runtime/plugin.rs` | 17241 | `f599954e3f6ba765c3c59ee7a2f2f3f85b642c10` |
| `apps/runenwerk_editor/src/runtime/preview_process/shader_status.rs` | 2961 | `3da1ba272f1913f86e3c5e99825c6070706c11c0` |
| `apps/runenwerk_editor/src/runtime/resources.rs` | 85397 | `c3c0302ebc81cfc79dcc60864559e97a0c1dbb32` |
| `apps/runenwerk_editor/src/runtime/systems/bootstrap.rs` | 4554 | `acdaabbcadb6796ca45785e10368bea6f4643878` |
| `apps/runenwerk_editor/src/runtime/systems/frame_submit.rs` | 50220 | `f9223e8fd5b6bb6d1736279a2b467e52a29c2d15` |
| `apps/runenwerk_editor/src/runtime/systems/input_bridge.rs` | 78764 | `d68d425112516b986f448beb8130188d613008b7` |
| `apps/runenwerk_editor/src/runtime/systems/material_preview.rs` | 49506 | `c4b41471ae454eff55b962f8e0328c0f5a46ae92` |
| `apps/runenwerk_editor/src/runtime/systems/picking.rs` | 42372 | `25471c8290f847222c870e81fbd3a63fd1f29171` |
| `apps/runenwerk_editor/src/runtime/systems/texture_preview.rs` | 5882 | `ce3011dcab123bffe2737cc1ff6191a5edfe781e` |
| `apps/runenwerk_editor/src/runtime/ui_gallery.rs` | 9871 | `d61953ba63df40865c3a2b5f6a6db0a8fe4fd6bd` |
| `apps/runenwerk_editor/src/runtime/ui_gallery_execution.rs` | 10522 | `d3c69574387bca4b82965ca473ac556a56a1cad7` |
| `apps/runenwerk_editor/src/runtime/viewport/gpu_residency.rs` | 2960 | `2e3765197a581c69f6601c52d9c0796ba3145831` |
| `apps/runenwerk_editor/src/runtime/viewport/picking_results.rs` | 4295 | `13b132e08cc3ba5784b6c2ee1af8675b4205db81` |
| `apps/runenwerk_editor/src/runtime/viewport/producer_ids.rs` | 643 | `dc625d0b4e91db1c4aa41aeb351333172ba85b7d` |
| `apps/runenwerk_editor/src/runtime/viewport/product_targets.rs` | 42695 | `3c389581df7b5bba71c370d878f60e0e889c8227` |
| `apps/runenwerk_editor/src/runtime/viewport/render_jobs.rs` | 18580 | `57569c0877781c9ad56949badb972921e073645f` |
| `apps/runenwerk_editor/src/runtime/viewport/render_product_selection.rs` | 21387 | `b33dc7449c8836ad000f2a747ce661e8f67f54de` |
| `apps/runenwerk_editor/src/shell/controller.rs` | 67848 | `c89fafb9653bc8c420f178b18c8d57f3cd762a93` |
| `apps/runenwerk_editor/src/shell/state.rs` | 83166 | `6ba216e8ecf1b6d18aeed796a10133a07f114d5b` |
| `apps/runenwerk_editor/src/shell/tests.rs` | 304925 | `e8537988484fd4890f777a2b1bdfb3342163c3dd` |
| `apps/runenwerk_editor/src/texture_preview/mod.rs` | 21437 | `9487c6974aed165ce74db8d78f9e3aa77513b4fb` |
| `apps/runenwerk_editor/tests/region_compass_visual_capture.rs` | 16547 | `a899c96fe2bf1d085be260c4eb618c72bb1568de` |
| `apps/runenwerk_editor/tests/startup_render_smoke.rs` | 21445 | `d811ab467207305d0bdcc22995485e5cfba009e3` |
| `apps/runenwerk_editor/tests/viewport_architecture_guards.rs` | 77035 | `b4f88ba7f35a87a5b0eb67d91cc1e43e3ca074f9` |
| `apps/runenwerk_editor/tests/viewport_gpu_truth_smoke.rs` | 70128 | `9b82e466a80fc9b6b85904f9fb8de6ae4d2e463e` |

## Draw — direct Render integration

| Path | Bytes | Blob |
| --- | ---: | --- |
| `apps/runenwerk_draw/src/runtime/gpu_ink.rs` | 19277 | `8335453262603488df7f4cb0bbc5254c81ad0b05` |
| `apps/runenwerk_draw/src/runtime/plugin.rs` | 3488 | `fb0384a14a655e48cf5fdc271b3ff05294a4fdd5` |
| `apps/runenwerk_draw/src/runtime/systems.rs` | 56554 | `14b95f9c80864a1000a065261005438690663302` |
| `apps/runenwerk_draw/tests/app_shell.rs` | 113081 | `5f448f2a8bf5a949dc692d6b98bba60b708238df` |

## Engine UI — direct Render integration

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/ui/diagnostics.rs` | 9792 | `365eb7ed9b1b7110c4a4d281696dc14de8bd6766` |
| `engine/src/plugins/ui/render_publish.rs` | 5850 | `d2ca778705ed5e54e4f608035533b84a5c41ef5d` |
| `engine/src/plugins/ui/report.rs` | 19018 | `4e7567580c3551d162a6faf2fc742496067adc2b` |
| `engine/src/plugins/ui/trace.rs` | 10028 | `9f801843f1064e322b4ea56fc2af9ba071ccb449` |

## Engine World — direct Render integration

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/world/chunks/render_cache_bridge.rs` | 4245 | `b0f397fdca15a98995d89b4d584f381ee5e2f610` |
| `engine/src/plugins/world/plugin.rs` | 8718 | `6a0e4efb37d7ae6601de3d5a94e50790a60f9794` |
| `engine/src/plugins/world/prepare/contributions.rs` | 9426 | `c863b6ef68510ef41d8915216b7c11866741e907` |
| `engine/src/plugins/world/prepare/sdf_render_bridge.rs` | 7886 | `c54c3491a287645ce8113dbebfed4e4d3cfe3cdb` |

## Other Engine host/scene/debug integration

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/src/plugins/debug_metrics/mod.rs` | 12117 | `b8f44439c34211f19424596a26d80bfa9bd2ac0d` |
| `engine/src/plugins/scene/lifecycle/overlay_update.rs` | 2434 | `3824786622464df09d6ee5d76db5bbe1d1f89778` |
| `engine/src/plugins/scene/plugin.rs` | 2110 | `f99e75d02b4aa9ca137d63abc42dd6884f527828` |
| `engine/src/plugins/scene/runtime/overlay_ui.rs` | 17138 | `a1c27ff51f2e6dc722cf87d28ee6ef1ef4612f0b` |
| `engine/src/prelude.rs` | 1167 | `d362c6bf5cf26d328798c6438413065cdb4fc791` |
| `engine/src/runtime/winit_runner.rs` | 44333 | `5b2c49a987014b40f2c7e6efbdddb30e768f623a` |
| `engine/src/runtime/winit_window_realizer.rs` | 8668 | `1d36026b2decb076328901a560dc53b335ab8d96` |
| `engine/src/state.rs` | 1187 | `2ce0709598438c37d05863be7c0aa81605f5ec85` |

## Engine examples/proofs — direct Render/RunenRender/RunenGPU

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/examples/boids_render_flow/rendering/evidence.rs` | 27396 | `95646104ca39429d07778f25aca8ee0ac36fc9af` |
| `engine/examples/boids_render_flow/rendering/graph.rs` | 20157 | `48f51ecf6af0401a7e775a7c088f24d8bd6fac89` |
| `engine/examples/boids_render_flow/rendering/state.rs` | 9625 | `eabb1629ca49bb7cb27aae44442e102cc7b0a8a0` |
| `engine/examples/game_of_life_sdf/rendering/graph.rs` | 4094 | `731e4fd8fe839ac34670667ce7ef0b68d83e1876` |
| `engine/examples/game_of_life_sdf/rendering/state.rs` | 5911 | `a98aa1d21bce7635841e3a127cb9d93063c7e368` |
| `engine/examples/procedural_sky_sdf_terrain/rendering/graph.rs` | 2928 | `6927a4cc7ba21f5ae3f66023cb3e4a913dd17a8f` |
| `engine/examples/procedural_sky_sdf_terrain/rendering/state.rs` | 8861 | `5bbab250d976b709db5a1af7996da287a3020254` |
| `engine/examples/render_flow_debug_inspect/main.rs` | 2764 | `245c3761006bdbdae70ea33050abd9340c60d35d` |
| `engine/examples/render_flow_fullscreen_minimal/main.rs` | 635 | `77ed100a197dda6e5ce2831fb5ccab1c5e8d1f4e` |
| `engine/examples/render_flow_postprocess_compositor/main.rs` | 1592 | `80555c99a5f0b2986517fe85ceda1303eb3a2ce0` |
| `engine/examples/render_fragment_compositor.rs` | 1968 | `4d81c7746a4fa57589bcf6dccbbb59f4f0347bba` |
| `engine/examples/render_hybrid_ray_sdf_raster_runtime_proof.rs` | 23113 | `cf06477279e87c71c8dd73cd5cd509ad419f134c` |
| `engine/examples/render_mesh_material_production_evidence.rs` | 6020 | `b5ea3d952f9c3d59e2364bbe42c03ab192f2dc3c` |
| `engine/examples/render_product_visual_evidence.rs` | 4831 | `b3d6d2a244d07e5701458c2519d5eb09f8790974` |
| `engine/examples/render_readiness_inspection.rs` | 2878 | `cda2c26680c80338d2d1b75f15769696fdc02628` |
| `engine/examples/render_scale_evidence.rs` | 6079 | `0c146e096095e2c014a8ebd1f4d9a8598ce3af2f` |
| `engine/examples/render_sdf_runtime_evidence.rs` | 7022 | `ac8de68cb4ddfac7b2c753c7c84e69bc99b87968` |
| `engine/examples/render_temporal_production_evidence.rs` | 9365 | `25851fa7c4dddbae076ad0cd5df8bfee24a38508` |
| `engine/examples/sdf_render_flow/rendering/evidence.rs` | 12232 | `13dfbfc5d51993580e18463a783a609279b57cc6` |
| `engine/examples/sdf_render_flow/rendering/graph.rs` | 5450 | `22967b3bbda136c28af88a335bfe465b93c63b03` |
| `engine/examples/sdf_render_flow/rendering/state.rs` | 5640 | `d5ed042929fe815c772829580170431a14b61c44` |

## Engine tests — direct Render/RunenRender/RunenGPU

| Path | Bytes | Blob |
| --- | ---: | --- |
| `engine/tests/native_host_smoke.rs` | 15511 | `c021bd398076e86bb416426238ff7cc76ecfe391` |
| `engine/tests/procedural_instance.rs` | 16935 | `7dfcc74d55d990ca05eb6ea6fb841f5775fae5d2` |
| `engine/tests/render_dynamic_targets.rs` | 46435 | `9c6365999109f91f3a58c78f8351b713a0aa6069` |
| `engine/tests/render_dynamic_texture_uploads.rs` | 2786 | `a2ad4a59d1bee58a294d63533f4d087bac9c39df` |
| `engine/tests/render_flow_fragments.rs` | 12705 | `cc438202fb84de226cfd9af883342cdc1bd8ed78` |
| `engine/tests/render_flow_v2.rs` | 20789 | `d653c2e8274c01da6d69611d58128e3c66ecf30e` |
| `engine/tests/render_frame_field_semantic_input.rs` | 10417 | `e26e7f9cd993f620caf42208ffab1fd769f8f4b2` |
| `engine/tests/render_gpu_params.rs` | 1403 | `b1b3048cce701e46b865f1dd734c650585f60e00` |
| `engine/tests/render_gpu_timing.rs` | 5545 | `db15f450d3e1f05f0ccaf82cc8b8bf24c1f0c979` |
| `engine/tests/render_import_contract.rs` | 4531 | `246edac8141e35dc6ee6fb19644232f3f21ad2f2` |
| `engine/tests/render_mesh_material_handoff.rs` | 8170 | `bf0f7f8be3cedf443eaf3b1df71d3cf5a9d55956` |
| `engine/tests/render_mesh_material_production_evidence.rs` | 8491 | `c60a90f1a6d44811deadc92b2091b7867365f36a` |
| `engine/tests/render_multi_surface.rs` | 4406 | `cb57127ecf2fa1e03086aba3951b354da1cfb776` |
| `engine/tests/render_pipeline_fallback.rs` | 9617 | `a508b42831ae83d3f579931e93a44aa6a50695da` |
| `engine/tests/render_product_visual_evidence.rs` | 8542 | `b7bb5ca8ad53072ba6c106526f772988adda93cb` |
| `engine/tests/render_ray_query.rs` | 7961 | `aa2f4a1be0b1e44bda77aa38955b92182e0e286c` |
| `engine/tests/render_readiness_ownership.rs` | 1100 | `2176b7cff0c4f3c152fdf28f23893af7fd54ce0f` |
| `engine/tests/render_resource_model.rs` | 12408 | `58b6730b94d0ca8b05b5687f051ebfbdc955562b` |
| `engine/tests/render_runtime_inspect.rs` | 48746 | `30cfa506416eb036671f4e28e0b9379165658036` |
| `engine/tests/render_scale_production_evidence.rs` | 6623 | `7a3bca01f59880c726a53935fd2c88c9d1a9b078` |
| `engine/tests/render_scale_visibility.rs` | 3678 | `959c961e8c13ca5fb8ed80024899395ed0a97d12` |
| `engine/tests/render_scale_working_set.rs` | 6186 | `137081a41913e649f70a186fc3110f49bce19272` |
| `engine/tests/render_sdf_raymarch.rs` | 6389 | `8d4e8c139c4780007186bbcfc3e76e7e42ce4c55` |
| `engine/tests/render_sdf_residency.rs` | 7470 | `b71c96a0fdb4077245d119d535e2d84d993ed003` |
| `engine/tests/render_sdf_runtime_evidence.rs` | 7990 | `0be78e9cab8b76a230132f9bbc8a2d814b998e48` |
| `engine/tests/render_temporal_inputs.rs` | 10054 | `244fdc634e83a958dbff695dee86373072928d15` |
| `engine/tests/render_temporal_production_evidence.rs` | 11722 | `46c9e158fd37a39a2d88b1bb2d637a0c33003da4` |
| `engine/tests/render_temporal_upscaling.rs` | 8999 | `7b6a2fa87fff32794bc5c3ba2316433b50ed7362` |
| `engine/tests/render_ui_composite_scope.rs` | 1644 | `e6dc1d148a0af3bace1bcb839b229366574bda24` |
| `engine/tests/runenrender_ordinary_public_api.rs` | 13189 | `7f475f47be08bb1a86028143f709620580b0af90` |
| `engine/tests/runtime_ui_producer_migration.rs` | 7997 | `ffab37313c756d91be4d414f6c453e7962fd3b0c` |
| `engine/tests/ui_render_publication.rs` | 12692 | `735279ee7b4af6871e1a7bf048a465cf1aa832c5` |
| `engine/tests/world_render_cache_invalidation_bridge.rs` | 13420 | `ca24e961d067f2e725b3b52c7449fa7dcb110250` |
| `engine/tests/world_render_sdf_bridge.rs` | 11833 | `7dbcd1174a94b2ad4ba3696c3ed0016239a43ebe` |
| `engine/tests/world_render_sdf_field_bridge.rs` | 19296 | `4bf06ba3d2d8a9b6b494bf255a089f9a7dcbdaf6` |

