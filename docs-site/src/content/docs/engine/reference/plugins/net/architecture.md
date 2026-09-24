---
title: "Net Plugin Architecture"
description: "Current engine-side ownership and composition boundary for networking integration."
status: active
owner: engine
layer: engine-runtime
canonical: true
last_reviewed: 2026-09-13
---

# Net Plugin Architecture

## Ownership Boundary

The Net plugin owns Runenwerk engine composition for retained networking integration:

- role-based installation of client/server/host resources;
- schedule placement for receive, streaming, prediction, replication, flush, and diagnostics;
- derived RunenNet session/owner projections used by engine routing and presentation;
- bounded Runenwerk message staging for retained replication/application payloads;
- product/session metadata and host reconnect/deployment policy.

It does **not** own reusable networking lifecycle, delivery, replication-consistency, recovery, or prediction/reconciliation semantics. Those belong to standalone RunenNet.

It also does not own concrete transport realization. `engine_net` is retained migration evidence for live payload/replication/input/authoring consumers, not a transport implementation. Concrete adapters such as `runen-net-quic` are selected only by maintained product consumers.

## Module Layout

- Primary plugin: `engine/src/plugins/net/plugin.rs`
- Session integration: `engine/src/plugins/net/session_core.rs`
- Retained I/O/staging: `engine/src/plugins/net/runtime_io.rs`
- Resources and schedule installation: `engine/src/plugins/net/resources.rs`
- Entry surface: `NetPlugin<TDriver>`
- Runtime schedule touchpoints: `PreUpdate`, `FixedUpdate`, `FrameEnd`

## Session Projection Rule

`RunenNetSessionCore` places public RunenNet negotiation/session owners at the engine boundary. `RunenNetSessionProjection` contains bindings already accepted by those owners and is used only as derived routing/diagnostic state.

The projection must never become a second authority for admission, loss, retention, replacement, expiry, removal, or closure.

## Staging Rule

`NetworkClientInbox`, `NetworkServerInbox`, `NetworkClientOutbox`, `NetworkServerOutbox`, `NetworkInboundQueue`, and `NetworkOutboundQueue` are bounded engine staging/projection surfaces.

They are not transport queues in the reusable semantic sense, queue admission is not RunenNet delivery acceptance, and frame-end flush does not perform transport I/O.

## Runtime Coupling

- Depend on engine runtime schedules/resources through typed system parameters and explicit system-set ordering.
- Keep cross-plugin coupling data-oriented through resources, projections, and maintained schedule relations.
- Preserve standalone RunenNet as the semantic owner for reusable networking concerns.
- Do not add a replacement engine networking runtime or concrete transport adapter without a proven maintained consumer.

## Current Authority

- [Net Plugin](../../../plugins/net/README.md)
- [Network Integration Flow](../../../plugins/net/network-runtime-flow.md)
- [Networking Usage Guide](../../../plugins/net/networking-usage-guide.md)
- [Runenwerk Networking Architecture](../../../../net/net-architecture.md)
- [Engine Net Integration Design](../../../../design/active/net-plugin-runtime-bridge.md)
