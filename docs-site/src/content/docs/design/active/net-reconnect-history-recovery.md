---
title: "Net Reconnect and History Recovery Design"
description: "Design for reconnect, replay history, checkpoints, validation, and recovery boundaries."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-05-05
related_roadmaps:
  - ../../net/multiplayer-replication-implementation-roadmap.md
---

# Net Reconnect and History Recovery Design

## Purpose

This design defines how reconnect and recovery use session state,
replication baselines, streaming history, and replay/checkpoint
substrate without changing server authority.

## Implemented Substrate

Implemented now:

- `engine_net` session admission and handoff contracts.
- `engine_net_quic` reconnect/backoff runtime modules.
- Full snapshot fallback for missing replication baselines.
- Engine plugin per-connection replication checkpoints.
- Engine plugin streaming state with pending cursor markers and
  full-resync flags.
- `engine_history` archive, recorder, controller, checkpoint policy, and
  validation report primitives.
- Session diagnostics for reconnect attempts and close/error events.

## Partial Contracts

Partial now:

- Reconnect recovery is primarily full-resync based.
- Generic ECS checkpoint hooks are not yet standardized.
- `engine_history` is a substrate, not yet wired as the default reconnect
  recovery path.
- Selective history by component/resource type remains future work.
- Rollback application boundaries remain partial.

## Ownership Rules

`engine_net` owns:

- session and protocol contracts;
- snapshot/delta/ACK vocabulary;
- reconnect-neutral authority semantics.

`engine_net_quic` owns:

- reconnect attempts;
- backoff timing;
- QUIC endpoint/session behavior.

`engine_history` owns:

- replay archives;
- checkpoints;
- journal frames;
- validation reports.

Engine/gameplay integration owns:

- when to restore a world checkpoint;
- which gameplay state is recoverable;
- how to present reconnect/correction to users.

## Invariants

- Reconnect must not make clients authoritative over replicated state.
- Unknown or unusable baselines recover through full resync or checkpoint
  restore, not silent partial state.
- Replay/history validation reports explain divergence; they do not
  mutate gameplay state by themselves.
- Transport reconnect policy must not own gameplay correction semantics.

## Future Work

Future work:

1. Define generic ECS checkpoint capture/restore hooks.
2. Connect replay checkpoints to reconnect admission where useful.
3. Add per-connection recovery diagnostics for baseline, history, and
   checkpoint decisions.
4. Add validation reports that include stream cursors, queue state, and
   ownership routing.
5. Add tests for reconnect after pruned baseline and after history gap.

## Validation Plan

Required validation:

- `engine_history` replay/archive tests;
- engine plugin reconnect-state tests;
- QUIC reconnect integration tests;
- recovery docs validation.
