---
title: engine_replay
description: Runenwerk-local replay, checkpoint, archive, controller, and validation domain substrate.
status: active
owner: replay
layer: domain
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# engine_replay

`engine_replay` is the Runenwerk-local replay/history domain crate at `domain/replay`.

It records replay headers, checkpoints and per-tick journal frames, builds compressed archives,
selects checkpoints/frames for playback, and reports deterministic validation mismatches.

## Purpose

- `ReplayHeader`, preserving replay-format v1 provenance;
- `ReplayCheckpoint` and `ReplayJournalFrame`;
- `ReplayRecorder` and `ReplayController`;
- checkpoint and storage policy;
- compressed archive encode/decode;
- replay validation reports.

The crate depends on `engine_sim` for simulation identity, tick, profile, seed, and hash
vocabulary. Engine/Scene plugins remain the integration owners that capture and realize concrete
Scene state.

## Identity and format boundary

Replay does not allocate simulation-session identity. Passive `ReplaySessionInfo` represents
absence explicitly as `None`; recording projects the active simulation identity as `Some(id)`.
There is no sentinel or hidden fallback session identity.

`ReplayHeader::FORMAT_VERSION` remains 1. This ownership move does not change serialized field
meaning or layout.

## Non-ownership

`engine_replay` does not own RunenNet connection/session lifecycle, multiplayer recovery,
transport, ECS scheduling, Scene simulation execution, or game-specific replay policy.
