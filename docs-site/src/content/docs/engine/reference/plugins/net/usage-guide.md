---
title: "Net Plugin Usage Guide"
description: "Documentation for Net Plugin Usage Guide."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-04-27
---

# Net Plugin Usage Guide

## Purpose

Composes networking runtime bridge, prediction, and replication behavior.

## Entry Points

- Module: engine/src/plugins/net/plugin.rs
- Entry: NetPlugin<TDriver>
- Local README: engine/src/plugins/net/README.md

## Minimal Setup

```rust
use engine::net::prelude::*;

app.add_plugin(NetPlugin::<MyDriver>::new(NetRole::Client));
```

## Runtime Contract

- Schedule placement: PreUpdate, FixedUpdate, FrameEnd
- Ownership: Role-based networking runtime composition.
- Non-ownership: Transport implementation internals in engine_net.

## Related

- Plugin guides index: [../index.md](../index.md)
- Plugin source map: [../../../../src/plugins/README.md](../../../plugins/README.md)
