---
title: PM-RENDER-PG-006 Multi-Surface Presentation Closeout
description: Closeout evidence for the bounded native multi-window and multi-surface presentation contract slice.
status: completed
owner: editor
layer: app / engine-runtime / render-runtime
canonical: false
last_reviewed: 2026-05-21
related_designs:
  - ../../../design/accepted/editor-native-multi-window-presentation-design.md
  - ../../../design/accepted/render-product-graph-platform-design.md
  - ../../../design/accepted/product-surface-platform-hardening-design.md
related_roadmaps:
  - ../../../workspace/roadmap-items.yaml
  - ../../../workspace/production-tracks.yaml
---

# PM-RENDER-PG-006 Multi-Surface Presentation Closeout

## Result

`PM-RENDER-PG-006` completed as a bounded multi-surface presentation contract slice.

The implementation adds typed logical editor-window records, app-owned editor-window to native-window to render-surface bindings, engine-native window records and window-scoped platform events, render-surface registry records, prepared-frame surface identity, prepared-frame inspection for surface ids, and submit-time rejection for prepared frames that target an unknown or mismatched render surface.

## Evidence

- `domain/editor/editor_shell/src/workspace/window.rs` owns logical editor-window records without native handles, swapchains, or renderer-private state.
- `apps/runenwerk_editor/src/shell/state.rs` owns editor-window presentation bindings and queues secondary presentation requests.
- `apps/runenwerk_editor/src/runtime/plugin.rs` converts queued editor-window presentation requests into engine-owned native-window and render-surface records.
- `engine/src/runtime/window.rs` owns `NativeWindowId`, native-window records, creation requests, and the runtime window registry.
- `engine/src/runtime/platform.rs` adds window-scoped platform events and tests resize behavior across primary and secondary windows.
- `engine/src/runtime/winit_runner.rs` syncs the primary runtime window registry and drains secondary native-window creation requests without routing secondary redraws through the singleton primary `Gfx`.
- `engine/src/plugins/render/backend/surface.rs` owns `RenderSurfaceId`, render-surface records, diagnostics, and the render-surface registry.
- `engine/src/plugins/render/frame/packet.rs` scopes `PreparedSurfaceInfo` by render surface and native window identity.
- `engine/src/plugins/render/runtime/frame_prepare.rs` publishes prepared frames with surface identity.
- `engine/src/plugins/render/runtime/frame_submit.rs` rejects prepared frames that target unknown or mismatched render surfaces.
- `engine/src/plugins/render/inspect/prepared_frame.rs` exposes render surface and native window ids in prepared-frame inspection.

## Validation

Passed:

```text
cargo test -p editor_shell multi_window
cargo test -p engine window
cargo test -p engine --test render_multi_surface
cargo test -p runenwerk_editor window
cargo test -p runenwerk_editor --test viewport_architecture_guards
```

The GPU/native two-window smoke proof was not run in this environment:

```text
RUNENWERK_ENABLE_GPU_SMOKE=1 RUNENWERK_ENABLE_MACOS_MAIN_THREAD_GPU_SMOKE=1 cargo test -p runenwerk_editor --test viewport_gpu_truth_smoke -- --ignored
```

## Completion Quality

`completion_quality: bounded_contract`

This closeout does not claim `runtime_proven` or `perfectionist_verified`. The deterministic contract path is in place and validated, but independent GPU presentation into two real native swapchains still needs host-supported runtime proof before this can be called runtime-proven.

## Known Gaps

- Secondary native windows are created from engine-owned requests, but secondary redraws intentionally do not submit through the singleton primary `Gfx`.
- Independent per-window shell UI frame submission and true two-swapchain GPU presentation need runtime/GPU proof in a host environment that supports the ignored smoke test.
- PM-RENDER-PG-007 still owns render fragments and hot reload.
- PM-RENDER-PG-008 still owns production readiness, capture/replay policy, performance budgets, final examples, and final inspection hardening.
- Product truth, product policy, material truth, freshness, authority, fallback legality, rebuild policy, and residency policy remain outside renderer ownership.
