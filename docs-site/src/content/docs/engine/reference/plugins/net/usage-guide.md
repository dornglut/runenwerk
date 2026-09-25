---
title: "Net Plugin Usage Guide"
description: "Minimal setup and current ownership boundary for the Runenwerk Net plugin."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# Net Plugin Usage Guide

## Purpose

Install the current Runenwerk Engine networking integration without recreating lifecycle, delivery, transport, simulation, or cadence authority owned elsewhere.

## Entry Points

- Module: `engine/src/plugins/net/plugin.rs`
- Entry: `NetPlugin<TDriver>`
- Local README: [Net Plugin](../../../plugins/net/README.md)
- Detailed low-level expert guide: [Networking Usage Guide](../../../plugins/net/networking-usage-guide.md)

## Composition

Net role configuration consumes Runenwerk simulation integration state. Compose `SimulationPlugin`
before the Net plugin when not using the ordinary default stack:

```rust
use engine::net::prelude::*;
use engine::plugins::SimulationPlugin;

app.add_plugin(SimulationPlugin);
app.add_plugin(NetPlugin::<MyDriver>::new(NetRole::Client));
```

Use `NetRole::Server` or `NetRole::Host` for the corresponding integration role.

Networking systems that run in `FixedUpdate` also require explicit fixed cadence:

```rust
use engine::plugins::{FixedStepPlugin, SimulationPlugin};

app.add_plugins((FixedStepPlugin, SimulationPlugin));
app.add_plugin(NetPlugin::<MyDriver>::new(NetRole::Server));
```

`default_plugins()` already selects FixedStep and Simulation integration for the ordinary Engine
stack. `NetPlugin` itself does not become a fallback provider for either capability.

There is no `NetworkRuntimeHandle` startup step. Application/host lifecycle code places and invokes
the accepted RunenNet negotiation/session owners; the engine consumes already-authorized bindings
through `RunenNetSessionProjection`.

## Runtime Contract

- Schedule placement: `PreUpdate`, `FixedUpdate`, `FrameEnd`.
- Connection identity for Engine routing/integration state: RunenNet `ConnectionHandle`.
- Lifecycle authority: standalone RunenNet, not the Net plugin or `engine_net`.
- Simulation identity/configuration: consumed from `SimulationPlugin` integration; not owned by Net.
- Fixed cadence: consumed when fixed networking systems are intended to run; not activated by Net.
- Engine inbox/outbox and `NetworkInboundQueue` / `NetworkOutboundQueue`: bounded staging/projection only.
- Concrete transport realization: separate adapter/product concern; the deleted `engine_net` shell is not a transport runtime or compatibility surface.

The current plugin is the maintained low-level/expert Engine integration. It does not define the future ordinary multiplayer authoring syntax or authorize final Replicated View syntax under #322.

## Related

- [Net Plugin Architecture](architecture.md)
- [Simulation Plugin](../simulation/usage-guide.md)
- [Fixed Step Plugin](../fixed-step/usage-guide.md)
- [Plugin guides index](../index.md)
- [Plugin source map](../../../plugins/README.md)
- [Runenwerk Networking Architecture](../../../../net/net-architecture.md)
