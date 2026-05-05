---
title: "Replication Pipeline"
description: "Documentation for Replication Pipeline."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
---

# Replication Pipeline

This document describes the authoritative replication model used by
`engine_net` contracts and consumed by engine/plugin runtime bridges.

Canonical design:

- [../../design/active/net-authoritative-replication-protocol.md](../../design/active/net-authoritative-replication-protocol.md)
- [../../design/active/net-plugin-runtime-bridge.md](../../design/active/net-plugin-runtime-bridge.md)

## Server Pipeline (Per Connection)

For each fixed tick:

1. Capture authoritative snapshot.
2. For each active `ConnectionId`, read its baseline checkpoint.
3. Choose payload:
   - full snapshot if `needs_full_resync` or missing baseline
   - delta snapshot when `last_ack_cursor` baseline is available
4. Emit targeted delivery command (`ServerToConnection`).
5. Update checkpoint cursors.

Per-connection checkpoint fields:

- `last_ack_cursor`
- `needs_full_resync`

This allows baseline divergence between clients without forcing global
fallback behavior.

Implementation note: the current `AuthoritativeServerRuntime` stores
`last_acknowledged` and `force_full_snapshot` per connection. It does not
yet expose separate sent/full cursor diagnostics; those remain richer
per-connection observability work.

Engine plugin checkpoint resources also track `last_sent_cursor`,
`last_full_snapshot_cursor`, and `last_full_snapshot_tick`. That broader
state belongs to the engine bridge layer until the lower-level
`engine_net` runtime contract is expanded.

## Client Apply Pipeline

On authoritative receive:

1. Validate ordering and cursor progression.
2. For delta snapshots, validate `base` cursor against the client's
   current local cursor.
3. Apply via driver:
   - full: `SnapshotApplyDriver::apply_snapshot`
   - delta: `SnapshotApplyDriver::apply_delta`
4. Update local replication cursor state.
5. Store the merged baseline for future delta validation.
6. Ack authoritative cursor back to server.

Implementation note: `ClientReplicationRuntime` currently returns an
operation plan and maintains cursor/baseline state. Concrete ECS mutation
and predicted-input replay belong to the owning runtime/game integration
layer.

## Failure / Recovery Rules

- Missing/evicted baseline on server forces full resync for that
  connection only.
- Base-cursor mismatch on client rejects delta apply.
- Malformed deltas request full resync.
- Duplicate cursors and stale ticks are rejected and counted.
- Connection close removes only the affected connection checkpoint; other
  peers continue with independent baselines.
