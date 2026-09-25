---
title: "Replication Pipeline"
description: "Current retained replication pipeline during the RunenNet cutover."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-24
---

# Replication Pipeline

This document describes the retained Runenwerk replication path during RN8 after the client-consistency cut to standalone RunenNet.

Connection/session lifecycle is not part of this pipeline. RunenNet Core authorizes participant/connection bindings first; retained replication consumes those bindings through RunenNet `ConnectionHandle` identity.

## Server Pipeline

For each fixed tick:

1. Read active connections from the engine `RunenNetSessionProjection`.
2. Capture authoritative state for each authorized `ConnectionHandle`.
3. Read that connection's retained baseline checkpoint.
4. Choose a full snapshot when full resync is required or no acknowledged baseline is available.
5. Otherwise build a delta from the acknowledged retained baseline.
6. Stage the snapshot/delta as `OutboundServerMessage::ToConnection`.
7. Record sent cursors and streaming markers for that connection.

Per-connection checkpoint state includes:

- last acknowledged cursor;
- last sent cursor;
- last full-snapshot cursor/tick;
- full-resync requirement;
- retained sent cursors and baselines.

Different clients may therefore advance independently without global fallback.

## Admission Rule

ACK and input processing is accepted only when:

- the inbound message identifies a `ConnectionHandle`; and
- that handle is still bound in the RunenNet-authorized engine projection.

The replication layer does not decide whether a connection should be admitted or retained.

## ACK Handling

An ACK is rejected when its cursor is stale, in the future, was never sent, or no longer has a retained baseline. Rejected ACKs do not become delta baselines.

Accepted ACKs advance the connection checkpoint and the corresponding streaming cursor marker.

## Client Apply Pipeline

On authoritative receive:

1. Require explicit client lineage and finite retention policy.
2. Validate a full payload codec before protocol commit, or submit delta base/target/tick plus raw delta bytes to RunenNet.
3. Let RunenNet `ClientReplicationSet` classify cursor/tick/base/recovery state.
4. For a delta, reconstruct from the exact RunenNet-retained declared complete base, then re-encode one complete product.
5. Account the exact complete-product byte length and atomically activate that product through the Runenwerk host-commit owner.
6. Realize the committed complete product through the retained `SnapshotApplyDriver::apply_snapshot` escape hatch.
7. Stage an ACK only from the RunenNet lineage acknowledgement cursor/tick after successful downstream realization.
8. Replay retained local prediction after realization.

Runenwerk no longer owns a client snapshot-history/cursor state machine. Prediction semantics remain a later RN8 cut and must consume the live RunenNet client replication authority rather than recreating it.

## Streaming Integration

`NetStreamingStateResource` is keyed by `ConnectionHandle` and synchronized from the RunenNet session projection.

When RunenNet lifecycle behavior removes a projected binding, the normal fixed-update streaming synchronization removes that connection's retained streaming state. No replication-specific connection-close authority is required.

## Failure / Recovery Rules

- Missing or evicted server baseline forces a full snapshot for that connection.
- Invalid/future/stale ACKs never mutate the accepted baseline.
- Missing bases, malformed/reconstruction failures, tick regression, connection replacement, and resource pressure are classified by RunenNet client replication state; rejected targets do not replace the active complete product.
- Connection loss is decided by RunenNet; retained replication state is reconciled from the resulting engine projection.
- Host reconnect scheduling remains Runenwerk policy and is distinct from RunenNet session retention.

## Scope

This pipeline remains migration evidence until later RN8 replication/prediction cuts. It must not acquire replacement session, protocol-negotiation, connection identity, or transport-runtime semantics.
