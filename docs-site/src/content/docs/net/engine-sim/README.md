---
title: engine_sim
description: Current documentation for the engine_sim crate.
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-28
---

# engine_sim

`engine_sim` defines simulation-wide core types and codec contracts shared by engine/runtime/network/history crates.

It is transport-agnostic and runtime-agnostic. It owns shared simulation vocabulary, not a concrete game loop, transport, replication backend, or renderer.

## Purpose

`engine_sim` provides:

- canonical simulation identity and timing types;
- profile, authority, and determinism vocabulary;
- deterministic RNG utilities;
- command frame structures;
- snapshot codec traits for host and ECS world usage.

## Module Layout

- `src/profile.rs`
  - `SimulationProfile`
  - `AuthorityRole`
  - `DeterminismLevel`
  - `SimulationProfileConfig`
- `src/identity.rs`
  - `SimulationTick`
  - `SimulationSessionId`
  - `SimulationSeed`
  - `SimulationHash`
  - `ActorId`
- `src/rng.rs`
  - `SimulationRng`
- `src/command.rs`
  - `CommandSource`
  - `SimulationCommandFrame<C>`
- `src/codec.rs`
  - `SimulationCodec`
  - `WorldSimulationCodec`

## Current Role

This crate is the shared simulation vocabulary used by:

- `engine`
- `engine_net`
- `engine_history`

`engine_sim` should stay below networking runtime and replay runtime policy. Higher layers may use these types to identify ticks, sessions, actors, hashes, and command frames, but they should not push concrete transport or replay behavior back into this crate.

## Ownership Boundaries

In scope:

- simulation tick/session/hash identity vocabulary;
- simulation profile and authority vocabulary;
- deterministic simulation RNG helpers;
- generic command-frame containers;
- codec traits for simulation snapshots.

Out of scope:

- QUIC transport;
- replication session runtime;
- replay archive storage;
- engine plugin scheduling;
- ECS world mutation policy;
- game-specific commands.

## Validation

Run:

```text
cargo test -p engine_sim
cargo check --workspace
```
