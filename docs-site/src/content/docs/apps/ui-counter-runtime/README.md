---
title: UI Counter Runtime App
description: Current app documentation for the Phase 012 live UiPlugin runtime counter product.
status: active
owner: apps
layer: app
canonical: true
last_reviewed: 2026-07-07
related_docs:
  - ../../design/active/live-uiplugin-runtime-full-cutover-plan.md
  - ../../architecture/live-uiplugin-runtime-platform-architecture.md
  - ../../workspace/specs/pt-ui-runtime-platform-012.ron
---

# UI Counter Runtime App

`apps/ui_counter_runtime` is the Phase 012 product app for the live `UiPlugin` runtime platform track.

It proves the app-facing runtime path with a bounded counter product:

- the app installs engine default plugins, `ScenePlugin`, `RenderPlugin`, `UiPlugin`, and a product-owned `CounterPlugin`;
- `CounterPlugin` mounts `CounterScreen` through `app.mount_ui(CounterScreen)`;
- `CounterScreen` implements the typed UI screen/source contract and exposes a header, count, increment/decrement/reset controls, a trace console, and a status line;
- human pointer controls, human keyboard input, and agent scripts all enter through the same route, capability, payload, host mapping, and `dispatch_ui_action` path;
- `Counter` mutates only through the product host executor after accepted UI action dispatch;
- the evaluated UI frame is published by `UiPlugin` through the generic surface-frame seam and consumed by `RenderPlugin`.

## Commands

Human app:

```text
cargo run -p ui_counter_runtime
```

Headless agent proof:

```text
cargo run -p ui_counter_runtime -- --headless --agent-script assets/ui_counter_runtime/scripts/increment_reset.ron --trace-jsonl target/ui_counter_runtime/trace.jsonl --exit-after-script
```

Focused product tests:

```text
cargo test -p ui_counter_runtime
```

## Boundaries

`ui_counter_runtime` is an app crate. It may depend on `engine` and UI domain crates through public contracts, but it must not become a domain authority.

It does not depend on `domain/ui/ui_app_integration`; that crate remains proof-local and is not the public runtime owner.

It does not own renderer semantics, input plugin internals, source reload, persistence, SDF, SpatialCanvas, or a generic plugin framework.
