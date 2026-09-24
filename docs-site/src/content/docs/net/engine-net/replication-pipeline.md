---
title: "Replication Pipeline"
description: "Current retained replication pipeline during the RunenNet cutover."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-09-01
---

# Replication Pipeline

This document describes the retained Runenwerk replication path after RN8 N2.

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

1. Validate cursor/tick progression.
2. For a delta, validate the declared base against retained client state.
3. Decode and apply through `SnapshotApplyDriver`.
4. Update the local retained snapshot/baseline state.
5. Stage an ACK for the applied cursor.
6. Reconcile retained prediction state through the existing engine integration.

N2 does not redesign prediction or replicated-view semantics.

## Streaming Integration

`NetStreamingStateResource` is keyed by `ConnectionHandle` and synchronized from the RunenNet session projection.

When RunenNet lifecycle behavior removes a projected binding, the normal fixed-update streaming synchronization removes that connection's retained streaming state. No replication-specific connection-close authority is required.

## Failure / Recovery Rules

- Missing or evicted server baseline forces a full snapshot for that connection.
- Invalid/future/stale ACKs never mutate the accepted baseline.
- Delta base mismatch or malformed payload does not redefine connection/session state.
- Connection loss is decided by RunenNet; retained replication state is reconciled from the resulting engine projection.
- Host reconnect scheduling remains Runenwerk policy and is distinct from RunenNet session retention.

## Scope

This pipeline remains migration evidence until later RN8 replication/prediction cuts. It must not acquire replacement session, protocol-negotiation, connection identity, or transport-runtime semantics.
