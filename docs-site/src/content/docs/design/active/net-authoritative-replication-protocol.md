---
title: "Net Authoritative Replication Protocol Design"
description: "Long-term design for authoritative snapshots, deltas, ACKs, baselines, and resync behavior in Runenwerk networking."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Authoritative Replication Protocol Design

## Purpose

This design defines the transport-agnostic replication protocol contract
for authoritative multiplayer state. It separates what `engine_net`
defines from what engine plugins, transport adapters, and gameplay code
execute.

## Scope

In scope:

- authoritative full snapshots;
- delta snapshots based on acknowledged baselines;
- snapshot cursors and simulation ticks;
- client ACKs;
- per-connection server baselines;
- full-resync fallback when a baseline cannot be used;
- client rejection and resync rules.

Out of scope:

- gameplay-specific snapshot contents;
- ECS mutation details;
- smoothing and presentation correction;
- concrete QUIC delivery mechanics.

## Architectural Position

`engine_net` owns the protocol vocabulary:

- `Snapshot`
- `DeltaSnapshot`
- `SnapshotPayload`
- `DeltaSnapshotPayload`
- `Ack`
- `SnapshotCursor`
- runtime client/server contract helpers

The engine net plugin bridges those contracts into ECS resources and
systems. Gameplay/app modules own payload extraction and application.
Transport adapters only move protocol envelopes.

## Implemented Substrate

Implemented now:

- `engine_net` protocol structs for snapshots, deltas, and ACKs.
- `SnapshotTimeline` cursor allocation, full snapshot storage, delta
  construction, baseline pruning, and merge helpers.
- `AuthoritativeServerRuntime` full/delta selection and per-connection
  full-resync fallback when a baseline is missing.
- `ClientReplicationRuntime` cursor/tick validation, strict delta-base
  validation, decode rejection, resync requests, and authoritative
  operation plans.
- Engine plugin replication state with per-connection checkpoints,
  snapshot histories, pending ACK state, and per-connection delivery.
- Docs and tests for duplicate cursors, out-of-order deltas, missing and
  pruned baselines, stale snapshots, failed delta decode, interest policy
  changes, and fallback diagnostics.

## Partial Contracts

Partial now:

- `AuthoritativeServerRuntime` does not yet track last-sent cursors, so
  future or unknown ACK rejection is a required hardening item.
- Engine plugin checkpoint state has `last_sent_cursor` and full snapshot
  cursor fields, but the lower-level server runtime contract is narrower.
- Client operation plans are generic protocol actions; concrete ECS apply
  is still driver/app work.
- Snapshot payloads are structural and byte-oriented. They do not yet
  encode a versioned schema contract per replicated component.

## Future Work

Future protocol work:

1. Accept ACKs only for sent and retained cursors.
2. Track ACK rejection reasons in diagnostics.
3. Define same-delta entity lifecycle conflicts, especially
   spawn/despawn for the same `NetEntityId`.
4. Add explicit full-resync request/response envelope if boolean resync
   flags become too weak.
5. Add schema/version identity to replicated component payloads.
6. Add end-to-end transport integration tests for snapshot loss, reorder,
   reconnect, and ACK replay.

## Invariants

- Server simulation is authoritative for replicated state.
- Clients send input/intent and ACKs, not authoritative replicated state.
- Snapshot cursors are monotonically increasing on each authoritative
  timeline.
- A client delta must advance from the client's current cursor.
- Missing, mismatched, malformed, or pruned delta baselines recover with
  a full resync.
- Full-resync fallback is per connection, not global.
- Transport lanes do not decide replication policy.

## Failure Modes

Expected failures:

- missing server baseline: send a full snapshot for that connection;
- missing client baseline: reject delta and request full resync;
- stale tick or duplicate cursor: reject without mutating local baseline;
- malformed full snapshot: reject and report decode failure;
- malformed delta: reject and request full resync;
- unknown ACK: do not advance server baseline, record diagnostic, and
  force full resync if needed.

## Validation Plan

Required validation:

- `cargo test -p engine_net -p engine_sim`
- engine plugin replication tests for sent-cursor ACK validation;
- QUIC integration tests for dropped/reordered snapshot and ACK traffic;
- docs validation with `python3 tools/docs/validate_docs.py`.
