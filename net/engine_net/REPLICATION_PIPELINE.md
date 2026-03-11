# Replication Pipeline

This document describes the authoritative replication model used by
`engine_net` contracts and consumed by engine/plugin runtime bridges.

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
- `last_sent_cursor`
- `last_full_snapshot_cursor`
- `last_full_snapshot_tick`
- `needs_full_resync`

This allows baseline divergence between clients without forcing global
fallback behavior.

## Client Apply Pipeline

On authoritative receive:

1. Validate ordering and cursor progression.
2. For delta snapshots, validate `base` cursor against the client’s last
   acknowledged cursor.
3. Apply via driver:
   - full: `SnapshotApplyDriver::apply_snapshot`
   - delta: `SnapshotApplyDriver::apply_delta`
4. Update local replication cursor state.
5. Replay pending predicted input frames newer than authoritative tick.
6. Ack authoritative cursor back to server.

## Failure / Recovery Rules

- Missing/evicted baseline on server forces full resync for that
  connection only.
- Base-cursor mismatch on client rejects delta apply.
- Connection close removes only the affected connection checkpoint; other
  peers continue with independent baselines.
