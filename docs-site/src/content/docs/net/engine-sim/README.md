---
title: "engine_sim"
description: "Documentation for engine_sim."
status: active
owner: net
layer: net
canonical: true
last_reviewed: 2026-04-27
---

# engine_sim

`engine_sim` defines simulation-wide core types and codec contracts shared by engine/runtime/network/history crates.

## Purpose

- Provide canonical simulation identity/timing types
- Define profile/authority/determinism vocabulary
- Provide deterministic RNG utilities
- Define command frame structures
- Define snapshot codec traits for host and ECS world usage

## Module Layout

- `src/profile.rs`
  - `SimulationProfile`, `AuthorityRole`, `DeterminismLevel`, `SimulationProfileConfig`
- `src/identity.rs`
  - `SimulationTick`, `SimulationSessionId`, `SimulationSeed`, `SimulationHash`, `ActorId`
- `src/rng.rs`
  - `SimulationRng`
- `src/command.rs`
  - `CommandSource`, `SimulationCommandFrame<C>`
- `src/codec.rs`
  - `SimulationCodec`, `WorldSimulationCodec`

## Current Role

This crate is the shared simulation vocabulary used by:

- `engine`
- `engine_net`
- `engine_history`

It is intentionally transport-agnostic and runtime-agnostic.
