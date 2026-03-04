# engine_sim

`engine_sim` contains shared simulation contracts that are independent of any specific transport or engine feature plugin.

## Purpose

- Define canonical simulation identity and timing types
- Define profile/authority/determinism vocabulary
- Define replay- and transport-friendly command/frame types
- Define explicit simulation codec traits

## Core Types

- `SimulationTick`
- `SimulationProfile`
- `AuthorityRole`
- `DeterminismLevel`
- `SimulationSessionId`
- `SimulationSeed`
- `SimulationRng`
- `SimulationCommandFrame<C>`
- `SimulationCodec`
- `WorldSimulationCodec`

## Current Role

This crate is the shared vocabulary layer used by:

- `engine`
- `engine_net`
- `engine_replay`

The current production path built on it is `SimulationProfile::DedicatedAuthority`.
