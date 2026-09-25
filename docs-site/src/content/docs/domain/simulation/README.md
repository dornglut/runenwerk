---
title: engine_sim
description: Runenwerk-local simulation identity, timing, determinism, and snapshot-codec vocabulary.
status: active
owner: simulation
layer: domain
canonical: true
last_reviewed: 2026-09-25
publication: primary
---

# engine_sim

`engine_sim` is the Runenwerk-local simulation domain crate at `domain/simulation`.

It owns reusable simulation vocabulary used by Engine, World, networking integration, and replay. It
does not own App lifecycle, fixed-step cadence execution, networking protocol/session semantics, or
replay policy.

## Purpose

`engine_sim` provides:

- `SimulationTick`;
- `SimulationSessionId` as a passive simulation-session value;
- `SimulationSeed`, `SimulationHash`, and deterministic `SimulationRng`;
- `SimulationProfile`, `AuthorityRole`, `DeterminismLevel`, and
  `SimulationProfileConfig`;
- `SimulationCodec` for typed simulation snapshot capture/restore/hash behavior.

Session allocation is not hidden in the value type. `SimulationPlugin` owns App-local initial
session identity installation and preserves an explicitly supplied `SimulationSessionId`.

## Ownership Boundaries

In scope:

- simulation tick/session/hash identity vocabulary;
- simulation profile and authority vocabulary;
- deterministic simulation RNG helpers;
- generic typed simulation snapshot codec behavior.

Out of scope:

- process-global identity allocation;
- RunenNet connection/session/replication semantics;
- Engine scheduling and fixed-cadence policy;
- replay archive/controller policy;
- game-specific commands or entity identity.

## Validation

Run:

```text
cargo test -p engine_sim
cargo check --workspace
```
