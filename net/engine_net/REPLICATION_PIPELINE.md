# Replication Pipeline

## Related Docs

- [engine net README](README.md)
- [engine net plugin README](../../engine/src/plugins/net/README.md)
- [NET_PLUGIN.md](../../engine/src/plugins/net/NET_PLUGIN.md)
- [NETWORKING_USAGE_GUIDE.md](../../engine/src/plugins/net/NETWORKING_USAGE_GUIDE.md)
- [NETWORK_RUNTIME_FLOW.md](../../engine/src/plugins/net/NETWORK_RUNTIME_FLOW.md)

This document describes the replication pipeline model used by
`engine_net` contracts and drivers across server/client runtime loops.

## Server Pipeline

After each simulation tick, the authoritative server emits updates:

Simulation Tick  
-> Dirty Detection  
-> Interest Filtering  
-> Baseline Selection  
-> Snapshot Build  
-> Delta Encoding  
-> Profile Routing  
-> Transport Lane Selection  
-> Packet Dispatch

### 1. Dirty Detection

Changed state is identified from simulation mutations:

- component changes
- spawn/despawn
- add/remove component transitions

### 2. Interest Filtering

Per-client relevance filtering (for example global/owner/spatial/team)
reduces payload surface.

### 3. Baseline Selection

Server chooses the last client-acknowledged snapshot cursor/tick as the
delta base. Missing/mismatched baseline falls back to full snapshot.

### 4. Snapshot Build

Authoritative snapshot content is assembled:

- spawn instructions
- component patches
- despawn instructions

### 5. Delta Encoding

When a baseline exists, a delta is generated from
`current - baseline`; otherwise a full snapshot is emitted.

### 6. Profile Routing

Each replicated component/entity path is routed by replication profile
(cadence/reliability/prediction priority model).

### 7. Transport Lane Selection

Profiles map to transport-lane semantics in
`engine_net/src/transport/` and adapter implementations
(`engine_net_quic/src/transport/`).

### 8. Packet Dispatch

Encoded frames are emitted to the transport adapter runtime.

## Client Pipeline

Client receive/apply flow:

Receive Packet  
-> Ordering / Tick Validation  
-> Entity Resolution  
-> Spawn / Update / Despawn Apply  
-> Prediction Reconciliation  
-> Interpolation / Smoothing

### 1. Tick Validation

Out-of-order/stale snapshots are rejected according to runtime policy.

### 2. Entity Resolution

Network entity IDs are resolved to runtime entities.

### 3. State Apply

Authoritative updates are applied for spawn/update/despawn.

### 4. Prediction Reconciliation

Predicted state is compared against authoritative state and corrected.

### 5. Smoothing

Presentation smoothing/interpolation is applied after correction.
Final smoothing policy remains game-owned.

## Summary

Replication is a deterministic pipeline from simulation changes to
client reconciliation:

Simulation  
-> Dirty  
-> Interest  
-> Snapshot/Delta  
-> Routing/Lanes  
-> Dispatch  
-> Client Apply  
-> Reconcile/Smooth
