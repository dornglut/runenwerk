---
title: "Net Plugin Usage Guide"
description: "Minimal setup and current ownership boundary for the Runenwerk Net plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Net Plugin Usage Guide

## Purpose

Install the current Runenwerk engine integration for retained networking migration consumers without recreating lifecycle, delivery, or transport authority already owned by standalone RunenNet.

## Entry Points

- Module: `engine/src/plugins/net/plugin.rs`
- Entry: `NetPlugin<TDriver>`
- Local README: [Net Plugin](../../../plugins/net/README.md)
- Detailed retained low-level guide: [Networking Usage Guide](../../../plugins/net/networking-usage-guide.md)

## Minimal Setup

```rust
use engine::net::prelude::*;

app.add_plugins(NetPlugin::<MyDriver>::new(NetRole::Client));
```

Use `NetRole::Server` or `NetRole::Host` for the corresponding retained integration role.

There is no `NetworkRuntimeHandle` startup step. Application/host lifecycle code places and invokes the accepted RunenNet negotiation/session owners; the engine consumes already-authorized bindings through `RunenNetSessionProjection`.

## Runtime Contract

- Schedule placement: `PreUpdate`, `FixedUpdate`, `FrameEnd`.
- Connection identity for retained routing/state: RunenNet `ConnectionHandle`.
- Lifecycle authority: standalone RunenNet, not the Net plugin or `engine_net`.
- Engine inbox/outbox and `NetworkInboundQueue` / `NetworkOutboundQueue`: bounded staging/projection only.
- Concrete transport realization: separate adapter/product concern; `engine_net` is not a transport runtime.

The current plugin remains transitional RN8 integration. It does not define the future ordinary multiplayer authoring syntax or authorize the next RN8 cut.

## Related

- [Net Plugin Architecture](architecture.md)
- [Plugin guides index](../index.md)
- [Plugin source map](../../../plugins/README.md)
- [Runenwerk Networking Architecture](../../../../net/net-architecture.md)
